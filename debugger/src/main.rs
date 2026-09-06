//! Frontend for the emulator.
//!
//! This thread owns the window and nothing else. It never emulates, never
//! blocks on the emulator, and never waits for a frame - it presents whatever
//! the emulator last published and goes straight back to pumping window events,
//! so the window stays responsive regardless of what the emulator is doing.

use std::{sync::mpsc::Receiver, thread, time::Duration};

use chlorosis_core::{
    channels, framebuffer::blank_frame, CoreMessage, Device, EmulatorState, Event, Frame,
    FrameConsumer, KeyCode, SCREEN_HEIGHT, SCREEN_WIDTH, TICKS_PER_FRAME,
};
use minifb::{Key, Menu, Window, WindowOptions, MENU_KEY_CTRL};

const MENU_OPEN_ROM: usize = 1;
const MENU_RESET: usize = 2;

fn main() {
    let mut window = build_window();
    let (core_channels, frontend) = channels();

    let core = thread::Builder::new()
        .name("core".to_owned())
        .spawn(move || Device::default().run(core_channels))
        .expect("failed to spawn emulation thread");

    let mut ui = Ui::new();
    while window.is_open() && !ui.quitting {
        ui.pump_messages(&frontend.messages);
        ui.present(&mut window, &frontend.frames);
        ui.handle_input(&mut window, &frontend.events);
        ui.refresh_title(&mut window);
    }

    // Ask the emulator to wind down, then wait for it. Dropping the sender
    // would also wake it, but an explicit request means a core blocked while
    // paused stops for a reason it can name.
    let _ = frontend.events.send(Event::Exit);
    if core.join().is_err() {
        eprintln!("Emulation thread panicked");
    }
}

/// Everything the frontend knows. Emulator state is a cached copy of what the
/// core reported, never something this side decides for itself.
struct Ui {
    frame: Frame,
    state: EmulatorState,
    title: String,
    shown_title: String,
    cartridge: Option<String>,
    speed: Option<f32>,
    fault: Option<String>,
    quitting: bool,
}

impl Ui {
    fn new() -> Self {
        Self {
            frame: blank_frame(),
            state: EmulatorState::Stopped,
            title: String::new(),
            shown_title: String::new(),
            cartridge: None,
            speed: None,
            fault: None,
            quitting: false,
        }
    }

    /// Drain everything the core has said since the last redraw.
    fn pump_messages(&mut self, messages: &Receiver<CoreMessage>) {
        while let Ok(message) = messages.try_recv() {
            match message {
                CoreMessage::State(state) => self.state = state,
                CoreMessage::CartridgeLoaded(title) => {
                    self.cartridge = Some(title);
                    self.fault = None;
                }
                CoreMessage::Error(e) => eprintln!("{e}"),
                CoreMessage::Faulted(e) => {
                    eprintln!("Emulation stopped: {e}");
                    self.fault = Some(e);
                }
                CoreMessage::Speed { fps, percent } => {
                    self.speed = Some(percent);
                    println!("{fps:.1} fps ({percent:.0}% speed)");
                }
            }
        }
    }

    fn present(&mut self, window: &mut Window, frames: &FrameConsumer) {
        if let Some(latest) = frames.take() {
            // Hand the buffer we were showing back to be drawn into again.
            frames.recycle(std::mem::replace(&mut self.frame, latest));
        }

        // Always redraw the frame we have. Blanking the screen whenever the
        // emulator has not finished a new frame is what makes the picture
        // flicker, and the emulator is under no obligation to keep pace with
        // the compositor.
        window
            .update_with_buffer(&self.frame[..], SCREEN_WIDTH, SCREEN_HEIGHT)
            .expect("failed to present frame");
    }

    fn handle_input(&mut self, window: &mut Window, events: &std::sync::mpsc::Sender<Event>) {
        if window.is_key_down(Key::Escape) {
            self.quitting = true;
            return;
        }

        if let Some(item) = window.is_menu_pressed() {
            self.handle_menu(item, events);
        }

        let pressed = window.get_keys_pressed(minifb::KeyRepeat::No);
        let released = window.get_keys_released();

        if released.contains(&Key::Space) {
            // Ask for the transition and let the core confirm it. Flipping our
            // own copy here is how the two sides drift apart.
            let _ = events.send(match self.state {
                EmulatorState::Running => Event::Pause,
                EmulatorState::Paused | EmulatorState::Stopped => Event::Run,
            });
        }

        if released.contains(&Key::Period) {
            let _ = events.send(Event::Step(TICKS_PER_FRAME));
        }

        // Press and release go out as they happen rather than being batched
        // into one poll, so the emulator sees the same edges the player made.
        let down: Vec<KeyCode> = pressed.iter().filter_map(key_to_keycode).collect();
        if !down.is_empty() {
            let _ = events.send(Event::KeyDown(down));
        }

        let up: Vec<KeyCode> = released.iter().filter_map(key_to_keycode).collect();
        if !up.is_empty() {
            let _ = events.send(Event::KeyUp(up));
        }
    }

    fn handle_menu(&mut self, item: usize, events: &std::sync::mpsc::Sender<Event>) {
        match item {
            MENU_OPEN_ROM => {
                // This blocks the frontend for as long as the dialog is up. The
                // emulator keeps running behind it, which is the point of the
                // split.
                let file = native_dialog::FileDialog::new()
                    .add_filter("GBC ROM", &["gbc", "gb"])
                    .show_open_single_file();

                match file {
                    Ok(Some(f)) => {
                        let _ = events.send(Event::LoadFile(f));
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("Could not open file dialog: {e}"),
                }
            }
            MENU_RESET => {
                let _ = events.send(Event::Reset);
            }
            _ => eprintln!("Unhandled menu item {item}"),
        }
    }

    fn refresh_title(&mut self, window: &mut Window) {
        self.title.clear();
        self.title.push_str("Chlorosis - Debugger");

        if let Some(cartridge) = &self.cartridge {
            self.title.push_str(" - ");
            self.title.push_str(cartridge);
        }

        self.title.push_str(match self.state {
            EmulatorState::Stopped => " [no cartrige]",
            EmulatorState::Running => " [running]",
            EmulatorState::Paused => " [paused]",
        });

        if let Some(fault) = &self.fault {
            self.title.push_str(" [faulted: ");
            self.title.push_str(fault);
            self.title.push(']');
        } else if let Some(percent) = self.speed {
            self.title
                .push_str(&format!(" - {percent:.0}% of real hardware"));
        }

        // set_title talks to the window system, so only pay for it on a change.
        if self.title != self.shown_title {
            window.set_title(&self.title);
            self.shown_title.clone_from(&self.title);
        }
    }
}

fn build_window() -> Window {
    let mut window = Window::new(
        "Chlorosis - Debugger",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            borderless: false,
            title: true,
            resize: true,
            scale: minifb::Scale::X4,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            topmost: false,
            transparency: false,
            none: false,
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Paces this thread only. The emulator keeps its own clock.
    window.limit_update_rate(Some(Duration::from_millis(16)));

    let mut menu = Menu::new("File").unwrap();
    menu.add_item("Open ROM", MENU_OPEN_ROM)
        .shortcut(Key::O, MENU_KEY_CTRL)
        .build();
    menu.add_item("Reset", MENU_RESET).build();
    window.add_menu(&menu);

    window
}

const fn key_to_keycode(k: &Key) -> Option<KeyCode> {
    match k {
        Key::W => Some(KeyCode::Up),
        Key::S => Some(KeyCode::Down),
        Key::A => Some(KeyCode::Left),
        Key::D => Some(KeyCode::Right),
        Key::O => Some(KeyCode::A),
        Key::P => Some(KeyCode::B),
        Key::Enter => Some(KeyCode::Start),
        Key::RightShift => Some(KeyCode::Select),

        _ => None,
    }
}

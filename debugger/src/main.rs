//! Frontend for the emulator.
//!
//! This thread owns the window and nothing else. It never emulates, never
//! blocks on the emulator, and never waits for a frame - it presents whatever
//! the emulator last published and goes straight back to pumping window events,
//! so the window stays responsive regardless of what the emulator is doing.

use std::{sync::mpsc::Receiver, thread};

use chlorosis_core::{
    channels, framebuffer::blank_frame, CoreMessage, Device, EmulatorState, Event, Frame,
    FrameConsumer, KeyCode, SCREEN_HEIGHT, SCREEN_WIDTH, TICKS_PER_FRAME,
};
use minifb::{Key, Menu, Window, WindowOptions};

const MENU_OPEN_ROM: usize = 1;
const MENU_RESET: usize = 2;

/// Modifier for menu shortcuts: Command on macOS (the platform convention),
/// Control elsewhere.
#[cfg(target_os = "macos")]
const MENU_MODIFIER: usize = minifb::MENU_KEY_COMMAND;
#[cfg(not(target_os = "macos"))]
const MENU_MODIFIER: usize = minifb::MENU_KEY_CTRL;

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
    keymap: Keymap,
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
            keymap: Keymap::load(),
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
            self.handle_menu(item, window, events);
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
        let down: Vec<KeyCode> = pressed.iter().filter_map(|k| self.keymap.button(k)).collect();
        if !down.is_empty() {
            let _ = events.send(Event::KeyDown(down));
        }

        let up: Vec<KeyCode> = released.iter().filter_map(|k| self.keymap.button(k)).collect();
        if !up.is_empty() {
            let _ = events.send(Event::KeyUp(up));
        }
    }

    fn handle_menu(
        &mut self,
        item: usize,
        window: &Window,
        events: &std::sync::mpsc::Sender<Event>,
    ) {
        match item {
            MENU_OPEN_ROM => {
                // This blocks the frontend for as long as the dialog is up. The
                // emulator keeps running behind it, which is the point of the
                // split. The dialog is parented to the window so it comes to the
                // front rather than opening behind it - which, unbundled on
                // macOS, otherwise just looks like the window losing focus.
                let file = native_dialog::DialogBuilder::file()
                    .add_filter("GBC ROM", ["gbc", "gb"])
                    .set_owner(window)
                    .open_single_file()
                    .show();

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

        // A window title cannot contain NUL or other control bytes - minifb
        // builds a CString from it and panics otherwise - and cartridge titles
        // are attacker-controlled bytes, so strip anything unprintable.
        self.title.retain(|c| !c.is_control());

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

    // Paces this thread only (~60 fps). The emulator keeps its own clock.
    window.set_target_fps(60);

    let mut menu = Menu::new("File").unwrap();
    menu.add_item("Open ROM", MENU_OPEN_ROM)
        .shortcut(Key::O, MENU_MODIFIER)
        .build();
    menu.add_item("Reset", MENU_RESET).build();
    window.add_menu(&menu);

    window
}

/// Maps host keys to Game Boy buttons. Several keys may map to one button
/// (arrows and WASD both drive the D-pad by default), and the bindings can be
/// overridden by a config file - see [`Keymap::load`].
struct Keymap {
    bindings: Vec<(Key, KeyCode)>,
}

impl Keymap {
    fn default_bindings() -> Vec<(Key, KeyCode)> {
        use KeyCode::{Down, Left, Right, Select, Start, Up, A, B};
        vec![
            (Key::Up, Up),
            (Key::W, Up),
            (Key::Down, Down),
            (Key::S, Down),
            (Key::Left, Left),
            (Key::A, Left),
            (Key::Right, Right),
            (Key::D, Right),
            (Key::X, A),
            (Key::Z, B),
            (Key::Enter, Start),
            (Key::Backspace, Select),
            (Key::RightShift, Select),
        ]
    }

    /// Load bindings, overriding the defaults from a config file when one is
    /// present. The path comes from `CHLOROSIS_KEYMAP`, else `keymap.conf` in
    /// the working directory. Each line is `button = key[, key ...]`; buttons
    /// are up/down/left/right/a/b/start/select and keys are names like `x`,
    /// `left`, `space`, `rshift`. A `#` starts a comment. A file with any valid
    /// binding replaces the defaults entirely, so it fully describes the layout.
    fn load() -> Self {
        let path = std::env::var_os("CHLOROSIS_KEYMAP").map_or_else(
            || std::path::PathBuf::from("keymap.conf"),
            std::path::PathBuf::from,
        );
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self {
                bindings: Self::default_bindings(),
            };
        };

        let mut bindings = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((button, keys)) = line.split_once('=') else {
                eprintln!("keymap.conf line {}: expected `button = key`", n + 1);
                continue;
            };
            let Some(code) = parse_button(button.trim()) else {
                eprintln!("keymap.conf line {}: unknown button `{}`", n + 1, button.trim());
                continue;
            };
            for name in keys.split(',') {
                let name = name.trim();
                if let Some(key) = parse_key(name) {
                    bindings.push((key, code));
                } else {
                    eprintln!("keymap.conf line {}: unknown key `{name}`", n + 1);
                }
            }
        }

        if bindings.is_empty() {
            eprintln!("keymap.conf had no valid bindings; using defaults");
            bindings = Self::default_bindings();
        }
        Self { bindings }
    }

    fn button(&self, key: &Key) -> Option<KeyCode> {
        self.bindings
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, code)| *code)
    }
}

fn parse_button(name: &str) -> Option<KeyCode> {
    match name.to_ascii_lowercase().as_str() {
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "a" => Some(KeyCode::A),
        "b" => Some(KeyCode::B),
        "start" => Some(KeyCode::Start),
        "select" => Some(KeyCode::Select),
        _ => None,
    }
}

/// Resolve a key name from the config file to a minifb key. Single letters and
/// digits map directly; a handful of named keys cover the rest.
fn parse_key(name: &str) -> Option<Key> {
    let lower = name.to_ascii_lowercase();
    if let [c] = lower.as_bytes() {
        return match c {
            b'a'..=b'z' => Some(LETTERS[(c - b'a') as usize]),
            b'0'..=b'9' => Some(DIGITS[(c - b'0') as usize]),
            _ => None,
        };
    }
    match lower.as_str() {
        "up" => Some(Key::Up),
        "down" => Some(Key::Down),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "enter" | "return" => Some(Key::Enter),
        "space" => Some(Key::Space),
        "backspace" => Some(Key::Backspace),
        "tab" => Some(Key::Tab),
        "lshift" | "leftshift" => Some(Key::LeftShift),
        "rshift" | "rightshift" => Some(Key::RightShift),
        "lctrl" | "leftctrl" => Some(Key::LeftCtrl),
        "rctrl" | "rightctrl" => Some(Key::RightCtrl),
        "comma" => Some(Key::Comma),
        "period" => Some(Key::Period),
        "slash" => Some(Key::Slash),
        "semicolon" => Some(Key::Semicolon),
        "apostrophe" => Some(Key::Apostrophe),
        _ => None,
    }
}

#[rustfmt::skip]
const LETTERS: [Key; 26] = [
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K, Key::L,
    Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X,
    Key::Y, Key::Z,
];
#[rustfmt::skip]
const DIGITS: [Key; 10] = [
    Key::Key0, Key::Key1, Key::Key2, Key::Key3, Key::Key4,
    Key::Key5, Key::Key6, Key::Key7, Key::Key8, Key::Key9,
];

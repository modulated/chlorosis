use std::{sync::mpsc::Sender, time::Duration};

use chlorosis_core::{Device, Event, KeyCode};
use minifb::{Key, Menu, Window, WindowOptions, MENU_KEY_CTRL};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let mut dev = Device::default();

    let mut window = build_window();

    let (buffer_sender, buffer_receiver) = std::sync::mpsc::channel();
    let (event_sender, event_receiver) = std::sync::mpsc::channel();

    const BLACK: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    std::thread::Builder::new()
        .name("Core".to_owned())
        .spawn(move || dev.run(buffer_sender, event_receiver))
        .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Ok(b) = buffer_receiver.try_recv() {
            window.update_with_buffer(&b, WIDTH, HEIGHT).unwrap();
        } else {
            window.update_with_buffer(&BLACK, WIDTH, HEIGHT).unwrap();
        }

        let keys_down: Vec<KeyCode> = window
            .get_keys_pressed(minifb::KeyRepeat::No)
            .iter()
            .filter_map(key_to_keycode)
            .collect();
        if !keys_down.is_empty() {
            event_sender.send(Event::KeyDown(keys_down)).unwrap();
        }

        let keys_up: Vec<KeyCode> = window
            .get_keys_released()
            .iter()
            .filter_map(key_to_keycode)
            .collect();
        if !keys_up.is_empty() {
            event_sender.send(Event::KeyUp(keys_up)).unwrap();
        }

        if let Some(n) = window.is_menu_pressed() {
            handle_menu(n, &event_sender)
        }
    }

    event_sender.send(Event::Exit).unwrap();
}

fn build_window() -> Window {
    let mut window = Window::new(
        "Chlorosis - Debugger",
        WIDTH,
        HEIGHT,
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

    window.limit_update_rate(Some(Duration::from_millis(16)));

    let mut menu = Menu::new("File").unwrap();
    menu.add_item("Open ROM", 1)
        .shortcut(Key::O, MENU_KEY_CTRL)
        .build();
    menu.add_item("Reset", 2).build();
    window.add_menu(&menu);

    window
}

fn key_to_keycode(k: &Key) -> Option<KeyCode> {
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

fn handle_menu(menu: usize, sender: &Sender<Event>) {
    match menu {
        1 => {
            let f = native_dialog::FileDialog::new()
                .add_filter("GBC ROM", &["gbc"])
                .show_open_single_file()
                .unwrap();
            if let Some(f) = f {
                sender.send(Event::LoadFile(f)).unwrap();
            }
        }
        2 => {
            sender.send(Event::Reset).unwrap();
        }
        _ => println!("Unhandled menu {menu}"),
    }
}

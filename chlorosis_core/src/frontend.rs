use std::path::PathBuf;

pub trait Frontend {
    fn draw(&self, buffer: &[u32]);
    fn get_input(&self) -> Event;
    // audio
}

#[derive(Debug)]
pub enum Event {
    KeyDown(Vec<KeyCode>),
    KeyUp(Vec<KeyCode>),
    LoadFile(PathBuf),
    SaveState(PathBuf),
    LoadState(PathBuf),
    Run,
    Pause,
    Reset,
    Exit,
}

#[derive(Debug, Copy, Clone)]
pub enum KeyCode {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

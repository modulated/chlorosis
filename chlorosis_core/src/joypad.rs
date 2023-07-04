use crate::{types::Byte, KeyCode};

#[derive(Debug, Default)]
pub struct Joypad {
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    actions: bool,
    directions: bool,
}

impl Joypad {
    pub fn press(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => self.up = true,
            KeyCode::Down => self.down = true,
            KeyCode::Left => self.left = true,
            KeyCode::Right => self.right = true,
            KeyCode::A => self.a = true,
            KeyCode::B => self.b = true,
            KeyCode::Start => self.start = true,
            KeyCode::Select => self.select = true,
        }
    }

    pub fn release(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => self.up = false,
            KeyCode::Down => self.down = false,
            KeyCode::Left => self.left = false,
            KeyCode::Right => self.right = false,
            KeyCode::A => self.a = false,
            KeyCode::B => self.b = false,
            KeyCode::Start => self.start = false,
            KeyCode::Select => self.select = false,
        }
    }

    pub fn read(&self) -> Byte {
        let mut out = Byte(0);
        match (self.actions, self.directions) {
            (true, false) => {
                out.write_bit(0, self.a);
                out.write_bit(1, self.b);
                out.write_bit(2, self.select);
                out.write_bit(3, self.start);
            }
            (false, true) => {
                out.write_bit(0, self.right);
                out.write_bit(1, self.left);
                out.write_bit(2, self.up);
                out.write_bit(3, self.down);
            }
            _ => {}
        }
        out
    }

    pub fn write(&mut self, value: Byte) {
        self.directions = value.is_bit_set(4);
        self.actions = value.is_bit_set(5);
    }
}

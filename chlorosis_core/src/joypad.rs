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
    /// Whether the action-button group is selected (P15 driven low).
    select_actions: bool,
    /// Whether the direction group is selected (P14 driven low).
    select_directions: bool,
}

impl Joypad {
    pub const fn press(&mut self, key: KeyCode) {
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

    pub const fn release(&mut self, key: KeyCode) {
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

    pub const fn read(&self) -> Byte {
        // Every line is active-low: a bit reads 0 when pressed or selected, 1
        // otherwise. The old code had this inverted (pressed = 1, and it treated
        // a set select bit as "selected"), so games read every button as held.
        // Bits 6-7 are unused and read as 1; start from "nothing pressed,
        // neither group selected" (all ones) and clear bits from there.
        let mut out = 0xFF;

        if self.select_directions {
            out &= !(1 << 4);
            clear_if(&mut out, 0, self.right);
            clear_if(&mut out, 1, self.left);
            clear_if(&mut out, 2, self.up);
            clear_if(&mut out, 3, self.down);
        }
        if self.select_actions {
            out &= !(1 << 5);
            clear_if(&mut out, 0, self.a);
            clear_if(&mut out, 1, self.b);
            clear_if(&mut out, 2, self.select);
            clear_if(&mut out, 3, self.start);
        }

        Byte(out)
    }

    pub const fn write(&mut self, value: Byte) {
        // Active-low select lines: a group is selected when its bit is written 0.
        self.select_directions = !value.is_bit_set(4);
        self.select_actions = !value.is_bit_set(5);
    }
}

/// Clear bit `n` of `out` when a line is active (pressed), for active-low reads.
const fn clear_if(out: &mut u8, n: u8, pressed: bool) {
    if pressed {
        *out &= !(1 << n);
    }
}

#[cfg(test)]
mod tests {
    use super::Joypad;
    use crate::{Byte, KeyCode};

    // Writing bit 4/5 low selects that group.
    const SELECT_DIRECTIONS: Byte = Byte(0b0010_0000); // bit4 low, bit5 high
    const SELECT_ACTIONS: Byte = Byte(0b0001_0000); // bit5 low, bit4 high

    #[test]
    fn nothing_is_held_when_no_group_is_selected() {
        let mut pad = Joypad::default();
        pad.press(KeyCode::A);
        pad.press(KeyCode::Right);
        // With neither group selected the lower nibble reads all ones (idle).
        assert_eq!(pad.read().0 & 0x0F, 0x0F);
    }

    #[test]
    fn pressed_direction_reads_low_only_when_directions_selected() {
        let mut pad = Joypad::default();
        pad.press(KeyCode::Right); // direction, bit 0
        pad.press(KeyCode::A); // action, bit 0

        pad.write(SELECT_DIRECTIONS);
        // Right pressed -> bit 0 low; the action press must not leak in.
        assert_eq!(pad.read().0 & 0x0F, 0b1110);
        assert_eq!(pad.read().0 & (1 << 4), 0, "P14 reads low while selected");

        pad.write(SELECT_ACTIONS);
        // Now only the action group is visible: A -> bit 0 low.
        assert_eq!(pad.read().0 & 0x0F, 0b1110);
        assert_eq!(pad.read().0 & (1 << 5), 0, "P15 reads low while selected");
    }

    #[test]
    fn release_restores_the_high_bit() {
        let mut pad = Joypad::default();
        pad.write(SELECT_ACTIONS);
        pad.press(KeyCode::Start); // bit 3
        assert_eq!(pad.read().0 & (1 << 3), 0);
        pad.release(KeyCode::Start);
        assert_eq!(pad.read().0 & (1 << 3), 1 << 3);
    }

    #[test]
    fn unused_bits_read_high() {
        let pad = Joypad::default();
        assert_eq!(pad.read().0 & 0b1100_0000, 0b1100_0000);
    }
}

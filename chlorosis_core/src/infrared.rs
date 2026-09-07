use serde::{Deserialize, Serialize};

use crate::types::Byte;

#[derive(Debug, Serialize, Deserialize)]
pub struct Infrared {
    read_enabled: bool,
    reading: bool,
    led_active: bool,
}

impl Default for Infrared {
    fn default() -> Self {
        Self {
            read_enabled: false,
            reading: true,
            led_active: false,
        }
    }
}

impl Infrared {
    pub const fn read(&self) -> Byte {
        let mut value = Byte(0);
        value.write_bit(0, self.led_active);
        value.write_bit(1, self.reading);
        value.write_bit(6, self.read_enabled);
        value.write_bit(7, self.read_enabled);
        value
    }

    pub const fn write(&mut self, value: Byte) {
        self.led_active = value.is_bit_set(0);
        self.read_enabled = value.is_bit_set(6);
        self.read_enabled = value.is_bit_set(7);
    }
}

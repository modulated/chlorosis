use serde::{Deserialize, Serialize};

use crate::types::{Address, Byte};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Timer {
    /// 16-bit internal counter incremented every master tick; DIV is its high
    /// byte, so DIV advances at 16384 Hz rather than the tick rate.
    system_counter: u16,
    /// Ticks accumulated toward the next TIMA increment.
    tima_sub: u32,
    counter: Byte, // TIMA
    modulo: Byte,  // TMA
    enabled: bool,
    clock_speed: ClockSpeed,
}

// TODO: change based on CGB double speed mode

impl Timer {
    /// Advance the timer one master tick. Returns `true` on the tick where TIMA
    /// overflowed, which is when a Timer interrupt must be requested.
    ///
    /// Previously this incremented DIV and TIMA on every call regardless of the
    /// enable bit or selected clock, and reloaded TIMA from DIV instead of the
    /// modulo on overflow - so a running timer fired far too often. TIMA now
    /// advances only when enabled, at its configured rate, and reloads from TMA.
    pub const fn tick(&mut self) -> bool {
        self.system_counter = self.system_counter.wrapping_add(1);

        if !self.enabled {
            return false;
        }

        self.tima_sub += 1;
        if self.tima_sub < self.clock_speed.period() {
            return false;
        }
        self.tima_sub = 0;

        match self.counter.0.checked_add(1) {
            Some(v) => {
                self.counter = Byte(v);
                false
            }
            None => {
                self.counter = self.modulo;
                true
            }
        }
    }

    pub fn read(&self, address: Address) -> Byte {
        match address.0 {
            0xFF04 => self.read_divider(),
            0xFF05 => self.read_counter(),
            0xFF06 => self.read_modulo(),
            0xFF07 => self.read_control(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: Address, value: Byte) {
        match address.0 {
            0xFF04 => self.write_divider(value),
            0xFF05 => self.write_counter(value),
            0xFF06 => self.write_modulo(value),
            0xFF07 => self.write_control(value),
            _ => unreachable!(),
        }
    }

    pub const fn read_divider(&self) -> Byte {
        Byte((self.system_counter >> 8) as u8)
    }

    pub const fn write_divider(&mut self, _: Byte) {
        // Any write resets the whole internal counter, not just the DIV byte.
        self.system_counter = 0;
    }

    pub const fn read_counter(&self) -> Byte {
        self.counter
    }

    pub const fn write_counter(&mut self, value: Byte) {
        self.counter = value;
    }

    pub const fn read_modulo(&self) -> Byte {
        self.modulo
    }

    pub const fn write_modulo(&mut self, value: Byte) {
        self.modulo = value;
    }

    pub const fn read_control(&self) -> Byte {
        let mut out = Byte(0);
        out.write_bit(2, self.enabled);
        match self.clock_speed {
            ClockSpeed::C1024 => {}
            ClockSpeed::C16 => out.write_bit(0, true),
            ClockSpeed::C64 => out.write_bit(1, true),
            ClockSpeed::C256 => {
                out.write_bit(0, true);
                out.write_bit(1, true);
            }
        }
        out
    }

    pub fn write_control(&mut self, value: Byte) {
        self.enabled = value.is_bit_set(2);
        let masked = 0b0000_0011 & value.0;
        self.clock_speed = match masked {
            0b00 => ClockSpeed::C1024,
            0b01 => ClockSpeed::C16,
            0b10 => ClockSpeed::C64,
            0b11 => ClockSpeed::C256,
            _ => unreachable!(),
        };
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
enum ClockSpeed {
    #[default]
    C1024,
    C16,
    C64,
    C256,
}

impl ClockSpeed {
    /// Master ticks between TIMA increments.
    const fn period(&self) -> u32 {
        match self {
            Self::C1024 => 1024,
            Self::C256 => 256,
            Self::C64 => 64,
            Self::C16 => 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;
    use crate::Byte;

    #[test]
    fn tima_overflow_reloads_modulo_and_signals() {
        let mut timer = Timer::default();
        timer.write_control(Byte(0b101)); // enabled, fastest clock (period 16)
        timer.write_modulo(Byte(0xAB));
        timer.write_counter(Byte(0xFF));

        let mut fired = false;
        for _ in 0..16 {
            fired |= timer.tick();
        }

        assert!(fired, "overflow should be reported exactly once");
        assert_eq!(timer.read_counter(), Byte(0xAB), "TIMA reloads from TMA");
    }

    #[test]
    fn disabled_timer_never_fires() {
        let mut timer = Timer::default();
        timer.write_control(Byte(0b001)); // clock selected but disabled
        timer.write_counter(Byte(0xFF));

        assert!(!(0..10_000).any(|_| timer.tick()));
    }

    #[test]
    fn div_advances_far_slower_than_the_tick_rate() {
        let mut timer = Timer::default();
        for _ in 0..255 {
            timer.tick();
        }
        assert_eq!(timer.read_divider(), Byte(0));
        timer.tick(); // 256th tick rolls the low byte into DIV
        assert_eq!(timer.read_divider(), Byte(1));
    }
}

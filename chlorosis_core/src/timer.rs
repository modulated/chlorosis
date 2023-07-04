use crate::types::{Byte, Address};

#[derive(Debug, Default)]
pub struct Timer {
	divider: Byte,
	counter: Byte,
	modulo: Byte,
	enabled: bool,
	clock_speed: ClockSpeed
}

// TODO: change based on CGB double speed mode

impl Timer {
	pub fn tick(&mut self) {
		// This function is called at CPU freq 4.19 MHz
		
		self.divider += 1;

		// incremented at clock_speed 
		// if overflow, set to modulo and request interrupt
		match self.counter.0.checked_add(1) {
			Some(v) => self.counter = Byte(v),
			None => {
				self.counter = self.divider;
				// TODO: call interrupt
			}
		}
	}

	pub fn read(&self, address: Address) -> Byte {
		match address.0 {
			0xFF04 => self.read_divider(),
			0xFF05 => self.read_counter(),
			0xFF06 => self.read_modulo(),
			0xFF07 => self.read_control(),
			_ => unreachable!()
		}
	}

	pub fn write(&mut self, address: Address, value: Byte) {
		match address.0 {
			0xFF04 => self.write_divider(value),
			0xFF05 => self.write_counter(value),
			0xFF06 => self.write_modulo(value),
			0xFF07 => self.write_control(value),
			_ => unreachable!()
		}
	}

	pub const fn read_divider(&self) -> Byte {
		self.divider
	}

	pub fn write_divider(&mut self, _: Byte) {
		self.divider = Byte(0);
	}

	pub const fn read_counter(&self) -> Byte {
		self.counter
	}
	
	pub fn write_counter(&mut self, value: Byte) {
		self.counter = value;
	}

	pub const fn read_modulo(&self) -> Byte {
		self.modulo
	}
	
	pub fn write_modulo(&mut self, value: Byte) {
		self.modulo = value;
	}

	pub fn read_control(&self) -> Byte {
		let mut out = Byte(0);
		out.write_bit(2, self.enabled);
		match self.clock_speed {
			ClockSpeed::C1024 => {},
			ClockSpeed::C16 => out.write_bit(0, true),
			ClockSpeed::C64 => out.write_bit(1, true),
			ClockSpeed::C256 => {
				out.write_bit(0, true);
				out.write_bit(1, true);
			},
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
			_ => unreachable!()
		};
	}
}

#[derive(Debug, Default)]
enum ClockSpeed {
	#[default] C1024,
	C16,
	C64,
	C256
}
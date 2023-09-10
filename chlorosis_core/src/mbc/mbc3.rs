use super::Memory;
use crate::types::{Address, Byte};

#[derive(Debug, Copy, Clone)]
enum RTCMode {
    Ram,
    Seconds,
    Minutes,
    Hours,
    LowerDay,
    UpperDay,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum LatchState {
    Unlatched,
    AttemptingLatch,
    Latched,
    AttemptingUnlatch,
}

pub struct MBC3 {
    rom_data: Vec<u8>,
    rom_bank: usize,
    ram_data: Vec<u8>,
    ram_bank: usize,
    ram_enabled: bool,
    rtc_selected: RTCMode,
    seconds: u8,
    minutes: u8,
    hours: u8,
    lower_day: u8,
    upper_day: u8,
    latch_state: LatchState,
    rtc_start_timestamp: std::time::SystemTime,
}

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

impl MBC3 {
    fn read_ram(&self, addr: Address) -> Byte {
        let offset: usize = (addr.0 - 0xA000) as usize + RAM_BANK_SIZE * self.ram_bank;
        Byte(self.ram_data[offset])
    }

    fn write_ram(&mut self, addr: Address, val: Byte) {
        let offset: usize = (addr.0 - 0xA000) as usize + RAM_BANK_SIZE * self.ram_bank;
        self.ram_data[offset] = val.0;
    }

    fn latch_time(&mut self) {
        let elapsed = self
            .rtc_start_timestamp
            .elapsed()
            .expect("Time drift error");
        self.seconds = (elapsed.as_secs() % 60) as u8;
        self.minutes = ((elapsed.as_secs() / 60) % 60) as u8;
        self.hours = ((elapsed.as_secs() / (60 * 60)) % 24) as u8;
        let days = elapsed.as_secs() / (60 * 60 * 24);
        self.lower_day = days as u8;
        self.upper_day = (days >> 8) as u8 & 0b0000_0001; // TODO : set overflow
    }
}

impl Memory for MBC3 {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            rom_data: bytes,
            rom_bank: 1,
            ram_data: vec![0; RAM_BANK_SIZE * 4],
            ram_bank: 0,
            ram_enabled: false,
            rtc_selected: RTCMode::Ram,
            seconds: 0,
            minutes: 0,
            hours: 0,
            lower_day: 0,
            upper_day: 0,
            latch_state: LatchState::Unlatched,
            rtc_start_timestamp: std::time::SystemTime::now(),
        }
    }

    fn read(&self, addr: Address) -> Byte {
        match addr.0 {
            0x0000..=0x3FFF => Byte(self.rom_data[addr.0 as usize]),
            0x4000..=0x7FFF => Byte(self.rom_data[addr.0 as usize + ROM_BANK_SIZE * self.rom_bank]),
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    match self.rtc_selected {
                        RTCMode::Ram => self.read_ram(addr),
                        RTCMode::Seconds => Byte(self.seconds),
                        RTCMode::Minutes => Byte(self.minutes),
                        RTCMode::Hours => Byte(self.hours),
                        RTCMode::LowerDay => Byte(self.lower_day),
                        RTCMode::UpperDay => Byte(self.upper_day),
                    }
                } else {
                    panic!("Attempted RAM/RTC read when disabled");
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: Address, val: Byte) {
        match addr.0 {
            0x0000..=0x1FFF => self.ram_enabled = val.0 == 0x0A,
            0x2000..=0x3FFF => {
                if val == Byte::ZERO {
                    self.rom_bank = 1;
                } else {
                    self.rom_bank = (0b0111_1111 & val.0) as usize;
                }
            }
            0x4000..=0x5FFF => {
                use RTCMode::*;
                match val.0 {
                    0x00..=0x03 => {
                        self.rtc_selected = Ram;
                        self.ram_bank = val.0 as usize;
                    }
                    0x08 => self.rtc_selected = Seconds,
                    0x09 => self.rtc_selected = Minutes,
                    0x0A => self.rtc_selected = Hours,
                    0x0B => self.rtc_selected = LowerDay,
                    0x0C => self.rtc_selected = UpperDay,
                    _ => unreachable!(),
                }
            }
            0x6000..=0x7FFF => {
                use LatchState::*;
                match (val.0, self.latch_state) {
                    (0x00, Unlatched) => self.latch_state = AttemptingLatch,
                    (0x01, AttemptingLatch) => {
                        self.latch_state = Latched;
                        self.latch_time();
                    }
                    (0x00, Latched) => self.latch_state = AttemptingUnlatch,
                    (0x01, AttemptingUnlatch) => self.latch_state = Unlatched,
                    (_, AttemptingLatch) => self.latch_state = Unlatched,
                    (_, AttemptingUnlatch) => self.latch_state = Latched,
                    (_, _) => {}
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.latch_state != LatchState::Latched {
                        self.latch_time();
                    }
                    match self.rtc_selected {
                        RTCMode::Ram => self.write_ram(addr, val),
                        RTCMode::Seconds => self.seconds = val.0,
                        RTCMode::Minutes => self.minutes = val.0,
                        RTCMode::Hours => self.hours = val.0,
                        RTCMode::LowerDay => self.lower_day = val.0,
                        RTCMode::UpperDay => self.upper_day = val.0,
                    }
                } else {
                    panic!("Attempted RAM/RTC write when disabled");
                }
            }
            _ => unreachable!(),
        }
    }
}

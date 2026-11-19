//! The audio processing unit (APU).
//!
//! Four channels - two square waves (the first with a frequency sweep), a
//! programmable wave, and a noise generator - mixed to stereo. Each is clocked
//! by the master clock through [`AudioProcessor::tick`], and a 512 Hz frame
//! sequencer drives the length counters, volume envelopes, and sweep. The mixed
//! output is down-sampled to [`SAMPLE_RATE`] and pushed into an internal buffer
//! the emulation loop drains each frame; nothing here talks to the sound card.

use serde::{Deserialize, Serialize};

use crate::{Address, Byte};

/// Output sample rate handed to the host. The APU resamples the ~4.19 MHz
/// master clock down to this.
pub const SAMPLE_RATE: u32 = 48_000;

/// Master clock ticks per second (4.194304 MHz).
const CLOCK_HZ: f32 = 4_194_304.0;

/// Duty-cycle patterns for the square channels, one bit per eighth of a period.
const DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

/// Noise divisor codes (NR43 bits 0-2).
const NOISE_DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

/// Read-back OR-masks for `0xFF10..=0xFF26`: bits that always read as 1
/// (write-only or unused bits). Wave RAM and the `0xFF27..=0xFF2F` gap are
/// handled separately.
const READ_MASKS: [u8; 0x17] = [
    0x80, 0x3F, 0x00, 0xFF, 0xBF, // NR10-NR14
    0xFF, 0x3F, 0x00, 0xFF, 0xBF, // NR20(unused)-NR24
    0x7F, 0xFF, 0x9F, 0xFF, 0xBF, // NR30-NR34
    0xFF, 0xFF, 0x00, 0x00, 0xBF, // NR40(unused)-NR44
    0x00, 0x00, 0x70, // NR50-NR52
];

#[derive(Debug, Default, Serialize, Deserialize)]
struct Envelope {
    /// Current output volume, 0-15.
    volume: u8,
    /// Initial volume latched on trigger (NRx2 bits 4-7).
    initial: u8,
    /// Whether the envelope steps up (NRx2 bit 3).
    add: bool,
    /// Envelope period in frame-sequencer envelope ticks (NRx2 bits 0-2).
    period: u8,
    timer: u8,
}

impl Envelope {
    const fn write(&mut self, value: u8) {
        self.initial = value >> 4;
        self.add = value & 0x08 != 0;
        self.period = value & 0x07;
    }

    /// Whether the DAC is powered - any non-zero volume or an upward envelope.
    const fn dac_on(&self) -> bool {
        self.initial != 0 || self.add
    }

    const fn trigger(&mut self) {
        self.volume = self.initial;
        self.timer = self.period;
    }

    const fn tick(&mut self) {
        if self.period == 0 {
            return;
        }
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.timer = self.period;
            if self.add && self.volume < 15 {
                self.volume += 1;
            } else if !self.add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

/// A square-wave channel. Channel 1 additionally uses the sweep fields.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Square {
    enabled: bool,
    dac_on: bool,
    duty: u8,
    duty_pos: u8,
    freq: u16,
    freq_timer: u16,
    length: u16,
    length_enabled: bool,
    env: Envelope,
    // Sweep (channel 1 only).
    has_sweep: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,
}

impl Square {
    const fn tick(&mut self) {
        if self.freq_timer == 0 {
            self.freq_timer = (2048 - self.freq) * 4;
            self.duty_pos = (self.duty_pos + 1) & 7;
        }
        self.freq_timer -= 1;
    }

    const fn tick_length(&mut self) {
        if self.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    const fn tick_sweep(&mut self) {
        if !self.has_sweep {
            return;
        }
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
            if self.sweep_enabled && self.sweep_period > 0 {
                let next = self.sweep_frequency();
                if next <= 2047 && self.sweep_shift > 0 {
                    self.sweep_shadow = next;
                    self.freq = next;
                    // A second calculation checks for overflow.
                    if self.sweep_frequency() > 2047 {
                        self.enabled = false;
                    }
                } else if next > 2047 {
                    self.enabled = false;
                }
            }
        }
    }

    const fn sweep_frequency(&self) -> u16 {
        let delta = self.sweep_shadow >> self.sweep_shift;
        if self.sweep_negate {
            self.sweep_shadow.wrapping_sub(delta)
        } else {
            self.sweep_shadow + delta
        }
    }

    const fn trigger(&mut self) {
        self.enabled = self.dac_on;
        if self.length == 0 {
            self.length = 64;
        }
        self.freq_timer = (2048 - self.freq) * 4;
        self.env.trigger();
        if self.has_sweep {
            self.sweep_shadow = self.freq;
            self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
            self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;
            if self.sweep_shift > 0 && self.sweep_frequency() > 2047 {
                self.enabled = false;
            }
        }
    }

    /// Digital output, 0-15.
    const fn sample(&self) -> u8 {
        if self.enabled && self.dac_on && DUTY[self.duty as usize][self.duty_pos as usize] == 1 {
            self.env.volume
        } else {
            0
        }
    }
}

/// The programmable wave channel (channel 3).
#[derive(Debug, Default, Serialize, Deserialize)]
struct Wave {
    enabled: bool,
    dac_on: bool,
    freq: u16,
    freq_timer: u16,
    position: u8,
    length: u16,
    length_enabled: bool,
    /// Volume code (NR32 bits 5-6): 0 mute, 1 full, 2 half, 3 quarter.
    volume_code: u8,
    ram: [u8; 16],
    current: u8,
}

impl Wave {
    const fn tick(&mut self) {
        if self.freq_timer == 0 {
            self.freq_timer = (2048 - self.freq) * 2;
            self.position = (self.position + 1) & 31;
            let byte = self.ram[self.position as usize / 2];
            self.current = if self.position & 1 == 0 { byte >> 4 } else { byte & 0x0F };
        }
        self.freq_timer -= 1;
    }

    const fn tick_length(&mut self) {
        if self.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    const fn trigger(&mut self) {
        self.enabled = self.dac_on;
        if self.length == 0 {
            self.length = 256;
        }
        self.freq_timer = (2048 - self.freq) * 2;
        self.position = 0;
    }

    const fn sample(&self) -> u8 {
        if !self.enabled || !self.dac_on {
            return 0;
        }
        match self.volume_code {
            0 => 0,
            1 => self.current,
            2 => self.current >> 1,
            _ => self.current >> 2,
        }
    }
}

/// The noise channel (channel 4).
#[derive(Debug, Serialize, Deserialize)]
struct Noise {
    enabled: bool,
    dac_on: bool,
    freq_timer: u16,
    lfsr: u16,
    clock_shift: u8,
    width_7bit: bool,
    divisor_code: u8,
    length: u16,
    length_enabled: bool,
    env: Envelope,
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            enabled: false,
            dac_on: false,
            freq_timer: 0,
            lfsr: 0x7FFF,
            clock_shift: 0,
            width_7bit: false,
            divisor_code: 0,
            length: 0,
            length_enabled: false,
            env: Envelope::default(),
        }
    }
}

impl Noise {
    const fn period(&self) -> u16 {
        NOISE_DIVISORS[self.divisor_code as usize] << self.clock_shift
    }

    fn tick(&mut self) {
        if self.freq_timer == 0 {
            self.freq_timer = self.period().max(1);
            let bit = (self.lfsr ^ (self.lfsr >> 1)) & 1;
            self.lfsr = (self.lfsr >> 1) | (bit << 14);
            if self.width_7bit {
                self.lfsr = (self.lfsr & !(1 << 6)) | (bit << 6);
            }
        }
        self.freq_timer -= 1;
    }

    const fn tick_length(&mut self) {
        if self.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_on;
        if self.length == 0 {
            self.length = 64;
        }
        self.freq_timer = self.period().max(1);
        self.lfsr = 0x7FFF;
        self.env.trigger();
    }

    const fn sample(&self) -> u8 {
        if self.enabled && self.dac_on && (self.lfsr & 1) == 0 {
            self.env.volume
        } else {
            0
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioProcessor {
    powered: bool,
    /// Frame-sequencer divider (counts master ticks to 8192 = 512 Hz) and step.
    fs_timer: u16,
    fs_step: u8,
    ch1: Square,
    ch2: Square,
    ch3: Wave,
    ch4: Noise,
    /// Master volume per side (NR50 bits 0-2 right, 4-6 left), each 0-7.
    vol_left: u8,
    vol_right: u8,
    /// Channel-to-side routing (NR51).
    panning: u8,
    /// Fractional master ticks accumulated toward the next output sample.
    sample_accum: f32,
    /// Interleaved stereo samples awaiting the host, drained each frame. Not
    /// part of the machine state, so a save state skips it.
    #[serde(skip)]
    output: Vec<f32>,
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self {
            powered: false,
            fs_timer: 8192,
            fs_step: 0,
            ch1: Square { has_sweep: true, ..Square::default() },
            ch2: Square::default(),
            ch3: Wave::default(),
            ch4: Noise::default(),
            vol_left: 0,
            vol_right: 0,
            panning: 0,
            sample_accum: 0.0,
            output: Vec::new(),
        }
    }
}

impl AudioProcessor {
    /// Advance the APU one master-clock tick, generating output samples as the
    /// resampler crosses each sample boundary.
    pub fn tick(&mut self) {
        if self.powered {
            self.ch1.tick();
            self.ch2.tick();
            self.ch3.tick();
            self.ch4.tick();

            if self.fs_timer == 0 {
                self.fs_timer = 8192;
                self.frame_sequencer_step();
            }
            self.fs_timer -= 1;
        }

        // Down-sample: emit one stereo frame every CLOCK_HZ / SAMPLE_RATE ticks.
        self.sample_accum += SAMPLE_RATE as f32 / CLOCK_HZ;
        if self.sample_accum >= 1.0 {
            self.sample_accum -= 1.0;
            let (l, r) = self.mix();
            self.output.push(l);
            self.output.push(r);
        }
    }

    const fn frame_sequencer_step(&mut self) {
        // 512 Hz sequence: length at 256 Hz, sweep at 128 Hz, envelope at 64 Hz.
        match self.fs_step {
            0 | 4 => self.tick_length(),
            2 | 6 => {
                self.tick_length();
                self.ch1.tick_sweep();
            }
            7 => self.tick_envelopes(),
            _ => {}
        }
        self.fs_step = (self.fs_step + 1) & 7;
    }

    const fn tick_length(&mut self) {
        self.ch1.tick_length();
        self.ch2.tick_length();
        self.ch3.tick_length();
        self.ch4.tick_length();
    }

    const fn tick_envelopes(&mut self) {
        self.ch1.env.tick();
        self.ch2.env.tick();
        self.ch4.env.tick();
    }

    /// Mix the four channels to a stereo pair in roughly `[-1.0, 1.0]`.
    fn mix(&self) -> (f32, f32) {
        // Each channel's DAC maps its 0-15 digital sample to [-1.0, 1.0].
        let dac = |s: u8| f32::from(s) / 7.5 - 1.0;
        let chans = [
            (self.ch1.dac_on, dac(self.ch1.sample())),
            (self.ch2.dac_on, dac(self.ch2.sample())),
            (self.ch3.dac_on, dac(self.ch3.sample())),
            (self.ch4.dac_on, dac(self.ch4.sample())),
        ];

        let mut left = 0.0;
        let mut right = 0.0;
        for (i, (dac_on, out)) in chans.iter().enumerate() {
            if !dac_on {
                continue;
            }
            if self.panning & (1 << (i + 4)) != 0 {
                left += out;
            }
            if self.panning & (1 << i) != 0 {
                right += out;
            }
        }

        // Average the four channels, then apply master volume (0-7 -> scaled).
        left = left / 4.0 * (f32::from(self.vol_left) + 1.0) / 8.0;
        right = right / 4.0 * (f32::from(self.vol_right) + 1.0) / 8.0;
        (left, right)
    }

    /// Take the stereo samples generated since the last call (interleaved L, R).
    pub fn drain(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.output)
    }

    pub fn read(&self, address: Address) -> Byte {
        let addr = address.0;
        // PCM12/PCM34 read-back of the current channel outputs (CGB only).
        if addr == 0xFF76 {
            return Byte(self.ch1.sample() | (self.ch2.sample() << 4));
        }
        if addr == 0xFF77 {
            return Byte(self.ch3.sample() | (self.ch4.sample() << 4));
        }
        // Wave RAM.
        if (0xFF30..=0xFF3F).contains(&addr) {
            return Byte(self.ch3.ram[(addr - 0xFF30) as usize]);
        }
        // NR52 status: power bit plus the four channel-enabled flags.
        if addr == 0xFF26 {
            let status = u8::from(self.ch1.enabled)
                | (u8::from(self.ch2.enabled) << 1)
                | (u8::from(self.ch3.enabled) << 2)
                | (u8::from(self.ch4.enabled) << 3);
            return Byte((u8::from(self.powered) << 7) | 0x70 | status);
        }
        // The rest read the last written value ORed with the write-only bits.
        if (0xFF10..=0xFF25).contains(&addr) {
            return Byte(self.reg_readback(addr));
        }
        // 0xFF27-0xFF2F are unused.
        Byte(0xFF)
    }

    fn reg_readback(&self, addr: u16) -> u8 {
        let index = (addr - 0xFF10) as usize;
        self.stored_reg(addr) | READ_MASKS[index]
    }

    /// Reconstruct the last stored value of a register from channel state, for
    /// read-back. Only the bits the mask does not force to 1 matter.
    fn stored_reg(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => {
                (self.ch1.sweep_period << 4)
                    | (u8::from(self.ch1.sweep_negate) << 3)
                    | self.ch1.sweep_shift
            }
            0xFF11 => self.ch1.duty << 6,
            0xFF12 => self.env_byte(&self.ch1.env),
            0xFF14 => u8::from(self.ch1.length_enabled) << 6,
            0xFF16 => self.ch2.duty << 6,
            0xFF17 => self.env_byte(&self.ch2.env),
            0xFF19 => u8::from(self.ch2.length_enabled) << 6,
            0xFF1A => u8::from(self.ch3.dac_on) << 7,
            0xFF1C => self.ch3.volume_code << 5,
            0xFF1E => u8::from(self.ch3.length_enabled) << 6,
            0xFF21 => self.env_byte(&self.ch4.env),
            0xFF22 => {
                (self.ch4.clock_shift << 4)
                    | (u8::from(self.ch4.width_7bit) << 3)
                    | self.ch4.divisor_code
            }
            0xFF23 => u8::from(self.ch4.length_enabled) << 6,
            0xFF24 => (self.vol_left << 4) | self.vol_right,
            0xFF25 => self.panning,
            _ => 0,
        }
    }

    fn env_byte(&self, env: &Envelope) -> u8 {
        (env.initial << 4) | (u8::from(env.add) << 3) | env.period
    }

    pub fn write(&mut self, address: Address, value: Byte) {
        let addr = address.0;
        let v = value.0;

        // Wave RAM is writable regardless of power.
        if (0xFF30..=0xFF3F).contains(&addr) {
            self.ch3.ram[(addr - 0xFF30) as usize] = v;
            return;
        }

        // NR52 bit 7 powers the APU. Powering off clears every register.
        if addr == 0xFF26 {
            let on = v & 0x80 != 0;
            if !on && self.powered {
                self.power_off();
            } else if on && !self.powered {
                self.powered = true;
                self.fs_step = 0;
            }
            return;
        }

        // While powered off, register writes are ignored (DMG lets length load
        // through, but ignoring is close enough and never faults).
        if !self.powered {
            return;
        }

        match addr {
            0xFF10 => {
                self.ch1.sweep_period = (v >> 4) & 0x07;
                self.ch1.sweep_negate = v & 0x08 != 0;
                self.ch1.sweep_shift = v & 0x07;
            }
            0xFF11 => {
                self.ch1.duty = v >> 6;
                self.ch1.length = 64 - u16::from(v & 0x3F);
            }
            0xFF12 => {
                self.ch1.env.write(v);
                self.ch1.dac_on = self.ch1.env.dac_on();
                if !self.ch1.dac_on {
                    self.ch1.enabled = false;
                }
            }
            0xFF13 => self.ch1.freq = (self.ch1.freq & 0x0700) | u16::from(v),
            0xFF14 => {
                self.ch1.freq = (self.ch1.freq & 0x00FF) | (u16::from(v & 0x07) << 8);
                self.ch1.length_enabled = v & 0x40 != 0;
                if v & 0x80 != 0 {
                    self.ch1.trigger();
                }
            }
            0xFF16 => {
                self.ch2.duty = v >> 6;
                self.ch2.length = 64 - u16::from(v & 0x3F);
            }
            0xFF17 => {
                self.ch2.env.write(v);
                self.ch2.dac_on = self.ch2.env.dac_on();
                if !self.ch2.dac_on {
                    self.ch2.enabled = false;
                }
            }
            0xFF18 => self.ch2.freq = (self.ch2.freq & 0x0700) | u16::from(v),
            0xFF19 => {
                self.ch2.freq = (self.ch2.freq & 0x00FF) | (u16::from(v & 0x07) << 8);
                self.ch2.length_enabled = v & 0x40 != 0;
                if v & 0x80 != 0 {
                    self.ch2.trigger();
                }
            }
            0xFF1A => {
                self.ch3.dac_on = v & 0x80 != 0;
                if !self.ch3.dac_on {
                    self.ch3.enabled = false;
                }
            }
            0xFF1B => self.ch3.length = 256 - u16::from(v),
            0xFF1C => self.ch3.volume_code = (v >> 5) & 0x03,
            0xFF1D => self.ch3.freq = (self.ch3.freq & 0x0700) | u16::from(v),
            0xFF1E => {
                self.ch3.freq = (self.ch3.freq & 0x00FF) | (u16::from(v & 0x07) << 8);
                self.ch3.length_enabled = v & 0x40 != 0;
                if v & 0x80 != 0 {
                    self.ch3.trigger();
                }
            }
            0xFF20 => self.ch4.length = 64 - u16::from(v & 0x3F),
            0xFF21 => {
                self.ch4.env.write(v);
                self.ch4.dac_on = self.ch4.env.dac_on();
                if !self.ch4.dac_on {
                    self.ch4.enabled = false;
                }
            }
            0xFF22 => {
                self.ch4.clock_shift = v >> 4;
                self.ch4.width_7bit = v & 0x08 != 0;
                self.ch4.divisor_code = v & 0x07;
            }
            0xFF23 => {
                self.ch4.length_enabled = v & 0x40 != 0;
                if v & 0x80 != 0 {
                    self.ch4.trigger();
                }
            }
            0xFF24 => {
                self.vol_right = v & 0x07;
                self.vol_left = (v >> 4) & 0x07;
            }
            0xFF25 => self.panning = v,
            _ => {}
        }
    }

    fn power_off(&mut self) {
        // Preserve wave RAM (survives power-off on hardware) and rebuild.
        let ram = self.ch3.ram;
        *self = Self::default();
        self.ch3.ram = ram;
    }
}

#[cfg(test)]
mod tests {
    use super::AudioProcessor;
    use crate::{Address, Byte};

    fn w(apu: &mut AudioProcessor, addr: u16, value: u8) {
        apu.write(Address(addr), Byte(value));
    }

    /// Power on, route everything to both sides at full volume, and play a tone
    /// on the square channel 2.
    fn playing_apu() -> AudioProcessor {
        let mut apu = AudioProcessor::default();
        w(&mut apu, 0xFF26, 0x80); // power on
        w(&mut apu, 0xFF24, 0x77); // master volume: 7 both sides
        w(&mut apu, 0xFF25, 0xFF); // pan every channel to both sides
        w(&mut apu, 0xFF16, 0x80); // CH2 duty 50%
        w(&mut apu, 0xFF17, 0xF0); // CH2 envelope: volume 15, no decay
        w(&mut apu, 0xFF18, 0x00); // CH2 freq low
        w(&mut apu, 0xFF19, 0x87); // CH2 trigger + freq high
        apu
    }

    #[test]
    fn a_triggered_square_channel_oscillates() {
        let mut apu = playing_apu();
        for _ in 0..200_000 {
            apu.tick();
        }
        let samples = apu.drain();
        assert!(!samples.is_empty(), "the resampler produced output");

        let max = samples.iter().copied().fold(f32::MIN, f32::max);
        let min = samples.iter().copied().fold(f32::MAX, f32::min);
        assert!(max > 0.01, "waveform rises above zero (max {max})");
        assert!(min < -0.01, "waveform falls below zero (min {min})");
    }

    #[test]
    fn powering_off_silences_and_disables_channels() {
        let mut apu = playing_apu();
        for _ in 0..1_000 {
            apu.tick();
        }
        apu.drain();

        w(&mut apu, 0xFF26, 0x00); // power off
        for _ in 0..10_000 {
            apu.tick();
        }
        let samples = apu.drain();
        assert!(
            samples.iter().all(|&s| s == 0.0),
            "no output while powered off"
        );
        // NR52 reports the APU off with all channels cleared.
        assert_eq!(apu.read(Address(0xFF26)), Byte(0x70));
    }

    #[test]
    fn nr52_reports_channel_status_and_read_masks() {
        let mut apu = playing_apu();
        apu.tick();
        // CH2 was triggered, so its status bit (bit 1) is set, power bit (7) set.
        let nr52 = apu.read(Address(0xFF26)).0;
        assert_eq!(nr52 & 0x80, 0x80, "power on");
        assert_eq!(nr52 & 0x02, 0x02, "channel 2 active");
        // A write-only register still reads its masked bits as 1.
        assert_eq!(apu.read(Address(0xFF13)), Byte(0xFF));
    }
}

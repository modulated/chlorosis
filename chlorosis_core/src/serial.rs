//! The serial link port (`0xFF01` SB, `0xFF02` SC).
//!
//! No real link partner is emulated, but test ROMs (Blargg's suite in
//! particular) use the port as a console: they put a byte in SB and start a
//! transfer with the internal clock, expecting it to shift out. Those bytes are
//! captured here so a headless harness can read the ROM's text output.

use serde::{Deserialize, Serialize};

use crate::types::{Address, Byte};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Serial {
    /// SB: the byte staged for transfer.
    data: Byte,
    /// SC: transfer control. Bit 7 starts a transfer, bit 0 selects the
    /// internal clock.
    control: Byte,
    /// Everything shifted out so far - the test-ROM console.
    output: Vec<u8>,
}

impl Serial {
    pub fn read(&self, address: Address) -> Byte {
        match address.0 {
            0xFF01 => self.data,
            // Only bits 7 and 0 are meaningful; the rest read as 1.
            0xFF02 => Byte(self.control.0 | 0x7E),
            _ => unreachable!("serial cannot read {address}"),
        }
    }

    pub fn write(&mut self, address: Address, value: Byte) {
        match address.0 {
            0xFF01 => self.data = value,
            0xFF02 => {
                self.control = value;
                // A requested transfer completes instantly in this model: the
                // staged byte is captured and the start bit clears.
                if value.is_bit_set(7) {
                    self.output.push(self.data.0);
                    self.control.clear_bit(7);
                }
            }
            _ => unreachable!("serial cannot write {address}"),
        }
    }

    /// Everything the ROM has shifted out of the port.
    pub fn output(&self) -> &[u8] {
        &self.output
    }
}

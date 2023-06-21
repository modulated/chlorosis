use crate::emu::{Address, Byte};

// Opcodes have a cycle count and byte count

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Opcode {
    NOP = 0x00,
    LD_BC_d16(Address) = 0x01,
    LD_BC_A = 0x02,
    INC_BC = 0x03,
    INC_B = 0x04,

    LD_SP_d16(Address) = 0x31,
    LD_A_d8(Byte) = 0x3A,
}

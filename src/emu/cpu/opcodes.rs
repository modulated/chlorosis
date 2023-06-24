use crate::emu::{Address, Byte};

// Opcodes have a cycle count and byte count

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Opcode {
    // 0 Row
    NOP = 0x00,
    LD_BC_d16(Address) = 0x01,
    LD_BC_A = 0x02,
    INC_BC = 0x03,
    INC_B = 0x04,
    DEC_B = 0x05,
    LD_B_d8(Byte) = 0x06,
    RLCA = 0x07,
    LD_a16_SP(Address) = 0x08,
    ADD_HL_BC = 0x09,
    LD_A_BC = 0x0A,
    DEC_BC = 0x0B,
    INC_C = 0x0C,
    DEC_C = 0x0D,
    LD_C_d8(Byte) = 0x0E,
    RRCA = 0x0F,
    // 0 Row

    // 1 Row
    STOP(Byte) = 0x10,
    LD_DE_d16(Address) = 0x11,

    RLA = 0x17,

    LD_A_DE = 0x1A,

    JR_NZ_s8(Byte) = 0x20,
    LD_HL_d16(Byte, Byte) = 0x21,
    LD_HL_inc_A = 0x22,

    CPL = 0x2F,

    LD_SP_d16(Address) = 0x31,
    LD_HL_dec_A = 0x32,
    LD_A_d8(Byte) = 0x3A,

    LD_C_A = 0x4F,

    LD_HL_A = 0x77,

    XOR_A = 0xAF,

    CP_H = 0xBC,

    POP_BC = 0xC1,
    JP_NZ_a16(Address) = 0xC2,
    JP_a16(Address) = 0xC3,
    CALL_NZ_a16(Address) = 0xC4,
    PUSH_BC = 0xC5,

    // CB = 0xCB,
    CALL_a16(Address) = 0xCD,

    LD_a8_A(Address) = 0xE0,
    POP_HL = 0xE1,
    LD_aC_A = 0xE2,

    // CB Extensions
    RL_C = 0xCB11,

    BIT_7_H = 0xCB7C,
}

// impl std::fmt::Display for Opcode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if (self as u16) > 0xCA00 {
//             Ok(write!(f, "{self} (0x{})", Byte(*self as u8))?)
//         } else {
//             Ok(write!(f, "{self} ({})", Byte(*self as u8))?)
//         }
//     }
// }

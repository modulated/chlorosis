use crate::emu::{types::SignedByte, Address, Byte};

// Opcodes have a cycle count and byte count

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Opcode {
    // Row 0
    NOP = 0x00,
    LD_BC_d16(Address) = 0x01,
    LD_aBC_A = 0x02,
    INC_BC = 0x03,
    INC_B = 0x04,
    DEC_B = 0x05,
    LD_B_d8(Byte) = 0x06,
    RLCA = 0x07,
    LD_a16_SP(Address) = 0x08,
    ADD_HL_BC = 0x09,
    LD_A_aBC = 0x0A,
    DEC_BC = 0x0B,
    INC_C = 0x0C,
    DEC_C = 0x0D,
    LD_C_d8(Byte) = 0x0E,
    RRCA = 0x0F,
    // Row 0

    // Row 1
    STOP(Byte) = 0x10,
    LD_DE_d16(Address) = 0x11,
    LD_aDE_A = 0x12,
    INC_DE = 0x13,
    INC_D = 0x14,
    DEC_D = 0x15,
    LD_D_d8(Byte) = 0x16,
    RLA = 0x17,
    JR_s8(SignedByte) = 0x18,
    ADD_HL_DE = 0x19,
    LD_A_aDE = 0x1A,
    DEC_DE = 0x1B,
    INC_E = 0x1C,
    DEC_E = 0x1D,
    LD_E_d8(Byte) = 0x1E,
    RRA,
    // Row 1

    // Row 2
    JR_NZ_s8(SignedByte) = 0x20,
    LD_HL_d16(Address) = 0x21,
    LD_aHL_inc_A = 0x22,
    INC_HL = 0x23,
    INC_H = 0x24,
    DEC_H = 0x25,
    LD_H_d8(Byte) = 0x26,
    DAA = 0x27,
    JR_Z_s8(SignedByte) = 0x28,
    ADD_HL_HL = 0x29,
    LD_A_aHL_inc = 0x2A,
    DEC_HL = 0x2B,
    INC_L = 0x2C,
    DEC_L = 0x2D,
    LD_L_d8(Byte) = 0x2E,
    CPL = 0x2F,
    // Row 2

    // Row 3
    JR_NC_s8(SignedByte) = 0x30,
    LD_SP_d16(Address) = 0x31,
    LD_aHL_dec_A = 0x32,
    INC_SP = 0x33,
    INC_aHL = 0x34,
    DEC_aHL = 0x35,
    LD_aHL_d8(Byte) = 0x36,
    SCF = 0x37,
    JR_C_s8(SignedByte) = 0x38,
    ADD_HL_SP = 0x39,
    LD_A_aHL_dec = 0x3A,
    DEC_SP = 0x3B,
    INC_A = 0x3C,
    DEC_A = 0x3D,
    LD_A_d8(Byte) = 0x3E,
    CCF = 0x3F,
    // Row 3

    // Row 4
    LD_C_A = 0x4F,


    LD_D_A = 0x57,

    LD_H_A = 0x67,

    LD_HL_A = 0x77,

    LD_A_E = 0x7B,

    SUB_B = 0x90,

    XOR_A = 0xAF,

    CP_H = 0xBC,

    POP_BC = 0xC1,
    JP_NZ_a16(Address) = 0xC2,
    JP_a16(Address) = 0xC3,
    CALL_NZ_a16(Address) = 0xC4,
    PUSH_BC = 0xC5,

    RET = 0xC9,
    CALL_a16(Address) = 0xCD,

    LD_a8_A(Address) = 0xE0,
    POP_HL = 0xE1,
    LD_aC_A = 0xE2,

    LD_a16_A(Address) = 0xEA,

    LD_A_a8(Byte) = 0xF0,

    CP_d8(Byte) = 0xFE,

    // CB Extensions
    RL_C = 0xCB11,

    BIT_7_H = 0xCB7C,
}

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
    LD_B_B = 0x40,
    LD_B_C = 0x41,
    LD_B_D = 0x42,
    LD_B_E = 0x43,
    LD_B_H = 0x44,
    LD_B_L = 0x45,
    LD_B_aHL = 0x46,
    LD_B_A = 0x47,
    LD_C_B = 0x48,
    LD_C_C = 0x49,
    LD_C_D = 0x4A,
    LD_C_E = 0x4B,
    LD_C_H = 0x4C,
    LD_C_L = 0x4D,
    LD_C_aHL = 0x4E,
    LD_C_A = 0x4F,
    // Row 4

    // Row 5
    LD_D_B = 0x50,
    LD_D_C = 0x51,
    LD_D_D = 0x52,
    LD_D_E = 0x53,
    LD_D_H = 0x54,
    LD_D_L = 0x55,
    LD_D_aHL = 0x56,
    LD_D_A = 0x57,
    LD_E_B = 0x58,
    LD_E_C = 0x59,
    LD_E_D = 0x5A,
    LD_E_E = 0x5B,
    LD_E_H = 0x5C,
    LD_E_L = 0x5D,
    LD_E_aHL = 0x5E,
    LD_E_A = 0x5F,
    // Row 5

    // Row 6
    LD_H_B = 0x60,
    LD_H_C = 0x61,
    LD_H_D = 0x62,
    LD_H_E = 0x63,
    LD_H_H = 0x64,
    LD_H_L = 0x65,
    LD_H_aHL = 0x66,
    LD_H_A = 0x67,
    LD_L_B = 0x68,
    LD_L_C = 0x69,
    LD_L_D = 0x6A,
    LD_L_E = 0x6B,
    LD_L_H = 0x6C,
    LD_L_L = 0x6D,
    LD_L_aHL = 0x6E,
    LD_L_A = 0x6F,
    // Row 6

    // Row 7
    LD_aHL_B = 0x70,
    LD_aHL_C = 0x71,
    LD_aHL_D = 0x72,
    LD_aHL_E = 0x73,
    LD_aHL_H = 0x74,
    LD_aHL_L = 0x75,
    HALT = 0x76,
    LD_aHL_A = 0x77,
    LD_A_B = 0x78,
    LD_A_C = 0x79,
    LD_A_D = 0x7A,
    LD_A_E = 0x7B,
    LD_A_H = 0x7C,
    LD_A_L = 0x7D,
    LD_A_aHL = 0x7E,
    LD_A_A = 0x7F,
    // Row 7
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

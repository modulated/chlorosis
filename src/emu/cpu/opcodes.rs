use crate::emu::types::BytePair;

// Opcodes have a cycle count and byte count

#[repr(u8)]
enum Opcode {
    Nop = 0x00,
    LdBc16(BytePair) = 0x01,
    LdBcA = 0x02,
    IncBc = 0x03,
    IncB = 0x04,
}

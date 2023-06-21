use crate::emu::MemoryMap;

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn execute(&mut self, mmap: &mut MemoryMap, op: Opcode) {
        use Opcode::*;
        match op {
            NOP => {}
            LD_SP_d16(val) => {
                self.sp = val;
                self.cycle_timer = 3;
            }
            LD_A_d8(val) => {
                self.a = val;
                self.cycle_timer = 2;
            }
            _ => panic!("Unimplemented execution {:?}", op),
        }

        self.cycle_timer -= 1;
    }
}

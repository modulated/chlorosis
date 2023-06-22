use crate::emu::{Address, MemoryMap};

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn execute(&mut self, mmap: &mut MemoryMap, op: Opcode) {
        use Opcode::*;
        assert!(self.cost == 0);
        match op {
            NOP => {
                // 0x00
                println!("{}: NOP", self.pc);
                self.cost = 1;
            }
            //
            DEC_C => {
                self.c -= 0x1;
                if self.c.0 == 0 {
                    self.z_flag = true;
                } else {
                    self.z_flag = false;
                }
                self.cost = 1;
            }
            LD_C_d8(val) => {
                // 0x0E
                self.c = val;
                self.cost = 2;
            }

            LD_HL_d16(l, h) => {
                // 0x21
                self.h = h;
                self.l = l;
                self.cost = 3;
            }
            LD_HL_inc_A => {
                // 0x22
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.write_hl(addr + 1);
                self.cost = 2;
            }

            CPL => {
                self.a = !self.a;
                self.n_flag = true;
                self.h_flag = true;
                self.cost = 1;
            }

            LD_SP_d16(val) => {
                // 0x31
                self.sp = val;
                self.cost = 3;
            }
            LD_A_d8(val) => {
                // 0x3E
                self.a = val;
                self.cost = 2;
            }

            XOR_A => {
                //0xAF
                self.a ^= self.a;
                self.clear_flags();
                self.z_flag = true;
                self.cost = 1;
            }

            JP_NZ_a16(addr) => {
                // 0xC2
                if self.z_flag {
                    self.pc = addr;
                    self.cost = 4;
                } else {
                    self.cost = 3;
                }
            }
            JP_a16(addr) => {
                // 0xC3
                self.pc = addr;
                self.cost = 4;
            }

            LD_a8_A(addr) => {
                // 0xE0
                let target = Address(0xFF00) + addr;
                mmap.write(target, self.a);
                self.cost = 3;
            }

            CALL_a16(addr) => {
                self.push_address(mmap, self.pc);
                self.pc = addr;
                self.cost = 6;
            }

            _ => panic!("Unimplemented execution of {:?}", op),
        }
        self.cost -= 1;
    }
}

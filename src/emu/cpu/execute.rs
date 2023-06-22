use crate::emu::{Address, MemoryMap};

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn execute(&mut self, mmap: &mut MemoryMap, op: Opcode) {
        use Opcode::*;
        assert!(self.cost == 0);
        match op {
            // 0x00
            NOP => {
                println!("{}: NOP", self.pc);
                self.dump_state();
                mmap.dump_rom();
                self.cost = 1;
            }
            DEC_B => {
                self.b -= 1;
                if self.b.0 == 0 {
                    self.z_flag = true;
                } else {
                    self.z_flag = false;
                }
                self.n_flag = true;
                // TODO H flag
                self.cost = 1;
            }
            // 0x06
            LD_B_d8(val) => {
                self.b = val;
                self.cost = 2;
            }
            // 0x0C
            INC_C => {
                self.c += 1;
                if self.c.0 == 0 {
                    self.z_flag = true;
                } else {
                    self.z_flag = false;
                }
                self.n_flag = false;
                // TODO H flag
                self.cost = 1;
            }
            // 0x0D
            DEC_C => {
                self.c -= 0x1;
                if self.c.0 == 0 {
                    self.z_flag = true;
                } else {
                    self.z_flag = false;
                }
                self.n_flag = true;
                // TODO H flag
                self.cost = 1;
            }
            // 0x0E
            LD_C_d8(val) => {
                self.c = val;
                self.cost = 2;
            }
            // 0x11 
            LD_DE_d16(addr) => {
                self.write_de(addr);
                self.cost = 3;
            }
            // 0x17
            RLA => {
                let old_cf = self.c_flag;
                self.clear_flags();
                if self.c.is_bit_set(7) {
                    self.c_flag = true;
                }
                let mut val = self.a << 1;
                if old_cf {
                    val.set_bit(0);
                }
                self.cost = 1;
            }
            // 0x1A
            LD_A_DE => {
                self.a = mmap.read(self.read_de());
                self.cost = 2;
            }
            // 0x20
            JR_NZ_s8(val) => {
                let val = val.to_signed();
                if !self.z_flag {         
                    let addr = (self.pc.0 - 2) as i32 + val as i32;   
                    assert!(addr > 0);        
                    self.pc = Address(addr as u16);
                    self.cost = 3;
                } else {
                    self.cost = 2;
                }    
            }
            // 0x21
            LD_HL_d16(l, h) => {
                self.h = h;
                self.l = l;
                self.cost = 3;
            }
            // 0x22
            LD_HL_inc_A => {
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.write_hl(addr + 1);
                self.cost = 2;
            }                       
            // 0x23
            LD_HL_dec_A => {
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.write_hl(addr - 1);
                self.cost = 2;
            }
            // 0x2F
            CPL => {
                self.a = !self.a;
                self.n_flag = true;
                self.h_flag = true;
                self.cost = 1;
            }
            // 0x31
            LD_SP_d16(val) => {
                self.sp = val;
                self.cost = 3;
            }
            // 0x3E
            LD_A_d8(val) => {
                self.a = val;
                self.cost = 2;
            }

            // 0x4F
            LD_C_A => {
                self.c = self.a;
                self.cost = 1;
            }

            // 0x77
            LD_HL_A => {
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.cost = 2;
            }

            //0xAF
            XOR_A => {
                self.a ^= self.a;
                self.clear_flags();
                if self.a.0 == 0 {
                    self.z_flag = true;
                }
                self.cost = 1;
            }
            // 0xBC
            CP_H => {
                if self.a == self.h {
                    self.z_flag = true;
                } else {
                    self.z_flag = false;
                }
                self.n_flag = true;
                // TODO: C and H flags
                self.cost = 1;
            }
            // 0xC1
            POP_BC => {
                let addr = self.pop_address(mmap);
                self.write_bc(addr);
                self.cost = 3;
            }
            // 0xC2
            JP_NZ_a16(addr) => {
                if self.z_flag {
                    self.pc = addr;
                    self.cost = 4;
                } else {
                    self.cost = 3;
                }
            }
            // 0xC3
            JP_a16(addr) => {
                self.pc = addr;
                self.cost = 4;
            }
            // 0xC5
            PUSH_BC => {
                self.push_address(mmap, self.read_bc());
                self.cost = 4;
            }
            // 0xCD
            CALL_a16(addr) => {
                self.push_address(mmap, self.pc);
                self.pc = addr;
                self.cost = 6;
            }
            // 0xE0
            LD_a8_A(addr) => {
                let target = Address(0xFF00) + addr;
                mmap.write(target, self.a);
                self.cost = 3;
            }

            LD_aC_A => {
                mmap.write(Address(0xFF00) + self.c.to_address(), self.a);
                self.cost = 2;
            }

            // CB Extensions
            // 0xCB11
            RL_C => {
                let old_cf = self.c_flag; 
                self.clear_flags();
                if self.c.is_bit_set(7) {
                    self.c_flag = true;
                }
                let mut val = self.c << 1;
                if old_cf {
                    val.set_bit(0);
                }
                if val.0 == 0 {
                    self.z_flag = true;
                }               
                self.cost = 2;
            }
            // 0xCB7C
            BIT_7_H => {
                self.z_flag = self.h.is_bit_set(7);
                self.n_flag = false;
                self.h_flag = true;
                self.cost = 2;
            }

            _ => panic!("Unimplemented instruction {:?}", op),
        }
        assert!(self.cost != 0, "Forgot to simulate instruction cycle cost");
        self.cost -= 1;
    }
}

#[cfg(test)]
mod test {
    use crate::emu::Byte;

    #[test]
    fn test_is_bit_set() {
        let b = Byte(0x9F);
        assert!(b.is_bit_set(7));
        assert!(!b.is_bit_set(6));
        assert!(b.is_bit_set(0));
    }
}
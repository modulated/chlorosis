use crate::{
    addition_register_pairs, decrement_register,
    emu::{Address, Byte, MemoryMap},
    increment_register,
};

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn execute(&mut self, mmap: &mut MemoryMap, op: Opcode) {
        use Opcode::*;
        assert!(self.cost == 0);
        match op {
            // Row 0
            // 0x00
            NOP => {
                println!("{}: NOP", self.pc);
                self.dump_state();
                mmap.dump_rom();
                self.cost = 1;
            }
            // 0x01
            LD_BC_d16(addr) => {
                self.write_bc(addr);
                self.cost = 3;
            }
            // 0x02
            LD_aBC_A => {
                let addr = self.read_bc();
                self.a = mmap.read(addr);
                self.cost = 2;
            }
            // 0x03
            INC_BC => {
                self.write_bc(self.read_bc() + 1);
                self.cost = 2;
            }
            // 0x04
            INC_B => {
                increment_register!(self, self.b);
                self.cost = 1;
            }
            // 0x05
            DEC_B => {
                decrement_register!(self, self.b);
                self.cost = 1;
            }
            // 0x06
            LD_B_d8(val) => {
                self.b = val;
                self.cost = 2;
            }
            // 0x07
            RLCA => {
                self.clear_flags();
                self.c_flag = self.a.is_bit_set(7);
                self.a = self.a << 1;
                self.a.write_bit(0, self.c_flag);
                self.cost = 1;
            }
            // 0x08
            LD_a16_SP(addr) => {
                let (h, l) = self.sp.split();
                mmap.write(addr, l);
                mmap.write(addr + 1, h);
                self.cost = 5;
            }
            // 0x09
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_BC => {
                let bc = self.read_bc();
                let hl = self.read_hl();
                addition_register_pairs!(self, bc, hl, (|x| { self.write_hl(x) }));
                self.cost = 2;
            }
            // 0x0A
            LD_A_aBC => {
                self.a = mmap.read(self.read_bc());
                self.cost = 2;
            }
            // 0x0B
            DEC_BC => {
                self.write_bc(self.read_bc() - 1);
                self.cost = 2;
            }
            // 0x0C
            INC_C => {
                increment_register!(self, self.c);
                self.cost = 1;
            }
            // 0x0D
            DEC_C => {
                decrement_register!(self, self.c);
                self.cost = 1;
            }
            // 0x0E
            LD_C_d8(val) => {
                self.c = val;
                self.cost = 2;
            }
            // 0x0F
            RRCA => {
                self.clear_flags();
                self.c_flag = self.a.is_bit_set(0);
                self.a = self.a >> 1;
                self.a.write_bit(7, self.c_flag);
                self.cost = 1;
            }
            // Row 0

            // Row 1
            // 0x10
            STOP(_) => {
                // TODO: Implement
                // IF all IE flags reset AND input P10 to P13 are LOW
                // STOP SYSTEM CLOCK and OSCILLATOR CIRCUIT and LCD controller
                // Cancelled by RESET signal

                panic!("Unimplemented STOP");
            }
            // 0x11
            LD_DE_d16(addr) => {
                self.write_de(addr);
                self.cost = 3;
            }
            // 0x12
            LD_aDE_A => {
                let addr = self.read_de();
                mmap.write(addr, self.a);
                self.cost = 2;
            }
            // 0x13
            INC_DE => {
                self.write_de(self.read_de() + 1);
                self.cost = 2;
            }
            // 0x14
            INC_D => {
                increment_register!(self, self.d);
                self.cost = 1;
            }
            // 0x15
            DEC_D => {
                decrement_register!(self, self.d);
                self.cost = 1;
            }
            // 0x16
            LD_D_d8(val) => {
                self.d = val;
                self.cost = 2;
            }
            // 0x17
            RLA => {
                let prev = self.c_flag;
                self.c_flag = self.a.is_bit_set(7);
                let mut val = self.a << 1;
                val.write_bit(0, prev);
                self.cost = 1;
            }
            // 0x18
            JR_s8(val) => {
                self.pc = Address(((self.pc.0) as i32 + val.0 as i32) as u16);
                self.cost = 3;
            }
            // 0x19
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_DE => {
                let hl = self.read_hl();
                let de = self.read_de();
                addition_register_pairs!(self, hl, de, (|x| { self.write_hl(x) }));
                self.cost = 2;
            }
            // 0x1A
            LD_A_aDE => {
                self.a = mmap.read(self.read_de());
                self.cost = 2;
            }
            // 0x1B
            DEC_DE => {
                self.write_de(self.read_de());
                self.cost = 2;
            }
            // 0x1C
            INC_E => {
                increment_register!(self, self.e);
                self.cost = 1;
            }
            // 0x1D
            DEC_E => {
                decrement_register!(self, self.e);
                self.cost = 1;
            }
            // 0x1E
            LD_E_d8(val) => {
                self.e = val;
                self.cost = 2;
            }
            // 0x1F
            RRA => {
                let prev = self.c_flag;
                self.c_flag = self.a.is_bit_set(0);
                let mut val = self.a >> 1;
                val.write_bit(7, prev);
                self.cost = 1;
            }
            // Row 1

            // Row 2
            // 0x20
            JR_NZ_s8(signed) => {
                if !self.z_flag {
                    self.pc = Address(((self.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cost = 3;
                } else {
                    self.cost = 2;
                }
            }
            // 0x21
            LD_HL_d16(addr) => {
                self.write_hl(addr);
                self.cost = 3;
            }
            // 0x22
            LD_aHL_inc_A => {
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.write_hl(addr + 1);
                self.cost = 2;
            }
            // 0x23
            INC_HL => {
                self.write_hl(self.read_hl() + 1);
                self.cost = 2;
            }
            // 0x24
            INC_H => {
                increment_register!(self, self.h);
                self.cost = 1;
            }
            // 0x25
            DEC_H => {
                decrement_register!(self, self.h);
                self.cost = 1;
            }
            // 0x26
            LD_H_d8(val) => {
                self.h = val;
                self.cost = 2;
            }
            // 0x27
            DAA => {
                // TODO: implement BCD operation
                unimplemented!()
            }
            // 0x28
            JR_Z_s8(signed) => {
                if self.z_flag {
                    self.pc = Address(((self.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cost = 3;
                } else {
                    self.cost = 2;
                }
            }
            // 0x29
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_HL => {
                let hl = self.read_hl();
                addition_register_pairs!(self, hl, hl, (|x| { self.write_hl(x) }));
                self.cost = 2;
            }
            // 0x2A
            LD_A_aHL_inc => {
                self.a = mmap.read(self.read_hl());
                self.write_hl(self.read_hl() + 1);
                self.cost = 2;
            }
            // 0x2B
            DEC_HL => {
                self.write_hl(self.read_hl() - 1);
                self.cost = 2;
            }
            // 0x2C
            INC_L => {
                increment_register!(self, self.l);
                self.cost = 1;
            }
            // 0x2D
            DEC_L => {
                decrement_register!(self, self.l);
                self.cost = 1;
            }
            // 0x2E
            LD_L_d8(val) => {
                self.l = val;
                self.cost = 2;
            }
            // 0x2F
            CPL => {
                self.a = !self.a;
                self.n_flag = true;
                self.h_flag = true;
                self.cost = 1;
            }
            // Row 2

            // Row 3
            // 0x31
            LD_SP_d16(val) => {
                self.sp = val;
                self.cost = 3;
            }
            // 0x32
            LD_aHL_dec_A => {
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.write_hl(addr - 1);
                self.cost = 2;
            }
            // 0x33
            INC_SP => {
                self.sp += 1;
                self.cost = 2;
            }
            // 0x34
            INC_aHL => {
                let old_val = mmap.read(self.read_hl());
                let new_val = Byte(old_val.0.wrapping_add(1));
                mmap.write(self.read_hl(), new_val);
                self.check_zero(new_val);
                self.check_half_carry_add_byte(old_val, Byte(1));
                self.n_flag = false;
                self.cost = 3;
            }
            // 0x35
            DEC_aHL => {
                let old_val = mmap.read(self.read_hl());
                let new_val = Byte(old_val.0.wrapping_sub(1));
                mmap.write(self.read_hl(), new_val);
                self.check_zero(new_val);
                self.check_half_carry_sub_byte(old_val, Byte(1));
                self.n_flag = true;
                self.cost = 3;
            }
            // 0x36
            LD_aHL_d8(val) => {
                mmap.write(self.read_hl(), val);
                self.cost = 3;
            }
            // 0x37
            SCF => {
                self.c_flag = true;
                self.n_flag = false;
                self.h_flag = false;
                self.cost = 1;
            }
            // 0x38
            JR_C_s8(signed) => {
                if self.c_flag {
                    self.pc = Address(((self.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cost = 3;
                } else {
                    self.cost = 2;
                }
            }
            // 0x39
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_SP => {
                let sp = self.sp;
                let hl = self.read_hl();
                addition_register_pairs!(self, sp, hl, (|x| { self.write_hl(x) }));
                self.cost = 2;
            }
            // 0x3A
            LD_A_aHL_dec => {
                self.a = mmap.read(self.read_hl());
                self.write_hl(self.read_hl() - 1);
                self.cost = 2;
            }
            // 0x3B
            DEC_SP => {
                self.sp -= 1;
                self.cost = 2;
            }
            // 0x3C
            INC_A => {
                increment_register!(self, self.a);
                self.cost = 1;
            }
            // 0x3D
            DEC_A => {
                decrement_register!(self, self.a);
                self.cost = 1;
            }
            // 0x3E
            LD_A_d8(val) => {
                self.a = val;
                self.cost = 2;
            }
            CCF => {
                self.c_flag = !self.c_flag;
                self.n_flag = false;
                self.h_flag = false;
                self.cost = 1;
            }
            // Row 3

            // Row 4
            // 0x4F
            LD_C_A => {
                self.c = self.a;
                self.cost = 1;
            }

            // 0x57
            LD_D_A => {
                self.d = self.a;
                self.cost = 1;
            }

            // 0x67
            LD_H_A => {
                self.h = self.a;
                self.cost = 1;
            }

            // 0x77
            LD_HL_A => {
                let addr = self.read_hl();
                mmap.write(addr, self.a);
                self.cost = 2;
            }

            // 0x7B
            LD_A_E => {
                self.a = self.e;
                self.cost = 1;
            }

            // 0x90
            SUB_B => {
                self.sub(self.b);
                self.cost = 1;
            }

            //0xAF
            XOR_A => {
                self.xor(self.a);
                self.cost = 1;
            }
            // 0xBC
            CP_H => {
                let prev = self.a;
                self.sub(self.h);
                self.a = prev;
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

            // 0xC9
            RET => {
                self.pc = self.pop_address(mmap);
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

            // 0xE2
            LD_aC_A => {
                mmap.write(Address(0xFF00) + self.c.to_address(), self.a);
                self.cost = 2;
            }

            // 0xEA
            LD_a16_A(addr) => {
                mmap.write(addr, self.a);
                self.cost = 4;
            }

            // 0xF0
            LD_A_a8(val) => {
                let addr = Address(0xFF00) + val.to_address();
                self.a = mmap.read(addr);
                self.cost = 3;
            }

            // 0xFE
            CP_d8(val) => {
                let prev = self.a;
                self.sub(val);
                self.a = prev;
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

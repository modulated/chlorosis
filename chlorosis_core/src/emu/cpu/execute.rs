use crate::{
    addition_register_pairs, decrement_register,
    emu::{memory::constants::*, Address, Byte, MemoryMap},
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
            // 0x30
            JR_NC_s8(signed) => {
                if !self.c_flag {
                    self.pc = Address(((self.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cost = 3;
                } else {
                    self.cost = 2;
                }
            }
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
            // 0x3F
            CCF => {
                self.c_flag = !self.c_flag;
                self.n_flag = false;
                self.h_flag = false;
                self.cost = 1;
            }
            // Row 3

            // Row 4
            // 0x40
            LD_B_B => {
                // self.b = self.b;
                self.cost = 1;
            }
            // 0x41
            LD_B_C => {
                self.b = self.c;
                self.cost = 1;
            }
            // 0x42
            LD_B_D => {
                self.b = self.d;
                self.cost = 1;
            }
            // 0x43
            LD_B_E => {
                self.b = self.e;
                self.cost = 1;
            }
            // 0x44
            LD_B_H => {
                self.b = self.h;
                self.cost = 1;
            }
            // 0x45
            LD_B_L => {
                self.b = self.l;
                self.cost = 1;
            }
            // 0x46
            LD_B_aHL => {
                self.b = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x47
            LD_B_A => {
                self.b = self.a;
                self.cost = 1;
            }
            // 0x48
            LD_C_B => {
                self.c = self.b;
                self.cost = 1;
            }
            // 0x49
            LD_C_C => {
                // self.c = self.c;
                self.cost = 1;
            }
            // 0x4A
            LD_C_D => {
                self.c = self.d;
                self.cost = 1;
            }
            // 0x4B
            LD_C_E => {
                self.c = self.e;
                self.cost = 1;
            }
            // 0x4C
            LD_C_H => {
                self.c = self.h;
                self.cost = 1;
            }
            // 0x4D
            LD_C_L => {
                self.c = self.l;
                self.cost = 1;
            }
            // 0x4E
            LD_C_aHL => {
                self.c = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x4F
            LD_C_A => {
                self.c = self.a;
                self.cost = 1;
            }
            // Row 4

            // Row 5
            // 0x50
            LD_D_B => {
                self.d = self.b;
                self.cost = 1;
            }
            // 0x51
            LD_D_C => {
                self.d = self.c;
                self.cost = 1;
            }
            // 0x52
            LD_D_D => {
                // self.d = self.d;
                self.cost = 1;
            }
            // 0x53
            LD_D_E => {
                self.d = self.e;
                self.cost = 1;
            }
            // 0x54
            LD_D_H => {
                self.d = self.h;
                self.cost = 1;
            }
            // 0x55
            LD_D_L => {
                self.d = self.l;
                self.cost = 1;
            }
            // 0x56
            LD_D_aHL => {
                self.d = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x57
            LD_D_A => {
                self.d = self.a;
                self.cost = 1;
            }
            // 0x58
            LD_E_B => {
                self.e = self.b;
                self.cost = 1;
            }
            // 0x59
            LD_E_C => {
                self.e = self.c;
                self.cost = 1;
            }
            // 0x5A
            LD_E_D => {
                self.e = self.d;
                self.cost = 1;
            }
            // 0x5B
            LD_E_E => {
                // self.e = self.e;
                self.cost = 1;
            }
            // 0x5C
            LD_E_H => {
                self.e = self.h;
                self.cost = 1;
            }
            // 0x5D
            LD_E_L => {
                self.e = self.l;
                self.cost = 1;
            }
            // 0x5E
            LD_E_aHL => {
                self.e = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x5F
            LD_E_A => {
                self.e = self.a;
                self.cost = 1;
            }
            // Row 5

            // Row 6
            // 0x60
            LD_H_B => {
                self.h = self.b;
                self.cost = 1;
            }
            // 0x61
            LD_H_C => {
                self.h = self.c;
                self.cost = 1;
            }
            // 0x62
            LD_H_D => {
                self.h = self.d;
                self.cost = 1;
            }
            // 0x63
            LD_H_E => {
                self.h = self.e;
                self.cost = 1;
            }
            // 0x64
            LD_H_H => {
                // self.h = self.h;
                self.cost = 1;
            }
            // 0x65
            LD_H_L => {
                self.h = self.l;
                self.cost = 1;
            }
            // 0x66
            LD_H_aHL => {
                self.h = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x67
            LD_H_A => {
                self.h = self.a;
                self.cost = 1;
            }
            // 0x68
            LD_L_B => {
                self.l = self.b;
                self.cost = 1;
            }
            // 0x69
            LD_L_C => {
                self.l = self.c;
                self.cost = 1;
            }
            // 0x6A
            LD_L_D => {
                self.l = self.d;
                self.cost = 1;
            }
            // 0x6B
            LD_L_E => {
                self.l = self.e;
                self.cost = 1;
            }
            // 0x6C
            LD_L_H => {
                self.l = self.h;
                self.cost = 1;
            }
            // 0x6D
            LD_L_L => {
                // self.l = self.l;
                self.cost = 1;
            }
            // 0x6E
            LD_L_aHL => {
                self.l = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x6F
            LD_L_A => {
                self.l = self.a;
                self.cost = 1;
            }
            // Row 6

            // Row 7
            // 0x70
            LD_aHL_B => {
                mmap.write(self.read_hl(), self.b);
                self.cost = 2;
            }
            // 0x71
            LD_aHL_C => {
                mmap.write(self.read_hl(), self.c);
                self.cost = 2;
            }
            // 0x72
            LD_aHL_D => {
                mmap.write(self.read_hl(), self.d);
                self.cost = 2;
            }
            // 0x73
            LD_aHL_E => {
                mmap.write(self.read_hl(), self.e);
                self.cost = 2;
            }
            // 0x74
            LD_aHL_H => {
                mmap.write(self.read_hl(), self.h);
                self.cost = 2;
            }
            // 0x75
            LD_aHL_L => {
                mmap.write(self.read_hl(), self.l);
                self.cost = 2;
            }
            // 0x76
            HALT => {
                // TODO: implement
                // STOP system clock
                // Cancelled by interrupt or reset
                // if interrupt master enable set PC is pushed to stack and jump to interrupt address
                self.cost = 1;
                unimplemented!();
            }
            // 0x77
            LD_aHL_A => {
                mmap.write(self.read_hl(), self.a);
                self.cost = 2;
            }
            // 0x78
            LD_A_B => {
                self.a = self.b;
                self.cost = 1;
            }
            // 0x79
            LD_A_C => {
                self.a = self.c;
                self.cost = 1;
            }
            // 0x7A
            LD_A_D => {
                self.a = self.d;
                self.cost = 1;
            }
            // 0x7B
            LD_A_E => {
                self.a = self.e;
                self.cost = 1;
            }
            // 0x7C
            LD_A_H => {
                self.a = self.h;
                self.cost = 1;
            }
            // 0x7D
            LD_A_L => {
                self.a = self.l;
                self.cost = 1;
            }
            // 0x7E
            LD_A_aHL => {
                self.a = mmap.read(self.read_hl());
                self.cost = 2;
            }
            // 0x7F
            LD_A_A => {
                // self.a = self.a;
                self.cost = 1;
            }
            // Row 7

            // Row 8
            // 0x80
            ADD_B => {
                self.add(self.b);
                self.cost = 1;
            }
            // 0x81
            ADD_C => {
                self.add(self.c);
                self.cost = 1;
            }
            // 0x82
            ADD_D => {
                self.add(self.d);
                self.cost = 1;
            }
            // 0x83
            ADD_E => {
                self.add(self.e);
                self.cost = 1;
            }
            // 0x84
            ADD_H => {
                self.add(self.h);
                self.cost = 1;
            }
            // 0x85
            ADD_L => {
                self.add(self.l);
                self.cost = 1;
            }
            // 0x86
            ADD_aHL => {
                let val = mmap.read(self.read_hl());
                self.add(val);
                self.cost = 2;
            }
            // 0x87
            ADD_A => {
                self.add(self.a);
                self.cost = 1;
            }
            // 0x88
            ADC_B => {
                self.adc(self.b);
                self.cost = 1;
            }
            // 0x89
            ADC_C => {
                self.adc(self.c);
                self.cost = 1;
            }
            // 0x8A
            ADC_D => {
                self.adc(self.d);
                self.cost = 1;
            }
            // 0x8B
            ADC_E => {
                self.adc(self.e);
                self.cost = 1;
            }
            // 0x8C
            ADC_H => {
                self.adc(self.h);
                self.cost = 1;
            }
            // 0x8D
            ADC_L => {
                self.adc(self.l);
                self.cost = 1;
            }
            // 0x8E
            ADC_aHL => {
                let val = mmap.read(self.read_hl());
                self.adc(val);
                self.cost = 2;
            }
            // 0x8F
            ADC_A => {
                self.adc(self.a);
                self.cost = 1;
            }
            // Row 8

            // Row 9
            // 0x90
            SUB_B => {
                self.sub(self.b);
                self.cost = 1;
            }
            // 0x91
            SUB_C => {
                self.sub(self.c);
                self.cost = 1;
            }
            // 0x92
            SUB_D => {
                self.sub(self.d);
                self.cost = 1;
            }
            // 0x93
            SUB_E => {
                self.sub(self.e);
                self.cost = 1;
            }
            // 0x94
            SUB_H => {
                self.sub(self.h);
                self.cost = 1;
            }
            // 0x95
            SUB_L => {
                self.sub(self.l);
                self.cost = 1;
            }
            // 0x96
            SUB_aHL => {
                let val = mmap.read(self.read_hl());
                self.sub(val);
                self.cost = 2;
            }
            // 0x97
            SUB_A => {
                self.sub(self.a);
                self.cost = 1;
            }
            // 0x98
            SBC_B => {
                self.sbc(self.b);
                self.cost = 1;
            }
            // 0x99
            SBC_C => {
                self.sbc(self.c);
                self.cost = 1;
            }
            // 0x9A
            SBC_D => {
                self.sbc(self.d);
                self.cost = 1;
            }
            // 0x9B
            SBC_E => {
                self.sbc(self.e);
                self.cost = 1;
            }
            // 0x9C
            SBC_H => {
                self.sbc(self.h);
                self.cost = 1;
            }
            // 0x9D
            SBC_L => {
                self.sbc(self.l);
                self.cost = 1;
            }
            // 0x9E
            SBC_aHL => {
                let val = mmap.read(self.read_hl());
                self.sbc(val);
                self.cost = 2;
            }
            // 0x9F
            SBC_A => {
                self.sbc(self.a);
                self.cost = 1;
            }
            // Row 9

            // Row A
            // 0xA0
            AND_B => {
                self.and(self.b);
                self.cost = 1;
            }
            // 0xA1
            AND_C => {
                self.and(self.c);
                self.cost = 1;
            }
            // 0xA2
            AND_D => {
                self.and(self.d);
                self.cost = 1;
            }
            // 0xA3
            AND_E => {
                self.and(self.e);
                self.cost = 1;
            }
            // 0xA4
            AND_H => {
                self.and(self.h);
                self.cost = 1;
            }
            // 0xA5
            AND_L => {
                self.and(self.l);
                self.cost = 1;
            }
            // 0xA6
            AND_aHL => {
                let val = mmap.read(self.read_hl());
                self.and(val);
                self.cost = 2;
            }
            // 0xA7
            AND_A => {
                self.and(self.a);
                self.cost = 1;
            }
            // 0xA8
            XOR_B => {
                self.xor(self.b);
                self.cost = 1;
            }
            // 0xA9
            XOR_C => {
                self.xor(self.c);
                self.cost = 1;
            }
            // 0xAA
            XOR_D => {
                self.xor(self.d);
                self.cost = 1;
            }
            // 0xAB
            XOR_E => {
                self.xor(self.e);
                self.cost = 1;
            }
            // 0xAC
            XOR_H => {
                self.xor(self.h);
                self.cost = 1;
            }
            // 0xAD
            XOR_L => {
                self.xor(self.l);
                self.cost = 1;
            }
            // 0xAE
            XOR_aHL => {
                let val = mmap.read(self.read_hl());
                self.xor(val);
                self.cost = 2;
            }
            // 0xAF
            XOR_A => {
                self.xor(self.a);
                self.cost = 1;
            }
            // Row A

            // Row B
            // 0xB0
            OR_B => {
                self.or(self.b);
                self.cost = 1;
            }
            // 0xB1
            OR_C => {
                self.or(self.c);
                self.cost = 1;
            }
            // 0xB2
            OR_D => {
                self.or(self.d);
                self.cost = 1;
            }
            // 0xB3
            OR_E => {
                self.or(self.e);
                self.cost = 1;
            }
            // 0xB4
            OR_H => {
                self.or(self.h);
                self.cost = 1;
            }
            // 0xB5
            OR_L => {
                self.or(self.l);
                self.cost = 1;
            }
            // 0xB6
            OR_aHL => {
                let val = mmap.read(self.read_hl());
                self.or(val);
                self.cost = 2;
            }
            // 0xB7
            OR_A => {
                self.or(self.a);
                self.cost = 1;
            }
            // 0xB8
            CP_B => {
                self.cp(self.b);
                self.cost = 1;
            }
            // 0xB9
            CP_C => {
                self.cp(self.c);
                self.cost = 1;
            }
            // 0xBA
            CP_D => {
                self.cp(self.d);
                self.cost = 1;
            }
            // 0xBB
            CP_E => {
                self.cp(self.e);
                self.cost = 1;
            }
            // 0xBC
            CP_H => {
                self.cp(self.h);
                self.cost = 1;
            }
            // 0xBD
            CP_L => {
                self.cp(self.l);
                self.cost = 1;
            }
            // 0xBE
            CP_aHL => {
                let val = mmap.read(self.read_hl());
                self.cp(val);
                self.cost = 2;
            }
            // 0xBF
            CP_A => {
                self.cp(self.a);
                self.cost = 1;
            }
            // Row B

            // Row C
            // 0xC0
            RET_NZ => {
                if !self.z_flag {
                    self.pc = self.pop_address(mmap);
                    self.cost = 5;
                } else {
                    self.cost = 2;
                }
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
            // 0xC4
            CALL_NZ_a16(addr) => {
                if !self.z_flag {
                    self.push_address(mmap, self.pc);
                    self.pc = addr;
                    self.cost = 6;
                } else {
                    self.cost = 3;
                }
            }
            // 0xC5
            PUSH_BC => {
                self.push_address(mmap, self.read_bc());
                self.cost = 4;
            }
            // 0xC6
            ADD_A_d8(val) => {
                self.check_carry_add_byte(self.a, val);
                self.check_half_carry_add_byte(self.a, val);
                self.n_flag = false;
                self.a = val;
                self.check_zero(self.a);
                self.cost = 2;
            }
            // 0xC7
            RST_0 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_0_ADDRESS.into();
                self.cost = 4;
            }
            // 0xC8
            RET_Z => {
                if self.z_flag {
                    self.pc = self.pop_address(mmap);
                    self.cost = 5
                } else {
                    self.cost = 2;
                }
            }
            // 0xC9
            RET => {
                self.pc = self.pop_address(mmap);
                self.cost = 4;
            }
            // 0xCA
            JP_Z_a16(addr) => {
                if self.z_flag {
                    self.pc = addr;
                    self.cost = 4;
                } else {
                    self.cost = 3;
                }
            }
            // 0xCB => Extended Instructions
            // 0xCC
            CALL_Z_a16(addr) => {
                if self.z_flag {
                    self.push_address(mmap, self.pc);
                    self.pc = addr;
                    self.cost = 6;
                } else {
                    self.cost = 3;
                }
            }
            // 0xCD
            CALL_a16(addr) => {
                self.push_address(mmap, self.pc);
                self.pc = addr;
                self.cost = 6;
            }
            // 0xCE
            ADC_A_d8(val) => {
                self.adc(val);
                self.cost = 2;
            }
            // 0xCF
            RST_1 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_1_ADDRESS.into();
                self.cost = 4;
            }
            // Row C

            // Row D
            // 0xD0
            RET_NC => {
                if !self.c_flag {
                    self.pc = self.pop_address(mmap);
                    self.cost = 5;
                } else {
                    self.cost = 2;
                }
            }
            // 0xD1
            POP_DE => {
                let addr = self.pop_address(mmap);
                self.write_de(addr);
                self.cost = 3;
            }
            // 0xD2
            JP_NC_a16(addr) => {
                if !self.c_flag {
                    self.pc = addr;
                    self.cost = 4;
                } else {
                    self.cost = 3;
                }
            }
            // 0xD3 = Illegal Instruction
            // 0xD4
            CALL_NC_a16(addr) => {
                if !self.c_flag {
                    self.push_address(mmap, self.pc);
                    self.pc = addr;
                    self.cost = 6;
                } else {
                    self.cost = 3;
                }
            }
            // 0xD5
            PUSH_DE => {
                self.push_address(mmap, self.read_de());
                self.cost = 4;
            }
            // 0xD6
            SUB_d8(val) => {
                self.sub(val);
                self.cost = 2;
            }
            // 0xD7
            RST_2 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_2_ADDRESS.into();
                self.cost = 4;
            }
            // 0xD8
            RET_C => {
                if self.c_flag {
                    self.sp = self.pop_address(mmap);
                    self.cost = 5;
                } else {
                    self.cost = 2;
                }
            }
            // 0xD9
            RETI => {
                // TODO: RETI instruction
                unimplemented!("RETI instruction not implemented");
                // toggle master interrupt enable flag
                // load PC from SP? or other
            }
            // 0xDA
            JP_C_a16(addr) => {
                if self.c_flag {
                    self.pc = addr;
                    self.cost = 4;
                } else {
                    self.cost = 3;
                }
            }
            // 0xDB = Illegal Instruction
            // 0xDC
            CALL_C_a16(addr) => {
                if self.c_flag {
                    self.push_address(mmap, self.pc);
                    self.pc = addr;
                    self.cost = 6;
                } else {
                    self.cost = 3;
                }
            }
            // 0xDD = Illegal Instruction
            // 0xDE
            SBC_A_d8(val) => {
                self.sbc(val);
                self.cost = 2;
            }
            // 0xDF
            RST_3 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_3_ADDRESS.into();
                self.cost = 4;
            }
            // Row D

            // Row E
            // 0xE0
            LD_a8_A(addr) => {
                let target = Address(0xFF00) + addr;
                mmap.write(target, self.a);
                self.cost = 3;
            }
            // 0xE1
            POP_HL => {
                let addr = self.pop_address(mmap);
                self.write_hl(addr);
                self.cost = 3;
            }
            // 0xE2
            LD_aC_A => {
                mmap.write(Address(0xFF00) + self.c.to_address(), self.a);
                self.cost = 2;
            }
            // 0xE3 = Illegal Instruction
            // 0xE4 = Illegal Instruction
            // 0xE5
            PUSH_HL => {
                self.push_address(mmap, self.read_hl());
                self.cost = 4;
            }
            // 0xE6
            AND_d8(val) => {
                self.and(val);
                self.cost = 2;
            }
            // 0xE7
            RST_4 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_4_ADDRESS.into();
                self.cost = 4;
            }
            // 0xE8
            ADD_SP_s8(signed) => {
                self.clear_flags();
                self.check_carry_signed_address(self.sp, signed); // TODO: this may introduce bugs? need to write test
                self.pc = Address(((self.sp.0 as i32) + (signed.0 as i32)) as u16);
            }
            // 0xE9
            JP_HL => {
                self.pc = self.read_hl();
                self.cost = 1;
            }
            // 0xEA
            LD_a16_A(addr) => {
                mmap.write(addr, self.a);
                self.cost = 4;
            }
            // 0xEB = Illegal Instruction
            // 0xEC = Illegal Instruction
            // 0xED = Illegal Instruction
            // 0xEE
            XOR_d8(val) => {
                self.xor(val);
                self.cost = 2;
            }
            // 0xEF
            RST_5 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_5_ADDRESS.into();
                self.cost = 4;
            }
            // Row E

            // Row F
            // 0xF0
            LD_A_a8(addr) => {
                let addr = Address(0xFF00) + addr;
                self.a = mmap.read(addr);
                self.cost = 3;
            }
            // 0xF1
            POP_AF => {
                let addr = self.pop_address(mmap);
                self.write_af(addr);
                self.cost = 3;
            }
            // 0xF2
            LD_A_aC => {
                self.a = mmap.read(Address(0xFF00) + self.c.to_address());
                self.cost = 2;
            }
            // 0xF3
            DI => {
                self.interupt_master_enable = false;
                self.cost = 1;
            }
            // 0xF4 = Illegal Instruction
            // 0xF5
            PUSH_AF => {
                self.push_address(mmap, self.read_af());
                self.cost = 4;
            }
            // 0xF6
            OR_d8(val) => {
                self.or(val);
                self.cost = 2;
            }
            // 0xF7
            RST_6 => {
                self.push_address(mmap, self.pc);
                self.pc = RST_6_ADDRESS.into();
                self.cost = 4;
            }
            // 0xF8
            LD_HL_SP_s8(signed) => {
                let addr = Address(((self.sp.0 as i32) + (signed.0 as i32)) as u16);
                self.clear_flags();
                self.check_carry_signed_address(self.sp, signed);
                self.write_hl(addr);
                self.cost = 3;
            }
            // 0xF9
            LD_SP_HL => {
                self.sp = self.read_hl();
                self.cost = 2;
            }
            // 0xFA
            LD_A_a16(addr) => {
                self.a = mmap.read(addr);
                self.cost = 4;
            }
            // 0xFB
            EI => {
                self.interupt_master_enable = true;
                self.cost = 1;
            }
            // 0xFC = Illegal Instruction
            // 0xFD = Illegal Instruction
            // 0xFE
            CP_d8(val) => {
                let prev = self.a;
                self.sub(val);
                self.a = prev;
                self.cost = 2;
            }
            // 0xFF
            RST_7 => {
                println!("RST_7 => may indicate 0xFF bug");
                self.push_address(mmap, self.pc);
                self.pc = RST_5_ADDRESS.into();
                self.cost = 4;
            }
            // Row F

            // CB Extensions
            // Row 0
            // Row 1
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
                self.z_flag = !self.h.is_bit_set(7);
                self.n_flag = false;
                self.h_flag = true;
                self.cost = 2;
            }
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

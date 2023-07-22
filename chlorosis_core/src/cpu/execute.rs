use crate::{
    addition_register_pairs, constants::*, decrement_register, increment_register, Address, Byte,
    Device,
};

use super::opcodes::Opcode;

impl Device {
    pub fn execute(&mut self, op: Opcode) {
        use Opcode::*;
        assert!(self.cpu.cost == 0);
        match op {
            // Row 0
            // 0x00
            NOP => {
                self.cpu.cost = 1;
            }
            // 0x01
            LD_BC_d16(addr) => {
                self.cpu.write_bc(addr);
                self.cpu.cost = 3;
            }
            // 0x02
            LD_aBC_A => {
                let addr = self.cpu.read_bc();
                self.cpu.a = self.read(addr);
                self.cpu.cost = 2;
            }
            // 0x03
            INC_BC => {
                self.cpu.write_bc(self.cpu.read_bc() + 1);
                self.cpu.cost = 2;
            }
            // 0x04
            INC_B => {
                increment_register!(self, self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0x05
            DEC_B => {
                decrement_register!(self, self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0x06
            LD_B_d8(val) => {
                self.cpu.b = val;
                self.cpu.cost = 2;
            }
            // 0x07
            RLCA => {
                self.cpu.clear_flags();
                self.cpu.c_flag = self.cpu.a.is_bit_set(7);
                self.cpu.a = self.cpu.a << 1;
                self.cpu.a.write_bit(0, self.cpu.c_flag);
                self.cpu.cost = 1;
            }
            // 0x08
            LD_a16_SP(addr) => {
                let (h, l) = self.cpu.sp.split();
                self.write(addr, l);
                self.write(addr + 1, h);
                self.cpu.cost = 5;
            }
            // 0x09
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_BC => {
                let bc = self.cpu.read_bc();
                let hl = self.cpu.read_hl();
                addition_register_pairs!(self, bc, hl, (|x| { self.cpu.write_hl(x) }));
                self.cpu.cost = 2;
            }
            // 0x0A
            LD_A_aBC => {
                self.cpu.a = self.read(self.cpu.read_bc());
                self.cpu.cost = 2;
            }
            // 0x0B
            DEC_BC => {
                self.cpu.write_bc(self.cpu.read_bc() - 1);
                self.cpu.cost = 2;
            }
            // 0x0C
            INC_C => {
                increment_register!(self, self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0x0D
            DEC_C => {
                decrement_register!(self, self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0x0E
            LD_C_d8(val) => {
                self.cpu.c = val;
                self.cpu.cost = 2;
            }
            // 0x0F
            RRCA => {
                self.cpu.clear_flags();
                self.cpu.c_flag = self.cpu.a.is_bit_set(0);
                self.cpu.a = self.cpu.a >> 1;
                self.cpu.a.write_bit(7, self.cpu.c_flag);
                self.cpu.cost = 1;
            }
            // Row 0

            // Row 1
            // 0x10
            STOP(_) => {
                // TODO: Implement STOP operation
                // IF all IE flags reset AND input P10 to P13 are LOW
                // STOP SYSTEM CLOCK and OSCILLATOR CIRCUIT and LCD controller
                // Cancelled by RESET signal

                panic!("Unimplemented STOP");
            }
            // 0x11
            LD_DE_d16(addr) => {
                self.cpu.write_de(addr);
                self.cpu.cost = 3;
            }
            // 0x12
            LD_aDE_A => {
                let addr = self.cpu.read_de();
                self.write(addr, self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0x13
            INC_DE => {
                self.cpu.write_de(self.cpu.read_de() + 1);
                self.cpu.cost = 2;
            }
            // 0x14
            INC_D => {
                increment_register!(self, self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0x15
            DEC_D => {
                decrement_register!(self, self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0x16
            LD_D_d8(val) => {
                self.cpu.d = val;
                self.cpu.cost = 2;
            }
            // 0x17
            RLA => {
                let prev = self.cpu.c_flag;
                self.cpu.c_flag = self.cpu.a.is_bit_set(7);
                let mut val = self.cpu.a << 1;
                val.write_bit(0, prev);
                self.cpu.cost = 1;
            }
            // 0x18
            JR_s8(val) => {
                self.cpu.pc = Address(((self.cpu.pc.0) as i32 + val.0 as i32) as u16);
                self.cpu.cost = 3;
            }
            // 0x19
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_DE => {
                let hl = self.cpu.read_hl();
                let de = self.cpu.read_de();
                addition_register_pairs!(self, hl, de, (|x| { self.cpu.write_hl(x) }));
                self.cpu.cost = 2;
            }
            // 0x1A
            LD_A_aDE => {
                self.cpu.a = self.read(self.cpu.read_de());
                self.cpu.cost = 2;
            }
            // 0x1B
            DEC_DE => {
                self.cpu.write_de(self.cpu.read_de());
                self.cpu.cost = 2;
            }
            // 0x1C
            INC_E => {
                increment_register!(self, self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0x1D
            DEC_E => {
                decrement_register!(self, self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0x1E
            LD_E_d8(val) => {
                self.cpu.e = val;
                self.cpu.cost = 2;
            }
            // 0x1F
            RRA => {
                let prev = self.cpu.c_flag;
                self.cpu.c_flag = self.cpu.a.is_bit_set(0);
                let mut val = self.cpu.a >> 1;
                val.write_bit(7, prev);
                self.cpu.cost = 1;
            }
            // Row 1

            // Row 2
            // 0x20
            JR_NZ_s8(signed) => {
                if !self.cpu.z_flag {
                    self.cpu.pc = Address(((self.cpu.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cpu.cost = 3;
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0x21
            LD_HL_d16(addr) => {
                self.cpu.write_hl(addr);
                self.cpu.cost = 3;
            }
            // 0x22
            LD_aHL_inc_A => {
                let addr = self.cpu.read_hl();
                self.write(addr, self.cpu.a);
                self.cpu.write_hl(addr + 1);
                self.cpu.cost = 2;
            }
            // 0x23
            INC_HL => {
                self.cpu.write_hl(self.cpu.read_hl() + 1);
                self.cpu.cost = 2;
            }
            // 0x24
            INC_H => {
                increment_register!(self, self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0x25
            DEC_H => {
                decrement_register!(self, self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0x26
            LD_H_d8(val) => {
                self.cpu.h = val;
                self.cpu.cost = 2;
            }
            // 0x27
            DAA => {
                // TODO: implement BCD operation
                unimplemented!()
            }
            // 0x28
            JR_Z_s8(signed) => {
                if self.cpu.z_flag {
                    self.cpu.pc = Address(((self.cpu.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cpu.cost = 3;
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0x29
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_HL => {
                let hl = self.cpu.read_hl();
                addition_register_pairs!(self, hl, hl, (|x| { self.cpu.write_hl(x) }));
                self.cpu.cost = 2;
            }
            // 0x2A
            LD_A_aHL_inc => {
                self.cpu.a = self.read(self.cpu.read_hl());
                self.cpu.write_hl(self.cpu.read_hl() + 1);
                self.cpu.cost = 2;
            }
            // 0x2B
            DEC_HL => {
                self.cpu.write_hl(self.cpu.read_hl() - 1);
                self.cpu.cost = 2;
            }
            // 0x2C
            INC_L => {
                increment_register!(self, self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0x2D
            DEC_L => {
                decrement_register!(self, self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0x2E
            LD_L_d8(val) => {
                self.cpu.l = val;
                self.cpu.cost = 2;
            }
            // 0x2F
            CPL => {
                self.cpu.a = !self.cpu.a;
                self.cpu.n_flag = true;
                self.cpu.h_flag = true;
                self.cpu.cost = 1;
            }
            // Row 2

            // Row 3
            // 0x30
            JR_NC_s8(signed) => {
                if !self.cpu.c_flag {
                    self.cpu.pc = Address(((self.cpu.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cpu.cost = 3;
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0x31
            LD_SP_d16(val) => {
                self.cpu.sp = val;
                self.cpu.cost = 3;
            }
            // 0x32
            LD_aHL_dec_A => {
                let addr = self.cpu.read_hl();
                self.write(addr, self.cpu.a);
                self.cpu.write_hl(addr - 1);
                self.cpu.cost = 2;
            }
            // 0x33
            INC_SP => {
                self.cpu.sp += 1;
                self.cpu.cost = 2;
            }
            // 0x34
            INC_aHL => {
                let old_val = self.read(self.cpu.read_hl());
                let new_val = Byte(old_val.0.wrapping_add(1));
                self.write(self.cpu.read_hl(), new_val);
                self.cpu.check_zero(new_val);
                self.cpu.check_half_carry_add_byte(old_val, Byte(1));
                self.cpu.n_flag = false;
                self.cpu.cost = 3;
            }
            // 0x35
            DEC_aHL => {
                let old_val = self.read(self.cpu.read_hl());
                let new_val = Byte(old_val.0.wrapping_sub(1));
                self.write(self.cpu.read_hl(), new_val);
                self.cpu.check_zero(new_val);
                self.cpu.check_half_carry_sub_byte(old_val, Byte(1));
                self.cpu.n_flag = true;
                self.cpu.cost = 3;
            }
            // 0x36
            LD_aHL_d8(val) => {
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 3;
            }
            // 0x37
            SCF => {
                self.cpu.c_flag = true;
                self.cpu.n_flag = false;
                self.cpu.h_flag = false;
                self.cpu.cost = 1;
            }
            // 0x38
            JR_C_s8(signed) => {
                if self.cpu.c_flag {
                    self.cpu.pc = Address(((self.cpu.pc.0) as i32 + signed.0 as i32) as u16);
                    self.cpu.cost = 3;
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0x39
            #[allow(clippy::redundant_closure_call)]
            ADD_HL_SP => {
                let sp = self.cpu.sp;
                let hl = self.cpu.read_hl();
                addition_register_pairs!(self, sp, hl, (|x| { self.cpu.write_hl(x) }));
                self.cpu.cost = 2;
            }
            // 0x3A
            LD_A_aHL_dec => {
                self.cpu.a = self.read(self.cpu.read_hl());
                self.cpu.write_hl(self.cpu.read_hl() - 1);
                self.cpu.cost = 2;
            }
            // 0x3B
            DEC_SP => {
                self.cpu.sp -= 1;
                self.cpu.cost = 2;
            }
            // 0x3C
            INC_A => {
                increment_register!(self, self.cpu.a);
                self.cpu.cost = 1;
            }
            // 0x3D
            DEC_A => {
                decrement_register!(self, self.cpu.a);
                self.cpu.cost = 1;
            }
            // 0x3E
            LD_A_d8(val) => {
                self.cpu.a = val;
                self.cpu.cost = 2;
            }
            // 0x3F
            CCF => {
                self.cpu.c_flag = !self.cpu.c_flag;
                self.cpu.n_flag = false;
                self.cpu.h_flag = false;
                self.cpu.cost = 1;
            }
            // Row 3

            // Row 4
            // 0x40
            LD_B_B => {
                // self.cpu.b = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x41
            LD_B_C => {
                self.cpu.b = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x42
            LD_B_D => {
                self.cpu.b = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x43
            LD_B_E => {
                self.cpu.b = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x44
            LD_B_H => {
                self.cpu.b = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x45
            LD_B_L => {
                self.cpu.b = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x46
            LD_B_aHL => {
                self.cpu.b = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x47
            LD_B_A => {
                self.cpu.b = self.cpu.a;
                self.cpu.cost = 1;
            }
            // 0x48
            LD_C_B => {
                self.cpu.c = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x49
            LD_C_C => {
                // self.cpu.c = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x4A
            LD_C_D => {
                self.cpu.c = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x4B
            LD_C_E => {
                self.cpu.c = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x4C
            LD_C_H => {
                self.cpu.c = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x4D
            LD_C_L => {
                self.cpu.c = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x4E
            LD_C_aHL => {
                self.cpu.c = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x4F
            LD_C_A => {
                self.cpu.c = self.cpu.a;
                self.cpu.cost = 1;
            }
            // Row 4

            // Row 5
            // 0x50
            LD_D_B => {
                self.cpu.d = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x51
            LD_D_C => {
                self.cpu.d = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x52
            LD_D_D => {
                // self.cpu.d = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x53
            LD_D_E => {
                self.cpu.d = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x54
            LD_D_H => {
                self.cpu.d = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x55
            LD_D_L => {
                self.cpu.d = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x56
            LD_D_aHL => {
                self.cpu.d = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x57
            LD_D_A => {
                self.cpu.d = self.cpu.a;
                self.cpu.cost = 1;
            }
            // 0x58
            LD_E_B => {
                self.cpu.e = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x59
            LD_E_C => {
                self.cpu.e = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x5A
            LD_E_D => {
                self.cpu.e = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x5B
            LD_E_E => {
                // self.cpu.e = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x5C
            LD_E_H => {
                self.cpu.e = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x5D
            LD_E_L => {
                self.cpu.e = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x5E
            LD_E_aHL => {
                self.cpu.e = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x5F
            LD_E_A => {
                self.cpu.e = self.cpu.a;
                self.cpu.cost = 1;
            }
            // Row 5

            // Row 6
            // 0x60
            LD_H_B => {
                self.cpu.h = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x61
            LD_H_C => {
                self.cpu.h = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x62
            LD_H_D => {
                self.cpu.h = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x63
            LD_H_E => {
                self.cpu.h = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x64
            LD_H_H => {
                // self.cpu.h = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x65
            LD_H_L => {
                self.cpu.h = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x66
            LD_H_aHL => {
                self.cpu.h = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x67
            LD_H_A => {
                self.cpu.h = self.cpu.a;
                self.cpu.cost = 1;
            }
            // 0x68
            LD_L_B => {
                self.cpu.l = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x69
            LD_L_C => {
                self.cpu.l = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x6A
            LD_L_D => {
                self.cpu.l = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x6B
            LD_L_E => {
                self.cpu.l = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x6C
            LD_L_H => {
                self.cpu.l = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x6D
            LD_L_L => {
                // self.cpu.l = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x6E
            LD_L_aHL => {
                self.cpu.l = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x6F
            LD_L_A => {
                self.cpu.l = self.cpu.a;
                self.cpu.cost = 1;
            }
            // Row 6

            // Row 7
            // 0x70
            LD_aHL_B => {
                self.write(self.cpu.read_hl(), self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0x71
            LD_aHL_C => {
                self.write(self.cpu.read_hl(), self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0x72
            LD_aHL_D => {
                self.write(self.cpu.read_hl(), self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0x73
            LD_aHL_E => {
                self.write(self.cpu.read_hl(), self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0x74
            LD_aHL_H => {
                self.write(self.cpu.read_hl(), self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0x75
            LD_aHL_L => {
                self.write(self.cpu.read_hl(), self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0x76
            HALT => {
                // TODO: implement HALT
                // STOP system clock
                // Cancelled by interrupt or reset
                // if interrupt master enable set PC is pushed to stack and jump to interrupt address
                self.cpu.cost = 1;
                unimplemented!();
            }
            // 0x77
            LD_aHL_A => {
                self.write(self.cpu.read_hl(), self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0x78
            LD_A_B => {
                self.cpu.a = self.cpu.b;
                self.cpu.cost = 1;
            }
            // 0x79
            LD_A_C => {
                self.cpu.a = self.cpu.c;
                self.cpu.cost = 1;
            }
            // 0x7A
            LD_A_D => {
                self.cpu.a = self.cpu.d;
                self.cpu.cost = 1;
            }
            // 0x7B
            LD_A_E => {
                self.cpu.a = self.cpu.e;
                self.cpu.cost = 1;
            }
            // 0x7C
            LD_A_H => {
                self.cpu.a = self.cpu.h;
                self.cpu.cost = 1;
            }
            // 0x7D
            LD_A_L => {
                self.cpu.a = self.cpu.l;
                self.cpu.cost = 1;
            }
            // 0x7E
            LD_A_aHL => {
                self.cpu.a = self.read(self.cpu.read_hl());
                self.cpu.cost = 2;
            }
            // 0x7F
            LD_A_A => {
                // self.cpu.a = self.cpu.a;
                self.cpu.cost = 1;
            }
            // Row 7

            // Row 8
            // 0x80
            ADD_B => {
                self.cpu.add(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0x81
            ADD_C => {
                self.cpu.add(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0x82
            ADD_D => {
                self.cpu.add(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0x83
            ADD_E => {
                self.cpu.add(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0x84
            ADD_H => {
                self.cpu.add(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0x85
            ADD_L => {
                self.cpu.add(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0x86
            ADD_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.add(val);
                self.cpu.cost = 2;
            }
            // 0x87
            ADD_A => {
                self.cpu.add(self.cpu.a);
                self.cpu.cost = 1;
            }
            // 0x88
            ADC_B => {
                self.cpu.adc(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0x89
            ADC_C => {
                self.cpu.adc(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0x8A
            ADC_D => {
                self.cpu.adc(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0x8B
            ADC_E => {
                self.cpu.adc(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0x8C
            ADC_H => {
                self.cpu.adc(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0x8D
            ADC_L => {
                self.cpu.adc(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0x8E
            ADC_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.adc(val);
                self.cpu.cost = 2;
            }
            // 0x8F
            ADC_A => {
                self.cpu.adc(self.cpu.a);
                self.cpu.cost = 1;
            }
            // Row 8

            // Row 9
            // 0x90
            SUB_B => {
                self.cpu.sub(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0x91
            SUB_C => {
                self.cpu.sub(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0x92
            SUB_D => {
                self.cpu.sub(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0x93
            SUB_E => {
                self.cpu.sub(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0x94
            SUB_H => {
                self.cpu.sub(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0x95
            SUB_L => {
                self.cpu.sub(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0x96
            SUB_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.sub(val);
                self.cpu.cost = 2;
            }
            // 0x97
            SUB_A => {
                self.cpu.sub(self.cpu.a);
                self.cpu.cost = 1;
            }
            // 0x98
            SBC_B => {
                self.cpu.sbc(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0x99
            SBC_C => {
                self.cpu.sbc(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0x9A
            SBC_D => {
                self.cpu.sbc(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0x9B
            SBC_E => {
                self.cpu.sbc(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0x9C
            SBC_H => {
                self.cpu.sbc(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0x9D
            SBC_L => {
                self.cpu.sbc(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0x9E
            SBC_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.sbc(val);
                self.cpu.cost = 2;
            }
            // 0x9F
            SBC_A => {
                self.cpu.sbc(self.cpu.a);
                self.cpu.cost = 1;
            }
            // Row 9

            // Row A
            // 0xA0
            AND_B => {
                self.cpu.and(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0xA1
            AND_C => {
                self.cpu.and(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0xA2
            AND_D => {
                self.cpu.and(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0xA3
            AND_E => {
                self.cpu.and(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0xA4
            AND_H => {
                self.cpu.and(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0xA5
            AND_L => {
                self.cpu.and(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0xA6
            AND_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.and(val);
                self.cpu.cost = 2;
            }
            // 0xA7
            AND_A => {
                self.cpu.and(self.cpu.a);
                self.cpu.cost = 1;
            }
            // 0xA8
            XOR_B => {
                self.cpu.xor(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0xA9
            XOR_C => {
                self.cpu.xor(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0xAA
            XOR_D => {
                self.cpu.xor(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0xAB
            XOR_E => {
                self.cpu.xor(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0xAC
            XOR_H => {
                self.cpu.xor(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0xAD
            XOR_L => {
                self.cpu.xor(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0xAE
            XOR_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.xor(val);
                self.cpu.cost = 2;
            }
            // 0xAF
            XOR_A => {
                self.cpu.xor(self.cpu.a);
                self.cpu.cost = 1;
            }
            // Row A

            // Row B
            // 0xB0
            OR_B => {
                self.cpu.or(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0xB1
            OR_C => {
                self.cpu.or(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0xB2
            OR_D => {
                self.cpu.or(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0xB3
            OR_E => {
                self.cpu.or(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0xB4
            OR_H => {
                self.cpu.or(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0xB5
            OR_L => {
                self.cpu.or(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0xB6
            OR_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.or(val);
                self.cpu.cost = 2;
            }
            // 0xB7
            OR_A => {
                self.cpu.or(self.cpu.a);
                self.cpu.cost = 1;
            }
            // 0xB8
            CP_B => {
                self.cpu.cp(self.cpu.b);
                self.cpu.cost = 1;
            }
            // 0xB9
            CP_C => {
                self.cpu.cp(self.cpu.c);
                self.cpu.cost = 1;
            }
            // 0xBA
            CP_D => {
                self.cpu.cp(self.cpu.d);
                self.cpu.cost = 1;
            }
            // 0xBB
            CP_E => {
                self.cpu.cp(self.cpu.e);
                self.cpu.cost = 1;
            }
            // 0xBC
            CP_H => {
                self.cpu.cp(self.cpu.h);
                self.cpu.cost = 1;
            }
            // 0xBD
            CP_L => {
                self.cpu.cp(self.cpu.l);
                self.cpu.cost = 1;
            }
            // 0xBE
            CP_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.cp(val);
                self.cpu.cost = 2;
            }
            // 0xBF
            CP_A => {
                self.cpu.cp(self.cpu.a);
                self.cpu.cost = 1;
            }
            // Row B

            // Row C
            // 0xC0
            RET_NZ => {
                if !self.cpu.z_flag {
                    self.cpu.pc = self.pop_address();
                    self.cpu.cost = 5;
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0xC1
            POP_BC => {
                let addr = self.pop_address();
                self.cpu.write_bc(addr);
                self.cpu.cost = 3;
            }
            // 0xC2
            JP_NZ_a16(addr) => {
                if self.cpu.z_flag {
                    self.cpu.pc = addr;
                    self.cpu.cost = 4;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xC3
            JP_a16(addr) => {
                self.cpu.pc = addr;
                self.cpu.cost = 4;
            }
            // 0xC4
            CALL_NZ_a16(addr) => {
                if !self.cpu.z_flag {
                    self.push_address(self.cpu.pc);
                    self.cpu.pc = addr;
                    self.cpu.cost = 6;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xC5
            PUSH_BC => {
                self.push_address(self.cpu.read_bc());
                self.cpu.cost = 4;
            }
            // 0xC6
            ADD_A_d8(val) => {
                self.cpu.check_carry_add_byte(self.cpu.a, val);
                self.cpu.check_half_carry_add_byte(self.cpu.a, val);
                self.cpu.n_flag = false;
                self.cpu.a = val;
                self.cpu.check_zero(self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xC7
            RST_0 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_0_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // 0xC8
            RET_Z => {
                if self.cpu.z_flag {
                    self.cpu.pc = self.pop_address();
                    self.cpu.cost = 5
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0xC9
            RET => {
                self.cpu.pc = self.pop_address();
                self.cpu.cost = 4;
            }
            // 0xCA
            JP_Z_a16(addr) => {
                if self.cpu.z_flag {
                    self.cpu.pc = addr;
                    self.cpu.cost = 4;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xCB => Extended Instructions
            // 0xCC
            CALL_Z_a16(addr) => {
                if self.cpu.z_flag {
                    self.push_address(self.cpu.pc);
                    self.cpu.pc = addr;
                    self.cpu.cost = 6;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xCD
            CALL_a16(addr) => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = addr;
                self.cpu.cost = 6;
            }
            // 0xCE
            ADC_A_d8(val) => {
                self.cpu.adc(val);
                self.cpu.cost = 2;
            }
            // 0xCF
            RST_1 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_1_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // Row C

            // Row D
            // 0xD0
            RET_NC => {
                if !self.cpu.c_flag {
                    self.cpu.pc = self.pop_address();
                    self.cpu.cost = 5;
                } else {
                    self.cpu.cost = 2;
                }
            }
            // 0xD1
            POP_DE => {
                let addr = self.pop_address();
                self.cpu.write_de(addr);
                self.cpu.cost = 3;
            }
            // 0xD2
            JP_NC_a16(addr) => {
                if !self.cpu.c_flag {
                    self.cpu.pc = addr;
                    self.cpu.cost = 4;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xD3 = Illegal Instruction
            // 0xD4
            CALL_NC_a16(addr) => {
                if !self.cpu.c_flag {
                    self.push_address(self.cpu.pc);
                    self.cpu.pc = addr;
                    self.cpu.cost = 6;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xD5
            PUSH_DE => {
                self.push_address(self.cpu.read_de());
                self.cpu.cost = 4;
            }
            // 0xD6
            SUB_d8(val) => {
                self.cpu.sub(val);
                self.cpu.cost = 2;
            }
            // 0xD7
            RST_2 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_2_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // 0xD8
            RET_C => {
                if self.cpu.c_flag {
                    self.cpu.sp = self.pop_address();
                    self.cpu.cost = 5;
                } else {
                    self.cpu.cost = 2;
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
                if self.cpu.c_flag {
                    self.cpu.pc = addr;
                    self.cpu.cost = 4;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xDB = Illegal Instruction
            // 0xDC
            CALL_C_a16(addr) => {
                if self.cpu.c_flag {
                    self.push_address(self.cpu.pc);
                    self.cpu.pc = addr;
                    self.cpu.cost = 6;
                } else {
                    self.cpu.cost = 3;
                }
            }
            // 0xDD = Illegal Instruction
            // 0xDE
            SBC_A_d8(val) => {
                self.cpu.sbc(val);
                self.cpu.cost = 2;
            }
            // 0xDF
            RST_3 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_3_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // Row D

            // Row E
            // 0xE0
            LD_a8_A(addr) => {
                let target = Address(0xFF00) + addr;
                self.write(target, self.cpu.a);
                self.cpu.cost = 3;
            }
            // 0xE1
            POP_HL => {
                let addr = self.pop_address();
                self.cpu.write_hl(addr);
                self.cpu.cost = 3;
            }
            // 0xE2
            LD_aC_A => {
                self.write(Address(0xFF00) + self.cpu.c.to_address(), self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xE3 = Illegal Instruction
            // 0xE4 = Illegal Instruction
            // 0xE5
            PUSH_HL => {
                self.push_address(self.cpu.read_hl());
                self.cpu.cost = 4;
            }
            // 0xE6
            AND_d8(val) => {
                self.cpu.and(val);
                self.cpu.cost = 2;
            }
            // 0xE7
            RST_4 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_4_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // 0xE8
            ADD_SP_s8(signed) => {
                self.cpu.clear_flags();
                self.cpu.check_carry_signed_address(self.cpu.sp, signed);
                self.cpu.pc = Address(((self.cpu.sp.0 as i32) + (signed.0 as i32)) as u16);
            }
            // 0xE9
            JP_HL => {
                self.cpu.pc = self.cpu.read_hl();
                self.cpu.cost = 1;
            }
            // 0xEA
            LD_a16_A(addr) => {
                self.write(addr, self.cpu.a);
                self.cpu.cost = 4;
            }
            // 0xEB = Illegal Instruction
            // 0xEC = Illegal Instruction
            // 0xED = Illegal Instruction
            // 0xEE
            XOR_d8(val) => {
                self.cpu.xor(val);
                self.cpu.cost = 2;
            }
            // 0xEF
            RST_5 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_5_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // Row E

            // Row F
            // 0xF0
            LD_A_a8(addr) => {
                let addr = Address(0xFF00) + addr;
                self.cpu.a = self.read(addr);
                self.cpu.cost = 3;
            }
            // 0xF1
            POP_AF => {
                let addr = self.pop_address();
                self.cpu.write_af(addr);
                self.cpu.cost = 3;
            }
            // 0xF2
            LD_A_aC => {
                self.cpu.a = self.read(Address(0xFF00) + self.cpu.c.to_address());
                self.cpu.cost = 2;
            }
            // 0xF3
            DI => {
                self.cpu.interupt_master_enable = false;
                self.cpu.cost = 1;
            }
            // 0xF4 = Illegal Instruction
            // 0xF5
            PUSH_AF => {
                self.push_address(self.cpu.read_af());
                self.cpu.cost = 4;
            }
            // 0xF6
            OR_d8(val) => {
                self.cpu.or(val);
                self.cpu.cost = 2;
            }
            // 0xF7
            RST_6 => {
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_6_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // 0xF8
            LD_HL_SP_s8(signed) => {
                let addr = Address(((self.cpu.sp.0 as i32) + (signed.0 as i32)) as u16);
                self.cpu.clear_flags();
                self.cpu.check_carry_signed_address(self.cpu.sp, signed);
                self.cpu.write_hl(addr);
                self.cpu.cost = 3;
            }
            // 0xF9
            LD_SP_HL => {
                self.cpu.sp = self.cpu.read_hl();
                self.cpu.cost = 2;
            }
            // 0xFA
            LD_A_a16(addr) => {
                self.cpu.a = self.read(addr);
                self.cpu.cost = 4;
            }
            // 0xFB
            EI => {
                self.cpu.interupt_master_enable = true;
                self.cpu.cost = 1;
            }
            // 0xFC = Illegal Instruction
            // 0xFD = Illegal Instruction
            // 0xFE
            CP_d8(val) => {
                let prev = self.cpu.a;
                self.cpu.sub(val);
                self.cpu.a = prev;
                self.cpu.cost = 2;
            }
            // 0xFF
            RST_7 => {
                println!("RST_7 => may indicate 0xFF bug");
                self.push_address(self.cpu.pc);
                self.cpu.pc = RST_7_ADDRESS.into();
                self.cpu.cost = 4;
            }
            // Row F

            // CB Extensions

            // Row 0
            // 0xCB00
            RLC_B => {
                self.cpu.b = self.cpu.rlc(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB01
            RLC_C => {
                self.cpu.c = self.cpu.rlc(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB02
            RLC_D => {
                self.cpu.d = self.cpu.rlc(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB03
            RLC_E => {
                self.cpu.e = self.cpu.rlc(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB04
            RLC_H => {
                self.cpu.h = self.cpu.rlc(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB05
            RLC_L => {
                self.cpu.l = self.cpu.rlc(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB06
            RLC_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.rlc(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB07
            RLC_A => {
                self.cpu.a = self.cpu.rlc(self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB08
            RRC_B => {
                self.cpu.b = self.cpu.rrc(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB09
            RRC_C => {
                self.cpu.c = self.cpu.rrc(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB0A
            RRC_D => {
                self.cpu.d = self.cpu.rrc(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB0B
            RRC_E => {
                self.cpu.e = self.cpu.rrc(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB0C
            RRC_H => {
                self.cpu.h = self.cpu.rrc(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB0D
            RRC_L => {
                self.cpu.l = self.cpu.rrc(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB0E
            RRC_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.rrc(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB0F
            RRC_A => {
                self.cpu.b = self.cpu.rrc(self.cpu.b);
                self.cpu.cost = 2;
            }
            // Row 0

            // Row 1
            // 0xCB10
            RL_B => {
                self.cpu.b = self.cpu.rl(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB11
            RL_C => {
                self.cpu.c = self.cpu.rl(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB12
            RL_D => {
                self.cpu.d = self.cpu.rl(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB13
            RL_E => {
                self.cpu.e = self.cpu.rl(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB14
            RL_H => {
                self.cpu.h = self.cpu.rl(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB15
            RL_L => {
                self.cpu.l = self.cpu.rl(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB16
            RL_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.rl(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB17
            RL_A => {
                self.cpu.a = self.cpu.rl(self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB18
            RR_B => {
                self.cpu.b = self.cpu.rr(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB19
            RR_C => {
                self.cpu.c = self.cpu.rr(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB1A
            RR_D => {
                self.cpu.d = self.cpu.rr(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB1B
            RR_E => {
                self.cpu.e = self.cpu.rr(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB1C
            RR_H => {
                self.cpu.h = self.cpu.rr(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB1D
            RR_L => {
                self.cpu.l = self.cpu.rr(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB1E
            RR_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.rr(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB1F
            RR_A => {
                self.cpu.b = self.cpu.rr(self.cpu.b);
                self.cpu.cost = 2;
            }
            // Row 1

            // Row 2
            // 0xCB20
            SLA_B => {
                self.cpu.b = self.cpu.sla(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB21
            SLA_C => {
                self.cpu.c = self.cpu.sla(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB22
            SLA_D => {
                self.cpu.d = self.cpu.sla(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB23
            SLA_E => {
                self.cpu.e = self.cpu.sla(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB24
            SLA_H => {
                self.cpu.h = self.cpu.sla(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB25
            SLA_L => {
                self.cpu.l = self.cpu.sla(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB26
            SLA_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.sla(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB27
            SLA_A => {
                self.cpu.a = self.cpu.sla(self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB28
            SRA_B => {
                self.cpu.b = self.cpu.sra(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB29
            SRA_C => {
                self.cpu.c = self.cpu.sra(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB2A
            SRA_D => {
                self.cpu.d = self.cpu.sra(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB2B
            SRA_E => {
                self.cpu.e = self.cpu.sra(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB2C
            SRA_H => {
                self.cpu.h = self.cpu.sra(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB2D
            SRA_L => {
                self.cpu.l = self.cpu.sra(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB2E
            SRA_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.sra(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB2F
            SRA_A => {
                self.cpu.b = self.cpu.sra(self.cpu.b);
                self.cpu.cost = 2;
            }
            // Row 2

            // Row 3
            // 0xCB30
            SWAP_B => {
                self.cpu.b = self.cpu.swap(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB31
            SWAP_C => {
                self.cpu.c = self.cpu.swap(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB32
            SWAP_D => {
                self.cpu.d = self.cpu.swap(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB33
            SWAP_E => {
                self.cpu.e = self.cpu.swap(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB34
            SWAP_H => {
                self.cpu.h = self.cpu.swap(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB35
            SWAP_L => {
                self.cpu.l = self.cpu.swap(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB36
            SWAP_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.swap(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB37
            SWAP_A => {
                self.cpu.a = self.cpu.swap(self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB38
            SRL_B => {
                self.cpu.b = self.cpu.srl(self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB39
            SRL_C => {
                self.cpu.c = self.cpu.srl(self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB3A
            SRL_D => {
                self.cpu.d = self.cpu.srl(self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB3B
            SRL_E => {
                self.cpu.e = self.cpu.srl(self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB3C
            SRL_H => {
                self.cpu.h = self.cpu.srl(self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB3D
            SRL_L => {
                self.cpu.l = self.cpu.srl(self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB3E
            SRL_aHL => {
                let val = self.read(self.cpu.read_hl());
                let val = self.cpu.srl(val);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB3F
            SRL_A => {
                self.cpu.b = self.cpu.srl(self.cpu.b);
                self.cpu.cost = 2;
            }
            // Row 3

            // Row 4
            // 0xCB40
            BIT_0_B => {
                self.cpu.bit(0, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB41
            BIT_0_C => {
                self.cpu.bit(0, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB42
            BIT_0_D => {
                self.cpu.bit(0, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB43
            BIT_0_E => {
                self.cpu.bit(0, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB44
            BIT_0_H => {
                self.cpu.bit(0, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB45
            BIT_0_L => {
                self.cpu.bit(0, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB46
            BIT_0_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(0, val);
                self.cpu.cost = 3;
            }
            // 0xCB47
            BIT_0_A => {
                self.cpu.bit(0, self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB48
            BIT_1_B => {
                self.cpu.bit(1, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB49
            BIT_1_C => {
                self.cpu.bit(1, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB4A
            BIT_1_D => {
                self.cpu.bit(1, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB4B
            BIT_1_E => {
                self.cpu.bit(1, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB4C
            BIT_1_H => {
                self.cpu.bit(1, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB4D
            BIT_1_L => {
                self.cpu.bit(1, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB4E
            BIT_1_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(1, val);
                self.cpu.cost = 3;
            }
            // 0xCB4F
            BIT_1_A => {
                self.cpu.bit(1, self.cpu.a);
                self.cpu.cost = 2;
            }
            // Row 4

            // Row 5
            // 0xCB50
            BIT_2_B => {
                self.cpu.bit(2, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB51
            BIT_2_C => {
                self.cpu.bit(2, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB52
            BIT_2_D => {
                self.cpu.bit(2, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB53
            BIT_2_E => {
                self.cpu.bit(2, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB54
            BIT_2_H => {
                self.cpu.bit(2, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB55
            BIT_2_L => {
                self.cpu.bit(2, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB56
            BIT_2_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(2, val);
                self.cpu.cost = 3;
            }
            // 0xCB57
            BIT_2_A => {
                self.cpu.bit(2, self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB58
            BIT_3_B => {
                self.cpu.bit(3, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB59
            BIT_3_C => {
                self.cpu.bit(3, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB5A
            BIT_3_D => {
                self.cpu.bit(3, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB5B
            BIT_3_E => {
                self.cpu.bit(3, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB5C
            BIT_3_H => {
                self.cpu.bit(3, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB5D
            BIT_3_L => {
                self.cpu.bit(3, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB5E
            BIT_3_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(3, val);
                self.cpu.cost = 3;
            }
            // 0xCB5F
            BIT_3_A => {
                self.cpu.bit(3, self.cpu.a);
                self.cpu.cost = 2;
            }
            // Row 5

            // Row 6
            // 0xCB60
            BIT_4_B => {
                self.cpu.bit(4, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB61
            BIT_4_C => {
                self.cpu.bit(4, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB62
            BIT_4_D => {
                self.cpu.bit(4, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB63
            BIT_4_E => {
                self.cpu.bit(4, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB64
            BIT_4_H => {
                self.cpu.bit(4, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB65
            BIT_4_L => {
                self.cpu.bit(4, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB66
            BIT_4_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(4, val);
                self.cpu.cost = 3;
            }
            // 0xCB67
            BIT_4_A => {
                self.cpu.bit(4, self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB68
            BIT_5_B => {
                self.cpu.bit(5, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB69
            BIT_5_C => {
                self.cpu.bit(5, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB6A
            BIT_5_D => {
                self.cpu.bit(5, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB6B
            BIT_5_E => {
                self.cpu.bit(5, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB6C
            BIT_5_H => {
                self.cpu.bit(5, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB6D
            BIT_5_L => {
                self.cpu.bit(5, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB6E
            BIT_5_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(5, val);
                self.cpu.cost = 3;
            }
            // 0xCB6F
            BIT_5_A => {
                self.cpu.bit(5, self.cpu.a);
                self.cpu.cost = 2;
            }
            // Row 6

            // Row 7
            // 0xCB70
            BIT_6_B => {
                self.cpu.bit(6, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB71
            BIT_6_C => {
                self.cpu.bit(6, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB72
            BIT_6_D => {
                self.cpu.bit(6, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB73
            BIT_6_E => {
                self.cpu.bit(6, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB74
            BIT_6_H => {
                self.cpu.bit(6, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB75
            BIT_6_L => {
                self.cpu.bit(6, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB76
            BIT_6_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(6, val);
                self.cpu.cost = 3;
            }
            // 0xCB77
            BIT_6_A => {
                self.cpu.bit(6, self.cpu.a);
                self.cpu.cost = 2;
            }
            // 0xCB78
            BIT_7_B => {
                self.cpu.bit(7, self.cpu.b);
                self.cpu.cost = 2;
            }
            // 0xCB79
            BIT_7_C => {
                self.cpu.bit(7, self.cpu.c);
                self.cpu.cost = 2;
            }
            // 0xCB7A
            BIT_7_D => {
                self.cpu.bit(7, self.cpu.d);
                self.cpu.cost = 2;
            }
            // 0xCB7B
            BIT_7_E => {
                self.cpu.bit(7, self.cpu.e);
                self.cpu.cost = 2;
            }
            // 0xCB7C
            BIT_7_H => {
                self.cpu.bit(7, self.cpu.h);
                self.cpu.cost = 2;
            }
            // 0xCB7D
            BIT_7_L => {
                self.cpu.bit(7, self.cpu.l);
                self.cpu.cost = 2;
            }
            // 0xCB7E
            BIT_7_aHL => {
                let val = self.read(self.cpu.read_hl());
                self.cpu.bit(7, val);
                self.cpu.cost = 3;
            }
            // 0xCB7F
            BIT_7_A => {
                self.cpu.bit(7, self.cpu.a);
                self.cpu.cost = 2;
            }
            // Row 7

            // Row 8
            // 0xCB80
            RES_0_B => {
                self.cpu.b.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB81
            RES_0_C => {
                self.cpu.c.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB82
            RES_0_D => {
                self.cpu.d.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB83
            RES_0_E => {
                self.cpu.e.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB84
            RES_0_H => {
                self.cpu.h.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB85
            RES_0_L => {
                self.cpu.l.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB86
            RES_0_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(0);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB87
            RES_0_A => {
                self.cpu.a.clear_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCB88
            RES_1_B => {
                self.cpu.b.clear_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCB89
            RES_1_C => {
                self.cpu.c.clear_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCB8A
            RES_1_D => {
                self.cpu.d.clear_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCB8B
            RES_1_E => {
                self.cpu.e.clear_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCB8C
            RES_1_H => {
                self.cpu.h.clear_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCB8D
            RES_1_L => {
                self.cpu.l.clear_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCB8E
            RES_1_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(1);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB8F
            RES_1_A => {
                self.cpu.a.clear_bit(1);
                self.cpu.cost = 2;
            }
            // Row 8

            // Row 9
            // 0xCB90
            RES_2_B => {
                self.cpu.b.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB91
            RES_2_C => {
                self.cpu.c.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB92
            RES_2_D => {
                self.cpu.d.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB93
            RES_2_E => {
                self.cpu.e.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB94
            RES_2_H => {
                self.cpu.h.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB95
            RES_2_L => {
                self.cpu.l.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB96
            RES_2_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(2);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB97
            RES_2_A => {
                self.cpu.a.clear_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCB98
            RES_3_B => {
                self.cpu.b.clear_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCB99
            RES_3_C => {
                self.cpu.c.clear_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCB9A
            RES_3_D => {
                self.cpu.d.clear_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCB9B
            RES_3_E => {
                self.cpu.e.clear_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCB9C
            RES_3_H => {
                self.cpu.h.clear_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCB9D
            RES_3_L => {
                self.cpu.l.clear_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCB9E
            RES_3_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(3);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCB9F
            RES_3_A => {
                self.cpu.a.clear_bit(3);
                self.cpu.cost = 2;
            }
            // Row 9

            // Row A
            // 0xCBA0
            RES_4_B => {
                self.cpu.b.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA1
            RES_4_C => {
                self.cpu.c.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA2
            RES_4_D => {
                self.cpu.d.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA3
            RES_4_E => {
                self.cpu.e.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA4
            RES_4_H => {
                self.cpu.h.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA5
            RES_4_L => {
                self.cpu.l.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA6
            RES_4_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(4);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBA7
            RES_4_A => {
                self.cpu.a.clear_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBA8
            RES_5_B => {
                self.cpu.b.clear_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBA9
            RES_5_C => {
                self.cpu.c.clear_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBAA
            RES_5_D => {
                self.cpu.d.clear_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBAB
            RES_5_E => {
                self.cpu.e.clear_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBAC
            RES_5_H => {
                self.cpu.h.clear_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBAD
            RES_5_L => {
                self.cpu.l.clear_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBAE
            RES_5_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(5);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBAF
            RES_5_A => {
                self.cpu.a.clear_bit(5);
                self.cpu.cost = 2;
            }
            // Row A

            // Row B
            // 0xCBB0
            RES_6_B => {
                self.cpu.b.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB1
            RES_6_C => {
                self.cpu.c.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB2
            RES_6_D => {
                self.cpu.d.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB3
            RES_6_E => {
                self.cpu.e.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB4
            RES_6_H => {
                self.cpu.h.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB5
            RES_6_L => {
                self.cpu.l.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB6
            RES_6_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(6);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBB7
            RES_6_A => {
                self.cpu.a.clear_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBB8
            RES_7_B => {
                self.cpu.b.clear_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBB9
            RES_7_C => {
                self.cpu.c.clear_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBBA
            RES_7_D => {
                self.cpu.d.clear_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBBB
            RES_7_E => {
                self.cpu.e.clear_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBBC
            RES_7_H => {
                self.cpu.h.clear_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBBD
            RES_7_L => {
                self.cpu.l.clear_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBBE
            RES_7_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.clear_bit(7);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBBF
            RES_7_A => {
                self.cpu.a.clear_bit(7);
                self.cpu.cost = 2;
            }
            // Row B

            // Row C
            // 0xCBC0
            SET_0_B => {
                self.cpu.b.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC1
            SET_0_C => {
                self.cpu.c.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC2
            SET_0_D => {
                self.cpu.d.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC3
            SET_0_E => {
                self.cpu.e.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC4
            SET_0_H => {
                self.cpu.h.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC5
            SET_0_L => {
                self.cpu.l.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC6
            SET_0_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(0);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBC7
            SET_0_A => {
                self.cpu.a.set_bit(0);
                self.cpu.cost = 2;
            }
            // 0xCBC8
            SET_1_B => {
                self.cpu.b.set_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCBC9
            SET_1_C => {
                self.cpu.c.set_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCBCA
            SET_1_D => {
                self.cpu.d.set_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCBCB
            SET_1_E => {
                self.cpu.e.set_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCBCC
            SET_1_H => {
                self.cpu.h.set_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCBCD
            SET_1_L => {
                self.cpu.l.set_bit(1);
                self.cpu.cost = 2;
            }
            // 0xCBCE
            SET_1_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(1);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBCF
            SET_1_A => {
                self.cpu.a.set_bit(1);
                self.cpu.cost = 2;
            }
            // Row C

            // Row D
            // 0xCBD0
            SET_2_B => {
                self.cpu.b.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD1
            SET_2_C => {
                self.cpu.c.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD2
            SET_2_D => {
                self.cpu.d.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD3
            SET_2_E => {
                self.cpu.e.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD4
            SET_2_H => {
                self.cpu.h.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD5
            SET_2_L => {
                self.cpu.l.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD6
            SET_2_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(2);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBD7
            SET_2_A => {
                self.cpu.a.set_bit(2);
                self.cpu.cost = 2;
            }
            // 0xCBD8
            SET_3_B => {
                self.cpu.b.set_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCBD9
            SET_3_C => {
                self.cpu.c.set_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCBDA
            SET_3_D => {
                self.cpu.d.set_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCBDB
            SET_3_E => {
                self.cpu.e.set_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCBDC
            SET_3_H => {
                self.cpu.h.set_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCBDD
            SET_3_L => {
                self.cpu.l.set_bit(3);
                self.cpu.cost = 2;
            }
            // 0xCBDE
            SET_3_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(3);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBDF
            SET_3_A => {
                self.cpu.a.set_bit(3);
                self.cpu.cost = 2;
            }
            // Row D

            // Row E
            // 0xCBE0
            SET_4_B => {
                self.cpu.b.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE1
            SET_4_C => {
                self.cpu.c.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE2
            SET_4_D => {
                self.cpu.d.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE3
            SET_4_E => {
                self.cpu.e.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE4
            SET_4_H => {
                self.cpu.h.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE5
            SET_4_L => {
                self.cpu.l.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE6
            SET_4_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(4);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBE7
            SET_4_A => {
                self.cpu.a.set_bit(4);
                self.cpu.cost = 2;
            }
            // 0xCBE8
            SET_5_B => {
                self.cpu.b.set_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBE9
            SET_5_C => {
                self.cpu.c.set_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBEA
            SET_5_D => {
                self.cpu.d.set_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBEB
            SET_5_E => {
                self.cpu.e.set_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBEC
            SET_5_H => {
                self.cpu.h.set_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBED
            SET_5_L => {
                self.cpu.l.set_bit(5);
                self.cpu.cost = 2;
            }
            // 0xCBEE
            SET_5_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(5);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBEF
            SET_5_A => {
                self.cpu.a.set_bit(5);
                self.cpu.cost = 2;
            }
            // Row E

            // ROw F
            // 0xCBF0
            SET_6_B => {
                self.cpu.b.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF1
            SET_6_C => {
                self.cpu.c.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF2
            SET_6_D => {
                self.cpu.d.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF3
            SET_6_E => {
                self.cpu.e.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF4
            SET_6_H => {
                self.cpu.h.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF5
            SET_6_L => {
                self.cpu.l.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF6
            SET_6_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(6);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBF7
            SET_6_A => {
                self.cpu.a.set_bit(6);
                self.cpu.cost = 2;
            }
            // 0xCBF8
            SET_7_B => {
                self.cpu.b.set_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBF9
            SET_7_C => {
                self.cpu.c.set_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBFA
            SET_7_D => {
                self.cpu.d.set_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBFB
            SET_7_E => {
                self.cpu.e.set_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBFC
            SET_7_H => {
                self.cpu.h.set_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBFD
            SET_7_L => {
                self.cpu.l.set_bit(7);
                self.cpu.cost = 2;
            }
            // 0xCBFE
            SET_7_aHL => {
                let mut val = self.read(self.cpu.read_hl());
                val.set_bit(7);
                self.write(self.cpu.read_hl(), val);
                self.cpu.cost = 4;
            }
            // 0xCBFF
            SET_7_A => {
                self.cpu.a.set_bit(7);
                self.cpu.cost = 2;
            } // Row F
              // End of CB
        }
        assert!(
            self.cpu.cost != 0,
            "Forgot to simulate instruction cycle cost"
        );
        self.cpu.cost -= 1;
    }
}

#[cfg(test)]
mod test {
    use crate::Byte;

    #[test]
    fn test_is_bit_set() {
        let b = Byte(0x9F);
        assert!(b.is_bit_set(7));
        assert!(!b.is_bit_set(6));
        assert!(b.is_bit_set(0));
    }
}

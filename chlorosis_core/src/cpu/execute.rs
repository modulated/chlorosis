use crate::{
    addition_register_pairs, constants::*, decrement_register, increment_register, Address, Byte, Device
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
                // TODO: Implement
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
                // TODO: implement
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
            // Row 1
            // 0xCB11
            RL_C => {
                let old_cf = self.cpu.c_flag;
                self.cpu.clear_flags();
                if self.cpu.c.is_bit_set(7) {
                    self.cpu.c_flag = true;
                }
                let mut val = self.cpu.c << 1;
                if old_cf {
                    val.set_bit(0);
                }
                if val.0 == 0 {
                    self.cpu.z_flag = true;
                }
                self.cpu.cost = 2;
            }
            // 0xCB7C
            BIT_7_H => {
                self.cpu.z_flag = !self.cpu.h.is_bit_set(7);
                self.cpu.n_flag = false;
                self.cpu.h_flag = true;
                self.cpu.cost = 2;
            }
        }
        assert!(self.cpu.cost != 0, "Forgot to simulate instruction cycle cost");
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

use crate::emu::{Address, Byte};

use super::CentralProcessor;

impl CentralProcessor {
    pub fn clear_flags(&mut self) {
        self.z_flag = false;
        self.n_flag = false;
        self.h_flag = false;
        self.c_flag = false;
    }

    pub fn check_zero(&mut self, val: Byte) {
        if val.0 == 0 {
            self.z_flag = true;
        } else {
            self.z_flag = false;
        }
    }

    pub fn sub(&mut self, val: Byte) {
        let dif = self.a.0.wrapping_sub(val.0);
        self.check_zero(Byte(dif));
        self.check_half_carry_sub_byte(self.a, val);
        self.check_carry_sub_byte(self.a, val);
        self.n_flag = true;
    }

    pub fn xor(&mut self, val: Byte) {
        self.a ^= val;
        self.clear_flags();
        self.check_zero(self.a);
    }

    // CHECK AND SET C AND H FLAGS
    pub fn check_carry_add_byte(&mut self, a: Byte, b: Byte) {
        let res = a.0.wrapping_add(b.0);
        if (res < a.0) || (res < b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }

    pub fn check_carry_add_address(&mut self, a: Address, b: Address) {
        let res = a.0.wrapping_add(b.0);
        if (res < a.0) || (res < b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }

    pub fn check_half_carry_add_byte(&mut self, a: Byte, b: Byte) {
        if (((a.0 & 0xF).wrapping_add(b.0 & 0xF)) & 0x10) == 0x10 {
            self.h_flag = true;
        } else {
            self.h_flag = false;
        }
    }

    pub fn check_half_carry_add_address(&mut self, a: Address, b: Address) {
        if (((a.0 & 0xFFF).wrapping_add(b.0 & 0xFFF)) & 0x1000) == 0x1000 {
            self.h_flag = true;
        } else {
            self.h_flag = false;
        }
    }

    pub fn check_half_carry_sub_byte(&mut self, a: Byte, b: Byte) {
        if (((a.0 & 0xF).wrapping_sub(b.0 & 0xF)) & 0x10) == 0x10 {
            self.h_flag = true;
        } else {
            self.h_flag = false;
        }
    }

    pub fn check_carry_sub_byte(&mut self, a: Byte, b: Byte) {
        let res = a.0.wrapping_sub(b.0);
        if (res > a.0) || (res > b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }
}

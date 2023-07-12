use crate::{types::SignedByte, Address, Byte};

use super::CentralProcessor;

impl CentralProcessor {
    #[inline(always)]
    pub fn clear_flags(&mut self) {
        self.z_flag = false;
        self.n_flag = false;
        self.h_flag = false;
        self.c_flag = false;
    }

    #[inline(always)]
    pub fn check_zero(&mut self, val: Byte) {
        if val.0 == 0 {
            self.z_flag = true;
        } else {
            self.z_flag = false;
        }
    }

    #[inline(always)]
    pub fn add(&mut self, val: Byte) {
        let old = self.a;
        self.a = Byte(self.a.0.wrapping_add(val.0));
        self.check_zero(self.a);
        self.check_half_carry_sub_byte(old, val);
        self.check_carry_sub_byte(old, val);
        self.n_flag = false;
    }

    #[inline(always)]
    pub fn adc(&mut self, val: Byte) {
        let old = self.a;
        if self.c_flag {
            self.a = Byte(self.a.0.wrapping_add(val.0 + 1));
            self.check_half_carry_sub_byte(old, Byte(val.0 + 1));
            self.check_carry_sub_byte(old, Byte(val.0 + 1));
        } else {
            self.a = Byte(self.a.0.wrapping_add(val.0));
            self.check_half_carry_sub_byte(old, val);
            self.check_carry_sub_byte(old, val);
        }
        self.check_zero(self.a);
        self.n_flag = false;
    }

    #[inline(always)]
    pub fn sub(&mut self, val: Byte) {
        let old = self.a;
        self.a = Byte(self.a.0.wrapping_sub(val.0));
        self.check_zero(self.a);
        self.check_half_carry_sub_byte(old, val);
        self.check_carry_sub_byte(old, val);
        self.n_flag = true;
    }

    #[inline(always)]
    pub fn sbc(&mut self, val: Byte) {
        let old = self.a;
        if self.c_flag {
            self.a = Byte(self.a.0.wrapping_sub(val.0 + 1));
            self.check_half_carry_sub_byte(old, Byte(val.0 + 1));
            self.check_carry_sub_byte(old, Byte(val.0 + 1));
        } else {
            self.a = Byte(self.a.0.wrapping_sub(val.0));
            self.check_half_carry_sub_byte(old, val);
            self.check_carry_sub_byte(old, val);
        }
        self.check_zero(self.a);
        self.n_flag = true;
    }

    #[inline(always)]
    pub fn cp(&mut self, val: Byte) {
        let prev = self.a;
        self.sub(val);
        self.a = prev;
    }

    #[inline(always)]
    pub fn and(&mut self, val: Byte) {
        self.a &= val;
        self.clear_flags();
        self.h_flag = true;
        self.check_zero(self.a);
    }

    #[inline(always)]
    pub fn or(&mut self, val: Byte) {
        self.a |= val;
        self.clear_flags();
        self.check_zero(self.a);
    }

    #[inline(always)]
    pub fn xor(&mut self, val: Byte) {
        self.a ^= val;
        self.clear_flags();
        self.check_zero(self.a);
    }

    #[inline(always)]
    pub fn rlc(&mut self, val: Byte) -> Byte {
        let b7 = val.is_bit_set(7);
        let mut val = val << 1;
        self.check_zero(val);
        self.c_flag = b7;
        val.write_bit(0, b7);
        val
    }

    #[inline(always)]
    pub fn rrc(&mut self, val: Byte) -> Byte {
        let b0 = val.is_bit_set(0);
        let mut val = val >> 1;
        self.check_zero(val);
        self.c_flag = b0;
        val.write_bit(7, b0);
        val
    }

    #[inline(always)]
    pub fn rl(&mut self, val: Byte) -> Byte {
        let old_cf = self.c_flag;
        self.clear_flags();
        if val.is_bit_set(7) {
            self.c_flag = true;
        }
        let mut val = val << 1;
        if old_cf {
            val.set_bit(0);
        }
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn rr(&mut self, val: Byte) -> Byte {
        let old_cf = self.c_flag;
        self.clear_flags();
        if val.is_bit_set(0) {
            self.c_flag = true;
        }
        let mut val = val >> 1;
        if old_cf {
            val.set_bit(7);
        }
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn sla(&mut self, val: Byte) -> Byte {
        self.clear_flags();
        self.c_flag = val.is_bit_set(7);
        let val = val << 1;
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn sra(&mut self, val: Byte) -> Byte {
        self.clear_flags();
        self.c_flag = val.is_bit_set(0);
        let mut val = val >> 1;
        val.write_bit(7, self.c_flag);
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn srl(&mut self, val: Byte) -> Byte {
        self.clear_flags();
        self.c_flag = val.is_bit_set(0);
        let val = val >> 1;
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn swap(&mut self, val: Byte) -> Byte {
        self.clear_flags();
        let upper = val & 0xF0;
        let lower = val & 0x0F;
        let val = (upper >> 4) + (lower << 4);
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn bit(&mut self, pos: u8, val: Byte) {
        self.z_flag = !val.is_bit_set(pos);
        self.n_flag = false;
        self.h_flag = true;
    }

    #[inline(always)]
    pub fn check_carry_add_byte(&mut self, a: Byte, b: Byte) {
        let res = a.0.wrapping_add(b.0);
        if (res < a.0) || (res < b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }

    #[inline(always)]
    pub fn check_carry_add_address(&mut self, a: Address, b: Address) {
        let res = a.0.wrapping_add(b.0);
        if (res < a.0) || (res < b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }

    #[inline(always)]
    pub fn check_carry_sub_address(&mut self, a: Address, b: Address) {
        let res = a.0.wrapping_sub(b.0);
        if (res > a.0) || (res > b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }

    #[inline(always)]
    pub fn check_carry_signed_address(&mut self, a: Address, b: SignedByte) {
        if b.0 >= 0 {
            let b = Address(b.0 as u16);
            self.check_carry_add_address(a, b)
        } else {
            let b = Address(b.0.unsigned_abs() as u16);
            self.check_carry_sub_address(a, b)
        }
    }

    #[inline(always)]
    pub fn check_half_carry_add_byte(&mut self, a: Byte, b: Byte) {
        self.h_flag = (a.0 & 0xF) + (b.0 & 0xF) > 0xF;
    }

    #[inline(always)]
    pub fn check_half_carry_add_address(&mut self, a: Address, b: Address) {
        if (((a.0 & 0xFFF).wrapping_add(b.0 & 0xFFF)) & 0x1000) == 0x1000 {
            self.h_flag = true;
        } else {
            self.h_flag = false;
        }
    }

    #[inline(always)]
    pub fn check_half_carry_sub_byte(&mut self, a: Byte, b: Byte) {
        self.h_flag = (a.0 & 0xF) < (b.0 & 0xF);
    }

    #[inline(always)]
    pub fn check_carry_sub_byte(&mut self, a: Byte, b: Byte) {
        let res = a.0.wrapping_sub(b.0);
        if (res > a.0) || (res > b.0) {
            self.c_flag = true;
        } else {
            self.c_flag = false;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Byte, CentralProcessor};
    #[test]
    fn test_carry_add_byte() {
        let mut cpu = CentralProcessor::new();
        cpu.check_carry_add_byte(Byte(0x80), Byte(0x80));
        assert!(cpu.c_flag);
        cpu.check_carry_add_byte(Byte(0x0F), Byte(0x70));
        assert!(!cpu.c_flag);
    }

    #[test]
    fn test_half_carry_add_byte() {
        let mut cpu = CentralProcessor::new();
        cpu.check_half_carry_add_byte(Byte(0x08), Byte(0x08));
        assert!(cpu.h_flag);
        cpu.check_half_carry_add_byte(Byte(0x04), Byte(0x10));
        assert!(!cpu.h_flag);
        cpu.check_half_carry_add_byte(Byte(0x08), Byte(0x01));
        assert!(!cpu.h_flag);
    }

    #[test]
    fn test_half_carry_sub_byte() {
        let mut cpu = CentralProcessor::new();
        cpu.check_half_carry_sub_byte(Byte(0x01), Byte(0x00));
        assert!(!cpu.h_flag);
        cpu.check_half_carry_sub_byte(Byte(0x02), Byte(0x10));
        assert!(cpu.h_flag);
        cpu.check_half_carry_sub_byte(Byte(0x08), Byte(0x01));
        assert!(!cpu.h_flag);
    }

    #[test]
    fn test_swap() {
        let mut cpu = CentralProcessor::new();
        assert_eq!(cpu.swap(Byte(0b1010_0101)), Byte(0b0101_1010));
        assert_eq!(cpu.swap(Byte(0b0000_1111)), Byte(0b1111_0000));
    }
}

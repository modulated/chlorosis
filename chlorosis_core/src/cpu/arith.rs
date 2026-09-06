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
        self.z_flag = val.0 == 0;
    }

    #[inline(always)]
    pub fn add(&mut self, val: Byte) {
        // ADD sets its flags from an *addition* - this used the subtract
        // helpers, so every ADD/ADC produced wrong carry and half-carry.
        let a = self.a.0;
        self.h_flag = (a & 0xF) + (val.0 & 0xF) > 0xF;
        self.c_flag = u16::from(a) + u16::from(val.0) > 0xFF;
        self.a = Byte(a.wrapping_add(val.0));
        self.check_zero(self.a);
        self.n_flag = false;
    }

    #[inline(always)]
    pub fn adc(&mut self, val: Byte) {
        let a = self.a.0;
        let carry = u8::from(self.c_flag);
        // Fold the carry into the nibble/byte sums directly. The old code did
        // `val + 1`, which both used the subtract helpers and overflowed when
        // val was 0xFF.
        self.h_flag = (a & 0xF) + (val.0 & 0xF) + carry > 0xF;
        self.c_flag = u16::from(a) + u16::from(val.0) + u16::from(carry) > 0xFF;
        self.a = Byte(a.wrapping_add(val.0).wrapping_add(carry));
        self.check_zero(self.a);
        self.n_flag = false;
    }

    #[inline(always)]
    pub fn sub(&mut self, val: Byte) {
        let a = self.a.0;
        self.h_flag = (a & 0xF) < (val.0 & 0xF);
        self.c_flag = a < val.0;
        self.a = Byte(a.wrapping_sub(val.0));
        self.check_zero(self.a);
        self.n_flag = true;
    }

    #[inline(always)]
    pub fn sbc(&mut self, val: Byte) {
        let a = self.a.0;
        let carry = u8::from(self.c_flag);
        self.h_flag = (a & 0xF) < (val.0 & 0xF) + carry;
        self.c_flag = u16::from(a) < u16::from(val.0) + u16::from(carry);
        self.a = Byte(a.wrapping_sub(val.0).wrapping_sub(carry));
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
        self.clear_flags(); // N and H must be cleared, not left as they were
        let b7 = val.is_bit_set(7);
        self.c_flag = b7;
        let mut val = val << 1;
        val.write_bit(0, b7);
        self.check_zero(val);
        val
    }

    #[inline(always)]
    pub fn rrc(&mut self, val: Byte) -> Byte {
        self.clear_flags();
        let b0 = val.is_bit_set(0);
        self.c_flag = b0;
        let mut val = val >> 1;
        val.write_bit(7, b0);
        self.check_zero(val);
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
        // Arithmetic shift right preserves the sign bit; this had set bit 7 to
        // the carry (the old bit 0) instead of keeping the old bit 7.
        self.clear_flags();
        let sign = val.is_bit_set(7);
        self.c_flag = val.is_bit_set(0);
        let mut val = val >> 1;
        val.write_bit(7, sign);
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
        self.c_flag = (res < a.0) || (res < b.0);
    }

    #[inline(always)]
    pub fn check_carry_add_address(&mut self, a: Address, b: Address) {
        let res = a.0.wrapping_add(b.0);
        self.c_flag = (res < a.0) || (res < b.0)
    }

    #[inline(always)]
    pub fn check_carry_sub_address(&mut self, a: Address, b: Address) {
        // A subtraction borrows exactly when the minuend is smaller. The old
        // `res > b` term also flagged cases like 0xFF - 0x01 that do not borrow.
        self.c_flag = a.0 < b.0;
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
        self.h_flag = (((a.0 & 0xFFF).wrapping_add(b.0 & 0xFFF)) & 0x1000) == 0x1000;
    }

    #[inline(always)]
    pub fn check_half_carry_sub_byte(&mut self, a: Byte, b: Byte) {
        self.h_flag = (a.0 & 0xF) < (b.0 & 0xF);
    }

    #[inline(always)]
    pub fn check_carry_sub_byte(&mut self, a: Byte, b: Byte) {
        self.c_flag = a.0 < b.0;
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
        // 0x10 - 0x01: low nibble 0 < 1, so a half-borrow occurs. (The previous
        // 0x02 - 0x10 case asserted a borrow, but 0x10's low nibble is 0, so
        // there is none - the assertion was wrong, not the implementation.)
        cpu.check_half_carry_sub_byte(Byte(0x10), Byte(0x01));
        assert!(cpu.h_flag);
        cpu.check_half_carry_sub_byte(Byte(0x08), Byte(0x01));
        assert!(!cpu.h_flag);
    }

    #[test]
    fn add_sets_addition_flags() {
        let mut cpu = CentralProcessor::new();
        cpu.a = Byte(0x0F);
        cpu.add(Byte(0x01)); // 0x10: half-carry, no carry
        assert_eq!(cpu.a, Byte(0x10));
        assert!(cpu.h_flag && !cpu.c_flag && !cpu.z_flag && !cpu.n_flag);

        cpu.a = Byte(0xFF);
        cpu.add(Byte(0x01)); // wraps to 0: carry + half-carry + zero
        assert_eq!(cpu.a, Byte(0x00));
        assert!(cpu.c_flag && cpu.h_flag && cpu.z_flag);
    }

    #[test]
    fn adc_includes_the_incoming_carry() {
        let mut cpu = CentralProcessor::new();
        cpu.a = Byte(0xFF);
        cpu.c_flag = true;
        cpu.adc(Byte(0x00)); // 0xFF + 0 + 1 = 0x100
        assert_eq!(cpu.a, Byte(0x00));
        assert!(cpu.c_flag && cpu.h_flag && cpu.z_flag && !cpu.n_flag);
    }

    #[test]
    fn sub_sets_borrow_flags() {
        let mut cpu = CentralProcessor::new();
        cpu.a = Byte(0x10);
        cpu.sub(Byte(0x01)); // 0x0F: half-borrow, no borrow
        assert_eq!(cpu.a, Byte(0x0F));
        assert!(cpu.h_flag && !cpu.c_flag && cpu.n_flag);

        cpu.a = Byte(0xFF);
        cpu.sub(Byte(0x01)); // 0xFE: no borrow (the old carry helper set it here)
        assert_eq!(cpu.a, Byte(0xFE));
        assert!(!cpu.c_flag && !cpu.h_flag);

        cpu.a = Byte(0x00);
        cpu.sub(Byte(0x01)); // wraps: borrow + half-borrow
        assert_eq!(cpu.a, Byte(0xFF));
        assert!(cpu.c_flag && cpu.h_flag);
    }

    #[test]
    fn sbc_includes_the_incoming_borrow() {
        let mut cpu = CentralProcessor::new();
        cpu.a = Byte(0x00);
        cpu.c_flag = true;
        cpu.sbc(Byte(0x00)); // 0 - 0 - 1 = 0xFF
        assert_eq!(cpu.a, Byte(0xFF));
        assert!(cpu.c_flag && cpu.h_flag && cpu.n_flag && !cpu.z_flag);
    }

    #[test]
    fn test_swap() {
        let mut cpu = CentralProcessor::new();
        assert_eq!(cpu.swap(Byte(0b1010_0101)), Byte(0b0101_1010));
        assert_eq!(cpu.swap(Byte(0b0000_1111)), Byte(0b1111_0000));
    }
}

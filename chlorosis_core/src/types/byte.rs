use std::ops::{Add, Sub};

use super::{Address, SignedByte};

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Byte(pub u8);

impl Byte {
    pub const fn to_address(self) -> Address {
        Address(self.0 as u16)
    }

    pub const fn is_bit_set(&self, n: u8) -> bool {
        let mask = 1 << n;
        self.0 & mask != 0
    }

    pub fn set_bit(&mut self, n: u8) {
        let mask = 1 << n;
        self.0 |= mask;
    }

    pub fn write_bit(&mut self, n: u8, set: bool) {
        if set {
            self.0 |= 1 << n;
        } else {
            self.0 &= !(1 << n)
        }
    }

    pub const fn to_signed(self) -> SignedByte {
        SignedByte(-((!self.0.wrapping_add(1)) as i8))
    }
}

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "{:02X}", self.0)?)
    }
}

impl std::fmt::Debug for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "{:02X}", self.0)?)
    }
}

impl std::ops::Not for Byte {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Add<Self> for Byte {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Self> for Byte {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Shl<u8> for Byte {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl std::ops::Shr<u8> for Byte {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl std::ops::BitAnd<u8> for Byte {
    type Output = Self;

    fn bitand(self, rhs: u8) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl std::ops::AddAssign<u8> for Byte {
    fn add_assign(&mut self, rhs: u8) {
        self.0 = self.0 + rhs;
    }
}

impl std::ops::AddAssign<i32> for Byte {
    fn add_assign(&mut self, rhs: i32) {
        self.0 = self.0 + rhs as u8;
    }
}

impl std::ops::SubAssign<u8> for Byte {
    fn sub_assign(&mut self, rhs: u8) {
        self.0 = self.0 - rhs;
    }
}

impl std::ops::SubAssign<i32> for Byte {
    fn sub_assign(&mut self, rhs: i32) {
        self.0 = self.0 - rhs as u8;
    }
}

impl std::ops::BitXorAssign<Self> for Byte {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl std::ops::BitAndAssign<Self> for Byte {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign<Self> for Byte {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use crate::Byte;

    #[test]
    fn test_write_bit() {
        let mut b = Byte(0b00101010);
        b.write_bit(2, true);
        assert_eq!(b, Byte(0b00101110));
        b.write_bit(5, false);
        assert_eq!(b, Byte(0b00001110));
    }
}

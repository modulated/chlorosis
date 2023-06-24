use super::Address;

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

    pub const fn to_signed(self) -> i8 {
        -((!self.0 + 1) as i8)
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
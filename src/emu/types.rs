use std::ops::{Add, IndexMut, Mul, Sub};

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

    pub(crate) fn to_signed(self) -> i8 {
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

impl Sub<Self> for Address {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<u8> for Address {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self(self.0 - rhs as u16)
    }
}

impl Sub<i32> for Address {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - rhs as u16)
    }
}

impl Sub<u16> for Address {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Add<Self> for Address {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<u8> for Address {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self(self.0 + rhs as u16)
    }
}

impl Add<i32> for Address {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs as u16)
    }
}

impl Add<u16> for Address {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Mul<usize> for Address {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as u16)
    }
}

impl std::ops::AddAssign<u8> for Address {
    fn add_assign(&mut self, rhs: u8) {
        self.0 = self.0 + (rhs as u16);
    }
}

impl std::ops::AddAssign<u16> for Address {
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0 + rhs;
    }
}

impl std::ops::AddAssign<i32> for Address {
    fn add_assign(&mut self, rhs: i32) {
        self.0 = self.0 + rhs as u16;
    }
}

impl std::ops::AddAssign<Byte> for Address {
    fn add_assign(&mut self, rhs: Byte) {
        self.0 = self.0 + (rhs.0 as u16);
    }
}

impl std::ops::SubAssign<u8> for Address {
    fn sub_assign(&mut self, rhs: u8) {
        self.0 = self.0 - (rhs as u16);
    }
}

impl std::ops::SubAssign<u16> for Address {
    fn sub_assign(&mut self, rhs: u16) {
        self.0 = self.0 - rhs;
    }
}

impl std::ops::SubAssign<i32> for Address {
    fn sub_assign(&mut self, rhs: i32) {
        self.0 = self.0 - (rhs as u16);
    }
}

impl std::ops::SubAssign<Byte> for Address {
    fn sub_assign(&mut self, rhs: Byte) {
        self.0 = self.0 - (rhs.0 as u16);
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

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(pub u16);

impl Address {
    pub const fn split(self) -> (Byte, Byte) {
        (Byte((self.0 >> 8) as u8), Byte((self.0 & 0x00FF) as u8))
    }

    pub const fn from_pair(h: Byte, l: Byte) -> Self {
        Self(((h.0 as u16) << 8) + (l.0 as u16))
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "{:#06X}", self.0)?)
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<usize> for Address {
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "{:#06X}", self.0)?)
    }
}

impl From<Address> for u16 {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl From<Address> for usize {
    fn from(value: Address) -> Self {
        value.0 as Self
    }
}

impl std::ops::Index<Address> for Vec<Byte> {
    type Output = Byte;

    fn index(&self, index: Address) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl IndexMut<Address> for Vec<Byte> {
    fn index_mut(&mut self, index: Address) -> &mut Byte {
        self.get_mut(index.0 as usize).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::{Address, Byte};

    #[test]
    fn test_split_addr() {
        let a = Address(0x1234);
        let (h, l) = a.split();

        assert_eq!(h, Byte(0x12));
        assert_eq!(l, Byte(0x34))
    }
}

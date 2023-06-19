use std::ops::{Add, IndexMut, Mul, Sub};

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Byte(pub u8);

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

impl Sub<Self> for Address {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add<Self> for Address {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Mul<usize> for Address {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as u16)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(pub u16);

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

pub struct BytePair(Byte, Byte);

impl IndexMut<Address> for Vec<Byte> {
    fn index_mut(&mut self, index: Address) -> &mut Byte {
        self.get_mut(index.0 as usize).unwrap()
    }
}

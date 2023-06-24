#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedByte(pub i8);

impl std::fmt::Display for SignedByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "{:02X}", self.0)?)
    }
}

impl std::fmt::Debug for SignedByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "{:02X}", self.0)?)
    }
}

//! Save-state file format and errors.
//!
//! A save state is the whole [`crate::Device`] serialized with `bincode`,
//! behind a small header that lets a load fail loudly rather than restore
//! garbage: a magic tag identifies the file, a format version rejects states
//! written by an incompatible build, and the cartridge's global checksum
//! rejects a state saved from a different ROM.

use std::fmt;

/// Magic bytes at the start of every save-state file: "CHLoroSis Save".
pub const MAGIC: [u8; 4] = *b"CHLS";

/// Save-state format version. Bump this whenever the serialized shape of
/// [`crate::Device`] changes in a way that would misread an older file.
pub const FORMAT_VERSION: u16 = 1;

/// Bytes of header before the `bincode` payload: magic(4) + version(2) +
/// ROM checksum(2).
pub const HEADER_LEN: usize = 8;

/// Why a save or load failed. Every variant carries enough to tell the user
/// what went wrong without guessing.
#[derive(Debug)]
pub enum SaveStateError {
    /// The state file could not be read or written.
    Io(std::io::Error),
    /// Serializing or deserializing the machine state failed.
    Codec(bincode::Error),
    /// Nothing to save into, or to validate a load against.
    NoCartridge,
    /// The file is not a Chlorosis save state (wrong or missing magic).
    NotASaveState,
    /// The file's format version is not the one this build writes.
    VersionMismatch { found: u16, expected: u16 },
    /// The state was saved from a different ROM than the one loaded now.
    WrongRom { found: u16, expected: u16 },
    /// The file is too short to even contain a header.
    Truncated,
}

impl fmt::Display for SaveStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "file error: {e}"),
            Self::Codec(e) => write!(f, "corrupt save state: {e}"),
            Self::NoCartridge => write!(f, "no cartridge loaded"),
            Self::NotASaveState => write!(f, "not a Chlorosis save state"),
            Self::VersionMismatch { found, expected } => write!(
                f,
                "save-state format v{found}, this build reads v{expected}"
            ),
            Self::WrongRom { found, expected } => write!(
                f,
                "save state is for a different ROM (checksum {found:#06X}, loaded {expected:#06X})"
            ),
            Self::Truncated => write!(f, "save state is truncated"),
        }
    }
}

impl std::error::Error for SaveStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Codec(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SaveStateError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<bincode::Error> for SaveStateError {
    fn from(e: bincode::Error) -> Self {
        Self::Codec(e)
    }
}

use thiserror::Error;

pub type Result<T> = std::result::Result<T, HuffmanError>;

#[derive(Error, Debug)]
pub enum HuffmanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("binary format error: {0}")]
    Binrw(#[from] binrw::Error),

    #[error("core algorithm error: {0}")]
    Core(#[from] CoreError),

    #[error("protocol format error: {0}")]
    Format(#[from] FormatError),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    #[error("frequency table is empty")]
    EmptyFrequencyTable,

    #[error("frequency table count mismatch: declared={declared}, actual={actual}")]
    FrequencyCountMismatch { declared: u16, actual: usize },

    #[error("symbol 0x{0:02X} not found in codebook")]
    SymbolNotFound(u8),

    #[error("failed to decode bitstream: reached an invalid state in Huffman tree")]
    DecodeError,

    #[error("invalid Huffman tree structure: {reason}")]
    InvalidTree { reason: String },

    #[error("frequency overflow when merging nodes: left={left}, right={right}")]
    FrequencyOverflow { left: u64, right: u64 },
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    #[error("invalid magic number: expected {expected:?}, found {found:?}")]
    InvalidMagic { expected: [u8; 4], found: [u8; 4] },

    #[error("invalid footer magic: expected {expected:?}, found {found:?}")]
    InvalidFooter { expected: [u8; 4], found: [u8; 4] },

    #[error("unsupported format version: found={found}, supported={supported}")]
    UnsupportedVersion { found: u8, supported: u8 },

    #[error("feature not supported in v1: {reason}")]
    NotSupported { reason: String },

    #[error("reserved field must be 0, found={found}")]
    ReservedNotZero { found: u16 },

    #[error("padding bits must be in [0, 7], found={pad}")]
    InvalidPadding { pad: u8 },

    #[error("mismatched data length: expected {expected}, actual {actual}")]
    MismatchedLength { expected: u64, actual: u64 },
}

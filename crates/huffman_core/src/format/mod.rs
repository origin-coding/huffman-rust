//! Huffman 压缩相关二进制格式。
//! 包含魔数、版本号、全局头、条目头、文件尾等结构体。

pub mod v1;

// 全局头、条目头、文件尾的魔数
pub const GLOBAL_HEADER_MAGIC: &[u8; 4] = b"HUFF";
pub const ENTRY_HEADER_MAGIC: &[u8; 4] = b"ENTR";
pub const FOOTER_MAGIC: &[u8; 4] = b"FOOT";

// 格式版本
pub const VERSION_1: u8 = 1;
pub const VERSION_CURRENT: u8 = VERSION_1;

// 导出子模块下的 API
pub use v1::{EntryHeader, FrequencyEntry, FrequencyTable, GlobalFooter, GlobalHeader};

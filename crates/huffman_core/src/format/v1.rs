//! 压缩文件头相关内容。

use crate::format::VERSION_1;
use binrw::{BinRead, BinWrite};
use bitflags::bitflags;

bitflags! {
    /// 全局功能标志位
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd, Hash)]
    pub struct GlobalFlags: u8 {
        // 数据已加密
        const IS_ENCRYPTED = 0x01;

        // 是否使用 SHA-256 校验
        const USE_SHA256 = 0x02;

        // 是否是分卷压缩
        const IS_SPLIT = 0x04;
    }

    /// Entry 的功能标志位
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd, Hash)]
    pub struct EntryFlags: u8 {
        // 是否是目录
        const IS_DIR = 0x01;

        // 是否包含元信息
        const HAS_METADATA = 0x02;

        // 是否使用 Canonical
        const USE_CANONICAL = 0x04;
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, PartialEq)]
#[brw(big)] // 全局大端序
#[brw(magic = b"HUFF")] // binrw 只支持使用字面量定义魔数
pub struct GlobalHeader {
    /// 协议版本
    #[br(assert(version == VERSION_1, "Unsupported version: {}", version))]
    #[bw(assert(*version == VERSION_1, "Unsupported version: {}", version))]
    pub version: u8,

    /// 标志位 (映射为 bitflags 结构体)
    #[br(map = |bits: u8| GlobalFlags::from_bits_truncate(bits))]
    #[bw(map = |flags: &GlobalFlags| flags.bits())]
    pub flags: GlobalFlags,

    /// 保留字节 (必须为 0)
    #[br(assert(reserved == 0u16, "Reserved field must be 0"))]
    #[bw(assert(*reserved == 0u16, "Reserved field must be 0"))]
    pub reserved: u16,
}

#[derive(BinRead, BinWrite, Debug, Clone, PartialEq)]
#[brw(big)] // 条目大端序
#[brw(magic = b"ENTR")]
pub struct EntryHeader {
    /// 标志位 (映射为 bitflags 结构体)
    #[br(map = |bits: u8| EntryFlags::from_bits_truncate(bits))]
    #[bw(map = |flags: &EntryFlags| flags.bits())]
    pub flags: EntryFlags,

    /// 数据填充位数（0-7）
    #[br(assert(pad < 8, "Padding bits must be less than 8"))]
    #[bw(assert(*pad < 8, "Padding bits must be less than 8"))]
    pub pad: u8,

    /// 元数据长度（字节）
    pub metadata_length: u16,

    /// 原始文件大小（字节）
    pub original_size: u64,

    /// Huffman 树大小（字节）
    pub huffman_tree_size: u32,

    /// 压缩后数据大小（字节）
    pub compressed_size: u32,
}

/// 全局文件尾结构体
#[derive(BinRead, BinWrite, Debug, Clone, PartialEq)]
#[brw(big)]
#[brw(magic = b"TAIL")]
pub struct GlobalFooter {
    /// 索引表偏移量（字节）
    /// V1 版本固定填 0
    #[br(assert(index_offset == 0, "Index offset must be 0"))]
    #[bw(assert(*index_offset == 0, "Index offset must be 0"))]
    pub index_offset: u64,

    /// 索引表大小（字节）
    /// V1 版本固定填 0
    #[br(assert(index_size == 0, "Index size must be 0"))]
    #[bw(assert(*index_size == 0, "Index size must be 0"))]
    pub index_size: u64,

    /// 校验和
    pub checksum: u32,
}

/// 频率表项结构体
#[derive(BinRead, BinWrite, Debug, Clone, PartialEq)]
#[brw(big)]
pub struct FrequencyEntry {
    /// 字符
    pub symbol: u8,

    /// 字符出现频率
    pub frequency: u32,
}

/// 频次表区块结构体
#[derive(BinRead, BinWrite, Debug, Clone, PartialEq)]
#[brw(big)]
pub struct FrequencyTable {
    /// 频率表项数量
    pub count: u32,

    /// 频率表项数组
    #[br(count = count)]
    pub entries: Vec<FrequencyEntry>,
}

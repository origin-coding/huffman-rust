pub mod core_tree_tests;

use binrw::{BinRead, BinWrite};
use huffman_core::format::v1::{EntryFlags, GlobalFlags};
use huffman_core::format::{EntryHeader, FrequencyTable, GlobalFooter, GlobalHeader, VERSION_1};
use std::io::Cursor;

#[test]
fn test_global_header_serialization() {
    // 1. 构造一个准备写入的 Header
    let mut flags = GlobalFlags::empty();
    flags.insert(GlobalFlags::IS_ENCRYPTED); // 开启加密标志

    let original_header = GlobalHeader {
        version: VERSION_1,
        flags,
        reserved: 0,
    };

    // 2. 准备一个内存游标 (模拟文件)
    let mut cursor = Cursor::new(Vec::new());

    // 3. 写入二进制数据
    original_header.write(&mut cursor).expect("写入失败");

    // 4. 重置游标位置到开头，准备读取
    cursor.set_position(0);

    // 5. 从二进制数据中反序列化
    let decoded_header = GlobalHeader::read(&mut cursor).expect("读取失败");

    // 6. 断言：解出来的结构体必须和原始的一模一样！
    assert_eq!(original_header, decoded_header);
    assert!(decoded_header.flags.contains(GlobalFlags::IS_ENCRYPTED));
}

#[test]
fn test_magic_number_validation() {
    // 测试如果魔数不对，binrw 是否能正确报错
    let bad_data = b"FAKE\x01\x00\x00\x00"; // 故意写错魔数
    let mut cursor = Cursor::new(bad_data);

    let result = GlobalHeader::read(&mut cursor);
    assert!(result.is_err(), "遇到错误的魔数应该报错，但却解析成功了！");
}

// 针对 GlobalHeader 的边界测试

#[test]
fn test_global_header_fails_on_unsupported_version() {
    // 【边界测试】版本号不匹配时，应当拒绝写入和读取
    let bad_header = GlobalHeader {
        version: 99, // 错误的协议版本
        flags: GlobalFlags::empty(),
        reserved: 0,
    };

    let mut cursor = Cursor::new(Vec::new());

    // 写入时触发 bw(assert) 拦截
    let write_result = bad_header.write(&mut cursor);
    assert!(write_result.is_err(), "写入不支持的版本号时应该报错！");

    // 模拟恶意文件的读取
    // 手工构造一个二进制序列：HUFF (4字节) + 99 (1字节) + flags(1字节) + reserved(2字节)
    let bad_data = b"HUFF\x63\x00\x00\x00";
    let mut read_cursor = Cursor::new(bad_data);
    let read_result = GlobalHeader::read(&mut read_cursor);
    assert!(read_result.is_err(), "读取到不支持的版本号时必须拦截！");
}

#[test]
fn test_global_header_fails_on_nonzero_reserved() {
    // 【边界测试】保留字段必须为 0
    let bad_header = GlobalHeader {
        version: VERSION_1,
        flags: GlobalFlags::empty(),
        reserved: 1, // 错误：非零保留位
    };

    let mut cursor = Cursor::new(Vec::new());
    assert!(
        bad_header.write(&mut cursor).is_err(),
        "保留字段不为0时应当报错！"
    );
}

#[test]
fn test_global_header_truncates_unknown_flags() {
    // 【容错测试】如果读取到未来版本加入的未知 Flag，from_bits_truncate 应该安全地将其丢弃
    // 构造数据：HUFF + 版本1 + flag设为 0xFF(全1) + reserved(0)
    let raw_data = b"HUFF\x01\xFF\x00\x00";
    let mut cursor = Cursor::new(raw_data);

    let header = GlobalHeader::read(&mut cursor).expect("读取应成功，未定义标志应被截断");

    // 当前我们只定义了 0x01, 0x02, 0x04，总和是 0x07
    assert_eq!(header.flags.bits(), 0x07, "未定义的标志位没有被正确过滤！");
}

// 针对 EntryHeader 的边界测试

#[test]
fn test_entry_header_pad_boundaries() {
    // 【边界测试】测试 pad 的极值 (7 是合法的，8 是非法的)

    // 1. 合法极值测试 (pad = 7)
    let valid_header = EntryHeader {
        flags: EntryFlags::empty(),
        pad: 7,
        metadata_length: 0,
        original_size: 0,
        huffman_tree_size: 0,
        compressed_size: 0,
    };
    let mut cursor = Cursor::new(Vec::new());
    assert!(
        valid_header.write(&mut cursor).is_ok(),
        "pad=7 应当允许写入"
    );

    // 2. 非法边界测试 (pad = 8)
    let mut invalid_header = valid_header.clone();
    invalid_header.pad = 8;
    let mut cursor = Cursor::new(Vec::new());
    assert!(
        invalid_header.write(&mut cursor).is_err(),
        "pad=8 必须被拦截"
    );
}

// 针对 GlobalFooter 的边界测试

#[test]
fn test_global_footer_v1_constraints() {
    // 【边界测试】V1 版本中，如果 offset 或 size 不为 0，程序应该拒绝处理
    let bad_footer_1 = GlobalFooter {
        index_offset: 100, // 错误
        index_size: 0,
        checksum: 12345,
    };

    let mut cursor = Cursor::new(Vec::new());
    assert!(
        bad_footer_1.write(&mut cursor).is_err(),
        "非法的 index_offset 未被拦截"
    );

    let bad_footer_2 = GlobalFooter {
        index_offset: 0,
        index_size: 50, // 错误
        checksum: 12345,
    };
    let mut cursor = Cursor::new(Vec::new());
    assert!(
        bad_footer_2.write(&mut cursor).is_err(),
        "非法的 index_size 未被拦截"
    );
}

// 针对 FrequencyEntry 和 FrequencyTable 的边界测试

#[test]
fn test_frequency_table_eof_protection() {
    // 【安全测试】测试由于文件损坏导致的提前结束 (Unexpected EOF)

    // 我们手工构造一个二进制序列：
    // count = 10 (需要占 2 字节：00 0A)
    // 但是后面我们只给 1 个 entry 的数据 (1 字节 symbol + 8 字节 frequency = 9 字节)
    // 总数据长度只有 11 字节，远远不够 10 个 entry 的长度
    let corrupted_data: &[u8] = &[
        0x00, 0x0A, // count = 10
        0x41, // symbol = 'A'
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // frequency = 5
    ];

    let mut cursor = Cursor::new(corrupted_data);
    let read_result = FrequencyTable::read(&mut cursor);

    // 如果程序没有崩溃，并且正确返回了 Err，说明 binrw 的越界保护生效了
    assert!(
        read_result.is_err(),
        "读取残缺的频率表时应当抛出 EOF 错误！"
    );
}

#[test]
fn test_frequency_table_empty_table() {
    // 【边界测试】如果是空文件（0字节），频率表也是空的，这在逻辑上是合法的
    let empty_table = FrequencyTable {
        count: 0,
        entries: vec![],
    };

    let mut cursor = Cursor::new(Vec::new());
    empty_table.write(&mut cursor).expect("写入空表失败");

    cursor.set_position(0);
    let decoded = FrequencyTable::read(&mut cursor).expect("读取空表失败");

    assert_eq!(decoded.count, 0);
    assert!(decoded.entries.is_empty());
}

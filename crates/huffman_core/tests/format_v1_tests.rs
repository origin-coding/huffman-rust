use binrw::{BinRead, BinWrite};
use std::io::Cursor;
use huffman_core::format::{GlobalHeader, VERSION_1};
use huffman_core::format::v1::GlobalFlags;

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
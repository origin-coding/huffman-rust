use bitvec::prelude::*;
use huffman_core::core::bit_io::BitWriter;
use std::io::Cursor;

#[test]
fn test_write_single_byte() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = BitWriter::new(&mut buffer);

    // 写入 0xB3: 10110011
    let bits = [true, false, true, true, false, false, true, true];
    for bit in bits {
        writer.write_bit(bit).unwrap();
    }

    let padding = writer.finalize().unwrap();
    assert_eq!(padding, 0);
    assert_eq!(buffer.into_inner(), vec![0xB3]);
}

#[test]
fn test_write_with_padding() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = BitWriter::new(&mut buffer);

    // 写入 110 (3 bits)
    writer.write_bit(true).unwrap();
    writer.write_bit(true).unwrap();
    writer.write_bit(false).unwrap();

    let padding = writer.finalize().unwrap();
    // 110 00000 -> 0xC0
    assert_eq!(padding, 5);
    assert_eq!(buffer.into_inner(), vec![0xC0]);
}

#[test]
fn test_write_multi_bytes() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = BitWriter::new(&mut buffer);

    // 写入 10 个比特: 11111111 01
    for _ in 0..8 {
        writer.write_bit(true).unwrap();
    }
    writer.write_bit(false).unwrap();
    writer.write_bit(true).unwrap();

    let padding = writer.finalize().unwrap();
    // 11111111 01000000 -> [0xFF, 0x40]
    assert_eq!(padding, 6);
    assert_eq!(buffer.into_inner(), vec![0xFF, 0x40]);
}

#[test]
fn test_write_bits_slice() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = BitWriter::new(&mut buffer);

    // 使用 bitvec 构造测试数据
    let bits = bitvec![u8, Msb0; 1, 0, 1];
    writer.write_bits(&bits).unwrap();

    let padding = writer.finalize().unwrap();
    assert_eq!(padding, 5);
    assert_eq!(buffer.into_inner(), vec![0xA0]); // 10100000
}

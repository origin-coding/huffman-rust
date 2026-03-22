use huffman_core::core::codebook::CodeBook;
use huffman_core::core::tree::HuffmanTree;
use huffman_core::format::{FrequencyEntry, FrequencyTable};

#[test]
fn test_huffman_logic_flow() {
    // 构造频次表
    let entries = vec![
        FrequencyEntry {
            symbol: b'A',
            frequency: 10,
        },
        FrequencyEntry {
            symbol: b'B',
            frequency: 20,
        },
        FrequencyEntry {
            symbol: b'C',
            frequency: 15,
        },
    ];

    let table = FrequencyTable { count: 3, entries };

    // 构造 Huffman 树和编码表
    let tree = HuffmanTree::try_from(&table).expect("构建 Huffman 树失败");
    let codebook = CodeBook::from(&tree);

    // 验证逻辑：频率最高的字符 B 应该拥有最短的编码
    assert_eq!(codebook.get_code(b'B').unwrap().len(), 1);

    // 验证确定性：由于 A(10) < C(15)，且 A+C(25) > B(20)
    // 最终 B 的编码应为 [false] (0)，A 为 [true, false] (10)
    assert_eq!(codebook.get_code(b'A').unwrap(), &vec![true, false]);
}

#[test]
fn test_empty_table_error() {
    let table = FrequencyTable { count: 0, entries: vec![] };
    assert!(HuffmanTree::try_from(&table).is_err());
}

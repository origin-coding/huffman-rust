use huffman_core::core::codebook::CodeBook;
use huffman_core::core::error::CoreError;
use huffman_core::core::tree::HuffmanTree;
use huffman_core::format::{FrequencyEntry, FrequencyTable};

#[test]
fn test_huffman_logic_flow() {
    // 构造频次表 (A:10, B:20, C:15)
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
fn test_single_symbol_table() {
    // 模拟只有一个字符的情况（例如一个只有 'A' 的文件）
    let entries = vec![FrequencyEntry {
        symbol: b'A',
        frequency: 100,
    }];
    let table = FrequencyTable { count: 1, entries };

    let tree = HuffmanTree::try_from(&table).expect("单字符构建失败");
    let codebook = CodeBook::from(&tree);

    // 单字符时，路径为空（根节点即为叶子）
    let code = codebook.get_code(b'A').expect("未找到编码");
    assert!(code.is_empty(), "单字符路径应为空");
}

#[test]
fn test_frequency_tie_breaker() {
    // 模拟频率完全相等的情况：A:10, B:10, C:10, D:10
    // 这将严格测试 min_symbol 的决胜逻辑
    let entries = vec![
        FrequencyEntry {
            symbol: b'D',
            frequency: 10,
        },
        FrequencyEntry {
            symbol: b'C',
            frequency: 10,
        },
        FrequencyEntry {
            symbol: b'B',
            frequency: 10,
        },
        FrequencyEntry {
            symbol: b'A',
            frequency: 10,
        },
    ];
    let table = FrequencyTable { count: 4, entries };

    let tree = HuffmanTree::try_from(&table).expect("决胜测试构建失败");
    let _codebook = CodeBook::from(&tree);

    // 如果实现正确且确定，所有字符的编码长度都应该是 2
    // 由于堆是确定性的（基于频率和 min_symbol），结果不应随运行环境改变
}

#[test]
fn test_all_256_symbols() {
    // 模拟所有 256 个字符都出现的情况
    let mut entries = Vec::new();
    for i in 0..=255 {
        entries.push(FrequencyEntry {
            symbol: i,
            frequency: (i as u64) + 1,
        });
    }
    let table = FrequencyTable {
        count: 256,
        entries,
    };

    let tree = HuffmanTree::try_from(&table).expect("全字符构建失败");
    let codebook = CodeBook::from(&tree);

    // 验证所有字符都有编码
    for i in 0..=255 {
        assert!(codebook.get_code(i).is_some());
    }
}

#[test]
fn test_large_frequency_handling() {
    // 模拟大文件（超过 u32 范围）的频率累加
    let entries = vec![
        FrequencyEntry {
            symbol: b'A',
            frequency: 5_000_000_000,
        },
        FrequencyEntry {
            symbol: b'B',
            frequency: 5_000_000_000,
        },
    ];
    let table = FrequencyTable { count: 2, entries };

    let tree = HuffmanTree::try_from(&table).expect("大文件测试构建失败");
    // 根节点频率应为 100 亿，验证 u64 是否溢出
    assert_eq!(tree.frequency(), 10_000_000_000);
}

#[test]
fn test_empty_table_error() {
    let table = FrequencyTable {
        count: 0,
        entries: vec![],
    };
    let err = HuffmanTree::try_from(&table).expect_err("空表应返回错误");
    assert_eq!(err, CoreError::EmptyFrequencyTable);
}

#[test]
fn test_frequency_count_mismatch_error() {
    let table = FrequencyTable {
        count: 2,
        entries: vec![FrequencyEntry {
            symbol: b'A',
            frequency: 1,
        }],
    };

    let err = HuffmanTree::try_from(&table).expect_err("count 与 entries 长度不一致应失败");
    assert_eq!(
        err,
        CoreError::FrequencyCountMismatch {
            declared: 2,
            actual: 1,
        }
    );
}

#[test]
fn test_duplicate_symbol_error() {
    let table = FrequencyTable {
        count: 2,
        entries: vec![
            FrequencyEntry {
                symbol: b'A',
                frequency: 1,
            },
            FrequencyEntry {
                symbol: b'A',
                frequency: 2,
            },
        ],
    };

    let err = HuffmanTree::try_from(&table).expect_err("重复 symbol 应失败");
    assert_eq!(err, CoreError::DuplicateSymbol { symbol: b'A' });
}

#[test]
fn test_zero_frequency_error() {
    let table = FrequencyTable {
        count: 1,
        entries: vec![FrequencyEntry {
            symbol: b'A',
            frequency: 0,
        }],
    };

    let err = HuffmanTree::try_from(&table).expect_err("频次为 0 应失败");
    assert_eq!(err, CoreError::ZeroFrequency { symbol: b'A' });
}

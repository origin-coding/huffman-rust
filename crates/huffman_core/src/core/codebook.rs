use crate::core::tree::HuffmanTree;
use bitvec::prelude::*;

/// 编码表：将字节符号映射到变长的比特流 (Huffman 编码)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBook {
    /// 256 个字节对应的可选比特序列
    /// 使用 BitVec<u8, Msb0> 实现真正的位存储，Msb0 保证位序与 BitWriter 一致
    pub map: [Option<BitVec<u8, Msb0>>; 256],
}

impl CodeBook {
    /// 根据字节符号获取对应的比特序列视图
    pub fn get_code(&self, symbol: u8) -> Option<&BitSlice<u8, Msb0>> {
        self.map[symbol as usize].as_deref()
    }

    /// 递归构建编码表 (DFS 遍历 Huffman 树)
    fn build_recursive(
        node: &HuffmanTree,
        path: &mut BitVec<u8, Msb0>,
        map: &mut [Option<BitVec<u8, Msb0>>; 256],
    ) {
        match node {
            // 到达叶子节点，将当前路径作为编码存储
            HuffmanTree::Leaf { symbol, .. } => {
                map[*symbol as usize] = Some(path.clone());
            }
            // 遍历左右子树
            HuffmanTree::Internal { left, right, .. } => {
                // 左分支为 0 (false)
                path.push(false);
                Self::build_recursive(left, path, map);
                path.pop(); // 回溯

                // 右分支为 1 (true)
                path.push(true);
                Self::build_recursive(right, path, map);
                path.pop(); // 回溯
            }
        }
    }
}

impl From<&HuffmanTree> for CodeBook {
    fn from(tree: &HuffmanTree) -> Self {
        let mut map = core::array::from_fn(|_| None);
        let mut path = BitVec::new();

        // 只有当树非空（即至少有一个叶子节点）时才开始构建
        // 实际上 HuffmanTree::try_from 已经保证了树至少有一个节点
        Self::build_recursive(tree, &mut path, &mut map);

        Self { map }
    }
}

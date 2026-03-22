use crate::core::tree::HuffmanTree;

pub struct CodeBook {
    pub map: [Option<Vec<bool>>; 256],
}

impl CodeBook {
    pub fn get_code(&self, symbol: u8) -> Option<&Vec<bool>> {
        self.map[symbol as usize].as_ref()
    }

    fn build_recursive(
        node: &HuffmanTree,
        path: &mut Vec<bool>,
        map: &mut [Option<Vec<bool>>; 256],
    ) {
        match node {
            // 到达叶子节点，将编码存储到 map 中
            HuffmanTree::Leaf { symbol, .. } => {
                map[*symbol as usize] = Some(path.clone());
            }
            // 遍历左右子树，添加当前节点的编码
            HuffmanTree::Internal { left, right, .. } => {
                path.push(false);
                Self::build_recursive(left, path, map);
                path.pop();  // 撤销刚才的移动，回溯到父节点，准备向右走

                path.push(true);
                Self::build_recursive(right, path, map);
                path.pop(); // 左右子树都遍历完成，回溯到上一层递归
            }
        }
    }
}

impl From<&HuffmanTree> for CodeBook {
    fn from(value: &HuffmanTree) -> Self {
        let mut map = core::array::from_fn(|_| None);
        let mut path = Vec::new();

        // 开始深度优先遍历，构建编码表
        Self::build_recursive(value, &mut path, &mut map);

        Self { map }
    }
}

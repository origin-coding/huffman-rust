在前面的工作中，我们定义了 Huffman 压缩包的二进制结构，并对其进行了读取/写入的测试。
现在，我们需要在内存中表示一颗 Huffman 树，以便后续的压缩与解压缩操作。在今天的工作中，我们将：

1. 定义 Huffman 树的节点结构以及相关方法。
2. 实现 Huffman 树的构建算法。

# Huffman 树的节点结构与方法

一颗 Huffman 树的节点可以分为内部节点和叶子节点两种类型。
其中内部节点存储了左右子树的引用，而叶子节点负责存储原始数据以及对应的频率。
我们可以使用 Rust 的枚举来定义 Huffman 树的节点类型。

```rust
pub enum HuffmanTree {
    Leaf {
        symbol: u8,
        frequency: u64,
    },
    Internal {
        frequency: u64,
        min_symbol: u8,
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
    },
}
```

关于类型定义，我们需要注意以下几点：

## 关于频率和最小符号的计算和缓存

叶子节点的频率与原始数据的频率相同，直接存储在叶子节点的 `frequency` 字段中即可。对于内部节点，其频率是左右子树之和。
但是为了避免在计算 `HuffmanTree` 的频率时频繁递归遍历子树，我们将频率额外使用 `frequency` 字段定义，在构建时计算出来。
同时，`frequency` 字段会被设置为只读，以确保在构建时计算出来的频率不会被错误地修改。

我们还需要存储每个节点的最小符号，用于保证编码时排序的正确性。 在排序时，如果两个节点的频率相同，那么我们就使用最小符号来进一步比较。
这样做能保证在编码时，即使两个节点的频率相同，也会因为最小符号的不同，而保证排序的一致性，不会因为顺序错误而导致不能正常编解码。
对于 `Internal` 类型的节点，我们使用 `min_symbol` 字段来存储左右子树中最小的符，像 `frequency` 字段一样避免重复计算。

## 关于左右子树的类型

对于内部节点的左右子树字段，我们无法使用类似 `left: HuffmanTree` 这样的定义，如果一定这么写，会导致无法编译。
这是因为 Rust 为了高效地管理内存，需要在编译时确定每个类型、每个变量的大小。这个特性在 Rust 的类型系统中体现为 `Sized`
Trait。
在 Rust 中，枚举类型的大小由其最大的成员（Variant）的大小决定。
如果使用 `HuffmanTree` 类型，就会导致递归定义，当编译器尝试计算 `HuffmanTree` 的大小时，就会陷入死循环：

1. 编译器想知道 `HuffmanTree` 类型的大小。
2. 它去检查 `Internal` 变体的大小。
3. `Internal` 的大小 = `u64` 的大小（8 字节）+ `left` 的大小 + `right` 的大小。
4. `left` 和 `right` 的类型也是 `HuffmanTree`！
5. 计算大小的方程就变成了：`Size(HuffmanTree) = 8 + Size(HuffmanTree) + Size(HuffmanTree)`

这是一个没有解的方程，因此，编译器会直接报错。
既然问题出在无法确定大小，那么我们可以引入间接层，来打破这里的递归定义。这就是我们使用 `Box<HuffmanTree>` 来存储左右子树的引用的原因。

`Box` 是 Rust 中的智能指针，它**把数据本身放在堆（Heap）上面，而在栈上只保留一个指向堆内存地址的指针**。
这样无论 `HuffmanTree` 有多大，指向它的指针的大小都是固定的（在 64 位系统上是 8 字节）。

此时编译器再次计算 `HuffmanTree` 的大小：

1. 编译器想知道 `HuffmanTree` 类型的大小。
2. 它去检查 `Internal` 变体的大小。
3. `Internal` 的大小 = `u64` 的大小（8 字节）+ `left` 的大小（8 字节）+ `right` 的大小（8 字节）。
4. `left` 和 `right` 都是 `Box<HuffmanTree>`，它们的大小都是固定的（8 字节）。
5. 因此，`HuffmanTree` 的大小为：`8 + 8 + 8 = 24 字节`。

方程变成了一个确定的常数，这样 Rust 编译器就能在编译期间确定 `HuffmanTree` 的大小。

## 关于智能指针的类型

我们注意到刚才使用 `Box<HuffmanTree>` 来存储左右子树的引用，这是为了避免递归定义导致的编译错误。
为什么不使用类似于 `Rc` 或者 `Arc` 这样的其他类型智能指针呢？这其实与 Rust 的**所有权语义**有关。

在 Rust 中，智能指针不仅用于内存管理，更是明确表达**所有权语义**的工具。在标准的层级树结构中，每一个子节点都有且仅有一个确定的父节点，它永远不会被树中的其他节点“共享”。
`Box` 恰恰是用来表示这种“独占所有权”的智能指针，它向编译器以及开发人员明确传达了：“这个子节点的生命周期完全绑定在它唯一的父节点上”。

另外，Huffman 树是自底向上构建的：我们不断从优先队列中挑选出两个权重最小的节点，将它们合并，作为一个全新父节点的左右子树，再把父节点放回队列。
在这个合并过程中，发生了及其明确的“所有权转移”操作：

- 将两个旧节点从队列中弹出，队列彻底交出这两个节点的所有权。
- 新诞生的父节点接管了它们，成为它们唯一的拥有者。

这是一种“一次性”的流转过程。没有谁会“共享”这两个旧节点，它们只能被合并成一个新节点，所有权的转移清晰且明确，与 `Box`
的转移语义完全相符。

## HuffmanTree 的方法定义

现在我们将实现一些 `HuffmanTree` 的方法，包括：

- `frequency`：返回节点的频率。
- `new_leaf`：创建一个新的叶子节点。
- `new_internal`：创建一个新的内部节点。

```rust
impl HuffmanTree {
    pub fn frequency(&self) -> u64 {
        match self {
            Self::Leaf { frequency, .. } => *frequency,
            Self::Internal { frequency, .. } => *frequency,
        }
    }

    pub fn min_symbol(&self) -> u8 {
        match self {
            Self::Leaf { symbol, .. } => *symbol,
            Self::Internal { min_symbol, .. } => *min_symbol,
        }
    }

    pub fn new_leaf(symbol: u8, frequency: u64) -> Self {
        Self::Leaf { symbol, frequency }
    }

    pub fn new_internal(left: Box<HuffmanTree>, right: Box<HuffmanTree>) -> Self {
        Self::Internal {
            frequency: left.frequency() + right.frequency(),
            min_symbol: left.min_symbol().min(right.min_symbol()),
            left,
            right,
        }
    }
}
```

这些方法，对于后续的 Huffman 树的构建和编码过程都是非常重要的，能够极大地简化代码的实现。

# Huffman 树的构建算法与逻辑实现

我们已经完成了 Huffman 树的类型定义，接下来我们将实现 Huffman 树的构建算法。

## 优先队列与排序策略

### 使用 BinaryHeap 实现优先队列

构建 Huffman 树的操作非常简单：不断从所有节点中，挑出最小的两个节点合并，再把合并后的节点塞回去。这个过程会一直循环，直到只剩下一个节点。

如果使用普通的数组 `Vec<HuffmanTree>` 来储存节点，每次挑选最小值都有两种笨办法：

1. **线性扫描**：每次都遍历数组，寻找最小值，这样的时间复杂度是 $O(N^2)$，效率极低。
2. **频繁排序**：每次合并后都调用一次 `sort`，性能损耗也很大。

为了高效地实现“频繁获取值并插入新值”的操作，最合适的数据结构就是**优先队列（Priority Queue）**。它能够在每次插入和弹出时自动维护顺序，保证操作十分高效。
单次操作的时间复杂度仅为 $O(\log N)$，其中 $N$ 是队列中的元素数量。

Rust 标准库为我们提供了线程的优先队列实现：`std::collections::BinaryHeap`。

但是，如果我们尝试直接构建 `BinaryHeap<HuffmanTree>`，会遇到一个编译错误：

```text
Note: the following trait bounds were not satisfied:
`HuffmanTree: Ord`
```

告诉我们，`HuffmanTree` 类型没有实现 `Ord`  trait，这是因为 `BinaryHeap` 需要能够比较元素的大小，才能正确维护堆的性质。

### 实现 Ord Trait

查看 `Ord` Trait 的为文档，我们可以发现它的定义如下：

```rust
pub const trait Ord: [ const ] Eq + [ const ] PartialOrd<Self> + PointeeSized {}
```

最后一个条件是 `PointeeSized`，它表明类型可能有大小，也可能没有大小，这一条可以忽略。
因此我们直接关注前面几条约束即可：

1. `Eq`：相等性比较。
2. `PartialOrd`：偏序比较。

其中 `Eq` 是 `PartialEq` 的特例，它要求所有相等的元素，必须有相同的偏序比较结果。
因此我们必须要实现 `PartialEq`、`Eq` 和 `PartialOrd` 三个 Trait。

因为我们只需要比较节点的频率，因此可以将这三个 Trait 的实现交给 `frequency` 方法。
它返回的 `u64` 类型，本身就支持这三个 Trait。

```rust
impl PartialEq<Self> for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.frequency() == other.frequency()
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut ord = self.frequency().cmp(&other.frequency());
        if ord == Ordering::Equal {
            ord = self.min_symbol().cmp(&other.min_symbol());
        }

        Some(ord)
    }
}

impl Eq for HuffmanTree {}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::partial_cmp(self, other).unwrap()
    }
}
```

1. 在 `PartialOrd` Trait 的比较实现中，我们先比较两个节点的频率，如果频率相等，再比较最小符号即可。
2. 因为 `PartialOrd` 的实现已经足够进行确定性比较，因此 `Ord` 的实现只需要调用 `partial_cmp` 即可。

至此我们就完成了 `HuffmanTree` 类型的 `Ord` Trait 实现，可以放心的将它放入 `BinaryHeap` 中。

### 修改排序顺序，使用最小堆

如果读者足够了解 `BinaryHeap` 类型，就会知道这是一个**最大堆**。但是我们每次需要提取出最小的两个节点，而不是最大的两个节点。
因此我们需要的是一个**最小堆**。在这里我们可以通过修改 `cmp` 方法，实现最小堆的性质。

我们只需要在 `PartialOrd` Trait 的方法中，使用 `reverse` 方法修改比较方向即可。

```rust
impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut ord = self.frequency().cmp(&other.frequency());
        if ord == Ordering::Equal {
            ord = self.min_symbol().cmp(&other.min_symbol());
        }

        Some(ord.reverse())  // 在比较结束后，将比较方向反转
    }
}
```

## 从频次表构建 Huffman 树

无论是压缩还是解压，我们最开始得到的都只是各个字节出现的频次，也就是 `FrequencyTable` 结构体。
我们需要根据这份频次表，构建出对应的 Huffman 树。如果不这样做，我们就无法得到每个字节对应的编码，压缩和解压也就无从谈起。

在前一篇文章没有提到的测试中，我们测试了空的频次表，也就是空文件情况下的二进制读写。在这里我们同样需要考虑频次表为空的情况。
一种可行的方式是使用 `From` Trait，然后使用注释说明“频次表不应为空”，在调用前先检查频次表是否为空。
但是这样显然不是最优解，每次调用都检查一次显然增加了开发时的心智负担。

因此为了保证程序的健壮性，我们将不再使用表示必然成功的 `From` Trait，而是选择使用 `TryFrom` Trait。

这个实现的定义将会是这样：

```rust
impl TryFrom<&FrequencyTable> for HuffmanTree {
    // 给出 Error 的定义和 try_from 方法的实现
}
```

读者可能会有疑问，为什么使用 `&FrequencyTable` 而不是 `FrequencyTable`？
这是因为无论是压缩还是解压，我们构建 Huffman 树时都只需要频次表的信息，而不需要频次表本身，因此不需要消耗频次表的所有权。
而且稍后我们在压缩时还能直接将频次表写入压缩包的文件头。

具体的代码实现如下：

```rust
impl TryFrom<&FrequencyTable> for HuffmanTree {
    type Error = anyhow::Error;

    fn try_from(value: &FrequencyTable) -> Result<Self, Self::Error> {
        if value.count == 0 || value.entries.is_empty() {
            anyhow::bail!("频次表不能为空")
        }

        let mut nodes = BinaryHeap::new();
        let iter = value
            .entries
            .iter()
            .map(|entry| Self::new_leaf(entry.symbol, entry.frequency));
        nodes.extend(iter);

        while nodes.len() > 1 {
            let left = nodes.pop().unwrap();
            let right = nodes.pop().unwrap();
            nodes.push(Self::new_internal(Box::new(left), Box::new(right)));
        }

        Ok(nodes.pop().unwrap())
    }
}
```

1. 使用 `anyhow::bail!` 宏可以在条件不满足时，快速返回错误。
2. 使用 `extend` 方法配合延迟求值的迭代器，比先创建 `Vec` 再 `for_each` 更符合 Rust 的编程习惯。
3. 在 `while nodes.len() > 1` 的循环中，我们已经能保证 `nodes` 最后只会有一个节点，因此最后直接使用 `nodes.pop().unwrap()`
   是合法的，不会引发崩溃。

## 生成编码表

构建完 Huffman 树之后，其实我们就可以根据树的结构，生成每个字节对应的 Huffman 编码了。
但是每次生成编码都遍历一次树的话，开销会非常大。而且既然频次已经确定下来了，我们可以先计算出每个字节的编码，在压缩时查表即可。
使用查表，可以将时间复杂度降低到 $O(1)$。

我们需要定义一个用于快速查找每个字节编码的结构。对于键值对的存储，最直观的办法是使用 `HashMap`。
但考虑到我们的键是 `u8` 类型，最多只有 256 种可能，因此我们可以使用一个固定长度的数组 `[Option<Vec<bool>>; 256]` 进行代替。
使用 `Vec<bool>` 是因为编码只有 0 和 1 两种可能，不会出现第三种情况。

如果要表示某个字符串的编码，更高效的做法是使用 `BitVec`（来自于 `bitvec` 库），但是考虑到实现的复杂度，我们现在暂时不会使用。
在后续写入到二进制文件中时，我们需要将 `Vec<bool>` 转换为按位存储的二进制数据，在那时我们将使用 `BitVec` 进行重构。

我们在 `core` 模块下创建 `codebook.rs` 文件，用于定义 `CodeBook` 结构体，作为编码表：

```rust
pub struct CodeBook {
    pub map: [Option<Vec<bool>>; 256],
}

impl CodeBook {
    pub fn get_code(&self, symbol: u8) -> Option<&Vec<bool>> {
        self.map[symbol as usize].as_ref()
    }
}
```

我们定义了 `CodeBook` 结构体，以及 `get_code` 方法，用于根据字节查找对应的编码。

为了实现 `HuffmanTree` 到 `CodeBook` 的转换，我们这里使用 `From` Trait。
因为构建完的 Huffman 树，对于每个字节，都一定会有一个确定的编码，无需进行额外的检查。
因为同样不需要转移所有权，所以我们使用 `From<&HuffmanTree>` 实现：

```rust
impl CodeBook {
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
                path.pop(); // 撤销刚才的移动，回溯到父节点，准备向右走

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
```

生成编码表的逻辑本质上是**深度优先搜索**。我们从根节点触发，每向左走一步就给路径添加上一个 `false`，每向右走一步就给路径添加上一个
`true`。
因此我们只需要维护**一个**动态数组 `path`，当我们遍历完左子树并返回父节点时，只需要调用 `path.pop()` 撤销刚才的移动，然后再尝试向右走。
这样能极大地减少内存分配的次数。

# 针对 Huffman 树和编码表的测试

在上一篇博客中，我们已经讨论了自动化测试对于二进制处理的重要性，对于 Huffman 树同样如此。
因此我们同样编写了几个测试用例，以保证从频次表到最终编码的每一跳都是准确且确定的。

我们构造了一个包含三个字符（A: 10, B: 20, C: 15）的模拟频次表。通过理论推算，频率最高的 B 应该拥有最短的编码，而 A 和
C 则会合并并获得较长的编码。同时，我们还验证了 min_symbol 逻辑是否能在频率冲突时产出唯一确定的树结构。

```rust
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

```

通过这些测试，我们不仅验证了 Huffman 编码生成的准确性，以及在面对边界条件（如空文件）时的健壮性。

在这篇文章中，我们完成了 Huffman 树的类型定义、逻辑实现，以及编码表的生成。
我们不仅在内存中构建了一棵能够跨平台复现的确定性 Huffman 树，还利用 DFS 算法产出了 $O(1)$ 查找速度的编码映射表。
至此，我们已经解决了“如何编解码”的问题。但在现实世界中，数据并不是现成的频次表，而是散落在硬盘上的原始字节流。

在下一篇文章中，我们将实现 ByteCounter，学习如何高效地扫描文件并统计字节频次，从而打通从原始文件到 Huffman
树的第一道关卡。
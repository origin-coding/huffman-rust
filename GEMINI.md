# Huffman Rust Project Context

## Project Overview
**huffman-rust** 是一个使用 Rust (Edition 2024) 开发的高效、模块化压缩工具。项目采用 **Rust Workspace** 架构，目前专注于 `huffman_core` 核心库的建设，旨在实现一套健壮、确定且可扩展的 Huffman 压缩协议。

### Core Goals & Progress
- [x] **Phase 1.1: Binary Format Design** - 完成 V1 版本协议定义，使用 `binrw` 实现声明式序列化。
- [x] **Phase 1.2: Deterministic Tree Engine** - 实现确定性 Huffman 树构建算法（引入 `min_symbol` 决胜逻辑）。
- [x] **Phase 1.3: CodeBook Generation** - 实现基于 DFS 的 $O(1)$ 查找编码映射表。
- [ ] **Phase 1.4: Stream Counter** - (Next) 实现高效的字节频次统计器（ByteCounter）。
- [ ] **Phase 2: Archive Protocol** - 支持多文件归档与目录结构。

---

## Technical Stack
- **Language**: Rust (Edition 2024)
- **Binary Processing**: `binrw` (Declarative I/O), `bitflags` (Flag management).
- **Error Handling**: `thiserror` (Library level), `anyhow` (Application flow).
- **Architecture**: Workspace-based modular design.
- **Testing**: Integration tests for format and core logic.

---

## Directory Structure
- `crates/huffman_core/`: 核心逻辑库
  - `src/core/`: 算法引擎
    - `tree.rs`: 确定性 Huffman 树实现（支持 `TryFrom<&FrequencyTable>`）。
    - `codebook.rs`: 编码本生成逻辑（DFS + 回溯）。
  - `src/format/`: 协议定义
    - `v1.rs`: V1 版本二进制格式（Magic Numbers, Headers, Footers）。
  - `tests/`: 集成测试
    - `format_v1_tests.rs`: 验证二进制序列化正确性。
    - `core_tree_tests.rs`: 验证从频次表到编码生成的全链路逻辑。
- `docs/`: 详细的工程设计日志（包含协议设计、算法原理等）。

---

## Key Engineering Conventions
1. **Determinism**: 为了保证解压一致性，Huffman 树在构建时必须处理频率相等的情况。本项目通过在 `Internal` 节点缓存 `min_symbol` 并作为 `Ord` 的次要排序键来实现确定性。
2. **Safety**: 核心库转换逻辑优先使用 `TryFrom` 配合 `anyhow`，显式处理空频次表等异常边界。
3. **Efficiency**: 
   - 使用 `Box<HuffmanTree>` 解决递归定义的大小问题。
   - `CodeBook` 采用固定长度数组 `[Option<Vec<bool>>; 256]` 实现 $O(1)$ 查表。
4. **Documentation**: 维护 `docs/` 下的 Mermaid 图表源码与导出的 SVG，确保架构设计的可视化与同步。

---

## Development Workflow
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Clippy**: `cargo clippy` (保持零警告)
- **Diagrams**: 使用 `mmdc` 将 `docs/diagrams/*.mmd` 导出至 `docs/images/*.svg`。

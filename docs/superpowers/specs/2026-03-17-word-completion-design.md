# 词频补全功能设计

## 概述

在无 LSP 支持的 buffer 中，提供基于已输入词汇的补全功能。

## 需求

- **触发方式**：自动触发，作为 LSP 的后备
- **词汇来源**：从所有打开的 Buffer 中收集
- **排序方式**：MRU（最近使用优先）
- **LSP 共存**：仅当无 LSP 时显示词频补全

## 设计

### 数据结构

```rust
// 新增文件: src/word_completion.rs

use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct WordCompletion {
    /// 词 → 最后使用时间
    words: HashMap<String, Instant>,
}

impl WordCompletion {
    pub fn new() -> Self;

    /// 从 buffer 内容更新词频
    pub fn update_from_buffer(&mut self, content: &str);

    /// 记录刚输入的词
    pub fn record_word(&mut self, word: String);

    /// 获取补全候选（按 MRU 排序）
    pub fn get_completions(&self, prefix: &str, limit: usize) -> Vec<String>;
}
```

### 集成流程

```
┌──────────────────────────────────────────────────────────┐
│                        App                               │
│                                                          │
│  word_completion: WordCompletion                        │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Dispatch::RequestCompletionDebounced             │  │
│  │                                                    │  │
│  │    ① 检查是否有 LSP 支持                          │  │
│  │         ↓                                          │  │
│  │    ② 有 LSP → 发送 LSP 请求                       │  │
│  │    ③ 无 LSP → 从 word_completion 获取候选        │  │
│  │         ↓                                          │  │
│  │    ④ 发送到 SuggestiveEditor                      │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Buffer 内容变化时更新                             │  │
│  │                                                    │  │
│  │    ① 打开文件 → 初始扫描                          │  │
│  │    ② Insert 模式输入 → 增量更新当前词             │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### 词汇提取规则

- 使用正则 `\b[a-zA-Z_][a-zA-Z0-9_]*\b` 提取单词
- 忽略长度 ≤ 2 的词
- 最大保留 1000 个词（避免内存过大）

### 文件结构

```
src/
├── word_completion.rs     # 新增：词频补全模块
├── app.rs                 # 修改：集成 WordCompletion
└── lib.rs                 # 修改：添加模块声明
```

## 实现要点

1. **判断 LSP 支持**：检查 `lsp_manager.lsp_server_process_channels` 是否有对应语言的 channel
2. **初始扫描**：在 `open_file` 时扫描 buffer 内容
3. **增量更新**：在 Insert 模式下，当用户输入非单词字符（如空格）时，记录刚输入的词
4. **补全项格式**：将词频补全转换为 `DropdownItem`，与 LSP 补全格式一致

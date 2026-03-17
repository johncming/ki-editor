# Word Completion Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在无 LSP 支持的 buffer 中，提供基于已输入词汇的补全功能。

**Architecture:** 创建 `WordCompletion` 模块维护全局词频索引，在 `LspManager` 中添加判断 LSP 支持的方法，修改 `RequestCompletionDebounced` 逻辑：有 LSP 则用 LSP，无 LSP 则用词频补全。

**Tech Stack:** Rust, regex, std::collections::HashMap

---

## File Structure

| File | Action | Description |
|------|--------|-------------|
| `src/word_completion.rs` | Create | 词频补全核心模块 |
| `src/lib.rs` | Modify | 添加 `mod word_completion` |
| `src/lsp/manager.rs` | Modify | 添加 `has_lsp_support` 方法 |
| `src/app.rs` | Modify | 集成 `WordCompletion`，修改补全逻辑 |

---

## Task 1: Create WordCompletion Module

**Files:**
- Create: `src/word_completion.rs`

- [ ] **Step 1: Write WordCompletion struct with basic methods**

```rust
use std::collections::HashMap;
use std::time::Instant;
use regex::Regex;

/// 词频补全模块
/// 从所有打开的 buffer 中收集词汇，按最近使用时间排序
#[derive(Debug, Clone)]
pub struct WordCompletion {
    /// 词 → 最后使用时间
    words: HashMap<String, Instant>,
    /// 最大保留词汇数
    max_words: usize,
    /// 最小词长
    min_word_length: usize,
}

impl Default for WordCompletion {
    fn default() -> Self {
        Self::new()
    }
}

impl WordCompletion {
    pub fn new() -> Self {
        Self {
            words: HashMap::new(),
            max_words: 1000,
            min_word_length: 3,
        }
    }

    /// 从文本中提取词汇并更新索引
    pub fn update_from_text(&mut self, text: &str) {
        let re = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
        for cap in re.find_iter(text) {
            let word = cap.as_str();
            if word.len() >= self.min_word_length {
                self.words.insert(word.to_string(), Instant::now());
            }
        }

        // 如果超过最大数量，移除最旧的词
        if self.words.len() > self.max_words {
            self.trim_oldest();
        }
    }

    /// 记录单个词（用于增量更新）
    pub fn record_word(&mut self, word: String) {
        if word.len() >= self.min_word_length {
            self.words.insert(word, Instant::now());
        }
    }

    /// 获取补全候选（按 MRU 排序）
    pub fn get_completions(&self, prefix: &str, limit: usize) -> Vec<String> {
        let prefix_lower = prefix.to_lowercase();

        let mut matches: Vec<_> = self
            .words
            .iter()
            .filter(|(word, _)| {
                word.to_lowercase().starts_with(&prefix_lower) && word != prefix
            })
            .collect();

        // 按最近使用时间排序（最新的在前）
        matches.sort_by(|a, b| b.1.cmp(a.1));

        matches
            .into_iter()
            .take(limit)
            .map(|(word, _)| word.clone())
            .collect()
    }

    /// 移除最旧的词汇
    fn trim_oldest(&mut self) {
        if self.words.len() <= self.max_words {
            return;
        }

        let mut entries: Vec<_> = self.words.iter().collect();
        entries.sort_by_key(|(_, time)| *time);

        let to_remove = entries.len() - self.max_words;
        for (word, _) in entries.into_iter().take(to_remove) {
            self.words.remove(word);
        }
    }

    /// 清空词汇索引
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.words.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_update_from_text() {
        let mut wc = WordCompletion::new();
        wc.update_from_text("hello world foo bar");

        let completions = wc.get_completions("he", 10);
        assert!(completions.contains(&"hello".to_string()));
    }

    #[test]
    fn test_get_completions_case_insensitive() {
        let mut wc = WordCompletion::new();
        wc.update_from_text("HelloWorld");

        let completions = wc.get_completions("hello", 10);
        assert!(completions.contains(&"HelloWorld".to_string()));
    }

    #[test]
    fn test_mru_ordering() {
        let mut wc = WordCompletion::new();

        wc.update_from_text("first_word");
        sleep(Duration::from_millis(10));
        wc.update_from_text("second_word");
        sleep(Duration::from_millis(10));
        wc.update_from_text("third_word");

        let completions = wc.get_completions("", 10);
        assert_eq!(completions[0], "third_word");
        assert_eq!(completions[1], "second_word");
        assert_eq!(completions[2], "first_word");
    }

    #[test]
    fn test_min_word_length() {
        let mut wc = WordCompletion::new();
        wc.update_from_text("hi hello"); // "hi" 只有 2 个字符

        let completions = wc.get_completions("h", 10);
        assert!(completions.contains(&"hello".to_string()));
        assert!(!completions.contains(&"hi".to_string()));
    }

    #[test]
    fn test_exclude_exact_match() {
        let mut wc = WordCompletion::new();
        wc.update_from_text("hello");

        let completions = wc.get_completions("hello", 10);
        assert!(!completions.contains(&"hello".to_string()));
    }
}
```

- [ ] **Step 2: Run tests to verify implementation**

Run: `cargo test word_completion::`
Expected: All tests pass

---

## Task 2: Add has_lsp_support Method to LspManager

**Files:**
- Modify: `src/lsp/manager.rs`

- [ ] **Step 1: Add has_lsp_support method**

在 `LspManager` impl 块中添加：

```rust
/// 检查指定路径是否有 LSP 支持
pub fn has_lsp_support(&self, path: &AbsolutePath) -> bool {
    crate::config::from_path(path)
        .and_then(|language| {
            let language_id = language.id()?;
            let channel = self.lsp_server_process_channels.get(&language_id)?;
            if channel.is_initialized() {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(false)
}
```

- [ ] **Step 2: Run tests to verify no breakage**

Run: `cargo test --lib`
Expected: All tests pass

---

## Task 3: Integrate WordCompletion into App

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add module declaration in lib.rs**

在 `src/lib.rs` 中添加：

```rust
mod word_completion;
```

- [ ] **Step 2: Add word_completion field to App struct**

在 `src/app.rs` 的 `App` struct 中添加字段：

```rust
// 在 App struct 中添加
word_completion: crate::word_completion::WordCompletion,
```

- [ ] **Step 3: Initialize word_completion in App::from_channel**

在 `App::from_channel` 方法中，初始化 `word_completion`：

```rust
// 在 App 初始化时
word_completion: crate::word_completion::WordCompletion::new(),
```

- [ ] **Step 4: Run tests to verify compilation**

Run: `cargo build`
Expected: Compilation succeeds

---

## Task 4: Modify Completion Request Logic

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Update RequestCompletionDebounced handler**

在 `src/app.rs` 中找到 `Dispatch::RequestCompletionDebounced` 处理（约 755-762 行），将现有代码替换为：

找到这段代码（约 755-762 行）：
```rust
Dispatch::RequestCompletionDebounced => {
    if let Some(params) = self.get_request_params() {
        self.lsp_manager().send_message(
            params.path.clone(),
            FromEditor::TextDocumentCompletion(params),
        )?;
    }
}
```

替换为：
```rust
Dispatch::RequestCompletionDebounced => {
    if let Some(params) = self.get_request_params() {
        // 检查是否有 LSP 支持
        if self.lsp_manager().has_lsp_support(&params.path) {
            // 有 LSP 支持，发送 LSP 请求
            self.lsp_manager().send_message(
                params.path.clone(),
                FromEditor::TextDocumentCompletion(params),
            )?;
        } else {
            // 无 LSP 支持，使用词频补全
            let current_word = self.current_component()
                .borrow()
                .editor()
                .get_current_word()
                .unwrap_or_default();
            let completions = self.word_completion.get_completions(&current_word, 50);

            if !completions.is_empty() {
                let completion = self.word_completions_to_completion(completions);
                self.handle_dispatch(Dispatch::ToSuggestiveEditor(
                    DispatchSuggestiveEditor::Completion(completion),
                ))?;
            }
        }
    }
}
```

- [ ] **Step 2: Add helper method to App**

在 `App` impl 块中添加辅助方法（注意：`editor.get_current_word()` 已存在于 `src/components/editor.rs:824`）：

```rust
/// 将词汇列表转换为 Completion
fn word_completions_to_completion(&self, words: Vec<String>) -> Completion {
    use crate::components::dropdown_sync::DropdownItem;
    use crate::lsp::completion::CompletionItem;

    let items: Vec<DropdownItem> = words
        .into_iter()
        .map(|word| {
            // 手动构造 CompletionItem（from_label 是测试专用方法）
            let completion_item = CompletionItem {
                label: word.clone(),
                kind: None,
                detail: None,
                documentation: None,
                sort_text: None,
                insert_text: Some(word.clone()),
                edit: None,
                completion_item: lsp_types::CompletionItem {
                    label: word,
                    ..Default::default()
                },
            };
            DropdownItem::from(completion_item)
        })
        .collect();

    Completion {
        items,
        trigger_characters: vec![],
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib`
Expected: All tests pass

---

## Task 5: Update Word Index on File Open and Content Change

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Update word index when opening file**

在 `src/app.rs` 的 `open_file` 方法（约 1487 行）中，在获取 `content` 之后（约 1520 行附近）添加词频索引更新：

找到这段代码（约 1519-1520 行）：
```rust
let language = buffer.language();
let content = buffer.content();
```

在其后添加：
```rust
// 更新词频索引
self.word_completion.update_from_text(&content);
```

- [ ] **Step 2: Handle DocumentDidChange to update word index**

在 `src/app.rs` 中找到 `Dispatch::DocumentDidChange` 处理（约 908 行），在 `lsp_manager().send_message` 调用之后（约 931 行附近）添加：

找到这段代码（约 931 行）：
```rust
                    FromEditor::TextDocumentDidChange {
                        content,
                        file_path: path,
                        version: 2,
                    },
                )?;
```

在其后添加：
```rust
                // 更新词频索引
                self.word_completion.update_from_text(&content);
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib`
Expected: All tests pass

---

## Task 6: Write Integration Tests

**Files:**
- Create: Test cases in `src/word_completion.rs` (tests module)

- [ ] **Step 1: Write integration test for word completion without LSP**

在 `src/word_completion.rs` 的 `#[cfg(test)] mod tests` 块中添加：

```rust
use crate::word_completion::WordCompletion;

#[test]
fn test_word_completion_integration() {
    let mut wc = WordCompletion::new();

    // 模拟打开文件时扫描内容
    wc.update_from_text("hello world rust programming");
    wc.update_from_text("world is beautiful");

    // 测试前缀匹配
    let completions = wc.get_completions("he", 10);
    assert!(completions.contains(&"hello".to_string()));

    // 测试 MRU 排序 - world 最近被更新
    let completions = wc.get_completions("w", 10);
    assert_eq!(completions[0], "world");

    // 测试大小写不敏感
    let completions = wc.get_completions("RU", 10);
    assert!(completions.iter().any(|w| w.to_lowercase().starts_with("ru")));
}

#[test]
fn test_word_completion_excludes_short_words() {
    let mut wc = WordCompletion::new();
    wc.update_from_text("hi hello a b c programming");

    let completions = wc.get_completions("h", 10);
    // "hi" 长度为 2，应该被排除
    assert!(!completions.iter().any(|w| w == "hi"));
    // "hello" 长度为 5，应该被包含
    assert!(completions.contains(&"hello".to_string()));
}

#[test]
fn test_word_completion_limit() {
    let mut wc = WordCompletion::new();
    wc.update_from_text("word1 word2 word3 word4 word5");

    let completions = wc.get_completions("word", 3);
    assert_eq!(completions.len(), 3);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test word_completion::`
Expected: All tests pass

- [ ] **Step 3: Commit all changes**

```bash
git add -A
git commit -m "feat: add word-based completion for buffers without LSP support

- Add WordCompletion module for managing word frequency index
- Add has_lsp_support method to LspManager
- Integrate word completion as fallback when no LSP available
- Support MRU (Most Recently Used) ordering
- Collect words from all open buffers

johncming@126.com"
```

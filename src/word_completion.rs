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
                word.to_lowercase().starts_with(&prefix_lower) && word.as_str() != prefix
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
        let words_to_remove: Vec<String> = entries
            .into_iter()
            .take(to_remove)
            .map(|(word, _)| word.clone())
            .collect();

        for word in words_to_remove {
            self.words.remove(&word);
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
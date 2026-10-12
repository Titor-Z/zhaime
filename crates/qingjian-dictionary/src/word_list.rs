use std::path::Path;

use crate::error::DictionaryError;

/// 英文词表：按小写编码查原样大小写的词，也能按前缀补全。
/// 文件格式 TSV：`词\t编码[\t词频]`（编码缺省为词的小写，词频缺省 0）。
///
/// 内存上整表只有两处分配：一个 `String` 装全部词与编码（下面 `buffer`），外加一个
/// `Vec<Entry>`。词和编码都用 `(偏移, 长度)` 指回 `buffer`，不各自复制 `String`。
/// `entries` 按编码字节序排好，查词与前缀补全都二分定位，不需要额外的索引表。
///
/// 9 万个英文词的表因此从「每词 3 次堆分配」降到每词 0 次，堆占用从 5.3 MB
/// 降到条目数组本身（20 字节/词，约 1.9 MB）加一次性读入的源文本。
pub struct WordList {
    /// 词与编码的原文，词目按 `(偏移, 长度)` 指回这里。
    buffer: String,
    /// (词偏移, 词长, 编码偏移, 编码长, 词频)，按编码字节序。编码是小写。
    entries: Vec<Entry>,
}

/// 一个词目在 `buffer` 里的位置与词频。
#[derive(Clone, Copy)]
struct Entry {
    word: Span,
    code: Span,
    frequency: u32,
}

/// `buffer` 里的一段字节。
#[derive(Clone, Copy)]
struct Span {
    offset: u32,
    length: u32,
}

impl std::fmt::Debug for WordList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WordList")
            .field("words", &self.entries.len())
            .finish()
    }
}

impl Entry {
    /// 取出该词目的编码与词，都是 `buffer` 里的切片。
    fn parts<'a>(&self, buffer: &'a str) -> (&'a str, &'a str) {
        (&buffer[self.code.range()], &buffer[self.word.range()])
    }
}

impl Span {
    fn range(&self) -> std::ops::Range<usize> {
        self.offset as usize..self.offset as usize + self.length as usize
    }
}

/// 往 `buffer` 追加一段，返回它的位置。TSV 扫描时用它把词和编码各存一份。
fn push(buffer: &mut String, part: &str) -> Span {
    let offset = buffer.len();
    buffer.push_str(part);
    Span {
        offset: offset as u32,
        length: part.len() as u32,
    }
}

impl WordList {
    pub fn parse(source: &str) -> Result<Self, DictionaryError> {
        // 词和编码不指向 `source`（source 是调用方的、编码还可能是小写化后的），
        // 而是指向自己攒的 `buffer`，两者都只存一份。
        let mut buffer = String::with_capacity(source.len() + source.len() / 4);
        let mut entries: Vec<Entry> = Vec::new();
        for (index, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.split('\t');
            let word = fields
                .next()
                .filter(|s| !s.is_empty())
                .ok_or(DictionaryError::Line {
                    line: index + 1,
                    reason: "missing word",
                })?;
            // 编码缺省为词的小写；显式给的也一律小写化，和查询端一致。
            let code = fields
                .next()
                .filter(|s| !s.is_empty())
                .map(str::to_ascii_lowercase)
                .unwrap_or_else(|| word.to_ascii_lowercase());
            let frequency = fields
                .next()
                .map(|s| s.trim().parse::<u32>())
                .transpose()
                .map_err(|_| DictionaryError::Line {
                    line: index + 1,
                    reason: "frequency is not a non-negative integer",
                })?
                .unwrap_or(0);
            let word = push(&mut buffer, word);
            let code = push(&mut buffer, &code);
            entries.push(Entry {
                word,
                code,
                frequency,
            });
        }
        // 按编码排序：查词与前缀补全都二分，排序前不查哈希表。
        entries.sort_by(|a, b| {
            let (ac, _) = a.parts(&buffer);
            let (bc, _) = b.parts(&buffer);
            ac.cmp(bc)
        });
        // 同一编码只留一个（词表按常用度排序时第一个即最常用的写法）：排序后线性去重。
        entries.dedup_by(|a, b| {
            let (ac, _) = a.parts(&buffer);
            let (bc, _) = b.parts(&buffer);
            ac == bc
        });
        tracing::debug!(words = entries.len(), "英文词表加载完成");
        Ok(Self { buffer, entries })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        Self::parse(&std::fs::read_to_string(path)?)
    }

    /// 输入（已小写）对应的英文词。
    pub fn get(&self, code: &str) -> Option<&str> {
        let index = self.position(code)?;
        Some(self.entries[index].parts(&self.buffer).1)
    }

    /// 输入（已小写）对应的词频；词表里没有为 `None`。
    pub fn frequency(&self, code: &str) -> Option<u32> {
        let index = self.position(code)?;
        Some(self.entries[index].frequency)
    }

    /// 以 `prefix` 开头（不含正好相等的）的词里词频最高的 `limit` 个，按词频降序。
    pub fn complete(&self, prefix: &str, limit: usize) -> Vec<&str> {
        if prefix.is_empty() || limit == 0 {
            return Vec::new();
        }
        let buffer = &self.buffer;
        let start = self
            .entries
            .partition_point(|entry| entry.parts(buffer).0 < prefix);
        let mut hits: Vec<Entry> = self.entries[start..]
            .iter()
            .take_while(|entry| entry.parts(buffer).0.starts_with(prefix))
            .filter(|entry| entry.parts(buffer).0 != prefix)
            .copied()
            .collect();
        hits.sort_by(|a, b| {
            b.frequency
                .cmp(&a.frequency)
                .then_with(|| a.parts(buffer).0.cmp(b.parts(buffer).0))
        });
        hits.into_iter()
            .take(limit)
            .map(|entry| entry.parts(buffer).1)
            .collect()
    }

    /// 全部词目：(小写编码, 原样词, 词频)，按编码字节序。
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str, u32)> {
        let buffer = &self.buffer;
        self.entries.iter().map(move |entry| {
            let (code, word) = entry.parts(buffer);
            (code, word, entry.frequency)
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 二分定位编码正好等于 `code` 的词目。
    fn position(&self, code: &str) -> Option<usize> {
        let buffer = &self.buffer;
        self.entries
            .binary_search_by(|entry| entry.parts(buffer).0.cmp(code))
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_up_by_lowercase_code() {
        let list = WordList::parse("GitHub\tgithub\nhello\thello\niPhone\n").unwrap();
        assert_eq!(list.get("github"), Some("GitHub"));
        assert_eq!(list.get("iphone"), Some("iPhone"));
        assert_eq!(list.get("hello"), Some("hello"));
        assert_eq!(list.get("nope"), None);
    }

    #[test]
    fn completes_prefixes_by_frequency() {
        let list = WordList::parse(
            "compass\tcompass\t300\ncompany\tcompany\t900\ncompare\tcompare\t500\ncom\tcom\t100\ncomma\tcomma\n",
        )
        .unwrap();
        assert_eq!(list.complete("comp", 2), ["company", "compare"]);
        assert_eq!(
            list.complete("com", 10),
            ["company", "compare", "compass", "comma"]
        );
        assert!(list.complete("zzz", 3).is_empty());
        assert!(list.complete("", 3).is_empty());
    }

    #[test]
    fn from_path_matches_parse() {
        let dir = std::env::temp_dir().join("qj-wordlist-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("words.tsv");
        let text = "GitHub\tgithub\t900\niPhone\tiphone\t800\napple\tapple\t700\n";
        std::fs::write(&path, text).unwrap();

        let parsed = WordList::parse(text).unwrap();
        let mapped = WordList::from_path(&path).unwrap();

        assert_eq!(mapped.len(), parsed.len());
        let from_file: Vec<_> = mapped.entries().collect();
        let from_str: Vec<_> = parsed.entries().collect();
        assert_eq!(from_file, from_str);
        assert_eq!(mapped.get("github"), Some("GitHub"));
        assert_eq!(mapped.complete("a", 10), ["apple"]);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn duplicate_code_keeps_first_entry() {
        // 排序后按编码去重，保留的是「第一条原始记录」——排序稳定时即原表里先出现的那条。
        let list = WordList::parse("Foo\tfoo\t900\nfoo\tfoo\t1\n").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list.get("foo"), Some("Foo"));
    }
}

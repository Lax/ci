use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use jieba_rs::Jieba;

use crate::db::entry::{Entry, EntrySource};
use crate::db::store::FreqDb;

/// Scan a local repo for Chinese text, return scan-result FreqDb for merging.
pub fn scan_repo(repo_path: &str) -> Result<FreqDb> {
    let repo = Path::new(repo_path);
    if !repo.exists() {
        log::warn!("repo not found: {repo_path}");
        return Ok(FreqDb::new(crate::db::device::Device {
            id: String::new(),
            name: String::new(),
            total_entries: 0,
        }));
    }

    let jieba = Jieba::new();
    let repo_name = repo
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| repo_path.to_string());

    let mut word_freq: HashMap<String, u32> = HashMap::new();
    let mut bigrams: HashMap<String, Vec<String>> = HashMap::new();
    let mut files_scanned = 0u32;

    walk_files(repo, &mut |path| {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "md" | "txt" | "markdown") {
            return;
        }
        if path
            .components()
            .any(|c| c.as_os_str().to_str().map_or(false, |s| s.starts_with('.')))
        {
            return;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let text = if ext == "md" || ext == "markdown" {
            strip_code_blocks(&content)
        } else {
            content
        };

        let text = text.trim();
        if text.is_empty() {
            return;
        }

        let words: Vec<&str> = jieba.cut(text, true);
        let chinese_words: Vec<&&str> = words
            .iter()
            .filter(|w| is_chinese(w) && w.len() >= 2)
            .collect();

        for w in &chinese_words {
            *word_freq.entry((*w).to_string()).or_insert(0) += 1;
        }

        for pair in chinese_words.windows(2) {
            let prev = pair[0].to_string();
            let word = pair[1].to_string();
            bigrams.entry(word).or_default().push(prev);
        }

        files_scanned += 1;
    })?;

    let rel_path = repo.to_string_lossy().to_string();
    let now = Utc::now();
    let mut entries = Vec::with_capacity(word_freq.len());

    for (word, freq) in &word_freq {
        let prev = bigrams.get(word).and_then(|v| v.first().cloned());
        entries.push(Entry {
            code: word.clone(),
            word: word.clone(),
            freq: *freq,
            updated: now,
            prev,
            source: EntrySource::Scan {
                repo: repo_name.clone(),
                path: rel_path.clone(),
            },
        });
    }

    log::info!(
        "scanned {files_scanned} files in {repo_path}, found {} words",
        entries.len()
    );

    let mut result = FreqDb::new(crate::db::device::Device {
        id: String::new(),
        name: repo_name,
        total_entries: entries.len() as u32,
    });
    result.entries = entries;
    Ok(result)
}

fn walk_files(dir: &Path, cb: &mut dyn FnMut(&Path)) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            walk_files(&path, cb)?;
        } else if entry.file_type()?.is_file() {
            cb(&path);
        }
    }
    Ok(())
}

fn strip_code_blocks(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_code_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn is_chinese(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| {
        matches!(c,
            '\u{4E00}'..='\u{9FFF}' |
            '\u{3400}'..='\u{4DBF}' |
            '\u{2E80}'..='\u{2EFF}' |
            '\u{3000}'..='\u{303F}' |
            '\u{FF00}'..='\u{FFEF}'
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chinese_pure() {
        assert!(is_chinese("你好"));
        assert!(is_chinese("世界"));
    }

    #[test]
    fn test_is_chinese_mixed() {
        assert!(!is_chinese("hello你好"));
        assert!(!is_chinese("test123"));
        assert!(!is_chinese(""));
        assert!(!is_chinese("a"));
    }

    #[test]
    fn test_strip_code_blocks() {
        let md = "\
# Title
some text
```rust
fn main() {}
```
more text
";
        let result = strip_code_blocks(md);
        assert!(result.contains("some text"));
        assert!(result.contains("more text"));
        assert!(!result.contains("fn main()"));
    }

    #[test]
    fn test_strip_code_blocks_no_blocks() {
        let md = "just plain text\nno code";
        let result = strip_code_blocks(md);
        assert_eq!(result, "just plain text\nno code\n");
    }

    #[test]
    fn test_scan_repo_nonexistent() {
        let result = scan_repo("/nonexistent/path");
        assert!(result.is_ok());
        assert!(result.unwrap().entries.is_empty());
    }

    #[test]
    fn test_scan_repo_with_files() {
        // Use a non-hidden temp path (tempfile creates .tmp prefixed dirs)
        let dir = std::path::Path::new("/tmp/opencode/scan-test");
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).unwrap();

        // jieba-rs 0.7.4 cuts "你好世界" into ["你好", "世界"]
        let md_content = "你好世界";
        std::fs::write(dir.join("test.md"), md_content).unwrap();
        std::fs::write(dir.join("notes.txt"), "hello world").unwrap();
        std::fs::write(dir.join(".hidden.md"), "隐藏文件").unwrap();

        let result = scan_repo(dir.to_string_lossy().as_ref()).unwrap();
        std::fs::remove_dir_all(dir).unwrap();

        assert_eq!(result.entries.len(), 2, "expected 2 words from '你好世界'");
    }
}

use anyhow::Result;

use crate::db::store::FreqDb;

/// Scan local git repos for Chinese text,
/// extract word frequency with context.
pub fn scan_repo(_freq_db: &mut FreqDb, _repo_path: &str) -> Result<()> {
    // TODO:
    // 1. List .md, .txt files
    // 2. Parse front matter, skip code blocks
    // 3. Use jieba-rs for Chinese word segmentation
    // 4. Extract bigrams (prev, word)
    // 5. Merge into freq_db with source=Scan
    Ok(())
}

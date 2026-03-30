use std::path::{Path, PathBuf};

use super::{TextToolApp, SearchResult, rfd_save_file, rfd_pick_folder};

// ── Full-text search ──────────────────────────────────────────────────────────

impl TextToolApp {
    /// Scan all `.md` and `.json` files under the project root for
    /// `self.search_query` and populate `self.search_results`.
    pub(super) fn run_search(&mut self) {
        self.search_results.clear();
        let query = self.search_query.clone();
        if query.is_empty() {
            return;
        }
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        search_dir(&root, &query, &mut self.search_results);
        self.status = format!(
            "搜索「{}」找到 {} 处结果",
            query,
            self.search_results.len()
        );
    }

    // ── Export & Backup ───────────────────────────────────────────────────────

    /// Concatenate all `chapters/*.md` files in alphabetical order and save to a
    /// user-chosen file via a save-file dialog.
    pub(super) fn export_chapters_merged(&mut self) {
        let Some(root) = self.project_root.as_ref() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content_dir = root.join("Content");
        let mut md_files: Vec<PathBuf> = std::fs::read_dir(&content_dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect();
        md_files.sort();

        let mut merged = String::new();
        for path in &md_files {
            if let Ok(text) = std::fs::read_to_string(path) {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                merged.push_str(&format!("# ── {name} ──\n\n"));
                merged.push_str(&text);
                merged.push_str("\n\n");
            }
        }

        let dummy = PathBuf::from("merged.md");
        if let Some(dest) = rfd_save_file(&dummy) {
            match std::fs::write(&dest, &merged) {
                Ok(_) => self.status = format!("已导出合集到 {}", dest.display()),
                Err(e) => self.status = format!("导出失败: {e}"),
            }
        }
    }

    /// Copy the entire project folder to a user-selected destination directory.
    pub(super) fn backup_project(&mut self) {
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let Some(dest_parent) = rfd_pick_folder() else {
            return;
        };
        let folder_name = root.file_name().unwrap_or_default();
        let dest = dest_parent.join(folder_name);
        match copy_dir_all(&root, &dest) {
            Ok(_) => self.status = format!("已备份到 {}", dest.display()),
            Err(e) => self.status = format!("备份失败: {e}"),
        }
    }

    /// Export all `chapters/*.md` chapters as a single plain-text `.txt` file,
    /// stripping Markdown syntax markers.
    pub(super) fn export_plain_text(&mut self) {
        let Some(root) = self.project_root.as_ref() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content_dir = root.join("chapters");
        let mut md_files: Vec<PathBuf> = std::fs::read_dir(&content_dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect();
        md_files.sort();

        let mut merged = String::new();
        for path in &md_files {
            if let Ok(text) = std::fs::read_to_string(path) {
                merged.push_str(&text);
                merged.push_str("\n\n");
            }
        }

        let plain = markdown_to_plain_text(&merged);
        let dummy = PathBuf::from("novel.txt");
        if let Some(dest) = rfd_save_file(&dummy) {
            match std::fs::write(&dest, &plain) {
                Ok(_) => self.status = format!("已导出纯文本: {}", dest.display()),
                Err(e) => self.status = format!("导出纯文本失败: {e}"),
            }
        }
    }
}

// ── File utilities ────────────────────────────────────────────────────────────

/// Recursively scan `dir` for lines in `.md` / `.json` files that contain
/// `query`.  Results are appended to `results`.
pub(super) fn search_dir(dir: &Path, query: &str, results: &mut Vec<SearchResult>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            search_dir(&path, query, results);
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "md" || ext == "json" {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    for (line_no, line) in text.lines().enumerate() {
                        if line.contains(query) {
                            results.push(SearchResult {
                                file_path: path.clone(),
                                line_no: line_no + 1,
                                line: line.to_owned(),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Recursively copy directory `src` to `dst`, creating it if necessary.
pub(super) fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

/// Count the meaningful words in a Markdown string:
/// - Each CJK Unified Ideograph (U+4E00–U+9FFF and CJK Extension A) counts as 1 word.
/// - Consecutive ASCII/Latin alphabetic characters count as 1 word-token.
/// - Punctuation, whitespace, and Markdown syntax are ignored.
///
/// This mirrors common "字数" counting conventions used for Chinese-language fiction.
pub(in crate::app) fn count_words(md: &str) -> usize {
    let plain = markdown_to_plain_text(md);
    let mut count = 0usize;
    let mut in_latin_word = false;
    for ch in plain.chars() {
        let cp = ch as u32;
        if (0x4E00..=0x9FFF).contains(&cp)   // CJK Unified Ideographs
            || (0x3400..=0x4DBF).contains(&cp) // CJK Extension A
            || (0x20000..=0x2A6DF).contains(&cp) // CJK Extension B
        {
            count += 1;
            in_latin_word = false;
        } else if ch.is_alphabetic() {
            if !in_latin_word {
                count += 1;
            }
            in_latin_word = true;
        } else {
            in_latin_word = false;
        }
    }
    count
}

/// Sum word counts for all `.md` files in `dir`.
pub(in crate::app) fn count_words_in_dir(dir: &std::path::Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else { return 0 };
    entries.flatten().filter_map(|e| {
        let p = e.path();
        if p.extension().and_then(|ext| ext.to_str()) == Some("md") {
            std::fs::read_to_string(&p).ok().map(|t| count_words(&t))
        } else {
            None
        }
    }).sum()
}


///
/// Handles:
/// - ATX headings (`# Title` → `Title`)
/// - Setext heading underlines (`===`, `---` lines are skipped)
/// - Bold/italic asterisks and inline code backticks
/// - Blockquote prefixes (`>`)
/// - Unordered list prefixes (`- `, `* `, `+ `)
/// - Ordered list prefixes (`1. `, `10. `, …)
/// - Inline links `[text](url)` → `text`
pub(super) fn markdown_to_plain_text(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    for line in md.lines() {
        // Skip Setext-style heading underline rows (only `=` or `-` chars, ≥3)
        let trimmed = line.trim();
        if trimmed.len() >= 3
            && (trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-'))
        {
            continue;
        }

        // Remove ATX heading prefix (any number of leading `#` followed by space)
        let line = line.trim_start_matches('#').trim_start();

        // Remove blockquote prefix
        let line = line.trim_start_matches('>').trim_start();

        // Remove unordered list prefix (- / * / + followed by a space)
        let line = if let Some(rest) = line.strip_prefix("- ")
            .or_else(|| line.strip_prefix("* "))
            .or_else(|| line.strip_prefix("+ "))
        { rest } else { line };

        // Remove ordered list prefix (digits followed by ". ")
        let line: &str = if let Some(dot_pos) = line.find(". ") {
            let prefix = &line[..dot_pos];
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                &line[dot_pos + 2..]
            } else {
                line
            }
        } else {
            line
        };

        // Convert the rest to an owned String so we can do replacements.
        // Strip inline links `[text](url)` → `text`
        let mut line = line.to_owned();
        while let (Some(open_br), Some(close_br)) = (line.find('['), line.find(']')) {
            if close_br > open_br {
                // Check for `(url)` immediately after `]`
                let after_br = close_br + 1;
                if line[after_br..].starts_with('(') {
                    if let Some(close_par) = line[after_br..].find(')') {
                        let text = line[open_br + 1..close_br].to_owned();
                        let end = after_br + close_par + 1;
                        line.replace_range(open_br..end, &text);
                        continue;
                    }
                }
            }
            break;
        }
        // Remove bold/italic markers and inline code backticks
        let line = line.replace("**", "").replace('*', "").replace('`', "");

        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    // ── helper: create a unique temp directory for each test ──────────────────

    fn tmp_dir(suffix: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("qingmo_search_test_{suffix}"));
        let _ = std::fs::remove_dir_all(&dir); // clean slate
        std::fs::create_dir_all(&dir).expect("temp dir creation");
        dir
    }

    // ── count_words ───────────────────────────────────────────────────────────

    /// Pure CJK text: each character is one word.
    #[test]
    fn test_count_words_cjk_only() {
        assert_eq!(count_words("你好世界"), 4);
        assert_eq!(count_words("第一章"), 3);
    }

    /// Pure ASCII: space-separated words.
    #[test]
    fn test_count_words_ascii_words() {
        assert_eq!(count_words("hello world foo"), 3);
        assert_eq!(count_words("Hello"), 1);
    }

    /// Mixed CJK + ASCII.
    #[test]
    fn test_count_words_mixed() {
        // "主角 entered the 森林" → 主(1) 角(2) entered(3) the(4) 森(5) 林(6)
        assert_eq!(count_words("主角 entered the 森林"), 6);
    }

    /// Markdown syntax is stripped before counting.
    #[test]
    fn test_count_words_strips_markdown() {
        // "**你好**" → just "你好" → 2 words
        assert_eq!(count_words("**你好**"), 2);
        // "# 第一章" → "第一章" → 3 words
        assert_eq!(count_words("# 第一章"), 3);
    }

    /// Punctuation is not counted.
    #[test]
    fn test_count_words_ignores_punctuation() {
        assert_eq!(count_words("你好，世界！"), 4);
        assert_eq!(count_words("hello, world!"), 2);
    }

    /// Empty string returns 0.
    #[test]
    fn test_count_words_empty() {
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("   "), 0);
        assert_eq!(count_words("\n\n"), 0);
    }

    /// Only whitespace/punctuation returns 0.
    #[test]
    fn test_count_words_punctuation_only() {
        assert_eq!(count_words("。，！？——"), 0);
    }

    /// ATX headings at all levels (`#` through `######`) must lose their prefix.
    #[test]
    fn test_plain_text_atx_headings_all_levels() {
        for level in 1..=6usize {
            let hashes = "#".repeat(level);
            let md = format!("{hashes} 标题文字");
            let plain = markdown_to_plain_text(&md);
            assert!(plain.contains("标题文字"), "level {level} heading text missing");
            assert!(!plain.contains('#'), "level {level} heading still has #");
        }
    }

    /// Bold `**text**` and italic `*text*` markers must be removed.
    #[test]
    fn test_plain_text_bold_italic_and_code() {
        let plain = markdown_to_plain_text("**粗体** 和 *斜体* 以及 `代码`");
        assert!(plain.contains("粗体"));
        assert!(plain.contains("斜体"));
        assert!(plain.contains("代码"));
        assert!(!plain.contains('*'));
        assert!(!plain.contains('`'));
    }

    /// Blockquote `>` prefixes must be stripped.
    #[test]
    fn test_plain_text_blockquote() {
        let plain = markdown_to_plain_text("> 引用内容\n>> 嵌套引用");
        assert!(plain.contains("引用内容"));
        assert!(!plain.contains('>'));
    }

    /// Setext-style heading underlines (`===` / `---`) must be skipped entirely.
    #[test]
    fn test_plain_text_setext_underlines_skipped() {
        let md = "章节标题\n========\n小节标题\n--------";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("章节标题"));
        assert!(plain.contains("小节标题"));
        assert!(!plain.contains('='), "=== line should be removed");
        // The --- setext underline should be gone, but a legitimate `---` HR
        // in the same test is not tested here to keep the assertion simple.
    }

    /// Unordered list prefixes (`- `, `* `, `+ `) must be stripped.
    #[test]
    fn test_plain_text_unordered_list_prefixes() {
        let md = "- 第一项\n* 第二项\n+ 第三项";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("第一项"));
        assert!(plain.contains("第二项"));
        assert!(plain.contains("第三项"));
        // No list bullet characters should remain at the start of lines
        for line in plain.lines() {
            let l = line.trim();
            assert!(
                !l.starts_with("- ") && !l.starts_with("+ "),
                "list prefix remained: {l:?}"
            );
        }
    }

    /// Ordered list prefixes (`1. `, `10. `) must be stripped.
    #[test]
    fn test_plain_text_ordered_list_prefixes() {
        let md = "1. 有序一\n2. 有序二\n10. 有序十";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("有序一"));
        assert!(plain.contains("有序二"));
        assert!(plain.contains("有序十"));
        for line in plain.lines() {
            let l = line.trim();
            // A digit followed by ". " at the start of a non-empty line means the prefix wasn't stripped
            let starts_with_num_dot = l.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
                && l.contains(". ");
            assert!(!starts_with_num_dot, "ordered list prefix remained: {l:?}");
        }
    }

    /// Inline links `[text](url)` must become just `text`.
    #[test]
    fn test_plain_text_inline_links() {
        let md = "参见 [官方文档](https://example.com) 获取帮助。";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("官方文档"), "link text should be preserved");
        assert!(plain.contains("参见"), "surrounding text should be preserved");
        assert!(!plain.contains("https://"), "URL should be removed");
        assert!(!plain.contains("]("), "link syntax should be removed");
    }

    /// Multiple links on one line should all be stripped.
    #[test]
    fn test_plain_text_multiple_links_on_line() {
        let md = "[链接A](http://a.com) 和 [链接B](http://b.com)";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("链接A"));
        assert!(plain.contains("链接B"));
        assert!(!plain.contains("http://"));
    }

    /// Empty input produces only a newline (or empty) – no panic.
    #[test]
    fn test_plain_text_empty_input() {
        let plain = markdown_to_plain_text("");
        assert!(plain.is_empty() || plain == "\n" || plain.trim().is_empty());
    }

    // ── search_dir ────────────────────────────────────────────────────────────

    /// `search_dir` must find matches in `.md` and `.json` but not `.txt`.
    #[test]
    fn test_search_dir_finds_matches_in_md_and_json_only() {
        let dir = tmp_dir("search1");
        std::fs::write(dir.join("chapter1.md"), "# 第一章\n\n主角走进了森林。").unwrap();
        std::fs::write(dir.join("notes.json"), r#"{"title":"主角笔记"}"#).unwrap();
        std::fs::write(dir.join("ignore.txt"), "主角 should not be found").unwrap();

        let mut results = Vec::new();
        search_dir(&dir, "主角", &mut results);

        let exts: Vec<_> = results.iter()
            .map(|r| r.file_path.extension().unwrap_or_default().to_string_lossy().into_owned())
            .collect();
        assert!(exts.iter().any(|e| e == "md"),   "should match inside .md");
        assert!(exts.iter().any(|e| e == "json"), "should match inside .json");
        assert!(!exts.iter().any(|e| e == "txt"), "should NOT match inside .txt");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `search_dir` must report correct 1-based line numbers.
    #[test]
    fn test_search_dir_line_numbers() {
        let dir = tmp_dir("search2");
        std::fs::write(dir.join("ch.md"), "line one\nline two\nfind me\nline four").unwrap();

        let mut results = Vec::new();
        search_dir(&dir, "find me", &mut results);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_no, 3);
        assert!(results[0].line.contains("find me"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `search_dir` recurses into subdirectories.
    #[test]
    fn test_search_dir_recursive() {
        let dir = tmp_dir("search3");
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("deep.md"), "深层匹配内容").unwrap();

        let mut results = Vec::new();
        search_dir(&dir, "深层匹配", &mut results);

        assert_eq!(results.len(), 1);
        assert!(results[0].file_path.ends_with("deep.md"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `search_dir` on a non-existent path returns zero results without panic.
    #[test]
    fn test_search_dir_missing_path_is_silent() {
        let mut results = Vec::new();
        search_dir(std::path::Path::new("/nonexistent/qingmo_test"), "query", &mut results);
        assert!(results.is_empty());
    }

    // ── copy_dir_all ──────────────────────────────────────────────────────────

    /// Files and nested subdirectories must be copied faithfully.
    #[test]
    fn test_copy_dir_all_preserves_structure_and_content() {
        let src = tmp_dir("copy_src");
        let dst = env::temp_dir().join("qingmo_search_test_copy_dst");
        let _ = std::fs::remove_dir_all(&dst);

        std::fs::write(src.join("readme.md"), "hello world").unwrap();
        let sub = src.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("data.json"), "{}").unwrap();

        copy_dir_all(&src, &dst).expect("copy_dir_all should succeed");

        assert!(dst.join("readme.md").exists(), "top-level file missing");
        assert!(dst.join("sub").join("data.json").exists(), "nested file missing");
        let content = std::fs::read_to_string(dst.join("readme.md")).unwrap();
        assert_eq!(content, "hello world");

        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);
    }

    /// Copying to an already-existing destination should still succeed.
    #[test]
    fn test_copy_dir_all_into_existing_dst() {
        let src = tmp_dir("copy_src2");
        let dst = tmp_dir("copy_dst2"); // already exists
        std::fs::write(src.join("file.md"), "content").unwrap();

        copy_dir_all(&src, &dst).expect("copy into existing dir should succeed");
        assert!(dst.join("file.md").exists());

        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);
    }
}

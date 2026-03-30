use std::path::Path;

use super::{TextToolApp, WorldObject, StructNode, Foreshadow, Milestone, StructKind};

// ── Atomic file write ─────────────────────────────────────────────────────────

/// Write `content` to `path` atomically: first write to a `.swp` sibling file,
/// then rename it over `path`.  Within a single filesystem this rename is
/// atomic on most POSIX systems, preventing a half-written file on crash.
///
/// The `.swp` file is cleaned up on success; on failure the original `path` is
/// left untouched.
pub(super) fn write_atomically(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("swp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Recursively scan `project_dir` for leftover `.swp` files (crash drafts).
/// Returns a sorted list of paths so the UI can present them to the user.
pub(super) fn scan_swp_files(project_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    scan_swp_recursive(project_dir, &mut found);
    found.sort();
    found
}

fn scan_swp_recursive(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            scan_swp_recursive(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("swp") {
            out.push(p);
        }
    }
}

// ── Data persistence helpers ──────────────────────────────────────────────────

impl TextToolApp {
    /// Write `content` to `<project_root>/<subdir>/<filename>`.
    /// Sets `self.status` on error or when no project is open.
    /// Returns `true` on success.
    pub(super) fn write_project_file(&mut self, subdir: &str, filename: &str, content: &str) -> bool {
        if let Some(root) = self.project_root.as_ref() {
            let path = root.join(subdir).join(filename);
            if let Err(e) = write_atomically(&path, content) {
                self.status = format!("写入 {} 失败: {e}", path.display());
                return false;
            }
            true
        } else {
            self.status = "请先打开一个项目".to_owned();
            false
        }
    }

    /// Read `<project_root>/<subdir>/<filename>` as a UTF-8 string.
    /// Returns `Err` with a Chinese-language message if no project is open or
    /// the file cannot be read.
    pub(super) fn read_project_file(&self, subdir: &str, filename: &str) -> Result<(String, String), String> {
        let root = self.project_root.as_ref()
            .ok_or_else(|| "请先打开一个项目".to_owned())?;
        let path = root.join(subdir).join(filename);
        let display = path.display().to_string();
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取失败: {e}"))?;
        Ok((text, display))
    }

    // ── Save (app state → file) ───────────────────────────────────────────────

    /// Save world objects to `data/world.json`.
    pub(super) fn sync_world_objects_to_json(&mut self) {
        match serde_json::to_string_pretty(&self.world_objects) {
            Ok(json) => {
                if self.write_project_file("data", "world.json", &json) {
                    self.status = "世界对象已同步到 data/world.json".to_owned();
                }
            }
            Err(e) => self.status = format!("序列化失败: {e}"),
        }
    }

    /// Save chapter structure to `data/structure.json`.
    pub(super) fn sync_struct_to_json(&mut self) {
        match serde_json::to_string_pretty(&self.struct_roots) {
            Ok(json) => {
                if self.write_project_file("data", "structure.json", &json) {
                    self.status = "章节结构已同步到 data/structure.json".to_owned();
                }
            }
            Err(e) => self.status = format!("序列化失败: {e}"),
        }
    }

    /// Save milestones to `data/milestones.json`.
    pub(super) fn sync_milestones_to_json(&mut self) {
        match serde_json::to_string_pretty(&self.milestones) {
            Ok(json) => {
                if self.write_project_file("data", "milestones.json", &json) {
                    self.status = "里程碑已同步到 data/milestones.json".to_owned();
                }
            }
            Err(e) => self.status = format!("序列化失败: {e}"),
        }
    }

    /// Save foreshadows to `data/foreshadows.md`.
    pub(super) fn sync_foreshadows_to_md(&mut self) {
        let mut md = String::from("# 伏笔列表\n\n");
        for fs in &self.foreshadows {
            let status = if fs.resolved { "✅ 已解决" } else { "⏳ 未解决" };
            md.push_str(&format!("## {} {}\n\n", fs.name, status));
            if !fs.description.is_empty() {
                md.push_str(&format!("{}\n\n", fs.description));
            }
            if !fs.related_chapters.is_empty() {
                md.push_str(&format!("**关联章节**: {}\n\n", fs.related_chapters.join("、")));
            }
        }
        if self.write_project_file("data", "foreshadows.md", &md) {
            self.status = "伏笔已同步到 data/foreshadows.md".to_owned();
        }
    }

    // ── Load (file → app state) ───────────────────────────────────────────────

    /// Load world objects from `data/world.json` into `self.world_objects`.
    pub(super) fn load_world_objects_from_json(&mut self) {
        match self.read_project_file("data", "world.json") {
            Ok((text, display)) => match serde_json::from_str::<Vec<WorldObject>>(&text) {
                Ok(objs) => {
                    self.world_objects = objs;
                    self.selected_obj_idx = None;
                    self.status = format!("已从 {display} 加载世界对象");
                }
                Err(e) => self.status = format!("解析失败: {e}"),
            },
            Err(msg) => self.status = msg,
        }
    }

    /// Load chapter structure from `data/structure.json` into `self.struct_roots`.
    pub(super) fn load_struct_from_json(&mut self) {
        match self.read_project_file("data", "structure.json") {
            Ok((text, display)) => match serde_json::from_str::<Vec<StructNode>>(&text) {
                Ok(nodes) => {
                    self.struct_roots = nodes;
                    self.selected_node_path.clear();
                    self.status = format!("已从 {display} 加载章节结构");
                }
                Err(e) => self.status = format!("解析失败: {e}"),
            },
            Err(msg) => self.status = msg,
        }
    }

    /// Load milestones from `data/milestones.json` into `self.milestones`.
    pub(super) fn load_milestones_from_json(&mut self) {
        match self.read_project_file("data", "milestones.json") {
            Ok((text, display)) => match serde_json::from_str::<Vec<Milestone>>(&text) {
                Ok(ms) => {
                    self.milestones = ms;
                    self.selected_ms_idx = None;
                    self.status = format!("已从 {display} 加载里程碑");
                }
                Err(e) => self.status = format!("解析失败: {e}"),
            },
            Err(msg) => self.status = msg,
        }
    }

    /// Parse `data/foreshadows.md` → `self.foreshadows`.
    ///
    /// `## name` headings become foreshadow entries; `✅` in the heading marks
    /// them as resolved.
    pub(super) fn load_foreshadows_from_md(&mut self) {
        match self.read_project_file("data", "foreshadows.md") {
            Ok((text, display)) => {
                let mut foreshadows = Vec::new();
                for line in text.lines() {
                    if let Some(rest) = line.strip_prefix("## ") {
                        let resolved = rest.contains('✅');
                        let name = rest.replace("✅", "").replace("已解决", "")
                            .replace("⏳", "").replace("未解决", "").trim().to_owned();
                        if !name.is_empty() {
                            let mut fs = Foreshadow::new(&name);
                            fs.resolved = resolved;
                            foreshadows.push(fs);
                        }
                    }
                }
                self.foreshadows = foreshadows;
                self.selected_fs_idx = None;
                self.status = format!("已从 {display} 加载伏笔");
            }
            Err(msg) => self.status = msg,
        }
    }

    /// Run all four reverse-sync loads in sequence.
    pub(super) fn load_all_from_files(&mut self) {
        self.load_world_objects_from_json();
        self.load_struct_from_json();
        self.load_milestones_from_json();
        self.load_foreshadows_from_md();
        self.status = "已从文件加载所有数据".to_owned();
    }

    // ── Structure extraction ──────────────────────────────────────────────────

    /// Extract Markdown headings from the current left-pane file and populate
    /// `struct_roots` with a hierarchical `StructNode` tree.
    ///
    /// Level mapping:
    ///   `#`  → `StructKind::Outline`
    ///   `##` → `StructKind::Volume`
    ///   `###` → `StructKind::Chapter`
    ///   `####`+ → `StructKind::Section`
    pub(super) fn extract_structure_from_left(&mut self) {
        let content = if let Some(lf) = &self.left_file {
            if lf.is_markdown() { Some(lf.content.clone()) } else { None }
        } else {
            None
        };
        let Some(content) = content else {
            self.status = "请先在左侧打开一个 Markdown 文件".to_owned();
            return;
        };

        let nodes = extract_struct_nodes_from_markdown(&content);
        let count = count_nodes(&nodes);
        self.struct_roots = nodes;
        self.selected_node_path.clear();
        self.status = format!("已从 Markdown 提取 {count} 个结构节点");
    }

    /// Build a chapter structure from the project's `chapters/` folder hierarchy.
    ///
    /// Convention (Req 2):
    ///   • Each `.md` file = one chapter
    ///   • Subdirectories = higher-level structural nodes (Volume, Outline…)
    ///   • Sub-sections within a `.md` file are below the chapter level and are
    ///     represented by headings inside the file, not by the tree here.
    pub(super) fn sync_struct_from_folders(&mut self) {
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content_dir = root.join("chapters");
        let nodes = build_struct_from_dir(&content_dir);
        let count = count_nodes(&nodes);
        self.struct_roots = nodes;
        self.selected_node_path.clear();
        self.status = format!("已从文件夹结构同步 {count} 个章节节点");
    }

    /// Create a short-novel project template under `self.project_root`:
    /// flat chapters/ structure (single layer — only `.md` chapters, no subdirs).
    pub(super) fn apply_template_short(&mut self) {
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content = root.join("chapters");
        if let Err(e) = std::fs::create_dir_all(&content) {
            self.status = format!("创建 chapters 目录失败: {e}");
            return;
        }
        let chapters = ["序章.md", "第一章.md", "第二章.md", "第三章.md", "尾声.md"];
        let mut errors = Vec::new();
        for name in &chapters {
            let path = content.join(name);
            if !path.exists() {
                let stem = Path::new(name).file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if let Err(e) = std::fs::write(&path, format!("# {}\n\n", stem)) {
                    errors.push(format!("{name}: {e}"));
                }
            }
        }
        self.sync_struct_from_folders();
        self.refresh_tree();
        if !errors.is_empty() {
            self.status = format!("模板创建部分失败（已成功创建的文件保留在磁盘）: {}", errors.join("; "));
        } else {
            self.status = "已创建短篇模板（单层章节结构）".to_owned();
        }
    }

    /// Create a long-novel project template under `self.project_root`:
    /// two-layer chapters/ structure (Volume subdirs → Chapter `.md` files).
    pub(super) fn apply_template_long(&mut self) {
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content = root.join("chapters");
        if let Err(e) = std::fs::create_dir_all(&content) {
            self.status = format!("创建 chapters 目录失败: {e}");
            return;
        }
        let volumes: &[(&str, &[&str])] = &[
            ("第一卷", &["序章.md", "第一章.md", "第二章.md"]),
            ("第二卷", &["第一章.md", "第二章.md", "第三章.md"]),
        ];
        let mut errors = Vec::new();
        for (vol, chapters) in volumes {
            let vol_dir = content.join(vol);
            if let Err(e) = std::fs::create_dir_all(&vol_dir) {
                errors.push(format!("{vol}: {e}"));
                continue;
            }
            for name in *chapters {
                let path = vol_dir.join(name);
                if !path.exists() {
                    let stem = Path::new(name).file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    if let Err(e) = std::fs::write(&path, format!("# {}\n\n", stem)) {
                        errors.push(format!("{vol}/{name}: {e}"));
                    }
                }
            }
        }
        self.sync_struct_from_folders();
        self.refresh_tree();
        if !errors.is_empty() {
            self.status = format!("模板创建部分失败（已成功创建的文件保留在磁盘）: {}", errors.join("; "));
        } else {
            self.status = "已创建长篇模板（卷→章二层结构）".to_owned();
        }
    }
    pub(super) fn migrate_legacy_layout(&mut self) {
        let Some(root) = self.project_root.clone() else { return; };
        if migrate_project_dir(&root) {
            // Reload llm_history_path to point at new location
            let new_history = root.join("data").join("llm_history.json");
            self.llm_history = super::LlmHistory::load(&new_history);
            self.llm_history_path = Some(new_history);
            self.status = "已自动迁移项目目录结构（Content→chapters, Design→data）".to_owned();
        }
    }
}

// ── Free functions: Markdown → StructNode extraction ─────────────────────────

/// Parse ATX headings from Markdown text into a `StructNode` tree.
///
/// Level mapping:
///   `#` → Outline,  `##` → Volume,  `###` → Chapter,  `####`+ → Section
pub(super) fn extract_struct_nodes_from_markdown(content: &str) -> Vec<StructNode> {
    let mut flat: Vec<(usize, String)> = Vec::new();
    for line in content.lines() {
        // Count leading '#' chars using bytes — '#' is ASCII so this is both
        // correct and faster than iterating over Unicode code points.
        let level = line.bytes().take_while(|&b| b == b'#').count();
        if level == 0 || level > 6 {
            continue;
        }
        let rest = &line[level..]; // safe: '#' is ASCII (1 byte each)
        // Standard ATX heading: at least one space (or empty body) after '#' run.
        if !rest.starts_with(' ') && !rest.is_empty() {
            continue;
        }
        let title = rest.trim().to_owned();
        if !title.is_empty() {
            flat.push((level, title));
        }
    }
    if flat.is_empty() {
        return vec![];
    }
    nest_struct_nodes(&flat, 0, flat[0].0)
}

/// Recursively nest the flat (level, title) list into `StructNode`s.
fn nest_struct_nodes(flat: &[(usize, String)], start: usize, min_level: usize) -> Vec<StructNode> {
    use StructKind::{Outline, Volume, Chapter, Section};
    let mut result = Vec::new();
    let mut i = start;
    while i < flat.len() {
        let (lvl, title) = &flat[i];
        if *lvl < min_level {
            break;
        }
        if *lvl == min_level {
            let kind = match lvl {
                1 => Outline,
                2 => Volume,
                3 => Chapter,
                _ => Section,
            };
            let mut node = StructNode::new(title, kind);
            let mut j = i + 1;
            while j < flat.len() && flat[j].0 > *lvl {
                j += 1;
            }
            node.children = nest_struct_nodes(flat, i + 1, *lvl + 1);
            result.push(node);
            i = j;
        } else {
            i += 1;
        }
    }
    result
}

/// Build a `StructNode` tree from a directory:
/// subdirectories → `Volume`, `.md` files → `Chapter`.
pub(super) fn build_struct_from_dir(dir: &Path) -> Vec<StructNode> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut nodes = Vec::new();
    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());
    for entry in sorted {
        let path = entry.path();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            let mut vol = StructNode::new(&name, StructKind::Volume);
            vol.children = build_struct_from_dir(&path);
            nodes.push(vol);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let stem = path.file_stem()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or(name);
            nodes.push(StructNode::new(&stem, StructKind::Chapter));
        }
    }
    nodes
}

/// Count the total number of nodes in a tree (depth-first).
pub(super) fn count_nodes(roots: &[StructNode]) -> usize {
    roots.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}

/// Migrate a legacy project layout (Content/ + Design/) to the new layout
/// (chapters/ + data/).  Returns `true` if migration was attempted (i.e.
/// either `Content/` or `Design/` was found), `false` otherwise.
pub(super) fn migrate_project_dir(root: &std::path::Path) -> bool {
    let old_content = root.join("Content");
    let old_design  = root.join("Design");

    if !old_content.exists() && !old_design.exists() {
        return false;
    }

    let new_chapters = root.join("chapters");
    let new_data     = root.join("data");
    let _ = std::fs::create_dir_all(&new_chapters);
    let _ = std::fs::create_dir_all(&new_data);

    // Move chapter .md files (all except 伏笔.md)
    if old_content.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&old_content) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.extension().and_then(|e| e.to_str()) == Some("md") {
                    let name = src.file_name().unwrap_or_default();
                    if name == "伏笔.md" {
                        continue;
                    }
                    let dst = new_chapters.join(name);
                    if src.exists() && !dst.exists() {
                        let _ = std::fs::rename(&src, &dst);
                    }
                }
            }
        }
    }

    // Move foreshadows
    let src = old_content.join("伏笔.md");
    let dst = new_data.join("foreshadows.md");
    if src.exists() && !dst.exists() { let _ = std::fs::rename(&src, &dst); }

    // Move design files
    for (name, new_name) in &[
        ("世界对象.json",  "world.json"),
        ("章节结构.json",  "structure.json"),
        ("里程碑.json",    "milestones.json"),
        ("llm_history.json", "llm_history.json"),
    ] {
        let src = old_design.join(name);
        let dst = new_data.join(new_name);
        if src.exists() && !dst.exists() { let _ = std::fs::rename(&src, &dst); }
    }

    // Try to remove now-empty legacy directories
    let _ = std::fs::remove_dir(&old_content);
    let _ = std::fs::remove_dir(&old_design);

    true
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── write_atomically tests ────────────────────────────────────────────────

    #[test]
    fn test_write_atomically_creates_file() {
        let dir = std::env::temp_dir().join("qingmo_test_wa_create");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("output.md");
        write_atomically(&path, "hello world").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");
        // No leftover .swp file
        assert!(!path.with_extension("swp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_write_atomically_overwrites_existing() {
        let dir = std::env::temp_dir().join("qingmo_test_wa_overwrite");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("file.txt");
        std::fs::write(&path, "old content").unwrap();
        write_atomically(&path, "new content").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new content");
        assert!(!path.with_extension("swp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_write_atomically_swp_cleaned_on_success() {
        let dir = std::env::temp_dir().join("qingmo_test_wa_swp");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc.md");
        write_atomically(&path, "content").unwrap();
        // Verify .swp does NOT linger after successful write
        assert!(!dir.join("doc.swp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_struct_nodes_h1_h2_h3() {
        let md = "# 总纲\n## 第一卷\n### 第一章\n### 第二章\n## 第二卷\n";
        let nodes = extract_struct_nodes_from_markdown(md);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].title, "总纲");
        assert_eq!(nodes[0].kind, StructKind::Outline);
        assert_eq!(nodes[0].children.len(), 2);
        assert_eq!(nodes[0].children[0].title, "第一卷");
        assert_eq!(nodes[0].children[0].children.len(), 2);
        assert_eq!(nodes[0].children[0].children[0].title, "第一章");
    }

    #[test]
    fn test_extract_struct_nodes_empty() {
        let nodes = extract_struct_nodes_from_markdown("no headings here");
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_extract_struct_nodes_flat_chapters() {
        let md = "### 序章\n### 第一章\n### 第二章\n";
        let nodes = extract_struct_nodes_from_markdown(md);
        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().all(|n| n.kind == StructKind::Chapter));
    }

    #[test]
    fn test_count_nodes_empty() {
        assert_eq!(count_nodes(&[]), 0);
    }

    #[test]
    fn test_count_nodes_nested() {
        let md = "# 卷一\n## 第一章\n### 第一节\n## 第二章\n";
        let nodes = extract_struct_nodes_from_markdown(md);
        // 1 (卷一) + 2 (两章) + 1 (一节) = 4
        assert_eq!(count_nodes(&nodes), 4);
    }

    #[test]
    fn test_build_struct_from_dir() {
        let dir = std::env::temp_dir().join("qingmo_test_struct_dir");
        let sub = dir.join("第一卷");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("第一章.md"), "").unwrap();
        std::fs::write(dir.join("序章.md"), "").unwrap();

        let nodes = build_struct_from_dir(&dir);
        // Dir 第一卷 comes after file 序章 (dirs sort first in the tree)
        assert!(nodes.iter().any(|n| n.title == "第一卷" && n.kind == StructKind::Volume));
        assert!(nodes.iter().any(|n| n.title == "序章"   && n.kind == StructKind::Chapter));

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Template helpers ──────────────────────────────────────────────────────

    #[test]
    fn test_short_template_creates_flat_structure() {
        let dir = std::env::temp_dir().join("qingmo_test_short_tpl");
        std::fs::create_dir_all(dir.join("chapters")).unwrap();
        std::fs::create_dir_all(dir.join("data")).unwrap();

        // Simulate apply_template_short logic (no TextToolApp needed)
        let content = dir.join("chapters");
        let chapters = ["序章.md", "第一章.md", "第二章.md", "第三章.md", "尾声.md"];
        for name in &chapters {
            let path = content.join(name);
            let stem = std::path::Path::new(name).file_stem()
                .map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            std::fs::write(&path, format!("# {}\n\n", stem)).unwrap();
        }

        // Verify all .md files exist and Content has no subdirs
        let nodes = build_struct_from_dir(&content);
        assert_eq!(nodes.len(), chapters.len());
        assert!(nodes.iter().all(|n| n.kind == StructKind::Chapter));
        assert!(nodes.iter().all(|n| n.children.is_empty())); // flat, no sub-volumes

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_long_template_creates_two_layer_structure() {
        let dir = std::env::temp_dir().join("qingmo_test_long_tpl");
        std::fs::create_dir_all(dir.join("chapters")).unwrap();

        let content = dir.join("chapters");
        let volumes: &[(&str, &[&str])] = &[
            ("第一卷", &["序章.md", "第一章.md", "第二章.md"]),
            ("第二卷", &["第一章.md", "第二章.md", "第三章.md"]),
        ];
        for (vol, chapters) in volumes {
            let vol_dir = content.join(vol);
            std::fs::create_dir_all(&vol_dir).unwrap();
            for name in *chapters {
                let stem = std::path::Path::new(name).file_stem()
                    .map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                std::fs::write(vol_dir.join(name), format!("# {}\n\n", stem)).unwrap();
            }
        }

        let nodes = build_struct_from_dir(&content);
        assert_eq!(nodes.len(), 2, "Should have 2 volumes");
        assert!(nodes.iter().all(|n| n.kind == StructKind::Volume));
        assert_eq!(nodes[0].children.len(), 3, "第一卷 should have 3 chapters");
        assert_eq!(nodes[1].children.len(), 3, "第二卷 should have 3 chapters");
        assert!(nodes[0].children.iter().all(|c| c.kind == StructKind::Chapter));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_migrate_legacy_layout() {
        let dir = std::env::temp_dir().join("qingmo_test_migrate");
        let _ = std::fs::remove_dir_all(&dir);
        // Create old layout
        std::fs::create_dir_all(dir.join("Content")).unwrap();
        std::fs::create_dir_all(dir.join("Design")).unwrap();
        std::fs::write(dir.join("Content").join("ch1.md"), "# 第一章\n").unwrap();
        std::fs::write(dir.join("Content").join("伏笔.md"), "# 伏笔列表\n").unwrap();
        std::fs::write(dir.join("Design").join("世界对象.json"), "[]").unwrap();
        std::fs::write(dir.join("Design").join("章节结构.json"), "[]").unwrap();

        let migrated = migrate_project_dir(&dir);
        assert!(migrated, "migration should have occurred");

        // Verify new paths exist
        assert!(dir.join("chapters").join("ch1.md").exists());
        assert!(dir.join("data").join("foreshadows.md").exists());
        assert!(dir.join("data").join("world.json").exists());
        assert!(dir.join("data").join("structure.json").exists());

        // Old files should be gone
        assert!(!dir.join("Content").join("ch1.md").exists());
        assert!(!dir.join("Content").join("伏笔.md").exists());
        assert!(!dir.join("Design").join("世界对象.json").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── scan_swp_files tests ──────────────────────────────────────────────────

    #[test]
    fn test_scan_swp_files_finds_swp_in_subdirs() {
        let dir = std::env::temp_dir().join("qingmo_test_scan_swp");
        let chapters = dir.join("chapters");
        std::fs::create_dir_all(&chapters).unwrap();

        // A leftover .swp from a crash
        std::fs::write(chapters.join("ch1.swp"), "draft content").unwrap();
        // A normal file — should not be returned
        std::fs::write(chapters.join("ch1.md"), "real content").unwrap();

        let found = scan_swp_files(&dir);
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("ch1.swp"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_swp_files_empty_when_none() {
        let dir = std::env::temp_dir().join("qingmo_test_scan_swp_empty");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ch1.md"), "content").unwrap();

        let found = scan_swp_files(&dir);
        assert!(found.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}

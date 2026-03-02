use std::path::Path;

use super::{TextToolApp, WorldObject, StructNode, Foreshadow, Milestone, StructKind};

// ── Data persistence helpers ──────────────────────────────────────────────────

impl TextToolApp {
    /// Write `content` to `<project_root>/<subdir>/<filename>`.
    /// Sets `self.status` on error or when no project is open.
    /// Returns `true` on success.
    pub(super) fn write_project_file(&mut self, subdir: &str, filename: &str, content: &str) -> bool {
        if let Some(root) = self.project_root.as_ref() {
            let path = root.join(subdir).join(filename);
            if let Err(e) = std::fs::write(&path, content) {
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

    /// Save world objects to `Design/世界对象.json`.
    pub(super) fn sync_world_objects_to_json(&mut self) {
        match serde_json::to_string_pretty(&self.world_objects) {
            Ok(json) => {
                if self.write_project_file("Design", "世界对象.json", &json) {
                    self.status = "世界对象已同步到 Design/世界对象.json".to_owned();
                }
            }
            Err(e) => self.status = format!("序列化失败: {e}"),
        }
    }

    /// Save chapter structure to `Design/章节结构.json`.
    pub(super) fn sync_struct_to_json(&mut self) {
        match serde_json::to_string_pretty(&self.struct_roots) {
            Ok(json) => {
                if self.write_project_file("Design", "章节结构.json", &json) {
                    self.status = "章节结构已同步到 Design/章节结构.json".to_owned();
                }
            }
            Err(e) => self.status = format!("序列化失败: {e}"),
        }
    }

    /// Save milestones to `Design/里程碑.json`.
    pub(super) fn sync_milestones_to_json(&mut self) {
        match serde_json::to_string_pretty(&self.milestones) {
            Ok(json) => {
                if self.write_project_file("Design", "里程碑.json", &json) {
                    self.status = "里程碑已同步到 Design/里程碑.json".to_owned();
                }
            }
            Err(e) => self.status = format!("序列化失败: {e}"),
        }
    }

    /// Save foreshadows to `Content/伏笔.md`.
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
        if self.write_project_file("Content", "伏笔.md", &md) {
            self.status = "伏笔已同步到 Content/伏笔.md".to_owned();
        }
    }

    // ── Load (file → app state) ───────────────────────────────────────────────

    /// Load world objects from `Design/世界对象.json` into `self.world_objects`.
    pub(super) fn load_world_objects_from_json(&mut self) {
        match self.read_project_file("Design", "世界对象.json") {
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

    /// Load chapter structure from `Design/章节结构.json` into `self.struct_roots`.
    pub(super) fn load_struct_from_json(&mut self) {
        match self.read_project_file("Design", "章节结构.json") {
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

    /// Load milestones from `Design/里程碑.json` into `self.milestones`.
    pub(super) fn load_milestones_from_json(&mut self) {
        match self.read_project_file("Design", "里程碑.json") {
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

    /// Parse `Content/伏笔.md` → `self.foreshadows`.
    ///
    /// `## name` headings become foreshadow entries; `✅` in the heading marks
    /// them as resolved.
    pub(super) fn load_foreshadows_from_md(&mut self) {
        match self.read_project_file("Content", "伏笔.md") {
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

    /// Build a chapter structure from the project's `Content/` folder hierarchy.
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
        let content_dir = root.join("Content");
        let nodes = build_struct_from_dir(&content_dir);
        let count = count_nodes(&nodes);
        self.struct_roots = nodes;
        self.selected_node_path.clear();
        self.status = format!("已从文件夹结构同步 {count} 个章节节点");
    }

    /// Create a short-novel project template under `self.project_root`:
    /// flat Content/ structure (single layer — only `.md` chapters, no subdirs).
    pub(super) fn apply_template_short(&mut self) {
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content = root.join("Content");
        if let Err(e) = std::fs::create_dir_all(&content) {
            self.status = format!("创建 Content 目录失败: {e}");
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
        if !errors.is_empty() {
            self.status = format!("模板创建部分失败: {}", errors.join("; "));
            return;
        }
        self.sync_struct_from_folders();
        self.refresh_tree();
        self.status = "已创建短篇模板（单层章节结构）".to_owned();
    }

    /// Create a long-novel project template under `self.project_root`:
    /// two-layer Content/ structure (Volume subdirs → Chapter `.md` files).
    pub(super) fn apply_template_long(&mut self) {
        let Some(root) = self.project_root.clone() else {
            self.status = "请先打开一个项目".to_owned();
            return;
        };
        let content = root.join("Content");
        if let Err(e) = std::fs::create_dir_all(&content) {
            self.status = format!("创建 Content 目录失败: {e}");
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
        if !errors.is_empty() {
            self.status = format!("模板创建部分失败: {}", errors.join("; "));
            return;
        }
        self.sync_struct_from_folders();
        self.refresh_tree();
        self.status = "已创建长篇模板（卷→章二层结构）".to_owned();
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
        std::fs::create_dir_all(dir.join("Content")).unwrap();
        std::fs::create_dir_all(dir.join("Design")).unwrap();

        // Simulate apply_template_short logic (no TextToolApp needed)
        let content = dir.join("Content");
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
        std::fs::create_dir_all(dir.join("Content")).unwrap();

        let content = dir.join("Content");
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
}

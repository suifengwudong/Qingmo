use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Returns the home directory, checking platform-appropriate env vars.
fn dirs_home() -> Option<PathBuf> {
    // On Windows USERPROFILE is the standard home location; on Unix $HOME.
    #[cfg(target_os = "windows")]
    { std::env::var_os("USERPROFILE").map(PathBuf::from) }
    #[cfg(not(target_os = "windows"))]
    { std::env::var_os("HOME").map(PathBuf::from) }
}

/// Return a local-time-like HH:MM:SS string for display in the status bar.
///
/// We derive hours/minutes/seconds from the local timezone offset by reading
/// the `TZ` environment variable offset (best-effort). If the offset cannot
/// be determined we fall back to showing elapsed seconds since epoch mod 86400,
/// which gives the correct value for UTC+0 and is always monotonically correct
/// within a day.  No external crate is needed.
fn chrono_label() -> String {
    // Best-effort local-time from SystemTime + timezone env var.
    let utc_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Read TZ offset hours from env (e.g. "Asia/Shanghai" won't parse but
    // "UTC+8" or "+0800" style vars might be set via TZOFFSET).
    let offset_secs: i64 = std::env::var("TZOFFSET")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .map(|h| h * 3600)
        .unwrap_or(0);

    let local = (utc_secs + offset_secs).rem_euclid(86400);
    let hh = local / 3600;
    let mm = (local % 3600) / 60;
    let ss = local % 60;
    format!("{hh:02}:{mm:02}:{ss:02}")
}

mod models;
mod file_manager;
mod llm_backend;
mod agent;
mod sync;
mod search;
mod panel;
mod ui_helpers;

pub use models::*;
pub use file_manager::*;
pub use llm_backend::{LlmBackend, LlmTask, MockBackend, ApiBackend, LocalServerBackend, PromptTemplate};
pub use agent::{Skill, SkillSet, AgentBackend};

// ── Application state ─────────────────────────────────────────────────────────

pub struct TextToolApp {
    // Panel
    pub(super) active_panel: Panel,

    // Project
    pub(super) project_root: Option<PathBuf>,
    pub(super) file_tree: Vec<FileNode>,

    // Editors
    pub(super) left_file: Option<OpenFile>,
    pub(super) right_file: Option<OpenFile>,

    // Undo stacks (simple: store last content)
    pub(super) left_undo_stack: VecDeque<String>,
    pub(super) right_undo_stack: VecDeque<String>,

    // Track which editor pane was last focused for undo
    pub(super) last_focused_left: bool,

    // Status bar message
    pub(super) status: String,

    // New file dialog
    pub(super) new_file_dialog: Option<NewFileDialog>,

    // Rename file dialog
    pub(super) rename_dialog: Option<RenameDialog>,
    /// Currently selected file path in the navigation tree (used for F2 rename).
    pub(super) selected_file_path: Option<PathBuf>,

    // ── World Objects (Panel::Objects) ────────────────────────────────────────
    pub(super) world_objects: Vec<WorldObject>,
    pub(super) selected_obj_idx: Option<usize>,
    pub(super) new_obj_name: String,
    pub(super) new_obj_kind: ObjectKind,
    /// Input fields for adding a new ObjectLink on the selected object.
    pub(super) new_link_name: String,
    pub(super) new_link_rel_kind: RelationKind,
    /// Whether the new link target is a StructNode title (true) or a WorldObject name (false).
    pub(super) new_link_is_node: bool,
    pub(super) new_link_note: String,
    /// Kind filter shown in the object list side-panel (None = show all).
    pub(super) obj_kind_filter: Option<ObjectKind>,

    // ── Structure (Panel::Structure) ──────────────────────────────────────────
    pub(super) struct_roots: Vec<StructNode>,
    /// Path of indices from struct_roots into the currently selected node.
    pub(super) selected_node_path: Vec<usize>,
    pub(super) new_node_title: String,
    pub(super) new_node_kind: StructKind,
    /// Input fields for adding a NodeLink on the selected node.
    pub(super) new_node_link_title: String,
    pub(super) new_node_link_kind: RelationKind,
    pub(super) new_node_link_note: String,
    /// Name input for linking a WorldObject to the selected StructNode.
    pub(super) new_node_obj_link: String,

    // ── Outline & Foreshadowing (Panel::Structure – foreshadow sub-section) ───
    pub(super) foreshadows: Vec<Foreshadow>,
    pub(super) selected_fs_idx: Option<usize>,
    pub(super) new_fs_name: String,

    // ── Milestones (Panel::Structure – milestone sub-section) ────────────────
    pub(super) milestones: Vec<Milestone>,
    pub(super) selected_ms_idx: Option<usize>,
    pub(super) new_ms_name: String,

    // ── View mode toggles ─────────────────────────────────────────────────────
    pub(super) obj_view_mode: ObjectViewMode,
    pub(super) struct_view_mode: StructViewMode,
    /// Toggle between filesystem and chapter-tree in the Novel panel left sidebar.
    pub(super) file_tree_mode: FileTreeMode,

    // ── LLM Assistance (Panel::Llm) ──────────────────────────────────────────
    pub(super) llm_config: LlmConfig,
    pub(super) llm_prompt: String,
    pub(super) llm_output: String,
    /// Currently selected backend index: 0 = mock, 1 = HTTP API, 2 = LocalServer, 3 = Agent.
    pub(super) llm_backend_idx: usize,
    /// Active non-blocking LLM task (Some while a request is in-flight).
    pub(super) llm_task: Option<LlmTask>,
    /// Character name selected for dialogue-style optimisation.
    pub(super) llm_dialogue_char: String,

    // ── Markdown preview ─────────────────────────────────────────────────────
    pub(super) left_preview_mode: bool,
    pub(super) md_settings: MarkdownSettings,
    pub(super) show_settings_window: bool,

    // ── Theme ─────────────────────────────────────────────────────────────────
    pub(super) theme: AppTheme,

    // ── Auto-save ─────────────────────────────────────────────────────────────
    /// When the last auto-save ran (None = not yet started this session).
    pub(super) last_auto_save: Option<Instant>,
    /// Human-readable HH:MM:SS of last auto-save shown in the status bar.
    pub(super) last_auto_save_label: String,

    // ── Delete confirmation ────────────────────────────────────────────────────
    /// File path pending deletion (move to 废稿) — shown in confirm dialog.
    pub(super) delete_confirm_path: Option<PathBuf>,

    // ── Config persistence ────────────────────────────────────────────────────
    pub(super) last_project: Option<PathBuf>,
    /// Auto-load world objects / struct / foreshadows / milestones from files when opening project.
    pub(super) auto_load_from_files: bool,

    // ── Full-text search ──────────────────────────────────────────────────────
    pub(super) show_search: bool,
    pub(super) search_query: String,
    pub(super) search_results: Vec<SearchResult>,

    // ── Structure panel auto-save ─────────────────────────────────────────────
    /// Serialised JSON snapshot of `struct_roots` as of the last save.
    /// Used to detect changes and trigger auto-save without a dirty flag.
    pub(super) struct_json_snapshot: Option<String>,

    // ── Panel-switch tracking (for Structure auto-load) ───────────────────────
    pub(super) last_active_panel: Panel,

    // ── Novel template dialog ─────────────────────────────────────────────────
    pub(super) show_template_dialog: bool,

    // ── Find & Replace bar ────────────────────────────────────────────────────
    pub(super) find_bar: Option<FindBar>,

    // ── LLM history ───────────────────────────────────────────────────────────
    /// Persisted conversation history for the current project.
    pub(super) llm_history: LlmHistory,
    /// On-disk path for `llm_history.json` (set when a project is opened).
    pub(super) llm_history_path: Option<PathBuf>,
    /// Active tab in the LLM panel: 0 = 任务, 1 = 历史.
    pub(super) llm_panel_tab: u8,
    /// Keyword filter in the LLM history tab.
    pub(super) llm_history_search: String,
    /// Index of the history entry expanded in the history list (if any).
    pub(super) llm_history_expanded: Option<usize>,
    /// Pending delete confirmation index in the history tab.
    pub(super) llm_history_delete_confirm: Option<usize>,

    // ── Full-book word count stats ────────────────────────────────────────────
    /// Word counts per file as of project open (snapshot for today's delta).
    pub(super) word_count_baseline: HashMap<PathBuf, usize>,
    /// Accumulated new words written this session (since project was opened).
    pub(super) today_added_words: usize,

    // ── Command palette ───────────────────────────────────────────────────────
    pub(super) show_command_palette: bool,
    pub(super) command_palette_query: String,
    pub(super) command_palette_selection: usize,
}

#[derive(Debug)]
pub(super) struct NewFileDialog {
    pub(super) name: String,
    pub(super) dir: PathBuf,
}

#[derive(Debug)]
pub(super) struct RenameDialog {
    pub(super) path: PathBuf,
    pub(super) new_name: String,
}

// ── Find / Replace bar ────────────────────────────────────────────────────────

/// A single match within the editor content, with both byte offsets (for
/// `replace_range`) and pre-computed char indices (for `CCursor`) cached at
/// match-find time to avoid repeated O(n) `chars().count()` traversals.
pub(super) struct MatchRange {
    pub byte_start: usize,
    pub byte_end:   usize,
    /// `content[..byte_start].chars().count()` – cached at match time.
    pub char_start: usize,
    /// `content[..byte_end].chars().count()` – cached at match time.
    pub char_end:   usize,
}

pub(super) struct FindBar {
    pub query: String,
    pub replace: String,
    pub case_sensitive: bool,
    pub replace_mode: bool,
    /// Cached match positions.  Refreshed whenever `query`, `case_sensitive`,
    /// or the editor content changes.
    pub match_ranges: Vec<MatchRange>,
    /// Index into `match_ranges` that is currently "selected".
    pub current_match: usize,
    /// True once the query box has received its initial focus on bar open.
    /// Prevents focus from being stolen back from the replace field every frame.
    pub focus_requested: bool,
    /// Cached lowercase version of the last content passed to `refresh_matches`.
    /// Avoids re-allocating a full lowercase copy on each query keystroke when
    /// the document content hasn't changed (the common case for large chapters).
    cached_lower: Option<String>,
    /// Byte length of the content when `cached_lower` was built.
    /// Used as a fast validity check: if lengths differ, rebuild the cache.
    cached_lower_len: usize,
}

impl FindBar {
    pub fn new(replace_mode: bool) -> Self {
        Self {
            query: String::new(),
            replace: String::new(),
            case_sensitive: false,
            replace_mode,
            match_ranges: Vec::new(),
            current_match: 0,
            focus_requested: false,
            cached_lower: None,
            cached_lower_len: 0,
        }
    }

    /// Rebuild `match_ranges` by scanning `text` for `self.query`.
    ///
    /// Char offsets are computed incrementally (single O(n) pass) so that
    /// `select_current_match` can look them up in O(1) instead of O(n).
    ///
    /// The lowercase version of `text` is cached inside `FindBar` and reused
    /// across consecutive keystrokes when the document content is unchanged.
    pub fn refresh_matches(&mut self, text: &str) {
        self.match_ranges.clear();
        self.current_match = 0;
        if self.query.is_empty() {
            return;
        }

        // Build or reuse the lowercase cache.
        let need_rebuild = !self.case_sensitive
            && (self.cached_lower.is_none() || self.cached_lower_len != text.len());
        if need_rebuild {
            self.cached_lower = Some(text.to_lowercase());
            self.cached_lower_len = text.len();
        }

        let haystack: &str = if self.case_sensitive {
            text
        } else {
            self.cached_lower.as_deref().unwrap_or(text)
        };
        let needle: std::borrow::Cow<str> = if self.case_sensitive {
            std::borrow::Cow::Borrowed(&self.query)
        } else {
            std::borrow::Cow::Owned(self.query.to_lowercase())
        };
        let nlen = needle.len();
        if nlen == 0 {
            return;
        }

        // Walk through all matches, accumulating char position incrementally.
        let mut byte_cursor = 0usize;
        let mut char_cursor = 0usize;
        loop {
            match haystack[byte_cursor..].find(needle.as_ref()) {
                Some(rel) => {
                    let match_byte = byte_cursor + rel;
                    // Advance char_cursor from byte_cursor → match start.
                    char_cursor += text[byte_cursor..match_byte].chars().count();
                    let char_start = char_cursor;
                    // Advance char_cursor across the match.
                    char_cursor += text[match_byte..match_byte + nlen].chars().count();
                    let char_end = char_cursor;

                    self.match_ranges.push(MatchRange {
                        byte_start: match_byte,
                        byte_end:   match_byte + nlen,
                        char_start,
                        char_end,
                    });

                    byte_cursor = match_byte + nlen;
                    if byte_cursor >= haystack.len() {
                        break;
                    }
                }
                None => break,
            }
        }
    }

    /// Invalidate the cached lowercase content.  Call this after the editor
    /// content changes (e.g., after a replace operation) so the next
    /// `refresh_matches` rebuilds the cache from the new text.
    pub fn invalidate_cache(&mut self) {
        self.cached_lower = None;
        self.cached_lower_len = 0;
    }

    pub fn go_next(&mut self) {
        if self.match_ranges.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.match_ranges.len();
    }

    pub fn go_prev(&mut self) {
        if self.match_ranges.is_empty() {
            return;
        }
        if self.current_match == 0 {
            self.current_match = self.match_ranges.len() - 1;
        } else {
            self.current_match -= 1;
        }
    }
}

impl TextToolApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load Chinese font
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "chinese".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/NotoSansCJKsc-Regular.otf")),
        );
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "chinese".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "chinese".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let mut app = TextToolApp {
            active_panel: Panel::Novel,
            project_root: None,
            file_tree: vec![],
            left_file: None,
            right_file: None,
            left_undo_stack: VecDeque::new(),
            right_undo_stack: VecDeque::new(),
            last_focused_left: true,
            status: "欢迎使用清墨".to_owned(),
            new_file_dialog: None,
            rename_dialog: None,
            selected_file_path: None,
            world_objects: vec![],
            selected_obj_idx: None,
            new_obj_name: String::new(),
            new_obj_kind: ObjectKind::Character,
            new_link_name: String::new(),
            new_link_rel_kind: RelationKind::Friend,
            new_link_is_node: false,
            new_link_note: String::new(),
            obj_kind_filter: None,
            struct_roots: vec![],
            selected_node_path: vec![],
            new_node_title: String::new(),
            new_node_kind: StructKind::Chapter,
            new_node_link_title: String::new(),
            new_node_link_kind: RelationKind::Foreshadows,
            new_node_link_note: String::new(),
            new_node_obj_link: String::new(),
            foreshadows: vec![],
            selected_fs_idx: None,
            new_fs_name: String::new(),
            milestones: vec![
                Milestone::new("完成 VS Code 风格 UI 复刻"),
                Milestone::new("实现本地 MD/JSON 文件操作"),
                Milestone::new("完成轻量化基础（体积/速度/内存）"),
                Milestone::new("完成人设图形化编辑器（卡片视图）"),
                Milestone::new("完成章节时间轴编辑器"),
                Milestone::new("完成大纲树与伏笔管理"),
                Milestone::new("接入本地 LLM 模型"),
            ],
            selected_ms_idx: None,
            new_ms_name: String::new(),
            obj_view_mode: ObjectViewMode::List,
            struct_view_mode: StructViewMode::Tree,
            file_tree_mode: FileTreeMode::Chapters,
            llm_config: LlmConfig {
                model_path: String::new(),
                api_url: "http://localhost:11434/api/generate".to_owned(),
                temperature: 0.7,
                max_tokens: 512,
                use_local: true,
                system_prompt: String::new(),
            },
            llm_prompt: String::new(),
            llm_output: String::new(),
            llm_backend_idx: 0,
            llm_task: None,
            llm_dialogue_char: String::new(),
            left_preview_mode: false,
            md_settings: MarkdownSettings::default(),
            show_settings_window: false,
            theme: AppTheme::Dark,
            last_auto_save: None,
            last_auto_save_label: String::new(),
            delete_confirm_path: None,
            last_project: None,
            auto_load_from_files: false,
            show_search: false,
            search_query: String::new(),
            search_results: vec![],
            struct_json_snapshot: None,
            last_active_panel: Panel::Novel,
            show_template_dialog: false,
            find_bar: None,
            llm_history: LlmHistory::default(),
            llm_history_path: None,
            llm_panel_tab: 0,
            llm_history_search: String::new(),
            llm_history_expanded: None,
            llm_history_delete_confirm: None,
            word_count_baseline: HashMap::new(),
            today_added_words: 0,
            show_command_palette: false,
            command_palette_query: String::new(),
            command_palette_selection: 0,
        };

        // Apply saved configuration (LLM settings, MD settings, last project).
        if let Some(cfg) = Self::load_config() {
            app.llm_config = cfg.llm_config;
            app.md_settings = cfg.md_settings;
            app.auto_load_from_files = cfg.auto_load;
            app.theme = cfg.theme;
            if let Some(p) = cfg.last_project {
                let pb = PathBuf::from(p);
                if pb.is_dir() {
                    app.last_project = Some(pb.clone());
                    app.open_project(pb);
                }
            }
        }

        app
    }

    // ── Project operations ────────────────────────────────────────────────────

    pub(super) fn open_project(&mut self, path: PathBuf) {
        // Migrate legacy layout before creating directories
        self.project_root = Some(path.clone());
        self.migrate_legacy_layout();
        // Ensure required subdirectories exist
        for sub in &["chapters", "data", "废稿"] {
            let _ = std::fs::create_dir_all(path.join(sub));
        }
        self.last_project = Some(path.clone());
        self.refresh_tree();
        self.status = format!("已打开项目: {}", path.display());
        self.save_config();
        if self.auto_load_from_files {
            self.load_all_from_files();
        }

        // Load LLM history for this project.
        let history_path = path.join("data").join("llm_history.json");
        self.llm_history = LlmHistory::load(&history_path);
        self.llm_history_path = Some(history_path);

        // Snapshot word counts for all chapters/*.md files so we can compute
        // today's writing delta during this session.
        self.word_count_baseline.clear();
        self.today_added_words = 0;
        let content_dir = path.join("chapters");
        if let Ok(entries) = std::fs::read_dir(&content_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        self.word_count_baseline.insert(
                            p, crate::app::search::count_words(&text));
                    }
                }
            }
        }
    }

    pub(super) fn refresh_tree(&mut self) {
        let hide_json = self.md_settings.hide_json;
        if let Some(root) = &self.project_root {
            self.file_tree = ["chapters", "data", "废稿"]
                .iter()
                .filter_map(|sub| {
                    let p = root.join(sub);
                    FileNode::from_path_filtered(&p, hide_json)
                })
                .collect();
        }
    }

    // ── File operations ───────────────────────────────────────────────────────

    pub(super) fn open_file_in_pane(&mut self, path: &Path, left: bool) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let f = OpenFile::new(path.to_owned(), content);
                if left {
                    // Apply the default preview setting for Markdown files
                    self.left_preview_mode = f.is_markdown() && self.md_settings.default_to_preview;
                    self.left_file = Some(f);
                    self.left_undo_stack.clear();
                } else {
                    self.right_file = Some(f);
                    self.right_undo_stack.clear();
                }
                self.status = format!("已打开: {}", path.display());
            }
            Err(e) => self.status = format!("打开失败: {e}"),
        }
    }

    pub(super) fn save_left(&mut self) {
        if let Some(f) = &mut self.left_file {
            let path = f.path.clone();
            let content = f.content.clone();
            match f.save() {
                Ok(_) => {
                    self.status = format!("已保存: {}", path.display());
                    self.update_word_count_delta(&path, &content);
                }
                Err(e) => self.status = format!("保存失败: {e}"),
            }
        }
    }

    pub(super) fn save_right(&mut self) {
        if let Some(f) = &mut self.right_file {
            let path = f.path.clone();
            let content = f.content.clone();
            match f.save() {
                Ok(_) => {
                    self.status = format!("已保存: {}", path.display());
                    self.update_word_count_delta(&path, &content);
                }
                Err(e) => self.status = format!("保存失败: {e}"),
            }
        }
    }

    /// Recompute today's writing delta after saving `path` with `content`.
    fn update_word_count_delta(&mut self, path: &Path, content: &str) {
        if path.extension().and_then(|e| e.to_str()) != Some("md") { return; }
        let current = crate::app::search::count_words(content);
        let baseline = self.word_count_baseline.entry(path.to_owned()).or_insert(0);
        if current > *baseline {
            self.today_added_words += current - *baseline;
        }
        *baseline = current;
    }

    pub(super) fn new_file(&mut self, dir: PathBuf) {
        self.new_file_dialog = Some(NewFileDialog {
            name: String::new(),
            dir,
        });
    }

    pub(super) fn create_file(&mut self, path: PathBuf) {
        if let Err(e) = std::fs::write(&path, "") {
            self.status = format!("创建失败: {e}");
        } else {
            self.refresh_tree();
            let open_in_left = !path.extension().and_then(|e| e.to_str()).eq(&Some("json"));
            self.open_file_in_pane(&path, open_in_left);
            self.status = format!("已创建: {}", path.display());
        }
    }

    // ── Structured context builders (used by LLM panel) ──────────────────────

    /// Build a dialogue-optimization prompt for a specific character.
    ///
    /// Looks up the named character in `world_objects`, injects their description
    /// and background, then wraps `dialogue_text` in an optimization request.
    /// Returns `None` if no matching character is found.
    pub(super) fn build_dialogue_optimization_prompt(
        &self,
        char_name: &str,
        dialogue_text: &str,
    ) -> Option<String> {
        let obj = self.world_objects.iter().find(|o| o.name == char_name)?;

        let mut ctx = format!("## 人物：{} ({})\n", obj.name, obj.kind.label());
        if !obj.description.is_empty() {
            ctx.push_str(&format!("- 特质：{}\n", obj.description));
        }
        if !obj.background.is_empty() {
            ctx.push_str(&format!("- 背景：{}\n", obj.background));
        }
        if !obj.links.is_empty() {
            let rels: Vec<String> = obj.links.iter()
                .map(|l| format!("{} → {}", l.kind.label(), l.target.display_name()))
                .collect();
            ctx.push_str(&format!("- 关系：{}\n", rels.join("、")));
        }

        Some(PromptTemplate::DialogueOptimize.fill(&ctx, dialogue_text))
    }

    /// Build a prompt context block listing all world objects and their links.
    pub(super) fn build_character_context(&self) -> String {
        if self.world_objects.is_empty() {
            return String::new();
        }
        let mut out = String::from("## 世界对象\n\n");
        for obj in &self.world_objects {
            out.push_str(&format!("- **{}** ({})", obj.name, obj.kind.label()));
            if !obj.description.is_empty() {
                out.push_str(&format!(": {}", obj.description));
            }
            if !obj.links.is_empty() {
                let links: Vec<String> = obj.links.iter()
                    .map(|l| format!("{} → {}", l.kind.label(), l.target.display_name()))
                    .collect();
                out.push_str(&format!("  [关联: {}]", links.join(", ")));
            }
            out.push('\n');
        }
        out
    }

    /// Build a prompt context block listing the chapter structure.
    pub(super) fn build_structure_context(&self) -> String {
        if self.struct_roots.is_empty() {
            return String::new();
        }
        let mut out = String::from("## 章节结构\n\n");
        fn walk(nodes: &[crate::app::StructNode], depth: usize, out: &mut String) {
            for n in nodes {
                let indent = "  ".repeat(depth);
                let done = if n.done { "✅" } else { "⏳" };
                out.push_str(&format!("{indent}- {done} **{}** ({})\n", n.title, n.kind.label()));
                if !n.summary.is_empty() {
                    out.push_str(&format!("{indent}  > {}\n", n.summary));
                }
                walk(&n.children, depth + 1, out);
            }
        }
        walk(&self.struct_roots, 0, &mut out);
        out
    }

    /// Rename a file or directory on disk and update open editor paths.
    pub(super) fn rename_file(&mut self, old_path: &std::path::Path, new_name: &str) {
        let new_name = new_name.trim();
        if new_name.is_empty() { return; }
        if let Some(parent) = old_path.parent() {
            let new_path = parent.join(new_name);
            if let Err(e) = std::fs::rename(old_path, &new_path) {
                self.status = format!("重命名失败: {e}");
                return;
            }
            // Update open file references if needed
            if let Some(f) = &mut self.left_file {
                if f.path == old_path { f.path = new_path.clone(); }
            }
            if let Some(f) = &mut self.right_file {
                if f.path == old_path { f.path = new_path.clone(); }
            }
            if self.selected_file_path.as_deref() == Some(old_path) {
                self.selected_file_path = Some(new_path);
            }
            self.refresh_tree();
            self.status = format!("已重命名: {}", new_name);
        }
    }    /// Move `path` into the project's `废稿/` folder.
    /// Creates `废稿/` if it doesn't exist. Appends a numeric suffix if a
    /// file with the same name already exists there.
    pub(super) fn move_to_trash(&mut self, path: &Path) {
        let Some(root) = self.project_root.clone() else {
            self.status = "无法删除：未打开项目".to_owned();
            return;
        };
        let trash_dir = root.join("废稿");
        if let Err(e) = std::fs::create_dir_all(&trash_dir) {
            self.status = format!("无法创建废稿文件夹: {e}");
            return;
        }
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_owned());

        // Resolve collision by appending _1, _2, … before the extension.
        let dest = {
            let stem = Path::new(&file_name).file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| file_name.clone());
            let ext  = Path::new(&file_name).extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            let mut candidate = trash_dir.join(&file_name);
            let mut idx = 1u32;
            while candidate.exists() {
                candidate = trash_dir.join(format!("{stem}_{idx}{ext}"));
                idx += 1;
            }
            candidate
        };

        // Close the file if it's currently open in an editor pane.
        if self.left_file.as_ref().map(|f| f.path.as_path()) == Some(path) {
            self.left_file = None;
        }
        if self.right_file.as_ref().map(|f| f.path.as_path()) == Some(path) {
            self.right_file = None;
        }
        if self.selected_file_path.as_deref() == Some(path) {
            self.selected_file_path = None;
        }

        if let Err(e) = std::fs::rename(path, &dest) {
            self.status = format!("移动失败: {e}");
        } else {
            let dest_name = dest.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            self.status = format!("已移入废稿: {dest_name}");
            self.refresh_tree();
        }
    }

    pub(super) fn build_skill_set(&self) -> SkillSet {
        SkillSet::new(
            self.world_objects.clone(),
            self.struct_roots.clone(),
            self.foreshadows.clone(),
            self.milestones.clone(),
            self.project_root.clone(),
        )
    }

    /// Construct the `AgentBackend` for the currently-open project.
    pub(super) fn make_agent_backend(&self) -> AgentBackend {
        AgentBackend { skills: self.build_skill_set() }
    }

    /// Return the human-readable name of the currently-selected LLM backend.
    /// Uses the `LlmBackend::name()` method on each concrete type.
    pub(super) fn current_backend_name(&self) -> &'static str {
        match self.llm_backend_idx {
            1 => ApiBackend.name(),
            2 => LocalServerBackend.name(),
            3 => AgentBackend::BACKEND_NAME,
            _ => MockBackend.name(),
        }
    }

    /// Return the LLM backend that corresponds to `self.llm_backend_idx`.
    ///
    /// | idx | Backend |
    /// |-----|---------|
    /// | 0   | `MockBackend` (default / offline) |
    /// | 1   | `ApiBackend` (Ollama or OpenAI-compat HTTP) |
    /// | 2   | `LocalServerBackend` (llama.cpp native `/completion`) |
    /// | 3   | `AgentBackend` (OpenAI tool-calling loop) |
    pub(super) fn make_llm_backend(&self) -> std::sync::Arc<dyn LlmBackend> {
        match self.llm_backend_idx {
            1 => std::sync::Arc::new(ApiBackend),
            2 => std::sync::Arc::new(LocalServerBackend),
            3 => std::sync::Arc::new(self.make_agent_backend()),
            _ => std::sync::Arc::new(MockBackend),
        }
    }

    // ── Tree helpers ──────────────────────────────────────────────────────────

    /// Collect the names of all world objects for auto-complete / validation.
    pub(super) fn all_object_names(&self) -> Vec<String> {
        self.world_objects.iter().map(|o| o.name.clone()).collect()
    }

    /// Collect all structure node titles (depth-first).
    pub(super) fn all_struct_node_titles(&self) -> Vec<String> {
        all_node_titles(&self.struct_roots)
    }

    // ── Config persistence ────────────────────────────────────────────────────

    /// Returns the path to `~/.config/qingmo/config.json`.
    fn config_path() -> Option<PathBuf> {
        dirs_home().map(|h| h.join(".config").join("qingmo").join("config.json"))
    }

    /// Save LLM config, Markdown settings, and last project to disk.
    pub(super) fn save_config(&self) {
        let Some(path) = Self::config_path() else { return };
        let cfg = AppConfig {
            llm_config: self.llm_config.clone(),
            md_settings: self.md_settings.clone(),
            last_project: self.last_project.as_ref().map(|p| p.to_string_lossy().into_owned()),
            auto_load: self.auto_load_from_files,
            theme: self.theme,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Load saved configuration from `~/.config/qingmo/config.json`.
    pub(super) fn load_config() -> Option<AppConfig> {
        let path = Self::config_path()?;
        let text = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&text).ok()
    }
}

// ── eframe::App impl ──────────────────────────────────────────────────────────

impl eframe::App for TextToolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme every frame (cheap: egui diffs visuals internally)
        ctx.set_visuals(match self.theme {
            AppTheme::Dark  => egui::Visuals::dark(),
            AppTheme::Light => egui::Visuals::light(),
        });

        // Keyboard shortcuts (checked before UI to avoid conflicts)
        self.handle_keyboard(ctx);

        // ── Auto-save tick ────────────────────────────────────────────────────
        if self.md_settings.auto_save_interval_secs > 0 {
            let interval = self.md_settings.auto_save_interval_secs as u64;
            let should_save = match self.last_auto_save {
                None => false, // don't save on the very first frame
                Some(last) => last.elapsed().as_secs() >= interval,
            };
            if should_save {
                let mut saved_any = false;
                if let Some(f) = &mut self.left_file {
                    if f.modified && f.save().is_ok() { saved_any = true; }
                }
                if let Some(f) = &mut self.right_file {
                    if f.modified && f.save().is_ok() { saved_any = true; }
                }
                self.last_auto_save = Some(Instant::now());
                if saved_any {
                    // Record elapsed time since session start as a human-readable label.
                    // (We avoid a UTC clock to sidestep timezone issues without a date library.)
                    self.last_auto_save_label = chrono_label();
                }
            }
            // Start the clock after the first frame so the user gets a full interval.
            if self.last_auto_save.is_none() {
                self.last_auto_save = Some(Instant::now());
            }
            // Request a repaint so we check again after the interval.
            ctx.request_repaint_after(std::time::Duration::from_secs(interval));
        }

        // UI layers always visible
        self.draw_menu_bar(ctx);
        self.draw_status_bar(ctx);
        self.draw_toolbar(ctx);

        // Content area switches based on active panel
        // ── Auto-load Structure panel on first switch ─────────────────────────
        if self.active_panel == Panel::Structure
            && self.last_active_panel != Panel::Structure
            && self.project_root.is_some()
        {
            // Silently try to load chapter structure; if file is missing, do nothing.
            if let Ok((text, _)) = self.read_project_file("Design", "章节结构.json") {
                if let Ok(nodes) = serde_json::from_str::<Vec<StructNode>>(&text) {
                    self.struct_roots = nodes;
                    self.selected_node_path.clear();
                }
            }
            // Reset snapshot so the freshly-loaded data is not immediately re-saved.
            self.struct_json_snapshot = serde_json::to_string(&self.struct_roots).ok();
        }
        self.last_active_panel = self.active_panel;

        match self.active_panel {
            Panel::Novel => {
                self.draw_file_tree(ctx);
                self.draw_editors(ctx);
            }
            Panel::Objects => {
                self.draw_objects_panel(ctx);
            }
            Panel::Structure => {
                self.draw_structure_panel(ctx);

                // ── Auto-save structure when changed ──────────────────────────
                if self.project_root.is_some() {
                    if let Ok(current_json) = serde_json::to_string(&self.struct_roots) {
                        let changed = self.struct_json_snapshot.as_deref() != Some(&current_json);
                        if changed {
                            self.struct_json_snapshot = Some(current_json.clone());
                            // Write silently — status bar only if there's an error.
                            if let Some(root) = &self.project_root.clone() {
                                let path = root.join("Design").join("章节结构.json");
                                if let Ok(pretty) = serde_json::to_string_pretty(&self.struct_roots) {
                                    let _ = std::fs::write(&path, pretty);
                                }
                            }
                        }
                    }
                }
            }
            Panel::Llm => {
                self.draw_llm_panel(ctx);
            }
        }

        // Dialogs
        self.draw_new_file_dialog(ctx);
        self.draw_rename_dialog(ctx);
        self.draw_delete_confirm_dialog(ctx);
        self.draw_settings_window(ctx);
        self.draw_search_window(ctx);
        self.draw_template_dialog(ctx);
        self.draw_command_palette(ctx);
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_dialogue_optimization_prompt tests ──────────────────────────────

    #[test]
    fn test_build_dialogue_optimization_prompt_found() {
        use crate::app::{ObjectLink, LinkTarget};
        let mut app_objs = vec![WorldObject::new("张三", ObjectKind::Character)];
        app_objs[0].description = "热情开朗".to_owned();
        app_objs[0].links.push(ObjectLink {
            target: LinkTarget::Object("李四".to_owned()),
            kind: RelationKind::Friend,
            note: String::new(),
        });

        let ctx = format!(
            "## 人物：{} ({})\n- 特质：{}\n- 关系：{} → {}\n",
            app_objs[0].name,
            app_objs[0].kind.label(),
            app_objs[0].description,
            app_objs[0].links[0].kind.label(),
            app_objs[0].links[0].target.display_name(),
        );
        let prompt = PromptTemplate::DialogueOptimize.fill(&ctx, "\"你好啊！\"");
        assert!(prompt.contains("张三"));
        assert!(prompt.contains("热情开朗"));
        assert!(prompt.contains("友好"));
        assert!(prompt.contains("你好啊"));
    }

    /// Tests the foreshadow-from-MD parsing logic in isolation using a temp file.
    #[test]
    fn test_load_foreshadows_from_md_via_files() {
        let dir = std::env::temp_dir().join("qingmo_test_fs");
        let content_dir = dir.join("Content");
        std::fs::create_dir_all(&content_dir).expect("test directory creation should succeed");
        let md_path = content_dir.join("伏笔.md");
        let md = "# 伏笔列表\n\n## 神秘信件 ✅ 已解决\n\n某内容\n\n## 古剑来历 ⏳ 未解决\n\n";
        std::fs::write(&md_path, md).expect("test MD file write should succeed");

        let text = std::fs::read_to_string(&md_path).expect("test MD file read should succeed");
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

        assert_eq!(foreshadows.len(), 2);
        assert_eq!(foreshadows[0].name, "神秘信件");
        assert!(foreshadows[0].resolved);
        assert_eq!(foreshadows[1].name, "古剑来历");
        assert!(!foreshadows[1].resolved);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Tests world-objects reverse sync roundtrip: serialize → write → deserialize.
    #[test]
    fn test_load_world_objects_roundtrip() {
        let dir = std::env::temp_dir().join("qingmo_test_wo");
        let design_dir = dir.join("Design");
        std::fs::create_dir_all(&design_dir).expect("test directory creation should succeed");

        let objects = vec![
            WorldObject::new("林枫", ObjectKind::Character),
            WorldObject::new("灵剑", ObjectKind::Item),
        ];
        let json = serde_json::to_string_pretty(&objects).expect("WorldObject list serialization should succeed");
        std::fs::write(design_dir.join("世界对象.json"), &json).expect("test JSON file write should succeed");

        let text = std::fs::read_to_string(design_dir.join("世界对象.json")).expect("test JSON file read should succeed");
        let loaded: Vec<WorldObject> = serde_json::from_str(&text).expect("WorldObject list deserialization should succeed");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "林枫");
        assert_eq!(loaded[1].kind, ObjectKind::Item);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

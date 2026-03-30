use egui::Color32;
use serde::{Deserialize, Serialize};

// ── ObjectKind ────────────────────────────────────────────────────────────────

/// The category of a world object (content element).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObjectKind {
    Character,  // 人物
    Scene,      // 场景
    Location,   // 地点
    Item,       // 道具
    Faction,    // 势力
    Other,      // 其他
}

impl ObjectKind {
    pub fn label(&self) -> &'static str {
        match self {
            ObjectKind::Character => "人物",
            ObjectKind::Scene     => "场景",
            ObjectKind::Location  => "地点",
            ObjectKind::Item      => "道具",
            ObjectKind::Faction   => "势力",
            ObjectKind::Other     => "其他",
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            ObjectKind::Character => "👤",
            ObjectKind::Scene     => "🎭",
            ObjectKind::Location  => "📍",
            ObjectKind::Item      => "🗡",
            ObjectKind::Faction   => "🏰",
            ObjectKind::Other     => "⬡",
        }
    }
    pub fn all() -> &'static [ObjectKind] {
        &[
            ObjectKind::Character,
            ObjectKind::Scene,
            ObjectKind::Location,
            ObjectKind::Item,
            ObjectKind::Faction,
            ObjectKind::Other,
        ]
    }
}

// ── RelationKind ──────────────────────────────────────────────────────────────

/// The semantic type of a link between two elements.
/// Works for Object↔Object, Object↔StructNode, and StructNode↔StructNode links.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationKind {
    // Object ↔ Object
    Friend,     // 友好
    Enemy,      // 敌对
    Family,     // 亲属
    Owns,       // 持有 (持有某道具)
    LocatedAt,  // 所在 (人物所在地点)
    BelongsTo,  // 所属 (人物所属势力)
    // Object ↔ StructNode
    AppearsIn,  // 出场 (对象在某章节出现)
    MentionedIn,// 提及 (对象在某章节被提及)
    // StructNode ↔ StructNode (non-parent cross links)
    Foreshadows,// 铺垫 (一节为另一节铺垫)
    Resolves,   // 回收 (一节回收另一节的伏笔)
    Parallels,  // 并行 (两节并行叙述)
    // Fallback
    Other,      // 其他
}

impl RelationKind {
    pub fn label(&self) -> &'static str {
        match self {
            RelationKind::Friend      => "友好",
            RelationKind::Enemy       => "敌对",
            RelationKind::Family      => "亲属",
            RelationKind::Owns        => "持有",
            RelationKind::LocatedAt   => "所在",
            RelationKind::BelongsTo   => "所属",
            RelationKind::AppearsIn   => "出场",
            RelationKind::MentionedIn => "提及",
            RelationKind::Foreshadows => "铺垫",
            RelationKind::Resolves    => "回收",
            RelationKind::Parallels   => "并行",
            RelationKind::Other       => "其他",
        }
    }
    pub fn all() -> &'static [RelationKind] {
        &[
            RelationKind::Friend,
            RelationKind::Enemy,
            RelationKind::Family,
            RelationKind::Owns,
            RelationKind::LocatedAt,
            RelationKind::BelongsTo,
            RelationKind::AppearsIn,
            RelationKind::MentionedIn,
            RelationKind::Foreshadows,
            RelationKind::Resolves,
            RelationKind::Parallels,
            RelationKind::Other,
        ]
    }
}

// ── LinkTarget ────────────────────────────────────────────────────────────────

/// What a link points to — another world object (by name) or a structure node
/// (by title).  Using names rather than integer IDs keeps the data human-readable
/// and consistent with the rest of the app, which uses names throughout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkTarget {
    /// Name of another `WorldObject`.
    Object(String),
    /// Title path of a `StructNode` (e.g. "第一卷/第一章").
    Node(String),
}

impl LinkTarget {
    pub fn display_name(&self) -> &str {
        match self {
            LinkTarget::Object(n) | LinkTarget::Node(n) => n,
        }
    }
    pub fn type_label(&self) -> &'static str {
        match self {
            LinkTarget::Object(_) => "对象",
            LinkTarget::Node(_)   => "章节",
        }
    }
}

// ── ObjectLink ────────────────────────────────────────────────────────────────

/// A directed association from a `WorldObject` to another element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLink {
    pub target: LinkTarget,
    pub kind: RelationKind,
    pub note: String,
}

// ── WorldObject ───────────────────────────────────────────────────────────────

/// A unified "content element": character, scene, location, item, faction, …
/// Replaces the old `Character` struct and extends it with a `kind` discriminant
/// and a generalised `links` list that can point to other objects *or* to
/// structure nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldObject {
    pub name: String,
    pub kind: ObjectKind,
    /// Core traits / description (what was `traits` in the old Character).
    pub description: String,
    pub background: String,
    pub links: Vec<ObjectLink>,
}

impl WorldObject {
    pub fn new(name: &str, kind: ObjectKind) -> Self {
        WorldObject {
            name: name.to_owned(),
            kind,
            description: String::new(),
            background: String::new(),
            links: vec![],
        }
    }
    pub fn icon(&self) -> &'static str { self.kind.icon() }
}

// ── ChapterTag ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChapterTag {
    Normal,     // 普通
    Climax,     // 高潮
    Foreshadow, // 伏笔
    Transition, // 过渡
}

impl ChapterTag {
    pub fn label(&self) -> &'static str {
        match self {
            ChapterTag::Normal     => "普通",
            ChapterTag::Climax     => "高潮",
            ChapterTag::Foreshadow => "伏笔",
            ChapterTag::Transition => "过渡",
        }
    }
    pub fn all() -> &'static [ChapterTag] {
        &[ChapterTag::Normal, ChapterTag::Climax, ChapterTag::Foreshadow, ChapterTag::Transition]
    }
    pub fn color(&self) -> Color32 {
        match self {
            ChapterTag::Normal     => Color32::from_gray(160),
            ChapterTag::Climax     => Color32::from_rgb(220, 80, 80),
            ChapterTag::Foreshadow => Color32::from_rgb(80, 160, 220),
            ChapterTag::Transition => Color32::from_rgb(120, 190, 120),
        }
    }
}

// ── StructKind ────────────────────────────────────────────────────────────────

/// The hierarchical level of a structure node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StructKind {
    Outline,  // 总纲
    Volume,   // 卷
    Chapter,  // 章
    Section,  // 节
}

impl StructKind {
    pub fn label(&self) -> &'static str {
        match self {
            StructKind::Outline => "总纲",
            StructKind::Volume  => "卷",
            StructKind::Chapter => "章",
            StructKind::Section => "节",
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            StructKind::Outline => "📋",
            StructKind::Volume  => "📚",
            StructKind::Chapter => "📖",
            StructKind::Section => "📑",
        }
    }
    pub fn all() -> &'static [StructKind] {
        &[StructKind::Outline, StructKind::Volume, StructKind::Chapter, StructKind::Section]
    }
    /// The natural child kind when adding a child to this level.
    pub fn default_child_kind(&self) -> StructKind {
        match self {
            StructKind::Outline => StructKind::Volume,
            StructKind::Volume  => StructKind::Chapter,
            StructKind::Chapter => StructKind::Section,
            StructKind::Section => StructKind::Section,
        }
    }
}

// ── NodeLink ──────────────────────────────────────────────────────────────────

/// A non-parent cross-link between two structure nodes (e.g. a chapter that
/// foreshadows another chapter many levels away).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLink {
    /// Title of the target node.
    pub target_title: String,
    pub kind: RelationKind,
    pub note: String,
}

// ── StructNode ────────────────────────────────────────────────────────────────

/// A hierarchical structure element (总纲 / 卷 / 章 / 节).
/// Replaces the old flat `Chapter` and adds nesting, a `kind` discriminant,
/// a list of linked world-objects, and cross-node links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructNode {
    pub title: String,
    pub kind: StructKind,
    pub tag: ChapterTag,
    pub summary: String,
    pub done: bool,
    /// Nested children (e.g. a Volume contains Chapters).
    pub children: Vec<StructNode>,
    /// Names of `WorldObject`s associated with this node.
    pub linked_objects: Vec<String>,
    /// Non-parent cross-links to other structure nodes.
    pub node_links: Vec<NodeLink>,
}

impl StructNode {
    pub fn new(title: &str, kind: StructKind) -> Self {
        StructNode {
            title: title.to_owned(),
            kind,
            tag: ChapterTag::Normal,
            summary: String::new(),
            done: false,
            children: vec![],
            linked_objects: vec![],
            node_links: vec![],
        }
    }

    /// Total number of leaf nodes (nodes without children).
    pub fn leaf_count(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|c| c.leaf_count()).sum()
        }
    }

    /// Number of done leaf nodes.
    pub fn done_count(&self) -> usize {
        if self.children.is_empty() {
            if self.done { 1 } else { 0 }
        } else {
            self.children.iter().map(|c| c.done_count()).sum()
        }
    }
}

// ── Tree helpers ──────────────────────────────────────────────────────────────

/// Navigate immutably into a tree of `StructNode`s by index path.
#[allow(dead_code)]
pub fn node_at<'a>(roots: &'a [StructNode], path: &[usize]) -> Option<&'a StructNode> {
    if path.is_empty() { return None; }
    let node = roots.get(path[0])?;
    if path.len() == 1 { Some(node) } else { node_at(&node.children, &path[1..]) }
}

/// Navigate mutably into a tree of `StructNode`s by index path.
pub fn node_at_mut<'a>(roots: &'a mut [StructNode], path: &[usize]) -> Option<&'a mut StructNode> {
    if path.is_empty() { return None; }
    if path.len() == 1 {
        return roots.get_mut(path[0]);
    }
    let node = roots.get_mut(path[0])?;
    node_at_mut(&mut node.children, &path[1..])
}

/// Collect the flat title of every node in the tree (depth-first).
pub fn all_node_titles(roots: &[StructNode]) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(nodes: &[StructNode], out: &mut Vec<String>) {
        for n in nodes {
            out.push(n.title.clone());
            walk(&n.children, out);
        }
    }
    walk(roots, &mut out);
    out
}

// ── Milestone ─────────────────────────────────────────────────────────────────

/// A project milestone – a named, describable, completable target for the novel.
/// Examples: "完成第一章草稿", "10万字初稿", "第一阶段验收".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub completed: bool,
}

impl Milestone {
    pub fn new(name: &str) -> Self {
        Milestone {
            name: name.to_owned(),
            description: String::new(),
            completed: false,
        }
    }
}

// ── Foreshadow ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foreshadow {
    pub name: String,
    pub description: String,
    pub related_chapters: Vec<String>,
    pub resolved: bool,
}

impl Foreshadow {
    pub fn new(name: &str) -> Self {
        Foreshadow {
            name: name.to_owned(),
            description: String::new(),
            related_chapters: vec![],
            resolved: false,
        }
    }
}

// ── LLM config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model_path: String,
    pub api_url: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub use_local: bool,
    /// Optional system prompt sent before the user message (OpenAI / llama.cpp).
    pub system_prompt: String,
}

// ── App theme ─────────────────────────────────────────────────────────────────

/// UI colour theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    /// Follow the operating-system dark/light preference (egui default).
    #[default]
    Dark,
    Light,
}

impl AppTheme {
    pub fn label(self) -> &'static str {
        match self {
            AppTheme::Dark  => "暗色",
            AppTheme::Light => "亮色",
        }
    }
    pub fn all() -> &'static [AppTheme] {
        &[AppTheme::Dark, AppTheme::Light]
    }
}

// ── Markdown rendering settings ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownSettings {
    /// Base font size used when rendering the preview.
    pub preview_font_size: f32,
    /// When a Markdown file is opened, default to preview mode.
    pub default_to_preview: bool,
    /// Hide `.json` files from the project file tree by default.
    /// JSON files are internal data files; users primarily write Markdown.
    #[serde(default = "default_true")]
    pub hide_json: bool,
    /// Number of spaces inserted when Tab is pressed in the Markdown editor.
    #[serde(default = "default_tab_size")]
    pub tab_size: u8,
    /// Automatically extract Markdown headings into the Structure panel when
    /// a file is saved (Ctrl+S).
    #[serde(default)]
    pub auto_extract_structure: bool,
    /// Font size for the plain-text Markdown editor (independent of preview).
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: f32,
    /// Auto-save interval in seconds. 0 = disabled.
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_secs: u32,
    /// Show the "Files" tab in the navigation sidebar.
    /// Off by default — users primarily navigate via the Chapter tree.
    /// Can be enabled in Settings.
    #[serde(default)]
    pub show_files_tab: bool,
    /// Daily writing goal in words (CJK characters + Latin word-tokens).
    /// 0 means no goal is set. Shown as a progress bar in the settings window
    /// and as "今日目标" progress in the editor status area.
    #[serde(default = "default_daily_word_goal")]
    pub daily_word_goal: u32,
}

fn default_true() -> bool { true }
fn default_tab_size() -> u8 { 2 }
fn default_editor_font_size() -> f32 { 13.0 }
fn default_auto_save_interval() -> u32 { 60 }
fn default_daily_word_goal() -> u32 { 1000 }

impl Default for MarkdownSettings {
    fn default() -> Self {
        MarkdownSettings {
            preview_font_size: 14.0,
            default_to_preview: false,
            hide_json: true,
            tab_size: 2,
            auto_extract_structure: false,
            editor_font_size: 13.0,
            auto_save_interval_secs: 60,
            show_files_tab: false,
            daily_word_goal: 1000,
        }
    }
}

/// Application configuration persisted to `~/.config/qingmo/config.json`.
///
/// ## Config file location
/// | Platform | Path |
/// |----------|------|
/// | Linux / macOS | `$HOME/.config/qingmo/config.json` |
/// | Windows | `%USERPROFILE%\.config\qingmo\config.json` |
///
/// ## Schema example
/// ```json
/// {
///   "llm_config": {
///     "model_path": "",
///     "api_url": "http://localhost:11434/api/generate",
///     "temperature": 0.7,
///     "max_tokens": 512,
///     "use_local": true,
///     "system_prompt": ""
///   },
///   "md_settings": {
///     "preview_font_size": 14.0,
///     "default_to_preview": false,
///     "hide_json": true,
///     "tab_size": 2,
///     "auto_extract_structure": false,
///     "editor_font_size": 13.0,
///     "auto_save_interval_secs": 60,
///     "show_files_tab": false,
///     "daily_word_goal": 1000
///   },
///   "last_project": "/home/user/my_novel",
///   "auto_load": true,
///   "theme": "Dark"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm_config: LlmConfig,
    pub md_settings: MarkdownSettings,
    pub last_project: Option<String>,
    /// Whether to automatically load JSON/MD data files when opening a project.
    pub auto_load: bool,
    /// UI colour theme.
    #[serde(default)]
    pub theme: AppTheme,
}

// ── LLM history ───────────────────────────────────────────────────────────────

/// A single persisted LLM interaction (prompt + response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmHistoryEntry {
    /// Monotonically-increasing entry ID (1-based).
    pub id: u64,
    /// Unix timestamp in seconds when the response was received.
    pub timestamp: u64,
    /// Grouping key: `"<project_path>::<YYYY-MM-DD>"`.
    pub session_key: String,
    pub prompt: String,
    pub response: String,
    /// Model name / path used for this request.
    pub model: String,
}

/// All persisted LLM history for a project, loaded from / saved to
/// `<project>/Design/llm_history.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmHistory {
    pub entries: Vec<LlmHistoryEntry>,
    /// Monotonically-increasing counter used to allocate entry IDs.
    /// Stored so that IDs stay unique even after entries are deleted.
    #[serde(default)]
    pub next_id: u64,
}

impl LlmHistory {
    /// Allocate the next unique entry ID and advance the internal counter.
    pub fn alloc_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Load from disk, returning an empty history if the file is missing or
    /// corrupted.  If the loaded file pre-dates the `next_id` field, the
    /// counter is inferred from the max existing entry id to avoid collisions.
    pub fn load(path: &std::path::Path) -> Self {
        let mut h: Self = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        // Repair next_id for files written before the counter existed.
        let max_id = h.entries.iter().map(|e| e.id).max().unwrap_or(0);
        if h.next_id < max_id {
            h.next_id = max_id;
        }
        h
    }

    /// Append `entry`, then flush the entire history to `path`.
    ///
    /// Before writing, calls [`maybe_archive`](Self::maybe_archive) to rotate
    /// the file if it exceeds 2 MB.
    pub fn append(&mut self, entry: LlmHistoryEntry, path: &std::path::Path) -> std::io::Result<()> {
        Self::maybe_archive(path);
        self.entries.push(entry);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// If `path` exceeds 2 MB, rename it to `llm_history_<timestamp>.json`
    /// so a fresh file is started.
    pub fn maybe_archive(path: &std::path::Path) {
        let Ok(meta) = std::fs::metadata(path) else { return };
        if meta.len() <= 2 * 1024 * 1024 { return; }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(parent) = path.parent() {
            let archive = parent.join(format!("llm_history_{ts}.json"));
            let _ = std::fs::rename(path, archive);
        }
    }
}

// ── Date helpers ──────────────────────────────────────────────────────────────

/// Convert a Unix timestamp (seconds since 1970-01-01 UTC) to an ISO 8601
/// date string `"YYYY-MM-DD"` using the proleptic Gregorian calendar.
///
/// This does not require any external date/time crate.
pub fn unix_secs_to_iso_date(secs: u64) -> String {
    let mut days_remaining = (secs / 86400) as u32;
    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_remaining < days_in_year { break; }
        days_remaining -= days_in_year;
        year += 1;
    }
    let leap = is_leap_year(year);
    let month_lengths = [31u32, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u32;
    for &ml in &month_lengths {
        if days_remaining < ml { break; }
        days_remaining -= ml;
        month += 1;
    }
    let day = days_remaining + 1;
    format!("{year:04}-{month:02}-{day:02}")
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ── Full-text search result ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: std::path::PathBuf,
    pub line_no: usize,
    pub line: String,
}

// ── View mode toggles ─────────────────────────────────────────────────────────

/// Toggle between list/card views in the Objects panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectViewMode {
    List,
    Card,
}

/// Toggle between tree/timeline views in the Structure panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructViewMode {
    Tree,
    Timeline,
}

/// Toggle between filesystem view and chapter-tree view in the Novel panel left sidebar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileTreeMode {
    /// Show the raw project filesystem (folders and files).
    Files,
    /// Show the chapter structure tree (from struct_roots). Each leaf chapter
    /// can be clicked to open its associated `.md` file in the editor.
    Chapters,
}

// ── Panel IDs ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Novel,
    /// 世界对象设计 (人物 / 场景 / 地点 / 道具 / 势力)
    Objects,
    /// 章节结构设计 (总纲 / 卷 / 章 / 节)
    Structure,
    Llm,
}

impl Panel {
    pub fn icon(self) -> &'static str {
        match self {
            Panel::Novel     => "📝",
            Panel::Objects   => "🌐",
            Panel::Structure => "纲",
            Panel::Llm       => "智",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Panel::Novel     => "小说编辑",
            Panel::Objects   => "世界对象",
            Panel::Structure => "章节结构",
            Panel::Llm       => "LLM辅助",
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_kind_labels() {
        assert_eq!(ObjectKind::Character.label(), "人物");
        assert_eq!(ObjectKind::Scene.label(), "场景");
        assert_eq!(ObjectKind::Location.label(), "地点");
        assert_eq!(ObjectKind::Item.label(), "道具");
        assert_eq!(ObjectKind::Faction.label(), "势力");
        assert_eq!(ObjectKind::Other.label(), "其他");
    }

    #[test]
    fn test_world_object_new() {
        let obj = WorldObject::new("张三", ObjectKind::Character);
        assert_eq!(obj.name, "张三");
        assert_eq!(obj.kind, ObjectKind::Character);
        assert!(obj.description.is_empty());
        assert!(obj.links.is_empty());
    }

    #[test]
    fn test_world_object_link() {
        let mut obj = WorldObject::new("张三", ObjectKind::Character);
        obj.links.push(ObjectLink {
            target: LinkTarget::Object("李四".to_owned()),
            kind: RelationKind::Friend,
            note: String::new(),
        });
        assert_eq!(obj.links.len(), 1);
        assert_eq!(obj.links[0].target.display_name(), "李四");
        assert_eq!(obj.links[0].target.type_label(), "对象");
    }

    #[test]
    fn test_world_object_link_to_node() {
        let mut obj = WorldObject::new("古剑", ObjectKind::Item);
        obj.links.push(ObjectLink {
            target: LinkTarget::Node("第一章".to_owned()),
            kind: RelationKind::AppearsIn,
            note: "在山洞中被发现".to_owned(),
        });
        assert_eq!(obj.links[0].target.type_label(), "章节");
        assert_eq!(obj.links[0].note, "在山洞中被发现");
    }

    #[test]
    fn test_world_object_json_serialization() {
        let mut obj = WorldObject::new("主角", ObjectKind::Character);
        obj.description = "勇敢、善良".to_owned();
        obj.links.push(ObjectLink {
            target: LinkTarget::Object("反派".to_owned()),
            kind: RelationKind::Enemy,
            note: String::new(),
        });
        let json = serde_json::to_string(&obj).expect("WorldObject serialization should succeed");
        let d: WorldObject = serde_json::from_str(&json).expect("WorldObject deserialization should succeed");
        assert_eq!(d.name, "主角");
        assert_eq!(d.kind, ObjectKind::Character);
        assert_eq!(d.links[0].kind, RelationKind::Enemy);
    }

    #[test]
    fn test_struct_kind_labels() {
        assert_eq!(StructKind::Outline.label(), "总纲");
        assert_eq!(StructKind::Volume.label(), "卷");
        assert_eq!(StructKind::Chapter.label(), "章");
        assert_eq!(StructKind::Section.label(), "节");
    }

    #[test]
    fn test_struct_kind_default_child() {
        assert_eq!(StructKind::Outline.default_child_kind(), StructKind::Volume);
        assert_eq!(StructKind::Volume.default_child_kind(), StructKind::Chapter);
        assert_eq!(StructKind::Chapter.default_child_kind(), StructKind::Section);
    }

    #[test]
    fn test_struct_node_new() {
        let n = StructNode::new("第一章", StructKind::Chapter);
        assert_eq!(n.title, "第一章");
        assert_eq!(n.kind, StructKind::Chapter);
        assert!(n.children.is_empty());
        assert!(n.linked_objects.is_empty());
        assert!(!n.done);
    }

    #[test]
    fn test_struct_node_leaf_count() {
        let mut vol = StructNode::new("第一卷", StructKind::Volume);
        vol.children.push(StructNode::new("第一章", StructKind::Chapter));
        vol.children.push(StructNode::new("第二章", StructKind::Chapter));
        assert_eq!(vol.leaf_count(), 2);
    }

    #[test]
    fn test_struct_node_done_count() {
        let mut vol = StructNode::new("第一卷", StructKind::Volume);
        let mut ch1 = StructNode::new("第一章", StructKind::Chapter);
        ch1.done = true;
        vol.children.push(ch1);
        vol.children.push(StructNode::new("第二章", StructKind::Chapter));
        assert_eq!(vol.done_count(), 1);
        assert_eq!(vol.leaf_count(), 2);
    }

    #[test]
    fn test_struct_node_json_serialization() {
        let mut node = StructNode::new("序章", StructKind::Chapter);
        node.tag = ChapterTag::Foreshadow;
        node.done = true;
        node.linked_objects.push("主角".to_owned());
        let json = serde_json::to_string(&node).expect("StructNode serialization should succeed");
        let d: StructNode = serde_json::from_str(&json).expect("StructNode deserialization should succeed");
        assert_eq!(d.title, "序章");
        assert_eq!(d.tag, ChapterTag::Foreshadow);
        assert!(d.done);
        assert_eq!(d.linked_objects[0], "主角");
    }

    #[test]
    fn test_node_at() {
        let mut roots = vec![StructNode::new("第一卷", StructKind::Volume)];
        roots[0].children.push(StructNode::new("第一章", StructKind::Chapter));
        assert_eq!(node_at(&roots, &[0]).expect("node should exist at [0]").title, "第一卷");
        assert_eq!(node_at(&roots, &[0, 0]).expect("node should exist at [0,0]").title, "第一章");
        assert!(node_at(&roots, &[1]).is_none());
    }

    #[test]
    fn test_node_at_mut() {
        let mut roots = vec![StructNode::new("第一卷", StructKind::Volume)];
        roots[0].children.push(StructNode::new("第一章", StructKind::Chapter));
        node_at_mut(&mut roots, &[0, 0]).expect("node should exist at [0,0]").done = true;
        assert!(roots[0].children[0].done);
    }

    #[test]
    fn test_all_node_titles() {
        let mut roots = vec![StructNode::new("第一卷", StructKind::Volume)];
        roots[0].children.push(StructNode::new("第一章", StructKind::Chapter));
        roots[0].children.push(StructNode::new("第二章", StructKind::Chapter));
        let titles = all_node_titles(&roots);
        assert_eq!(titles, vec!["第一卷", "第一章", "第二章"]);
    }

    #[test]
    fn test_relation_kind_labels() {
        assert_eq!(RelationKind::Friend.label(), "友好");
        assert_eq!(RelationKind::Enemy.label(), "敌对");
        assert_eq!(RelationKind::Family.label(), "亲属");
        assert_eq!(RelationKind::AppearsIn.label(), "出场");
        assert_eq!(RelationKind::Foreshadows.label(), "铺垫");
        assert_eq!(RelationKind::Resolves.label(), "回收");
    }

    #[test]
    fn test_chapter_tag_labels() {
        assert_eq!(ChapterTag::Climax.label(), "高潮");
        assert_eq!(ChapterTag::Foreshadow.label(), "伏笔");
        assert_eq!(ChapterTag::Transition.label(), "过渡");
        assert_eq!(ChapterTag::Normal.label(), "普通");
    }

    #[test]
    fn test_foreshadow_new() {
        let fs = Foreshadow::new("神秘礼物");
        assert_eq!(fs.name, "神秘礼物");
        assert!(!fs.resolved);
        assert!(fs.related_chapters.is_empty());
    }

    #[test]
    fn test_markdown_settings_default() {
        let s = MarkdownSettings::default();
        assert_eq!(s.preview_font_size, 14.0);
        assert!(!s.default_to_preview);
    }

    #[test]
    fn test_markdown_settings_custom() {
        let s = MarkdownSettings {
            preview_font_size: 18.0,
            default_to_preview: true,
            ..MarkdownSettings::default()
        };
        assert_eq!(s.preview_font_size, 18.0);
        assert!(s.default_to_preview);
    }

    #[test]
    fn test_milestone_new() {
        let m = Milestone::new("第一阶段完成");
        assert_eq!(m.name, "第一阶段完成");
        assert!(!m.completed);
        assert!(m.description.is_empty());
    }

    #[test]
    fn test_milestone_completion() {
        let mut m = Milestone::new("MVP");
        assert!(!m.completed);
        m.completed = true;
        assert!(m.completed);
    }

    #[test]
    fn test_milestone_json_serialization() {
        let mut m = Milestone::new("发布 v1.0");
        m.description = "第一个正式版本".to_owned();
        m.completed = true;
        let json = serde_json::to_string(&m).expect("Milestone serialization should succeed");
        let d: Milestone = serde_json::from_str(&json).expect("Milestone deserialization should succeed");
        assert_eq!(d.name, "发布 v1.0");
        assert_eq!(d.description, "第一个正式版本");
        assert!(d.completed);
    }

    #[test]
    fn test_llm_config_serialization() {
        let cfg = LlmConfig {
            model_path: "llama2".to_owned(),
            api_url: "http://localhost:11434/api/generate".to_owned(),
            temperature: 0.8,
            max_tokens: 256,
            use_local: false,
            system_prompt: "你是一个写作助手".to_owned(),
        };
        let json = serde_json::to_string(&cfg).expect("config serialization should succeed");
        let d: LlmConfig = serde_json::from_str(&json).expect("LlmConfig deserialization should succeed");
        assert_eq!(d.model_path, "llama2");
        assert_eq!(d.api_url, "http://localhost:11434/api/generate");
        assert!((d.temperature - 0.8).abs() < 1e-5);
        assert_eq!(d.max_tokens, 256);
        assert!(!d.use_local);
        assert_eq!(d.system_prompt, "你是一个写作助手");
    }

    #[test]
    fn test_markdown_settings_serialization() {
        let s = MarkdownSettings {
            preview_font_size: 18.0,
            default_to_preview: true,
            ..MarkdownSettings::default()
        };
        let json = serde_json::to_string(&s).expect("MarkdownSettings serialization should succeed");
        let d: MarkdownSettings = serde_json::from_str(&json).expect("MarkdownSettings deserialization should succeed");
        assert!((d.preview_font_size - 18.0).abs() < 1e-5);
        assert!(d.default_to_preview);
    }

    #[test]
    fn test_app_config_serialization_roundtrip() {
        let cfg = AppConfig {
            llm_config: LlmConfig {
                model_path: "phi2".to_owned(),
                api_url: "http://localhost:8080".to_owned(),
                temperature: 0.5,
                max_tokens: 1024,
                use_local: true,
                system_prompt: String::new(),
            },
            md_settings: MarkdownSettings {
                preview_font_size: 16.0,
                default_to_preview: true,
                ..MarkdownSettings::default()
            },
            last_project: Some("/home/user/my_novel".to_owned()),
            auto_load: true,
            theme: AppTheme::Dark,
        };
        let json = serde_json::to_string_pretty(&cfg).expect("AppConfig serialization should succeed");
        let d: AppConfig = serde_json::from_str(&json).expect("AppConfig deserialization should succeed");
        assert_eq!(d.llm_config.model_path, "phi2");
        assert_eq!(d.md_settings.preview_font_size, 16.0);
        assert_eq!(d.last_project, Some("/home/user/my_novel".to_owned()));
        assert!(d.auto_load);
    }

    #[test]
    fn test_markdown_settings_new_fields_defaults() {
        let s = MarkdownSettings::default();
        assert!(s.hide_json);
        assert_eq!(s.tab_size, 2);
        assert!(!s.auto_extract_structure);
        assert!((s.editor_font_size - 13.0).abs() < 1e-5);
        assert_eq!(s.auto_save_interval_secs, 60);
        assert!(!s.show_files_tab);
    }

    #[test]
    fn test_markdown_settings_hide_json_roundtrip() {
        let old_json = r#"{"preview_font_size":14.0,"default_to_preview":false}"#;
        let s: MarkdownSettings = serde_json::from_str(old_json).expect("old MarkdownSettings JSON should deserialize with defaults");
        assert!(s.hide_json);
        assert_eq!(s.tab_size, 2);
        assert!((s.editor_font_size - 13.0).abs() < 1e-5);
    }

    #[test]
    fn test_app_theme_default() {
        let cfg: AppConfig = serde_json::from_str(
            r#"{"llm_config":{"model_path":"","api_url":"","temperature":0.7,"max_tokens":512,"use_local":true,"system_prompt":""},
                "md_settings":{"preview_font_size":14.0,"default_to_preview":false},
                "last_project":null,"auto_load":false}"#
        ).expect("AppConfig deserialization should succeed");
        assert_eq!(cfg.theme, AppTheme::Dark);
    }

    #[test]
    fn test_build_character_context_empty() {
        let objects: Vec<WorldObject> = vec![];
        let ctx: String = if objects.is_empty() {
            String::new()
        } else {
            let mut out = String::from("## 世界对象\n\n");
            for o in &objects {
                out.push_str(&format!("- **{}** ({})\n", o.name, o.kind.label()));
            }
            out
        };
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_build_character_context_with_objects() {
        let objects = vec![
            WorldObject::new("主角", ObjectKind::Character),
            WorldObject::new("城堡", ObjectKind::Location),
        ];
        let mut out = String::from("## 世界对象\n\n");
        for o in &objects {
            out.push_str(&format!("- **{}** ({})\n", o.name, o.kind.label()));
        }
        assert!(out.contains("主角"));
        assert!(out.contains("城堡"));
        assert!(out.contains("人物"));
        assert!(out.contains("地点"));
    }

    #[test]
    fn test_unix_secs_to_iso_date_known_dates() {
        assert_eq!(unix_secs_to_iso_date(0), "1970-01-01");
        assert_eq!(unix_secs_to_iso_date(86400), "1970-01-02");
        assert_eq!(unix_secs_to_iso_date(20542 * 86400), "2026-03-30");
        assert_eq!(unix_secs_to_iso_date((10957 + 59) * 86400), "2000-02-29");
    }

    #[test]
    fn test_llm_history_alloc_id_is_monotonic() {
        let mut h = LlmHistory::default();
        assert_eq!(h.alloc_id(), 1);
        assert_eq!(h.alloc_id(), 2);
        assert_eq!(h.alloc_id(), 3);
    }

    #[test]
    fn test_llm_history_load_repairs_next_id() {
        let mut h = LlmHistory::default();
        h.entries.push(LlmHistoryEntry {
            id: 5,
            timestamp: 0,
            session_key: "2026-01-01".to_owned(),
            prompt: "test".to_owned(),
            response: "ok".to_owned(),
            model: "mock".to_owned(),
        });
        let json = serde_json::to_string(&h).unwrap();
        let old_json = json.replace(r#""next_id":5,"#, "").replace(r#","next_id":5"#, "");
        let dir = std::env::temp_dir().join("qingmo_test_llm_history_repair");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("llm_history.json");
        std::fs::write(&path, &old_json).unwrap();
        let loaded = LlmHistory::load(&path);
        assert!(loaded.next_id >= 5, "next_id should be repaired to at least 5, got {}", loaded.next_id);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

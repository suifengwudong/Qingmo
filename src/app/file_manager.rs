use std::path::{Path, PathBuf};

// ── File tree node ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    /// Build a file tree node, optionally hiding `.json` files.
    pub fn from_path_filtered(path: &Path, hide_json: bool) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        if path.is_dir() {
            let mut children: Vec<FileNode> = std::fs::read_dir(path)
                .ok()?
                .filter_map(|e| e.ok())
                .filter_map(|e| FileNode::from_path_filtered(&e.path(), hide_json))
                .collect();
            children.sort_by(|a, b| {
                b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
            });
            Some(FileNode {
                name,
                path: path.to_owned(),
                is_dir: true,
                expanded: true,
                children,
            })
        } else {
            // When hide_json is set, exclude .json files from the visible tree.
            if hide_json && path.extension().and_then(|e| e.to_str()) == Some("json") {
                return None;
            }
            Some(FileNode {
                name,
                path: path.to_owned(),
                is_dir: false,
                expanded: false,
                children: vec![],
            })
        }
    }
}

// ── Open file ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub path: PathBuf,
    pub content: String,
    pub modified: bool,
}

impl OpenFile {
    pub fn new(path: PathBuf, content: String) -> Self {
        OpenFile { path, content, modified: false }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        std::fs::write(&self.path, &self.content)?;
        self.modified = false;
        Ok(())
    }

    pub fn title(&self) -> String {
        let name = self.path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "untitled".to_owned());
        if self.modified {
            format!("● {name}")
        } else {
            name
        }
    }

    pub fn is_markdown(&self) -> bool {
        matches!(
            self.path.extension().and_then(|e| e.to_str()),
            Some("md") | Some("markdown")
        )
    }
}

// ── Thin wrappers around rfd ──────────────────────────────────────────────────

pub fn rfd_pick_folder() -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        rfd::FileDialog::new().pick_folder()
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

pub fn rfd_save_file(hint: &Path) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ext = hint.extension().and_then(|e| e.to_str()).unwrap_or("txt");
        let name = hint.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter("文件", &[ext])
            .save_file()
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_open_file_is_markdown() {
        let f = OpenFile::new(PathBuf::from("test.md"), String::new());
        assert!(f.is_markdown());
        let f2 = OpenFile::new(PathBuf::from("test.json"), String::new());
        assert!(!f2.is_markdown());
    }

    #[test]
    fn test_open_file_title_modified() {
        let mut f = OpenFile::new(PathBuf::from("test.md"), String::new());
        assert_eq!(f.title(), "test.md");
        f.modified = true;
        assert_eq!(f.title(), "● test.md");
    }

    #[test]
    fn test_file_node_from_path_filtered_hides_json() {
        let dir = std::env::temp_dir().join("qingmo_test_filetree");
        std::fs::create_dir_all(&dir).expect("test directory creation should succeed");
        std::fs::write(dir.join("chapter1.md"), "hello").expect("test file write should succeed");
        std::fs::write(dir.join("data.json"), "{}").expect("test file write should succeed");

        let node_show = FileNode::from_path_filtered(&dir, false).expect("FileNode creation should succeed");
        let node_hide = FileNode::from_path_filtered(&dir, true).expect("FileNode creation with hide_json should succeed");

        let show_names: Vec<_> = node_show.children.iter().map(|n| &n.name).collect();
        let hide_names: Vec<_> = node_hide.children.iter().map(|n| &n.name).collect();

        assert!(show_names.iter().any(|n| n.as_str() == "data.json"));
        assert!(!hide_names.iter().any(|n| n.as_str() == "data.json"));
        assert!(hide_names.iter().any(|n| n.as_str() == "chapter1.md"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}

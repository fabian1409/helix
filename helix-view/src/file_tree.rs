use std::cmp::Ordering;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub const FILE_TREE_WIDTH: u16 = 40;

#[derive(Clone)]
pub struct FileTreeItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
}

impl FileTreeItem {
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}

impl From<DirEntry> for FileTreeItem {
    fn from(value: DirEntry) -> Self {
        let meta = value.metadata().expect("can read meta");
        let name = value.file_name();
        FileTreeItem {
            name: name.into_string().expect("can conv to string"),
            path: value.path(),
            is_dir: meta.is_dir(),
            is_expanded: false,
            depth: 0,
        }
    }
}

pub struct FileTree {
    pub items: Vec<FileTreeItem>,
    pub selection: Option<usize>,
    pub focused: bool,
}

impl Default for FileTree {
    fn default() -> Self {
        FileTree::new()
    }
}

impl FileTree {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let dir = read_dir_sorted(&cwd, false);
        let items = dir.into_iter().map(FileTreeItem::from).collect::<Vec<_>>();
        Self {
            items,
            selection: Some(0),
            focused: true,
        }
    }

    pub fn move_up(&mut self) {
        self.selection = self.selection.map(|s| s.saturating_sub(1));
    }

    pub fn move_down(&mut self) {
        self.selection = self
            .selection
            .map(|s| if s < self.items.len() - 1 { s + 1 } else { s });
    }

    pub fn expand(&mut self) {
        let selection = self.selection.unwrap();
        let item = self.items.get_mut(selection).unwrap();
        item.is_expanded = true;
        let items = read_dir_sorted(&item.path, false)
            .into_iter()
            .map(|entry| FileTreeItem::from(entry).with_depth(item.depth + 1))
            .collect::<Vec<FileTreeItem>>();
        let len = items.len();
        self.items.extend(items);
        self.items[selection + 1..].rotate_right(len);
    }

    pub fn collapse(&mut self) {
        let selection = self.selection.unwrap();
        let item = self.items.get_mut(selection).unwrap();
        item.is_expanded = false;
        let depth = self.items.get(selection).unwrap().depth;
        let remove_len = self
            .items
            .iter()
            .skip(selection + 1)
            .take_while(|i| i.depth > depth)
            .count();
        for _ in 0..remove_len {
            self.items.remove(selection + 1);
        }
    }
}

fn read_dir_sorted(path: &Path, show_hidden: bool) -> Vec<DirEntry> {
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut entries = entries
                .flatten()
                .filter(|x| {
                    !x.path().symlink_metadata().unwrap().is_symlink()
                        && (show_hidden || !x.file_name().to_string_lossy().starts_with('.'))
                })
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| {
                let a = a.path();
                let b = b.path();
                let a_name = a.file_name().unwrap().to_string_lossy();
                let b_name = b.file_name().unwrap().to_string_lossy();
                if a.is_dir() && b.is_dir() {
                    a_name.cmp(&b_name)
                } else if a.is_dir() && !b.is_dir() {
                    Ordering::Less
                } else if !a.is_dir() && b.is_dir() {
                    Ordering::Greater
                } else {
                    a_name.cmp(&b_name)
                }
            });
            entries
        }
        Err(_) => vec![],
    }
}

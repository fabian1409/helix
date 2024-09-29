use std::cmp::Ordering;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use super::keyboard::KeyCode;
use helix_stdx::path::fold_home_dir;

pub const FILE_TREE_WIDTH: u16 = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
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
            depth: 1,
        }
    }
}

impl Ord for FileTreeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        let name = self.name.to_lowercase();
        let other_name = other.name.to_lowercase();
        if self.is_dir && other.is_dir {
            name.cmp(&other_name)
        } else if self.is_dir && !other.is_dir {
            Ordering::Less
        } else if !self.is_dir && other.is_dir {
            Ordering::Greater
        } else {
            name.cmp(&other_name)
        }
    }
}

impl PartialOrd for FileTreeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default)]
pub struct FileTree {
    pub items: Vec<FileTreeItem>,
    pub selection: Option<usize>,
    pub open: bool,
    pub focused: bool,
    pub copied: Option<FileTreeItem>,
    pub last_key: Option<KeyCode>,
}

impl FileTree {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let dir = read_dir_sorted(&cwd, false);
        let mut items = vec![FileTreeItem {
            name: fold_home_dir(&cwd).to_string_lossy().to_string(),
            path: cwd,
            is_dir: true,
            is_expanded: true,
            depth: 0,
        }];
        items.extend(dir.into_iter().map(FileTreeItem::from).collect::<Vec<_>>());
        Self {
            items,
            selection: Some(0),
            open: false,
            focused: true,
            copied: None,
            last_key: None,
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

    pub fn insert_and_adjust(&mut self, item: FileTreeItem) {
        let selection = self.selection.unwrap_or_default();
        let index = self
            .items
            .iter()
            .skip(selection + 1)
            .position(|e| e.depth == item.depth && e > &item)
            .unwrap()
            + selection
            + 1;
        self.items.insert(index, item);
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
                let a_name = a.file_name().unwrap().to_string_lossy().to_lowercase();
                let b_name = b.file_name().unwrap().to_string_lossy().to_lowercase();
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

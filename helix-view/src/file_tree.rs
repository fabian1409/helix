use std::fs::DirEntry;
use std::path::PathBuf;
use std::{cmp::Ordering, path::Path};

use helix_stdx::path::{fold_home_dir, read_dir_sorted};

pub const FILE_TREE_MAX_WIDTH: u16 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<FileTreeItem>,
}

impl FileTreeItem {
    pub fn new(name: String, path: PathBuf, is_dir: bool) -> Self {
        Self {
            name,
            path,
            is_dir,
            is_expanded: false,
            children: vec![],
        }
    }

    pub fn root(name: String, path: PathBuf, children: Vec<FileTreeItem>) -> Self {
        Self {
            name,
            path,
            is_dir: true,
            is_expanded: true,
            children,
        }
    }

    pub fn expand(&mut self) {
        self.is_expanded = true;
        let children = read_dir_sorted(&self.path, false)
            .into_iter()
            .map(FileTreeItem::from)
            .collect::<Vec<FileTreeItem>>();
        self.children = children
    }

    pub fn collapse(&mut self) {
        self.is_expanded = false;
        self.children.clear();
    }
}

impl From<DirEntry> for FileTreeItem {
    fn from(value: DirEntry) -> Self {
        let meta = value.metadata().expect("can read meta");
        let name = value.file_name();
        Self::new(
            name.into_string().expect("can conv to string"),
            value.path(),
            meta.is_dir(),
        )
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

#[derive(Debug)]
pub struct FileTree {
    pub root: FileTreeItem,
    pub selection: usize,
    pub open: bool,
    pub copied: Option<FileTreeItem>,
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTree {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().expect("can get cwd");
        let dir = read_dir_sorted(&cwd, false);
        let children = dir.into_iter().map(FileTreeItem::from).collect::<Vec<_>>();
        let root = FileTreeItem::root(
            fold_home_dir(&cwd).to_string_lossy().to_string(),
            cwd,
            children,
        );
        Self {
            root,
            selection: 0,
            open: false,
            copied: None,
        }
    }

    fn flatten_(root: &FileTreeItem, depth: usize) -> Vec<(&FileTreeItem, usize)> {
        let mut items = vec![];
        if root.children.is_empty() {
            items.push((root, depth));
        } else {
            items.push((root, depth));
            for child in root.children.iter() {
                items.extend(Self::flatten_(child, depth + 1))
            }
        }
        items
    }

    fn selected_mut_<'a>(
        node: &'a mut FileTreeItem,
        n: &mut usize,
    ) -> Option<&'a mut FileTreeItem> {
        if *n == 0 {
            return Some(node);
        }
        *n -= 1;
        for child in node.children.iter_mut() {
            if let Some(result) = Self::selected_mut_(child, n) {
                return Some(result);
            }
        }
        None
    }

    fn selected_<'a>(node: &'a FileTreeItem, n: &mut usize) -> Option<&'a FileTreeItem> {
        if *n == 0 {
            return Some(node);
        }
        *n -= 1;
        for child in &node.children {
            if let Some(result) = Self::selected_(child, n) {
                return Some(result);
            }
        }
        None
    }

    fn find_with_path_<'a>(
        node: &'a mut FileTreeItem,
        path: &Path,
    ) -> Option<&'a mut FileTreeItem> {
        if node.path == path {
            return Some(node);
        } else {
            for ch in node.children.iter_mut() {
                if let Some(parent) = Self::find_with_path_(ch, path) {
                    return Some(parent);
                }
            }
        }
        None
    }

    pub fn flatten(&self) -> Vec<&FileTreeItem> {
        Self::flatten_(&self.root, 0)
            .into_iter()
            .map(|e| e.0)
            .collect()
    }

    pub fn flatten_with_depth(&self) -> Vec<(&FileTreeItem, usize)> {
        Self::flatten_(&self.root, 0)
    }

    pub fn selected(&self) -> Option<&FileTreeItem> {
        let mut n = self.selection;
        Self::selected_(&self.root, &mut n)
    }

    pub fn selected_mut(&mut self) -> Option<&mut FileTreeItem> {
        let mut n = self.selection;
        Self::selected_mut_(&mut self.root, &mut n)
    }

    pub fn take_selected(&mut self) -> Option<FileTreeItem> {
        let selected = self.selected().cloned()?;
        let parent = self.find_with_path(selected.path.parent().unwrap())?;
        Some(
            parent
                .children
                .remove(parent.children.iter().position(|c| *c == selected).unwrap()),
        )
    }

    pub fn find_with_path(&mut self, path: &Path) -> Option<&mut FileTreeItem> {
        Self::find_with_path_(&mut self.root, path)
    }

    pub fn move_up(&mut self) {
        self.selection = self.selection.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        let len = self.flatten().len();
        if self.selection < len - 1 {
            self.selection += 1;
        }
    }

    pub fn goto_start(&mut self) {
        self.selection = 0;
    }

    pub fn goto_end(&mut self) {
        let len = self.flatten().len();
        self.selection = len - 1;
    }

    pub fn delete_selection(&mut self) {}

    pub fn reload(&mut self) {
        *self = Self::new();
        self.open = true;
    }
}

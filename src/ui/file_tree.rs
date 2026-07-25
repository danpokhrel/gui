use std::fs;
use std::path::PathBuf;

/// One entry in the cached file tree.
///
/// `FileEntry` is a plain data snapshot of the filesystem taken at scan time.
/// It is *not* live: filesystem changes only appear after a rescan. This keeps
/// the UI fast (no per-frame directory reads) and avoids borrowing the OS
/// filesystem handle across ImGui frames.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
}

impl FileEntry {
    /// Recursively scan `root` and build a cached tree.
    ///
    /// Hidden entries (names starting with `.`) and the `target` build-output
    /// directory are skipped to keep the tree focused on source. Directories are
    /// listed before files, both alphabetically — the standard file-tree order.
    pub fn scan(root: &PathBuf) -> Option<Self> {
        let meta = fs::metadata(root).ok()?;
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_owned)
            .unwrap_or_else(|| root.display().to_string());
        let mut entry = FileEntry {
            name,
            path: root.clone(),
            is_dir: meta.is_dir(),
            children: Vec::new(),
        };
        if entry.is_dir {
            entry.children = read_dir_sorted(root);
        }
        Some(entry)
    }
}

/// Read a directory and return sorted, filtered child entries (recursively).
fn read_dir_sorted(dir: &PathBuf) -> Vec<FileEntry> {
    let mut dirs: Vec<FileEntry> = Vec::new();
    let mut files: Vec<FileEntry> = Vec::new();

    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };

    for entry in rd.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Skip hidden entries and the build output directory.
        if name.starts_with('.') || name == "target" {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        let mut child = FileEntry {
            name: name.to_owned(),
            path,
            is_dir,
            children: Vec::new(),
        };
        if is_dir {
            child.children = read_dir_sorted(&child.path);
            dirs.push(child);
        } else {
            files.push(child);
        }
    }

    dirs.sort_by_cached_key(|e| e.name.to_lowercase());
    files.sort_by_cached_key(|e| e.name.to_lowercase());
    dirs.append(&mut files);
    dirs
}

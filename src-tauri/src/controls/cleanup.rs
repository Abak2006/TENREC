// TENREC - Storage cleanup
//
// Deleting files cannot be undone, so the UI always shows a scan first and asks for
// confirmation. Only files inside each category's own folder are touched, reparse points
// (symlinks, junctions) are never followed, and locked files are skipped.
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use windows_sys::Win32::UI::Shell::{SHEmptyRecycleBinW, SHQueryRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND, SHQUERYRBINFO};

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const MAX_FILES: usize = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    UserTemp,
    WindowsTemp,
    UpdateCache,
    CrashDumps,
    RecycleBin,
}

impl Category {
    pub const ALL: [Category; 5] = [Self::UserTemp, Self::CrashDumps, Self::RecycleBin, Self::WindowsTemp, Self::UpdateCache];

    fn info(self) -> (&'static str, &'static str, bool, Duration) {
        let day = Duration::from_secs(86_400);
        match self {
            Self::UserTemp => ("Your temporary files", "Files in your Temp folder not changed for 24 hours. Apps that are running keep theirs.", false, day),
            Self::CrashDumps => ("App crash dumps", "Memory dumps apps wrote when they crashed. Only useful when reporting a crash.", false, Duration::ZERO),
            Self::RecycleBin => ("Recycle Bin", "Everything in the Recycle Bin on all drives.", false, Duration::ZERO),
            Self::WindowsTemp => ("Windows temporary files", "Files in the Windows Temp folder not changed for 24 hours.", true, day),
            Self::UpdateCache => ("Windows Update downloads", "Update packages already downloaded more than 10 days ago. Windows downloads them again if it still needs them.", true, day * 10),
        }
    }

    fn root(self) -> Option<PathBuf> {
        let env = |k: &str| std::env::var(k).ok().map(PathBuf::from);
        match self {
            Self::UserTemp => env("TEMP"),
            Self::CrashDumps => env("LOCALAPPDATA").map(|p| p.join("CrashDumps")),
            Self::WindowsTemp => env("SystemRoot").map(|p| p.join("Temp")),
            Self::UpdateCache => env("SystemRoot").map(|p| p.join("SoftwareDistribution").join("Download")),
            Self::RecycleBin => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryScan {
    pub category: Category,
    pub title: String,
    pub description: String,
    pub requires_admin: bool,
    pub bytes: u64,
    pub files: u64,
    /// Set when the folder could not be read (e.g. needs administrator rights).
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub category: Category,
    pub freed_bytes: u64,
    pub deleted_files: u64,
    pub skipped_files: u64,
    pub error: Option<String>,
}

fn is_reparse(meta: &fs::Metadata) -> bool {
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

/// Files under `root` older than `min_age`, without following reparse points.
fn old_files(root: &Path, min_age: Duration) -> Result<Vec<(PathBuf, u64)>, String> {
    let meta = fs::symlink_metadata(root).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => String::new(),
        std::io::ErrorKind::PermissionDenied => "Needs administrator rights to read.".to_string(),
        _ => e.to_string(),
    })?;
    if is_reparse(&meta) || !meta.is_dir() {
        return Ok(Vec::new());
    }
    let cutoff = SystemTime::now().checked_sub(min_age).unwrap_or(SystemTime::UNIX_EPOCH);
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut root_denied = None;
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                if dir == root && e.kind() == std::io::ErrorKind::PermissionDenied {
                    root_denied = Some("Needs administrator rights to read.".to_string());
                }
                continue;
            }
        };
        for entry in entries.flatten() {
            let Ok(m) = entry.metadata() else { continue };
            if is_reparse(&m) {
                continue;
            }
            if m.is_dir() {
                stack.push(entry.path());
            } else if m.modified().map(|t| t <= cutoff).unwrap_or(false) {
                out.push((entry.path(), m.len()));
                if out.len() >= MAX_FILES {
                    return Ok(out);
                }
            }
        }
    }
    match root_denied {
        Some(e) => Err(e),
        None => Ok(out),
    }
}

fn recycle_bin() -> Result<(u64, u64), String> {
    let mut info = SHQUERYRBINFO { cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32, i64Size: 0, i64NumItems: 0 };
    let hr = unsafe { SHQueryRecycleBinW(std::ptr::null(), &mut info) };
    if hr < 0 {
        return Err(format!("Could not read the Recycle Bin (0x{:08X})", hr as u32));
    }
    let (size, items) = (info.i64Size, info.i64NumItems);
    Ok((size.max(0) as u64, items.max(0) as u64))
}

pub fn scan() -> Vec<CategoryScan> {
    Category::ALL
        .iter()
        .map(|&c| {
            let (title, description, requires_admin, age) = c.info();
            let (bytes, files, error) = match c {
                Category::RecycleBin => match recycle_bin() {
                    Ok((b, n)) => (b, n, None),
                    Err(e) => (0, 0, Some(e)),
                },
                _ => match c.root().ok_or("Folder not found".to_string()).and_then(|r| old_files(&r, age)) {
                    Ok(files) => (files.iter().map(|f| f.1).sum(), files.len() as u64, None),
                    Err(e) if e.is_empty() => (0, 0, None),
                    Err(e) => (0, 0, Some(e)),
                },
            };
            CategoryScan { category: c, title: title.into(), description: description.into(), requires_admin, bytes, files, error }
        })
        .collect()
}

pub fn clean(category: Category) -> CleanResult {
    let mut r = CleanResult { category, freed_bytes: 0, deleted_files: 0, skipped_files: 0, error: None };
    if category == Category::RecycleBin {
        let before = recycle_bin().unwrap_or((0, 0));
        let hr = unsafe { SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND) };
        // E_UNEXPECTED (0x8000FFFF) is returned when the bin is already empty.
        if hr < 0 && hr as u32 != 0x8000_FFFF {
            r.error = Some(format!("Could not empty the Recycle Bin (0x{:08X})", hr as u32));
        } else {
            let after = recycle_bin().unwrap_or((0, 0));
            r.freed_bytes = before.0.saturating_sub(after.0);
            r.deleted_files = before.1.saturating_sub(after.1);
        }
        return r;
    }
    let Some(root) = category.root() else {
        r.error = Some("Folder not found".into());
        return r;
    };
    let (_, _, _, age) = category.info();
    let (freed, deleted, skipped, error) = clean_folder(&root, age);
    r.freed_bytes = freed;
    r.deleted_files = deleted;
    r.skipped_files = skipped;
    r.error = error;
    r
}

/// Deletes files under `root` older than `min_age`. Never follows reparse points and
/// never deletes outside `root`. Returns (bytes freed, files deleted, files skipped, error).
pub fn clean_folder(root: &Path, min_age: Duration) -> (u64, u64, u64, Option<String>) {
    let files = match old_files(root, min_age) {
        Ok(f) => f,
        Err(e) if e.is_empty() => return (0, 0, 0, None),
        Err(e) => return (0, 0, 0, Some(e)),
    };
    let (mut freed, mut deleted, mut skipped) = (0u64, 0u64, 0u64);
    for (path, size) in files {
        if !path.starts_with(root) {
            skipped += 1;
            continue;
        }
        match fs::remove_file(&path) {
            Ok(_) => {
                freed += size;
                deleted += 1;
            }
            Err(_) => skipped += 1,
        }
    }
    remove_empty_dirs(root, root);
    (freed, deleted, skipped, None)
}

fn remove_empty_dirs(dir: &Path, root: &Path) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        if let Ok(m) = e.metadata() {
            if m.is_dir() && !is_reparse(&m) {
                remove_empty_dirs(&e.path(), root);
            }
        }
    }
    if dir != root {
        let _ = fs::remove_dir(dir); // only succeeds when empty
    }
}

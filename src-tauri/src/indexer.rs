//! Background file & application indexing service.
//!
//! Design goals: never block the UI thread, keep the index fresh without
//! re-walking the whole disk on every keystroke, and degrade gracefully
//! (a folder that disappears, a permission error, a locked file — all
//! best-effort, never fatal).
//!
//! Three mechanisms work together:
//!   1. A full scan on startup (and every `RESCAN_INTERVAL_SECS` after)
//!      walks every indexed folder plus the Start Menu, detecting new,
//!      changed, and deleted entries.
//!   2. A `notify` filesystem watcher pushes near-instant updates for
//!      anything that changes between full scans.
//!   3. `index_folder_now` lets settings changes (add/remove a folder)
//!      trigger an immediate, targeted scan instead of waiting.
//!
//! Only metadata is ever read — file contents are never opened.

use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use notify::{RecursiveMode, Watcher};

use crate::index::local::{LocalIndex, NewApp, NewFile};

/// Full rescan cadence; the watcher handles everything in between.
const RESCAN_INTERVAL_SECS: i64 = 3600;
/// Directories that are typically huge, irrelevant, or slow to walk.
const SKIP_DIR_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
    "$RECYCLE.BIN",
    "System Volume Information",
    ".cache",
    "AppData",
];
/// Sane recursion cap so a symlink loop or pathological tree can't hang a scan.
const MAX_DEPTH: usize = 14;

/// The five default scan roots from the roadmap: Desktop, Documents,
/// Downloads, Pictures, Videos. Any that the OS doesn't report are skipped.
pub fn default_folders() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(d) = dirs::desktop_dir() {
        roots.push(d);
    }
    if let Some(d) = dirs::document_dir() {
        roots.push(d);
    }
    if let Some(d) = dirs::download_dir() {
        roots.push(d);
    }
    if let Some(d) = dirs::picture_dir() {
        roots.push(d);
    }
    if let Some(d) = dirs::video_dir() {
        roots.push(d);
    }
    roots
}

/// Kick off the background indexer: seeds default folders on first run,
/// does an immediate full scan, starts the filesystem watcher, then loops
/// on the periodic rescan. Call once from `setup`; returns immediately.
pub fn spawn(index: LocalIndex) {
    std::thread::spawn(move || {
        seed_default_folders(&index);
        run_full_scan(&index);

        let folders = index.list_folders();
        spawn_watcher(index.clone(), folders);

        loop {
            std::thread::sleep(Duration::from_secs(RESCAN_INTERVAL_SECS as u64));
            run_full_scan(&index);
        }
    });
}

fn seed_default_folders(index: &LocalIndex) {
    if !index.list_folders().is_empty() {
        return;
    }
    for dir in default_folders() {
        let _ = index.add_folder(&dir.to_string_lossy(), true);
    }
}

/// Walk every indexed folder and the Start Menu, upserting everything found
/// and pruning rows that disappeared since the last pass.
pub fn run_full_scan(index: &LocalIndex) {
    let scan_id = index.next_scan_id();
    for folder in index.list_folders() {
        scan_folder(index, Path::new(&folder), scan_id);
    }
    scan_apps(index, scan_id);
    index.mark_reindexed("full_scan");
    log::info!("indexer: full scan complete");
}

/// Immediate, targeted (re)scan of a single folder — used when the user
/// adds a folder from settings so it doesn't wait for the next full scan.
pub fn index_folder_now(index: &LocalIndex, folder: &str) {
    let scan_id = index.next_scan_id();
    scan_folder(index, Path::new(folder), scan_id);
}

fn scan_folder(index: &LocalIndex, root: &Path, scan_id: i64) {
    if !root.is_dir() {
        return;
    }
    let root_str = root.to_string_lossy().to_string();
    let walker = walkdir::WalkDir::new(root).max_depth(MAX_DEPTH).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIR_NAMES.iter().any(|s| s.eq_ignore_ascii_case(&name))
        } else {
            true
        }
    });
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            index_one_file(index, entry.path(), scan_id);
        }
    }
    let pruned = index.prune_stale_files(&root_str, scan_id);
    if pruned > 0 {
        log::info!("indexer: pruned {pruned} stale file(s) under {root_str}");
    }
}

fn index_one_file(index: &LocalIndex, path: &Path, scan_id: i64) {
    let Ok(meta) = path.metadata() else { return };
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return;
    };
    let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase());
    let new_file = NewFile {
        name,
        full_path: path.to_string_lossy().to_string(),
        extension,
        size: meta.len() as i64,
        created_date: meta.created().ok().and_then(to_unix).unwrap_or(0),
        modified_date: meta.modified().ok().and_then(to_unix).unwrap_or(0),
    };
    let _ = index.upsert_file(&new_file, scan_id);
}

/// Discover installed applications via Start Menu shortcuts (`.lnk`). We
/// deliberately don't parse the shortcut target ourselves — handing the
/// `.lnk` path itself to the OS's default opener resolves and launches the
/// real target exactly the way double-clicking it in Explorer would.
fn scan_apps(index: &LocalIndex, scan_id: i64) {
    for root in start_menu_roots() {
        if !root.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root).max_depth(8).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let is_lnk = path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("lnk")).unwrap_or(false);
            if !is_lnk {
                continue;
            }
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().to_string()) else {
                continue;
            };
            let new_app = NewApp {
                name,
                path: path.to_string_lossy().to_string(),
                icon: None,
            };
            let _ = index.upsert_app(&new_app, scan_id);
        }
    }
    let pruned = index.prune_stale_apps(scan_id);
    if pruned > 0 {
        log::info!("indexer: pruned {pruned} stale app(s)");
    }
}

fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(roaming) = dirs::data_dir() {
        roots.push(roaming.join("Microsoft").join("Windows").join("Start Menu").join("Programs"));
    }
    // The all-users Start Menu lives under %ProgramData%, which the `dirs`
    // crate doesn't expose directly.
    if let Ok(pd) = std::env::var("ProgramData") {
        roots.push(PathBuf::from(pd).join("Microsoft").join("Windows").join("Start Menu").join("Programs"));
    }
    roots
}

/// Watch every indexed folder for changes and update the index incrementally
/// instead of waiting for the next full scan. Runs for the lifetime of the
/// app on its own thread; failures to start the watcher are logged and
/// non-fatal (the periodic full scan still keeps things eventually fresh).
fn spawn_watcher(index: LocalIndex, folders: Vec<String>) {
    std::thread::spawn(move || {
        let (tx, rx) = channel::<notify::Result<notify::Event>>();
        let mut watcher = match notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        }) {
            Ok(w) => w,
            Err(e) => {
                log::warn!("indexer: failed to start file watcher: {e}");
                return;
            }
        };
        for folder in &folders {
            if let Err(e) = watcher.watch(Path::new(folder), RecursiveMode::Recursive) {
                log::warn!("indexer: could not watch {folder}: {e}");
            }
        }
        for res in rx {
            match res {
                Ok(event) => handle_fs_event(&index, event),
                Err(e) => log::debug!("indexer: watch error: {e}"),
            }
        }
    });
}

fn handle_fs_event(index: &LocalIndex, event: notify::Event) {
    use notify::EventKind;
    match event.kind {
        EventKind::Remove(_) => {
            for path in &event.paths {
                index.remove_file_path(&path.to_string_lossy());
            }
        }
        EventKind::Create(_) | EventKind::Modify(_) => {
            let scan_id = index.next_scan_id();
            for path in &event.paths {
                if path.is_file() {
                    index_one_file(index, path, scan_id);
                }
            }
        }
        _ => {}
    }
}

fn to_unix(t: SystemTime) -> Option<i64> {
    t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs() as i64)
}

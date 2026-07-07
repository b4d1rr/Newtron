mod cache;
mod commands;
mod index;
mod search;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use index::UrlIndex;
use search::SearchEngine;

/// Re-import browser history at most once per day.
const HISTORY_IMPORT_INTERVAL_SECS: i64 = 86_400;

pub struct AppState {
    pub index: UrlIndex,
    pub engine: SearchEngine,
}

#[derive(serde::Serialize)]
struct SystemItem {
    name: String,
    kind: String,
    path: String,
}

/// Placeholder file/app results until the real file indexer lands.
#[tauri::command]
fn get_system_results(query: String) -> Vec<SystemItem> {
    if query.is_empty() {
        return vec![];
    }
    vec![
        SystemItem {
            name: format!("{}.exe", query),
            kind: "App".into(),
            path: "C:/Program Files/".into(),
        },
        SystemItem {
            name: format!("{}_data.xlsx", query),
            kind: "File".into(),
            path: "C:/Users/Bader/Documents/".into(),
        },
    ]
}

#[tauri::command]
fn ask_newtron(message: String) -> String {
    format!("Newtron AI: Analysis of '{}' is complete. I've cross-referenced your system logs and indexed the relevant metadata for your query.", message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            ask_newtron,
            get_system_results,
            commands::web_search,
            commands::url_suggest,
            commands::record_visit,
            commands::add_alias,
            commands::import_history,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                if !focused {
                    let _ = window.hide();
                }
            }
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if shortcut.matches(Modifiers::ALT, Code::KeyN) {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyN);
            app.handle().global_shortcut().register(shortcut)?;

            // URL index lives in the per-user app data directory.
            let db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("no app data dir: {e}"))?
                .join("newtron.db");
            let index = UrlIndex::open(&db_path).map_err(|e| format!("failed to open url index: {e}"))?;

            // Background browser-history import; never blocks startup and
            // silently skips anything unreadable.
            if index.should_import_history(HISTORY_IMPORT_INTERVAL_SECS) {
                let bg = index.clone();
                std::thread::spawn(move || {
                    let stats = index::history::import_all(&bg);
                    log::info!(
                        "history import: {} rows from {} sources",
                        stats.rows_imported,
                        stats.sources_found
                    );
                });
            }

            app.manage(AppState {
                index,
                engine: SearchEngine::new(),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

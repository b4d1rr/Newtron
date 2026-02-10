use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};
use serde::Serialize;

#[derive(Serialize)]
struct SystemItem {
    name: String,
    kind: String,
    path: String,
}

#[tauri::command]
fn ask_newtron(message: String) -> String {
    format!("Newtron AI: Analysis of '{}' is complete. I've cross-referenced your system logs and indexed the relevant metadata for your query.", message)
}

#[tauri::command]
fn get_system_results(query: String) -> Vec<SystemItem> {
    if query.is_empty() { return vec![]; }
    vec![
        SystemItem { name: format!("{}.exe", query), kind: "App".into(), path: "C:/Program Files/".into() },
        SystemItem { name: format!("{}_data.xlsx", query), kind: "File".into(), path: "C:/Users/Bader/Documents/".into() },
        SystemItem { name: format!("Search web for '{}'", query), kind: "Web".into(), path: "https://google.com".into() },
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ask_newtron, get_system_results])
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
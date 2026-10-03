use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// Messages we send to the UI. The UI owns the animation, so it decides
// when the native window actually gets hidden.
const EVT_SHOW: &str = "newtron:show";
const EVT_TOGGLE: &str = "newtron:toggle";
const EVT_HIDE: &str = "newtron:hide";

fn tell_ui(app: &AppHandle, event: &str) {
    let _ = app.emit_to("main", event, ());
}

// Make the native window visible and focused, then tell the UI to play its open animation.
fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        tell_ui(app, EVT_SHOW);
    }
}

// Alt+N: if the window is up, let the UI decide (close, or reopen if it's mid-close).
// If it's hidden, show it.
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            tell_ui(app, EVT_TOGGLE);
        } else {
            show_window(app);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyN);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, pressed, event| {
                    // The shortcut fires on press and on release; we only want the press.
                    if pressed == &shortcut && event.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            // If another app already owns Alt+N, don't crash, just say so.
            if let Err(err) = app.global_shortcut().register(shortcut) {
                eprintln!("Could not register Alt+N: {err}");
            }

            let show = MenuItem::with_id(app, "show", "Show Newtron", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Newtron", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Closing the window never quits, it just asks the UI to close itself.
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                tell_ui(window.app_handle(), EVT_HIDE);
            }
            // Clicking anywhere outside Newtron takes focus away, so ask the UI to close.
            tauri::WindowEvent::Focused(false) => {
                tell_ui(window.app_handle(), EVT_HIDE);
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#![windows_subsystem = "windows"]
mod data;
mod desktop;

use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

struct NativeState {
    editing: AtomicBool,
    ready: AtomicBool,
    shortcut: Mutex<String>,
    warning: Mutex<Option<String>>,
}
struct StartupMenu(CheckMenuItem<tauri::Wry>);
#[derive(Clone, Serialize)]
struct NativeStatus {
    editing: bool,
    shortcut: String,
    autostart: bool,
    warning: Option<String>,
}

fn status(app: &tauri::AppHandle) -> Result<NativeStatus, String> {
    let state = app.state::<NativeState>();
    let shortcut = state.shortcut.lock().map_err(|e| e.to_string())?.clone();
    let warning = state.warning.lock().map_err(|e| e.to_string())?.clone();
    Ok(NativeStatus {
        editing: state.editing.load(Ordering::SeqCst),
        shortcut,
        autostart: app.autolaunch().is_enabled().map_err(|e| e.to_string())?,
        warning,
    })
}
fn report(app: &tauri::AppHandle, error: String) {
    eprintln!("{error}");
    if let Ok(mut warning) = app.state::<NativeState>().warning.lock() {
        *warning = Some(error.clone());
    }
    let _ = app.emit("native-error", error);
}
#[tauri::command]
fn set_editing(app: tauri::AppHandle, editing: bool) -> Result<NativeStatus, String> {
    desktop::apply(&app, editing)?;
    app.state::<NativeState>()
        .editing
        .store(editing, Ordering::SeqCst);
    let current = status(&app)?;
    app.emit("native-status", current.clone())
        .map_err(|e| e.to_string())?;
    Ok(current)
}
fn toggle(app: &tauri::AppHandle) {
    let editing = !app.state::<NativeState>().editing.load(Ordering::SeqCst);
    if let Err(e) = set_editing(app.clone(), editing) {
        report(app, e);
    }
}
#[tauri::command]
fn frontend_ready(app: tauri::AppHandle) -> Result<NativeStatus, String> {
    app.state::<NativeState>()
        .ready
        .store(true, Ordering::SeqCst);
    let editing = app.state::<NativeState>().editing.load(Ordering::SeqCst);
    set_editing(app, editing)
}
#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|e| e.to_string())?;
    app.state::<StartupMenu>()
        .0
        .set_checked(enabled)
        .map_err(|e| e.to_string())?;
    app.emit("native-status", status(&app)?)
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
fn set_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    let parsed: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("Invalid shortcut: {e}"))?;
    let state = app.state::<NativeState>();
    let mut old = state.shortcut.lock().map_err(|e| e.to_string())?;
    if *old == shortcut && app.global_shortcut().is_registered(parsed) {
        return Ok(());
    }
    // Register first: a conflict must not strand the user without their previous hotkey.
    app.global_shortcut()
        .register(parsed)
        .map_err(|e| format!("That shortcut is unavailable. Try another combination. ({e})"))?;
    if let Ok(previous) = old.parse::<Shortcut>() {
        if previous != parsed && app.global_shortcut().is_registered(previous) {
            if let Err(e) = app.global_shortcut().unregister(previous) {
                let _ = app.global_shortcut().unregister(parsed);
                return Err(e.to_string());
            }
        }
    }
    *old = shortcut;
    if let Ok(mut warning) = state.warning.lock() {
        *warning = None;
    }
    Ok(())
}
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Err(e) = set_editing(app.clone(), true) {
                report(app, e);
            }
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            data::load_board,
            data::save_board,
            frontend_ready,
            set_editing,
            set_autostart,
            set_shortcut,
            quit_app
        ])
        .setup(|app| {
            // The environment override exists only in debug builds for isolated native tests.
            let data_dir = app.path().app_data_dir()?;
            #[cfg(debug_assertions)]
            let data_dir = std::env::var_os("LIFE_KANBAN_TEST_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or(data_dir);
            let path = data_dir.join("board.json");
            let saved = data::load(&path);
            let shortcut = saved
                .as_ref()
                .map(|s| s.board.settings.shortcut.clone())
                .unwrap_or_else(|_| "Control+Shift+K".into());
            let warning = saved.err();
            app.manage(data::DataState {
                path,
                lock: Mutex::new(()),
            });
            app.manage(NativeState {
                editing: AtomicBool::new(false),
                ready: AtomicBool::new(false),
                shortcut: Mutex::new(shortcut.clone()),
                warning: Mutex::new(warning),
            });
            // Keep an existing startup preference, but refresh its executable
            // after an update from a portable/development copy to the installed app.
            #[cfg(not(debug_assertions))]
            if app.autolaunch().is_enabled()? {
                if let Err(e) = app.autolaunch().enable() {
                    report(app.handle(), format!("Could not update Windows startup: {e}"));
                }
            }
            let title = MenuItem::with_id(app, "title", "Life Kanban", false, None::<&str>)?;
            let edit = MenuItem::with_id(app, "edit", "Edit Kanban", true, None::<&str>)?;
            let view = MenuItem::with_id(app, "view", "View Mode", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Open Settings", true, None::<&str>)?;
            let startup = CheckMenuItem::with_id(
                app,
                "startup",
                "Start with Windows",
                true,
                app.autolaunch().is_enabled()?,
                None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(
                app,
                &[&title, &separator, &edit, &view, &settings, &startup, &quit],
            )?;
            app.manage(StartupMenu(startup));
            TrayIconBuilder::with_id("life-kanban")
                .tooltip("Life Kanban — your quiet desktop board")
                .icon(app.default_window_icon().ok_or("Missing app icon")?.clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let result: Result<(), String> = (|| {
                        match event.id.as_ref() {
                            "edit" => {
                                set_editing(app.clone(), true)?;
                            }
                            "view" => {
                                set_editing(app.clone(), false)?;
                            }
                            "settings" => {
                                set_editing(app.clone(), true)?;
                                app.emit("open-settings", ()).map_err(|e| e.to_string())?;
                            }
                            "startup" => {
                                let enabled =
                                    !app.autolaunch().is_enabled().map_err(|e| e.to_string())?;
                                set_autostart(app.clone(), enabled)?;
                            }
                            "quit" => {
                                if app.state::<NativeState>().ready.load(Ordering::SeqCst) {
                                    app.emit("request-quit", ()).map_err(|e| e.to_string())?;
                                } else {
                                    app.exit(0);
                                }
                            }
                            _ => {}
                        }
                        Ok(())
                    })();
                    if let Err(e) = result {
                        report(app, e);
                    }
                })
                .build(app)?;
            if let Err(e) = set_shortcut(app.handle().clone(), shortcut) {
                report(
                    app.handle(),
                    format!("{e} Use the tray icon to edit your board or change the shortcut."),
                );
            }
            let window = app
                .get_webview_window("main")
                .ok_or("Missing board window")?;
            desktop::fit_work_area(&window)?;
            // Start click-through even before the frontend's first paint.
            window.set_ignore_cursor_events(true)?;
            window.set_focusable(false)?;
            window.set_always_on_bottom(true)?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                if let Err(e) = set_editing(window.app_handle().clone(), false) {
                    report(window.app_handle(), e);
                }
            }
            tauri::WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(webview) = window.app_handle().get_webview_window("main") {
                    if let Err(e) = desktop::fit_work_area(&webview) {
                        report(window.app_handle(), e);
                    }
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("Life Kanban could not start");
}

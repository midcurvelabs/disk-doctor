mod catalog;
mod clean;
mod commands;
mod disk;
mod history;
mod memory;
mod scanner;

use std::time::Duration;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn format_free_label(pct: f64) -> String {
    format!("{:>3.0}%", pct.clamp(0.0, 100.0))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::disk_status,
            commands::memory_pressure,
            commands::scan_all,
            commands::scan_one,
            commands::clean,
            commands::history,
            commands::open_in_finder,
        ])
        .setup(|app| {
            // Build the tray menu.
            let show_item = MenuItem::with_id(app, "show", "Show Disk Doctor", true, None::<&str>)?;
            let scan_item = MenuItem::with_id(app, "scan", "Re-scan", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &scan_item, &sep, &quit_item])?;

            // Tray icon — uses the default app icon for v1; custom icon is P1.
            let icon_bytes = include_bytes!("../icons/icon.png");
            let icon = Image::from_bytes(icon_bytes)?;

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .icon_as_template(true)
                .title("…")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "scan" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.emit("trigger-scan", ());
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let visible = w.is_visible().unwrap_or(false);
                            if visible {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Periodically update the tray title with current free disk %.
            let tray_handle = tray.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let status = disk::status();
                    let _ = tray_handle.set_title(Some(format_free_label(status.free_percent)));
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            });

            // Hide the dock icon on macOS — we live in the menubar.
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Don't quit — just hide the popover.
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running disk-doctor");
}

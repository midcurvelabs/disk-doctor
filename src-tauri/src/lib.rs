mod catalog;
mod clean;
mod commands;
mod disk;
mod history;
mod memory;
pub mod pipeline;
mod scanner;

use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

fn format_free_label(free_bytes: u64, threshold_gb: u64) -> String {
    let gb = free_bytes as f64 / 1_073_741_824.0;
    let mark = if gb < threshold_gb as f64 { " !" } else { "" };
    format!("{gb:.0} GB{mark}")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::disk_status,
            commands::memory_pressure,
            commands::top_processes,
            commands::scan_all,
            commands::scan_one,
            commands::clean,
            commands::history,
            commands::open_in_finder,
            commands::open_activity_monitor,
            commands::quit_process,
            commands::pipeline_scan,
            commands::pipeline_run,
            commands::pipeline_set_threshold,
            commands::pipeline_edit_rules,
            commands::empty_trash,
        ])
        .setup(|app| {
            // Build the tray menu.
            let show_item = MenuItem::with_id(app, "show", "Show Disk Doctor", true, None::<&str>)?;
            let scan_item = MenuItem::with_id(app, "scan", "Re-scan", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &scan_item, &sep, &quit_item])?;

            // Tray icon — uses the default app icon for v1; custom icon is P1.
            let icon = tauri::include_image!("icons/icon.png");

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

            // Tray title shows free GB (with "!" under the threshold). Every 10
            // minutes under the threshold, check the Pipeline rules and notify
            // (at most once per `notify_every_hours`) when leftovers are ready.
            let tray_handle = tray.clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_notification::NotificationExt;
                let mut last_check = std::time::Instant::now() - Duration::from_secs(3600);
                let mut last_notify: Option<std::time::Instant> = None;
                loop {
                    let status = disk::status();
                    let cfg = pipeline::load_config().ok();
                    let threshold = cfg.as_ref().map(|c| c.threshold_gb).unwrap_or(30);
                    let _ = tray_handle.set_title(Some(format_free_label(status.free_bytes, threshold)));

                    let low = status.free_bytes < threshold * 1_073_741_824;
                    let every = Duration::from_secs(3600 * cfg.as_ref().map(|c| c.notify_every_hours).unwrap_or(6).max(1));
                    let quiet = last_notify.map_or(true, |t| t.elapsed() >= every);
                    if low && quiet && last_check.elapsed() >= Duration::from_secs(600) {
                        last_check = std::time::Instant::now();
                        let ready = tokio::task::spawn_blocking(pipeline::ready_bytes).await.unwrap_or(0);
                        let gb = |b: u64| b as f64 / 1_073_741_824.0;
                        let body = if ready >= 1_073_741_824 {
                            format!("{:.0} GB free. {:.1} GB of pipeline leftovers are ready to clean.", gb(status.free_bytes), gb(ready))
                        } else {
                            format!("{:.0} GB free, under your {} GB line. No pipeline leftovers are ready yet.", gb(status.free_bytes), threshold)
                        };
                        let _ = app_handle.notification().builder().title("Disk Doctor: space is low").body(body).show();
                        last_notify = Some(std::time::Instant::now());
                    }
                    tokio::time::sleep(Duration::from_secs(60)).await;
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

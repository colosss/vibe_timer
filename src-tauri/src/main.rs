// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod notifications;
mod power;
mod presets;
mod settings;
mod timer;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use timer::TimerManager;

/// Определяет, сворачивать ли приложение в трей вместо закрытия.
pub(crate) struct CloseGuard(pub(crate) AtomicBool);

fn main() {
    tauri::Builder::default()
        .manage(TimerManager::default())
        .manage(CloseGuard(AtomicBool::new(true)))
        .setup(|app| {
            let saved = settings::read_settings(app.handle());

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_always_on_top(saved.always_on_top);
            }
            app.state::<CloseGuard>()
                .0
                .store(saved.minimize_to_tray, Ordering::Relaxed);

            let show = MenuItem::with_id(app, "show", "Показать", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Скрыть", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or_else(|| "Не удалось загрузить иконку приложения".to_string())?;

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .tooltip("Vibe Timer")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
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
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app);

            if let Err(error) = tray {
                // Без трея скрытое окно нельзя было бы вернуть.
                eprintln!("Не удалось создать иконку в трее: {}", error);
                app.state::<CloseGuard>().0.store(false, Ordering::Relaxed);
            } else if saved.start_minimized {
                // Скрываем только когда трей успешно создан.
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let guard = window.app_handle().state::<CloseGuard>();
                if guard.0.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            timer::start_timer,
            timer::pause_timer,
            timer::cancel_timer,
            timer::get_timer_state,
            power::execute_power_action,
            power::test_power_action,
            notifications::trigger_notification,
            notifications::test_notification,
            presets::load_presets,
            presets::add_preset,
            presets::update_preset,
            presets::delete_preset,
            settings::load_settings,
            settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

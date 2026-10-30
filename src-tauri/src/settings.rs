use crate::notifications::NotificationType;
use crate::CloseGuard;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub window_opacity: u8,       // 30-100
    pub default_sound_volume: u8, // 0-100
    pub default_notification: NotificationType,
    pub always_on_top: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            window_opacity: 95,
            default_sound_volume: 80,
            default_notification: NotificationType::Both,
            always_on_top: false,
            minimize_to_tray: true,
            start_minimized: false,
        }
    }
}

fn normalize(mut settings: AppSettings) -> AppSettings {
    settings.window_opacity = settings.window_opacity.clamp(30, 100);
    settings.default_sound_volume = settings.default_sound_volume.min(100);
    settings
}

fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Не удалось получить путь к директории приложения: {}", e))?;

    fs::create_dir_all(&app_dir).map_err(|e| format!("Ошибка создания директории: {}", e))?;

    Ok(app_dir.join("settings.json"))
}

fn read_settings_file(app: &AppHandle) -> Result<AppSettings, String> {
    let path = get_settings_path(app)?;

    if !path.exists() {
        let defaults = AppSettings::default();
        write_settings_file(app, &defaults)?;
        return Ok(defaults);
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Ошибка чтения настроек: {}", e))?;
    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Ошибка парсинга настроек: {}", e))?;

    Ok(normalize(settings))
}

fn write_settings_file(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app)?;
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Ошибка сериализации настроек: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Ошибка записи настроек: {}", e))
}

/// Загрузка настроек при старте приложения: при любой ошибке — значения по умолчанию.
pub fn read_settings(app: &AppHandle) -> AppSettings {
    read_settings_file(app).unwrap_or_default()
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    read_settings_file(&app)
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    window: WebviewWindow,
    settings: AppSettings,
) -> Result<(), String> {
    let settings = normalize(settings);
    write_settings_file(&app, &settings)?;

    window
        .set_always_on_top(settings.always_on_top)
        .map_err(|e| format!("Ошибка установки режима «поверх всех окон»: {}", e))?;

    if let Some(guard) = app.try_state::<CloseGuard>() {
        guard.0.store(settings.minimize_to_tray, Ordering::Relaxed);
    }

    Ok(())
}

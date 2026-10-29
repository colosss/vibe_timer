use crate::notifications::NotificationType;
use crate::timer::PowerAction;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub duration: u64,
    pub action: PowerAction,
    pub warning_time: u64,
    pub notification_type: NotificationType,
    pub sound_volume: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetsStorage {
    pub presets: Vec<Preset>,
}

impl Default for PresetsStorage {
    fn default() -> Self {
        let preset = |name: &str, duration: u64, warning_time: u64| Preset {
            name: name.to_string(),
            duration,
            action: PowerAction::Sleep,
            warning_time,
            notification_type: NotificationType::Both,
            sound_volume: 80,
        };

        Self {
            presets: vec![
                preset("30 минут", 1800, 30),
                preset("1 час", 3600, 30),
                preset("2 часа", 7200, 60),
                preset("3 часа", 10800, 60),
            ],
        }
    }
}

fn get_presets_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Не удалось получить путь к директории приложения: {}", e))?;

    fs::create_dir_all(&app_dir).map_err(|e| format!("Ошибка создания директории: {}", e))?;

    Ok(app_dir.join("presets.json"))
}

fn read_presets(app: &AppHandle) -> Result<Vec<Preset>, String> {
    let path = get_presets_path(app)?;

    if !path.exists() {
        let defaults = PresetsStorage::default();
        write_presets(app, &defaults.presets)?;
        return Ok(defaults.presets);
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Ошибка чтения файла пресетов: {}", e))?;
    let storage: PresetsStorage =
        serde_json::from_str(&content).map_err(|e| format!("Ошибка парсинга пресетов: {}", e))?;

    Ok(storage.presets)
}

fn write_presets(app: &AppHandle, presets: &[Preset]) -> Result<(), String> {
    let path = get_presets_path(app)?;
    let storage = PresetsStorage {
        presets: presets.to_vec(),
    };

    let content = serde_json::to_string_pretty(&storage)
        .map_err(|e| format!("Ошибка сериализации пресетов: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Ошибка записи файла пресетов: {}", e))
}

fn normalize_preset(mut preset: Preset) -> Result<Preset, String> {
    preset.name = preset.name.trim().to_string();
    preset.sound_volume = preset.sound_volume.min(100);

    if preset.name.is_empty() {
        return Err("Название пресета не может быть пустым".to_string());
    }
    if preset.duration == 0 {
        return Err("Длительность пресета должна быть больше нуля".to_string());
    }

    Ok(preset)
}

#[tauri::command]
pub fn load_presets(app: AppHandle) -> Result<Vec<Preset>, String> {
    read_presets(&app)
}

#[tauri::command]
pub fn add_preset(app: AppHandle, preset: Preset) -> Result<(), String> {
    let preset = normalize_preset(preset)?;
    let mut presets = read_presets(&app)?;

    if presets.iter().any(|p| p.name == preset.name) {
        return Err("Пресет с таким названием уже существует".to_string());
    }

    presets.push(preset);
    write_presets(&app, &presets)
}

#[tauri::command]
pub fn update_preset(app: AppHandle, old_name: String, preset: Preset) -> Result<(), String> {
    let preset = normalize_preset(preset)?;
    let mut presets = read_presets(&app)?;

    let index = presets
        .iter()
        .position(|p| p.name == old_name)
        .ok_or_else(|| "Пресет не найден".to_string())?;

    if presets
        .iter()
        .enumerate()
        .any(|(i, p)| i != index && p.name == preset.name)
    {
        return Err("Пресет с таким названием уже существует".to_string());
    }

    presets[index] = preset;
    write_presets(&app, &presets)
}

#[tauri::command]
pub fn delete_preset(app: AppHandle, name: String) -> Result<(), String> {
    let mut presets = read_presets(&app)?;
    let before = presets.len();
    presets.retain(|p| p.name != name);

    if presets.len() == before {
        return Err("Пресет не найден".to_string());
    }

    write_presets(&app, &presets)
}

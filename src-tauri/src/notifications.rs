use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Sound,
    /// Показать окно с предупреждением и обратным отсчётом.
    /// `alias` сохраняет совместимость со старыми настройками и пресетами,
    /// где этот вариант назывался `visual`.
    #[serde(alias = "visual")]
    Notification,
    #[default]
    Both,
    Silent,
}

/// Воспроизведение системного beep-звука.
pub fn play_beep(volume: u8) -> Result<(), String> {
    use rodio::{OutputStream, Sink, Source};
    use std::time::Duration;

    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Не удалось инициализировать аудио: {}", e))?;
    let sink =
        Sink::try_new(&stream_handle).map_err(|e| format!("Не удалось создать sink: {}", e))?;

    sink.set_volume((volume.min(100) as f32) / 100.0);

    let source = rodio::source::SineWave::new(800.0)
        .take_duration(Duration::from_millis(300))
        .amplify(0.3);

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

/// Показывает окно приложения, даже если оно свёрнуто в трей,
/// чтобы предупреждение действительно было заметно.
fn show_notification_window(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

async fn play_beep_async(volume: u8) -> Result<(), String> {
    tokio::task::spawn_blocking(move || play_beep(volume))
        .await
        .map_err(|e| format!("Ошибка аудиопотока: {}", e))?
}

#[tauri::command]
pub async fn trigger_notification(
    notification_type: NotificationType,
    volume: u8,
    window: WebviewWindow,
) -> Result<(), String> {
    if matches!(
        notification_type,
        NotificationType::Notification | NotificationType::Both
    ) {
        show_notification_window(&window);
    }

    if matches!(
        notification_type,
        NotificationType::Sound | NotificationType::Both
    ) {
        play_beep_async(volume).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::NotificationType;

    #[test]
    fn legacy_visual_deserializes_as_notification() {
        let value: NotificationType = serde_json::from_str("\"visual\"").unwrap();
        assert_eq!(value, NotificationType::Notification);
    }

    #[test]
    fn notification_serializes_without_legacy_name() {
        let value = serde_json::to_string(&NotificationType::Notification).unwrap();
        assert_eq!(value, "\"notification\"");
    }
}

use serde::{Deserialize, Serialize};
use tauri::{UserAttentionType, WebviewWindow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Sound,
    Visual,
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

/// Привлечение внимания к окну (мигание в панели задач).
pub fn flash_window(window: &WebviewWindow) -> Result<(), String> {
    window
        .request_user_attention(Some(UserAttentionType::Critical))
        .map_err(|e| format!("Ошибка привлечения внимания: {}", e))
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
    match notification_type {
        NotificationType::Sound => play_beep_async(volume).await?,
        NotificationType::Visual => flash_window(&window)?,
        NotificationType::Both => {
            play_beep_async(volume).await?;
            flash_window(&window)?;
        }
        NotificationType::Silent => {}
    }

    Ok(())
}

#[tauri::command]
pub async fn test_notification(
    notification_type: NotificationType,
    volume: u8,
    window: WebviewWindow,
) -> Result<String, String> {
    match notification_type {
        NotificationType::Sound => {
            play_beep_async(volume).await?;
            Ok("Звуковое оповещение воспроизведено".to_string())
        }
        NotificationType::Visual => {
            flash_window(&window)?;
            Ok("Световое оповещение выполнено".to_string())
        }
        NotificationType::Both => {
            play_beep_async(volume).await?;
            flash_window(&window)?;
            Ok("Звуковое и световое оповещения выполнены".to_string())
        }
        NotificationType::Silent => Ok("Беззвучный режим".to_string()),
    }
}

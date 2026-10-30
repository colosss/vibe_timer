import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, Save } from "lucide-react";
import { useTimerStore } from "../stores/timerStore";
import type { AppSettings, NotificationType } from "../types";
import "./Settings.css";

interface SettingsProps {
  onClose: () => void;
}

function Settings({ onClose }: SettingsProps) {
  const settings = useTimerStore((state) => state.settings);
  const setSettings = useTimerStore((state) => state.setSettings);
  const [localSettings, setLocalSettings] = useState<AppSettings>(settings);
  const [saving, setSaving] = useState(false);
  const [preview, setPreview] = useState(false);

  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  // Превью уведомления автоматически скрывается.
  useEffect(() => {
    if (!preview) return;

    const timeout = window.setTimeout(() => setPreview(false), 2500);
    return () => window.clearTimeout(timeout);
  }, [preview]);

  // Живой предпросмотр прозрачности окна.
  useEffect(() => {
    document.documentElement.style.setProperty(
      "--app-opacity",
      String(localSettings.window_opacity / 100)
    );
  }, [localSettings.window_opacity]);

  const handleClose = () => {
    if (saving) return;

    // Возвращаем сохранённое значение, если пользователь отменил изменения.
    document.documentElement.style.setProperty(
      "--app-opacity",
      String(settings.window_opacity / 100)
    );
    onClose();
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("save_settings", { settings: localSettings });
      setSettings(localSettings);
      onClose();
    } catch (error) {
      console.error("Ошибка сохранения настроек:", error);
      alert(`Не удалось сохранить настройки: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  const testNotification = async () => {
    const type = localSettings.default_notification;

    try {
      await invoke("trigger_notification", {
        notificationType: type,
        volume: localSettings.default_sound_volume,
      });

      if (type === "notification" || type === "both") {
        setPreview(true);
      }
    } catch (error) {
      console.error("Ошибка теста оповещения:", error);
      alert(`Не удалось воспроизвести оповещение: ${error}`);
    }
  };

  return (
    <div className="settings-overlay" onClick={handleClose}>
      <div className="settings-modal" onClick={(event) => event.stopPropagation()}>
        {preview && (
          <div className="notification-preview" role="status">
            🔔 Уведомление: таймер завершён
          </div>
        )}
        <div className="settings-header">
          <h2>⚙️ Настройки</h2>
          <button className="close-btn" onClick={handleClose} title="Закрыть">
            <X size={24} />
          </button>
        </div>

        <div className="settings-content">
          <div className="setting-group">
            <label>Прозрачность окна: {localSettings.window_opacity}%</label>
            <input
              type="range"
              min="30"
              max="100"
              value={localSettings.window_opacity}
              onChange={(event) =>
                setLocalSettings({
                  ...localSettings,
                  window_opacity: Number(event.target.value),
                })
              }
            />
            <span className="setting-hint">От 30% до 100%</span>
          </div>

          <div className="setting-group">
            <label>Громкость звука: {localSettings.default_sound_volume}%</label>
            <input
              type="range"
              min="0"
              max="100"
              value={localSettings.default_sound_volume}
              onChange={(event) =>
                setLocalSettings({
                  ...localSettings,
                  default_sound_volume: Number(event.target.value),
                })
              }
            />
            <span className="setting-hint">От 0% (без звука) до 100%</span>
          </div>

          <div className="setting-group">
            <label>Тип оповещения по умолчанию:</label>
            <select
              value={localSettings.default_notification}
              onChange={(event) =>
                setLocalSettings({
                  ...localSettings,
                  default_notification: event.target.value as NotificationType,
                })
              }
            >
              <option value="sound">🔊 Только звук</option>
              <option value="notification">🔔 Только уведомление</option>
              <option value="both">🔊🔔 Звук + уведомление</option>
              <option value="silent">🔇 Без оповещения</option>
            </select>
            <span className="setting-hint">
              Уведомление — окно с обратным отсчётом и кнопкой отмены.
            </span>
            <button className="test-btn" onClick={testNotification}>
              Протестировать
            </button>
          </div>

          <div className="setting-group checkbox-group">
            <label>
              <input
                type="checkbox"
                checked={localSettings.always_on_top}
                onChange={(event) =>
                  setLocalSettings({
                    ...localSettings,
                    always_on_top: event.target.checked,
                  })
                }
              />
              <span>Поверх всех окон</span>
            </label>
            <label>
              <input
                type="checkbox"
                checked={localSettings.minimize_to_tray}
                onChange={(event) =>
                  setLocalSettings({
                    ...localSettings,
                    minimize_to_tray: event.target.checked,
                  })
                }
              />
              <span>Сворачивать в системный трей</span>
            </label>
            <label>
              <input
                type="checkbox"
                checked={localSettings.start_minimized}
                onChange={(event) =>
                  setLocalSettings({
                    ...localSettings,
                    start_minimized: event.target.checked,
                  })
                }
              />
              <span>Запускать свёрнутым (при следующем старте)</span>
            </label>
          </div>
        </div>

        <div className="settings-footer">
          <button className="cancel-btn" onClick={handleClose} disabled={saving}>
            Отмена
          </button>
          <button className="save-btn" onClick={handleSave} disabled={saving}>
            {saving ? (
              <span>Сохранение...</span>
            ) : (
              <>
                <Save size={18} />
                Сохранить
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}

export default Settings;

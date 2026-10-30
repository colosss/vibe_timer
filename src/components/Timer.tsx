import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTimerStore } from "../stores/timerStore";
import type { NotificationType, PowerAction, Preset, TimerConfig } from "../types";
import { formatTime } from "../utils/format";
import Presets from "./Presets";
import Settings from "./Settings";

const TIME_PRESETS = [
  { label: "30 мин", value: 1800 },
  { label: "1 час", value: 3600 },
  { label: "2 часа", value: 7200 },
  { label: "3 часа", value: 10800 },
];

const ACTIONS: { label: string; value: PowerAction }[] = [
  { label: "Сон", value: "sleep" },
  { label: "Гибернация", value: "hibernate" },
  { label: "Выключение", value: "shutdown" },
  { label: "Перезагрузка", value: "restart" },
  { label: "Перезагрузка + Сон", value: "restart_and_sleep" },
];

const WARNING_TIMES = [
  { label: "10 сек", value: 10 },
  { label: "30 сек", value: 30 },
  { label: "1 мин", value: 60 },
  { label: "2 мин", value: 120 },
];

function Timer() {
  const {
    isRunning,
    isPaused,
    remaining,
    total,
    settings,
    setRunning,
    setPaused,
    setRemaining,
    setTotal,
    setConfig,
    reset,
  } = useTimerStore();

  const [selectedDuration, setSelectedDuration] = useState(3600);
  const [customDuration, setCustomDuration] = useState("");
  const [selectedAction, setSelectedAction] = useState<PowerAction>("sleep");
  const [selectedWarningTime, setSelectedWarningTime] = useState(30);
  const [notificationType, setNotificationType] = useState<NotificationType>(
    settings.default_notification
  );
  const [soundVolume, setSoundVolume] = useState(settings.default_sound_volume);
  const starting = useRef(false);
  const [showPresets, setShowPresets] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  // Значения по умолчанию из настроек (применяются при их загрузке/сохранении).
  useEffect(() => {
    setNotificationType(settings.default_notification);
    setSoundVolume(settings.default_sound_volume);
  }, [settings.default_notification, settings.default_sound_volume]);

  const customMinutes = Number(customDuration);
  const hasCustomDuration =
    customDuration.trim() !== "" &&
    Number.isFinite(customMinutes) &&
    customMinutes > 0;
  const customSeconds = hasCustomDuration ? Math.round(customMinutes * 60) : 0;
  const previewSeconds = hasCustomDuration ? customSeconds : selectedDuration;
  const displaySeconds = isRunning ? remaining : previewSeconds;

  const handleStart = async () => {
    if (starting.current) return;
    const duration = hasCustomDuration ? customSeconds : selectedDuration;

    if (!Number.isFinite(duration) || duration <= 0) {
      alert("Укажите корректное время");
      return;
    }

    starting.current = true;
    const config: TimerConfig = {
      duration,
      action: selectedAction,
      warning_time: selectedWarningTime,
      notification_type: notificationType,
      sound_volume: soundVolume,
    };

    // Состояние выставляем до запуска, чтобы обработчик предупреждения уже видел config.
    setConfig(config);
    setTotal(duration);
    setRemaining(duration);
    setPaused(false);

    try {
      await invoke("start_timer", { config });
      setRunning(true);
    } catch (error) {
      console.error("Ошибка запуска таймера:", error);
      alert(`Не удалось запустить таймер: ${error}`);
      reset();
    } finally {
      starting.current = false;
    }
  };

  const handlePause = async () => {
    try {
      const paused = await invoke<boolean>("pause_timer");
      setPaused(paused);
    } catch (error) {
      console.error("Ошибка паузы:", error);
    }
  };

  const handleCancel = async () => {
    try {
      await invoke("cancel_timer");
      reset();
    } catch (error) {
      console.error("Ошибка отмены:", error);
      alert(`Не удалось отменить таймер: ${error}`);
    }
  };

  const handleSelectPreset = (preset: Preset) => {
    setSelectedDuration(preset.duration);
    setCustomDuration("");
    setSelectedAction(preset.action);
    setSelectedWarningTime(preset.warning_time);
    setNotificationType(preset.notification_type);
    setSoundVolume(preset.sound_volume);
  };

  const progress = total > 0 ? ((total - remaining) / total) * 100 : 0;

  return (
    <>
      {showPresets && (
        <Presets
          onClose={() => setShowPresets(false)}
          onSelectPreset={handleSelectPreset}
        />
      )}

      {showSettings && <Settings onClose={() => setShowSettings(false)} />}

      <div className="timer-container">
        <div className="timer-header">
          <h1>⏱ Vibe Timer</h1>
          <div className="header-buttons">
            <button
              className="header-btn"
              onClick={() => setShowPresets(true)}
              title="Пресеты"
            >
              📋
            </button>
            <button
              className="header-btn"
              onClick={() => setShowSettings(true)}
              title="Настройки"
            >
              ⚙️
            </button>
          </div>
        </div>

        <div className="timer-display">
          <div className="time-value">{formatTime(displaySeconds)}</div>
          {isRunning && (
            <div className="timer-status">
              {isPaused ? "⏸ Пауза" : "▶ Запущен"}
            </div>
          )}
        </div>

        {!isRunning && (
          <>
            <div className="presets">
              {TIME_PRESETS.map((preset) => (
                <button
                  key={preset.value}
                  className={`preset-btn ${
                    !hasCustomDuration && selectedDuration === preset.value
                      ? "active"
                      : ""
                  }`}
                  onClick={() => {
                    setSelectedDuration(preset.value);
                    setCustomDuration("");
                  }}
                >
                  {preset.label}
                </button>
              ))}
            </div>

            <div className="custom-input">
              <label>Своё время (минуты):</label>
              <input
                type="number"
                value={customDuration}
                onChange={(event) => setCustomDuration(event.target.value)}
                placeholder="Например: 90"
                min="1"
                step="1"
              />
            </div>

            <div className="select-group">
              <label>Действие:</label>
              <select
                value={selectedAction}
                onChange={(event) =>
                  setSelectedAction(event.target.value as PowerAction)
                }
              >
                {ACTIONS.map((action) => (
                  <option key={action.value} value={action.value}>
                    {action.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="select-group">
              <label>Время предупреждения:</label>
              <select
                value={selectedWarningTime}
                onChange={(event) =>
                  setSelectedWarningTime(Number(event.target.value))
                }
              >
                {WARNING_TIMES.map((time) => (
                  <option key={time.value} value={time.value}>
                    {time.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="select-group">
              <label>Оповещение:</label>
              <select
                value={notificationType}
                onChange={(event) =>
                  setNotificationType(event.target.value as NotificationType)
                }
              >
                <option value="sound">🔊 Звук</option>
                <option value="notification">🔔 Уведомление</option>
                <option value="both">🔊🔔 Звук + уведомление</option>
                <option value="silent">🔇 Без оповещения</option>
              </select>
            </div>

            <div className="slider-group">
              <label>Громкость: {soundVolume}%</label>
              <input
                type="range"
                min="0"
                max="100"
                value={soundVolume}
                onChange={(event) => setSoundVolume(Number(event.target.value))}
              />
            </div>

            <button className="start-btn" onClick={handleStart}>
              Запустить таймер
            </button>
          </>
        )}

        {isRunning && (
          <>
            <div className="progress-bar">
              <div
                className="progress-fill"
                style={{ width: `${progress}%` }}
              />
            </div>

            <div className="control-buttons">
              <button className="control-btn" onClick={handlePause}>
                {isPaused ? "▶ Продолжить" : "⏸ Пауза"}
              </button>
              <button className="control-btn cancel" onClick={handleCancel}>
                ✕ Отменить
              </button>
            </div>
          </>
        )}
      </div>
    </>
  );
}

export default Timer;

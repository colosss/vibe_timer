import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTimerStore } from "../stores/timerStore";
import type { NotificationType, PowerAction, Preset } from "../types";
import { formatDuration } from "../utils/format";
import "./Presets.css";

interface PresetsProps {
  onClose: () => void;
  onSelectPreset: (preset: Preset) => void;
}

const ACTIONS: { label: string; value: PowerAction }[] = [
  { label: "Сон", value: "sleep" },
  { label: "Гибернация", value: "hibernate" },
  { label: "Выключение", value: "shutdown" },
  { label: "Перезагрузка", value: "restart" },
  { label: "Перезагрузка + Сон", value: "restart_and_sleep" },
];

const NOTIFICATION_TYPES: { label: string; value: NotificationType }[] = [
  { label: "🔊 Звук", value: "sound" },
  { label: "🔔 Уведомление", value: "notification" },
  { label: "🔊🔔 Звук + уведомление", value: "both" },
  { label: "🔇 Без оповещения", value: "silent" },
];

interface PresetFormProps {
  title: string;
  form: Preset;
  busy: boolean;
  onChange: (preset: Preset) => void;
  onCancel: () => void;
  onSave: () => void;
}

function PresetForm({ title, form, busy, onChange, onCancel, onSave }: PresetFormProps) {
  const update = (patch: Partial<Preset>) => onChange({ ...form, ...patch });

  return (
    <div className="preset-edit-form">
      <h3>{title}</h3>

      <div className="form-group">
        <label>Название:</label>
        <input
          type="text"
          value={form.name}
          onChange={(event) => update({ name: event.target.value })}
          placeholder="Например: Ночной таймер"
        />
      </div>

      <div className="form-group">
        <label>Длительность (минуты):</label>
        <input
          type="number"
          value={Math.round(form.duration / 60)}
          onChange={(event) =>
            update({ duration: Math.max(0, Math.round(Number(event.target.value) * 60)) })
          }
          min="1"
          step="1"
        />
      </div>

      <div className="form-group">
        <label>Действие:</label>
        <select
          value={form.action}
          onChange={(event) => update({ action: event.target.value as PowerAction })}
        >
          {ACTIONS.map((action) => (
            <option key={action.value} value={action.value}>
              {action.label}
            </option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label>Предупреждение (сек):</label>
        <input
          type="number"
          value={form.warning_time}
          onChange={(event) =>
            update({ warning_time: Math.max(0, Math.round(Number(event.target.value))) })
          }
          min="0"
          step="1"
        />
      </div>

      <div className="form-group">
        <label>Оповещение:</label>
        <select
          value={form.notification_type}
          onChange={(event) =>
            update({ notification_type: event.target.value as NotificationType })
          }
        >
          {NOTIFICATION_TYPES.map((type) => (
            <option key={type.value} value={type.value}>
              {type.label}
            </option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label>Громкость: {form.sound_volume}%</label>
        <input
          type="range"
          min="0"
          max="100"
          value={form.sound_volume}
          onChange={(event) => update({ sound_volume: Number(event.target.value) })}
        />
      </div>

      <div className="form-actions">
        <button className="cancel-btn" onClick={onCancel} disabled={busy}>
          Отмена
        </button>
        <button className="save-btn" onClick={onSave} disabled={busy}>
          {busy ? "Сохранение..." : "Сохранить"}
        </button>
      </div>
    </div>
  );
}

function Presets({ onClose, onSelectPreset }: PresetsProps) {
  const { presets, setPresets } = useTimerStore();
  const [editing, setEditing] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<Preset | null>(null);
  const [isAdding, setIsAdding] = useState(false);
  const [busy, setBusy] = useState(false);

  const refresh = async () => {
    const list = await invoke<Preset[]>("load_presets");
    setPresets(list);
  };

  const handleAdd = () => {
    setIsAdding(true);
    setEditing(null);
    setEditForm({
      name: "",
      duration: 3600,
      action: "sleep",
      warning_time: 30,
      notification_type: "both",
      sound_volume: 80,
    });
  };

  const handleEdit = (preset: Preset) => {
    setIsAdding(false);
    setEditing(preset.name);
    setEditForm({ ...preset });
  };

  const handleSaveNew = async () => {
    if (!editForm) return;

    setBusy(true);
    try {
      await invoke("add_preset", { preset: editForm });
      await refresh();
      setIsAdding(false);
      setEditForm(null);
    } catch (error) {
      alert(`Не удалось добавить пресет: ${error}`);
    } finally {
      setBusy(false);
    }
  };

  const handleSaveEdit = async () => {
    if (!editForm || !editing) return;

    setBusy(true);
    try {
      await invoke("update_preset", { oldName: editing, preset: editForm });
      await refresh();
      setEditing(null);
      setEditForm(null);
    } catch (error) {
      alert(`Не удалось сохранить пресет: ${error}`);
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`Удалить пресет «${name}»?`)) return;

    setBusy(true);
    try {
      await invoke("delete_preset", { name });
      await refresh();
    } catch (error) {
      alert(`Не удалось удалить пресет: ${error}`);
    } finally {
      setBusy(false);
    }
  };

  const closeForm = () => {
    setIsAdding(false);
    setEditing(null);
    setEditForm(null);
  };

  return (
    <div className="presets-overlay" onClick={onClose}>
      <div className="presets-modal" onClick={(event) => event.stopPropagation()}>
        <div className="presets-header">
          <h2>📋 Управление пресетами</h2>
          <button className="close-btn" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="presets-content">
          {(isAdding || editing) && editForm ? (
            <PresetForm
              title={isAdding ? "Новый пресет" : `Редактирование: ${editing}`}
              form={editForm}
              busy={busy}
              onChange={setEditForm}
              onCancel={closeForm}
              onSave={isAdding ? handleSaveNew : handleSaveEdit}
            />
          ) : (
            <>
              <button className="add-preset-btn" onClick={handleAdd}>
                + Добавить пресет
              </button>

              <div className="presets-list">
                {presets.length === 0 && (
                  <p className="presets-empty">Пока нет сохранённых пресетов</p>
                )}

                {presets.map((preset) => (
                  <div key={preset.name} className="preset-card">
                    <div className="preset-info">
                      <h3>{preset.name}</h3>
                      <div className="preset-details">
                        <span>⏱ {formatDuration(preset.duration)}</span>
                        <span>
                          🔌{" "}
                          {ACTIONS.find((action) => action.value === preset.action)?.label ??
                            preset.action}
                        </span>
                        <span>⚠️ {preset.warning_time} сек</span>
                      </div>
                    </div>
                    <div className="preset-actions">
                      <button
                        className="use-btn"
                        onClick={() => {
                          onSelectPreset(preset);
                          onClose();
                        }}
                        disabled={busy}
                      >
                        Применить
                      </button>
                      <button
                        className="edit-btn"
                        onClick={() => handleEdit(preset)}
                        disabled={busy}
                        title="Редактировать"
                      >
                        ✏️
                      </button>
                      <button
                        className="delete-btn"
                        onClick={() => handleDelete(preset.name)}
                        disabled={busy}
                        title="Удалить"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

export default Presets;

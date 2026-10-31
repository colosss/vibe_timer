import { useEffect, useState } from "react";
import { TriangleAlert } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useTimerStore } from "./stores/timerStore";
import type { AppSettings, Preset } from "./types";
import Timer from "./components/Timer";

const ACTION_LABELS: Record<string, string> = {
  sleep: "Сон",
  hibernate: "Гибернация",
  shutdown: "Выключение",
  restart: "Перезагрузка",
  restart_and_sleep: "Перезагрузка + Сон",
};

function App() {
  const setRunning = useTimerStore((state) => state.setRunning);
  const setRemaining = useTimerStore((state) => state.setRemaining);
  const setPresets = useTimerStore((state) => state.setPresets);
  const setSettings = useTimerStore((state) => state.setSettings);
  const reset = useTimerStore((state) => state.reset);
  const config = useTimerStore((state) => state.config);
  const windowOpacity = useTimerStore((state) => state.settings.window_opacity);

  const [warning, setWarning] = useState<{ seconds: number } | null>(null);
  // Загружаем настройки и пресеты при старте.
  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        const [settings, presets] = await Promise.all([
          invoke<AppSettings>("load_settings"),
          invoke<Preset[]>("load_presets"),
        ]);
        if (!cancelled) {
          setSettings(settings);
          setPresets(presets);
        }
      } catch (error) {
        console.error("Ошибка загрузки настроек и пресетов:", error);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [setPresets, setSettings]);

  // Прозрачность окна реализована на уровне CSS: окно Tauri прозрачное.
  useEffect(() => {
    document.documentElement.style.setProperty(
      "--app-opacity",
      String(windowOpacity / 100)
    );
  }, [windowOpacity]);

  // Единственное место, где обрабатываются события таймера.
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let disposed = false;

    const register = (promise: Promise<UnlistenFn>) => {
      promise
        .then((unlisten) => {
          if (disposed) {
            unlisten();
          } else {
            unlisteners.push(unlisten);
          }
        })
        .catch(console.error);
    };

    register(
      listen<number>("timer_tick", (event) => {
        setRemaining(event.payload);
        setWarning((current) => current ? { seconds: event.payload } : null);
      })
    );

    register(
      listen<number>("timer_warning", (event) => {
        const currentConfig = useTimerStore.getState().config;
        if (!currentConfig) {
          return;
        }

        const type = currentConfig.notification_type;
        if (type === "notification" || type === "both") {
          setWarning({ seconds: event.payload });
        }

        invoke("trigger_notification", {
          notificationType: type,
          volume: currentConfig.sound_volume,
        }).catch(console.error);
      })
    );

    register(
      listen("timer_completed", () => {
        setWarning(null);
        setRunning(false);
        reset();
      })
    );

    register(
      listen<string>("timer_action_failed", (event) => {
        console.error("Ошибка выполнения действия:", event.payload);
        alert(`Не удалось выполнить действие: ${event.payload}`);
      })
    );

    register(
      listen("timer_cancelled", () => {
        setWarning(null);
        setRunning(false);
        reset();
      })
    );

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [reset, setRemaining, setRunning]);

  const handleCancelWarning = async () => {
    try {
      await invoke("cancel_timer");
      setWarning(null);
      reset();
    } catch (error) {
      console.error("Ошибка отмены таймера:", error);
      alert(`Не удалось отменить таймер: ${error}`);
    }
  };

  return (
    <div className="app">
      <Timer />

      {warning && (
        <div className="warning-overlay">
          <div className="warning-modal">
            <div className="warning-icon"><TriangleAlert size={48} aria-hidden="true" /></div>
            <h2>Предупреждение!</h2>
            <p>
              Система выполнит действие
              <br />
              <strong>
                {config ? ACTION_LABELS[config.action] ?? config.action : "—"}
              </strong>
              <br />
              через <strong>{warning.seconds}</strong> сек.
            </p>
            <button className="cancel-warning-btn" onClick={handleCancelWarning}>
              Отменить
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;

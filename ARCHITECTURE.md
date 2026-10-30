# 🏗️ Архитектура Vibe Timer

Документация внутреннего устройства приложения для разработчиков.

---

## 📊 Общая архитектура

```
┌─────────────────────────────────────────────────┐
│              React Frontend (UI)                 │
│  ┌───────────┐  ┌──────────┐  ┌──────────────┐ │
│  │  Timer    │  │ Presets  │  │   Settings   │ │
│  │ Component │  │ Component│  │  Component   │ │
│  └─────┬─────┘  └────┬─────┘  └──────┬───────┘ │
│        └─────────────┴────────────────┘          │
│                      │                           │
│            ┌─────────▼─────────┐                │
│            │  Zustand Store    │                │
│            │  (timerStore.ts)  │                │
│            └─────────┬─────────┘                │
└──────────────────────┼──────────────────────────┘
                       │ @tauri-apps/api (IPC + events)
                       │
┌──────────────────────▼──────────────────────────┐
│              Tauri 2 Backend (Rust)              │
│  ┌──────────┐ ┌──────────┐ ┌─────────────────┐ │
│  │  Timer   │ │  Power   │ │  Notifications  │ │
│  │  Logic   │ │ Commands │ │ (Sound/Attention)│ │
│  └────┬─────┘ └────┬─────┘ └────────┬────────┘ │
│       │            │                 │          │
│  ┌────▼────────────▼─────────────────▼────────┐ │
│  │         Event System (Tauri)               │ │
│  │  timer_tick / timer_warning / ...          │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌──────────┐           ┌──────────────────┐   │
│  │ Presets  │           │    Settings      │   │
│  │   JSON   │           │      JSON        │   │
│  └──────────┘           └──────────────────┘   │
└──────────────────────────────────────────────────┘
                       │
                       ▼
              ┌────────────────┐
              │  OS Commands   │
              │ systemctl /    │
              │ shutdown.exe   │
              └────────────────┘
```

---

## 🔀 Поток данных

### 1. Запуск таймера

```
[Пользователь нажимает «Запустить таймер»]
         │
         ▼
    Timer.tsx: handleStart()
         │
         ├─ Валидация длительности
         ├─ setConfig / setTotal / setRemaining в store
         │
         ▼
    invoke("start_timer", { config })
         │
         ▼
    timer.rs: start_timer()
         │
         ├─ Проверка duration > 0
         ├─ Запись TimerState (is_running = true)
         ├─ Сброс cancel_flag
         ├─ Инкремент generation (старые циклы завершаются)
         │
         ▼
    tokio::spawn(async move { ... })
         │
         ├─ Каждые 100 мс:
         │   ├─ Проверить cancel_flag / generation
         │   ├─ Если пауза — не накапливать время
         │   ├─ emit("timer_tick", remaining_secs)
         │   ├─ Один раз emit("timer_warning", remaining_secs)
         │   └─ При remaining == 0 → emit("timer_completed", config)
```

Пауза реализована через накопление только «активного» времени: пока
`is_paused == true`, прошедшие интервалы не прибавляются к `elapsed_active`.

### 2. Предупреждение

```
Backend: remaining <= warning_time (один раз)
         │
         ▼
    emit("timer_warning", remaining_secs)
         │
         ▼
    App.tsx: listen("timer_warning")
         │
         ├─ Notification | Both → показать модальное окно с обратным отсчётом
         ├─ Прочитать config из store
         └─ invoke("trigger_notification", { notificationType, volume })
                  │
                  ▼
             notifications.rs
             ├─ Sound        → rodio (синусоида 800 Гц, 300 мс)
             ├─ Notification → показать и сфокусировать окно
             ├─ Both         → показать окно + звук
             └─ Silent       → ничего
```

### 3. Завершение таймера

```
Backend: remaining == 0
         │
         ▼
    state.is_running = false
    emit("timer_completed")            ← UI сбрасывает состояние
         │
         ▼
    Rust сам выполняет действие питания
    (не зависит от активности WebView)
         │
         ├─ Linux:   systemctl suspend | hibernate | poweroff | reboot
         ├─ Windows: rundll32 powrprof.dll | shutdown /h | /s | /r
         └─ restart_and_sleep:
              Linux   — временный systemd user unit, который после
                        следующего входа усыпит систему и удалится
              Windows — RunOnce-запись в HKCU, выполняемая после входа
         │
         ├─ Успех → ничего
         └─ Ошибка → emit("timer_action_failed", error)
                     Frontend показывает alert
```

Важно: действие питания выполняет Rust-бэкенд, а не JavaScript.
Это гарантирует срабатывание таймера, даже если окно свёрнуто в трей.
Событие `timer_completed` обрабатывается только в `App.tsx` и служит
лишь для сброса интерфейса — двойного выполнения команды нет.

### 4. Отмена

```
[«Отменить» на главном экране или в окне предупреждения]
         │
         ▼
    invoke("cancel_timer")
         │
         ▼
    timer.rs: cancel_timer() → cancel_flag = true
         │
         ▼
    Async loop:
         ├─ emit("timer_cancelled")
         └─ state.is_running = false
         │
         ▼
    App.tsx: reset() store, скрыть предупреждение
```

### 5. Настройки и пресеты

- На старте `App.tsx` вызывает `load_settings` и `load_presets`.
- `save_settings` сохраняет JSON, применяет `always_on_top`
  и обновляет режим закрытия окна.
- `add_preset` / `update_preset` / `delete_preset` пишут файл на диск,
  после чего фронтенд перечитывает список через `load_presets`.
- Прозрачность окна хранится в настройках и применяется CSS-переменной
  `--app-opacity` (окно Tauri создаётся с `transparent: true`).

---

## 🗂️ Модули Rust Backend

### `main.rs` — точка входа и системный трей

- Инициализация Tauri 2.
- Регистрация `TimerManager` и `CloseGuard`.
- System tray: показать / скрыть / выход, клик по иконке.
- Загрузка настроек при старте (`always_on_top`, `start_minimized`,
  `minimize_to_tray`).
- Закрытие окна: если `minimize_to_tray` включён — окно скрывается,
  иначе приложение завершается.
- Регистрация всех команд.

### `timer.rs` — логика таймера

- `TimerManager` хранит состояние под `Mutex`.
- `start_timer` запускает Tokio-задачу с шагом 100 мс.
- `pause_timer` переключает паузу и возвращает новое состояние.
- `cancel_timer` выставляет флаг отмены.
- `get_timer_state` возвращает снимок состояния.
- `generation` защищает от «осиротевших» циклов при перезапуске.

### `power.rs` — команды питания

- Linux: `systemctl suspend | hibernate | poweroff | reboot`.
- Windows: `rundll32.exe powrprof.dll,SetSuspendState 0,1,0`,
  `shutdown /h | /s | /r`.
- `restart_and_sleep` дополнительно создаёт отложенное задание сна
  (systemd user unit на Linux, RunOnce в реестре на Windows).
  При неудачной перезагрузке задание удаляется.
- `test_power_action` возвращает команду без выполнения.

### `notifications.rs` — оповещения

- `NotificationType`: `sound | notification | both | silent`.
- Звук генерируется через rodio в `spawn_blocking`, чтобы не блокировать
  IPC-поток.
- Уведомление показывает и фокусирует окно (в том числе из трея).
- Старое значение `visual` читается как `notification` благодаря
  `#[serde(alias = "visual")]`.

### `presets.rs` — пресеты

- JSON-файл `presets.json` в `app_data_dir`.
- При первом запуске создаются 4 пресета по умолчанию.
- Имя пресета уникально и используется как идентификатор.
- Валидация: непустое имя, длительность > 0, громкость ≤ 100.

### `settings.rs` — настройки

- JSON-файл `settings.json` в `app_data_dir`.
- Поля: `window_opacity`, `default_sound_volume`,
  `default_notification`, `always_on_top`, `minimize_to_tray`,
  `start_minimized`.
- `#[serde(default)]` обеспечивает совместимость с частично
  заполненными файлами.
- `save_settings` применяет `always_on_top` и обновляет `CloseGuard`.

---

## 🖥️ Frontend

| Файл | Ответственность |
|------|-----------------|
| `src/App.tsx` | Загрузка настроек/пресетов, подписка на события таймера, окно предупреждения, прозрачность |
| `src/components/Timer.tsx` | Выбор параметров, запуск/пауза/отмена, отображение времени и прогресса |
| `src/components/Presets.tsx` | CRUD пресетов через IPC |
| `src/components/Settings.tsx` | Редактирование и сохранение настроек |
| `src/stores/timerStore.ts` | Zustand-хранилище (состояние таймера, пресеты, настройки) |
| `src/types.ts` | Типы, синхронизированные с Rust-структурами |
| `src/utils/format.ts` | Форматирование времени и длительности |

---

## 📡 События (Backend → Frontend)

| Событие | Payload | Когда |
|---------|---------|-------|
| `timer_tick` | `number` (секунды) | Каждые ~100 мс, кроме паузы |
| `timer_warning` | `number` (секунды) | Один раз при `remaining <= warning_time` |
| `timer_completed` | — | При `remaining == 0` |
| `timer_action_failed` | `string` | Если команда питания завершилась ошибкой |
| `timer_cancelled` | — | После `cancel_timer` |

## 🔌 Команды (Frontend → Backend)

| Команда | Аргументы | Результат |
|---------|-----------|-----------|
| `start_timer` | `config` | `()` |
| `pause_timer` | — | `boolean` (пауза активна) |
| `cancel_timer` | — | `()` |
| `get_timer_state` | — | `TimerState` |
| `execute_power_action` | `action` | `string` |
| `test_power_action` | `action` | `string` |
| `trigger_notification` | `notificationType`, `volume` | `()` |
| `load_presets` | — | `Preset[]` |
| `add_preset` | `preset` | `()` |
| `update_preset` | `oldName`, `preset` | `()` |
| `delete_preset` | `name` | `()` |
| `load_settings` | — | `AppSettings` |
| `save_settings` | `settings` | `()` |

---

## 🛡️ Безопасность

- В `tauri.conf.json` включена capability `core:default` для окна `main`.
- Shell-команды не доступны фронтенду; все системные действия выполняются
  только Rust-кодом через `std::process::Command` с фиксированным argv.
- CSP не задан (`null`) — при необходимости можно включить.
- Системные команды питания на Linux могут требовать polkit-разрешений,
  на Windows — прав администратора (см. README).

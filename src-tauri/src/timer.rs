use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Emitter, State, WebviewWindow};
use tokio::time::{sleep, Instant};

use crate::notifications::NotificationType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerAction {
    Sleep,
    Hibernate,
    Shutdown,
    Restart,
    #[serde(rename = "restart_and_sleep")]
    RestartAndSleep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub duration: u64,       // Длительность в секундах
    pub action: PowerAction, // Действие при завершении
    pub warning_time: u64,   // Время предупреждения в секундах
    pub notification_type: NotificationType,
    pub sound_volume: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimerState {
    pub is_running: bool,
    pub is_paused: bool,
    pub remaining: u64,
    pub total: u64,
    pub config: Option<TimerConfig>,
}

pub struct TimerManager {
    state: Arc<Mutex<TimerState>>,
    cancel_flag: Arc<Mutex<bool>>,
    /// Поколение таймера: позволяет старому циклу понять, что запущен новый.
    generation: Arc<AtomicU64>,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(TimerState {
                is_running: false,
                is_paused: false,
                remaining: 0,
                total: 0,
                config: None,
            })),
            cancel_flag: Arc::new(Mutex::new(false)),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[tauri::command]
pub async fn start_timer(
    config: TimerConfig,
    timer_manager: State<'_, TimerManager>,
    window: WebviewWindow,
) -> Result<(), String> {
    if config.duration == 0 {
        return Err("Длительность таймера должна быть больше нуля".to_string());
    }

    {
        let mut state = timer_manager.state.lock().unwrap();
        state.is_running = true;
        state.is_paused = false;
        state.remaining = config.duration;
        state.total = config.duration;
        state.config = Some(config.clone());
    }
    {
        let mut cancel = timer_manager.cancel_flag.lock().unwrap();
        *cancel = false;
    }

    let state = timer_manager.state.clone();
    let cancel_flag = timer_manager.cancel_flag.clone();
    let generation = timer_manager.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let generation_handle = timer_manager.generation.clone();

    tokio::spawn(async move {
        let duration = Duration::from_secs(config.duration);
        let warning_time = Duration::from_secs(config.warning_time).min(duration);

        let mut last_tick = Instant::now();
        let mut elapsed_active = Duration::ZERO;
        let mut warning_sent = false;

        loop {
            if *cancel_flag.lock().unwrap() {
                let mut state = state.lock().unwrap();
                state.is_running = false;
                state.is_paused = false;
                let _ = window.emit("timer_cancelled", ());
                return;
            }

            // Если таймер перезапустили — этот цикл больше не нужен.
            if generation_handle.load(Ordering::SeqCst) != generation {
                return;
            }

            let now = Instant::now();
            let delta = now.saturating_duration_since(last_tick);
            last_tick = now;

            let is_paused = state.lock().unwrap().is_paused;
            if is_paused {
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            elapsed_active += delta;
            let remaining = duration.saturating_sub(elapsed_active);
            let remaining_secs = remaining.as_secs();

            {
                let mut state = state.lock().unwrap();
                state.remaining = remaining_secs;
            }
            let _ = window.emit("timer_tick", remaining_secs);

            if !warning_sent && remaining <= warning_time {
                warning_sent = true;
                let _ = window.emit("timer_warning", remaining_secs);
            }

            if remaining.is_zero() {
                {
                    let mut state = state.lock().unwrap();
                    state.is_running = false;
                    state.is_paused = false;
                    state.remaining = 0;
                }
                let _ = window.emit("timer_completed", ());

                // Действие питания выполняем в Rust, не полагаясь на активность WebView:
                // окно может быть свёрнуто в трей.
                let action = config.action.clone();
                let action_window = window.clone();
                tokio::spawn(async move {
                    if let Err(error) = crate::power::perform_power_action(action).await {
                        let _ = action_window.emit("timer_action_failed", error);
                    }
                });
                return;
            }

            sleep(Duration::from_millis(100)).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn pause_timer(timer_manager: State<'_, TimerManager>) -> Result<bool, String> {
    let mut state = timer_manager.state.lock().unwrap();
    if !state.is_running {
        return Ok(false);
    }
    state.is_paused = !state.is_paused;
    Ok(state.is_paused)
}

#[tauri::command]
pub fn cancel_timer(timer_manager: State<'_, TimerManager>) -> Result<(), String> {
    let is_running = timer_manager.state.lock().unwrap().is_running;
    if !is_running {
        return Ok(());
    }
    let mut cancel = timer_manager.cancel_flag.lock().unwrap();
    *cancel = true;
    Ok(())
}

#[tauri::command]
pub fn get_timer_state(timer_manager: State<'_, TimerManager>) -> Result<TimerState, String> {
    let state = timer_manager.state.lock().unwrap();
    Ok(state.clone())
}

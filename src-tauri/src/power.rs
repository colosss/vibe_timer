use crate::timer::PowerAction;
use std::process::Command;

#[cfg(target_os = "linux")]
fn power_command(action: &PowerAction) -> Vec<String> {
    let arg = match action {
        PowerAction::Sleep => "suspend",
        PowerAction::Hibernate => "hibernate",
        PowerAction::Shutdown => "poweroff",
        PowerAction::Restart | PowerAction::RestartAndSleep => "reboot",
    };
    vec!["systemctl".to_string(), arg.to_string()]
}

#[cfg(target_os = "windows")]
fn power_command(action: &PowerAction) -> Vec<String> {
    match action {
        PowerAction::Sleep => vec![
            "rundll32.exe".to_string(),
            "powrprof.dll,SetSuspendState".to_string(),
            "0,1,0".to_string(),
        ],
        PowerAction::Hibernate => vec!["shutdown".to_string(), "/h".to_string()],
        PowerAction::Shutdown => vec![
            "shutdown".to_string(),
            "/s".to_string(),
            "/t".to_string(),
            "0".to_string(),
        ],
        PowerAction::Restart | PowerAction::RestartAndSleep => vec![
            "shutdown".to_string(),
            "/r".to_string(),
            "/t".to_string(),
            "0".to_string(),
        ],
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn power_command(_action: &PowerAction) -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "linux")]
const SUSPEND_UNIT_NAME: &str = "vibe-timer-suspend.service";

#[cfg(target_os = "linux")]
fn suspend_unit_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "Не удалось определить домашнюю директорию")?;
    Ok(std::path::PathBuf::from(home)
        .join(".config/systemd/user")
        .join(SUSPEND_UNIT_NAME))
}

/// После перезагрузки пользовательский systemd-юнит один раз усыпит систему.
#[cfg(target_os = "linux")]
fn schedule_suspend_after_reboot() -> Result<(), String> {
    let path = suspend_unit_path()?;
    let dir = path.parent().ok_or("Некорректный путь юнита")?;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Ошибка создания директории systemd: {}", e))?;

    let unit = "[Unit]\n\
Description=Vibe Timer: suspend after reboot\n\
After=graphical-session.target\n\
\n\
[Service]\n\
Type=oneshot\n\
ExecStart=/bin/sh -c 'systemctl --user disable vibe-timer-suspend.service; systemctl suspend'\n\
\n\
[Install]\n\
WantedBy=default.target\n";

    std::fs::write(&path, unit).map_err(|e| format!("Ошибка записи systemd-юнита: {}", e))?;

    let output = Command::new("systemctl")
        .args(["--user", "enable", SUSPEND_UNIT_NAME])
        .output()
        .map_err(|e| format!("Ошибка вызова systemctl: {}", e))?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&path);
        return Err(format!(
            "Не удалось включить юнит сна: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn cancel_scheduled_suspend() {
    let _ = Command::new("systemctl")
        .args(["--user", "disable", SUSPEND_UNIT_NAME])
        .output();
    if let Ok(path) = suspend_unit_path() {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(target_os = "windows")]
const RUN_ONCE_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce";
#[cfg(target_os = "windows")]
const RUN_ONCE_VALUE: &str = "VibeTimerSuspendAfterReboot";

/// После перезагрузки Windows выполнит RunOnce-запись и усыпит систему.
#[cfg(target_os = "windows")]
fn schedule_suspend_after_reboot() -> Result<(), String> {
    let output = Command::new("reg")
        .args([
            "add",
            RUN_ONCE_KEY,
            "/v",
            RUN_ONCE_VALUE,
            "/t",
            "REG_SZ",
            "/d",
            "rundll32.exe powrprof.dll,SetSuspendState 0,1,0",
            "/f",
        ])
        .output()
        .map_err(|e| format!("Ошибка записи в реестр: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Не удалось записать задачу сна: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn cancel_scheduled_suspend() {
    let _ = Command::new("reg")
        .args(["delete", RUN_ONCE_KEY, "/v", RUN_ONCE_VALUE, "/f"])
        .output();
}

fn run_command(parts: &[String]) -> Result<String, String> {
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| "Не удалось определить команду".to_string())?;

    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Ошибка выполнения команды: {}", e))?;

    if output.status.success() {
        Ok(format!("Выполнено: {}", parts.join(" ")))
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Команда завершилась с ошибкой: {}", error.trim()))
    }
}

pub(crate) async fn perform_power_action(action: PowerAction) -> Result<String, String> {
    let schedule_suspend = matches!(action, PowerAction::RestartAndSleep);
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let _ = schedule_suspend;

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if schedule_suspend {
        schedule_suspend_after_reboot()?;
    }

    let command_parts = power_command(&action);
    let command_for_log = command_parts.join(" ");

    let result = tokio::task::spawn_blocking(move || run_command(&command_parts))
        .await
        .map_err(|e| format!("Ошибка выполнения команды: {}", e))?;

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if result.is_err() && schedule_suspend {
        // Перезагрузка не удалась — не оставляем «бомбу замедленного действия».
        cancel_scheduled_suspend();
    }

    result.map(|_| format!("Действие выполнено: {}", command_for_log))
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
#[tauri::command]
pub async fn execute_power_action(action: PowerAction) -> Result<String, String> {
    perform_power_action(action).await
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[tauri::command]
pub async fn execute_power_action(_action: PowerAction) -> Result<String, String> {
    Err("Неподдерживаемая операционная система".to_string())
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
#[tauri::command]
pub fn test_power_action(action: PowerAction) -> Result<String, String> {
    Ok(format!(
        "Команда для {:?}: {}",
        action,
        power_command(&action).join(" ")
    ))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[tauri::command]
pub fn test_power_action(_action: PowerAction) -> Result<String, String> {
    Err("Неподдерживаемая операционная система".to_string())
}

export type PowerAction =
  | 'sleep'
  | 'hibernate'
  | 'shutdown'
  | 'restart'
  | 'restart_and_sleep';

export type NotificationType = 'sound' | 'notification' | 'both' | 'silent';

export interface TimerConfig {
  duration: number;
  action: PowerAction;
  warning_time: number;
  notification_type: NotificationType;
  sound_volume: number;
}

export interface TimerState {
  is_running: boolean;
  is_paused: boolean;
  remaining: number;
  total: number;
  config: TimerConfig | null;
}

export interface Preset {
  name: string;
  duration: number;
  action: PowerAction;
  warning_time: number;
  notification_type: NotificationType;
  sound_volume: number;
}

export interface AppSettings {
  window_opacity: number;
  default_sound_volume: number;
  default_notification: NotificationType;
  always_on_top: boolean;
  minimize_to_tray: boolean;
  start_minimized: boolean;
}

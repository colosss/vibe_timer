import { create } from 'zustand';
import type { AppSettings, Preset, TimerConfig } from '../types';

export const defaultSettings: AppSettings = {
  window_opacity: 95,
  default_sound_volume: 80,
  default_notification: 'both',
  always_on_top: false,
  minimize_to_tray: true,
  start_minimized: false,
};

interface TimerStore {
  isRunning: boolean;
  isPaused: boolean;
  remaining: number;
  total: number;
  config: TimerConfig | null;
  presets: Preset[];
  settings: AppSettings;

  setRunning: (running: boolean) => void;
  setPaused: (paused: boolean) => void;
  setRemaining: (remaining: number) => void;
  setTotal: (total: number) => void;
  setConfig: (config: TimerConfig | null) => void;
  setPresets: (presets: Preset[]) => void;
  setSettings: (settings: AppSettings) => void;
  reset: () => void;
}

const initialTimerState = {
  isRunning: false,
  isPaused: false,
  remaining: 0,
  total: 0,
  config: null as TimerConfig | null,
};

export const useTimerStore = create<TimerStore>((set) => ({
  ...initialTimerState,
  presets: [],
  settings: defaultSettings,

  setRunning: (isRunning) => set({ isRunning }),
  setPaused: (isPaused) => set({ isPaused }),
  setRemaining: (remaining) => set({ remaining }),
  setTotal: (total) => set({ total }),
  setConfig: (config) => set({ config }),
  setPresets: (presets) => set({ presets }),
  setSettings: (settings) => set({ settings }),

  // Сбрасываем только состояние таймера, не трогая настройки и пресеты.
  reset: () => set({ ...initialTimerState }),
}));

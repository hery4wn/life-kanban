export interface Card {
  id: string;
  title: string;
  description: string;
  tag: string;
  createdAt: string;
}
export interface Column {
  id: string;
  name: string;
  cards: Card[];
}
export interface Settings {
  opacity: number;
  blur: number;
  theme: "light" | "dark" | "system";
  shortcut: string;
}
export interface QuickTask {
  id: string;
  title: string;
  done: boolean;
}
export interface TimerState {
  durationSeconds: number;
  remainingSeconds: number;
  endsAt: number | null;
}
export interface Board {
  version: 1;
  quickTasks: QuickTask[];
  timer: TimerState;
  columns: Column[];
  settings: Settings;
}
export interface NativeStatus {
  editing: boolean;
  shortcut: string;
  autostart: boolean;
  warning: string | null;
}
export interface LoadResult {
  board: Board;
  warning: string | null;
}

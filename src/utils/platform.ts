import { invoke, isTauri } from "@tauri-apps/api/core";
import { emptyBoard, validateBoard } from "./board";
import type { Board, LoadResult } from "../types/board";
export const desktop = isTauri();
const key = "life-kanban-browser-preview-v1";
export async function loadBoard(): Promise<LoadResult> {
  if (desktop) return invoke("load_board");
  const raw = localStorage.getItem(key);
  return {
    board: raw ? validateBoard(JSON.parse(raw)) : emptyBoard(),
    warning: null,
  };
}
export async function saveBoard(board: Board): Promise<void> {
  validateBoard(board);
  if (desktop) await invoke("save_board", { board });
  else localStorage.setItem(key, JSON.stringify(board));
}

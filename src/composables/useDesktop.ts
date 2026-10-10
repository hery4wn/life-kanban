import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { desktop } from "../utils/platform";
import type { NativeStatus } from "../types/board";
export function useDesktop() {
  const editing = ref(false),
    autostart = ref(false),
    error = ref(""),
    shortcut = ref("Control+Shift+K");
  const cleanups: UnlistenFn[] = [];
  const apply = (s: NativeStatus) => {
    editing.value = s.editing;
    autostart.value = s.autostart;
    shortcut.value = s.shortcut;
    if (s.warning) error.value = s.warning;
  };
  async function initialize(
    openSettings: () => void,
    requestQuit: () => Promise<void>,
  ) {
    if (!desktop) return;
    cleanups.push(
      await listen<NativeStatus>("native-status", (e) => apply(e.payload)),
    );
    cleanups.push(await listen("open-settings", openSettings));
    cleanups.push(
      await listen<string>("native-error", (e) => {
        error.value = e.payload;
      }),
    );
    cleanups.push(
      await listen("request-quit", () => {
        void requestQuit();
      }),
    );
    apply(await invoke<NativeStatus>("frontend_ready"));
  }
  async function setEditing(value: boolean) {
    try {
      if (desktop)
        apply(await invoke<NativeStatus>("set_editing", { editing: value }));
      else editing.value = value;
    } catch (e) {
      error.value = String(e);
    }
  }
  async function setAutostart(enabled: boolean) {
    if (desktop) {
      await invoke("set_autostart", { enabled });
      autostart.value = enabled;
    }
  }
  async function setShortcut(value: string) {
    if (desktop) await invoke("set_shortcut", { shortcut: value });
    shortcut.value = value;
    error.value = "";
  }
  return {
    editing,
    autostart,
    error,
    shortcut,
    initialize,
    setEditing,
    setAutostart,
    setShortcut,
    dispose: () => cleanups.forEach((fn) => fn()),
  };
}

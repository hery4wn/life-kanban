<script setup lang="ts">
import { reactive, ref } from "vue";
import type { Settings } from "../types/board";
import ModalShell from "./ModalShell.vue";
import { desktop } from "../utils/platform";
const props = defineProps<{
  settings: Settings;
  autostart: boolean;
  apply: (settings: Settings, autostart: boolean) => Promise<void>;
}>();
const emit = defineEmits<{ close: [] }>();
const draft = reactive({ ...props.settings, autostart: props.autostart });
const busy = ref(false),
  error = ref("");
function capture(event: KeyboardEvent) {
  if (event.key === "Tab") return;
  event.preventDefault();
  if (["Control", "Shift", "Alt", "Meta"].includes(event.key)) return;
  if (!event.ctrlKey && !event.altKey) {
    error.value = "Include Ctrl or Alt with a letter or number.";
    return;
  }
  if (!/^[a-z0-9]$/i.test(event.key)) {
    error.value = "Choose a letter or number with Ctrl or Alt.";
    return;
  }
  draft.shortcut = [
    event.ctrlKey && "Control",
    event.altKey && "Alt",
    event.shiftKey && "Shift",
    event.key.toUpperCase(),
  ]
    .filter(Boolean)
    .join("+");
  error.value = "";
}
async function save() {
  busy.value = true;
  error.value = "";
  try {
    await props.apply(
      {
        opacity: draft.opacity,
        blur: draft.blur,
        theme: draft.theme,
        shortcut: draft.shortcut,
      },
      draft.autostart,
    );
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <ModalShell title="Make yourself at home" @close="!busy && emit('close')">
    <form @submit.prevent="save">
      <p class="settings-intro">A few quiet adjustments to your workspace.</p>
      <label class="toggle-label"
        ><span
          >Launch with Windows<small>Start quietly in View Mode.</small></span
        ><input v-model="draft.autostart" type="checkbox" :disabled="!desktop"
      /></label>
      <p v-if="!desktop" class="help">
        Windows settings are available in the desktop app.
      </p>
      <label for="board-opacity"
        >Board opacity <output for="board-opacity">{{ draft.opacity }}%</output
        ><input
          id="board-opacity"
          v-model.number="draft.opacity"
          aria-label="Board opacity"
          type="range"
          min="35"
          max="100"
      /></label>
      <label for="surface-blur"
        >Surface blur <output for="surface-blur">{{ draft.blur }} px</output
        ><input
          id="surface-blur"
          v-model.number="draft.blur"
          aria-label="Surface blur"
          type="range"
          min="0"
          max="24"
      /></label>
      <p class="help">
        Softens layers inside the board. Your desktop wallpaper stays unchanged.
      </p>
      <label
        >Appearance<select v-model="draft.theme">
          <option value="system">Follow Windows</option>
          <option value="light">Light · soft paper</option>
          <option value="dark">Dark · evening ink</option>
        </select></label
      >
      <label
        >Global shortcut<input
          :value="
            draft.shortcut.replaceAll('Control', 'Ctrl').replaceAll('+', ' + ')
          "
          readonly
          :disabled="!desktop"
          aria-label="Global shortcut"
          @keydown="capture"
      /></label>
      <p class="help">
        Focus the field and press your new shortcut. Used to enter and leave
        Edit Mode.
      </p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <footer class="modal-actions">
        <button
          type="button"
          class="text-button"
          :disabled="busy"
          @click="emit('close')"
        >
          Cancel</button
        ><button class="primary" :disabled="busy">
          {{ busy ? "Saving…" : "Save settings" }}
        </button>
      </footer>
    </form>
  </ModalShell>
</template>

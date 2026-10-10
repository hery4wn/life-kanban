<script setup lang="ts">
import { reactive } from "vue";
import type { Card, Column } from "../types/board";
import ModalShell from "./ModalShell.vue";
const props = defineProps<{
  card: Card;
  columns: Column[];
  columnId: string;
}>();
const emit = defineEmits<{
  close: [];
  save: [fields: Pick<Card, "title" | "description" | "tag">, columnId: string];
  remove: [];
}>();
const draft = reactive({ title: props.card.title, columnId: props.columnId });
function save() {
  if (draft.title.trim())
    emit(
      "save",
      {
        title: draft.title,
        description: props.card.description,
        tag: props.card.tag,
      },
      draft.columnId,
    );
}
</script>
<template>
  <ModalShell title="Edit project" @close="emit('close')"
    ><form @submit.prevent="save">
      <label
        >Title<input
          v-model="draft.title"
          autofocus
          required
          maxlength="200" /></label
      ><label
        >Column<select v-model="draft.columnId" aria-label="Column">
          <option v-for="col in columns" :key="col.id" :value="col.id">
            {{ col.name }}
          </option>
        </select></label
      >
      <footer class="modal-actions">
        <button
          type="button"
          class="text-button danger"
          @click="emit('remove')"
        >
          Delete card</button
        ><button type="submit" class="primary" :disabled="!draft.title.trim()">
          Save card ↵
        </button>
      </footer>
    </form></ModalShell
  >
</template>

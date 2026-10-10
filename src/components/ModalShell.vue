<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
defineProps<{ title: string }>();
const emit = defineEmits<{ close: [] }>();
const dialog = ref<HTMLDialogElement>();
let previous: HTMLElement | null = null;
onMounted(() => {
  previous = document.activeElement as HTMLElement;
  dialog.value?.showModal();
});
onBeforeUnmount(() => {
  dialog.value?.close();
  previous?.focus();
});
</script>
<template>
  <dialog
    ref="dialog"
    class="modal"
    aria-labelledby="modal-title"
    @cancel.prevent="emit('close')"
    @click="(event) => event.target === dialog && emit('close')"
  >
    <div class="modal-inner">
      <header class="modal-header">
        <h2 id="modal-title">{{ title }}</h2>
        <button
          class="icon-button"
          aria-label="Close dialog"
          @click="emit('close')"
        >
          ×
        </button>
      </header>
      <slot />
    </div>
  </dialog>
</template>

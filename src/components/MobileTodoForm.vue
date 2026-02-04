<template>
  <div class="mobile-form">
    <input
      v-model="content"
      type="text"
      :placeholder="$t('form.placeholder')"
      class="mobile-input"
      maxlength="500"
      @keyup.enter="handleSubmit"
    />
    <button type="button" class="mobile-btn" :disabled="submitting" @click="handleSubmit">
      {{ $t('form.add') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{ (e: "submit", content: string): void }>();
const content = ref("");
const submitting = ref(false);

function handleSubmit() {
  const text = content.value.trim();
  if (!text) return;
  submitting.value = true;
  emit("submit", text);
  content.value = "";
  submitting.value = false;
}
</script>

<style scoped>
.mobile-form {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}
.mobile-input {
  flex: 1;
  padding: 14px 18px;
  font-size: 16px;
  border: 1px solid rgba(139, 115, 85, 0.25);
  border-radius: 12px;
  outline: none;
  background: #fffefc;
  color: #5c4a3a;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.mobile-input::placeholder {
  color: #9a8576;
}
.mobile-input:focus {
  border-color: #d4a574;
  box-shadow: 0 0 0 3px rgba(212, 165, 116, 0.15);
}
.mobile-btn {
  padding: 14px 22px;
  font-size: 16px;
  font-weight: 500;
  color: #fff;
  background: linear-gradient(180deg, #d4a574 0%, #c4956a 100%);
  border: none;
  border-radius: 12px;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(212, 165, 116, 0.35);
  transition: opacity 0.2s, box-shadow 0.2s;
}
.mobile-btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(212, 165, 116, 0.4);
}
.mobile-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>

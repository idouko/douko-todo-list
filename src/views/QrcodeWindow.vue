<template>
  <div class="qrcode-window-page">
    <QrcodeDisplay :url="qrcodeUrl" />
    <div class="qrcode-window-actions">
      <button type="button" class="btn-close-window" @click="closeWindow">{{ $t('qrcode.close') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import QrcodeDisplay from "@/components/QrcodeDisplay.vue";
import { getMobileUrl } from "@/utils/request";
import { getContrastTextColor } from "@/utils/theme";

const STORE_KEY = "app-settings.json";
const KEYS = { themeColor: "themeColor", backgroundColor: "backgroundColor" } as const;

const qrcodeUrl = ref("");
const themeColor = ref("rgba(64, 249, 255, 0.61)");
let pollTimer: ReturnType<typeof setInterval> | null = null;
let unlistenWindow: (() => void) | null = null;
let unlistenGlobal: (() => void) | null = null;

function applySettings(payload: { themeColor?: string; backgroundColor?: string }) {
  if (payload.themeColor != null) {
    themeColor.value = payload.themeColor;
    document.documentElement.style.setProperty("--el-color-primary", payload.themeColor);
    document.documentElement.style.setProperty("--app-border-color", payload.themeColor);
  }
  if (payload.backgroundColor != null) {
    document.documentElement.style.setProperty("--app-text-color", getContrastTextColor(payload.backgroundColor));
  }
}

function closeWindow() {
  if (typeof window !== "undefined" && (window as Window & { __TAURI__?: unknown }).__TAURI__) {
    import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      getCurrentWindow().close();
    });
  }
}

onMounted(async () => {
  getMobileUrl().then((url) => {
    if (url) qrcodeUrl.value = url;
  });
  pollTimer = setInterval(() => {
    getMobileUrl().then((url) => {
      if (url) qrcodeUrl.value = url;
    });
  }, 2000);

  if ((window as Window & { __TAURI__?: unknown }).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(STORE_KEY, { autoSave: false });
      const t = await store.get(KEYS.themeColor);
      const b = await store.get(KEYS.backgroundColor);
      if (typeof t === "string") applySettings({ themeColor: t });
      if (typeof b === "string") applySettings({ backgroundColor: b });
      document.documentElement.style.setProperty("--app-text-color", getContrastTextColor("#ffffff"));
    } catch {
      document.documentElement.style.setProperty("--el-color-primary", themeColor.value);
      document.documentElement.style.setProperty("--app-border-color", themeColor.value);
      document.documentElement.style.setProperty("--app-text-color", getContrastTextColor("#ffffff"));
    }
    const applyPayload = (e: { payload?: unknown }) => {
      const p = e?.payload;
      if (p && typeof p === "object") applySettings(p as { themeColor?: string; backgroundColor?: string });
    };
    // 使用当前窗口的 listen 接收 emitTo("qrcode", ...) 发来的设置事件
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    unlistenWindow = await getCurrentWindow().listen("settings-changed", applyPayload);
    // 同时监听全局 broadcast（设置窗口会 emit 一次）
    const { listen } = await import("@tauri-apps/api/event");
    unlistenGlobal = await listen("settings-changed", applyPayload);
  }
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
  unlistenWindow?.();
  unlistenGlobal?.();
});
</script>

<style scoped>
.qrcode-window-page {
  min-height: 100vh;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: #ffffff;
  color: var(--app-text-color, #1a1a1a);
}
.qrcode-window-actions {
  margin-top: 20px;
}
/* 按钮文字色与主窗体 primary 按钮一致：白字 */
.btn-close-window {
  padding: 8px 20px;
  font-size: 14px;
  color: #fff;
  background: var(--el-color-primary, #409eff);
  border: 1px solid var(--app-border-color, transparent);
  border-radius: 4px;
  cursor: pointer;
}
.btn-close-window:hover {
  opacity: 0.9;
}
</style>

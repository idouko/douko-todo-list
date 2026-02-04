<template>
  <el-config-provider :locale="elementLocale">
    <router-view />
  </el-config-provider>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import { setLocale, getElementPlusLocale } from "./i18n";

const elementLocale = ref(zhCn);

async function applyLocale(locale: string) {
  setLocale(locale);
  elementLocale.value = await getElementPlusLocale(locale);
}

function onSettingsChanged(payload: { locale?: string }) {
  if (payload.locale) applyLocale(payload.locale);
}

let unlistenSettings: (() => void) | null = null;

onMounted(async () => {
  if (typeof window !== "undefined" && (window as Window & { __TAURI__?: unknown }).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load("app-settings.json", { autoSave: false });
      const l = await store.get("locale");
      if (typeof l === "string" && l) applyLocale(l);
    } catch {
      // use default
    }
    const { listen } = await import("@tauri-apps/api/event");
    unlistenSettings = await listen<{ locale?: string }>("settings-changed", (e) => onSettingsChanged(e.payload));
  }
});

onUnmounted(() => {
  unlistenSettings?.();
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
html,
body,
#app {
  height: 100%;
  background: transparent;
  color: var(--app-text-color, #1a1a1a);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}
/* 图标随文字反色：SVG 使用当前文字色 */
#app svg[fill="currentColor"] {
  fill: currentColor;
  color: inherit;
}
#app svg[stroke="currentColor"] {
  stroke: currentColor;
  color: inherit;
}
/* 添加分组按钮：默认背景色 */
.sidebar-item-add {
  background: var(--sidebar-item-bg) !important;
}
html.sidebar-window .sidebar-item-add {
  background: var(--sidebar-item-bg) !important;
}
/* 侧栏独立窗口：透明、无边框、与主窗紧密贴合 */
html.sidebar-window,
html.sidebar-window body,
html.sidebar-window #app {
  background: transparent !important;
}
html.sidebar-window .desktop-layout,
html.sidebar-window .sidebar,
html.sidebar-window .sidebar-groups {
  border: none !important;
  outline: none !important;
  box-shadow: none !important;
  overflow-x: hidden !important;
  overflow-y: auto;
}
html.sidebar-window *,
html.sidebar-window *::before,
html.sidebar-window *::after {
  box-shadow: none !important;
  border: none !important;
  outline: none !important;
}
/* 分组窗体：分组项保留上左下主题色边框与阴影（覆盖上面的通配符） */
html.sidebar-window .sidebar-item {
  border: none !important;
  border-right: none !important;
  border-top: var(--sidebar-item-border-width, 1px) solid var(--el-color-primary, #409eff) !important;
  border-left: var(--sidebar-item-border-left-width, 1px) solid var(--el-color-primary, #409eff) !important;
  border-bottom: var(--sidebar-item-border-bottom-width, 1px) solid var(--el-color-primary, #409eff) !important;
  box-shadow: none !important;
}
/* 分组窗体：选中时只保留左边框，去除下边框 */
html.sidebar-window .sidebar-item.active {
  border-top: none !important;
  border-left: 5px solid var(--el-color-primary, #409eff) !important;
  border-bottom: none !important;
  box-shadow: -5px 0 10px color-mix(in srgb, var(--el-color-primary, #409eff) 28%, transparent) !important;
}
/* 分组窗体在右侧：分组项与新增按钮左对齐 */
html.sidebar-window.sidebar-on-right .sidebar-groups {
  align-items: flex-start !important;
}
html.sidebar-window.sidebar-on-right .sidebar-item-add {
  align-self: flex-start !important;
}
/* 分组窗体在右侧：边框在靠主窗的一侧 */
html.sidebar-window.sidebar-on-right .sidebar-item {
  border-left: none !important;
  border-right: var(--sidebar-item-border-width, 1px) solid var(--el-color-primary, #409eff) !important;
  border-top: var(--sidebar-item-border-width, 1px) solid var(--el-color-primary, #409eff) !important;
  border-bottom: var(--sidebar-item-border-bottom-width, 1px) solid var(--el-color-primary, #409eff) !important;
  border-radius: 0 8px 8px 0 !important;
  box-shadow: none !important;
}
html.sidebar-window.sidebar-on-right .sidebar-item.active {
  border-top: none !important;
  border-right: 5px solid var(--el-color-primary, #409eff) !important;
  border-bottom: none !important;
  box-shadow: 5px 0 10px color-mix(in srgb, var(--el-color-primary, #409eff) 28%, transparent) !important;
}
/* 分组窗体在右侧：选中项数字在名字左边显示 */
html.sidebar-window.sidebar-on-right .sidebar-item:not(.sidebar-item-add).active .sidebar-item-name {
  order: 1;
}
html.sidebar-window.sidebar-on-right .sidebar-item:not(.sidebar-item-add).active .sidebar-item-count {
  order: -1;
  margin-left: 0 !important;
  margin-right: 10px;
}
input,
textarea,
[contenteditable="true"] {
  -webkit-user-select: text;
  -moz-user-select: text;
  -ms-user-select: text;
  user-select: text;
}
</style>

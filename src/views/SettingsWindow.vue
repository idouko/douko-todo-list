<template>
  <div class="settings-window">
    <aside class="settings-menu">
      <button
        v-for="item in menuItems"
        :key="item.key"
        type="button"
        class="settings-menu-item"
        :class="{ active: currentMenu === item.key }"
        @click="currentMenu = item.key"
      >
        {{ $t(`settings.menu.${item.key}`) }}
      </button>
    </aside>
    <main class="settings-content-panel">
      <!-- 通用 -->
      <section v-show="currentMenu === 'general'" class="settings-pane">
        <h2 class="pane-title">{{ $t('settings.general.title') }}</h2>
        <div class="pane-body">
          <div class="form-row">
            <label class="form-label">{{ $t('settings.general.localeLabel') }}</label>
            <el-select v-model="locale" :placeholder="$t('settings.general.localePlaceholder')" style="width: 200px">
              <el-option :label="$t('settings.locale.zhCN')" value="zh-CN" />
              <el-option :label="$t('settings.locale.en')" value="en" />
              <el-option :label="$t('settings.locale.ja')" value="ja" />
            </el-select>
          </div>
        </div>
      </section>
      <!-- 外观 -->
      <section v-show="currentMenu === 'appearance'" class="settings-pane">
        <h2 class="pane-title">{{ $t('settings.appearance.title') }}</h2>
        <div class="pane-body">
          <div class="form-row">
            <label class="form-label">{{ $t('settings.appearance.themeColor') }}</label>
            <el-color-picker v-model="themeColor" :predefine="predefineColors" show-alpha />
            <span class="form-value">{{ themeColor }}</span>
          </div>
          <div class="form-row">
            <label class="form-label">{{ $t('settings.appearance.backgroundColor') }}</label>
            <el-color-picker v-model="backgroundColor" :predefine="predefineColors" show-alpha />
            <span class="form-value">{{ backgroundColor }}</span>
          </div>
          <div class="form-row form-row-block">
            <label class="form-label">{{ $t('settings.appearance.backgroundImage') }}</label>
            <div class="form-row-inner form-row-inner-bg-preview">
              <el-button size="small" @click="pickBackgroundImage">{{ $t('settings.appearance.pickImage') }}</el-button>
              <el-button v-if="backgroundImage" size="small" type="info" plain @click="clearBackgroundImage">{{ $t('settings.appearance.clearImage') }}</el-button>
              <img v-if="backgroundImage && backgroundImageSrc" :src="backgroundImageSrc" :alt="$t('settings.appearance.backgroundImage')" class="background-image-preview" />
            </div>
          </div>
          <div class="form-row">
            <label class="form-label">{{ $t('settings.appearance.sidebarPosition') }}</label>
            <el-radio-group v-model="sidebarPosition" @change="onSidebarPositionChange">
              <el-radio value="left">{{ $t('settings.appearance.sidebarPositionLeft') }}</el-radio>
              <el-radio value="right">{{ $t('settings.appearance.sidebarPositionRight') }}</el-radio>
            </el-radio-group>
          </div>
        </div>
      </section>
      <!-- 操作 -->
      <section v-show="currentMenu === 'operations'" class="settings-pane">
        <h2 class="pane-title">{{ $t('settings.operations.title') }}</h2>
        <div class="pane-body">
          <div class="form-row form-row-block">
            <label class="form-label">{{ $t('settings.operations.pagination.title') }}</label>
            <div class="form-row-inner">
              <el-switch v-model="paginationEnabled" />
              <span class="form-value">
                {{ paginationEnabled ? $t('settings.operations.pagination.enabled') : $t('settings.operations.pagination.disabled') }}
              </span>
            </div>
          </div>
          <div v-if="paginationEnabled" class="form-row">
            <label class="form-label">{{ $t('settings.operations.pagination.pageSize') }}</label>
            <el-select v-model="paginationPageSize" style="width: 200px">
              <el-option :label="'5'" :value="5" />
              <el-option :label="'10'" :value="10" />
              <el-option :label="'15'" :value="15" />
              <el-option :label="'20'" :value="20" />
              <el-option :label="'30'" :value="30" />
            </el-select>
          </div>
        </div>
      </section>
      <!-- 数据与安全 -->
      <section v-show="currentMenu === 'data'" class="settings-pane">
        <h2 class="pane-title">{{ $t('settings.data.title') }}</h2>
        <div class="pane-body">
          <template v-if="!detectTauri()">
            <p class="coming-soon">{{ $t('settings.data.comingSoon') }}</p>
          </template>
          <template v-else>
            <div v-if="storageInfoLoadError" class="form-row form-row-block">
              <span class="form-value form-value-error">{{ storageInfoLoadError }}</span>
            </div>
            <template v-else-if="storageInfo">
              <div class="form-row form-row-block">
                <label class="form-label">{{ $t('settings.data.sqlitePath') }}</label>
                <div class="form-row-inner form-row-inner-with-btn">
                  <span class="form-value form-value-path">{{ storageInfo.sqlitePath }}</span>
                  <el-button size="small" @click="openSqlitePathInFolder">
                    {{ $t('settings.data.openInFolder') }}
                  </el-button>
                </div>
              </div>
              <div class="form-row">
                <label class="form-label">{{ $t('settings.data.sqliteSize') }}</label>
                <span class="form-value">{{ formatBytes(storageInfo.sqliteSizeBytes) }}</span>
              </div>
              <div class="form-row">
                <label class="form-label">{{ $t('settings.data.appDataSize') }}</label>
                <span class="form-value">{{ formatBytes(storageInfo.appDataSizeBytes) }}</span>
              </div>
            </template>
            <div v-else-if="storageInfoLoading" class="form-row">
              <span class="form-value">{{ $t('settings.data.sqlitePath') }}…</span>
            </div>
          </template>
        </div>
      </section>
      <!-- 关于我们 -->
      <section v-show="currentMenu === 'about'" class="settings-pane">
        <h2 class="pane-title">{{ $t('settings.about.title') }}</h2>
        <div class="pane-body about-body">
          <p class="app-name">{{ $t('settings.about.appName') }}</p>
          <p class="app-version">{{ $t('settings.about.version', [appVersion]) }}</p>
          <template v-if="hasUpdate">
            <p class="update-available">{{ $t('settings.about.updateAvailable', [pendingUpdateVersion]) }}</p>
            <el-button
              type="primary"
              :loading="updateDownloading"
              @click="doUpdate"
            >
              {{ updateDownloading ? $t('settings.about.downloading') : $t('settings.about.updateNow') }}
            </el-button>
          </template>
          <template v-else>
            <p v-if="checkedUpdate" class="no-update">{{ $t('settings.about.latest') }}</p>
            <el-button v-else type="primary" @click="checkUpdate">{{ $t('settings.about.checkUpdate') }}</el-button>
          </template>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { setLocale } from "@/i18n";

const { t } = useI18n();

const STORE_KEY = "app-settings.json";
const KEYS = {
  themeColor: "themeColor",
  backgroundColor: "backgroundColor",
  backgroundImage: "backgroundImage",
  sidebarPosition: "sidebarPosition",
  locale: "locale",
  paginationEnabled: "paginationEnabled",
  paginationPageSize: "paginationPageSize",
} as const;

const menuItems = [
  { key: "general" },
  { key: "appearance" },
  { key: "operations" },
  { key: "data" },
  { key: "about" },
];

const currentMenu = ref<string>("general");
const storageInfo = ref<{
  sqlitePath: string;
  sqliteSizeBytes: number;
  appDataSizeBytes: number;
} | null>(null);
const storageInfoLoading = ref(false);
const storageInfoLoadError = ref("");
const locale = ref("zh-CN");
const settingsLoaded = ref(false);
const themeColor = ref("rgba(64, 249, 255, 0.61)");
const backgroundColor = ref("rgba(26, 26, 26, 0.56)");
const backgroundImage = ref("");
const sidebarPosition = ref<"left" | "right">("left");
/** 已持久化的分组栏位置（用于 Tauri 下取消时还原） */
const savedSidebarPosition = ref<"left" | "right">("left");
const paginationEnabled = ref(false);
const paginationPageSize = ref(10);
const predefineColors = [
  "#409eff",
  "#67c23a",
  "#e6a23c",
  "#f56c6c",
  "#909399",
  "#ff69b4",
  "#00ced1",
  "#ffffff",
  "#f5f5f5",
  "#1a1a1a",
];

/** 背景图用于 <img> 显示的 URL（Tauri 下用 convertFileSrc，回显与选中后一致显示图片内容） */
const backgroundImageSrc = computed(() => {
  const p = backgroundImage.value;
  if (!p) return "";
  if (detectTauri()) {
    const convert = (window as Window & { __TAURI_INTERNALS__?: { convertFileSrc?: (path: string) => string } })
      .__TAURI_INTERNALS__?.convertFileSrc;
    if (typeof convert === "function") return convert(p);
  }
  if (/^https?:\/\//i.test(p)) return p;
  return "";
});

const appVersion = ref("1.0.0");
const hasUpdate = ref(false);
const checkedUpdate = ref(false);
const updateDownloading = ref(false);
const pendingUpdateVersion = ref<string | null>(null);
let pendingUpdateObj: { downloadAndInstall: (cb?: (e: unknown) => void) => Promise<void> } | null = null;
let settingsStore: { get: (k: string) => Promise<unknown>; set: (k: string, v: unknown) => Promise<void>; save: () => Promise<void> } | null = null;

function detectTauri(): boolean {
  return typeof window !== "undefined" && !!(window as Window & { __TAURI__?: unknown }).__TAURI__;
}

function applyThemeToDocument() {
  document.documentElement.style.setProperty("--el-color-primary", themeColor.value);
  document.documentElement.style.setProperty("--app-border-color", themeColor.value);
}

function emitSettingsChanged() {
  if (!detectTauri()) return;
  const payload = {
    themeColor: themeColor.value,
    backgroundColor: backgroundColor.value,
    backgroundImage: backgroundImage.value,
    sidebarPosition: sidebarPosition.value,
    locale: locale.value,
    paginationEnabled: paginationEnabled.value,
    paginationPageSize: paginationPageSize.value,
  };
  import("@tauri-apps/api/event").then(({ emit, emitTo }) => {
    emit("settings-changed", payload).catch(() => {});
    emitTo("main", "settings-changed", payload).catch(() => {});
    emitTo("qrcode", "settings-changed", payload).catch(() => {});
    emitTo("sidebar", "settings-changed", payload).catch(() => {});
  });
}

async function saveAndEmit() {
  if (settingsStore) {
    await settingsStore.set(KEYS.themeColor, themeColor.value);
    await settingsStore.set(KEYS.backgroundColor, backgroundColor.value);
    await settingsStore.set(KEYS.backgroundImage, backgroundImage.value);
    await settingsStore.set(KEYS.sidebarPosition, sidebarPosition.value);
    await settingsStore.set(KEYS.locale, locale.value);
    await settingsStore.set(KEYS.paginationEnabled, paginationEnabled.value);
    await settingsStore.set(KEYS.paginationPageSize, paginationPageSize.value);
    await settingsStore.save();
  }
  savedSidebarPosition.value = sidebarPosition.value;
  emitSettingsChanged();
}

watch([themeColor, backgroundColor, backgroundImage, locale, paginationEnabled, paginationPageSize], () => {
  applyThemeToDocument();
  if (!settingsLoaded.value) return;
  setLocale(locale.value);
  saveAndEmit();
});

/** 分组栏位置变更：Tauri 下弹窗二次确认，确认后保存并重启；取消则还原。非 Tauri 直接保存。 */
async function onSidebarPositionChange() {
  if (!settingsLoaded.value) return;
  if (!detectTauri()) {
    await saveAndEmit();
    return;
  }
  const { ElMessageBox } = await import("element-plus");
  try {
    await ElMessageBox.confirm(t("settings.appearance.sidebarPositionRestartMessage"), t("settings.appearance.sidebarPositionRestartTitle"), {
      confirmButtonText: t("common.ok"),
      cancelButtonText: t("common.cancel"),
      type: "warning",
    });
  } catch {
    sidebarPosition.value = savedSidebarPosition.value;
    return;
  }
  if (settingsStore) {
    await settingsStore.set(KEYS.sidebarPosition, sidebarPosition.value);
    await settingsStore.save();
  }
  savedSidebarPosition.value = sidebarPosition.value;
  emitSettingsChanged();
  const { relaunch } = await import(/* @vite-ignore */ "@tauri-apps/plugin-process");
  await relaunch();
}

onMounted(async () => {
  if (detectTauri()) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      appVersion.value = (await invoke("get_app_version")) as string;
    } catch {
      appVersion.value = "1.0.0";
    }
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      settingsStore = await load(STORE_KEY, { autoSave: false });
      const t = await settingsStore.get(KEYS.themeColor);
      const b = await settingsStore.get(KEYS.backgroundColor);
      const bi = await settingsStore.get(KEYS.backgroundImage);
      const sp = await settingsStore.get(KEYS.sidebarPosition);
      const l = await settingsStore.get(KEYS.locale);
      const pe = await settingsStore.get(KEYS.paginationEnabled);
      const ps = await settingsStore.get(KEYS.paginationPageSize);
      if (typeof t === "string") themeColor.value = t;
      if (typeof b === "string") backgroundColor.value = b;
      if (typeof bi === "string") backgroundImage.value = bi;
      if (sp === "left" || sp === "right") {
        sidebarPosition.value = sp;
        savedSidebarPosition.value = sp;
      }
      if (typeof l === "string") {
        locale.value = l;
        setLocale(l);
      }
      if (typeof pe === "boolean") paginationEnabled.value = pe;
      if (typeof ps === "number") paginationPageSize.value = ps;
      settingsLoaded.value = true;
      emitSettingsChanged();
    } catch {
      settingsStore = null;
      settingsLoaded.value = true;
    }
  }
});

async function pickBackgroundImage() {
  if (!detectTauri()) return;
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const { invoke } = await import("@tauri-apps/api/core");
    const selected = await open({
      multiple: false,
      filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
    });
    const path = Array.isArray(selected) ? selected[0] : selected;
    if (path && typeof path === "string") {
      const cachedPath = (await invoke("save_background_image", { sourcePath: path })) as string;
      backgroundImage.value = cachedPath;
    }
  } catch (e) {
    console.error(e);
  }
}

async function clearBackgroundImage() {
  if (detectTauri()) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("clear_background_image");
    } catch (e) {
      console.error(e);
    }
  }
  backgroundImage.value = "";
}

function formatBytes(bytes: number): string {
  const t = (key: string, n: number) => `${n.toFixed(2)} ${key}`;
  if (bytes < 1024) return `${bytes} ${locale.value.startsWith("zh") ? "字节" : "bytes"}`;
  if (bytes < 1024 * 1024) return t("KB", bytes / 1024);
  return t("MB", bytes / (1024 * 1024));
}

async function loadStorageInfo() {
  if (!detectTauri()) return;
  storageInfoLoading.value = true;
  storageInfoLoadError.value = "";
  storageInfo.value = null;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const info = (await invoke("get_storage_info")) as {
      sqlitePath: string;
      sqliteSizeBytes: number;
      appDataSizeBytes: number;
    };
    storageInfo.value = info;
  } catch (e) {
    storageInfoLoadError.value = e instanceof Error ? e.message : t("settings.data.loadFailed");
  } finally {
    storageInfoLoading.value = false;
  }
}

async function openSqlitePathInFolder() {
  if (!storageInfo.value?.sqlitePath || !detectTauri()) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("open_path_in_folder", { path: storageInfo.value.sqlitePath });
  } catch (e) {
    console.error(e);
  }
}

watch(currentMenu, (menu) => {
  if (menu === "data" && detectTauri()) loadStorageInfo();
});

async function checkUpdate() {
  if (!detectTauri()) return;
  try {
    const { check } = await import(/* @vite-ignore */ "@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      hasUpdate.value = true;
      pendingUpdateVersion.value = update.version;
      pendingUpdateObj = update;
    }
    checkedUpdate.value = true;
  } catch {
    checkedUpdate.value = true;
  }
}

async function doUpdate() {
  if (!detectTauri() || !pendingUpdateObj) return;
  try {
    const { relaunch } = await import(/* @vite-ignore */ "@tauri-apps/plugin-process");
    updateDownloading.value = true;
    await pendingUpdateObj.downloadAndInstall();
    await relaunch();
  } catch (e) {
    console.error(e);
    updateDownloading.value = false;
  }
}
</script>

<style scoped>
.settings-window {
  display: flex;
  height: 100vh;
  background: #fff;
}
.settings-menu {
  width: 160px;
  flex-shrink: 0;
  padding: 16px 0;
  border-right: 1px solid var(--app-border-color, #e8e8e8);
  background: #fafafa;
}
.settings-menu-item {
  display: block;
  width: 100%;
  padding: 10px 20px;
  text-align: left;
  border: none;
  background: transparent;
  color: #333;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}
.settings-menu-item:hover {
  background: color-mix(in srgb, var(--el-color-primary, #409eff) 10%, #f5f5f5);
  color: var(--el-color-primary, #409eff);
}
.settings-menu-item.active {
  background: color-mix(in srgb, var(--el-color-primary, #409eff) 14%, white);
  color: var(--el-color-primary, #409eff);
  font-weight: 500;
}
.settings-content-panel {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}
.settings-pane {
  max-width: 480px;
}
.pane-title {
  margin: 0 0 20px;
  font-size: 18px;
  font-weight: 600;
  color: #1a1a1a;
}
.pane-body {
  margin-bottom: 24px;
}
.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.form-label {
  min-width: 80px;
  font-size: 14px;
  color: #606266;
}
.form-value {
  font-size: 12px;
  color: #909399;
}
.form-row-block {
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}
.form-row-inner {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.form-row-inner-bg-preview {
  align-items: flex-start;
}
.background-image-preview {
  display: block;
  max-width: 200px;
  max-height: 120px;
  width: auto;
  height: auto;
  object-fit: contain;
  border-radius: 6px;
  border: 1px solid var(--app-border-color, #e8e8e8);
}
.form-value-path {
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.form-row-inner-with-btn {
  width: 100%;
  max-width: 360px;
}
.form-row-inner-with-btn .form-value-path {
  max-width: none;
  flex: 1;
  min-width: 0;
}
.form-value-error {
  color: #f56c6c;
  font-size: 13px;
}
.coming-soon {
  color: #909399;
  font-size: 14px;
}
.about-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.app-name {
  font-size: 20px;
  font-weight: 600;
  color: #1a1a1a;
  margin: 0;
}
.app-version {
  font-size: 14px;
  color: #606266;
  margin: 0;
}
.no-update {
  font-size: 14px;
  color: #67c23a;
  margin: 0;
}
.update-available {
  font-size: 14px;
  color: #409eff;
  margin: 0 0 8px;
}
</style>

<template>
  <div class="qrcode-display">
    <h3 class="section-title">扫码用手机编辑</h3>
    <p class="hint">与电脑连接同一 WiFi，扫码即可在手机上管理任务</p>
    <div v-if="url" class="qrcode-wrap">
      <QrcodeCanvas :value="url" :level="'H'" :size="200" class="qrcode-img" />
      <p class="url-text">{{ url }}</p>
    </div>
    <div v-else class="placeholder">
      <span>等待服务启动…</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { QrcodeCanvas } from "qrcode.vue";

defineProps<{
  url: string;
}>();
</script>

<style scoped>
.qrcode-display {
  text-align: center;
}
.section-title {
  font-size: 1rem;
  margin-bottom: 8px;
  color: #1a1a2e;
}
.hint {
  font-size: 12px;
  color: #666;
  margin-bottom: 16px;
  line-height: 1.4;
}
.qrcode-wrap {
  padding: 12px;
  background: #fff;
  border-radius: 8px;
  border: 1px solid var(--app-border-color, #e8e8e8);
}
.qrcode-img {
  width: 200px;
  height: 200px;
  display: block;
  margin: 0 auto 12px;
}
.url-text {
  font-size: 12px;
  color: #666;
  word-break: break-all;
}
.placeholder {
  padding: 40px 20px;
  color: #999;
  font-size: 14px;
}
</style>

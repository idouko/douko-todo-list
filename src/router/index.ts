import { createRouter, createWebHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";
import Desktop from "@/views/Desktop.vue";
import MobileTodo from "@/views/MobileTodo.vue";
import QrcodeWindow from "@/views/QrcodeWindow.vue";
import SettingsWindow from "@/views/SettingsWindow.vue";

const routes: RouteRecordRaw[] = [
  { path: "/", name: "Desktop", component: Desktop },
  { path: "/mobile", name: "MobileTodo", component: MobileTodo },
  { path: "/qrcode-window", name: "QrcodeWindow", component: QrcodeWindow },
  { path: "/settings-window", name: "SettingsWindow", component: SettingsWindow },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;

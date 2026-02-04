import { createApp } from "vue";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import App from "./App.vue";
import router from "./router";
import { i18n } from "./i18n";

if (typeof window !== "undefined" && window.location.search.includes("window=sidebar")) {
  document.documentElement.classList.add("sidebar-window");
}

const app = createApp(App);
app.use(i18n);
app.use(ElementPlus);
app.use(router);
router.isReady().then(() => {
  app.mount("#app");
});

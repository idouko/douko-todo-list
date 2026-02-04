import { createI18n } from "vue-i18n";
import zhCN from "./locales/zh-CN";
import en from "./locales/en";
import ja from "./locales/ja";

export type LocaleKey = "zh-CN" | "en" | "ja";

const messages = {
  "zh-CN": zhCN,
  en,
  ja,
};

export const i18n = createI18n({
  legacy: false,
  locale: "zh-CN",
  fallbackLocale: "en",
  messages,
});

export function setLocale(locale: string) {
  const key = locale in messages ? (locale as LocaleKey) : "zh-CN";
  i18n.global.locale.value = key;
  return key;
}

export function getElementPlusLocale(locale: string) {
  switch (locale) {
    case "zh-CN":
      return import("element-plus/es/locale/lang/zh-cn").then((m) => m.default);
    case "ja":
      return import("element-plus/es/locale/lang/ja").then((m) => m.default);
    case "en":
    default:
      return import("element-plus/es/locale/lang/en").then((m) => m.default);
  }
}

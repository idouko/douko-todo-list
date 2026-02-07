#!/usr/bin/env node
/**
 * 重新生成 Tauri 更新签名的密钥对（无密码）
 * 用法：pnpm run key:regenerate
 *
 * 生成后需：
 * 1. 更新 src-tauri/tauri.conf.json 中的 plugins.updater.pubkey
 * 2. 更新 GitHub Secrets 中的 TAURI_SIGNING_PRIVATE_KEY
 * 3. 删除 GitHub Secrets 中的 TAURI_SIGNING_PRIVATE_KEY_PASSWORD（若存在）
 */
import { execSync } from "child_process";
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const keyPath = join(root, ".tauri", "xy-todo-list.key");

console.log("\n🔑 正在生成新的签名密钥对（无密码）...\n");

execSync(`pnpm tauri signer generate -w ${keyPath} -f --ci`, {
  cwd: root,
  stdio: "inherit",
});

// 读取私钥和公钥
const privateKey = readFileSync(keyPath, "utf-8");
const publicKey = readFileSync(keyPath + ".pub", "utf-8").trim();

// 更新 tauri.conf.json
const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf-8"));
tauriConf.plugins.updater.pubkey = publicKey;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

console.log("\n✅ 密钥已生成，tauri.conf.json 已更新公钥。");
console.log("\n📋 请将以下私钥完整复制到 GitHub Secrets → TAURI_SIGNING_PRIVATE_KEY：");
console.log("─".repeat(60));
console.log(privateKey);
console.log("─".repeat(60));
console.log("\n⚠️ 若 GitHub Secrets 中存在 TAURI_SIGNING_PRIVATE_KEY_PASSWORD，请删除。\n");

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
const b64 = Buffer.from(privateKey, "utf-8").toString("base64");
console.log("\n📋 请将以下 Base64 字符串（私钥）完整复制到 GitHub Secrets → TAURI_SIGNING_PRIVATE_KEY_BASE64：");
console.log("   （注意：仅复制 Base64，不要复制横线 ─）");
console.log("─".repeat(60));
console.log(b64);
console.log("─".repeat(60));
console.log("\n⚠️ 不要配置 TAURI_SIGNING_PRIVATE_KEY_PASSWORD；若存在请删除。详见 DEPLOY.md。\n");

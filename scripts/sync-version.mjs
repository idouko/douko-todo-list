#!/usr/bin/env node
/**
 * 从 package.json 读取 version，同步到 src-tauri/Cargo.toml
 * 用法：node scripts/sync-version.mjs
 * 在 pnpm version 后自动执行（见 package.json 的 version script）
 */
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf-8"));
const version = pkg.version;
if (!version) {
  console.error("package.json 中未找到 version 字段");
  process.exit(1);
}

const cargoPath = join(root, "src-tauri", "Cargo.toml");
let cargo = readFileSync(cargoPath, "utf-8");
cargo = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
writeFileSync(cargoPath, cargo);
console.log(`已同步 version ${version} 到 Cargo.toml`);

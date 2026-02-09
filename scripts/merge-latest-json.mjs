#!/usr/bin/env node
/**
 * 合并各平台的 latest-{platform}.json 为 latest.json
 * 用法：node scripts/merge-latest-json.mjs <dir> [<dir> ...]
 * 每个 dir 下应有 latest-darwin-aarch64.json 等文件
 */
import { existsSync, readFileSync, writeFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

const dirs = process.argv.slice(2).filter(Boolean);
if (dirs.length === 0) {
  console.error("用法: node merge-latest-json.mjs <dir> [<dir> ...]");
  process.exit(1);
}

const platforms = {};
let version = "";
let notes = "See release notes on GitHub";

for (const dir of dirs) {
  if (!existsSync(dir)) continue;
  const files = ["darwin-aarch64", "darwin-x86_64", "linux-x86_64", "windows-x86_64"]
    .map((p) => join(dir, `latest-${p}.json`))
    .filter((p) => existsSync(p));
  for (const f of files) {
    try {
      const data = JSON.parse(readFileSync(f, "utf-8"));
      if (data.platformKey && data.platformInfo) {
        platforms[data.platformKey] = data.platformInfo;
        if (data.version) version = data.version;
      }
    } catch (e) {
      console.warn("跳过:", f, e.message);
    }
  }
}

if (Object.keys(platforms).length === 0) {
  console.error("未找到任何 platform 信息");
  process.exit(1);
}

const latest = {
  version,
  notes,
  pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
  platforms,
};

const outPath = join(root, "latest.json");
writeFileSync(outPath, JSON.stringify(latest, null, 2) + "\n");
console.log("已生成:", outPath);

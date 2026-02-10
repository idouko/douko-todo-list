#!/usr/bin/env node
/**
 * 将各平台 build 产物展平到 release-assets，避免同名覆盖
 * macOS 两个架构的 .app 会重命名为含 arch 后缀
 */
import { cpSync, mkdirSync, readdirSync, statSync } from "fs";
import { join, dirname, basename } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const artifactsDir = join(root, "artifacts");
const outDir = join(root, "release-assets");

mkdirSync(outDir, { recursive: true });

const dirs = ["build-darwin-aarch64", "build-darwin-x86_64", "build-linux-x86_64", "build-windows-x86_64"];
// 平台前缀，便于在 Release 页面区分
const platformPrefix = {
  "build-darwin-aarch64": "macOS-AppleSilicon-",
  "build-darwin-x86_64": "macOS-Intel-",
  "build-linux-x86_64": "Linux-",
  "build-windows-x86_64": "Windows-",
};
const archSuffix = { "build-darwin-aarch64": "aarch64", "build-darwin-x86_64": "x64" };

function collect(dir, base = "", prefix = "") {
  const full = join(artifactsDir, dir, base);
  try {
    if (!statSync(full).isDirectory()) return;
  } catch {
    return;
  }
  const entries = readdirSync(full, { withFileTypes: true });
  for (const e of entries) {
    const srcPath = join(full, e.name);
    if (e.isDirectory()) {
      if (e.name.endsWith(".app")) {
        const suffix = archSuffix[dir];
        const destName = suffix ? e.name.replace(/\.app$/, `_${suffix}.app`) : e.name;
        cpSync(srcPath, join(outDir, prefix + destName), { recursive: true });
      } else {
        collect(dir, base ? `${base}/${e.name}` : e.name, prefix);
      }
    } else {
      const name = basename(e.name);
      if (name.startsWith("latest-") && name.endsWith(".json")) continue;
      cpSync(srcPath, join(outDir, prefix + name), { force: true });
    }
  }
}

for (const d of dirs) {
  const full = join(artifactsDir, d);
  try {
    if (statSync(full).isDirectory()) collect(d, "", platformPrefix[d] || "");
  } catch (_) {}
}

const latest = join(root, "latest.json");
if (statSync(latest, { throwIfNoEntry: false })) {
  cpSync(latest, join(outDir, "latest.json"));
}

console.log("Flattened assets:", readdirSync(outDir).length, "items");

#!/usr/bin/env node
/**
 * 仅收集安装包、签名和 updater 元数据到 release-artifacts/，供 workflow 上传
 * 用法：node scripts/collect-release-artifacts.mjs <platform_key> <rust_target>
 */
import { cpSync, existsSync, mkdirSync, readdirSync } from "fs";
import { join, dirname, basename } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const srcTauri = join(root, "src-tauri");
const targetDir = join(srcTauri, "target");
const outDir = join(root, "release-artifacts");

const [platformKey, rustTarget] = process.argv.slice(2);
if (!platformKey || !rustTarget) {
  console.error("用法: node collect-release-artifacts.mjs <platform_key> <rust_target>");
  process.exit(1);
}

let bundleBase = join(targetDir, rustTarget, "release", "bundle");
if (!existsSync(bundleBase)) bundleBase = join(targetDir, "release", "bundle");
if (!existsSync(bundleBase)) {
  console.error("bundle 根目录不存在");
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });

function copy(src, destName) {
  if (!existsSync(src)) return;
  cpSync(src, join(outDir, destName || basename(src)), { force: true });
}

if (platformKey.startsWith("darwin")) {
  const macosDir = join(bundleBase, "macos");
  if (!existsSync(macosDir)) {
    console.error("macOS 目录不存在:", macosDir);
    process.exit(1);
  }
  for (const f of readdirSync(macosDir)) {
    const path = join(macosDir, f);
    if (require("fs").statSync(path).isDirectory() && f.endsWith(".app")) {
      cpSync(path, join(outDir, f), { recursive: true });
    } else if (f.endsWith(".app.tar.gz") || f.endsWith(".sig")) {
      copy(path);
    }
  }
  copy(join(bundleBase, `latest-${platformKey}.json`));
} else if (platformKey.startsWith("linux")) {
  const dir = existsSync(join(bundleBase, "appimage")) ? join(bundleBase, "appimage") : join(bundleBase, "AppImage");
  if (existsSync(dir)) {
    for (const f of readdirSync(dir)) {
      if (f.endsWith(".AppImage.tar.gz") || f.endsWith(".sig")) copy(join(dir, f));
    }
  }
  copy(join(bundleBase, `latest-${platformKey}.json`));
} else if (platformKey.startsWith("windows")) {
  for (const sub of ["nsis", "msi"]) {
    const dir = join(bundleBase, sub);
    if (!existsSync(dir)) continue;
    for (const f of readdirSync(dir)) {
      const path = join(dir, f);
      if (f.endsWith(".nsis.zip") || f.endsWith(".msi.zip") || f.endsWith(".sig")) copy(path);
      else if (f.endsWith(".exe") || f.endsWith(".msi")) copy(path);
    }
  }
  copy(join(bundleBase, `latest-${platformKey}.json`));
} else {
  console.error("未知 platform_key:", platformKey);
  process.exit(1);
}

const count = readdirSync(outDir).length;
console.log("已收集", count, "个文件到 release-artifacts/");

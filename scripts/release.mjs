#!/usr/bin/env node
/**
 * 打包发布脚本：更新版本号、同步配置、打 tag 并推送以触发 GitHub Actions 构建
 *
 * 用法：
 *   pnpm release 1.0.1
 *   pnpm release v1.0.1
 *   pnpm release 1.0.1 --no-push   # 仅本地操作，网络恢复后手动 git push
 *
 * 会执行：
 * 1. 更新 package.json 的 version
 * 2. 同步到 Cargo.toml (pnpm version:sync)
 * 3. git add 并 commit
 * 4. 删除已存在的同名 tag（本地 + 远程，--no-push 时跳过远程）
 * 5. 创建 tag v{version} 并推送（--no-push 时跳过）
 */
import { readFileSync, writeFileSync } from "fs";
import { execSync } from "child_process";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

// 解析参数
const args = process.argv.slice(2);
const noPush = args.includes("--no-push");
const raw = args.find((a) => !a.startsWith("-"));
if (!raw) {
  console.error("用法: pnpm release <版本号> [--no-push]");
  console.error("示例: pnpm release 1.0.1 或 pnpm release 1.0.1 --no-push");
  process.exit(1);
}
const version = raw.startsWith("v") ? raw.slice(1) : raw;
const tagName = `v${version}`;

// 简单校验版本格式 (x.y.z)
if (!/^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?$/.test(version)) {
  console.error(`无效的版本号: ${version}，期望格式如 1.0.0 或 1.0.0-beta.1`);
  process.exit(1);
}

console.log(`\n📦 准备发布版本 ${version} (tag: ${tagName})\n`);

// 1. 更新 package.json
const pkgPath = join(root, "package.json");
const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
pkg.version = version;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n", "utf-8");
console.log(`✓ 已更新 package.json version -> ${version}`);

// 2. 同步到 Cargo.toml
execSync("pnpm version:sync", { cwd: root, stdio: "inherit" });

// 3. git add & commit
const files = ["package.json", "src-tauri/Cargo.toml"];
execSync(`git add ${files.join(" ")}`, { cwd: root, stdio: "inherit" });
try {
  execSync(`git commit -m "chore: bump version to ${version}"`, {
    cwd: root,
    stdio: "inherit",
  });
} catch {
  console.log("(无变更或已提交，跳过 commit)");
}

// 4. 删除已存在的 tag
try {
  execSync(`git tag -d ${tagName}`, { cwd: root, stdio: "pipe" });
  console.log(`✓ 已删除本地 tag ${tagName}`);
} catch {
  // 本地不存在则忽略
}
if (!noPush) {
  try {
    execSync(`git push origin :refs/tags/${tagName}`, {
      cwd: root,
      stdio: "pipe",
    });
    console.log(`✓ 已删除远程 tag ${tagName}`);
  } catch {
    // 远程不存在则忽略
  }
}

// 5. 创建 tag
execSync(`git tag ${tagName}`, { cwd: root, stdio: "inherit" });
console.log(`✓ 已创建 tag ${tagName}`);

if (noPush) {
  console.log("\n⏸ 已跳过推送 (--no-push)。网络恢复后执行：");
  console.log(`   git push origin main`);
  console.log(`   git push origin ${tagName}\n`);
} else {
  console.log("\n推送中...");
  execSync("git push origin main", { cwd: root, stdio: "inherit" });
  execSync(`git push origin ${tagName}`, { cwd: root, stdio: "inherit" });
  console.log(`\n✅ 发布流程已完成，GitHub Actions 将开始构建。`);
  console.log(`   到 Actions 页查看 release 工作流进度。\n`);
}

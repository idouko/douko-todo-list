#!/usr/bin/env node
/**
 * Moss 方案：构建后手动创建并签名 updater 产物
 * 用法：node scripts/create-and-sign-updater.mjs <platform> <target>
 * platform: darwin-aarch64 | darwin-x86_64 | linux-x86_64 | windows-x86_64
 * target: 如 aarch64-apple-darwin、x86_64-unknown-linux-gnu、x86_64-pc-windows-msvc
 */
import { execFileSync, execSync } from "child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, unlinkSync, writeFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const srcTauri = join(root, "src-tauri");
const targetDir = join(srcTauri, "target");

const [platform, rustTarget] = process.argv.slice(2);
if (!platform || !rustTarget) {
  console.error("用法: node create-and-sign-updater.mjs <platform> <rustTarget>");
  process.exit(1);
}

const keyBase64 = process.env.TAURI_SIGNING_PRIVATE_KEY_BASE64;
if (!keyBase64) {
  console.error("TAURI_SIGNING_PRIVATE_KEY_BASE64 未设置");
  process.exit(1);
}

const keyBase64Trimmed = keyBase64.replace(/\s/g, "");
const decodedKey = Buffer.from(keyBase64Trimmed, "base64");
const decodedStr = decodedKey.toString("utf-8");
const firstLine = decodedStr.split("\n")[0] || decodedStr.slice(0, 80);

// 输出脱敏信息便于对比
console.log("[签名] Base64 长度:", keyBase64Trimmed.length, "| 解码后字节:", decodedKey.length, "| 首行:", firstLine);

if (decodedKey.length < 50) {
  console.error("错误：解码后密钥过短，请检查是否完整复制了 base64 到 GitHub Secrets");
  process.exit(1);
}
if (decodedStr.includes("minisign public key")) {
  console.error("错误：你填入了公钥，请使用私钥的 base64。运行 pnpm run key:regenerate 获取正确格式。");
  process.exit(1);
}
if (!decodedStr.includes("untrusted comment") || !decodedStr.includes("rsign")) {
  console.error("错误：解码后的密钥格式无效。请执行 pnpm run key:regenerate，将输出的 Base64 完整复制到 GitHub Secrets。");
  process.exit(1);
}

const bundleBase = join(targetDir, rustTarget, "release", "bundle");
let updaterBundle = null;
let updaterBasename = null;
let platformKey = platform;

if (platform.startsWith("darwin")) {
  const macosDir = join(bundleBase, "macos");
  if (!existsSync(macosDir)) {
    console.error("macOS bundle 目录不存在:", macosDir);
    process.exit(1);
  }
  const apps = readdirSync(macosDir).filter((f) => f.endsWith(".app"));
  if (apps.length === 0) {
    console.error("未找到 .app 包");
    process.exit(1);
  }
  const appName = apps[0];
  const version = JSON.parse(readFileSync(join(root, "package.json"), "utf-8")).version;
  const arch = platform.includes("aarch64") ? "aarch64" : "x64";
  updaterBasename = `${appName.replace(/\.app$/, "")}_${version}_${arch}.app.tar.gz`;
  updaterBundle = join(macosDir, updaterBasename);
  console.log("创建 .app.tar.gz...");
  execSync(`tar -czf "${updaterBundle}" -C "${macosDir}" "${appName}"`, {
    cwd: root,
    stdio: "inherit",
  });
} else if (platform.startsWith("linux")) {
  const appimageDir = join(bundleBase, "appimage");
  if (!existsSync(appimageDir)) {
    console.error("AppImage 目录不存在:", appimageDir);
    process.exit(1);
  }
  const appimages = readdirSync(appimageDir).filter((f) => f.endsWith(".AppImage"));
  if (appimages.length === 0) {
    console.error("未找到 .AppImage");
    process.exit(1);
  }
  const appimageName = appimages[0];
  const appimagePath = join(appimageDir, appimageName);
  updaterBasename = `${appimageName}.tar.gz`;
  updaterBundle = join(appimageDir, updaterBasename);
  console.log("创建 .AppImage.tar.gz...");
  execSync(`tar -czf "${updaterBundle}" -C "${appimageDir}" "${appimageName}"`, {
    cwd: root,
    stdio: "inherit",
  });
} else if (platform.startsWith("windows")) {
  // Windows: nsis 或 msi，updater 用 .zip（Tauri 格式：xxx-setup.nsis.zip 或 xxx.msi.zip）
  const nsisDir = join(bundleBase, "nsis");
  const msiDir = join(bundleBase, "msi");
  let dir = nsisDir;
  let pattern = /\.exe$/;
  let zipExt = "nsis.zip";
  if (!existsSync(nsisDir) && existsSync(msiDir)) {
    dir = msiDir;
    pattern = /\.msi$/;
    zipExt = "msi.zip";
  }
  if (!existsSync(dir)) {
    console.error("Windows bundle 目录不存在:", dir);
    process.exit(1);
  }
  const files = readdirSync(dir);
  const installer = files.find((f) => pattern.test(f));
  if (!installer) {
    console.error("未找到 Windows 安装包");
    process.exit(1);
  }
  const base = installer.replace(pattern, "");
  updaterBasename = `${base}.${zipExt}`;
  updaterBundle = join(dir, updaterBasename);
  console.log("创建 .zip...");
  execSync(`tar -a -cf "${updaterBundle}" -C "${dir}" "${installer}"`, {
    cwd: root,
    stdio: "inherit",
  });
} else {
  console.error("未知 platform:", platform);
  process.exit(1);
}

if (!existsSync(updaterBundle)) {
  console.error("updater 包创建失败");
  process.exit(1);
}

console.log("使用 tauri signer sign 签名...");
// 推荐方案：使用“有密码”的私钥，在 CI 中通过环境变量传入密码。
// - 本地生成密钥时使用 tauri signer generate（不加 --ci），设置一个非空密码
// - GitHub Secrets:
//   - TAURI_SIGNING_PRIVATE_KEY_BASE64: .tauri/xy-todo-list.key 的完整 Base64 内容
//   - TAURI_SIGNING_PRIVATE_KEY_PASSWORD: 上面生成密钥时设置的密码
const signingPassword = process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD;
if (!signingPassword) {
  console.error(
    "错误：未配置 TAURI_SIGNING_PRIVATE_KEY_PASSWORD。请使用带密码的密钥，并在 GitHub Secrets 中设置该密码。"
  );
  process.exit(1);
}

// 将 Base64 写入临时文件，用 -f 传路径；密码仅通过环境变量传递，避免 -p 空字符串在 CI 中被吃掉
const tmpKeyPath = join(root, ".tauri", "ci-signing.key");
mkdirSync(dirname(tmpKeyPath), { recursive: true });
writeFileSync(tmpKeyPath, keyBase64Trimmed);
const tauriBin = join(
  root,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "tauri.CMD" : "tauri"
);
try {
  execFileSync(
    tauriBin,
    ["signer", "sign", "-f", tmpKeyPath, updaterBundle],
    {
      cwd: root,
      stdio: "inherit",
      env: { ...process.env, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: signingPassword },
    }
  );
} finally {
  if (existsSync(tmpKeyPath)) {
    unlinkSync(tmpKeyPath);
  }
}

const sigPath = `${updaterBundle}.sig`;
if (!existsSync(sigPath)) {
  console.error("签名文件未生成");
  process.exit(1);
}

const signature = readFileSync(sigPath, "utf-8").trim();
const version = JSON.parse(readFileSync(join(root, "package.json"), "utf-8")).version;
const platformInfo = {
  signature,
  url: `https://github.com/${process.env.GITHUB_REPOSITORY || "OWNER/REPO"}/releases/download/app-v${version}/${updaterBasename}`,
};

// 写入到 bundle 根目录，便于 workflow 收集
const latestPartPath = join(bundleBase, `latest-${platformKey}.json`);
writeFileSync(
  latestPartPath,
  JSON.stringify({ platformKey, version, platformInfo }, null, 2)
);
console.log("已写入:", latestPartPath);

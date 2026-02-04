# XY Todo List 部署与自动更新指南

本文档说明如何将项目上传到 GitHub、配置自动打包（Linux / macOS / Windows），以及应用内检查更新与安装新版本的完整流程。

---

## 一、将项目上传到 GitHub

### 1.1 在 GitHub 上创建仓库

1. 登录 [GitHub](https://github.com)，点击右上角 **+** → **New repository**。
2. 填写：
   - **Repository name**：例如 `xy-todo-list`
   - **Description**：可选，如「轻量化桌面 Todo List，支持三端与扫码编辑」
   - 选择 **Public**，**不要**勾选 “Add a README file”（本地已有代码）。
3. 点击 **Create repository**，记下仓库地址，例如：  
   `https://github.com/YOUR_USERNAME/xy-todo-list.git`

### 1.2 本地初始化并推送（若尚未初始化 Git）

在项目根目录执行：

```bash
# 若尚未初始化
git init
git add .
git commit -m "Initial commit: XY Todo List"

# 添加远程仓库（将 YOUR_USERNAME/xy-todo-list 换成你的仓库）
git remote add origin https://github.com/YOUR_USERNAME/xy-todo-list.git

# 推送主分支（默认 main 或 master，按你当前分支名）
git branch -M main
git push -u origin main
```

若项目已有 Git 且未添加过该远程：

```bash
git remote add origin https://github.com/YOUR_USERNAME/xy-todo-list.git
git push -u origin main
```

---

## 二、配置 Tauri 更新器（应用内更新）

应用内更新依赖 **Tauri Updater**：需要一对密钥对更新包签名，并在配置里填写公钥和更新地址。

### 2.1 生成更新签名密钥对

在项目根目录执行（需已安装 [Tauri CLI](https://v2.tauri.app/start/install/)）：

```bash
pnpm tauri signer generate -w ~/.tauri/xy-todo-list.key
```

- 会生成 **私钥** 和 **公钥**。
- 终端会输出类似：
  - `Your keypair was generated successfully`
  - `Private: ...`（一长串，仅出现一次，务必保存）
  - `Public: dcdc8d...`（公钥）

**重要**：把 **Private** 整段复制保存到安全处，后面要放进 GitHub Secrets，且只显示一次。

### 2.2 在配置中填写公钥和更新地址

编辑 `src-tauri/tauri.conf.json`，找到 `plugins.updater`，改成你的仓库和公钥：

```json
"plugins": {
  "updater": {
    "pubkey": "这里粘贴 2.1 步生成的 Public 公钥",
    "endpoints": [
      "https://github.com/YOUR_USERNAME/xy-todo-list/releases/latest/download/latest.json"
    ]
  }
}
```

- 将 `YOUR_USERNAME/xy-todo-list` 换成你的 **GitHub 用户名/仓库名**。
- `latest.json` 会在每次 CI 打包时由 Tauri 自动生成并上传到对应 Release，无需手写。

保存后提交：

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: config updater pubkey and endpoint"
git push
```

**注意**：公钥可以提交到仓库；私钥绝不能提交，只放在本机和 GitHub Secrets 中。

### 2.3 在 GitHub 中配置私钥（用于 CI 签名）

1. 打开仓库 → **Settings** → **Secrets and variables** → **Actions**。
2. 点击 **New repository secret**。
3. **Name**：`TAURI_SIGNING_PRIVATE_KEY`
4. **Value**：粘贴 2.1 步保存的 **Private** 整段（包含 `dcdc8d...` 那一整行）。
5. 保存。

这样 GitHub Actions 在打包时会用该私钥签名更新包，应用内更新时用公钥校验，保证安全。

---

## 三、自动打包：GitHub Actions 说明

项目已包含工作流：`.github/workflows/release.yml`。

### 3.1 触发方式

满足其一即可触发打包：

- **方式 A**：推送或合并到 `release` 分支。
- **方式 B**：推送版本号 tag，例如 `v1.0.0`（`v*` 形式）。
- **方式 C**：在 GitHub 仓库 **Actions** 页选择 **release** 工作流，点击 **Run workflow** 手动运行。

### 3.2 会打包的平台

| 平台        | 说明                    |
|-------------|-------------------------|
| Windows     | windows-latest，x64     |
| macOS       | 两个 job：x86_64 + Arm64（M1 等） |
| Linux       | ubuntu-22.04，x64       |

产物会出现在 **同一个** GitHub Release 的 **Assets** 中（安装包、更新用的 `.sig` 和 `latest.json` 等）。

### 3.3 版本号来源

- Release 的版本号和 tag 名来自 **应用版本**，不是随便写的。
- 应用版本在这里定义：
  - **前端**：`package.json` 的 `version`
  - **Tauri**：`src-tauri/tauri.conf.json` 的 `version` 和 `src-tauri/Cargo.toml` 的 `[package].version`
- 建议三处保持一致，例如都设为 `1.0.0`。  
  发布新版本时，同时改这三处再触发 workflow。

### 3.4 发布一次新版本的推荐步骤

1. **更新版本号**（三处一致）  
   - `package.json` → `"version": "1.0.1"`
   - `src-tauri/tauri.conf.json` → `"version": "1.0.1"`
   - `src-tauri/Cargo.toml` → `version = "1.0.1"`

2. **提交并推送**
   ```bash
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
   git commit -m "chore: bump version to 1.0.1"
   git push origin main
   ```

3. **触发打包（任选一种）**
   - **用 tag**（推荐，便于和版本一一对应）：
     ```bash
     git tag v1.0.1
     git push origin v1.0.1
     ```
   - 或：创建并推送 `release` 分支：  
     `git push origin main:release`
   - 或：在 Actions 里对 **release** workflow 点 **Run workflow**。

4. **等待 Actions 跑完**  
   到 **Actions** 页查看 **release** 工作流，四个 job（Windows、macOS x64、macOS Arm64、Linux）都成功即可。

5. **发布 Draft Release**  
   - 打开仓库 **Releases**。
   - 会有一个 **Draft**（例如 “XY Todo List v1.0.1”）。
   - 检查附件是否包含各平台安装包和 `latest.json`。
   - 点击 **Publish release** 发布。  
   发布后，`/releases/latest/download/latest.json` 会指向当前最新版，应用内更新即可检测到。

---

## 四、应用内如何检查与安装更新

### 4.1 入口

- 打开应用 → **设置**（齿轮）→ **关于我们**。
- 有 **「检查更新」** 按钮；若检测到新版本，会显示 **「发现新版本 vx.x.x」** 和 **「立即更新」**。

### 4.2 流程说明

1. 用户点击 **「检查更新」**  
   - 前端调用 Tauri 的 `@tauri-apps/plugin-updater` 的 `check()`。
   - 应用会请求你在 `tauri.conf.json` 里配置的 `endpoints`（即 `latest.json` 的 URL）。

2. 服务器（GitHub Releases）  
   - `latest.json` 由 CI 在打包时生成并上传到当前 Release。
   - 内容包含：最新版本号、各平台下载地址、签名等。

3. 若当前安装的版本 **低于** `latest.json` 里的版本  
   - 设置页显示「发现新版本 vx.x.x」和「立即更新」。

4. 用户点击 **「立即更新」**  
   - 调用更新对象的 `downloadAndInstall()` 下载并安装。
   - 安装完成后调用 `relaunch()` 重启应用，新版本生效。

### 4.3 相关代码位置（供排查问题）

- 设置页关于/更新 UI：`src/views/SettingsWindow.vue`（关于我们区块）。
- 检查更新：`checkUpdate()`，调用 `@tauri-apps/plugin-updater` 的 `check()`。
- 下载并安装 + 重启：`doUpdate()`，调用 `downloadAndInstall()` 与 `@tauri-apps/plugin-process` 的 `relaunch()`。
- 更新器配置：`src-tauri/tauri.conf.json` → `plugins.updater`（pubkey + endpoints）。

---

## 五、检查清单（上线前自检）

- [ ] GitHub 仓库已创建，代码已推送。
- [ ] 已用 `tauri signer generate` 生成密钥对，公钥已写入 `tauri.conf.json`，私钥已保存。
- [ ] `tauri.conf.json` 中 `plugins.updater.endpoints` 的 URL 已改为你的 `https://github.com/用户名/仓库名/releases/latest/download/latest.json`。
- [ ] 在 GitHub 仓库 Settings → Actions → Secrets 中已添加 `TAURI_SIGNING_PRIVATE_KEY`（私钥整段）。
- [ ] `package.json`、`tauri.conf.json`、`Cargo.toml` 三处版本号一致。
- [ ] 已用 tag 或 `release` 分支或手动 Run workflow 触发过一次 **release** 工作流，且四个平台 job 都成功。
- [ ] 在 Releases 中发布了对应的 Draft，确认 Assets 中有各平台安装包和 `latest.json`。
- [ ] 在已安装的旧版应用里，设置 → 关于我们 → 检查更新，能检测到新版本并完成「立即更新」和重启。

---

## 六、常见问题

**Q：推送 tag 或 release 分支后没有自动打包？**  
- 到 **Actions** 页看是否有 **release** 工作流在跑或报错。
- 确认 **Workflow permissions** 为 “Read and write permissions”（Settings → Actions → General）。

**Q：Release 里没有 `latest.json` 或安装包？**  
- 必须配置并填好 `TAURI_SIGNING_PRIVATE_KEY`，否则 Tauri 不会生成更新相关产物。
- 确认 `src-tauri/tauri.conf.json` 里 `bundle.createUpdaterArtifacts` 为 `true`（当前已为 true）。

**Q：应用内检查更新提示失败或一直最新？**  
- 确认 endpoints 的 URL 与仓库名、用户名一致，且该 Release 已 **Publish**（不是 Draft）。
- 确认当前安装的版本号 **小于** Release 的版本号（三处版本号是否都已改并重新打包）。

**Q：只想打包某一平台？**  
- 可临时修改 `.github/workflows/release.yml` 的 `matrix.include`，注释掉不需要的 platform/args，再推送或手动运行。

---

按上述步骤完成后，即可实现：**代码推送到 GitHub → 自动打 Linux/mac/Windows 包并生成 Release → 应用内通过设置检查并安装新版本**。

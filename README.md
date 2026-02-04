# XY Todo List

支持 **Windows、macOS、Linux** 三端的轻量化桌面 Todo List 应用，基于设计文档 [Todo List 桌面应用设计文档（Tauri + Vue 3 + Axum）.md](./Todo%20List%20桌面应用设计文档（Tauri%20+%20Vue%203%20+%20Axum）.md) 实现。

## 功能概览

- **Todo 任务管理**：新增、编辑、删除、状态切换（待处理/已完成）、列表刷新
- **本地持久化**：任务数据存储在本地，应用重启不丢失
- **移动端扫码编辑**：内置 Axum 局域网 HTTP 服务（默认 8080），生成二维码，手机扫码即可在浏览器中增删改查 Todo，与桌面端实时同步
- **基础 UI 适配**：桌面端两栏布局（任务区 + 二维码区），移动端卡片式列表

## 技术栈

| 模块     | 技术 |
|----------|------|
| 桌面框架 | Tauri 2 |
| 前端     | Vue 3 + Vite + TypeScript + Element Plus |
| 后端服务 | Rust + Axum 0.7（局域网 API + 静态托管） |
| 数据存储 | 本地 SQLite |
| 二维码   | qrcode.vue |
| 自动更新 | tauri-plugin-updater + GitHub Releases |

## 环境要求

- **Node.js** v16+
- **pnpm**（或 npm）
- **Rust** 1.70+
- **Tauri 2** 所需系统依赖（见 [Tauri 文档](https://v2.tauri.app/start/prerequisites/)）

## 快速开始

### 安装依赖

```bash
pnpm install
```

### 开发运行

```bash
pnpm tauri dev
```

- 桌面窗口会打开，加载 Vite 开发服务器（http://localhost:5173）
- 同时会启动 Axum 服务（http://127.0.0.1:8080），桌面端请求会发往 8080 的 API
- 二维码中的地址为 `http://<本机局域网IP>:8080/mobile`，手机与电脑需在同一 WiFi 下

**移动端调试**：先执行一次 `pnpm build` 生成 `dist`，再运行 `pnpm tauri dev`，然后可在电脑浏览器访问 http://127.0.0.1:8080/mobile 或手机扫码访问。

### 打包

```bash
pnpm build
pnpm tauri build
```

产物在 `src-tauri/target/release/` 下，按系统生成对应安装包。

### 图标

若打包报错缺少图标，可在项目根目录执行：

```bash
pnpm tauri icon path/to/your/icon.png
```

会生成所需尺寸并写入 `src-tauri/icons/`。

## 项目结构

```
.
├── src/                    # Vue 3 前端
│   ├── components/         # 桌面端与移动端组件
│   ├── views/             # 桌面页、移动页
│   ├── router/             # Vue Router（/ 桌面，/mobile 移动）
│   ├── utils/              # request 封装（API 请求）
│   ├── App.vue
│   └── main.ts
├── src-tauri/              # Tauri + Rust
│   ├── src/
│   │   ├── lib.rs          # 主进程、状态、持久化、启动 Axum
│   │   ├── main.rs         # 入口
│   │   └── server.rs       # Axum 路由与静态托管
│   ├── tauri.conf.json
│   └── Cargo.toml
├── index.html
├── vite.config.ts
└── package.json
```

## API 说明（Axum）

| 方法   | 路径           | 说明       |
|--------|----------------|------------|
| GET    | /api/todo      | 获取全部任务 |
| POST   | /api/todo      | 新增任务（body: `{ content, status? }`） |
| PATCH  | /api/todo/:id  | 更新状态（body: `{ status }`） |
| DELETE | /api/todo/:id  | 删除任务   |

移动端页面：`GET /mobile` 返回 SPA 的 `index.html`，静态资源由同一服务提供。

## GitHub 托管与自动更新

项目支持完全托管到 GitHub，并通过 **GitHub Actions** 自动打包 **Windows、Linux、macOS** 三端，发布到 **GitHub Releases**；客户端支持**自动检查更新**并展示**更新日志**。

### 1. 推送代码到 GitHub

在 GitHub 创建仓库后，将本地仓库关联并推送：

```bash
git remote add origin https://github.com/你的用户名/你的仓库名.git
git branch -M main
git push -u origin main
```

### 2. 配置更新器（必须）

发布前需完成以下两步，否则更新检查会失败。

**① 生成签名密钥**

在项目根目录执行（将私钥保存到安全位置，**切勿提交到仓库**）：

```bash
pnpm tauri signer generate -w ~/.tauri/xy-todo-list.key
```

会生成 `xy-todo-list.key`（私钥）和 `xy-todo-list.key.pub`（公钥）。

**② 修改 `src-tauri/tauri.conf.json`**

- 将 `plugins.updater.pubkey` 的值改为 **公钥文件内容**（打开 `xy-todo-list.key.pub`，整段复制粘贴，不能是文件路径）。
- 将 `plugins.updater.endpoints` 中的 `https://github.com/OWNER/REPO/...` 里的 `OWNER`、`REPO` 替换为你的 **GitHub 用户名** 和 **仓库名**。

### 3. GitHub Actions 发布

- **触发方式**：推送到 `release` 分支，或推送版本标签（如 `v1.0.1`）。
- **密钥**：在仓库 **Settings → Secrets and variables → Actions** 中新增 Secret：
  - 名称：`TAURI_SIGNING_PRIVATE_KEY`
  - 值：`xy-todo-list.key` 文件的**完整内容**（或私钥字符串）。
- **工作流权限**：在 **Settings → Actions → General** 中，将 **Workflow permissions** 设为 **Read and write permissions**，否则无法创建 Release。

推送 `release` 分支或打 tag 后，Actions 会自动构建三端安装包并创建/更新 GitHub Release，同时生成 `latest.json` 供客户端检查更新。

### 4. 客户端行为

- 应用启动时会**自动检查**是否有新版本（请求配置的 `endpoints`）。
- 若有新版本，会弹出**发现新版本**对话框，展示**版本号**和**更新日志**（来自 Release 的 body），用户可选择**立即更新**或**稍后**。
- 选择立即更新会下载并安装，安装完成后自动重启应用。

更新日志建议写在仓库根目录的 **CHANGELOG.md**，并在创建 GitHub Release 时把当次版本的更新说明粘贴到 Release 的 **Describe** 中，这样用户在更新弹窗中即可看到。

## 许可证

MIT

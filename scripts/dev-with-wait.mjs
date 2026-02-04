/**
 * 后台启动 Vite，轮询直到 localhost:5173 就绪后退出，
 * 供 Tauri beforeDevCommand 使用，避免窗口先于 Vite 打开导致白屏。
 */
import { spawn } from "child_process";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

const child = spawn("pnpm", ["dev"], {
  cwd: root,
  stdio: "inherit",
  shell: true,
  detached: true,
});
child.unref();

async function waitForVite() {
  for (let i = 0; i < 50; i++) {
    try {
      const r = await fetch("http://localhost:5173");
      if (r.ok) {
        console.log("[dev-with-wait] Vite ready.");
        process.exit(0);
      }
    } catch (_) {}
    await new Promise((r) => setTimeout(r, 300));
  }
  process.exit(0);
}
setTimeout(waitForVite, 500);

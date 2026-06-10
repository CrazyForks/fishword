/**
 * M0 可行性探针 — pi extension 最小验证
 *
 * 安装方式（任选其一）：
 *   全局: cp probe.ts ~/.pi/agent/extensions/fishword-probe.ts && pi /reload
 *   临时: pi -e ./probe.ts
 *
 * 验收检查项：
 *   1. /vocab     → 状态栏显示 mock 单词
 *   2. ctrl+alt+v → notify 弹出提示
 *   3. /vocab-overlay → 浮层词卡（overlay 实验性功能）
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Box, Text } from "@earendil-works/pi-tui";

const MOCK_CARD = "📚 cancel  /ˈkænsl/  取消，撤销";

export default function (pi: ExtensionAPI) {
  // ── 验收1：slash command + 状态栏持续显示 ──────────────────────────────
  pi.registerCommand("vocab", {
    description: "Show current vocab card in status bar",
    handler: async (_args, ctx) => {
      ctx.ui.setStatus("fishword", MOCK_CARD);
      ctx.ui.notify("fishword: status bar updated", "info");
    },
  });

  // ── 验收2：快捷键触发 notify ────────────────────────────────────────────
  pi.registerShortcut("ctrl+alt+v", {
    description: "Show vocab card (fishword)",
    handler: async (ctx) => {
      ctx.ui.notify("fishword shortcut works ✓", "info");
    },
  });

  // ── 验收3：overlay 浮层词卡（实验性）──────────────────────────────────
  pi.registerCommand("vocab-overlay", {
    description: "Show vocab overlay card (experimental)",
    handler: async (_args, ctx) => {
      // 最简 TUI 组件：渲染单行文字，按任意键关闭
      await ctx.ui.custom<void>(
        (_tui, _theme, _keybindings, done) => {
          return {
            render(screen, x, y) {
              const label = `  ${MOCK_CARD}  (press any key to close)`;
              new Text(label, x + 1, y + 1).render(screen, x + 1, y + 1);
            },
            handleInput(_key) {
              done(undefined);
              return true;
            },
            width: 60,
            height: 3,
          };
        },
        {
          overlay: true,
          overlayOptions: { anchor: "top-right", width: "60%", margin: 1 },
        }
      );
    },
  });

  // ── 启动确认 ────────────────────────────────────────────────────────────
  pi.on("session_start", async (_event, ctx) => {
    ctx.ui.setStatus("fishword", MOCK_CARD);
  });
}

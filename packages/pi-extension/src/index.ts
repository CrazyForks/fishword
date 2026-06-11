import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { getErrorCode, isErrorResponse, parseCard, runFishword } from "./fishword";
import { showCardOverlay } from "./overlays/card";
import { showDeckSelectorOverlay } from "./overlays/deckSelector";
import { showStatsOverlay } from "./overlays/stats";
import type { DeckItem, Rating, StatsResponse, StatusResponse } from "./types";
import { RATINGS } from "./types";

export default function (pi: ExtensionAPI) {
  let cardOverlayHandle: OverlayHandle | null = null;
  let deckSelectorHandle: OverlayHandle | null = null;
  let statsOverlayHandle: OverlayHandle | null = null;

  function hideCardOverlay(): void {
    cardOverlayHandle?.hide();
    cardOverlayHandle = null;
  }

  function hideDeckSelector(): void {
    deckSelectorHandle?.hide();
    deckSelectorHandle = null;
  }

  function hideStatsOverlay(): void {
    statsOverlayHandle?.hide();
    statsOverlayHandle = null;
  }

  function showCurrentCard(ctx: ExtensionContext, cardResponse: Record<string, unknown>): void {
    hideCardOverlay();
    showCardOverlay(ctx, parseCard(cardResponse), (handle) => {
      cardOverlayHandle = handle;
    });
  }

  async function refreshDisplay(ctx: ExtensionContext): Promise<void> {
    try {
      const res = await runFishword(["current", "--json"]);
      if (isErrorResponse(res)) {
        hideCardOverlay();
        ctx.ui.setStatus("fishword", getErrorCode(res) === "no_active_deck" ? "no active deck" : undefined);
      } else {
        ctx.ui.setStatus("fishword", undefined);
        showCurrentCard(ctx, res);
      }
    } catch {
      hideCardOverlay();
    }
  }

  async function rateAndAdvance(ctx: ExtensionContext, rating: Rating): Promise<void> {
    try {
      const res = await runFishword(["rate", rating, "--json"]);
      if (isErrorResponse(res)) {
        hideCardOverlay();
        ctx.ui.setStatus("fishword", getErrorCode(res) === "no_active_deck" ? "no active deck" : undefined);
      } else {
        const next = res["next"] as Record<string, unknown> | null;
        if (next) {
          ctx.ui.setStatus("fishword", undefined);
          showCurrentCard(ctx, next);
        } else {
          hideCardOverlay();
          ctx.ui.setStatus("fishword", "🎉 all done for today!");
        }
      }
    } catch {
      hideCardOverlay();
    }
  }

  async function openStatsOverlay(ctx: ExtensionContext): Promise<void> {
    let statusRes: Record<string, unknown>;
    let statsRes: Record<string, unknown>;
    try {
      [statusRes, statsRes] = await Promise.all([
        runFishword(["status", "--json"]),
        runFishword(["stats", "--range", "7d", "--json"]),
      ]);
    } catch {
      ctx.ui.notify("无法读取 Fishword 学习统计", "error");
      return;
    }

    if (isErrorResponse(statusRes) || isErrorResponse(statsRes)) {
      const code = getErrorCode(isErrorResponse(statusRes) ? statusRes : statsRes);
      ctx.ui.notify(code === "no_active_deck" ? "请先选择词库" : "暂无可展示的学习统计", "info");
      return;
    }
    if (statsRes["schema"] !== "fishword.protocol.stats.v1" || statusRes["schema"] !== "fishword.protocol.status.v1") {
      ctx.ui.notify("Fishword 统计协议不匹配", "error");
      return;
    }

    hideCardOverlay();
    hideDeckSelector();
    hideStatsOverlay();
    showStatsOverlay(ctx, {
      status: statusRes as StatusResponse,
      stats: statsRes as StatsResponse,
      onHandle: (handle) => {
        statsOverlayHandle = handle;
      },
      onDone: () => {
        statsOverlayHandle = null;
      },
      onRefresh: () => {
        void openStatsOverlay(ctx);
      },
      onClose: () => {
        void refreshDisplay(ctx);
      },
    });
  }

  async function openDeckSelector(ctx: ExtensionContext): Promise<void> {
    let res: Record<string, unknown>;
    try {
      res = await runFishword(["deck", "list", "--json"]);
    } catch {
      ctx.ui.notify("Failed to list decks", "error");
      return;
    }
    if (res["schema"] !== "fishword.protocol.decks.v1") {
      ctx.ui.notify("Failed to list decks", "error");
      return;
    }
    const decks = res["decks"] as DeckItem[];
    if (decks.length === 0) {
      ctx.ui.notify("No decks found. Import a deck first.", "info");
      return;
    }

    hideCardOverlay();
    hideStatsOverlay();

    showDeckSelectorOverlay(ctx, {
      decks,
      activeIndex: decks.findIndex((d) => d.active),
      onHandle: (handle) => {
        deckSelectorHandle = handle;
      },
      onCancel: () => {
        hideDeckSelector();
        void refreshDisplay(ctx);
      },
      onSelect: async (deck) => {
        hideDeckSelector();
        const res = await runFishword(["deck", "use", deck.name, "--json"]);
        if (isErrorResponse(res)) {
          ctx.ui.notify(`Failed: ${getErrorCode(res) ?? "unknown error"}`, "error");
        } else {
          await refreshDisplay(ctx);
          ctx.ui.notify(`Switched to: ${deck.description ?? deck.name}`, "info");
        }
      },
    });
  }

  pi.on("session_start", async (_event, ctx) => {
    await refreshDisplay(ctx);
  });

  pi.registerCommand("fw-deck", {
    description: "Fishword: switch active deck — shows interactive selector",
    handler: async (_args, ctx) => {
      await openDeckSelector(ctx);
    },
  });

  pi.registerCommand("fw-stats", {
    description: "Fishword: show learning stats overlay",
    handler: async (_args, ctx) => {
      await openStatsOverlay(ctx);
    },
  });

  for (const { rating } of RATINGS) {
    pi.registerCommand(`fw-${rating}`, {
      description: `Fishword: rate ${rating} → next card`,
      handler: async (_args, ctx) => {
        await rateAndAdvance(ctx, rating);
      },
    });
  }

  pi.registerShortcut("ctrl+shift+v", {
    description: "Fishword: refresh vocab card",
    handler: async (ctx) => {
      await refreshDisplay(ctx);
    },
  });

  for (const { rating, key } of RATINGS) {
    pi.registerShortcut(key, {
      description: `Fishword: rate ${rating} → next card`,
      handler: async (ctx) => {
        await rateAndAdvance(ctx, rating);
      },
    });
  }
}

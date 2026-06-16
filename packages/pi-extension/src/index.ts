import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { KeyId, OverlayHandle } from "@earendil-works/pi-tui";
import { seedDefaultDecks } from "./defaultDecks.ts";
import { getErrorCode, isErrorResponse, parseCardResponse, runFishword } from "./fishword.ts";
import { showCardOverlay, showDoneOverlay } from "./overlays/card.ts";
import { showCardDetailOverlay } from "./overlays/cardDetail.ts";
import { showDeckSelectorOverlay } from "./overlays/deckSelector.ts";
import { showStatsOverlay } from "./overlays/stats.ts";
import type { CardResponse, DeckItem, Rating, StatsResponse, StatusResponse } from "./types.ts";
import { RATINGS } from "./types.ts";
import { formatStatusLine, formatStatusLineMessage } from "./ui/statusLine.ts";

const HIDE_OR_SUMMON_KEY: KeyId = "ctrl+shift+f";
const CARD_DETAIL_KEY: KeyId = "ctrl+shift+i";

type FishwordAction = {
  command: string;
  description: string;
  shortcut?: KeyId;
  shortcutDescription?: string;
  handler: (ctx: ExtensionContext) => Promise<void> | void;
};

function formatShortcutLabel(key: string): string {
  return key
    .split("+")
    .map((part) => (part.length === 1 ? part.toUpperCase() : part.charAt(0).toUpperCase() + part.slice(1)))
    .join("+");
}

function commandDescription(description: string, shortcut?: string): string {
  return shortcut ? `${description} (${formatShortcutLabel(shortcut)})` : description;
}

export default function (pi: ExtensionAPI) {
  let cardOverlayHandle: OverlayHandle | null = null;
  let cardDetailHandle: OverlayHandle | null = null;
  let deckSelectorHandle: OverlayHandle | null = null;
  let statsOverlayHandle: OverlayHandle | null = null;
  let doneCheckTimer: ReturnType<typeof setInterval> | null = null;
  let isDone = false;
  let isFishwordHidden = false;
  let lastStatusLine: string | undefined;
  let currentCardResponse: CardResponse | null = null;

  function setFishwordStatus(ctx: ExtensionContext, text: string | undefined): void {
    lastStatusLine = text;
    ctx.ui.setStatus("fishword", isFishwordHidden ? undefined : text);
  }

  function applyFishwordHidden(ctx: ExtensionContext): void {
    cardOverlayHandle?.setHidden(isFishwordHidden);
    cardDetailHandle?.setHidden(isFishwordHidden);
    deckSelectorHandle?.setHidden(isFishwordHidden);
    statsOverlayHandle?.setHidden(isFishwordHidden);
    ctx.ui.setStatus("fishword", isFishwordHidden ? undefined : lastStatusLine);
  }

  async function toggleFishwordVisibility(ctx: ExtensionContext): Promise<void> {
    isFishwordHidden = !isFishwordHidden;
    applyFishwordHidden(ctx);

    if (
      !isFishwordHidden &&
      !cardOverlayHandle &&
      !cardDetailHandle &&
      !deckSelectorHandle &&
      !statsOverlayHandle
    ) {
      await refreshDisplay(ctx);
    }
  }

  function hideCardOverlay(): void {
    cardOverlayHandle?.hide();
    cardOverlayHandle = null;
    isDone = false;
    currentCardResponse = null;
    if (doneCheckTimer) {
      clearInterval(doneCheckTimer);
      doneCheckTimer = null;
    }
  }

  function hideCardDetail(): void {
    cardDetailHandle?.hide();
    cardDetailHandle = null;
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
    const parsed = parseCardResponse(cardResponse);
    currentCardResponse = parsed;
    showCardOverlay(ctx, parsed, (handle) => {
      cardOverlayHandle = handle;
      handle.setHidden(isFishwordHidden);
    });
  }

  function showDone(ctx: ExtensionContext): void {
    hideCardOverlay();
    isDone = true;
    showDoneOverlay(ctx, (handle) => {
      cardOverlayHandle = handle;
      handle.setHidden(isFishwordHidden);
    });
    doneCheckTimer = setInterval(() => {
      void (async () => {
        const status = await refreshStatusLine(ctx);
        if (status && status.mode !== "complete") {
          await refreshDisplay(ctx);
        }
      })();
    }, 60_000);
  }

  async function refreshStatusLine(ctx: ExtensionContext): Promise<StatusResponse | null> {
    try {
      const res = await runFishword(["status", "--json"]);
      if (isErrorResponse(res)) {
        const code = getErrorCode(res);
        setFishwordStatus(
          ctx,
          code === "no_active_deck" || code === "no_cards"
            ? formatStatusLineMessage("no-deck")
            : formatStatusLineMessage("unavailable"),
        );
        return null;
      }
      if (res["schema"] !== "fishword.protocol.status.v1") {
        setFishwordStatus(ctx, formatStatusLineMessage("unavailable"));
        return null;
      }
      const status = res as StatusResponse;
      setFishwordStatus(ctx, formatStatusLine(status));
      return status;
    } catch {
      setFishwordStatus(ctx, formatStatusLineMessage("unavailable"));
      return null;
    }
  }

  async function refreshDisplay(ctx: ExtensionContext): Promise<void> {
    const status = await refreshStatusLine(ctx);
    if (status?.mode === "complete") {
      showDone(ctx);
      return;
    }
    try {
      const res = await runFishword(["current", "--json"]);
      if (isErrorResponse(res)) {
        hideCardOverlay();
      } else {
        showCurrentCard(ctx, res);
      }
    } catch {
      hideCardOverlay();
    }
  }

  async function rateAndAdvance(ctx: ExtensionContext, rating: Rating): Promise<void> {
    if (isFishwordHidden) return;
    if (isDone) return;
    try {
      const res = await runFishword(["rate", rating, "--json"]);
      if (isErrorResponse(res)) {
        hideCardOverlay();
        await refreshStatusLine(ctx);
      } else {
        const latestStatus = await refreshStatusLine(ctx);
        const next = res["next"] as Record<string, unknown> | null;
        if (next) {
          showCurrentCard(ctx, next);
        } else if (latestStatus?.mode === "complete") {
          showDone(ctx);
        } else {
          hideCardOverlay();
        }
      }
    } catch {
      hideCardOverlay();
      setFishwordStatus(ctx, formatStatusLineMessage("unavailable"));
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
        handle.setHidden(isFishwordHidden);
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
        handle.setHidden(isFishwordHidden);
      },
      onCancel: () => {
        hideDeckSelector();
        void refreshDisplay(ctx);
      },
      onSelect: async (deck) => {
        hideDeckSelector();
        const res = await runFishword(["deck", "use", String(deck.id), "--json"]);
        if (isErrorResponse(res)) {
          ctx.ui.notify(`Failed: ${getErrorCode(res) ?? "unknown error"}`, "error");
        } else {
          await refreshDisplay(ctx);
          ctx.ui.notify(`Switched to: ${deck.description ?? deck.name}`, "info");
        }
      },
    });
  }

  function openCardDetail(ctx: ExtensionContext): void {
    // Hide card / done overlay before showing detail
    cardOverlayHandle?.hide();
    cardOverlayHandle = null;
    if (doneCheckTimer) {
      clearInterval(doneCheckTimer);
      doneCheckTimer = null;
    }
    hideCardDetail();

    showCardDetailOverlay(ctx, {
      response: currentCardResponse,
      onHandle: (handle) => {
        cardDetailHandle = handle;
        handle.setHidden(isFishwordHidden);
      },
      onClose: () => {
        cardDetailHandle = null;
        // Restore card overlay when user dismisses detail
        if (currentCardResponse) {
          showCardOverlay(ctx, currentCardResponse, (handle) => {
            cardOverlayHandle = handle;
            handle.setHidden(isFishwordHidden);
          });
        }
      },
      onRate: (rating) => {
        void rateInDetail(ctx, rating);
      },
    });
  }

  async function rateInDetail(ctx: ExtensionContext, rating: Rating): Promise<void> {
    if (isFishwordHidden) return;
    if (!currentCardResponse) return;
    cardDetailHandle = null;
    try {
      const res = await runFishword(["rate", rating, "--json"]);
      if (isErrorResponse(res)) {
        await refreshStatusLine(ctx);
        currentCardResponse = null;
        openCardDetail(ctx);
        return;
      }
      await refreshStatusLine(ctx);
      const next = res["next"] as Record<string, unknown> | null;
      if (next) {
        currentCardResponse = parseCardResponse(next);
      } else {
        currentCardResponse = null;
      }
      openCardDetail(ctx);
    } catch {
      setFishwordStatus(ctx, formatStatusLineMessage("unavailable"));
    }
  }

  const fishwordActions: FishwordAction[] = [
    {
      command: "fw-deck",
      description: "Fishword: switch active deck — shows interactive selector",
      handler: openDeckSelector,
    },
    {
      command: "fw-stats",
      description: "Fishword: show learning stats overlay",
      handler: openStatsOverlay,
    },
    {
      command: "fw",
      description: "Fishword: hide or summon review UI",
      shortcut: HIDE_OR_SUMMON_KEY,
      handler: toggleFishwordVisibility,
    },
    ...RATINGS.map(({ rating, key }): FishwordAction => ({
      command: `fw-${rating}`,
      description: `Fishword: rate ${rating} → next card`,
      shortcut: key,
      handler: (ctx) => rateAndAdvance(ctx, rating),
    })),
    {
      command: "fw-detail",
      description: "Fishword: show detailed card info (phonetics, meanings, examples)",
      shortcut: CARD_DETAIL_KEY,
      shortcutDescription: "Fishword: show detailed card info",
      handler: openCardDetail,
    },
  ];

  pi.on("session_start", async (_event, ctx) => {
    await seedDefaultDecks(ctx);
    await refreshDisplay(ctx);
  });

  for (const action of fishwordActions) {
    pi.registerCommand(action.command, {
      description: commandDescription(action.description, action.shortcut),
      handler: async (_args, ctx) => {
        await action.handler(ctx);
      },
    });

    if (action.shortcut) {
      pi.registerShortcut(action.shortcut, {
        description: action.shortcutDescription ?? action.description,
        handler: async (ctx) => {
          await action.handler(ctx);
        },
      });
    }
  }
}

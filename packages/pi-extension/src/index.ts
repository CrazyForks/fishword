import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { SelectList, truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fishwordPath } from "@fishword/cli";

const execAsync = promisify(execFile);

type Card = {
  term: string;
  phonetic?: { us?: string; uk?: string };
  meanings: string[];
  deck: { name: string };
};

type Rating = "again" | "hard" | "good" | "easy";

type DeckItem = {
  id: number;
  name: string;
  description?: string;
  active: boolean;
};

const RATINGS: { rating: Rating; key: string }[] = [
  { rating: "again", key: "ctrl+shift+a" },
  { rating: "hard",  key: "ctrl+shift+h" },
  { rating: "good",  key: "ctrl+shift+g" },
  { rating: "easy",  key: "ctrl+shift+e" },
];

async function runFishword(args: string[]): Promise<Record<string, unknown>> {
  try {
    const { stdout } = await execAsync(fishwordPath, args);
    return JSON.parse(stdout.trim()) as Record<string, unknown>;
  } catch (err: unknown) {
    const execErr = err as { stdout?: string };
    if (execErr.stdout) {
      try {
        return JSON.parse(execErr.stdout.trim()) as Record<string, unknown>;
      } catch { /* fall through */ }
    }
    throw err;
  }
}

function isErrorResponse(res: Record<string, unknown>): boolean {
  return res["schema"] === "fishword.protocol.error.v1";
}

function getErrorCode(res: Record<string, unknown>): string | undefined {
  return (res["error"] as { code?: string })?.code;
}

function parseCard(res: Record<string, unknown>): Card {
  return res["card"] as Card;
}

function formatPhonetic(card: Card): string {
  const raw = card.phonetic?.us || card.phonetic?.uk || "";
  return raw ? `/${raw.replace(/^\/|\/$/g, "")}/` : "";
}

function formatMeaning(card: Card): string {
  return card.meanings
    .map((m) => m.replace(/\s+/g, " ").trim())
    .filter(Boolean)
    .join("；");
}

export default function (pi: ExtensionAPI) {
  let overlayHandle: OverlayHandle | null = null;
  let deckSelectorHandle: OverlayHandle | null = null;

  function hideOverlay(): void {
    overlayHandle?.hide();
    overlayHandle = null;
  }

  function showCardOverlay(ctx: ExtensionContext, card: Card): void {
    hideOverlay();

    const term = card.term;
    const phonetic = formatPhonetic(card);
    const meaning = formatMeaning(card);

    const plainLine1 = term + (phonetic ? "  " + phonetic : "");
    const overlayWidth = Math.max(visibleWidth(plainLine1), visibleWidth(meaning)) + 4;

    void ctx.ui.custom(
      (_tui, theme) => ({
        render(width: number) {
          const innerW = width - 2;
          const l1 = theme.fg("accent", term) + (phonetic ? "  " + theme.fg("dim", phonetic) : "");
          const title = ` ${card.deck.name} `;
          const leftDashes = Math.max(0, innerW - visibleWidth(title) - 2);
          const topBorder =
            theme.fg("border", "╭" + "─".repeat(leftDashes)) +
            theme.fg("accent", title) +
            theme.fg("border", "──╮");
          const row = (content: string) =>
            theme.fg("border", "│") +
            truncateToWidth(content, innerW, "...", true) +
            theme.fg("border", "│");
          return [
            topBorder,
            row(l1),
            row(meaning),
            theme.fg("border", `╰${"─".repeat(innerW)}╯`),
          ];
        },
        invalidate() {},
      }),
      {
        overlay: true,
        overlayOptions: { anchor: "right-center", width: overlayWidth, margin: 1, offsetY: 5 },
        onHandle: (handle) => {
          handle.unfocus();
          overlayHandle = handle;
        },
      },
    );
  }

  async function refreshDisplay(ctx: ExtensionContext): Promise<void> {
    try {
      const res = await runFishword(["current", "--json"]);
      if (isErrorResponse(res)) {
        hideOverlay();
        ctx.ui.setStatus("fishword", getErrorCode(res) === "no_active_deck" ? "no active deck" : undefined);
      } else {
        ctx.ui.setStatus("fishword", undefined);
        showCardOverlay(ctx, parseCard(res));
      }
    } catch {
      hideOverlay();
    }
  }

  async function rateAndAdvance(ctx: ExtensionContext, rating: Rating): Promise<void> {
    try {
      const res = await runFishword(["rate", rating, "--json"]);
      if (isErrorResponse(res)) {
        hideOverlay();
        ctx.ui.setStatus("fishword", getErrorCode(res) === "no_active_deck" ? "no active deck" : undefined);
      } else {
        const next = res["next"] as Record<string, unknown> | null;
        if (next) {
          ctx.ui.setStatus("fishword", undefined);
          showCardOverlay(ctx, parseCard(next));
        } else {
          hideOverlay();
          ctx.ui.setStatus("fishword", "🎉 all done for today!");
        }
      }
    } catch {
      hideOverlay();
    }
  }

  // ── Lifecycle ──────────────────────────────────────────────────────────────
  pi.on("session_start", async (_event, ctx) => {
    await refreshDisplay(ctx);
  });

  // ── Slash commands ─────────────────────────────────────────────────────────
  pi.registerCommand("fw-deck", {
    description: "Fishword: switch active deck — shows interactive selector",
    handler: async (_args, ctx) => {
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

      // 隐藏词卡 overlay，避免与选择器重叠
      hideOverlay();

      const activeIndex = decks.findIndex((d) => d.active);
      const selectItems = decks.map((d) => ({
        value: d.name,
        label: d.name,
        description: d.description,
      }));
      const hint = "Enter to confirm  Esc to cancel";
      const overlayWidth = Math.max(
        ...decks.map((d) => {
          const label = d.description ? `${d.name}  ${d.description}` : d.name;
          return visibleWidth(label);
        }),
        visibleWidth(hint),
      ) + 4;

      void ctx.ui.custom(
        (_tui, theme) => {
          const list = new SelectList(
            selectItems,
            10,
            {
              selectedPrefix: (t) => theme.fg("accent", `▶ ${t}`),
              selectedText: (t) => theme.fg("accent", t),
              description: (t) => theme.fg("dim", t),
              scrollInfo: (t) => theme.fg("dim", t),
              noMatch: (t) => theme.fg("dim", t),
            },
          );
          if (activeIndex >= 0) list.setSelectedIndex(activeIndex);
          list.onSelect = async (item) => {
            deckSelectorHandle?.hide();
            deckSelectorHandle = null;
            const res = await runFishword(["deck", "use", item.value, "--json"]);
            if (isErrorResponse(res)) {
              ctx.ui.notify(`Failed: ${getErrorCode(res) ?? "unknown error"}`, "error");
            } else {
              await refreshDisplay(ctx);
              ctx.ui.notify(`Switched to: ${item.description ?? item.label}`, "info");
            }
          };
          list.onCancel = () => {
            deckSelectorHandle?.hide();
            deckSelectorHandle = null;
            void refreshDisplay(ctx);
          };

          // 包一层边框
          return {
            render(width: number) {
              const w = Math.min(width, overlayWidth);
              const iw = w - 2;
              const rows = list.render(iw);
              const hintLine = truncateToWidth(theme.fg("dim", hint), iw, "...", true);
              return [
                theme.fg("border", "╭" + "─".repeat(iw) + "╮"),
                ...rows.map((row) =>
                  theme.fg("border", "│") + truncateToWidth(row, iw, "...", true) + theme.fg("border", "│")
                ),
                theme.fg("border", "│") + hintLine + theme.fg("border", "│"),
                theme.fg("border", "╰" + "─".repeat(iw) + "╯"),
              ];
            },
            invalidate() { list.invalidate(); },
            handleInput(keyData: string) { list.handleInput(keyData); },
          };
        },
        {
          overlay: true,
          overlayOptions: {
            anchor: "right-center",
            width: overlayWidth,
            margin: 1,
            offsetY: 5,
          },
          onHandle: (handle) => {
            deckSelectorHandle = handle;
          },
        },
      );
    },
  });

  for (const { rating } of RATINGS) {
    pi.registerCommand(`fw-${rating}`, {
      description: `Fishword: rate ${rating} → next card`,
      handler: async (_args, ctx) => { await rateAndAdvance(ctx, rating); },
    });
  }

  // ── Keyboard shortcuts ─────────────────────────────────────────────────────
  pi.registerShortcut("ctrl+shift+v", {
    description: "Fishword: refresh vocab card",
    handler: async (ctx) => { await refreshDisplay(ctx); },
  });

  for (const { rating, key } of RATINGS) {
    pi.registerShortcut(key, {
      description: `Fishword: rate ${rating} → next card`,
      handler: async (ctx) => { await rateAndAdvance(ctx, rating); },
    });
  }
}

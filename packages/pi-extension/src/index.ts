import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { Key, matchesKey, SelectList, truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
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

type StatusResponse = {
  schema: "fishword.protocol.status.v1";
  deck: { id: string; name: string; db_id: number };
  today: { due: number; new_remaining: number; reviewed: number };
};

type DailyStats = {
  date: string;
  reviews: number;
  again: number;
  hard: number;
  good: number;
  easy: number;
  good_or_easy_rate: number | null;
};

type StatsResponse = {
  schema: "fishword.protocol.stats.v1";
  deck: { id: string; name: string; db_id: number };
  range: { kind: "days"; days: number };
  summary: {
    reviews: number;
    reviewed_today: number;
    good_or_easy_rate: number | null;
  };
  series: DailyStats[];
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

function fitCell(content: string, width: number): string {
  const clipped = truncateToWidth(content, width, "...", true);
  return clipped + " ".repeat(Math.max(0, width - visibleWidth(clipped)));
}

function formatPercent(value: number | null): string {
  return value === null ? "--" : `${Math.round(value * 100)}%`;
}

function formatShortDate(date: string): string {
  const parts = date.split("-");
  return parts.length === 3 ? `${parts[1]}-${parts[2]}` : date;
}

function ratingTotals(series: DailyStats[]) {
  return series.reduce(
    (acc, day) => ({
      again: acc.again + day.again,
      hard: acc.hard + day.hard,
      good: acc.good + day.good,
      easy: acc.easy + day.easy,
    }),
    { again: 0, hard: 0, good: 0, easy: 0 },
  );
}

function niceYAxisMax(maxValue: number): number {
  if (maxValue <= 4) return 4;
  if (maxValue <= 10) return 10;
  if (maxValue <= 20) return 20;
  const magnitude = 10 ** Math.floor(Math.log10(maxValue));
  return Math.ceil(maxValue / magnitude) * magnitude;
}

function placeLabel(line: string[], text: string, center: number): void {
  const start = Math.max(0, Math.min(line.length - text.length, center - Math.floor(text.length / 2)));
  for (let i = 0; i < text.length && start + i < line.length; i += 1) {
    line[start + i] = text[i]!;
  }
}

function drawTrendLine(series: DailyStats[], visiblePoints: number, width: number): string[] {
  const plotHeight = 6;
  const rowCount = plotHeight + 1;
  const plotWidth = Math.max(28, width - 8);
  const visibleCount = Math.max(1, Math.min(series.length, visiblePoints));
  const maxReviews = Math.max(0, ...series.map((day) => day.reviews));
  const yMax = niceYAxisMax(maxReviews);
  const midValue = Math.round(yMax / 2);
  const grid = Array.from({ length: rowCount }, () => Array.from({ length: plotWidth }, () => " "));
  const xPositions = series.map((_, index) =>
    series.length === 1
      ? 0
      : Math.round((index * (plotWidth - 1)) / (series.length - 1)),
  );

  for (let x = 0; x < plotWidth; x += 1) {
    grid[plotHeight]![x] = "─";
  }
  for (const x of xPositions) {
    grid[plotHeight]![x] = "┬";
  }

  const points = series.slice(0, visibleCount).map((day, index) => {
    const x = xPositions[index]!;
    const y = plotHeight - Math.round((day.reviews * plotHeight) / yMax);
    return { x, y };
  });

  for (let i = 0; i < points.length - 1; i += 1) {
    const start = points[i]!;
    const end = points[i + 1]!;
    const midX = Math.floor((start.x + end.x) / 2);

    if (start.y === end.y) {
      for (let x = start.x + 1; x < end.x; x += 1) {
        grid[start.y]![x] = "─";
      }
      continue;
    }

    for (let x = start.x + 1; x < midX; x += 1) {
      grid[start.y]![x] = "─";
    }

    const topY = Math.min(start.y, end.y);
    const bottomY = Math.max(start.y, end.y);
    for (let y = topY + 1; y < bottomY; y += 1) {
      grid[y]![midX] = "│";
    }

    if (end.y < start.y) {
      grid[start.y]![midX] = "╯";
      grid[end.y]![midX] = "╭";
    } else {
      grid[start.y]![midX] = "╮";
      grid[end.y]![midX] = "╰";
    }

    for (let x = midX + 1; x < end.x; x += 1) {
      grid[end.y]![x] = "─";
    }
  }

  for (const point of points) {
    grid[point.y]![point.x] = "●";
  }

  const topLabel = yMax.toString().padStart(3, " ");
  const midLabel = midValue > 0 && midValue < yMax ? midValue.toString().padStart(3, " ") : "   ";
  const lines = grid.map((row, index) => {
    if (index === plotHeight) {
      return `  0 └${row.join("")}`;
    }
    const label = index === 0 ? topLabel : index === Math.floor(plotHeight / 2) ? midLabel : "   ";
    return `${label} │${row.join("")}`;
  });

  const labels = Array.from({ length: plotWidth }, () => " ");
  series.forEach((day, index) => {
    placeLabel(labels, formatShortDate(day.date), xPositions[index]!);
  });
  lines.push(`     ${labels.join("")}`);
  return lines;
}

export default function (pi: ExtensionAPI) {
  let overlayHandle: OverlayHandle | null = null;
  let deckSelectorHandle: OverlayHandle | null = null;
  let statsOverlayHandle: OverlayHandle | null = null;
  let statsAnimationTimer: ReturnType<typeof setInterval> | null = null;
  let requestStatsRender: (() => void) | null = null;

  function hideOverlay(): void {
    overlayHandle?.hide();
    overlayHandle = null;
  }

  function hideStatsOverlay(): void {
    if (statsAnimationTimer) {
      clearInterval(statsAnimationTimer);
      statsAnimationTimer = null;
    }
    statsOverlayHandle?.hide();
    statsOverlayHandle = null;
    requestStatsRender = null;
  }

  function hideDeckSelector(): void {
    deckSelectorHandle?.hide();
    deckSelectorHandle = null;
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

  async function showStatsOverlay(ctx: ExtensionContext): Promise<void> {
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

    const status = statusRes as StatusResponse;
    const stats = statsRes as StatsResponse;
    const totals = ratingTotals(stats.series);
    let frame = 1;
    const maxFrame = Math.max(1, stats.series.length);
    const overlayWidth = 72;

    hideOverlay();
    hideDeckSelector();
    hideStatsOverlay();

    void ctx.ui.custom<"close" | "refresh">(
      (tui, theme, _kb, done) => {
        requestStatsRender = () => tui.requestRender();
        return {
          render(width: number) {
            const w = Math.min(width, overlayWidth);
            const iw = w - 2;
            const row = (content: string) =>
              theme.fg("border", "│") +
              fitCell(content, iw) +
              theme.fg("border", "│");
            const separator = theme.fg("border", "├" + "─".repeat(iw) + "┤");
            const title = "Fishword 学习统计";
            const deck = stats.deck.name;
            const titleLine = fitCell(title, Math.max(0, iw - visibleWidth(deck))) + theme.fg("accent", deck);
            const metricWidth = Math.floor(iw / 4);
            const metricLine = [
              `今日评分 ${stats.summary.reviewed_today} 次`,
              `7日评分 ${stats.summary.reviews} 次`,
              `Good+Easy ${formatPercent(stats.summary.good_or_easy_rate)}`,
              `今日新词 ${status.today.new_remaining} 个`,
            ].map((item) => fitCell(item, metricWidth)).join("");
            const chartLines = drawTrendLine(stats.series, frame, iw - 2);
            return [
              theme.fg("border", "╭" + "─".repeat(iw) + "╮"),
              row(titleLine),
              row(`最近 ${stats.range.days} 天`),
              separator,
              row(metricLine),
              separator,
              row("每日评分次数"),
              ...chartLines.map((line) => row(theme.fg("dim", line))),
              separator,
              row("评分分布"),
              row(`Again ${totals.again}      Hard ${totals.hard}      Good ${totals.good}      Easy ${totals.easy}`),
              separator,
              row(theme.fg("dim", "Esc 关闭    r 刷新")),
              theme.fg("border", "╰" + "─".repeat(iw) + "╯"),
            ];
          },
          invalidate() {},
          handleInput(keyData: string) {
            if (keyData.toLowerCase() === "r") {
              done("refresh");
            } else if (matchesKey(keyData, Key.escape)) {
              done("close");
            }
          },
        };
      },
      {
        overlay: true,
        overlayOptions: {
          anchor: "center",
          width: overlayWidth,
          maxHeight: 24,
          margin: 1,
        },
        onHandle: (handle) => {
          statsOverlayHandle = handle;
          statsAnimationTimer = setInterval(() => {
            frame += 1;
            requestStatsRender?.();
            if (frame >= maxFrame && statsAnimationTimer) {
              clearInterval(statsAnimationTimer);
              statsAnimationTimer = null;
            }
          }, 80);
        },
      },
    ).then((result) => {
      hideStatsOverlay();
      if (result === "refresh") {
        void showStatsOverlay(ctx);
      } else {
        void refreshDisplay(ctx);
      }
    });
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

  pi.registerCommand("fw-stats", {
    description: "Fishword: show learning stats overlay",
    handler: async (_args, ctx) => {
      await showStatsOverlay(ctx);
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

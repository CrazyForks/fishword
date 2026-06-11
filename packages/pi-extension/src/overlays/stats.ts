import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { Key, matchesKey, visibleWidth } from "@earendil-works/pi-tui";
import type { StatsResponse, StatusResponse } from "../types.ts";
import { drawTrendLine, ratingTotals } from "../ui/statsChart.ts";
import { fitCell, formatPercent } from "../ui/text.ts";

type StatsOverlayOptions = {
  status: StatusResponse;
  stats: StatsResponse;
  onClose: () => void;
  onRefresh: () => void;
  onHandle: (handle: OverlayHandle) => void;
  onDone: () => void;
};

export function showStatsOverlay(ctx: ExtensionContext, options: StatsOverlayOptions): void {
  const { onClose, onDone, onHandle, onRefresh, stats, status } = options;
  const totals = ratingTotals(stats.series);
  let frame = 1;
  const maxFrame = Math.max(1, stats.series.length);
  const overlayWidth = 72;
  let animationTimer: ReturnType<typeof setInterval> | null = null;
  let requestRender: (() => void) | null = null;

  void ctx.ui.custom<"close" | "refresh">(
    (tui, theme, _kb, done) => {
      requestRender = () => tui.requestRender();
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
        onHandle(handle);
        animationTimer = setInterval(() => {
          frame += 1;
          requestRender?.();
          if (frame >= maxFrame && animationTimer) {
            clearInterval(animationTimer);
            animationTimer = null;
          }
        }, 80);
      },
    },
  ).then((result) => {
    if (animationTimer) {
      clearInterval(animationTimer);
      animationTimer = null;
    }
    onDone();
    if (result === "refresh") {
      onRefresh();
    } else {
      onClose();
    }
  });
}

import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import type { Card } from "../types";
import { formatMeaning, formatPhonetic } from "../ui/text";

export function showCardOverlay(
  ctx: ExtensionContext,
  card: Card,
  onHandle: (handle: OverlayHandle) => void,
): void {
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
        onHandle(handle);
      },
    },
  );
}

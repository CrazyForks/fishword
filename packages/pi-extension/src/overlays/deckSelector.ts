import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { SelectList, truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import type { DeckItem } from "../types.ts";

type DeckSelectorOptions = {
  decks: DeckItem[];
  activeIndex: number;
  onSelect: (deck: DeckItem) => Promise<void>;
  onCancel: () => void;
  onHandle: (handle: OverlayHandle) => void;
};

export function showDeckSelectorOverlay(ctx: ExtensionContext, options: DeckSelectorOptions): void {
  const { decks, activeIndex, onCancel, onHandle, onSelect } = options;
  const decksByName = new Map(decks.map((deck) => [deck.name, deck]));
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
        const deck = decksByName.get(item.value);
        if (deck) {
          await onSelect(deck);
        }
      };
      list.onCancel = onCancel;

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
        invalidate() {
          list.invalidate();
        },
        handleInput(keyData: string) {
          list.handleInput(keyData);
        },
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
      onHandle,
    },
  );
}

import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { OverlayHandle, OverlayOptions, TUI } from "@earendil-works/pi-tui";
import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import type { CardResponse, SelectionReason } from "../types.ts";
import { formatMeaning, formatPhonetic } from "../ui/text.ts";


const DONE_MESSAGES = [
  "公司是老板的，身体是自己的，记得按时吃饭喔。",
  "恭喜你，在工位上偷偷变强了一点。",
  "这波不亏：工资照拿，单词照背。",
  "单词已清空，建议切回代码界面装作刚才在思考架构。",
  "知识已入库，疲惫请出栈。",
  "你在摸鱼，但鱼也在学习。",
  "今日已偷偷进步，建议保持神秘。",
  "你刚刚完成了一次合法的精神出逃。",
  "单词搞定，接下来请带薪呼吸三十秒。",
  "老板看不见你的努力，但单词记住了。",
];
const DONE_MESSAGE_CYCLE_MS = 5 * 60 * 1_000;
const CARD_OVERLAY_PREFERRED_ROW = 16;
const CARD_OVERLAY_RIGHT_MARGIN = 2;

type OverlayLayout = {
  row: number;
  visible: boolean;
};

function topAnchoredOverlayLayout(
  tui: TUI,
  overlayHeight: number,
  termWidth = tui.terminal.columns,
  termHeight = tui.terminal.rows,
): OverlayLayout {
  const baseHeight = tui.render(termWidth).length;
  const viewportTop = Math.max(0, baseHeight - termHeight);
  const preferredViewportRow = CARD_OVERLAY_PREFERRED_ROW - viewportTop;
  const maxRow = termHeight - overlayHeight;
  const visible = preferredViewportRow >= 0 && maxRow >= 0;
  const row = Math.max(0, Math.min(preferredViewportRow, maxRow));
  return { row, visible };
}

function createCardOverlayOptions(width: number): OverlayOptions {
  return {
    anchor: "top-right",
    width,
    row: CARD_OVERLAY_PREFERRED_ROW,
    margin: { right: CARD_OVERLAY_RIGHT_MARGIN },
    nonCapturing: true,
  };
}

function updateCardOverlayRow(
  tui: TUI,
  options: OverlayOptions,
  overlayHeight: number,
): OverlayLayout {
  const layout = topAnchoredOverlayLayout(tui, overlayHeight);
  options.row = layout.row;
  return layout;
}

function hideCardOverlayWhenPastTop(tui: TUI, options: OverlayOptions, overlayHeight: number): void {
  options.visible = (termWidth, termHeight) =>
    topAnchoredOverlayLayout(tui, overlayHeight, termWidth, termHeight).visible;
}

export function showDoneOverlay(
  ctx: ExtensionContext,
  onHandle: (handle: OverlayHandle) => void,
): void {
  const title = " DONE ";
  let messageIndex = Math.floor(Math.random() * DONE_MESSAGES.length);
  const overlayWidth = Math.max(...DONE_MESSAGES.map(visibleWidth), visibleWidth(title)) + 4;
  const overlayOptions = createCardOverlayOptions(overlayWidth);
  let requestRender: (() => void) | null = null;

  const cycleTimer = setInterval(() => {
    messageIndex = (messageIndex + 1) % DONE_MESSAGES.length;
    requestRender?.();
  }, DONE_MESSAGE_CYCLE_MS);

  void ctx.ui.custom(
    (tui, theme) => {
      const overlayHeight = 3;
      hideCardOverlayWhenPastTop(tui, overlayOptions, overlayHeight);
      requestRender = () => tui.requestRender();
      return {
        render(_width: number) {
          const message = DONE_MESSAGES[messageIndex]!;
          const innerW = Math.max(visibleWidth(message) + 2, visibleWidth(title));
          const leftPad = " ".repeat(Math.max(0, overlayWidth - innerW - 2));
          const leftDashes = Math.max(0, innerW - visibleWidth(title) - 2);
          const topBorder =
            leftPad +
            theme.fg("border", "╭" + "─".repeat(leftDashes)) +
            theme.fg("accent", title) +
            theme.fg("border", "──╮");
          const row = (content: string) =>
            leftPad +
            theme.fg("border", "│") +
            " " +
            truncateToWidth(content, innerW - 2, "...") +
            " " +
            theme.fg("border", "│");
          const contentLines = [row(theme.fg("dim", message))];
          updateCardOverlayRow(tui, overlayOptions, overlayHeight);
          const lines = [topBorder, ...contentLines, leftPad + theme.fg("border", `╰${"─".repeat(innerW)}╯`)];
          return lines;
        },
        invalidate() {},
      };
    },
    {
      overlay: true,
      overlayOptions,
      onHandle: (handle) => {
        handle.unfocus();
        onHandle(handle);
      },
    },
  ).then(() => {
    clearInterval(cycleTimer);
  });
}

function selectionTitle(reason: SelectionReason): string {
  switch (reason) {
    case "due":
      return "DUE";
    case "new":
      return "NEW";
  }
}

export function showCardOverlay(
  ctx: ExtensionContext,
  response: CardResponse,
  onHandle: (handle: OverlayHandle) => void,
): void {
  const card = response.card;
  const term = card.term;
  const phonetic = formatPhonetic(card);
  const meaning = formatMeaning(card);
  const title = ` ${selectionTitle(response.selection.reason)} `;

  const plainLine1 = term + (phonetic ? "  " + phonetic : "");
  const overlayWidth = Math.max(visibleWidth(plainLine1), visibleWidth(meaning), visibleWidth(title)) + 4;
  const overlayOptions = createCardOverlayOptions(overlayWidth);

  void ctx.ui.custom(
    (tui, theme) => {
      const overlayHeight = 4;
      hideCardOverlayWhenPastTop(tui, overlayOptions, overlayHeight);

      return {
        render(width: number) {
          const innerW = width - 2;
          updateCardOverlayRow(tui, overlayOptions, overlayHeight);
          const l1 = theme.fg("accent", term) + (phonetic ? "  " + theme.fg("dim", phonetic) : "");
          const leftDashes = Math.max(0, innerW - visibleWidth(title) - 2);
          const topBorder =
            theme.fg("border", "╭" + "─".repeat(leftDashes)) +
            theme.fg("accent", title) +
            theme.fg("border", "──╮");
          const row = (content: string) =>
            theme.fg("border", "│") +
            truncateToWidth(content, innerW, "...", true) +
            theme.fg("border", "│");
          const lines = [
            topBorder,
            row(l1),
            row(meaning),
            theme.fg("border", `╰${"─".repeat(innerW)}╯`),
          ];
          return lines;
        },
        invalidate() {},
      };
    },
    {
      overlay: true,
      overlayOptions,
      onHandle: (handle) => {
        handle.unfocus();
        onHandle(handle);
      },
    },
  );
}

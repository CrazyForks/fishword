import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import type { Card } from "../types";

export function formatPhonetic(card: Card): string {
  const raw = card.phonetic?.us || card.phonetic?.uk || "";
  return raw ? `/${raw.replace(/^\/|\/$/g, "")}/` : "";
}

export function formatMeaning(card: Card): string {
  return card.meanings
    .map((m) => m.replace(/\s+/g, " ").trim())
    .filter(Boolean)
    .join("；");
}

export function fitCell(content: string, width: number): string {
  const clipped = truncateToWidth(content, width, "...", true);
  return clipped + " ".repeat(Math.max(0, width - visibleWidth(clipped)));
}

export function formatPercent(value: number | null): string {
  return value === null ? "--" : `${Math.round(value * 100)}%`;
}

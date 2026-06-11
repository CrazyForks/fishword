import type { KeyId } from "@earendil-works/pi-tui";

export type Card = {
  id: string;
  term: string;
  language: string;
  phonetic?: { us?: string | null; uk?: string | null; other?: string[] };
  meanings: Array<string | { part_of_speech: string; definition: string; example?: string }>;
  deck: { id: string; name: string; db_id: number };
  tags: string[];
  source?: { name: string; license?: string | null } | null;
};

export type SelectionReason = "due" | "new" | "mature";

export type CardResponse = {
  schema: "fishword.protocol.current.v1" | "fishword.protocol.next.v1";
  card: Card;
  selection: { reason: SelectionReason };
};

export type Rating = "again" | "hard" | "good" | "easy";

export type DeckItem = {
  id: number;
  name: string;
  description?: string;
  active: boolean;
};

export type StatusResponse = {
  schema: "fishword.protocol.status.v1";
  deck: { id: string; name: string; db_id: number };
  mode: "review" | "complete" | "empty";
  today: { due: number; new_remaining: number; new_today: number; reviewed: number };
  display: { plain: string; compact: string; statusline: string };
  next_action: { kind: "review" | "none"; label: string; command: string };
};

export type DailyStats = {
  date: string;
  reviews: number;
  again: number;
  hard: number;
  good: number;
  easy: number;
  good_or_easy_rate: number | null;
};

export type StatsResponse = {
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

export const RATINGS: { rating: Rating; key: KeyId }[] = [
  { rating: "again", key: "ctrl+shift+a" },
  { rating: "hard", key: "ctrl+shift+h" },
  { rating: "good", key: "ctrl+shift+g" },
  { rating: "easy", key: "ctrl+shift+e" },
];

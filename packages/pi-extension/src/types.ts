import type { KeyId } from "@earendil-works/pi-tui";

export type Card = {
  term: string;
  phonetic?: { us?: string; uk?: string };
  meanings: string[];
  deck: { name: string };
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
  today: { due: number; new_remaining: number; reviewed: number };
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

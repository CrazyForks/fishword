import { fileURLToPath } from "node:url";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { getErrorCode, isErrorResponse, runFishword, runFishwordText } from "./fishword.ts";
import type { DeckItem } from "./types.ts";

type DefaultDeck = {
  name: string;
  description: string;
  asset: string;
};

type CardListResponse = {
  schema: "fishword.protocol.card_list.v1";
  pagination: { total: number };
};

const DEFAULT_DECKS: DefaultDeck[] = [
  {
    name: "CET-4",
    description: "大学英语四级",
    asset: "cet4.jsonl",
  },
  {
    name: "CET-6",
    description: "大学英语六级",
    asset: "cet6.jsonl",
  },
  {
    name: "TOEFL",
    description: "托福",
    asset: "toefl.jsonl",
  },
];

function assetPath(fileName: string): string {
  return fileURLToPath(new URL(`../assets/dicts/kajweb/${fileName}`, import.meta.url));
}

async function listDecks(): Promise<DeckItem[]> {
  const res = await runFishword(["deck", "list", "--json"]);
  if (res["schema"] !== "fishword.protocol.decks.v1") {
    throw new Error("unexpected deck list response");
  }
  return res["decks"] as DeckItem[];
}

async function createDeck(defaultDeck: DefaultDeck): Promise<DeckItem> {
  const res = await runFishword([
    "deck",
    "create",
    defaultDeck.name,
    "--description",
    defaultDeck.description,
    "--json",
  ]);

  if (isErrorResponse(res)) {
    const code = getErrorCode(res);
    if (code !== "deck_already_exists") {
      throw new Error(`failed to create default deck ${defaultDeck.name}: ${code ?? "unknown error"}`);
    }
    const existing = (await listDecks()).find((deck) => deck.name === defaultDeck.name);
    if (existing) return existing;
    throw new Error(`default deck exists but cannot be listed: ${defaultDeck.name}`);
  }

  const deck = res["deck"] as { id: number; name: string; description?: string };
  return {
    id: deck.id,
    name: deck.name,
    description: deck.description,
    active: false,
  };
}

async function cardCount(deckId: number): Promise<number> {
  const res = await runFishword(["card", "list", "--deck", String(deckId), "--page-size", "1", "--json"]);
  if (res["schema"] !== "fishword.protocol.card_list.v1") {
    throw new Error(`unexpected card list response for deck ${deckId}`);
  }
  return (res as CardListResponse).pagination.total;
}

async function ensureDefaultDeck(defaultDeck: DefaultDeck, decks: DeckItem[]): Promise<DeckItem> {
  const deck = decks.find((item) => item.name === defaultDeck.name) ?? (await createDeck(defaultDeck));
  if ((await cardCount(deck.id)) === 0) {
    // Keep the explicit create + --deck flow so bundled decks retain descriptions.
    await runFishwordText([
      "import",
      "jsonl",
      assetPath(defaultDeck.asset),
      "--deck",
      String(deck.id),
      "--duplicates",
      "merge",
    ]);
  }
  return deck;
}

export async function seedDefaultDecks(ctx: ExtensionContext): Promise<void> {
  try {
    await runFishwordText(["init"]);

    let decks = await listDecks();
    const hadActiveDeck = decks.some((deck) => deck.active);
    const seededDecks: DeckItem[] = [];

    for (const defaultDeck of DEFAULT_DECKS) {
      const deck = await ensureDefaultDeck(defaultDeck, decks);
      seededDecks.push(deck);
      decks = await listDecks();
    }

    if (!hadActiveDeck && seededDecks[0]) {
      await runFishword(["deck", "use", String(seededDecks[0].id), "--json"]);
    }
  } catch {
    ctx.ui.notify("Fishword 默认词库初始化失败", "error");
  }
}

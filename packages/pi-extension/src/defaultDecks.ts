import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import {
  describeFishwordError,
  getErrorCode,
  getErrorMessage,
  isErrorResponse,
  runFishword,
  runFishwordText,
} from "./fishword.ts";
import type { DeckItem } from "./types.ts";

type DefaultDeck = {
  catalogId: string;
  name: string;
};

type CardListResponse = {
  schema: "fishword.protocol.card_list.v1";
  pagination: { total: number };
};

const DEFAULT_DECKS: DefaultDeck[] = [
  {
    catalogId: "kajweb:cet4",
    name: "CET-4",
  },
  {
    catalogId: "kajweb:cet6",
    name: "CET-6",
  },
  {
    catalogId: "kajweb:toefl",
    name: "TOEFL",
  },
];

async function listDecks(): Promise<DeckItem[]> {
  const res = await runFishword(["deck", "list", "--json"]);
  if (res["schema"] !== "fishword.protocol.decks.v1") {
    throw new Error("unexpected deck list response");
  }
  return res["decks"] as DeckItem[];
}

async function cardCount(deckId: number): Promise<number> {
  const res = await runFishword(["card", "list", "--deck", String(deckId), "--page-size", "1", "--json"]);
  if (res["schema"] !== "fishword.protocol.card_list.v1") {
    throw new Error(`unexpected card list response for deck ${deckId}`);
  }
  return (res as CardListResponse).pagination.total;
}

async function fetchCatalogDeck(defaultDeck: DefaultDeck): Promise<DeckItem> {
  const res = await runFishword([
    "catalog",
    "fetch",
    defaultDeck.catalogId,
    "--duplicates",
    "merge",
    "--json",
  ]);

  if (isErrorResponse(res)) {
    throw new Error(
      `failed to fetch default deck ${defaultDeck.catalogId}: ${getErrorMessage(res) ?? getErrorCode(res) ?? "unknown error"}`,
    );
  }
  if (res["schema"] !== "fishword.protocol.catalog_fetch.v1") {
    throw new Error(`unexpected catalog fetch response for ${defaultDeck.catalogId}`);
  }

  const imported = res["import"] as { deck_id?: number };
  const decks = await listDecks();
  const deck = decks.find((item) => item.id === imported.deck_id);
  if (!deck) {
    throw new Error(`fetched default deck cannot be listed: ${defaultDeck.catalogId}`);
  }
  return deck;
}

async function ensureDefaultDeck(defaultDeck: DefaultDeck, decks: DeckItem[]): Promise<DeckItem> {
  const existing = decks.find((item) => item.name === defaultDeck.name);
  if (existing && (await cardCount(existing.id)) > 0) {
    return existing;
  }

  return fetchCatalogDeck(defaultDeck);
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
  } catch (err) {
    ctx.ui.notify(`Fishword 默认词库初始化失败: ${describeFishwordError(err)}`, "error");
  }
}

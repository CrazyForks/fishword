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

const DEFAULT_DECK_NOTIFY_DELAY_MS = 700;

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

async function existingSeededDeck(defaultDeck: DefaultDeck, decks: DeckItem[]): Promise<DeckItem | null> {
  const existing = decks.find((item) => item.name === defaultDeck.name);
  if (existing && (await cardCount(existing.id)) > 0) {
    return existing;
  }

  return null;
}

function createDefaultDeckNotifier(ctx: ExtensionContext): {
  progress: (message: string) => void;
  success: (message: string) => void;
  cancel: () => void;
} {
  let hasNotified = false;
  let latestMessage = "Fishword 正在准备默认词库...";
  let timer: ReturnType<typeof setTimeout> | null = null;

  function clearTimer(): void {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  return {
    progress(message: string) {
      latestMessage = message;
      if (hasNotified) {
        ctx.ui.notify(message, "info");
        return;
      }
      if (!timer) {
        timer = setTimeout(() => {
          timer = null;
          hasNotified = true;
          ctx.ui.notify(latestMessage, "info");
        }, DEFAULT_DECK_NOTIFY_DELAY_MS);
      }
    },
    success(message: string) {
      clearTimer();
      if (hasNotified) {
        ctx.ui.notify(message, "info");
      }
    },
    cancel() {
      clearTimer();
    },
  };
}

export async function seedDefaultDecks(ctx: ExtensionContext): Promise<void> {
  const notifier = createDefaultDeckNotifier(ctx);

  try {
    await runFishwordText(["init"]);

    let decks = await listDecks();
    const hadActiveDeck = decks.some((deck) => deck.active);
    const seededDecks: DeckItem[] = [];

    for (let i = 0; i < DEFAULT_DECKS.length; i++) {
      const defaultDeck = DEFAULT_DECKS[i];
      let deck = await existingSeededDeck(defaultDeck, decks);

      if (!deck) {
        notifier.progress(`Fishword 正在准备默认词库 ${i + 1}/${DEFAULT_DECKS.length}: ${defaultDeck.name}`);
        deck = await fetchCatalogDeck(defaultDeck);
      }

      seededDecks.push(deck);
      decks = await listDecks();
    }

    if (!hadActiveDeck && seededDecks[0]) {
      await runFishword(["deck", "use", String(seededDecks[0].id), "--json"]);
    }

    notifier.success("Fishword 默认词库已准备好");
  } catch (err) {
    notifier.cancel();
    ctx.ui.notify(`Fishword 默认词库初始化失败: ${describeFishwordError(err)}`, "error");
  }
}

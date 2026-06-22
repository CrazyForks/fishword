use std::str::FromStr;

use anyhow::{Context, Result};
use fishword_core::{
    card::Source,
    error::Error as CoreError,
    importer::{import_jsonl_str, DuplicateStrategy},
};

use crate::protocol::{
    CatalogDeckEntry, CatalogFetchResponse, CatalogListResponse, ImportResponse,
    CATALOG_FETCH_SCHEMA, CATALOG_LIST_SCHEMA, IMPORT_SCHEMA,
};
use serde::Deserialize;

use crate::{
    args::CatalogCmd,
    util::{cmd_error, open_storage, print_human, print_json},
};

const DEFAULT_CATALOG_URL: &str = "https://chenggou1.github.io/fishword/catalog/catalog.json";

fn catalog_url() -> String {
    std::env::var("FISHWORD_CATALOG_URL").unwrap_or_else(|_| DEFAULT_CATALOG_URL.to_string())
}

// ── Internal deserialization types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CatalogJson {
    decks: Vec<CatalogEntryJson>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntryJson {
    id: String,
    slug: String,
    source_id: String,
    name: String,
    description: Option<String>,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default)]
    word_count: u64,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    source: Option<Source>,
    url: String,
    #[serde(default)]
    size_bytes: u64,
}

fn default_language() -> String {
    "en".to_string()
}

// ── Command entry point ──────────────────────────────────────────────────────

pub fn cmd_catalog(sub: CatalogCmd) -> Result<()> {
    match sub {
        CatalogCmd::List { json } => catalog_list(json),
        CatalogCmd::Fetch {
            catalog_id,
            duplicates,
            json,
        } => catalog_fetch(&catalog_id, &duplicates, json),
    }
}

// ── List ─────────────────────────────────────────────────────────────────────

fn catalog_list(json: bool) -> Result<()> {
    let catalog = fetch_catalog(json)?;
    if json {
        let decks = catalog
            .decks
            .into_iter()
            .map(|e| CatalogDeckEntry {
                id: e.id,
                slug: e.slug,
                source_id: e.source_id,
                name: e.name,
                description: e.description,
                language: e.language,
                word_count: e.word_count,
                tags: e.tags,
                source: e.source,
                url: e.url,
                size_bytes: e.size_bytes,
            })
            .collect();
        return print_json(&CatalogListResponse {
            schema: CATALOG_LIST_SCHEMA,
            decks,
        });
    }
    print_human(format!(
        "{:<16} {:<12} {:>6}  {:<18} Tags",
        "ID", "Name", "Words", "Source"
    ));
    print_human("-".repeat(72));
    for e in &catalog.decks {
        let source = e.source.as_ref().map(|s| s.name.as_str()).unwrap_or("-");
        print_human(format!(
            "{:<16} {:<12} {:>6}  {:<18} {}",
            e.id,
            e.name,
            e.word_count,
            source,
            e.tags.join(" ")
        ));
    }
    Ok(())
}

// ── Fetch ────────────────────────────────────────────────────────────────────

fn catalog_fetch(catalog_id: &str, duplicates: &str, json: bool) -> Result<()> {
    let duplicate_strategy = DuplicateStrategy::from_str(duplicates).map_err(|_| {
        cmd_error(
            json,
            "invalid_duplicate_strategy",
            &format!("invalid --duplicates value '{duplicates}'"),
        )
    })?;

    let catalog = fetch_catalog(json)?;
    let entry = match catalog.decks.into_iter().find(|e| e.id == catalog_id) {
        Some(e) => e,
        None => return Err(cmd_error(
            json,
            "deck_not_found",
            &format!("Catalog deck '{catalog_id}' not found. Run `fishword catalog list` to see available decks."),
        )),
    };

    let jsonl_body = fetch_url(&entry.url, json).with_context(|| {
        format!(
            "failed to download catalog deck '{catalog_id}' from {}",
            entry.url
        )
    })?;

    let cards = import_jsonl_str(&jsonl_body)
        .with_context(|| format!("failed to parse JSONL for catalog deck '{catalog_id}'"))?;

    let storage = open_storage()?;

    // Match by catalog_id, not by display name — a deck previously fetched from this
    // same catalog entry is safe to merge into. A deck that merely *happens* to share
    // the display name (e.g. one the user created by hand) is never touched silently;
    // see the AlreadyExists arm below for that case.
    let (db_deck, summary) = if let Some(existing) = storage
        .get_deck_by_catalog_id(catalog_id)
        .context("failed to look up existing catalog deck")?
    {
        let s = storage
            .import_cards(existing.id, &cards, duplicate_strategy)
            .context("failed to import cards")?;
        (existing, s)
    } else {
        match storage.import_cards_into_new_deck_with_catalog_id(
            &entry.name,
            entry.description.as_deref(),
            &cards,
            duplicate_strategy,
            Some(catalog_id),
        ) {
            Ok(result) => result,
            Err(CoreError::AlreadyExists(_)) => {
                let message = format!(
                    "A deck named '{}' already exists but was not created by `fishword catalog fetch {catalog_id}`. \
                     Refusing to merge into it to avoid mixing unrelated data. \
                     Rename or delete that deck, then retry.",
                    entry.name
                );
                return Err(cmd_error(json, "deck_name_conflict", &message));
            }
            Err(e) => return Err(anyhow::anyhow!(e)).context("failed to write imported cards"),
        }
    };

    // Auto-activate if no active deck is set.
    if storage
        .get_active_deck_id()
        .context("failed to read active deck")?
        .is_none()
    {
        storage
            .set_active_deck_id(Some(db_deck.id))
            .context("failed to set active deck")?;
    }

    let import_response = ImportResponse {
        schema: IMPORT_SCHEMA,
        deck_id: db_deck.id,
        deck: db_deck.name.clone(),
        input: summary.input_count,
        inserted: summary.inserted,
        updated: summary.updated,
        merged: summary.merged,
        skipped: summary.skipped,
    };

    if json {
        return print_json(&CatalogFetchResponse {
            schema: CATALOG_FETCH_SCHEMA,
            catalog_id: catalog_id.to_string(),
            slug: entry.slug,
            source_id: entry.source_id,
            name: db_deck.name,
            import: import_response,
        });
    }

    print_human(format!(
        "Fetched deck={} input={} inserted={} updated={} merged={} skipped={}",
        import_response.deck,
        import_response.input,
        import_response.inserted,
        import_response.updated,
        import_response.merged,
        import_response.skipped,
    ));
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn fetch_catalog(json_errors: bool) -> Result<CatalogJson> {
    let url = catalog_url();
    let body = fetch_url(&url, json_errors)
        .with_context(|| format!("failed to fetch catalog from {url}"))?;
    serde_json::from_str::<CatalogJson>(&body).with_context(|| "failed to parse catalog JSON")
}

fn fetch_url(url: &str, json_errors: bool) -> Result<String> {
    match ureq::get(url).call() {
        Ok(resp) => resp
            .into_body()
            .read_to_string()
            .context("failed to read response body"),
        Err(e) => Err(cmd_error(
            json_errors,
            "network_error",
            &format!("Network request failed: {e}"),
        )),
    }
}

use std::str::FromStr;

use anyhow::{Context, Result};
use fishword_core::{
    deck::Deck,
    error::Error as CoreError,
    importer::{
        import_anki_tsv_file, import_csv_file, import_jsonl_file, import_qwerty_file,
        DuplicateStrategy,
    },
    protocol::{ImportResponse, IMPORT_SCHEMA},
};

use crate::{
    args::{ImportArgs, ImportCmd},
    util::{open_storage, print_json},
};

pub fn cmd_import(command: ImportCmd) -> Result<()> {
    let (args, cards) = match command {
        ImportCmd::Qwerty(args) => {
            let deck = import_qwerty_file(&args.path, "", None)
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck.cards)
        }
        ImportCmd::Csv(args) => {
            let deck = import_csv_file(&args.path, "", None)
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck.cards)
        }
        ImportCmd::Jsonl(args) => {
            let deck = import_jsonl_file(&args.path, "", None)
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck.cards)
        }
        ImportCmd::AnkiTsv(args) => {
            let deck = import_anki_tsv_file(&args.path, "", None)
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck.cards)
        }
    };
    persist_import(args, cards)
}

fn persist_import(args: ImportArgs, cards: Vec<fishword_core::importer::ImportCard>) -> Result<()> {
    let duplicate_strategy = DuplicateStrategy::from_str(&args.duplicates)
        .with_context(|| format!("invalid --duplicates value '{}'", args.duplicates))?;
    let storage = open_storage()?;
    let (db_deck, summary) = match (args.deck, args.name.as_deref()) {
        (Some(deck_id), None) => {
            let db_deck = storage
                .get_deck_by_id(deck_id)
                .with_context(|| format!("failed to read deck {}", deck_id))?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "deck not found: {}. Run `fishword deck create <name>` first.",
                        deck_id
                    )
                })?;
            let summary = storage
                .import_cards(deck_id, &cards, duplicate_strategy)
                .context("failed to write imported cards")?;
            (db_deck, summary)
        }
        (None, Some(name)) => import_into_new_deck(&storage, name, &cards, duplicate_strategy)?,
        _ => anyhow::bail!("pass exactly one of --deck or --name"),
    };
    if storage
        .get_active_deck_id()
        .context("failed to read active deck")?
        .is_none()
    {
        storage
            .set_active_deck_id(Some(db_deck.id))
            .context("failed to set active deck")?;
    }
    if args.json {
        return print_json(&ImportResponse {
            schema: IMPORT_SCHEMA,
            deck_id: db_deck.id,
            deck: db_deck.name,
            input: summary.input_count,
            inserted: summary.inserted,
            updated: summary.updated,
            merged: summary.merged,
            skipped: summary.skipped,
        });
    }
    println!(
        "Imported deck={} input={} inserted={} updated={} merged={} skipped={}",
        db_deck.name,
        summary.input_count,
        summary.inserted,
        summary.updated,
        summary.merged,
        summary.skipped
    );
    Ok(())
}

fn import_into_new_deck(
    storage: &fishword_core::storage::Storage,
    name: &str,
    cards: &[fishword_core::importer::ImportCard],
    duplicate_strategy: DuplicateStrategy,
) -> Result<(Deck, fishword_core::importer::ImportSummary)> {
    match storage.import_cards_into_new_deck(name, None, cards, duplicate_strategy) {
        Ok(result) => Ok(result),
        Err(CoreError::AlreadyExists(_)) => anyhow::bail!(
            "Deck already exists: {name}. Use `fishword deck list` to find its id, then import with `--deck <id>`."
        ),
        Err(e) => Err(anyhow::anyhow!(e)).context("failed to write imported cards"),
    }
}

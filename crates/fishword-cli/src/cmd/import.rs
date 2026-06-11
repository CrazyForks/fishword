use std::str::FromStr;

use anyhow::{Context, Result};
use fishword_core::importer::{
    import_anki_tsv_file, import_csv_file, import_jsonl_file, import_qwerty_file, DuplicateStrategy,
};

use crate::{
    args::{ImportArgs, ImportCmd},
    util::open_storage,
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

fn persist_import(
    args: ImportArgs,
    cards: Vec<fishword_core::importer::ImportCard>,
) -> Result<()> {
    let duplicate_strategy = DuplicateStrategy::from_str(&args.duplicates)
        .with_context(|| format!("invalid --duplicates value '{}'", args.duplicates))?;
    let storage = open_storage()?;
    let db_deck = storage
        .get_deck_by_id(args.deck)
        .with_context(|| format!("failed to read deck {}", args.deck))?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "deck not found: {}. Run `fishword deck create <name>` first.",
                args.deck
            )
        })?;
    let summary = storage
        .import_cards(args.deck, &cards, duplicate_strategy)
        .context("failed to write imported cards")?;
    if storage
        .get_active_deck_id()
        .context("failed to read active deck")?
        .is_none()
    {
        storage
            .set_active_deck_id(Some(args.deck))
            .context("failed to set active deck")?;
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

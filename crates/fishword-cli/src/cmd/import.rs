use std::str::FromStr;

use anyhow::{Context, Result};
use fishword_core::importer::{
    import_anki_tsv_file, import_csv_file, import_jsonl_file, import_qwerty_file,
    DuplicateStrategy, ImportDeck,
};

use crate::{
    args::{ImportArgs, ImportCmd},
    util::open_storage,
};

pub fn cmd_import(command: ImportCmd) -> Result<()> {
    let (args, deck) = match command {
        ImportCmd::Qwerty(args) => {
            let deck = import_qwerty_file(&args.path, &args.deck, args.name.as_deref())
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck)
        }
        ImportCmd::Csv(args) => {
            let deck = import_csv_file(&args.path, &args.deck, args.name.as_deref())
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck)
        }
        ImportCmd::Jsonl(args) => {
            let deck = import_jsonl_file(&args.path, &args.deck, args.name.as_deref())
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck)
        }
        ImportCmd::AnkiTsv(args) => {
            let deck = import_anki_tsv_file(&args.path, &args.deck, args.name.as_deref())
                .with_context(|| format!("failed to parse {}", args.path.display()))?;
            (args, deck)
        }
    };
    persist_import(args, deck)
}

fn persist_import(args: ImportArgs, deck: ImportDeck) -> Result<()> {
    let duplicate_strategy = DuplicateStrategy::from_str(&args.duplicates)
        .with_context(|| format!("invalid --duplicates value '{}'", args.duplicates))?;
    let storage = open_storage()?;
    let summary = storage
        .import_cards(
            &deck.deck_id,
            deck.deck_name.as_deref(),
            &deck.cards,
            duplicate_strategy,
        )
        .context("failed to write imported cards")?;
    if storage
        .get_active_deck_id()
        .context("failed to read active deck")?
        .is_none()
    {
        storage
            .set_active_deck_id(Some(summary.deck_id))
            .context("failed to set active deck")?;
    }
    println!(
        "Imported deck={} input={} inserted={} updated={} merged={} skipped={}",
        summary.deck_name,
        summary.input_count,
        summary.inserted,
        summary.updated,
        summary.merged,
        summary.skipped
    );
    Ok(())
}

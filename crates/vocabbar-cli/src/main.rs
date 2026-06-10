use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use vocabbar_core::{
    card::{Card, Rating},
    importer::{
        import_anki_tsv_file, import_csv_file, import_jsonl_file, import_qwerty_file,
        DuplicateStrategy, ImportDeck,
    },
    scheduler::Scheduler,
    selector::Selector,
    storage::Storage,
};

#[derive(Parser)]
#[command(name = "vocabbar", about = "Vocabulary flashcard CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Initialize the database at the platform-default path.
    Init,
    /// Manage decks.
    Deck {
        #[command(subcommand)]
        sub: DeckCmd,
    },
    /// Manage cards.
    Card {
        #[command(subcommand)]
        sub: CardCmd,
    },
    /// Import vocabulary decks.
    Import {
        #[command(subcommand)]
        sub: ImportCmd,
    },
    /// Show the current selected card.
    Current,
    /// Select the next card without writing review history.
    Next,
    /// Rate the current card: again, hard, good, easy.
    Rate {
        /// Review rating.
        rating: String,
    },
}

#[derive(Subcommand)]
enum DeckCmd {
    /// List all decks.
    List,
}

#[derive(Subcommand)]
enum CardCmd {
    /// List cards in a deck.
    List {
        /// Deck name (e.g. cet4)
        #[arg(long)]
        deck: String,
    },
}

#[derive(Subcommand)]
enum ImportCmd {
    /// Import Qwerty Learner JSON.
    Qwerty(ImportArgs),
    /// Import minimal CSV.
    Csv(ImportArgs),
    /// Import vocabbar.deck.v1 JSONL.
    Jsonl(ImportArgs),
    /// Import Anki exported TSV.
    AnkiTsv(ImportArgs),
}

#[derive(Parser)]
struct ImportArgs {
    /// Input file path.
    path: PathBuf,
    /// Deck id/name used by the local database.
    #[arg(long)]
    deck: String,
    /// Human-readable deck name/description.
    #[arg(long)]
    name: Option<String>,
    /// Duplicate strategy: merge, skip, overwrite, keep.
    #[arg(long, default_value = "merge")]
    duplicates: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Init => cmd_init(),
        Cmd::Deck { sub: DeckCmd::List } => cmd_deck_list(),
        Cmd::Card {
            sub: CardCmd::List { deck },
        } => cmd_card_list(&deck),
        Cmd::Import { sub } => cmd_import(sub),
        Cmd::Current => cmd_current(),
        Cmd::Next => cmd_next(),
        Cmd::Rate { rating } => cmd_rate(&rating),
    }
}

fn open_storage() -> Result<Storage> {
    let path = Storage::default_path().context("cannot determine data directory")?;
    Storage::open(&path).with_context(|| format!("cannot open database at {}", path.display()))
}

fn cmd_init() -> Result<()> {
    let path = Storage::default_path().context("cannot determine data directory")?;
    Storage::open(&path)
        .with_context(|| format!("cannot initialize database at {}", path.display()))?;
    println!("Initialized: {}", path.display());
    Ok(())
}

fn cmd_deck_list() -> Result<()> {
    let storage = open_storage()?;
    let decks = storage.list_decks().context("failed to list decks")?;
    if decks.is_empty() {
        println!("No decks found.");
        return Ok(());
    }
    println!("{:<6}  {:<20}  DESCRIPTION", "ID", "NAME");
    println!("{}", "-".repeat(50));
    for d in decks {
        println!(
            "{:<6}  {:<20}  {}",
            d.id,
            d.name,
            d.description.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

fn cmd_card_list(deck: &str) -> Result<()> {
    let storage = open_storage()?;
    let cards = storage
        .list_cards_by_deck(deck)
        .with_context(|| format!("failed to list cards for deck '{deck}'"))?;
    if cards.is_empty() {
        println!("No cards in deck '{deck}'.");
        return Ok(());
    }
    println!("{:<6}  {:<20}  MEANINGS", "ID", "WORD");
    println!("{}", "-".repeat(60));
    for c in cards {
        let meanings_summary = c
            .meanings
            .iter()
            .map(|m| format!("[{}] {}", m.part_of_speech, m.definition))
            .collect::<Vec<_>>()
            .join("; ");
        println!("{:<6}  {:<20}  {}", c.id, c.word, meanings_summary);
    }
    Ok(())
}

fn cmd_current() -> Result<()> {
    let storage = open_storage()?;
    match Selector::select_current(&storage).context("failed to select current card")? {
        Some(selected) => print_card(&selected.card),
        None => println!("No cards found. Import a deck first."),
    }
    Ok(())
}

fn cmd_next() -> Result<()> {
    let storage = open_storage()?;
    match Selector::select_next(&storage).context("failed to select next card")? {
        Some(selected) => print_card(&selected.card),
        None => println!("No cards found. Import a deck first."),
    }
    Ok(())
}

fn cmd_rate(value: &str) -> Result<()> {
    let rating = value
        .parse::<Rating>()
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("invalid rating '{value}', expected again/hard/good/easy"))?;
    let storage = open_storage()?;
    let card_id = storage
        .get_current_card_id()
        .context("failed to read current card")?
        .context("No current card. Run `vocabbar next` first.")?;
    let review = Scheduler::review(&storage, card_id, rating).context("failed to rate card")?;
    let card = storage
        .get_card_by_id(card_id)
        .context("failed to read reviewed card")?
        .context("Reviewed card disappeared")?;
    println!(
        "Rated {} as {}. due={} scheduled_days={}",
        card.word, review.rating, review.due, review.scheduled_days
    );
    Ok(())
}

fn print_card(card: &Card) {
    let meanings = card
        .meanings
        .iter()
        .map(|meaning| meaning.definition.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let pronunciations = card
        .pronunciations
        .iter()
        .map(|pronunciation| pronunciation.notation.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if pronunciations.is_empty() {
        println!("{} - {}", card.word, meanings);
    } else {
        println!("{} {} - {}", card.word, pronunciations, meanings);
    }
}

fn cmd_import(command: ImportCmd) -> Result<()> {
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

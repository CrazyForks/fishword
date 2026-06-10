use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use vocabbar_core::{
    card::Rating,
    importer::{
        import_anki_tsv_file, import_csv_file, import_jsonl_file, import_qwerty_file,
        DuplicateStrategy, ImportDeck,
    },
    protocol::{render_card, CardResponse, ErrorResponse, RateResponse, TextFormat},
    scheduler::Scheduler,
    selector::{SelectedCard, Selector},
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
    Current(CardOutputArgs),
    /// Select the next card without writing review history.
    Next(CardOutputArgs),
    /// Rate the current card: again, hard, good, easy.
    Rate(RateArgs),
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

#[derive(Parser)]
struct CardOutputArgs {
    /// Emit stable JSON protocol output.
    #[arg(long)]
    json: bool,
    /// Human-readable output format: plain, compact, status.
    #[arg(long, default_value = "plain")]
    format: String,
}

#[derive(Parser)]
struct RateArgs {
    /// Review rating: again, hard, good, easy.
    rating: String,
    /// Emit stable JSON protocol output.
    #[arg(long)]
    json: bool,
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
        Cmd::Current(args) => cmd_current(&args),
        Cmd::Next(args) => cmd_next(&args),
        Cmd::Rate(args) => cmd_rate(&args),
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

fn cmd_current(args: &CardOutputArgs) -> Result<()> {
    let storage = open_storage()?;
    match Selector::select_current(&storage).context("failed to select current card")? {
        Some(selected) => print_selected_card(&storage, &selected, args, true)?,
        None if args.json => exit_json_error("no_cards", "No cards found. Import a deck first."),
        None => println!("No cards found. Import a deck first."),
    }
    Ok(())
}

fn cmd_next(args: &CardOutputArgs) -> Result<()> {
    let storage = open_storage()?;
    match Selector::select_next(&storage).context("failed to select next card")? {
        Some(selected) => print_selected_card(&storage, &selected, args, false)?,
        None if args.json => exit_json_error("no_cards", "No cards found. Import a deck first."),
        None => println!("No cards found. Import a deck first."),
    }
    Ok(())
}

fn cmd_rate(args: &RateArgs) -> Result<()> {
    let rating = args
        .rating
        .parse::<Rating>()
        .map_err(anyhow::Error::msg)
        .with_context(|| {
            format!(
                "invalid rating '{}', expected again/hard/good/easy",
                args.rating
            )
        })?;
    let storage = open_storage()?;
    let Some(card_id) = storage
        .get_current_card_id()
        .context("failed to read current card")?
    else {
        if args.json {
            exit_json_error(
                "no_current_card",
                "No current card. Run `vocabbar next` first.",
            );
        }
        anyhow::bail!("No current card. Run `vocabbar next` first.");
    };
    let review = Scheduler::review(&storage, card_id, rating).context("failed to rate card")?;
    let card = storage
        .get_card_by_id(card_id)
        .context("failed to read reviewed card")?
        .context("Reviewed card disappeared")?;
    if args.json {
        let deck = storage
            .get_deck_by_id(card.deck_id)
            .context("failed to read deck")?
            .context("Reviewed card deck disappeared")?;
        let progress = storage
            .progress_counts(20)
            .context("failed to read progress")?;
        print_json(&RateResponse::new(&card, &deck, &review, progress))?;
    } else {
        println!(
            "Rated {} as {}. due={} scheduled_days={}",
            card.word, review.rating, review.due, review.scheduled_days
        );
    }
    Ok(())
}

fn print_selected_card(
    storage: &Storage,
    selected: &SelectedCard,
    args: &CardOutputArgs,
    current: bool,
) -> Result<()> {
    let deck = storage
        .get_deck_by_id(selected.card.deck_id)
        .context("failed to read deck")?
        .context("Selected card deck disappeared")?;
    let progress = storage
        .progress_counts(20)
        .context("failed to read progress")?;
    let response = if current {
        CardResponse::current(selected, &deck, progress)
    } else {
        CardResponse::next(selected, &deck, progress)
    };
    if args.json {
        print_json(&response)?;
    } else {
        let format = args
            .format
            .parse::<TextFormat>()
            .map_err(anyhow::Error::msg)
            .with_context(|| {
                format!(
                    "invalid --format '{}', expected plain/compact/status",
                    args.format
                )
            })?;
        println!("{}", render_card(&response, format));
    }
    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

fn exit_json_error(code: &str, message: &str) -> ! {
    println!(
        "{}",
        serde_json::to_string(&ErrorResponse::new(code, message))
            .expect("serializing protocol error should not fail")
    );
    std::process::exit(2);
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

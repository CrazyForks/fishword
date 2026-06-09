use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use vocabbar_core::storage::Storage;

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Init => cmd_init(),
        Cmd::Deck { sub: DeckCmd::List } => cmd_deck_list(),
        Cmd::Card { sub: CardCmd::List { deck } } => cmd_card_list(&deck),
    }
}

fn open_storage() -> Result<Storage> {
    let path = Storage::default_path().context("cannot determine data directory")?;
    Storage::open(&path)
        .with_context(|| format!("cannot open database at {}", path.display()))
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
        println!("No decks found. Run `vocabbar init` first.");
        return Ok(());
    }
    println!("{:<6}  {:<20}  {}", "ID", "NAME", "DESCRIPTION");
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
    println!("{:<6}  {:<20}  {}", "ID", "WORD", "MEANINGS");
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

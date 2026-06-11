use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fishword", about = "Vocabulary flashcard CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
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
    /// Show today's learning status.
    Status(StatusArgs),
    /// Show review statistics.
    Stats(StatsArgs),
    /// Rate the current card: again, hard, good, easy.
    Rate(RateArgs),
}

#[derive(Subcommand)]
pub enum DeckCmd {
    /// List all decks.
    List {
        /// Emit stable JSON protocol output.
        #[arg(long)]
        json: bool,
    },
    /// Set the active deck used by current/next/rate.
    Use {
        /// Deck name (e.g. cet4)
        deck: String,
        /// Emit stable JSON protocol output.
        #[arg(long)]
        json: bool,
    },
    /// Show the active deck.
    Current,
}

#[derive(Subcommand)]
pub enum CardCmd {
    /// List cards in a deck.
    List(CardListArgs),
}

#[derive(Subcommand)]
pub enum ImportCmd {
    /// Import Qwerty Learner JSON.
    Qwerty(ImportArgs),
    /// Import minimal CSV.
    Csv(ImportArgs),
    /// Import fishword.deck.v1 JSONL.
    Jsonl(ImportArgs),
    /// Import Anki exported TSV.
    AnkiTsv(ImportArgs),
}

#[derive(Parser)]
pub struct ImportArgs {
    /// Input file path.
    pub path: PathBuf,
    /// Deck id/name used by the local database.
    #[arg(long)]
    pub deck: String,
    /// Human-readable deck name/description.
    #[arg(long)]
    pub name: Option<String>,
    /// Duplicate strategy: merge, skip, overwrite, keep.
    #[arg(long, default_value = "merge")]
    pub duplicates: String,
}

#[derive(Parser)]
pub struct CardListArgs {
    /// Deck name (e.g. cet4)
    #[arg(long)]
    pub deck: String,
    /// Page number, starting from 1.
    #[arg(long, default_value_t = 1)]
    pub page: i64,
    /// Number of cards per page.
    #[arg(long, default_value_t = 50)]
    pub page_size: i64,
    /// Emit stable JSON protocol output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser)]
pub struct CardOutputArgs {
    /// Emit stable JSON protocol output.
    #[arg(long)]
    pub json: bool,
    /// Deck name used as this command's learning scope.
    #[arg(long)]
    pub deck: Option<String>,
    /// Human-readable output format: plain, compact, status.
    #[arg(long, default_value = "plain")]
    pub format: String,
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Emit stable JSON protocol output.
    #[arg(long)]
    pub json: bool,
    /// Deck name used as this command's learning scope.
    #[arg(long)]
    pub deck: Option<String>,
    /// Human-readable output format: plain, compact, statusline.
    #[arg(long, default_value = "plain")]
    pub format: String,
}

#[derive(Parser)]
pub struct StatsArgs {
    /// Emit stable JSON protocol output.
    #[arg(long)]
    pub json: bool,
    /// Deck name used as this command's learning scope.
    #[arg(long)]
    pub deck: Option<String>,
    /// Time range. The first implementation supports 7d.
    #[arg(long, default_value = "7d")]
    pub range: String,
}

#[derive(Parser)]
pub struct RateArgs {
    /// Review rating: again, hard, good, easy.
    pub rating: String,
    /// Deck name used as this command's learning scope.
    #[arg(long)]
    pub deck: Option<String>,
    /// Emit stable JSON protocol output.
    #[arg(long)]
    pub json: bool,
}

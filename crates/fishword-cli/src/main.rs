mod args;
mod cmd;
mod util;

use anyhow::Result;
use args::{CardCmd, Cli, Cmd};
use clap::Parser;
use cmd::{
    cmd_card_list, cmd_current, cmd_deck, cmd_import, cmd_init, cmd_rate, cmd_stats, cmd_status,
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Init => cmd_init(),
        Cmd::Deck { sub } => cmd_deck(sub),
        Cmd::Card {
            sub: CardCmd::List { deck },
        } => cmd_card_list(&deck),
        Cmd::Import { sub } => cmd_import(sub),
        Cmd::Current(args) => cmd_current(&args),
        Cmd::Status(args) => cmd_status(&args),
        Cmd::Stats(args) => cmd_stats(&args),
        Cmd::Rate(args) => cmd_rate(&args),
    }
}

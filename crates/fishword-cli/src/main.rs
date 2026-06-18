mod args;
mod cmd;
mod protocol;
mod util;

use args::{CardCmd, Cli, Cmd};
use clap::Parser;
use cmd::{
    cmd_card_list, cmd_catalog, cmd_current, cmd_deck, cmd_import, cmd_init, cmd_rate, cmd_stats,
    cmd_status,
};
use util::render_cli_error;

fn main() {
    let cli = Cli::parse();
    let json = cli.command.wants_json();
    let result = match cli.command {
        Cmd::Init => cmd_init(),
        Cmd::Deck { sub } => cmd_deck(sub),
        Cmd::Card {
            sub: CardCmd::List(args),
        } => cmd_card_list(&args),
        Cmd::Import { sub } => cmd_import(sub),
        Cmd::Current(args) => cmd_current(&args),
        Cmd::Status(args) => cmd_status(&args),
        Cmd::Stats(args) => cmd_stats(&args),
        Cmd::Rate(args) => cmd_rate(&args),
        Cmd::Catalog { sub } => cmd_catalog(sub),
    };
    if let Err(err) = result {
        render_cli_error(json, err);
    }
}

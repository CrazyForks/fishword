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
use util::{render_clap_error_json, render_cli_error};

fn main() {
    let raw_args = std::env::args_os().collect::<Vec<_>>();
    let raw_wants_json = raw_args.iter().any(|arg| arg == "--json");
    let cli = match Cli::try_parse_from(raw_args) {
        Ok(cli) => cli,
        Err(err) => {
            if raw_wants_json && !is_display_request(err.kind()) {
                render_clap_error_json(err);
            }
            err.exit();
        }
    };
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

fn is_display_request(kind: clap::error::ErrorKind) -> bool {
    matches!(
        kind,
        clap::error::ErrorKind::DisplayHelp
            | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | clap::error::ErrorKind::DisplayVersion
    )
}

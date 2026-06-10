use anyhow::{Context, Result};
use fishword_core::{
    deck::Deck,
    protocol::{render_card, CardResponse, ErrorResponse, TextFormat},
    selector::SelectedCard,
    storage::Storage,
};

use crate::args::CardOutputArgs;

pub fn open_storage() -> Result<Storage> {
    let path = Storage::default_path().context("cannot determine data directory")?;
    Storage::open(&path).with_context(|| format!("cannot open database at {}", path.display()))
}

pub fn resolve_deck_scope(
    storage: &Storage,
    deck_name: Option<&str>,
    json_errors: bool,
) -> Result<Option<Deck>> {
    if let Some(deck_name) = deck_name {
        return match storage
            .get_deck_by_name(deck_name)
            .with_context(|| format!("failed to read deck '{deck_name}'"))?
        {
            Some(deck) => Ok(Some(deck)),
            None if json_errors => {
                exit_json_error("deck_not_found", &format!("Deck not found: {deck_name}"));
            }
            None => anyhow::bail!("Deck not found: {deck_name}"),
        };
    }

    if let Some(deck) = storage
        .get_active_deck()
        .context("failed to read active deck")?
    {
        return Ok(Some(deck));
    }

    let decks = storage.list_decks().context("failed to list decks")?;
    match decks.as_slice() {
        [] => Ok(None),
        [deck] => {
            storage
                .set_active_deck_id(Some(deck.id))
                .context("failed to set active deck")?;
            Ok(Some(deck.clone()))
        }
        _ if json_errors => {
            exit_json_error(
                "no_active_deck",
                "Multiple decks found. Run `fishword deck use <deck>` or pass `--deck <deck>`.",
            );
        }
        _ => anyhow::bail!(
            "Multiple decks found. Run `fishword deck use <deck>` or pass `--deck <deck>`."
        ),
    }
}

pub fn print_selected_card(
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
        .progress_counts_by_deck(deck.id, 20)
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

pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

pub fn exit_json_error(code: &str, message: &str) -> ! {
    println!(
        "{}",
        serde_json::to_string(&ErrorResponse::new(code, message))
            .expect("serializing protocol error should not fail")
    );
    std::process::exit(2);
}

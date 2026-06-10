use anyhow::{Context, Result};
use fishword_core::{
    card::Rating,
    protocol::RateResponse,
    scheduler::Scheduler,
    selector::Selector,
};

use crate::{
    args::{CardOutputArgs, RateArgs},
    util::{exit_json_error, open_storage, print_json, print_selected_card, resolve_deck_scope},
};

pub fn cmd_current(args: &CardOutputArgs) -> Result<()> {
    let storage = open_storage()?;
    let Some(deck) = resolve_deck_scope(&storage, args.deck.as_deref(), args.json)? else {
        if args.json {
            exit_json_error("no_cards", "No cards found. Import a deck first.");
        }
        println!("No cards found. Import a deck first.");
        return Ok(());
    };
    match Selector::select_current_in_deck(&storage, deck.id)
        .context("failed to select current card")?
    {
        Some(selected) => print_selected_card(&storage, &selected, args, true)?,
        None if args.json => exit_json_error("no_cards", "No cards found. Import a deck first."),
        None => println!("No cards found. Import a deck first."),
    }
    Ok(())
}

pub fn cmd_rate(args: &RateArgs) -> Result<()> {
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
    let Some(scope_deck) = resolve_deck_scope(&storage, args.deck.as_deref(), args.json)? else {
        if args.json {
            exit_json_error("no_cards", "No cards found. Import a deck first.");
        }
        anyhow::bail!("No cards found. Import a deck first.");
    };
    let Some(card_id) = storage
        .get_current_card_id()
        .context("failed to read current card")?
    else {
        if args.json {
            exit_json_error(
                "no_current_card",
                "No current card. Run `fishword current` first.",
            );
        }
        anyhow::bail!("No current card. Run `fishword current` first.");
    };
    let card = storage
        .get_card_by_id(card_id)
        .context("failed to read current card")?
        .context("Current card disappeared")?;
    if card.deck_id != scope_deck.id {
        if args.json {
            exit_json_error(
                "no_current_card",
                "No current card in this deck. Run `fishword current` first.",
            );
        }
        anyhow::bail!("No current card in this deck. Run `fishword current` first.");
    }
    let review = Scheduler::review(&storage, card_id, rating).context("failed to rate card")?;
    let next = Selector::select_next_in_deck(&storage, scope_deck.id)
        .context("failed to select next card after rating")?;
    let next_deck = if let Some(ref selected) = next {
        storage
            .get_deck_by_id(selected.card.deck_id)
            .context("failed to read next card deck")?
    } else {
        None
    };
    if args.json {
        let progress = storage
            .progress_counts_by_deck(scope_deck.id, 20)
            .context("failed to read progress")?;
        let next_ref = next.as_ref().zip(next_deck.as_ref()).map(|(s, d)| (s, d));
        print_json(&RateResponse::new(&card, &scope_deck, &review, progress, next_ref))?;
    } else {
        println!(
            "Rated {} as {}. due={} scheduled_days={}",
            card.word, review.rating, review.due, review.scheduled_days
        );
        if let Some(ref selected) = next {
            println!("Next: {}", selected.card.word);
        }
    }
    Ok(())
}

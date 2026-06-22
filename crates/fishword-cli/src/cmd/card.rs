use anyhow::{Context, Result};

use crate::protocol::CardListResponse;

use crate::{
    args::CardListArgs,
    util::{cmd_error, open_storage, print_human, print_json},
};

pub fn cmd_card_list(args: &CardListArgs) -> Result<()> {
    if args.page < 1 {
        return Err(cmd_error(
            args.json,
            "invalid_page",
            "Page must be greater than or equal to 1.",
        ));
    }
    if args.page_size < 1 {
        return Err(cmd_error(
            args.json,
            "invalid_page_size",
            "Page size must be greater than or equal to 1.",
        ));
    }
    let storage = open_storage()?;
    let deck = match storage
        .get_deck_by_id(args.deck)
        .with_context(|| format!("failed to read deck {}", args.deck))?
    {
        Some(deck) => deck,
        None => {
            return Err(cmd_error(
                args.json,
                "deck_not_found",
                &format!("Deck not found: {}", args.deck),
            ));
        }
    };
    let total = storage
        .card_count_by_deck(deck.id)
        .with_context(|| format!("failed to count cards for deck '{}'", args.deck))?;
    let offset = (args.page - 1) * args.page_size;
    let cards = storage
        .list_cards_by_deck_paginated(deck.id, args.page_size, offset)
        .with_context(|| format!("failed to list cards for deck '{}'", args.deck))?;
    if args.json {
        return print_json(CardListResponse::new(
            &deck,
            cards,
            args.page,
            args.page_size,
            total,
        ));
    }
    if total == 0 {
        print_human(format!("No cards in deck '{}'.", args.deck));
        return Ok(());
    }
    let page_count = (total + args.page_size - 1) / args.page_size;
    if cards.is_empty() {
        print_human(format!(
            "No cards on page {} for deck '{}' ({} cards, {} pages).",
            args.page, args.deck, total, page_count
        ));
        return Ok(());
    }
    print_human(format!(
        "Deck: {} ({})  Page {}/{}  Total {}",
        deck.name, deck.id, args.page, page_count, total
    ));
    print_human(format!("{:<6}  {:<20}  MEANINGS", "ID", "WORD"));
    print_human("-".repeat(60));
    for c in cards {
        let meanings_summary = c
            .meanings
            .iter()
            .map(|m| {
                if m.part_of_speech.is_empty() {
                    m.definition.clone()
                } else {
                    format!("[{}] {}", m.part_of_speech, m.definition)
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        print_human(format!("{:<6}  {:<20}  {}", c.id, c.word, meanings_summary));
    }
    Ok(())
}

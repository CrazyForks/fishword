use anyhow::{Context, Result};
use fishword_core::error::Error as CoreError;

use crate::protocol::{
    CardListResponse, DeckCreateResponse, DeckDeleteResponse, DeckListResponse, DeckRenameResponse,
    DeckUseResponse,
};

use crate::{
    args::{CardListArgs, DeckCmd},
    util::{cmd_error, open_storage, print_human, print_json},
};

pub fn cmd_deck(sub: DeckCmd) -> Result<()> {
    match sub {
        DeckCmd::List { json } => list(json),
        DeckCmd::Create {
            name,
            description,
            json,
        } => create(&name, description.as_deref(), json),
        DeckCmd::Use { deck, json } => use_deck(deck, json),
        DeckCmd::Delete { id, json } => delete(id, json),
        DeckCmd::Rename { id, new_name, json } => rename(id, &new_name, json),
        DeckCmd::Current => current(),
    }
}

fn list(json: bool) -> Result<()> {
    let storage = open_storage()?;
    let decks = storage.list_decks().context("failed to list decks")?;
    let active_deck_id = storage
        .get_active_deck_id()
        .context("failed to read active deck")?;
    if json {
        return print_json(&DeckListResponse::new(decks, active_deck_id));
    }
    if decks.is_empty() {
        print_human("No decks found.");
        return Ok(());
    }
    print_human(format!(
        "{:<6}  {:<6}  {:<20}  {:<14}  DESCRIPTION",
        "ACTIVE", "ID", "NAME", "CATALOG"
    ));
    print_human("-".repeat(72));
    for d in decks {
        print_human(format!(
            "{:<6}  {:<6}  {:<20}  {:<14}  {}",
            if Some(d.id) == active_deck_id {
                "*"
            } else {
                ""
            },
            d.id,
            d.name,
            d.catalog_id.as_deref().unwrap_or("-"),
            d.description.as_deref().unwrap_or("")
        ));
    }
    Ok(())
}

fn create(name: &str, description: Option<&str>, json: bool) -> Result<()> {
    let storage = open_storage()?;
    match storage.insert_deck(name, description) {
        Ok(deck) => {
            if json {
                print_json(&DeckCreateResponse::new(&deck))?;
            } else {
                print_human(format!("Created deck: {} (id={})", deck.name, deck.id));
            }
        }
        Err(CoreError::AlreadyExists(_)) => {
            return Err(cmd_error(
                json,
                "deck_already_exists",
                &format!("Deck already exists: {name}"),
            ));
        }
        Err(e) => return Err(anyhow::anyhow!(e)),
    }
    Ok(())
}

fn use_deck(deck_id: i64, json: bool) -> Result<()> {
    let storage = open_storage()?;
    let deck = match storage
        .get_deck_by_id(deck_id)
        .with_context(|| format!("failed to read deck {deck_id}"))?
    {
        Some(d) => d,
        None => {
            return Err(cmd_error(
                json,
                "deck_not_found",
                &format!("Deck not found: {deck_id}"),
            ))
        }
    };
    storage
        .set_active_deck_id(Some(deck.id))
        .with_context(|| format!("failed to set active deck {deck_id}"))?;
    storage
        .set_current_card_id(None)
        .with_context(|| "failed to clear current card on deck switch")?;
    if json {
        print_json(&DeckUseResponse::new(&deck))?;
    } else {
        print_human(format!("Active deck: {} ({})", deck.name, deck.id));
    }
    Ok(())
}

fn delete(id: i64, json: bool) -> Result<()> {
    let storage = open_storage()?;
    match storage.delete_deck(id) {
        Ok(deck) => {
            if json {
                print_json(&DeckDeleteResponse::new(&deck))?;
            } else {
                print_human(format!("Deleted deck: {} (id={})", deck.name, deck.id));
            }
        }
        Err(CoreError::NotFound(_)) => {
            return Err(cmd_error(
                json,
                "deck_not_found",
                &format!("Deck not found: {id}"),
            ));
        }
        Err(e) => return Err(anyhow::anyhow!(e)),
    }
    Ok(())
}

fn rename(id: i64, new_name: &str, json: bool) -> Result<()> {
    let storage = open_storage()?;
    match storage.update_deck_name(id, new_name) {
        Ok(deck) => {
            if json {
                print_json(&DeckRenameResponse::new(&deck))?;
            } else {
                print_human(format!("Renamed deck {} to: {}", id, deck.name));
            }
        }
        Err(CoreError::NotFound(_)) => {
            return Err(cmd_error(
                json,
                "deck_not_found",
                &format!("Deck not found: {id}"),
            ));
        }
        Err(CoreError::AlreadyExists(_)) => {
            return Err(cmd_error(
                json,
                "deck_already_exists",
                &format!("Deck already exists: {new_name}"),
            ));
        }
        Err(e) => return Err(anyhow::anyhow!(e)),
    }
    Ok(())
}

fn current() -> Result<()> {
    let storage = open_storage()?;
    match storage
        .get_active_deck()
        .context("failed to read active deck")?
    {
        Some(deck) => print_human(format!(
            "Active deck: {} ({}) {}",
            deck.name,
            deck.id,
            deck.description.as_deref().unwrap_or("")
        )),
        None => print_human("No decks found."),
    }
    Ok(())
}

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
            ))
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
        return print_json(&CardListResponse::new(
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

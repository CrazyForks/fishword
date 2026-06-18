use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use fishword_core::{
    card::Rating,
    protocol::{RateResponse, StatsResponse, StatusResponse},
    scheduler::Scheduler,
    selector,
};

use crate::{
    args::{CardOutputArgs, RateArgs, StatsArgs, StatusArgs},
    util::{exit_json_error, open_storage, print_json, print_selected_card, resolve_deck_scope},
};

pub fn cmd_current(args: &CardOutputArgs) -> Result<()> {
    let storage = open_storage()?;
    let Some(deck) = resolve_deck_scope(&storage, args.deck, args.json)? else {
        if args.json {
            exit_json_error("no_cards", "No cards found. Import a deck first.");
        }
        println!("No cards found. Import a deck first.");
        return Ok(());
    };
    match selector::select_current(&storage, deck.id).context("failed to select current card")? {
        Some(selected) => print_selected_card(&storage, &selected, args, true)?,
        None if args.json => exit_json_error("no_cards", "No cards found. Import a deck first."),
        None => println!("No cards found. Import a deck first."),
    }
    Ok(())
}

pub fn cmd_status(args: &StatusArgs) -> Result<()> {
    let storage = open_storage()?;
    let Some(deck) = resolve_deck_scope(&storage, args.deck, args.json)? else {
        if args.json {
            exit_json_error("no_cards", "No cards found. Import a deck first.");
        }
        println!("No cards found. Import a deck first.");
        return Ok(());
    };
    let progress = storage
        .progress_counts_by_deck(deck.id, 20)
        .context("failed to read progress")?;
    let card_count = storage
        .card_count_by_deck(deck.id)
        .context("failed to count cards")?;
    let response = StatusResponse::new(&deck, progress, card_count);
    if args.json {
        print_json(&response)?;
    } else {
        match args.format.as_str() {
            "plain" => println!("{}", response.display.plain),
            "compact" => println!("{}", response.display.compact),
            "statusline" => println!("{}", response.display.statusline),
            other => anyhow::bail!(
                "invalid --format '{}', expected plain/compact/statusline",
                other
            ),
        }
    }
    Ok(())
}

pub fn cmd_stats(args: &StatsArgs) -> Result<()> {
    if args.range != "7d" {
        if args.json {
            exit_json_error("invalid_range", &format!("Invalid range: {}", args.range));
        }
        anyhow::bail!("invalid --range '{}', expected 7d", args.range);
    }
    let storage = open_storage()?;
    let Some(deck) = resolve_deck_scope(&storage, args.deck, args.json)? else {
        if args.json {
            exit_json_error("no_cards", "No cards found. Import a deck first.");
        }
        println!("No cards found. Import a deck first.");
        return Ok(());
    };
    let today = Utc::now().date_naive();
    let start = today - Duration::days(6);
    let buckets = storage
        .review_stats_by_deck_and_day_range(deck.id, &start.to_string(), &today.to_string())
        .context("failed to read review stats")?;
    let response = StatsResponse::new(&deck, 7, buckets);
    if args.json {
        print_json(&response)?;
    } else {
        println!("Today: {} reviews", response.summary.reviewed_today);
        match response.summary.good_or_easy_rate {
            Some(rate) => println!(
                "7 days: {} reviews, {}% good/easy",
                response.summary.reviews,
                (rate * 100.0).round() as i64
            ),
            None => println!(
                "7 days: {} reviews, no ratings yet",
                response.summary.reviews
            ),
        }
        println!(
            "{:<12} {:>7} {:>7} {:>7} {:>7} {:>7}",
            "DATE", "REVIEWS", "AGAIN", "HARD", "GOOD", "EASY"
        );
        for day in response.series {
            println!(
                "{:<12} {:>7} {:>7} {:>7} {:>7} {:>7}",
                day.date, day.reviews, day.again, day.hard, day.good, day.easy
            );
        }
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
    let Some(scope_deck) = resolve_deck_scope(&storage, args.deck, args.json)? else {
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
    let next = selector::select_next(&storage, scope_deck.id)
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
        let next_ref = next.as_ref().zip(next_deck.as_ref());
        print_json(&RateResponse::new(
            &card,
            &scope_deck,
            &review,
            progress,
            next_ref,
        ))?;
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

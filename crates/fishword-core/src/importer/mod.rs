use std::{collections::HashMap, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    card::{Meaning, Pronunciation, Source},
    error::{Error, Result},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DuplicateStrategy {
    #[default]
    Merge,
    Skip,
    Overwrite,
    Keep,
}

impl FromStr for DuplicateStrategy {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "merge" => Ok(Self::Merge),
            "skip" => Ok(Self::Skip),
            "overwrite" => Ok(Self::Overwrite),
            "keep" => Ok(Self::Keep),
            other => Err(Error::InvalidInput(format!(
                "unknown duplicate strategy '{other}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCard {
    pub word: String,
    pub language: String,
    pub meanings: Vec<Meaning>,
    pub pronunciations: Vec<Pronunciation>,
    pub tags: Vec<String>,
    pub source: Option<Source>,
}

#[derive(Debug, Clone)]
pub struct ImportDeck {
    pub deck_id: String,
    pub deck_name: Option<String>,
    pub cards: Vec<ImportCard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub deck_id: i64,
    pub deck_name: String,
    pub input_count: usize,
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub merged: usize,
}

#[derive(Debug, Deserialize)]
struct QwertyWord {
    name: String,
    #[serde(default)]
    trans: Vec<String>,
    #[serde(default)]
    usphone: Option<String>,
    #[serde(default)]
    ukphone: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeckCardV1 {
    #[serde(alias = "term")]
    word: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default)]
    meanings: Vec<DeckMeaningV1>,
    #[serde(default)]
    pronunciation: Option<DeckPronunciationV1>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    source: Option<Source>,
}

#[derive(Debug, Deserialize)]
struct DeckMeaningV1 {
    #[serde(default = "default_meaning_lang")]
    lang: String,
    text: String,
    #[serde(default)]
    example: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeckPronunciationV1 {
    #[serde(default)]
    us: Option<String>,
    #[serde(default)]
    uk: Option<String>,
}

pub fn import_qwerty_file(
    path: &Path,
    deck_id: &str,
    deck_name: Option<&str>,
) -> Result<ImportDeck> {
    let text = std::fs::read_to_string(path)?;
    import_qwerty_str(&text, deck_id, deck_name)
}

pub fn import_qwerty_str(text: &str, deck_id: &str, deck_name: Option<&str>) -> Result<ImportDeck> {
    let words: Vec<QwertyWord> = serde_json::from_str(text)?;
    let cards = words
        .into_iter()
        .map(|word| qwerty_word_to_card(word, deck_id))
        .collect::<Result<Vec<_>>>()?;
    Ok(ImportDeck {
        deck_id: deck_id.to_string(),
        deck_name: deck_name.map(str::to_string),
        cards,
    })
}

pub fn import_csv_file(path: &Path, deck_id: &str, deck_name: Option<&str>) -> Result<ImportDeck> {
    let text = std::fs::read_to_string(path)?;
    import_csv_str(&text, deck_id, deck_name)
}

pub fn import_csv_str(text: &str, deck_id: &str, deck_name: Option<&str>) -> Result<ImportDeck> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| Error::InvalidInput("CSV is empty".to_string()))?;
    let headers = parse_csv_line(header_line)?;
    let mut cards = Vec::new();

    for (index, line) in lines.enumerate() {
        let values = parse_csv_line(line)?;
        let row = headers
            .iter()
            .cloned()
            .zip(values)
            .collect::<HashMap<_, _>>();
        cards.push(
            csv_row_to_card(&row, deck_id)
                .map_err(|error| Error::InvalidInput(format!("CSV row {}: {error}", index + 2)))?,
        );
    }

    Ok(ImportDeck {
        deck_id: deck_id.to_string(),
        deck_name: deck_name.map(str::to_string),
        cards,
    })
}

pub fn import_jsonl_file(
    path: &Path,
    deck_id: &str,
    deck_name: Option<&str>,
) -> Result<ImportDeck> {
    let text = std::fs::read_to_string(path)?;
    import_jsonl_str(&text, deck_id, deck_name)
}

pub fn import_jsonl_str(text: &str, deck_id: &str, deck_name: Option<&str>) -> Result<ImportDeck> {
    let mut cards = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let card: DeckCardV1 = serde_json::from_str(line)
            .map_err(|error| Error::InvalidInput(format!("JSONL line {}: {error}", index + 1)))?;
        cards.push(deck_v1_to_import_card(card, deck_id)?);
    }
    Ok(ImportDeck {
        deck_id: deck_id.to_string(),
        deck_name: deck_name.map(str::to_string),
        cards,
    })
}

pub fn import_anki_tsv_file(
    path: &Path,
    deck_id: &str,
    deck_name: Option<&str>,
) -> Result<ImportDeck> {
    let text = std::fs::read_to_string(path)?;
    import_anki_tsv_str(&text, deck_id, deck_name)
}

pub fn import_anki_tsv_str(
    text: &str,
    deck_id: &str,
    deck_name: Option<&str>,
) -> Result<ImportDeck> {
    let mut cards = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').map(str::trim).collect::<Vec<_>>();
        if columns.len() < 2 {
            return Err(Error::InvalidInput(format!(
                "Anki TSV line {} needs at least word and meaning columns",
                index + 1
            )));
        }
        cards.push(ImportCard {
            word: columns[0].to_string(),
            language: default_language(),
            meanings: vec![Meaning {
                part_of_speech: "".to_string(),
                definition: columns[1].to_string(),
                example: None,
            }],
            pronunciations: columns
                .get(2)
                .filter(|value| !value.is_empty())
                .map(|notation| {
                    vec![Pronunciation {
                        notation: (*notation).to_string(),
                        audio_url: None,
                    }]
                })
                .unwrap_or_default(),
            tags: vec![deck_id.to_string(), "anki".to_string()],
            source: Some(Source {
                name: "anki-tsv".to_string(),
                license: None,
            }),
        });
    }
    Ok(ImportDeck {
        deck_id: deck_id.to_string(),
        deck_name: deck_name.map(str::to_string),
        cards,
    })
}

fn qwerty_word_to_card(word: QwertyWord, deck_id: &str) -> Result<ImportCard> {
    if word.name.trim().is_empty() {
        return Err(Error::InvalidInput("Qwerty word name is empty".to_string()));
    }

    let meanings = word
        .trans
        .into_iter()
        .filter(|text| !text.trim().is_empty())
        .map(|text| Meaning {
            part_of_speech: "".to_string(),
            definition: text,
            example: None,
        })
        .collect::<Vec<_>>();
    let mut pronunciations = Vec::new();
    if let Some(us) = word.usphone.filter(|value| !value.trim().is_empty()) {
        pronunciations.push(Pronunciation {
            notation: format!("US {us}"),
            audio_url: None,
        });
    }
    if let Some(uk) = word.ukphone.filter(|value| !value.trim().is_empty()) {
        pronunciations.push(Pronunciation {
            notation: format!("UK {uk}"),
            audio_url: None,
        });
    }

    Ok(ImportCard {
        word: word.name,
        language: default_language(),
        meanings,
        pronunciations,
        tags: vec![deck_id.to_string()],
        source: Some(Source {
            name: "qwerty-learner".to_string(),
            license: Some("GPL-3.0".to_string()),
        }),
    })
}

fn csv_row_to_card(row: &HashMap<String, String>, deck_id: &str) -> Result<ImportCard> {
    let word = first_value(row, &["word", "term", "name"])
        .ok_or_else(|| Error::InvalidInput("missing word/term/name column".to_string()))?;
    let meaning =
        first_value(row, &["meaning", "definition", "trans", "translation"]).ok_or_else(|| {
            Error::InvalidInput("missing meaning/definition/trans column".to_string())
        })?;
    let source_name = first_value(row, &["source"]).unwrap_or("csv");
    let source_license = first_value(row, &["license"]).map(str::to_string);
    let tags = first_value(row, &["tags"])
        .map(|value| split_tags(value, deck_id))
        .unwrap_or_else(|| vec![deck_id.to_string()]);

    let mut pronunciations = Vec::new();
    for key in ["pronunciation", "phone", "usphone", "ukphone"] {
        if let Some(value) = first_value(row, &[key]).filter(|value| !value.trim().is_empty()) {
            pronunciations.push(Pronunciation {
                notation: value.to_string(),
                audio_url: None,
            });
        }
    }

    Ok(ImportCard {
        word: word.to_string(),
        language: first_value(row, &["language", "lang"])
            .unwrap_or(default_language().as_str())
            .to_string(),
        meanings: vec![Meaning {
            part_of_speech: first_value(row, &["part_of_speech", "pos"])
                .unwrap_or("")
                .to_string(),
            definition: meaning.to_string(),
            example: first_value(row, &["example"]).map(str::to_string),
        }],
        pronunciations,
        tags,
        source: Some(Source {
            name: source_name.to_string(),
            license: source_license,
        }),
    })
}

fn deck_v1_to_import_card(card: DeckCardV1, deck_id: &str) -> Result<ImportCard> {
    if card.word.trim().is_empty() {
        return Err(Error::InvalidInput(
            "deck.v1 card word is empty".to_string(),
        ));
    }
    let meanings = card
        .meanings
        .into_iter()
        .map(|meaning| Meaning {
            part_of_speech: meaning.lang,
            definition: meaning.text,
            example: meaning.example,
        })
        .collect::<Vec<_>>();
    let mut pronunciations = Vec::new();
    if let Some(pronunciation) = card.pronunciation {
        if let Some(us) = pronunciation.us.filter(|value| !value.trim().is_empty()) {
            pronunciations.push(Pronunciation {
                notation: format!("US {us}"),
                audio_url: None,
            });
        }
        if let Some(uk) = pronunciation.uk.filter(|value| !value.trim().is_empty()) {
            pronunciations.push(Pronunciation {
                notation: format!("UK {uk}"),
                audio_url: None,
            });
        }
    }
    let tags = if card.tags.is_empty() {
        vec![deck_id.to_string()]
    } else {
        card.tags
    };
    Ok(ImportCard {
        word: card.word,
        language: card.language,
        meanings,
        pronunciations,
        tags,
        source: card.source,
    })
}

fn parse_csv_line(line: &str) -> Result<Vec<String>> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                values.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err(Error::InvalidInput("unterminated CSV quote".to_string()));
    }

    values.push(current.trim().to_string());
    Ok(values)
}

fn first_value<'a>(row: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| row.get(*key))
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn split_tags(value: &str, deck_id: &str) -> Vec<String> {
    let mut tags = value
        .split([';', ','])
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !tags.iter().any(|tag| tag == deck_id) {
        tags.insert(0, deck_id.to_string());
    }
    tags
}

fn default_language() -> String {
    "en".to_string()
}

fn default_meaning_lang() -> String {
    "zh-CN".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn open_temp() -> Storage {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep().join("importer.db");
        Storage::open(&path).unwrap()
    }

    #[test]
    fn imports_qwerty_json() {
        let deck =
            import_qwerty_str(&fixture("qwerty_cet4_sample.json"), "cet4", Some("CET-4")).unwrap();
        assert_eq!(deck.cards.len(), 2);
        assert_eq!(deck.cards[0].word, "cancel");
        assert!(deck.cards[0].tags.is_empty());
        assert_eq!(
            deck.cards[0].source.as_ref().unwrap().name,
            "qwerty-learner"
        );
        assert_eq!(
            deck.cards[0].source.as_ref().unwrap().license.as_deref(),
            Some("GPL-3.0")
        );
    }

    #[test]
    fn imports_minimal_csv() {
        let deck = import_csv_str(&fixture("minimal_words.csv"), "custom", None).unwrap();
        assert_eq!(deck.cards.len(), 2);
        assert_eq!(deck.cards[1].word, "abandon");
        assert_eq!(deck.cards[1].meanings[0].definition, "放弃");
        assert!(deck.cards[0].tags.is_empty());
    }

    #[test]
    fn imports_jsonl_and_anki_tsv() {
        let jsonl = import_jsonl_str(&fixture("deck_v1_sample.jsonl"), "jsonl", None).unwrap();
        assert_eq!(jsonl.cards[0].word, "cancel");
        assert_eq!(jsonl.cards[0].pronunciations.len(), 2);

        let anki = import_anki_tsv_str(&fixture("anki_sample.tsv"), "custom", None).unwrap();
        assert_eq!(anki.cards.len(), 2);
        assert_eq!(anki.cards[0].source.as_ref().unwrap().name, "anki-tsv");
        assert_eq!(anki.cards[0].tags, vec!["anki"]);
    }

    #[test]
    fn importer_preserves_only_explicit_tags() {
        let csv =
            import_csv_str("word,meaning,tags\ncancel,取消,review;hard\n", "cet4", None).unwrap();
        assert_eq!(csv.cards[0].tags, vec!["review", "hard"]);

        let jsonl = import_jsonl_str(
            r#"{"term":"cancel","meanings":[{"lang":"zh-CN","text":"取消"}]}"#,
            "jsonl",
            None,
        )
        .unwrap();
        assert!(jsonl.cards[0].tags.is_empty());
    }

    #[test]
    fn duplicate_skip_and_merge_work_in_storage() {
        let storage = open_temp();
        let deck = storage.insert_deck("cet4", Some("CET-4")).unwrap();
        let initial = import_csv_str("word,meaning\ncancel,取消\n", "cet4", None).unwrap();
        let summary = storage
            .import_cards(deck.id, &initial.cards, DuplicateStrategy::Merge)
            .unwrap();
        assert_eq!(summary.inserted, 1);

        let duplicate =
            import_csv_str("word,meaning,tags\ncancel,撤销,review\n", "cet4", None).unwrap();
        let skipped = storage
            .import_cards(deck.id, &duplicate.cards, DuplicateStrategy::Skip)
            .unwrap();
        assert_eq!(skipped.skipped, 1);
        assert_eq!(
            storage
                .list_cards_by_deck_paginated(deck.id, 100, 0)
                .unwrap()
                .len(),
            1
        );

        let merged = storage
            .import_cards(deck.id, &duplicate.cards, DuplicateStrategy::Merge)
            .unwrap();
        assert_eq!(merged.merged, 1);
        let cards = storage
            .list_cards_by_deck_paginated(deck.id, 100, 0)
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].meanings.len(), 2);
        assert!(cards[0].tags.iter().any(|tag| tag == "review"));
    }

    #[test]
    fn duplicate_overwrite_and_keep_work_in_storage() {
        let storage = open_temp();
        let deck = storage.insert_deck("cet4", Some("CET-4")).unwrap();
        let initial = import_csv_str("word,meaning\ncancel,取消\n", "cet4", None).unwrap();
        storage
            .import_cards(deck.id, &initial.cards, DuplicateStrategy::Merge)
            .unwrap();

        let replacement = import_csv_str("word,meaning\ncancel,撤销\n", "cet4", None).unwrap();
        let overwritten = storage
            .import_cards(deck.id, &replacement.cards, DuplicateStrategy::Overwrite)
            .unwrap();
        assert_eq!(overwritten.updated, 1);
        assert_eq!(
            storage
                .list_cards_by_deck_paginated(deck.id, 100, 0)
                .unwrap()[0]
                .meanings[0]
                .definition,
            "撤销"
        );

        let kept = storage
            .import_cards(deck.id, &replacement.cards, DuplicateStrategy::Keep)
            .unwrap();
        assert_eq!(kept.inserted, 1);
        assert_eq!(
            storage
                .list_cards_by_deck_paginated(deck.id, 100, 0)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn jsonl_example_field_is_parsed() {
        let jsonl = r#"{"term":"absorb","meanings":[{"lang":"v","text":"吸收","example":"Plants absorb nutrients from the soil."}]}"#;
        let deck = import_jsonl_str(jsonl, "test", None).unwrap();
        assert_eq!(deck.cards.len(), 1);
        let meaning = &deck.cards[0].meanings[0];
        assert_eq!(meaning.definition, "吸收");
        assert_eq!(
            meaning.example.as_deref(),
            Some("Plants absorb nutrients from the soil.")
        );
    }

    #[test]
    fn jsonl_missing_example_is_none() {
        let jsonl = r#"{"term":"cancel","meanings":[{"lang":"zh-CN","text":"取消"}]}"#;
        let deck = import_jsonl_str(jsonl, "test", None).unwrap();
        assert!(deck.cards[0].meanings[0].example.is_none());
    }

    #[test]
    fn jsonl_multiple_meanings_example_per_meaning() {
        let jsonl = r#"{"term":"access","meanings":[{"lang":"n","text":"入口","example":"The only access is across the bridge."},{"lang":"vt","text":"访问"}]}"#;
        let deck = import_jsonl_str(jsonl, "test", None).unwrap();
        let meanings = &deck.cards[0].meanings;
        assert_eq!(
            meanings[0].example.as_deref(),
            Some("The only access is across the bridge.")
        );
        assert!(meanings[1].example.is_none());
    }

    #[test]
    fn example_persisted_and_retrieved_from_storage() {
        let storage = open_temp();
        let db_deck = storage.insert_deck("test", None).unwrap();
        let jsonl = r#"{"term":"absorb","meanings":[{"lang":"v","text":"吸收","example":"Plants absorb nutrients from the soil."}]}"#;
        let parsed = import_jsonl_str(jsonl, "test", None).unwrap();
        storage
            .import_cards(db_deck.id, &parsed.cards, DuplicateStrategy::Merge)
            .unwrap();

        let cards = storage
            .list_cards_by_deck_paginated(db_deck.id, 100, 0)
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(
            cards[0].meanings[0].example.as_deref(),
            Some("Plants absorb nutrients from the soil.")
        );
    }
}

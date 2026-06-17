use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_home(test_name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fishword-{test_name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_jsonl(home: &Path) -> PathBuf {
    let path = home.join("words.jsonl");
    fs::write(
        &path,
        r#"{"term":"cancel","meanings":[{"lang":"zh-CN","text":"取消"}]}"#,
    )
    .unwrap();
    path
}

fn fishword(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fishword"))
        .env("HOME", home)
        .args(args)
        .output()
        .unwrap()
}

fn assert_success(output: std::process::Output) -> String {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn assert_failure(output: std::process::Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn import_with_create_deck_creates_deck_and_imports_cards() {
    let home = temp_home("import-name");
    let jsonl = write_jsonl(&home);

    assert_success(fishword(&home, &["init"]));
    let output = assert_success(fishword(
        &home,
        &[
            "import",
            "jsonl",
            jsonl.to_str().unwrap(),
            "--create-deck",
            "Imported",
        ],
    ));
    let decks = assert_success(fishword(&home, &["deck", "list"]));
    let cards = assert_success(fishword(&home, &["card", "list", "--deck", "1"]));

    assert!(output.contains("Imported deck=Imported input=1 inserted=1"));
    assert!(decks.contains("Imported"));
    assert!(cards.contains("cancel"));
    assert!(cards.contains("取消"));
}

#[test]
fn import_with_deck_id_still_imports_into_existing_deck() {
    let home = temp_home("import-deck");
    let jsonl = write_jsonl(&home);

    assert_success(fishword(&home, &["init"]));
    assert_success(fishword(&home, &["deck", "create", "Existing"]));
    let output = assert_success(fishword(
        &home,
        &["import", "jsonl", jsonl.to_str().unwrap(), "--deck-id", "1"],
    ));
    let cards = assert_success(fishword(&home, &["card", "list", "--deck", "1"]));

    assert!(output.contains("Imported deck=Existing input=1 inserted=1"));
    assert!(cards.contains("cancel"));
    assert!(cards.contains("取消"));
}

#[test]
fn import_target_must_be_create_deck_or_deck_id() {
    let home = temp_home("import-target");
    let jsonl = write_jsonl(&home);

    assert_success(fishword(&home, &["init"]));
    assert_failure(fishword(
        &home,
        &["import", "jsonl", jsonl.to_str().unwrap()],
    ));
    assert_failure(fishword(
        &home,
        &[
            "import",
            "jsonl",
            jsonl.to_str().unwrap(),
            "--deck-id",
            "1",
            "--create-deck",
            "Imported",
        ],
    ));
}

#[test]
fn import_with_create_deck_rejects_existing_deck() {
    let home = temp_home("import-name-existing");
    let jsonl = write_jsonl(&home);

    assert_success(fishword(&home, &["init"]));
    assert_success(fishword(&home, &["deck", "create", "Existing"]));
    assert_failure(fishword(
        &home,
        &[
            "import",
            "jsonl",
            jsonl.to_str().unwrap(),
            "--create-deck",
            "Existing",
        ],
    ));
}

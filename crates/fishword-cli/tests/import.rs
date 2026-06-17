use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    thread,
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

fn fishword_with_env(home: &Path, args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fishword"));
    command.env("HOME", home).args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().unwrap()
}

fn start_catalog_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let catalog_url = format!("http://{addr}/catalog.json");
    let deck_url = format!("http://{addr}/deck.jsonl");
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.unwrap();
            handle_catalog_request(&mut stream, &deck_url);
        }
    });
    (catalog_url, handle)
}

fn handle_catalog_request(stream: &mut TcpStream, deck_url: &str) {
    let mut request = [0_u8; 1024];
    let n = stream.read(&mut request).unwrap();
    let request = String::from_utf8_lossy(&request[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let body = match path {
        "/catalog.json" => format!(
            r#"{{
                "decks": [{{
                    "id": "test:sample",
                    "slug": "sample",
                    "source_id": "test",
                    "name": "Sample",
                    "description": "Demo catalog description",
                    "word_count": 1,
                    "tags": ["test"],
                    "url": "{deck_url}",
                    "size_bytes": 64
                }}]
            }}"#
        ),
        "/deck.jsonl" => {
            r#"{"term":"cancel","meanings":[{"lang":"zh-CN","text":"取消"}]}"#.to_string()
        }
        _ => String::new(),
    };
    let status = if body.is_empty() {
        "HTTP/1.1 404 Not Found"
    } else {
        "HTTP/1.1 200 OK"
    };
    write!(
        stream,
        "{status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
    .unwrap();
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

#[test]
fn import_catalog_fetch_preserves_manifest_description() {
    let home = temp_home("import-catalog-description");
    let (catalog_url, server) = start_catalog_server();

    assert_success(fishword(&home, &["init"]));
    assert_success(fishword_with_env(
        &home,
        &["catalog", "fetch", "test:sample", "--json"],
        &[("FISHWORD_CATALOG_URL", &catalog_url)],
    ));
    server.join().unwrap();

    let decks = assert_success(fishword(&home, &["deck", "list", "--json"]));
    let decks: serde_json::Value = serde_json::from_str(&decks).unwrap();

    assert_eq!(
        decks["decks"][0]["description"].as_str(),
        Some("Demo catalog description")
    );
}

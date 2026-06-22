use std::fs;

#[test]
fn command_modules_do_not_write_directly_to_stdio() {
    let cmd_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cmd");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&cmd_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        for (index, line) in source.lines().enumerate() {
            if line.contains("println!(") || line.contains("print!(") || line.contains("eprintln!(")
            {
                violations.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "command modules must route output through util.rs:\n{}",
        violations.join("\n")
    );
}

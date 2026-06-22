use anyhow::{Context, Result};
use fishword_core::storage::Storage;

use crate::util::print_human;

pub fn cmd_init() -> Result<()> {
    let path = Storage::default_path().context("cannot determine data directory")?;
    Storage::open(&path)
        .with_context(|| format!("cannot initialize database at {}", path.display()))?;
    print_human(format!("Initialized: {}", path.display()));
    Ok(())
}

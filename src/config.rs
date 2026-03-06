use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn db_path(override_path: Option<&str>) -> Result<PathBuf> {
    if let Some(p) = override_path {
        return Ok(PathBuf::from(p));
    }

    if let Ok(dir) = std::env::var("PPL_DIR") {
        return Ok(PathBuf::from(dir).join("ppl.db"));
    }

    let data_dir = dirs::data_dir()
        .context("Could not determine data directory")?
        .join("ppl");

    Ok(data_dir.join("ppl.db"))
}

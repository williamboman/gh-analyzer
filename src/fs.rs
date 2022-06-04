use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Serialize;
use tokio::fs::File;

#[inline]
fn dirname(path: &Path) -> Result<&Path> {
    path.ancestors()
        .nth(1)
        .ok_or_else(|| anyhow!("Failed to find ancestor of {}.", path.display()))
}

pub async fn write_json<T>(rel_path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let cwd = std::env::current_dir()?;
    let path = cwd.join(rel_path);
    tokio::fs::create_dir_all(dirname(&path)?).await?;
    let file = File::create(&path).await?;
    println!("Writing JSON file at {}", path.display());
    serde_json::to_writer_pretty(&file.into_std().await, value)?;
    Ok(())
}

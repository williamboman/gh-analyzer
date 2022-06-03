use std::fmt::Display;

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::process::Command;

use super::{GitHubClones, GitHubRepo, GitHubTraffic};

async fn api_call<T>(path: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let result = Command::new("gh").args(["api", path]).output().await?;

    if result.status.success() {
        Ok(serde_json::from_slice::<T>(&result.stdout)
            .map_err(|err| anyhow!("Failed to parse `gh` output.").context(err))?)
    } else {
        Err(anyhow!(
            "Failed to execute GitHub API call to {}:\n{}",
            path,
            std::str::from_utf8(&result.stdout).unwrap_or("Unable to read `gh` error.")
        ))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Frequency {
    Day,
    Week,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubClonesContainer {
    pub repo: GitHubRepo,
    pub frequency: Frequency,
    pub payload: GitHubClones,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTrafficContainer {
    pub repo: GitHubRepo,
    pub frequency: Frequency,
    pub payload: GitHubTraffic,
}

impl Display for Frequency {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(match self {
            Frequency::Day => "day",
            Frequency::Week => "week",
        })
    }
}

pub async fn fetch_traffic(
    repo: &GitHubRepo,
    frequency: Frequency,
) -> Result<GitHubTrafficContainer> {
    let payload = api_call(&repo.api_path(&format!("traffic/views?per={}", frequency))).await?;
    Ok(GitHubTrafficContainer {
        repo: repo.to_owned(),
        frequency,
        payload,
    })
}

pub async fn fetch_clones(
    repo: &GitHubRepo,
    frequency: Frequency,
) -> Result<GitHubClonesContainer> {
    let payload = api_call(&repo.api_path(&format!("traffic/clones?per={}", frequency))).await?;
    Ok(GitHubClonesContainer {
        repo: repo.to_owned(),
        frequency,
        payload,
    })
}

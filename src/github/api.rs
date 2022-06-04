use std::fmt::Display;

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::process::Command;

use crate::iso8601date::ISO8601Date;

use super::GitHubRepoId;

async fn api_call<T>(path: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let result = Command::new("gh")
        .args(["api", path])
        .output()
        .await
        .map_err(|err| anyhow!("Failed to spawn `gh`").context(err))?;

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
pub struct GitHubTrafficStat {
    pub timestamp: ISO8601Date,
    pub count: u32,
    pub uniques: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTraffic {
    pub count: u32,
    pub uniques: u32,
    pub views: Vec<GitHubTrafficStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubClones {
    pub count: u32,
    pub uniques: u32,
    pub clones: Vec<GitHubTrafficStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepoLicense {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub full_name: String,
    pub forks_count: u32,
    pub stargazers_count: u32,
    pub watchers_count: u32,
    pub open_issues_count: u32,
    pub subscribers_count: u32,
    pub has_wiki: bool,
    pub archived: bool,
    pub has_projects: bool,
    pub size: u32,
    pub topics: Vec<String>,
    pub license: GitHubRepoLicense,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubClonesContainer {
    pub repo: GitHubRepoId,
    pub frequency: Frequency,
    pub payload: GitHubClones,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTrafficContainer {
    pub repo: GitHubRepoId,
    pub frequency: Frequency,
    pub payload: GitHubTraffic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepoContainer {
    pub repo: GitHubRepoId,
    pub payload: GitHubRepo,
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
    repo: &GitHubRepoId,
    frequency: Frequency,
) -> Result<GitHubTrafficContainer> {
    let payload =
        api_call(format!("repos/{}/traffic/views?per={}", repo.as_slug(), frequency).as_str())
            .await?;
    Ok(GitHubTrafficContainer {
        repo: repo.to_owned(),
        frequency,
        payload,
    })
}

pub async fn fetch_clones(
    repo: &GitHubRepoId,
    frequency: Frequency,
) -> Result<GitHubClonesContainer> {
    let payload =
        api_call(format!("repos/{}/traffic/clones?per={}", repo.as_slug(), frequency).as_str())
            .await?;
    Ok(GitHubClonesContainer {
        repo: repo.to_owned(),
        frequency,
        payload,
    })
}

pub async fn fetch_repo(repo: &GitHubRepoId) -> Result<GitHubRepoContainer> {
    let payload = api_call(format!("repos/{}", repo.as_slug()).as_str()).await?;
    Ok(GitHubRepoContainer {
        repo: repo.to_owned(),
        payload,
    })
}

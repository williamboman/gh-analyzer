pub mod api;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::{iso8601date::ISO8601Date, StdResult};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub owner: String,
    pub repo: String,
}

impl GitHubRepo {
    pub fn as_slug(&self) -> String {
        format!("{owner}/{repo}", owner = self.owner, repo = self.repo)
    }

    pub fn api_path(&self, path: &str) -> String {
        format!("repos/{}/{}", self.as_slug(), path)
    }
}

impl Display for GitHubRepo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.as_slug())
    }
}

impl FromStr for GitHubRepo {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        if let Some((owner, repo)) = s.split_once("/") {
            Ok(Self {
                repo: repo.to_string(),
                owner: owner.to_string(),
            })
        } else {
            Err(anyhow!("Failed to parse GitHub repository \"{}\".", s))
        }
    }
}

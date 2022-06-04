pub mod api;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::StdResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubRepoId {
    pub owner: String,
    pub repo: String,
}

impl GitHubRepoId {
    pub fn as_slug(&self) -> String {
        format!("{owner}/{repo}", owner = self.owner, repo = self.repo)
    }
}

impl Display for GitHubRepoId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.as_slug())
    }
}

impl FromStr for GitHubRepoId {
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

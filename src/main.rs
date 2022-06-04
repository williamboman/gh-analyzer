mod cli;
mod fs;
mod github;
mod iso8601date;

use std::str::FromStr;

use anyhow::Result;

use github::{
    api::{GitHubClonesContainer, GitHubRepoContainer, GitHubTrafficContainer},
    GitHubRepoId,
};
use iso8601date::ISO8601Date;

type StdResult<T, E> = std::result::Result<T, E>;

const OUT_DIR: &str = "gh-analyzer-output";

// TODO skip incomplete data points (i.e. today and current week)
// TODO DRY me
async fn write_traffic(traffic: &GitHubTrafficContainer) -> Result<()> {
    for traffic_stat in &traffic.payload.views {
        fs::write_json(
            format!(
                "{}/{}/traffic/{}/{}.json",
                OUT_DIR,
                traffic.repo,
                traffic.frequency,
                traffic_stat.timestamp.as_date_str()
            )
            .as_str(),
            &traffic_stat,
        )
        .await?;
    }
    Ok(())
}
async fn write_clones(clones: &GitHubClonesContainer) -> Result<()> {
    for traffic_stat in &clones.payload.clones {
        fs::write_json(
            format!(
                "{}/{}/clones/{}/{}.json",
                OUT_DIR,
                clones.repo,
                clones.frequency,
                traffic_stat.timestamp.as_date_str()
            )
            .as_str(),
            &traffic_stat,
        )
        .await?;
    }
    Ok(())
}
async fn write_repo(repo_container: &GitHubRepoContainer) -> Result<()> {
    fs::write_json(
        format!(
            "{}/{}/repo/{}.json",
            OUT_DIR,
            repo_container.repo,
            ISO8601Date::now_utc().as_date_str()
        )
        .as_str(),
        &repo_container.payload,
    )
    .await
}

enum Command {
    Traffic,
    Clones,
    Repo,
}

impl FromStr for Command {
    type Err = cli::CliError;

    fn from_str(s: &str) -> Result<Self, cli::CliError> {
        match s {
            "traffic" => Ok(Self::Traffic),
            "clones" => Ok(Self::Clones),
            "repo" => Ok(Self::Repo),
            _ => Err(cli::CliError::BadInput(format!(
                "{} is not a valid command",
                s
            ))),
        }
    }
}

impl cli::Cli {
    fn get_repo(&self) -> Result<GitHubRepoId> {
        self.sub_commands
            .first()
            .ok_or(cli::CliError::BadInput(
                "No repository provided.".to_owned(),
            ))?
            .parse()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::init(&mut std::env::args())?;
    if cli.flags.contains_key("help") || cli.flags.contains_key("h") {
        cli::print_help();
        return Ok(());
    }

    let command = cli
        .command
        .clone()
        .ok_or(cli::CliError::BadInput("No command provided.".to_owned()))?
        .parse()?;

    match command {
        Command::Traffic => {
            let repo: GitHubRepoId = cli.get_repo()?;

            let (weekly, daily) = tokio::join!(
                github::api::fetch_traffic(&repo, github::api::Frequency::Week),
                github::api::fetch_traffic(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = &weekly {
                write_traffic(container).await?;
            }
            if let Ok(container) = &daily {
                write_traffic(container).await?;
            }
            weekly.and(daily)?;
            // ---
        }
        Command::Clones => {
            let repo = cli.get_repo()?;

            let (weekly, daily) = tokio::join!(
                github::api::fetch_clones(&repo, github::api::Frequency::Week),
                github::api::fetch_clones(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = &weekly {
                write_clones(container).await?;
            }
            if let Ok(container) = &daily {
                write_clones(container).await?;
            }
            weekly.and(daily)?;
            // ---
        }
        Command::Repo => {
            let repo = cli.get_repo()?;
            let repo_container = github::api::fetch_repo(&repo).await?;
            write_repo(&repo_container).await?;
        }
    }

    Ok(())
}

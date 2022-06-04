mod cli;
mod fs;
mod github;
mod iso8601date;

use std::str::FromStr;

use anyhow::Result;

use github::api::{GitHubClonesContainer, GitHubRepoContainer, GitHubTrafficContainer};
use iso8601date::ISO8601Date;

type StdResult<T, E> = std::result::Result<T, E>;

// TODO DRY me
async fn write_traffic(out_dir: &str, traffic: &GitHubTrafficContainer) -> Result<()> {
    for traffic_stat in &traffic.payload.views {
        fs::write_json(
            format!(
                "{}/{}/traffic/{}/{}.json",
                out_dir,
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
async fn write_clones(out_dir: &str, clones: &GitHubClonesContainer) -> Result<()> {
    for traffic_stat in &clones.payload.clones {
        fs::write_json(
            format!(
                "{}/{}/clones/{}/{}.json",
                out_dir,
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
async fn write_repo(out_dir: &str, repo_container: &GitHubRepoContainer) -> Result<()> {
    fs::write_json(
        format!(
            "{}/{}/repo/{}.json",
            out_dir,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::init(&mut std::env::args())?;
    if cli.flags.contains("h") || cli.options.contains_key("help") {
        cli::print_help();
        return Ok(());
    }

    if cli.flags.contains("v") || cli.options.contains_key("version") {
        cli::print_version();
        return Ok(());
    }

    let out_dir = cli.options.get("out-dir").ok_or(cli::CliError::BadInput(
        "Missing --out-dir option.".to_owned(),
    ))?;

    let mut commands = cli.commands.into_iter();

    let command = commands
        .next()
        .ok_or(cli::CliError::BadInput("No command provided.".to_owned()))?
        .parse()?;

    let repo = commands
        .next()
        .ok_or(cli::CliError::BadInput(
            "No repository provided.".to_owned(),
        ))?
        .parse()?;

    match command {
        Command::Traffic => {
            let (weekly, daily) = tokio::join!(
                github::api::fetch_traffic(&repo, github::api::Frequency::Week),
                github::api::fetch_traffic(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = &weekly {
                write_traffic(out_dir, container).await?;
            }
            if let Ok(container) = &daily {
                write_traffic(out_dir, container).await?;
            }
            weekly.and(daily)?;
            // ---
        }
        Command::Clones => {
            let (weekly, daily) = tokio::join!(
                github::api::fetch_clones(&repo, github::api::Frequency::Week),
                github::api::fetch_clones(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = &weekly {
                write_clones(out_dir, container).await?;
            }
            if let Ok(container) = &daily {
                write_clones(out_dir, container).await?;
            }
            weekly.and(daily)?;
            // ---
        }
        Command::Repo => {
            let repo_container = github::api::fetch_repo(&repo).await?;
            write_repo(out_dir, &repo_container).await?;
        }
    }

    Ok(())
}

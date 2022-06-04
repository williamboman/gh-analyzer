mod cli;
mod fs;
mod github;
mod iso8601date;

use std::{fmt::Display, path::PathBuf, str::FromStr};

use anyhow::Result;

use github::{GitHubRepoId, GitHubStats};
use iso8601date::ISO8601Date;
use serde::Serialize;

type StdResult<T, E> = std::result::Result<T, E>;

async fn write_stats(out_dir: &PathBuf, stats: &dyn GitHubStats) -> Result<()> {
    for stat in stats.get_stats() {
        let mut stat_path = out_dir.to_owned();
        stat_path.push(format!(
            "{}/{}.json",
            stats.get_frequency(),
            stat.timestamp.as_date_str()
        ));
        fs::write_json(&stat_path, &stat).await?;
    }
    Ok(())
}

async fn write_single<T>(out_dir: &PathBuf, data: &T) -> Result<()>
where
    T: Serialize,
{
    let mut path = out_dir.to_owned();
    path.push(format!("{}.json", ISO8601Date::now_utc().as_date_str()));
    fs::write_json(&path, data).await
}

enum Command {
    Traffic,
    Clones,
    Repo,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Traffic => f.write_str("traffic"),
            Self::Clones => f.write_str("clones"),
            Self::Repo => f.write_str("repo"),
        }
    }
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

    let mut out_dir: PathBuf = cli
        .options
        .get("out-dir")
        .ok_or(cli::CliError::BadInput(
            "Missing --out-dir option.".to_owned(),
        ))?
        .into();

    let mut commands = cli.commands.into_iter();

    let command: Command = commands
        .next()
        .ok_or(cli::CliError::BadInput("No command provided.".to_owned()))?
        .parse()?;

    let repo: GitHubRepoId = commands
        .next()
        .ok_or(cli::CliError::BadInput(
            "No repository provided.".to_owned(),
        ))?
        .parse()?;

    out_dir.push(repo.as_slug());
    out_dir.push(command.to_string());

    match command {
        Command::Traffic => {
            let (weekly, daily) = tokio::join!(
                github::api::fetch_traffic(&repo, github::api::Frequency::Week),
                github::api::fetch_traffic(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = &weekly {
                write_stats(&out_dir, container).await?;
            }
            if let Ok(container) = &daily {
                write_stats(&out_dir, container).await?;
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
                write_stats(&out_dir, container).await?;
            }
            if let Ok(container) = &daily {
                write_stats(&out_dir, container).await?;
            }
            weekly.and(daily)?;
            // ---
        }
        Command::Repo => {
            let repo_container = github::api::fetch_repo(&repo).await?;
            write_single(&out_dir, &repo_container.payload).await?;
        }
    }

    Ok(())
}

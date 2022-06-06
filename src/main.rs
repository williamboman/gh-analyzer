mod cli;
mod fs;
mod github;
mod iso8601date;

use std::path::PathBuf;

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
    Traffic(GitHubRepoId),
    Clones(GitHubRepoId),
    Repo(GitHubRepoId),
}

fn parse_repo_from_arg(argument: Option<String>) -> Result<GitHubRepoId> {
    let argument_str = argument.ok_or(cli::CliError::BadInput(
        "No repository provided.".to_owned(),
    ))?;
    let github_repo_id = argument_str
        .parse()
        .map_err(|_| cli::CliError::BadInput("Unable to parse repository.".to_owned()))?;
    Ok(github_repo_id)
}

impl TryFrom<cli::Commands> for Command {
    type Error = anyhow::Error;

    fn try_from(commands: cli::Commands) -> Result<Self, Self::Error> {
        let mut iter = commands.into_iter();
        let command = iter
            .next()
            .ok_or(cli::CliError::BadInput("No command provided.".to_owned()))?;

        match command.as_str() {
            "traffic" => Ok(Command::Traffic(parse_repo_from_arg(iter.next())?)),
            "clones" => Ok(Command::Clones(parse_repo_from_arg(iter.next())?)),
            "repo" => Ok(Command::Repo(parse_repo_from_arg(iter.next())?)),
            _ => {
                Err(cli::CliError::BadInput(format!("Command {} does not exist.", command)).into())
            }
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

    let command: Command = cli.commands.try_into()?;

    match command {
        Command::Traffic(repo) => {
            let (weekly, daily) = tokio::join!(
                github::api::fetch_traffic(&repo, github::api::Frequency::Week),
                github::api::fetch_traffic(&repo, github::api::Frequency::Day)
            );
            out_dir.push(repo.to_slug());
            out_dir.push("traffic");
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
        Command::Clones(repo) => {
            let (weekly, daily) = tokio::join!(
                github::api::fetch_clones(&repo, github::api::Frequency::Week),
                github::api::fetch_clones(&repo, github::api::Frequency::Day)
            );
            out_dir.push(repo.to_slug());
            out_dir.push("traffic");
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
        Command::Repo(repo) => {
            out_dir.push(repo.to_slug());
            out_dir.push("traffic");
            let repo_container = github::api::fetch_repo(&repo).await?;
            write_single(&out_dir, &repo_container.payload).await?;
        }
    }

    Ok(())
}

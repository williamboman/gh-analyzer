mod cli;
mod fs;
mod github;
mod iso8601date;

use std::str::FromStr;

use anyhow::{anyhow, Result};

use github::{
    api::{GitHubClonesContainer, GitHubTrafficContainer},
    GitHubRepo,
};

use crate::cli::PrintHelp;

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

enum Command {
    Traffic,
    Clones,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "traffic" => Ok(Self::Traffic),
            "clones" => Ok(Self::Clones),
            _ => Err(anyhow!("{} is not a valid command", s)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::init(&mut std::env::args())?;
    if cli.flags.contains_key("help") || cli.flags.contains_key("h") {
        cli::print_help();
        return Ok(());
    }
    let command: Command = cli
        .command
        .ok_or_print_help("No command provided.")?
        .parse()?;

    match command {
        Command::Traffic => {
            let repo: GitHubRepo = cli
                .sub_commands
                .first()
                .ok_or_print_help("No repository provided.")?
                .parse()?;

            let (weekly, daily) = tokio::join!(
                github::api::fetch_traffic(&repo, github::api::Frequency::Week),
                github::api::fetch_traffic(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = weekly {
                write_traffic(&container).await?;
            }
            if let Ok(container) = daily {
                write_traffic(&container).await?;
            }
            // ---
        }
        Command::Clones => {
            let repo: GitHubRepo = cli
                .sub_commands
                .first()
                .ok_or_print_help("No repository provided.")?
                .parse()?;

            let (weekly, daily) = tokio::join!(
                github::api::fetch_clones(&repo, github::api::Frequency::Week),
                github::api::fetch_clones(&repo, github::api::Frequency::Day)
            );
            // --- TODO better
            if let Ok(container) = weekly {
                write_clones(&container).await?;
            }
            if let Ok(container) = daily {
                write_clones(&container).await?;
            }
            // ---
        }
    }

    Ok(())
}

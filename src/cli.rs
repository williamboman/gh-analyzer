use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    str::FromStr,
};
use thiserror::Error;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_help() {
    println!(
        r#"Usage:
{} [-v] [--version] [-h] [--help] --out-dir=<out_dir> <command> <repo>

These are the available commands:

    traffic    Fetch traffic data.
    clones     Fetch clones data.
    repo       Fetch repo data (stars, forks, watchers, watchers)
"#,
        PKG_NAME
    );
    print_version();
}

pub fn print_version() {
    println!("{} v{}", PKG_NAME, PKG_VERSION);
}

pub type Flags = BTreeSet<String>;
pub type Options = BTreeMap<String, String>;
pub type Commands = Vec<String>;

#[derive(Debug)]
pub struct Cli {
    pub commands: Commands,
    pub flags: Flags,
    pub options: Options,
}

#[derive(Debug)]
enum ParsedArg {
    Flag(String),
    Option(String, String),
    Argument(String),
}

#[derive(Error, Debug)]
pub enum CliError {
    BadInput(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadInput(x) => f.write_str(x),
        }
    }
}

impl FromStr for ParsedArg {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("--") {
            let (key, value) = s[2..]
                .split_once('=')
                .or(Some((&s[2..], "")))
                .ok_or(CliError::BadInput("Failed to parse option".to_owned()))?;
            Ok(Self::Option(key.to_owned(), value.to_owned()))
        } else if s.starts_with("-") {
            match s.len() {
                2 => {
                    let key = &s[1..2];
                    Ok(Self::Flag(key.to_owned()))
                }
                _ => Err(CliError::BadInput(format!("Invalid option format: {}", s))),
            }
        } else {
            Ok(Self::Argument(s.to_owned()))
        }
    }
}

pub fn init(args: &mut std::env::Args) -> Result<Cli> {
    let mut flags = Flags::new();
    let mut options = Options::new();
    let mut commands = Commands::new();

    for arg in args.skip(1) {
        let parsed: ParsedArg = arg.parse()?;
        match parsed {
            ParsedArg::Flag(key) => {
                flags.insert(key);
            }
            ParsedArg::Argument(arg) => {
                commands.push(arg);
            }
            ParsedArg::Option(key, value) => {
                options.insert(key, value);
            }
        }
    }

    Ok(Cli {
        commands,
        options,
        flags,
    })
}

use anyhow::{anyhow, Result};
use std::{collections::BTreeMap, str::FromStr};

pub fn print_help() {
    print!(
        r#"Usage:
$ gh-analyzer <command> <repo>

These are the available commands:

    traffic    Fetch traffic data.
    clones     Fetch clones data.

"#
    )
}

pub trait PrintHelp<T> {
    fn ok_or_print_help(self, error_msg: &'static str) -> Result<T>;
}

impl<T> PrintHelp<T> for Option<T> {
    fn ok_or_print_help(self, error_msg: &'static str) -> Result<T> {
        return match self {
            Some(v) => Ok(v),
            None => {
                print_help();
                Err(anyhow!(error_msg))
            }
        };
    }
}

type Flags = BTreeMap<String, Option<String>>;

#[derive(Debug)]
pub struct Cli {
    pub command: Option<String>,
    pub sub_commands: Vec<String>,
    pub flags: Flags,
}

#[derive(Debug)]
enum ParsedArg {
    Flag { key: String, value: Option<String> },
    Argument(String),
}

impl FromStr for ParsedArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with("--") {
            let (key, _value) = s[2..]
                .split_once('=')
                .or(Some((&s[2..], "")))
                .ok_or_print_help("Failed to parse option.")?;
            Ok(Self::Flag {
                key: key.to_owned(),
                value: Some(key.to_owned()),
            })
        } else if s.starts_with("-") {
            let key = &s[1..];
            Ok(Self::Flag {
                key: key.to_owned(),
                value: None,
            })
        } else {
            Ok(Self::Argument(s.to_owned()))
        }
    }
}

pub fn init(args: &mut std::env::Args) -> Result<Cli> {
    let mut flags: Flags = Flags::new();
    let mut commands: Vec<String> = Vec::new();
    for arg in args.skip(1) {
        let parsed: ParsedArg = arg.parse()?;
        match parsed {
            ParsedArg::Flag { key, value } => {
                flags.insert(key, value);
            }
            ParsedArg::Argument(arg) => {
                commands.push(arg);
            }
        }
    }

    Ok(Cli {
        command: commands.first().map(String::to_owned),
        sub_commands: if commands.len() > 1 {
            commands[1..].to_vec()
        } else {
            Vec::with_capacity(0)
        },
        flags,
    })
}

use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Result};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::StdResult;

type Timezone = String;

#[derive(Debug)]
pub struct ISO8601Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub tz: Timezone,
    raw: String,
}

impl<'a> Deserialize<'a> for ISO8601Date {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for ISO8601Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.raw.serialize(serializer)
    }
}

impl Display for ISO8601Date {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&self.raw)
    }
}

// TODO generics, macro?
fn parse_u16(s: &str) -> Option<u16> {
    s.parse().ok()
}
fn parse_u8(s: &str) -> Option<u8> {
    s.parse().ok()
}

fn remove_padding(s: &str) -> Option<&str> {
    let trimmed_str = s.trim_start_matches('0');
    Some(if trimmed_str.len() == 0 {
        "0"
    } else {
        trimmed_str
    })
}

impl ISO8601Date {
    pub fn as_date_str(&self) -> String {
        format!("{}-{:02}-{:02}", self.year, self.month, self.day)
    }

    fn parse_date_component(s: &str) -> Result<(u16, u8, u8)> {
        let (date, _) = s
            .split_once("T")
            .ok_or_else(|| anyhow!("Failed to parse date ({}).", s))?;
        let mut date_components = date.split("-");

        let year = date_components
            .next()
            .and_then(remove_padding)
            .and_then(parse_u16);
        let month = date_components
            .next()
            .and_then(remove_padding)
            .and_then(parse_u8);
        let day = date_components
            .next()
            .and_then(remove_padding)
            .and_then(parse_u8);

        if let (Some(year), Some(month), Some(day)) = (year, month, day) {
            Ok((year, month, day))
        } else {
            Err(anyhow!("Failed to parse year, month, date ({}).", s))
        }
    }

    fn parse_time_component(s: &str) -> Result<(u8, u8, u8, String)> {
        let (_, timestamp) = s
            .split_once("T")
            .ok_or_else(|| anyhow!("Failed to parse time component ({}).", s))?;
        let mut time_components = timestamp[..8].split(":");
        let parse_u8 = |s: &str| -> Option<u8> { s.parse().ok() };
        let hours = time_components
            .next()
            .and_then(remove_padding)
            .and_then(parse_u8);
        let minutes = time_components
            .next()
            .and_then(remove_padding)
            .and_then(parse_u8);
        let seconds = time_components
            .next()
            .and_then(remove_padding)
            .and_then(parse_u8);
        let tz = timestamp.get(8..);
        if let (Some(hours), Some(minutes), Some(seconds), Some(tz)) = (hours, minutes, seconds, tz)
        {
            Ok((hours, minutes, seconds, tz.to_owned()))
        } else {
            Err(anyhow!(
                "Failed to parse hours, minutes, seconds, tz ({}).",
                s
            ))
        }
    }
}

impl FromStr for ISO8601Date {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let (year, month, day) = ISO8601Date::parse_date_component(s)?;
        let (hours, minutes, seconds, tz) = ISO8601Date::parse_time_component(s)?;
        Ok(ISO8601Date {
            year,
            month,
            day,
            hours,
            minutes,
            seconds,
            tz,
            raw: s.to_owned(),
        })
    }
}
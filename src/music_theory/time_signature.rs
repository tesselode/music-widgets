use std::fmt::Display;

use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "&str")]
pub struct TimeSignature {
	pub top: u32,
	pub bottom: u32,
}

impl Display for TimeSignature {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}/{}", self.top, self.bottom))
	}
}

impl TryFrom<&str> for TimeSignature {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> anyhow::Result<Self> {
		let regex = Regex::new("(\\d)+/(\\d)+").unwrap();
		let captures = regex.captures(value).unwrap();
		let top = captures[1]
			.parse()
			.map_err(|_| anyhow!("{} is not a valid time signature", value))?;
		let bottom = captures[2]
			.parse()
			.map_err(|_| anyhow!("{} is not a valid time signature", value))?;
		Ok(Self { top, bottom })
	}
}

impl From<TimeSignature> for String {
	fn from(value: TimeSignature) -> Self {
		format!("{}", value)
	}
}

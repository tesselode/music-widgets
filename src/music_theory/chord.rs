use std::fmt::Display;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Note;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "&str")]
pub struct Chord {
	pub note: Note,
	pub text: String,
}

impl Display for Chord {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}{}", self.note, self.text))
	}
}

impl From<Chord> for String {
	fn from(value: Chord) -> Self {
		format!("{}", value)
	}
}

impl TryFrom<&str> for Chord {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> anyhow::Result<Self> {
		let regex = Regex::new("([abcdefgABCDEFG][#b]?)(.*)").unwrap();
		let captures = regex.captures(value).unwrap();
		let note = captures[1].try_into()?;
		let text = captures[2].to_string();
		Ok(Self { note, text })
	}
}

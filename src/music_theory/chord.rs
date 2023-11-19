use regex::Regex;
use serde::Deserialize;

use super::Note;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "&str")]
pub struct Chord {
	pub note: Note,
	pub text: String,
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

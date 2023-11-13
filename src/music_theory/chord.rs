use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use super::Note;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "&str")]
pub struct Chord {
	pub note: Note,
	pub text: String,
}

impl TryFrom<&str> for Chord {
	type Error = InvalidChord;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let regex = Regex::new("([abcdefgABCDEFG][#b]?)(.*)").unwrap();
		let captures = regex.captures(value).unwrap();
		let note = captures[1]
			.try_into()
			.map_err(|_| InvalidChord(value.to_string()))?;
		let text = captures[2].to_string();
		Ok(Self { note, text })
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("'{0}' is not a valid chord")]
pub struct InvalidChord(pub String);

use regex::Regex;
use thiserror::Error;

use super::{Accidental, NoteName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
	pub note_name: NoteName,
	pub accidental: Accidental,
}

impl TryFrom<&str> for Note {
	type Error = InvalidNote;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let regex = Regex::new("([abcdefgABCDEFG])([b#]?)").unwrap();
		let captures = regex
			.captures(value)
			.ok_or_else(|| InvalidNote(value.to_string()))?;
		let note_name = captures[1]
			.try_into()
			.map_err(|_| InvalidNote(value.to_string()))?;
		let accidental = captures[2]
			.try_into()
			.map_err(|_| InvalidNote(value.to_string()))?;
		Ok(Self {
			note_name,
			accidental,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("'{0}' is not a valid note")]
pub struct InvalidNote(pub String);

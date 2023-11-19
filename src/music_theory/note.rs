use anyhow::anyhow;
use regex::Regex;

use super::{Accidental, NoteName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
	pub note_name: NoteName,
	pub accidental: Accidental,
}

impl TryFrom<&str> for Note {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> anyhow::Result<Self> {
		let regex = Regex::new("([abcdefgABCDEFG])([b#]?)").unwrap();
		let captures = regex
			.captures(value)
			.ok_or_else(|| anyhow!("{} is not a valid note", value))?;
		let note_name = captures[1].try_into()?;
		let accidental = captures[2].try_into()?;
		Ok(Self {
			note_name,
			accidental,
		})
	}
}

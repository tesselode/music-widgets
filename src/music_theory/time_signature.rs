use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "&str")]
pub struct TimeSignature {
	pub top: u8,
	pub bottom: u8,
}

impl TryFrom<&str> for TimeSignature {
	type Error = InvalidTimeSignature;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let regex = Regex::new("(\\d)+/(\\d)+").unwrap();
		let captures = regex.captures(value).unwrap();
		let top = captures[1]
			.parse()
			.map_err(|_| InvalidTimeSignature(value.to_string()))?;
		let bottom = captures[2]
			.parse()
			.map_err(|_| InvalidTimeSignature(value.to_string()))?;
		Ok(Self { top, bottom })
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("'{0}' is not a valid time signature")]
pub struct InvalidTimeSignature(pub String);

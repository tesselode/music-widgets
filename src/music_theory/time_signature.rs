use anyhow::anyhow;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "&str")]
pub struct TimeSignature {
	pub top: u32,
	pub bottom: u32,
}

impl ToString for TimeSignature {
	fn to_string(&self) -> String {
		format!("{}/{}", self.top, self.bottom)
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

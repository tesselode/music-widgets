use std::fmt::Display;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
	None,
	Flat,
	Sharp,
}

impl Display for Accidental {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Accidental::None => "",
			Accidental::Flat => "b",
			Accidental::Sharp => "#",
		})
	}
}

impl TryFrom<&str> for Accidental {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> anyhow::Result<Self> {
		match value {
			"" => Ok(Self::None),
			"b" => Ok(Self::Flat),
			"#" => Ok(Self::Sharp),
			_ => bail!("{} is not a valid accidental", value),
		}
	}
}

use std::fmt::{Display, Write};

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteName {
	A,
	B,
	C,
	D,
	E,
	F,
	G,
}

impl Display for NoteName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_char(match self {
			NoteName::A => 'A',
			NoteName::B => 'B',
			NoteName::C => 'C',
			NoteName::D => 'D',
			NoteName::E => 'E',
			NoteName::F => 'F',
			NoteName::G => 'G',
		})
	}
}

impl TryFrom<&str> for NoteName {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> anyhow::Result<Self> {
		match value.to_lowercase().as_str() {
			"a" => Ok(Self::A),
			"b" => Ok(Self::B),
			"c" => Ok(Self::C),
			"d" => Ok(Self::D),
			"e" => Ok(Self::E),
			"f" => Ok(Self::F),
			"g" => Ok(Self::G),
			_ => bail!("{} is not a valid note name", value),
		}
	}
}

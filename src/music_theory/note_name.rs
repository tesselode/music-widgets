use thiserror::Error;

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

impl TryFrom<&str> for NoteName {
	type Error = InvalidNoteName;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value.to_lowercase().as_str() {
			"a" => Ok(Self::A),
			"b" => Ok(Self::B),
			"c" => Ok(Self::C),
			"d" => Ok(Self::D),
			"e" => Ok(Self::E),
			"f" => Ok(Self::F),
			"g" => Ok(Self::G),
			_ => Err(InvalidNoteName(value.to_string())),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("'{0}' is not a valid note name")]
pub struct InvalidNoteName(pub String);

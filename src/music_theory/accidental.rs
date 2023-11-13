use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
	None,
	Flat,
	Sharp,
}

impl TryFrom<&str> for Accidental {
	type Error = InvalidAccidental;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"" => Ok(Self::None),
			"b" => Ok(Self::Flat),
			"#" => Ok(Self::Sharp),
			_ => Err(InvalidAccidental(value.to_string())),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("'{0}' is not a valid accidental")]
pub struct InvalidAccidental(pub String);

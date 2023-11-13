use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::music_theory::{Chord, TimeSignature};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct TrackInfo {
	pub bpm: u16,
	pub ticks_per_beat: u16,
	pub time_signature: Option<TimeSignature>,
	pub key: Option<Chord>,
	pub chord: Option<Chord>,
	#[serde(default)]
	pub changes: Vec<Change>,
}

impl TrackInfo {
	pub fn from_file(path: impl AsRef<Path>) -> Result<Self, TrackInfoFromFileError> {
		let track_info_string = std::fs::read_to_string(path)?;
		let track_info = serde_json::from_str(&track_info_string)?;
		Ok(track_info)
	}
}

#[derive(Debug, Error)]
pub enum TrackInfoFromFileError {
	#[error("{0}")]
	IoError(#[from] std::io::Error),
	#[error("{0}")]
	ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Change {
	pub tick: u32,
	pub bpm: Option<u16>,
	pub time_signature: Option<TimeSignature>,
	pub key: Option<Chord>,
	pub chord: Option<Chord>,
}

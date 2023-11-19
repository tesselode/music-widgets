use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::{
	music_state::MusicState,
	music_theory::{Chord, TimeSignature},
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UserTrackInfo {
	#[serde(flatten)]
	pub initial_state: MusicState,
	pub ticks_per_beat: u32,
	#[serde(default)]
	pub changes: Vec<Change>,
}

impl UserTrackInfo {
	pub fn from_file(path: impl AsRef<Path>) -> Result<Self, UserTrackInfoFromFileError> {
		let track_info_string = std::fs::read_to_string(path)?;
		let track_info = serde_json::from_str(&track_info_string)?;
		Ok(track_info)
	}
}

#[derive(Debug, Error)]
pub enum UserTrackInfoFromFileError {
	#[error("{0}")]
	IoError(#[from] std::io::Error),
	#[error("{0}")]
	ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Change {
	pub after: u32,
	pub bpm: Option<f64>,
	pub time_signature: Option<TimeSignature>,
	pub key: Option<Chord>,
	pub chord: Option<Chord>,
}
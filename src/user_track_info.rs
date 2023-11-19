use std::path::Path;

use serde::Deserialize;

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
	pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let track_info_string = std::fs::read_to_string(path)?;
		let track_info = serde_json::from_str(&track_info_string)?;
		Ok(track_info)
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Change {
	pub after: u32,
	pub bpm: Option<f64>,
	pub time_signature: Option<TimeSignature>,
	pub key: Option<Chord>,
	pub chord: Option<Chord>,
}

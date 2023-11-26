use serde::{Deserialize, Serialize};

use crate::{music_state::MusicState, music_theory::TimeSignature};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserTrackInfo {
	#[serde(flatten)]
	pub initial_state: MusicState,
	pub ticks_per_beat: u32,
	#[serde(default)]
	pub changes: Vec<Change>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
	pub after: u32,
	pub bpm: Option<f64>,
	pub time_signature: Option<TimeSignature>,
	pub key: Option<String>,
	pub chord: Option<String>,
}

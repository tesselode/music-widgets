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
	pub bpm_hidden: Option<bool>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "::serde_with::rust::double_option"
	)]
	pub time_signature: Option<Option<TimeSignature>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "::serde_with::rust::double_option"
	)]
	pub key: Option<Option<String>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "::serde_with::rust::double_option"
	)]
	pub chord: Option<Option<String>>,
}

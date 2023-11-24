use serde::{Deserialize, Serialize};

use crate::{
	music_theory::{Chord, TimeSignature},
	user_track_info::Change,
};

/// Basic info about music at an instant in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicState {
	pub bpm: f64,
	pub time_signature: TimeSignature,
	pub key: Chord,
	pub chord: Chord,
}

impl MusicState {
	pub fn changed(&self, change: &Change) -> Self {
		let mut new = self.clone();
		if let Some(new_bpm) = change.bpm {
			new.bpm = new_bpm;
		}
		if let Some(new_time_signature) = change.time_signature {
			new.time_signature = new_time_signature;
		}
		if let Some(new_key) = &change.key {
			new.key = new_key.clone();
		}
		if let Some(new_chord) = &change.chord {
			new.chord = new_chord.clone();
		}
		new
	}
}

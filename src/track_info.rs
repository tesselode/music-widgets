use std::time::Duration;

use crate::{music_state::MusicState, user_track_info::UserTrackInfo};

pub struct TrackInfo {
	pub music_states: Vec<TimestampedMusicState>,
}

impl TrackInfo {
	pub fn new(user_track_info: &UserTrackInfo) -> Self {
		let mut music_states = vec![TimestampedMusicState {
			timestamp: Duration::ZERO,
			music_state: user_track_info.initial_state.clone(),
		}];
		let mut bpm = music_states[0].music_state.bpm;
		let mut last_state_timestamp = Duration::ZERO;
		for change in &user_track_info.changes {
			let timestamp = last_state_timestamp
				+ change.after * tick_duration(bpm, user_track_info.ticks_per_beat);
			let new_state = music_states.last().unwrap().music_state.changed(change);
			bpm = new_state.bpm;
			last_state_timestamp = timestamp;
			music_states.push(TimestampedMusicState {
				timestamp,
				music_state: new_state,
			});
		}
		Self { music_states }
	}

	pub fn music_state(&self, timestamp: Duration) -> &TimestampedMusicState {
		self.music_states
			.iter()
			.rev()
			.find(|state| state.timestamp <= timestamp)
			.unwrap()
	}

	pub fn current_beat(&self, timestamp: Duration) -> f64 {
		let state = self.music_state(timestamp);
		let beats_per_second = state.music_state.bpm / 60.0;
		let total_beats_elapsed = (timestamp - state.timestamp).as_secs_f64() * beats_per_second;
		total_beats_elapsed % state.music_state.time_signature.top as f64
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimestampedMusicState {
	pub timestamp: Duration,
	pub music_state: MusicState,
}

fn tick_duration(bpm: f64, ticks_per_beat: u32) -> Duration {
	Duration::from_secs_f64(60.0 / bpm / ticks_per_beat as f64)
}

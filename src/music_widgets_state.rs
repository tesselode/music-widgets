use std::time::Duration;

use crate::{
	music_theory::{Chord, TimeSignature},
	track_info::{Change, TrackInfo},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MusicWidgetsState {
	pub track_info: TrackInfo,
	pub bpm: u32,
	pub ticks_per_beat: u32,
	pub time_signature: TimeSignature,
	pub key: Chord,
	pub chord: Chord,
	pub current_tick: u32,
	pub time_since_last_tick: Duration,
	pub last_time_signature_change_tick: u32,
}

impl MusicWidgetsState {
	pub fn new(track_info: TrackInfo) -> Self {
		Self {
			bpm: track_info.bpm,
			ticks_per_beat: track_info.ticks_per_beat,
			time_signature: track_info.time_signature,
			key: track_info.key.clone(),
			chord: track_info.chord.clone(),
			current_tick: 0,
			time_since_last_tick: Duration::ZERO,
			track_info,
			last_time_signature_change_tick: 0,
		}
	}

	pub fn update(&mut self, delta_time: Duration) {
		self.time_since_last_tick += delta_time;
		let tick_duration = self.tick_duration();
		while self.time_since_last_tick >= tick_duration {
			self.time_since_last_tick -= tick_duration;
			self.current_tick += 1;
			if let Some(Change {
				bpm,
				time_signature,
				key,
				chord,
				..
			}) = self.track_info.changes.get(&self.current_tick)
			{
				if let Some(new_bpm) = bpm {
					self.bpm = *new_bpm;
				}
				if let Some(new_time_signature) = time_signature {
					self.time_signature = *new_time_signature;
					self.last_time_signature_change_tick = self.current_tick;
				}
				if let Some(new_key) = key {
					self.key = new_key.clone();
				}
				if let Some(new_chord) = chord {
					self.chord = new_chord.clone();
				}
			}
		}
	}

	pub fn current_beat(&self) -> u32 {
		((self.current_tick - self.last_time_signature_change_tick) / self.ticks_per_beat)
			% self.time_signature.top
	}

	fn tick_duration(&self) -> Duration {
		Duration::from_secs_f64(60.0 / self.bpm as f64 / self.ticks_per_beat as f64)
	}
}

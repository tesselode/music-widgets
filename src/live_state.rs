use std::{path::Path, time::Duration};

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundHandle,
	tween::Tween,
};
use micro::Context;

use crate::{loaded_project::LoadedProject, rendering_state::RenderingState};

pub struct LiveState {
	pub loaded_project: LoadedProject,
	pub audio_manager: AudioManager,
	pub playing_sound: Option<StaticSoundHandle>,
	pub time_elapsed: Duration,
}

impl LiveState {
	pub fn new(ctx: &mut Context, project_path: impl AsRef<Path>) -> anyhow::Result<Self> {
		Self::from_loaded_project(LoadedProject::load(ctx, project_path)?)
	}

	pub fn from_loaded_project(loaded_project: LoadedProject) -> anyhow::Result<Self> {
		Ok(Self {
			loaded_project,
			audio_manager: AudioManager::new(AudioManagerSettings::default())?,
			playing_sound: None,
			time_elapsed: Duration::ZERO,
		})
	}

	pub fn set_playing(&mut self, playing: bool) -> anyhow::Result<()> {
		if playing {
			self.playing_sound = Some(self.audio_manager.play(
				self.loaded_project.sound_data.with_modified_settings(|s| {
					s.playback_region(self.time_elapsed.as_secs_f64()..)
				}),
			)?);
		} else {
			if let Some(playing_sound) = &mut self.playing_sound {
				playing_sound.stop(Tween::default())?;
			}
			self.playing_sound = None;
		}
		Ok(())
	}

	pub fn seek(&mut self, time: Duration, seek_audio: bool) -> anyhow::Result<()> {
		self.time_elapsed = time;
		if seek_audio {
			if let Some(playing_sound) = &mut self.playing_sound {
				playing_sound.set_playback_region(..)?;
				playing_sound.seek_to(time.as_secs_f64())?;
			}
		}
		Ok(())
	}

	pub fn start_rendering(
		self,
		ctx: &mut Context,
		output_path: impl AsRef<Path>,
	) -> anyhow::Result<RenderingState> {
		RenderingState::new(ctx, self.loaded_project, output_path)
	}
}

use std::path::{Path, PathBuf};

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use micro::Context;

use crate::{loaded_shader::LoadedShader, project::Project, track_info::TrackInfo};

pub struct LoadedProject {
	pub sound_data: StaticSoundData,
	pub audio_path: PathBuf,
	pub shader: Option<LoadedShader>,
	pub track_info: TrackInfo,
}

impl LoadedProject {
	pub fn load(ctx: &mut Context, project_path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let project_path = project_path.as_ref();
		let project = Project::from_file(project_path)?;
		let audio_path = project_path.parent().unwrap().join(project.audio_file_path);
		let shader = project
			.shader_path
			.map(|shader_path| {
				let shader_full_path = project_path.parent().unwrap().join(shader_path);
				LoadedShader::load(ctx, shader_full_path)
			})
			.transpose()?;
		Ok(Self {
			sound_data: StaticSoundData::from_file(&audio_path, StaticSoundSettings::default())?,
			audio_path,
			shader,
			track_info: TrackInfo::new(&project.track_info),
		})
	}
}

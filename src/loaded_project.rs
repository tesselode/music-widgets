use std::path::{Path, PathBuf};

use anyhow::Context as AnyhowContext;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use micro::Context;

use crate::{
	loaded_shader::LoadedShader, project::Project, shader_param::ShaderParam, track_info::TrackInfo,
};

pub struct LoadedProject {
	pub project: Project,
	pub project_path: PathBuf,
	pub sound_data: StaticSoundData,
	pub audio_path: PathBuf,
	pub shader: Option<LoadedShader>,
	pub shader_params: Vec<ShaderParam>,
	pub track_info: TrackInfo,
}

impl LoadedProject {
	pub fn load(ctx: &mut Context, project_path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let project_path = project_path.as_ref();
		let project = Project::from_file(project_path).context("error loading project")?;
		let audio_path = project_path
			.parent()
			.unwrap()
			.join(&project.audio_file_path);
		let shader = project
			.shader_path
			.as_ref()
			.map(|shader_path| {
				let shader_full_path = project_path.parent().unwrap().join(shader_path);
				LoadedShader::load(ctx, shader_full_path).context("error loading shader")
			})
			.transpose()?;
		let shader_params = project.shader_params.clone();
		let track_info = TrackInfo::new(&project.track_info);
		Ok(Self {
			project,
			project_path: project_path.to_path_buf(),
			sound_data: StaticSoundData::from_file(&audio_path, StaticSoundSettings::default())
				.context("error loading audio")?,
			audio_path,
			shader,
			shader_params,
			track_info,
		})
	}
}

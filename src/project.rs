use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{shader_param::ShaderParam, user_track_info::UserTrackInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
	pub audio_file_path: PathBuf,
	pub shader_path: Option<PathBuf>,
	#[serde(default)]
	pub shader_params: Vec<ShaderParam>,
	pub track_info: UserTrackInfo,
}

impl Project {
	pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let project_string = std::fs::read_to_string(path)?;
		let project = serde_json::from_str(&project_string)?;
		Ok(project)
	}
}

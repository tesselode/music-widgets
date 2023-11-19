use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::user_track_info::UserTrackInfo;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Project {
	pub audio_file_path: PathBuf,
	pub shader_path: Option<PathBuf>,
	pub track_info: UserTrackInfo,
}

impl Project {
	pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let project_string = std::fs::read_to_string(path)?;
		let project = serde_json::from_str(&project_string)?;
		Ok(project)
	}
}

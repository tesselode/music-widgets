use std::{
	path::{Path, PathBuf},
	time::{Duration, SystemTime},
};

use micro::{graphics::shader::Shader, Context};

use crate::BASE_RESOLUTION;

pub struct LoadedShader {
	pub shader: Shader,
	pub path: PathBuf,
	pub last_modified_time: SystemTime,
	pub time_since_last_hot_reload: Duration,
}

impl LoadedShader {
	const HOT_RELOAD_INTERVAL: Duration = Duration::from_secs(1);

	pub fn load(ctx: &Context, path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let path = path.as_ref();
		let shader = Shader::from_fragment_file(ctx, path)?;
		shader.send_vec2("iResolution", BASE_RESOLUTION.as_vec2())?;
		let last_modified_time = std::fs::metadata(path)?.modified()?;
		Ok(Self {
			shader,
			path: path.to_path_buf(),
			last_modified_time,
			time_since_last_hot_reload: Duration::ZERO,
		})
	}

	pub fn update_hot_reload(&mut self, ctx: &Context, delta_time: Duration) -> anyhow::Result<()> {
		self.time_since_last_hot_reload += delta_time;
		while self.time_since_last_hot_reload >= Self::HOT_RELOAD_INTERVAL {
			let last_modified_time = std::fs::metadata(&self.path)?.modified()?;
			if last_modified_time > self.last_modified_time {
				self.shader = Shader::from_fragment_file(ctx, &self.path)?;
				self.shader
					.send_vec2("iResolution", BASE_RESOLUTION.as_vec2())?;
			}
			self.time_since_last_hot_reload -= delta_time;
		}
		Ok(())
	}
}

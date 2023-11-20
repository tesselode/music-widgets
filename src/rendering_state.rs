use std::{
	path::Path,
	process::{Child, Command, Stdio},
};

use micro::{graphics::SwapInterval, Context};

use crate::{live_state::LiveState, loaded_project::LoadedProject, BASE_RESOLUTION, EXPORT_FPS};

pub struct RenderingState {
	pub loaded_project: LoadedProject,
	pub current_frame: u32,
	pub canvas_read_buffer: Vec<u8>,
	pub ffmpeg_process: Child,
}

impl RenderingState {
	pub fn new(
		ctx: &mut Context,
		loaded_project: LoadedProject,
		output_path: impl AsRef<Path>,
	) -> anyhow::Result<Self> {
		ctx.set_swap_interval(SwapInterval::Immediate)?;
		let ffmpeg_process = Command::new("ffmpeg")
			.stdin(Stdio::piped())
			.arg("-y")
			.arg("-f")
			.arg("rawvideo")
			.arg("-vcodec")
			.arg("rawvideo")
			.arg("-s")
			.arg(&format!("{}x{}", BASE_RESOLUTION.x, BASE_RESOLUTION.y))
			.arg("-pix_fmt")
			.arg("rgba")
			.arg("-r")
			.arg(EXPORT_FPS.to_string())
			.arg("-i")
			.arg("-")
			.arg("-i")
			.arg(&loaded_project.audio_path)
			.arg("-b:a")
			.arg("320k")
			.arg("-c:v")
			.arg("libx264")
			.arg("-r")
			.arg(EXPORT_FPS.to_string())
			.arg("-shortest")
			.arg(output_path.as_ref())
			.spawn()?;
		Ok(Self {
			loaded_project,
			current_frame: 0,
			canvas_read_buffer: vec![0; (BASE_RESOLUTION.x * BASE_RESOLUTION.y * 4) as usize],
			ffmpeg_process,
		})
	}

	pub fn cancel(mut self) -> anyhow::Result<LiveState> {
		self.ffmpeg_process.kill().ok();
		LiveState::from_loaded_project(self.loaded_project)
	}
}

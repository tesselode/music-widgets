mod format;
mod music_state;
mod music_theory;
mod project;
mod track_info;
mod user_track_info;
mod widgets;

use std::{
	path::{Path, PathBuf},
	time::{Duration, SystemTime},
};

use clap::Parser;
use egui::{Slider, TopBottomPanel};
use format::{format_time, parse_time};
use glam::{UVec2, Vec2};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use micro::{
	graphics::{
		mesh::Mesh,
		shader::Shader,
		text::{Font, FontSettings},
		Canvas, CanvasSettings, DrawParams,
	},
	input::Scancode,
	math::Rect,
	Context, ContextSettings, Event, State, WindowMode,
};
use palette::LinSrgba;
use project::Project;
use rfd::{FileDialog, MessageDialog, MessageLevel};
use track_info::TrackInfo;
use widgets::{draw_bpm_panel, draw_metronome_panel};

const BASE_RESOLUTION: UVec2 = UVec2::new(3840, 2160);
const OFFWHITE: LinSrgba = LinSrgba::new(0.8, 0.8, 0.8, 1.0);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Parser)]
struct Args {
	project_path: Option<PathBuf>,
}

fn main() {
	micro::run(
		ContextSettings {
			window_mode: WindowMode::Windowed {
				size: UVec2::new(2560, 1440),
			},
			..Default::default()
		},
		MainState::new,
	)
}

struct MainState {
	loaded_project: Option<LoadedProject>,
	playing: bool,
	time_elapsed: Duration,
	fonts: Fonts,
	canvas: Canvas,
	// canvas_read_buffer: Vec<u8>,
	// ffmpeg_process: Child,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		let args = Args::parse();
		Ok(Self {
			loaded_project: args
				.project_path
				.map(|project_path| LoadedProject::load(ctx, project_path))
				.transpose()?,
			playing: false,
			time_elapsed: Duration::ZERO,
			fonts: Fonts {
				small: Font::from_file(
					ctx,
					"resources/traceroute.ttf",
					FontSettings {
						scale: 64.0,
						..Default::default()
					},
				)?,
				large: Font::from_file(
					ctx,
					"resources/traceroute.ttf",
					FontSettings {
						scale: 128.0,
						..Default::default()
					},
				)?,
			},
			canvas: Canvas::new(ctx, BASE_RESOLUTION, CanvasSettings::default()),
			// canvas_read_buffer: vec![0; (BASE_RESOLUTION.x * BASE_RESOLUTION.y * 4) as usize],
			/* ffmpeg_process: Command::new("ffmpeg")
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
			.arg("60")
			.arg("-i")
			.arg("-")
			.arg("-i")
			.arg("continuum.flac")
			.arg("-b:a")
			.arg("264k")
			.arg("-c:v")
			.arg("libx264")
			.arg("-r")
			.arg("60")
			.arg("-shortest")
			.arg("test.mp4")
			.spawn()?, */
		})
	}
}

impl State<anyhow::Error> for MainState {
	fn ui(&mut self, ctx: &mut Context, egui_ctx: &egui::Context) -> Result<(), anyhow::Error> {
		TopBottomPanel::bottom("main_menu").show(egui_ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				if ui.button("Load").clicked() {
					if let Some(project_path) = FileDialog::new()
						.set_directory(std::env::current_exe().unwrap())
						.add_filter("Project file", &["json"])
						.pick_file()
					{
						match LoadedProject::load(ctx, project_path) {
							Ok(loaded_project) => {
								self.loaded_project = Some(loaded_project);
								self.playing = false;
								self.time_elapsed = Duration::ZERO;
							}
							Err(err) => {
								MessageDialog::new()
									.set_level(MessageLevel::Error)
									.set_description(format!("Error loading project: {}", err))
									.show();
							}
						}
					}
				}
				if let Some(LoadedProject { sound_data, .. }) = &self.loaded_project {
					ui.checkbox(&mut self.playing, "Playing");
					let mut time_elapsed_f64 = self.time_elapsed.as_secs_f64();
					let position_slider = Slider::new(
						&mut time_elapsed_f64,
						0.0..=sound_data.duration().as_secs_f64(),
					)
					.trailing_fill(true)
					.custom_formatter(format_time)
					.custom_parser(parse_time);
					ui.style_mut().spacing.slider_width = 200.0;
					if ui.add(position_slider).changed() {
						self.time_elapsed = Duration::from_secs_f64(time_elapsed_f64);
					}
				}
			});
		});
		Ok(())
	}

	fn event(&mut self, ctx: &mut Context, event: Event) -> Result<(), anyhow::Error> {
		if let Event::KeyPressed {
			key: Scancode::Escape,
			..
		} = event
		{
			ctx.quit();
		}
		Ok(())
	}

	fn update(&mut self, ctx: &mut Context, delta_time: Duration) -> Result<(), anyhow::Error> {
		if self.playing {
			self.time_elapsed += delta_time;
		}
		if let Some(LoadedProject {
			shader: Some(loaded_shader),
			..
		}) = &mut self.loaded_project
		{
			if let Err(err) = loaded_shader.update_hot_reload(ctx, delta_time) {
				MessageDialog::new()
					.set_level(MessageLevel::Error)
					.set_description(format!("Error hot reloading shader: {}", err))
					.show();
			}
			loaded_shader
				.shader
				.send_f32("iTime", self.time_elapsed.as_secs_f32())?;
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		self.canvas.render_to(ctx, |ctx| -> anyhow::Result<()> {
			ctx.clear(OFFWHITE);
			if let Some(LoadedProject {
				shader, track_info, ..
			}) = &self.loaded_project
			{
				if let Some(LoadedShader { shader, .. }) = shader {
					Mesh::rectangle(ctx, Rect::new(Vec2::ZERO, BASE_RESOLUTION.as_vec2()))
						.draw(ctx, shader);
				}
				draw_bpm_panel(
					ctx,
					track_info,
					self.time_elapsed,
					&self.fonts,
					Vec2::new(1.0, 1.0),
				)?;
				draw_metronome_panel(
					ctx,
					track_info,
					self.time_elapsed,
					&self.fonts,
					Vec2::new(1.0, 7.0),
				)?;
			}
			Ok(())
		})?;
		self.canvas.draw(
			ctx,
			DrawParams::new().scaled(ctx.window_size().as_vec2() / self.canvas.size().as_vec2()),
		);
		// self.canvas.read(&mut self.canvas_read_buffer);
		/* self.ffmpeg_process
		.stdin
		.as_mut()
		.unwrap()
		.write_all(&self.canvas_read_buffer)?; */
		Ok(())
	}
}

struct Fonts {
	small: Font,
	large: Font,
}

struct LoadedProject {
	sound_data: StaticSoundData,
	shader: Option<LoadedShader>,
	track_info: TrackInfo,
}

impl LoadedProject {
	fn load(ctx: &mut Context, project_path: impl AsRef<Path>) -> anyhow::Result<Self> {
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
			sound_data: StaticSoundData::from_file(audio_path, StaticSoundSettings::default())?,
			shader,
			track_info: TrackInfo::new(&project.track_info),
		})
	}
}

struct LoadedShader {
	shader: Shader,
	path: PathBuf,
	last_modified_time: SystemTime,
	time_since_last_hot_reload: Duration,
}

impl LoadedShader {
	const HOT_RELOAD_INTERVAL: Duration = Duration::from_secs(1);

	fn load(ctx: &Context, path: impl AsRef<Path>) -> anyhow::Result<Self> {
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

	fn update_hot_reload(&mut self, ctx: &Context, delta_time: Duration) -> anyhow::Result<()> {
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

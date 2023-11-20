mod format;
mod music_state;
mod music_theory;
mod project;
mod track_info;
mod ui;
mod user_track_info;
mod widgets;

use std::{
	io::Write,
	path::{Path, PathBuf},
	process::{Child, Command, Stdio},
	time::{Duration, SystemTime},
};

use clap::Parser;
use egui::TopBottomPanel;
use glam::{UVec2, Vec2};
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
	tween::Tween,
};
use micro::{
	graphics::{
		mesh::Mesh,
		shader::Shader,
		text::{Font, FontSettings},
		Canvas, CanvasSettings, DrawParams, SwapInterval,
	},
	input::Scancode,
	math::Rect,
	Context, ContextSettings, Event, State, WindowMode,
};
use palette::LinSrgba;
use project::Project;
use rfd::{MessageDialog, MessageLevel};
use track_info::TrackInfo;
use ui::{IdleModeMenuAction, LiveModeMenuAction, RenderingModeMenuAction};
use widgets::{draw_bpm_panel, draw_metronome_panel};

const BASE_RESOLUTION: UVec2 = UVec2::new(3840, 2160);
const EXPORT_FPS: f64 = 60.0;
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
	mode: Mode,
	fonts: Fonts,
	canvas: Canvas,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		let args = Args::parse();
		Ok(Self {
			mode: args
				.project_path
				.map(|project_path| -> anyhow::Result<Mode> {
					Ok(Mode::Live(LiveState::new(ctx, project_path)?))
				})
				.transpose()?
				.unwrap_or_default(),
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
		})
	}

	fn draw_canvas_contents(
		ctx: &mut Context,
		shader: &Option<LoadedShader>,
		fonts: &Fonts,
		track_info: &TrackInfo,
		time_elapsed: Duration,
	) -> Result<(), anyhow::Error> {
		if let Some(LoadedShader { shader, .. }) = shader {
			Mesh::rectangle(ctx, Rect::new(Vec2::ZERO, BASE_RESOLUTION.as_vec2()))
				.draw(ctx, shader);
		}
		draw_bpm_panel(ctx, track_info, time_elapsed, fonts, Vec2::new(1.0, 1.0))?;
		draw_metronome_panel(ctx, track_info, time_elapsed, fonts, Vec2::new(1.0, 7.0))?;
		Ok(())
	}
}

impl State<anyhow::Error> for MainState {
	fn ui(&mut self, ctx: &mut Context, egui_ctx: &egui::Context) -> Result<(), anyhow::Error> {
		TopBottomPanel::bottom("main_menu")
			.show(egui_ctx, |ui| -> anyhow::Result<()> {
				egui::menu::bar(ui, |ui| -> anyhow::Result<()> {
					match &mut self.mode {
						Mode::Idle => {
							if let Some(action) = Self::render_idle_mode_menu(ui) {
								match action {
									IdleModeMenuAction::LoadProject { path } => {
										self.mode = Mode::Live(LiveState::new(ctx, path)?);
									}
								}
							}
						}
						Mode::Live(live_state) => {
							if let Some(action) = Self::render_live_mode_menu(ui, live_state) {
								match action {
									LiveModeMenuAction::LoadProject { path } => {
										self.mode = Mode::Live(LiveState::new(ctx, path)?);
									}
									LiveModeMenuAction::StartRendering { output_path } => {
										let Mode::Live(live_state) = std::mem::take(&mut self.mode)
										else {
											unreachable!();
										};
										self.mode = Mode::Rendering(
											live_state.start_rendering(ctx, output_path)?,
										);
									}
									LiveModeMenuAction::SetPlaying(playing) => {
										live_state.set_playing(playing)?;
									}
									LiveModeMenuAction::Seek { time, seek_audio } => {
										live_state.seek(time, seek_audio)?;
									}
								}
							}
						}
						Mode::Rendering(rendering_state) => {
							if let Some(action) =
								Self::render_rendering_mode_menu(ui, rendering_state)
							{
								match action {
									RenderingModeMenuAction::CancelRendering => {
										let Mode::Rendering(rendering_state) =
											std::mem::take(&mut self.mode)
										else {
											unreachable!();
										};
										self.mode = Mode::Live(rendering_state.cancel()?);
									}
								}
							}
						}
					}
					Ok(())
				})
				.inner?;
				Ok(())
			})
			.inner?;
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
		match &mut self.mode {
			Mode::Live(LiveState {
				loaded_project: LoadedProject { shader, .. },
				playing_sound,
				time_elapsed,
				..
			}) => {
				if playing_sound.is_some() {
					*time_elapsed += delta_time;
				}
				if let Some(shader) = shader {
					if let Err(err) = shader.update_hot_reload(ctx, delta_time) {
						MessageDialog::new()
							.set_level(MessageLevel::Error)
							.set_description(format!("Error hot reloading shader: {}", err))
							.show();
					}
					shader
						.shader
						.send_f32("iTime", time_elapsed.as_secs_f32())?;
				}
			}
			Mode::Rendering(RenderingState {
				loaded_project: LoadedProject {
					shader: Some(shader),
					..
				},
				current_frame,
				..
			}) => {
				let time_elapsed = *current_frame * Duration::from_secs_f64(1.0 / EXPORT_FPS);
				shader
					.shader
					.send_f32("iTime", time_elapsed.as_secs_f32())?;
			}
			_ => (),
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		self.canvas.render_to(ctx, |ctx| -> anyhow::Result<()> {
			ctx.clear(OFFWHITE);
			match &self.mode {
				Mode::Idle => {}
				Mode::Live(LiveState {
					loaded_project: LoadedProject {
						shader, track_info, ..
					},
					time_elapsed,
					..
				}) => {
					Self::draw_canvas_contents(
						ctx,
						shader,
						&self.fonts,
						track_info,
						*time_elapsed,
					)?;
				}
				Mode::Rendering(RenderingState {
					loaded_project: LoadedProject {
						shader, track_info, ..
					},
					current_frame,
					..
				}) => {
					let time_elapsed = *current_frame * Duration::from_secs_f64(1.0 / EXPORT_FPS);
					Self::draw_canvas_contents(ctx, shader, &self.fonts, track_info, time_elapsed)?;
				}
			}
			Ok(())
		})?;
		self.canvas.draw(
			ctx,
			DrawParams::new().scaled(ctx.window_size().as_vec2() / self.canvas.size().as_vec2()),
		);
		if let Mode::Rendering(rendering_state) = &mut self.mode {
			self.canvas.read(&mut rendering_state.canvas_read_buffer);
			if rendering_state
				.ffmpeg_process
				.stdin
				.as_mut()
				.unwrap()
				.write_all(&rendering_state.canvas_read_buffer)
				.is_err()
			{
				let Mode::Rendering(rendering_state) = std::mem::take(&mut self.mode) else {
					unreachable!();
				};
				self.mode = Mode::Live(LiveState::from_loaded_project(
					rendering_state.loaded_project,
				)?);
			} else {
				rendering_state.current_frame += 1;
			}
		}
		Ok(())
	}
}

#[derive(Default)]
enum Mode {
	#[default]
	Idle,
	Live(LiveState),
	Rendering(RenderingState),
}

struct LiveState {
	loaded_project: LoadedProject,
	audio_manager: AudioManager,
	playing_sound: Option<StaticSoundHandle>,
	time_elapsed: Duration,
}

impl LiveState {
	fn new(ctx: &mut Context, project_path: impl AsRef<Path>) -> anyhow::Result<Self> {
		Self::from_loaded_project(LoadedProject::load(ctx, project_path)?)
	}

	fn from_loaded_project(loaded_project: LoadedProject) -> anyhow::Result<Self> {
		Ok(Self {
			loaded_project,
			audio_manager: AudioManager::new(AudioManagerSettings::default())?,
			playing_sound: None,
			time_elapsed: Duration::ZERO,
		})
	}

	fn set_playing(&mut self, playing: bool) -> anyhow::Result<()> {
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

	fn seek(&mut self, time: Duration, seek_audio: bool) -> anyhow::Result<()> {
		self.time_elapsed = time;
		if seek_audio {
			if let Some(playing_sound) = &mut self.playing_sound {
				playing_sound.set_playback_region(..)?;
				playing_sound.seek_to(time.as_secs_f64())?;
			}
		}
		Ok(())
	}

	fn start_rendering(
		self,
		ctx: &mut Context,
		output_path: impl AsRef<Path>,
	) -> anyhow::Result<RenderingState> {
		RenderingState::new(ctx, self.loaded_project, output_path)
	}
}

struct RenderingState {
	loaded_project: LoadedProject,
	current_frame: u32,
	canvas_read_buffer: Vec<u8>,
	ffmpeg_process: Child,
}

impl RenderingState {
	fn new(
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

	fn cancel(mut self) -> anyhow::Result<LiveState> {
		self.ffmpeg_process.kill().ok();
		LiveState::from_loaded_project(self.loaded_project)
	}
}

struct Fonts {
	small: Font,
	large: Font,
}

struct LoadedProject {
	sound_data: StaticSoundData,
	audio_path: PathBuf,
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
			sound_data: StaticSoundData::from_file(&audio_path, StaticSoundSettings::default())?,
			audio_path,
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

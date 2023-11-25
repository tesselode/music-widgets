mod format;
mod live_state;
mod loaded_project;
mod loaded_shader;
mod music_state;
mod music_theory;
mod project;
mod rendering_state;
mod shader_param;
mod track_info;
mod ui;
mod user_track_info;
mod widgets;

use std::{io::Write, path::PathBuf, time::Duration};

use clap::Parser;
use glam::{UVec2, Vec2};
use live_state::LiveState;
use loaded_project::LoadedProject;
use loaded_shader::LoadedShader;
use micro::{
	graphics::{
		mesh::Mesh,
		text::{Font, FontSettings},
		Canvas, CanvasSettings, DrawParams,
	},
	input::Scancode,
	math::Rect,
	Context, ContextSettings, Event, State, WindowMode,
};
use palette::LinSrgba;
use rendering_state::RenderingState;
use shader_param::ShaderParamKind;
use track_info::TrackInfo;
use ui::show_dialog_if_error;
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
	show_shader_params_editor: bool,
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
					"resources/IBMPlexMono-Italic.ttf",
					FontSettings {
						scale: 48.0,
						..Default::default()
					},
				)?,
				large: Font::from_file(
					ctx,
					"resources/IBMPlexMono-Bold.ttf",
					FontSettings {
						scale: 128.0,
						..Default::default()
					},
				)?,
			},
			canvas: Canvas::new(ctx, BASE_RESOLUTION, CanvasSettings::default()),
			show_shader_params_editor: false,
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
		self.render_main_menu(egui_ctx, ctx);
		if self.show_shader_params_editor {
			self.render_shader_params_editor(egui_ctx);
		}
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
				loaded_project: LoadedProject {
					shader,
					shader_params,
					..
				},
				playing_sound,
				time_elapsed,
				..
			}) => {
				if playing_sound.is_some() {
					*time_elapsed += delta_time;
				}
				if let Some(shader) = shader {
					show_dialog_if_error(shader.update_hot_reload(ctx, delta_time));
					shader
						.shader
						.send_f32("iTime", time_elapsed.as_secs_f32())?;
					for param in shader_params {
						match &param.kind {
							ShaderParamKind::Float { value, .. } => {
								shader.shader.send_f32(&param.name, *value)?;
							}
							ShaderParamKind::Color { value } => {
								shader.shader.send_color(&param.name, value.0)?;
							}
						}
					}
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

struct Fonts {
	small: Font,
	large: Font,
}

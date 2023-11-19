mod music_state;
mod music_theory;
mod project;
mod track_info;
mod user_track_info;
mod widgets;

use std::{path::Path, time::Duration};

use egui::TopBottomPanel;
use glam::{UVec2, Vec2};
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
	time_elapsed: Duration,
	fonts: Fonts,
	canvas: Canvas,
	// canvas_read_buffer: Vec<u8>,
	// ffmpeg_process: Child,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		Ok(Self {
			loaded_project: None,
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
		TopBottomPanel::top("main_menu").show(egui_ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				if ui.button("Load").clicked() {
					if let Some(project_path) = FileDialog::new()
						.set_directory(std::env::current_exe().unwrap())
						.add_filter("Project file", &["json"])
						.pick_file()
					{
						match LoadedProject::load(ctx, project_path) {
							Ok(loaded_project) => self.loaded_project = Some(loaded_project),
							Err(err) => {
								MessageDialog::new()
									.set_level(MessageLevel::Error)
									.set_description(format!("Error loading project: {}", err))
									.show();
							}
						}
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

	fn update(
		&mut self,
		_ctx: &mut Context,
		delta_time: std::time::Duration,
	) -> Result<(), anyhow::Error> {
		self.time_elapsed += delta_time;
		if let Some(LoadedProject {
			shader: Some(shader),
			..
		}) = &self.loaded_project
		{
			shader.send_f32("iTime", self.time_elapsed.as_secs_f32())?;
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		self.canvas.render_to(ctx, |ctx| -> anyhow::Result<()> {
			ctx.clear(OFFWHITE);
			if let Some(LoadedProject { shader, track_info }) = &self.loaded_project {
				if let Some(shader) = shader {
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
	shader: Option<Shader>,
	track_info: TrackInfo,
}

impl LoadedProject {
	fn load(ctx: &mut Context, project_path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let project_path = project_path.as_ref();
		let project = Project::from_file(project_path)?;
		let shader = project
			.shader_path
			.map(|shader_path| {
				let shader_full_path = project_path.parent().unwrap().join(shader_path);
				Shader::from_fragment_file(ctx, shader_full_path)
			})
			.transpose()?;
		if let Some(shader) = &shader {
			shader.send_vec2("iResolution", BASE_RESOLUTION.as_vec2())?;
		}
		Ok(Self {
			shader,
			track_info: TrackInfo::new(&project.track_info),
		})
	}
}

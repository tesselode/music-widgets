mod music_theory;
mod music_widgets_state;
mod track_info;
mod widgets;

use std::time::Duration;

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
use music_widgets_state::MusicWidgetsState;
use palette::LinSrgba;
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
	canvas: Canvas,
	resources: MusicWidgetResources,
	canvas_read_buffer: Vec<u8>,
	// ffmpeg_process: Child,
	time: f32,
	shader: Shader,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		let track_info = TrackInfo::from_file("tracks/test.json")?;
		Ok(Self {
			canvas: Canvas::new(ctx, BASE_RESOLUTION, CanvasSettings::default()),
			resources: MusicWidgetResources {
				music_widgets_state: MusicWidgetsState::new(track_info),
				small_font: Font::from_file(
					ctx,
					"resources/traceroute.ttf",
					FontSettings {
						scale: 64.0,
						..Default::default()
					},
				)?,
				large_font: Font::from_file(
					ctx,
					"resources/traceroute.ttf",
					FontSettings {
						scale: 128.0,
						..Default::default()
					},
				)?,
			},
			canvas_read_buffer: vec![0; (BASE_RESOLUTION.x * BASE_RESOLUTION.y * 4) as usize],
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
			time: 0.0,
			shader: Shader::from_fragment_str(ctx, include_str!("../infloresce.glsl"))?,
		})
	}
}

impl State<anyhow::Error> for MainState {
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
		self.time += delta_time.as_secs_f32();
		self.shader.send_f32("iTime", self.time)?;
		self.shader
			.send_vec2("iResolution", BASE_RESOLUTION.as_vec2())?;
		self.resources
			.music_widgets_state
			.update(Duration::from_secs_f64(1.0 / 60.0));
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		self.canvas.render_to(ctx, |ctx| -> anyhow::Result<()> {
			ctx.clear(OFFWHITE);
			Mesh::rectangle(ctx, Rect::new(Vec2::ZERO, BASE_RESOLUTION.as_vec2()))
				.draw(ctx, &self.shader);
			draw_bpm_panel(ctx, &self.resources, Vec2::new(1.0, 1.0))?;
			draw_metronome_panel(ctx, &self.resources, Vec2::new(1.0, 7.0))?;
			Ok(())
		})?;
		self.canvas.draw(
			ctx,
			DrawParams::new().scaled(ctx.window_size().as_vec2() / self.canvas.size().as_vec2()),
		);
		self.canvas.read(&mut self.canvas_read_buffer);
		/* self.ffmpeg_process
		.stdin
		.as_mut()
		.unwrap()
		.write_all(&self.canvas_read_buffer)?; */
		Ok(())
	}
}

struct MusicWidgetResources {
	music_widgets_state: MusicWidgetsState,
	small_font: Font,
	large_font: Font,
}

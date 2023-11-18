mod beat_indicator;
pub mod music_theory;
pub mod music_widgets_state;
pub mod track_info;

use std::{
	io::Write,
	process::{Child, Command, Stdio},
	time::Duration,
};

use beat_indicator::draw_beat_indicator;
use glam::{UVec2, Vec2};
use micro::{
	graphics::{
		mesh::{Mesh, ShapeStyle},
		text::{Font, FontSettings, LayoutSettings, Text},
		Canvas, CanvasSettings, ColorConstants, DrawParams, StencilAction, StencilTest,
	},
	input::Scancode,
	math::{Rect, VecConstants},
	Context, ContextSettings, Event, State, WindowMode,
};
use music_widgets_state::MusicWidgetsState;
use palette::LinSrgba;
use track_info::TrackInfo;

const BASE_RESOLUTION: UVec2 = UVec2::new(3840, 2160);
const GRID_CELL_SIZE: f32 = 48.0;
const STROKE_WIDTH: f32 = 8.0;
const OFFWHITE: LinSrgba = LinSrgba::new(0.8, 0.8, 0.8, 1.0);
const PANEL_LABEL_PADDING: f32 = 16.0;

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
	ffmpeg_process: Child,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		let track_info = TrackInfo::from_file("test.json")?;
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
			ffmpeg_process: Command::new("ffmpeg")
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
				.spawn()?,
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
		self.resources
			.music_widgets_state
			.update(Duration::from_secs_f64(1.0 / 60.0));
		self.canvas.read(&mut self.canvas_read_buffer);
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		self.canvas.render_to(ctx, |ctx| -> anyhow::Result<()> {
			ctx.clear(OFFWHITE);
			draw_bpm_panel(ctx, &self.resources, Vec2::new(1.0, 1.0))?;
			draw_metronome_panel(ctx, &self.resources, Vec2::new(1.0, 7.0))?;
			Ok(())
		})?;
		self.canvas.draw(
			ctx,
			DrawParams::new().scaled(ctx.window_size().as_vec2() / self.canvas.size().as_vec2()),
		);
		self.ffmpeg_process
			.stdin
			.as_mut()
			.unwrap()
			.write_all(&self.canvas_read_buffer)?;
		Ok(())
	}
}

fn text_translation(text: &Text, target_position: Vec2, anchor: Vec2) -> Vec2 {
	let previous_rect = text.bounds().unwrap();
	let target_rect = previous_rect.positioned(target_position, anchor);
	target_rect.top_left - previous_rect.top_left
}

struct MusicWidgetResources {
	music_widgets_state: MusicWidgetsState,
	small_font: Font,
	large_font: Font,
}

fn draw_panel(
	ctx: &mut Context,
	resources: &MusicWidgetResources,
	title: &str,
	grid_bounds: Rect,
	mut content: impl FnMut(&mut Context, Rect) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
	let polygon_grid_points = [
		grid_bounds.top_left + Vec2::RIGHT,
		grid_bounds.top_right(),
		grid_bounds.bottom_right() + Vec2::UP,
		grid_bounds.bottom_right() + Vec2::LEFT,
		grid_bounds.bottom_left(),
		grid_bounds.top_left + Vec2::DOWN,
	];
	Mesh::simple_polygon(
		ctx,
		ShapeStyle::Stroke(STROKE_WIDTH),
		polygon_grid_points
			.iter()
			.map(|point| *point * GRID_CELL_SIZE),
		LinSrgba::BLACK,
	)?
	.draw(ctx, DrawParams::new());
	let text = Text::new(ctx, &resources.small_font, title, LayoutSettings::default());
	let text_position = text_translation(
		&text,
		(grid_bounds.top_left + Vec2::RIGHT * 1.5) * GRID_CELL_SIZE,
		Vec2::new(0.0, 0.5),
	);
	Mesh::styled_rectangle(
		ctx,
		ShapeStyle::Fill,
		text.bounds()
			.unwrap()
			.translated(text_position)
			.padded(Vec2::splat(PANEL_LABEL_PADDING)),
		LinSrgba::BLACK,
	)?
	.draw(ctx, DrawParams::new());
	text.draw(
		ctx,
		DrawParams::new().translated(text_position).color(OFFWHITE),
	);
	ctx.clear_stencil();
	ctx.write_to_stencil(StencilAction::Replace(1), |ctx| -> anyhow::Result<()> {
		Mesh::simple_polygon(
			ctx,
			ShapeStyle::Fill,
			polygon_grid_points
				.iter()
				.map(|point| *point * GRID_CELL_SIZE),
			LinSrgba::BLACK,
		)?
		.draw(ctx, DrawParams::new());
		Ok(())
	})?;
	ctx.with_stencil(StencilTest::Equal, 1, |ctx| -> anyhow::Result<()> {
		content(ctx, grid_bounds)?;
		Ok(())
	})?;
	Ok(())
}

fn draw_bpm_panel(
	ctx: &mut Context,
	resources: &MusicWidgetResources,
	position: Vec2,
) -> Result<(), anyhow::Error> {
	draw_panel(
		ctx,
		resources,
		"bpm",
		Rect::new(position, Vec2::new(12.0, 4.0)),
		|ctx, grid_bounds| {
			let text = Text::new(
				ctx,
				&resources.large_font,
				&resources.music_widgets_state.bpm.to_string(),
				LayoutSettings::default(),
			);
			text.draw(
				ctx,
				DrawParams::new()
					.translated(text_translation(
						&text,
						grid_bounds.center() * GRID_CELL_SIZE,
						Vec2::splat(0.5),
					))
					.color(LinSrgba::BLACK),
			);
			Ok(())
		},
	)?;
	Ok(())
}

fn draw_metronome_panel(
	ctx: &mut Context,
	resources: &MusicWidgetResources,
	position: Vec2,
) -> Result<(), anyhow::Error> {
	draw_panel(
		ctx,
		resources,
		"metronome",
		Rect::new(position, Vec2::new(12.0, 5.0)),
		|ctx, grid_bounds| {
			let text_region = grid_bounds.resized_y(4.0, 0.0);
			let text = Text::new(
				ctx,
				&resources.large_font,
				&resources.music_widgets_state.time_signature.to_string(),
				LayoutSettings::default(),
			);
			text.draw(
				ctx,
				DrawParams::new()
					.translated(text_translation(
						&text,
						text_region.center() * GRID_CELL_SIZE,
						Vec2::splat(0.5),
					))
					.color(LinSrgba::BLACK),
			);
			draw_beat_indicator(
				ctx,
				grid_bounds.resized_y(1.0, 1.0),
				resources.music_widgets_state.time_signature,
				resources.music_widgets_state.current_beat(),
			)?;
			Ok(())
		},
	)?;
	Ok(())
}

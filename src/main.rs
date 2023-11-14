pub mod music_theory;
pub mod music_widgets_state;
pub mod track_info;

use glam::{UVec2, Vec2};
use micro::{
	graphics::{
		mesh::{Mesh, ShapeStyle},
		text::{Font, FontSettings, LayoutSettings, Text},
		ColorConstants, DrawParams,
	},
	input::Scancode,
	math::{Rect, VecConstants},
	Context, ContextSettings, Event, ScalingMode, State, WindowMode,
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
			scaling_mode: ScalingMode::Smooth {
				base_size: BASE_RESOLUTION,
			},
			..Default::default()
		},
		MainState::new,
	)
}

struct MainState {
	music_widgets_state: MusicWidgetsState,
	small_font: Font,
	large_font: Font,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		let track_info = TrackInfo::from_file("test.json")?;
		Ok(Self {
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
		})
	}

	fn draw_panel(
		&self,
		ctx: &mut Context,
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
		let text = Text::new(ctx, &self.small_font, title, LayoutSettings::default());
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
		content(ctx, grid_bounds)?;
		Ok(())
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
		self.music_widgets_state.update(delta_time);
		println!(
			"{}: {:?}",
			self.music_widgets_state.current_tick, self.music_widgets_state.chord
		);
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		ctx.clear(OFFWHITE);
		self.draw_panel(
			ctx,
			"bpm",
			Rect::from_xywh(1.0, 1.0, 12.0, 4.0),
			|ctx, grid_bounds| {
				let text = Text::new(ctx, &self.large_font, "135", LayoutSettings::default());
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
}

fn text_translation(text: &Text, target_position: Vec2, anchor: Vec2) -> Vec2 {
	let previous_rect = text.bounds().unwrap();
	let target_rect = previous_rect.positioned(target_position, anchor);
	target_rect.top_left - previous_rect.top_left
}

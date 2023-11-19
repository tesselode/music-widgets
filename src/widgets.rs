mod beat_indicator;

use std::time::Duration;

use glam::Vec2;
use micro::{
	graphics::{
		mesh::{Mesh, ShapeStyle},
		text::{LayoutSettings, Text},
		ColorConstants, DrawParams, StencilAction, StencilTest,
	},
	math::{Rect, VecConstants},
	Context,
};
use palette::LinSrgba;

use crate::{track_info::TrackInfo, Fonts, OFFWHITE};

use self::beat_indicator::draw_beat_indicator;

const GRID_CELL_SIZE: f32 = 48.0;
const STROKE_WIDTH: f32 = 8.0;
const PANEL_LABEL_PADDING: f32 = 16.0;

pub(super) fn draw_panel(
	ctx: &mut Context,
	fonts: &Fonts,
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
	let text = Text::new(ctx, &fonts.small, title, LayoutSettings::default());
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

pub(super) fn draw_bpm_panel(
	ctx: &mut Context,
	track_info: &TrackInfo,
	timestamp: Duration,
	fonts: &Fonts,
	position: Vec2,
) -> Result<(), anyhow::Error> {
	draw_panel(
		ctx,
		fonts,
		"bpm",
		Rect::new(position, Vec2::new(12.0, 4.0)),
		|ctx, grid_bounds| {
			let text = Text::new(
				ctx,
				&fonts.large,
				&track_info
					.music_state(timestamp)
					.music_state
					.bpm
					.to_string(),
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

pub(super) fn draw_metronome_panel(
	ctx: &mut Context,
	track_info: &TrackInfo,
	timestamp: Duration,
	fonts: &Fonts,
	position: Vec2,
) -> Result<(), anyhow::Error> {
	draw_panel(
		ctx,
		fonts,
		"metronome",
		Rect::new(position, Vec2::new(12.0, 5.0)),
		|ctx, grid_bounds| {
			let text_region = grid_bounds.resized_y(4.0, 0.0);
			let text = Text::new(
				ctx,
				&fonts.large,
				&track_info
					.music_state(timestamp)
					.music_state
					.time_signature
					.to_string(),
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
				track_info.music_state(timestamp).music_state.time_signature,
				track_info.current_beat(timestamp) as u32,
			)?;
			Ok(())
		},
	)?;
	Ok(())
}

fn text_translation(text: &Text, target_position: Vec2, anchor: Vec2) -> Vec2 {
	let previous_rect = text.bounds().unwrap();
	let target_rect = previous_rect.positioned(target_position, anchor);
	target_rect.top_left - previous_rect.top_left
}

use glam::Vec2;
use micro::{
	graphics::{
		mesh::{MeshBuilder, ShapeStyle},
		ColorConstants, DrawParams,
	},
	math::Rect,
	Context,
};
use palette::LinSrgba;

use crate::music_theory::TimeSignature;

use super::{GRID_CELL_SIZE, STROKE_WIDTH};

pub fn draw_beat_indicator(
	ctx: &mut Context,
	rect: Rect,
	time_signature: TimeSignature,
	current_beat: u32,
) -> anyhow::Result<()> {
	let mut mesh_builder = MeshBuilder::new();
	let num_ticks = time_signature.top;
	mesh_builder.add_rectangle(
		ShapeStyle::Stroke(STROKE_WIDTH),
		Rect::new(rect.top_left * GRID_CELL_SIZE, rect.size * GRID_CELL_SIZE),
		LinSrgba::BLACK,
	)?;
	for i in 1..num_ticks {
		mesh_builder.add_simple_polyline(
			STROKE_WIDTH,
			beat_indicator_tick_points(rect, i, num_ticks),
			LinSrgba::BLACK,
		)?;
	}
	let current_tick = beat_indicator_tick_points(rect, current_beat, num_ticks);
	let next_tick = beat_indicator_tick_points(rect, current_beat + 1, num_ticks);
	mesh_builder.add_simple_polygon(
		ShapeStyle::Fill,
		[current_tick[0], next_tick[0], next_tick[1], current_tick[1]],
		LinSrgba::BLACK,
	)?;
	mesh_builder.build(ctx).draw(ctx, DrawParams::new());
	Ok(())
}

fn beat_indicator_tick_points(rect: Rect, tick_index: u32, num_ticks: u32) -> [Vec2; 2] {
	let should_shear = tick_index != 0 && tick_index != num_ticks;
	let x = tick_index as f32 / num_ticks as f32;
	let mut points = [
		Vec2::new(rect.fractional_x(x), rect.top()),
		Vec2::new(rect.fractional_x(x), rect.bottom()),
	];
	if should_shear {
		points[0].x += rect.size.y / 2.0;
		points[1].x -= rect.size.y / 2.0;
	}
	points[0] *= GRID_CELL_SIZE;
	points[1] *= GRID_CELL_SIZE;
	points
}

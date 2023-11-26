mod beat_indicator;

use std::time::Duration;

use glam::Vec2;
use micro::{
	graphics::{
		mesh::{Mesh, ShapeStyle},
		text::{LayoutSettings, Text, TextFragment},
		ColorConstants, DrawParams, StencilAction, StencilTest,
	},
	math::{Rect, VecConstants},
	Context,
};
use palette::LinSrgba;
use regex::Regex;

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

pub(super) fn draw_key_panel(
	ctx: &mut Context,
	track_info: &TrackInfo,
	timestamp: Duration,
	fonts: &Fonts,
	position: Vec2,
) -> Result<(), anyhow::Error> {
	draw_panel(
		ctx,
		fonts,
		"key",
		Rect::new(position, Vec2::new(12.0, 4.0)),
		|ctx, grid_bounds| {
			let text = chord_text(
				ctx,
				&track_info.music_state(timestamp).music_state.key,
				fonts,
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

pub(super) fn draw_chord_panel(
	ctx: &mut Context,
	track_info: &TrackInfo,
	timestamp: Duration,
	fonts: &Fonts,
	position: Vec2,
) -> Result<(), anyhow::Error> {
	draw_panel(
		ctx,
		fonts,
		"chord",
		Rect::new(position, Vec2::new(12.0, 4.0)),
		|ctx, grid_bounds| {
			let text = chord_text(
				ctx,
				&track_info.music_state(timestamp).music_state.chord,
				fonts,
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

fn text_translation(text: &Text, target_position: Vec2, anchor: Vec2) -> Vec2 {
	let previous_rect = text.bounds().unwrap();
	let target_rect = previous_rect.positioned(target_position, anchor);
	target_rect.top_left - previous_rect.top_left
}

fn chord_text(ctx: &mut Context, chord: &str, fonts: &Fonts) -> Text {
	let fonts = &[
		&fonts.large,
		&fonts.medium,
		&fonts.music_large,
		&fonts.music_medium,
	];
	let mut chord_string_fragments = split_chord_str(chord);
	for (_, s) in &mut chord_string_fragments {
		*s = s.replace('b', "♭").replace('#', "♯");
	}
	let fragments = chord_string_fragments
		.iter()
		.map(|(font, text)| TextFragment {
			font_index: match font {
				ChordTextFont::Big => 0,
				ChordTextFont::Small => 1,
				ChordTextFont::MusicBig => 2,
				ChordTextFont::MusicSmall => 3,
			},
			text,
		})
		.collect::<Vec<_>>();
	Text::with_multiple_fonts(ctx, fonts, fragments.iter(), LayoutSettings::default())
}

fn split_chord_str(chord: &str) -> Vec<(ChordTextFont, String)> {
	let regex = Regex::new("([ABCDEFG][b#]?)(.*)").unwrap();
	let captures = regex.captures(chord).expect("invalid chord");
	let big_text = &captures[1];
	let small_text = &captures[2];
	let mut fragments = vec![];
	for (is_accidental, string_fragment) in split_chord_str_by_accidentals(big_text) {
		let font = if is_accidental {
			ChordTextFont::MusicBig
		} else {
			ChordTextFont::Big
		};
		fragments.push((font, string_fragment));
	}
	for (is_accidental, string_fragment) in split_chord_str_by_accidentals(small_text) {
		let font = if is_accidental {
			ChordTextFont::MusicSmall
		} else {
			ChordTextFont::Small
		};
		fragments.push((font, string_fragment));
	}
	fragments
}

fn split_chord_str_by_accidentals(s: &str) -> Vec<(bool, String)> {
	let accidental_indices =
		(0..s.len()).filter(|i| &s[*i..*i + 1] == "b" || &s[*i..*i + 1] == "#");
	let mut next_substr_start = 0;
	let mut fragments = vec![];
	for accidental_index in accidental_indices {
		fragments.push((false, s[next_substr_start..accidental_index].to_string()));
		fragments.push((true, s[accidental_index..accidental_index + 1].to_string()));
		next_substr_start = accidental_index + 1;
	}
	if next_substr_start < s.len() {
		fragments.push((false, s[next_substr_start..].to_string()));
	}
	fragments
}

enum ChordTextFont {
	Big,
	Small,
	MusicBig,
	MusicSmall,
}

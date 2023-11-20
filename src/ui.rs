use std::{path::PathBuf, time::Duration};

use egui::{ProgressBar, Slider, Ui};
use rfd::FileDialog;

use crate::{
	format::{format_time, parse_time},
	LiveState, LoadedProject, MainState, RenderingState, EXPORT_FPS,
};

impl MainState {
	pub(super) fn render_idle_mode_menu(ui: &mut Ui) -> Option<IdleModeMenuAction> {
		let mut action = None;
		if ui.button("Load").clicked() {
			if let Some(project_path) = FileDialog::new()
				.set_directory(std::env::current_exe().unwrap())
				.add_filter("Project file", &["json"])
				.pick_file()
			{
				action = Some(IdleModeMenuAction::LoadProject { path: project_path });
			}
		}
		action
	}

	pub(super) fn render_live_mode_menu(
		ui: &mut Ui,
		LiveState {
			loaded_project: LoadedProject { sound_data, .. },
			playing_sound,
			time_elapsed,
			..
		}: &LiveState,
	) -> Option<LiveModeMenuAction> {
		let mut action = None;
		if ui.button("Load").clicked() {
			if let Some(project_path) = FileDialog::new()
				.set_directory(std::env::current_exe().unwrap())
				.add_filter("Project file", &["json"])
				.pick_file()
			{
				action = Some(LiveModeMenuAction::LoadProject { path: project_path });
			}
		}
		if ui.button("Render").clicked() {
			if let Some(output_path) = FileDialog::new()
				.set_directory(std::env::current_exe().unwrap())
				.add_filter("mp4 video", &["mp4"])
				.save_file()
			{
				action = Some(LiveModeMenuAction::StartRendering { output_path });
			}
		}
		let mut playing = playing_sound.is_some();
		if ui.checkbox(&mut playing, "Playing").changed() {
			action = Some(LiveModeMenuAction::SetPlaying(playing));
		}
		let mut time_elapsed_f64 = time_elapsed.as_secs_f64();
		let position_slider = Slider::new(
			&mut time_elapsed_f64,
			0.0..=sound_data.duration().as_secs_f64(),
		)
		.trailing_fill(true)
		.custom_formatter(format_time)
		.custom_parser(parse_time);
		ui.style_mut().spacing.slider_width = 200.0;
		let position_slider_response = &ui.add(position_slider);
		if position_slider_response.changed() {
			action = Some(LiveModeMenuAction::Seek {
				time: Duration::from_secs_f64(time_elapsed_f64),
				seek_audio: position_slider_response.drag_released(),
			});
		}
		action
	}

	pub(super) fn render_rendering_mode_menu(
		ui: &mut Ui,
		RenderingState {
			loaded_project: LoadedProject { sound_data, .. },
			current_frame,
			..
		}: &RenderingState,
	) -> Option<RenderingModeMenuAction> {
		let mut action = None;
		let total_frames = (sound_data.duration().as_secs_f64() * EXPORT_FPS) as u32;
		ui.add(ProgressBar::new(*current_frame as f32 / total_frames as f32).desired_width(500.0));
		ui.label(format!("Rendering ({}/{})", *current_frame, total_frames));
		if ui.button("Cancel").clicked() {
			action = Some(RenderingModeMenuAction::CancelRendering);
		}
		action
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum IdleModeMenuAction {
	LoadProject { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum LiveModeMenuAction {
	LoadProject { path: PathBuf },
	StartRendering { output_path: PathBuf },
	SetPlaying(bool),
	Seek { time: Duration, seek_audio: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum RenderingModeMenuAction {
	CancelRendering,
}

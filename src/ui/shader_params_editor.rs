use anyhow::Context;
use egui::{DragValue, Label, Slider, Window};
use palette::LinSrgba;

use crate::{
	live_state::LiveState, loaded_project::LoadedProject, rendering_state::RenderingState,
	shader_param::ShaderParamKind, MainState, Mode,
};

use super::show_dialog_if_error;

impl MainState {
	pub fn render_shader_params_editor(&mut self, egui_ctx: &egui::Context) {
		Window::new("Shader Params")
			.open(&mut self.show_shader_params_editor)
			.enabled(!matches!(&self.mode, Mode::Rendering(..)))
			.show(egui_ctx, |ui| {
				let (Mode::Live(LiveState { loaded_project, .. })
				| Mode::Rendering(RenderingState { loaded_project, .. })) = &mut self.mode
				else {
					ui.label("No project loaded");
					return;
				};
				let LoadedProject {
					project,
					project_path,
					shader_params,
					..
				} = loaded_project;
				for param in &mut *shader_params {
					ui.horizontal(|ui| {
						ui.add_sized((100.0, ui.available_height()), Label::new(&param.name));
						match &mut param.kind {
							ShaderParamKind::Float { value, min, max } => {
								if let (Some(min), Some(max)) = (min.as_mut(), max.as_mut()) {
									ui.add(Slider::new(value, *min..=*max));
								} else {
									ui.add(DragValue::new(value));
									if let Some(min) = min {
										*value = value.max(*min);
									}
									if let Some(max) = max {
										*value = value.min(*max);
									}
								}
							}
							ShaderParamKind::Color { value } => {
								let mut components =
									[value.0.red, value.0.green, value.0.blue, value.0.alpha];
								ui.color_edit_button_rgba_unmultiplied(&mut components);
								value.0 = LinSrgba::new(
									components[0],
									components[1],
									components[2],
									components[3],
								);
							}
						}
					});
				}
				ui.horizontal(|ui| {
					if ui.button("Save").clicked() {
						project.shader_params = shader_params.clone();
						show_dialog_if_error(
							project.save(project_path).context("error saving project"),
						);
					}
					if ui.button("Discard changes").clicked() {
						*shader_params = project.shader_params.clone();
					}
				});
			});
	}
}

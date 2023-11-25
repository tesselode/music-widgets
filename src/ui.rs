pub mod main_menu;
pub mod shader_params_editor;

use std::fmt::{Debug, Display};

use rfd::{MessageDialog, MessageLevel};

pub fn show_dialog_if_error<T, E: Debug + Display>(result: Result<T, E>) {
	if let Err(err) = result {
		MessageDialog::new()
			.set_title("Error")
			.set_level(MessageLevel::Error)
			.set_description(format!("{:?}", err))
			.show();
	}
}

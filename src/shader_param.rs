use anyhow::bail;
use palette::{rgb::channels::Rgba, LinSrgba, Srgba};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShaderParam {
	pub name: String,
	#[serde(flatten)]
	pub kind: ShaderParamKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ShaderParamKind {
	Float {
		value: f32,
		min: Option<f32>,
		max: Option<f32>,
	},
	Color {
		value: ShaderColor,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "&str")]
pub struct ShaderColor(pub LinSrgba);

impl From<ShaderColor> for String {
	fn from(value: ShaderColor) -> Self {
		format!("#{:X}", Srgba::from(value.0).into_u32::<Rgba>())
	}
}

impl TryFrom<&str> for ShaderColor {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> anyhow::Result<Self> {
		let mut hex_string = value[1..].to_string();
		match hex_string.len() {
			6 => hex_string += "ff",
			8 => {}
			_ => bail!("invalid hex color"),
		}
		Ok(Self(
			Srgba::from_u32::<Rgba>(u32::from_str_radix(&hex_string, 16)?).into_linear(),
		))
	}
}

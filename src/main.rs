use glam::UVec2;
use micro::{input::Scancode, Context, ContextSettings, Event, ScalingMode, State, WindowMode};
use palette::LinSrgba;

const BASE_RESOLUTION: UVec2 = UVec2::new(3840, 2160);

fn main() {
	micro::run(
		ContextSettings {
			window_mode: WindowMode::Windowed {
				size: UVec2::new(1920, 1080),
			},
			scaling_mode: ScalingMode::Smooth {
				base_size: BASE_RESOLUTION,
			},
			..Default::default()
		},
		MainState::new,
	)
}

struct MainState {}

impl MainState {
	pub fn new(ctx: &mut Context) -> anyhow::Result<Self> {
		Ok(Self {})
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

	fn draw(&mut self, ctx: &mut Context) -> Result<(), anyhow::Error> {
		ctx.clear(LinSrgba::new(0.8, 0.8, 0.8, 1.0));
		Ok(())
	}
}

use std::ops::RangeInclusive;

pub fn format_time(seconds: f64, _decimals: RangeInclusive<usize>) -> String {
	let seconds = seconds as i32;
	let hours = seconds / (60 * 60);
	let mins = (seconds / 60) % 60;
	let secs = seconds % 60;
	format!("{hours:02}:{mins:02}:{secs:02}")
}

pub fn parse_time(time: &str) -> Option<f64> {
	let parts: Vec<&str> = time.split(':').collect();
	if parts.len() == 3 {
		parts[0]
			.parse::<i32>()
			.and_then(|h| {
				parts[1].parse::<i32>().and_then(|m| {
					parts[2]
						.parse::<i32>()
						.map(|s| ((h * 60 * 60) + (m * 60) + s) as f64)
				})
			})
			.ok()
	} else {
		None
	}
}

use std::collections::HashMap;

use openaction::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};

struct BatteryAction;
#[async_trait]
impl Action for BatteryAction {
	const UUID: ActionUuid = "dev.lyzev.oag.battery";
	type Settings = HashMap<String, String>;
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> String {
	let c = v * s;
	let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
	let m = v - c;
	let (r, g, b) = if h < 60.0 {
		(c, x, 0.0)
	} else if h <= 120.0 {
		(x, c, 0.0)
	} else {
		(0.0, 0.0, 0.0)
	};
	let r_u8 = ((r + m) * 255.0).round() as u8;
	let g_u8 = ((g + m) * 255.0).round() as u8;
	let b_u8 = ((b + m) * 255.0).round() as u8;
	format!("#{:02X}{:02X}{:02X}", r_u8, g_u8, b_u8)
}

fn generate_battery_svg(percentage: Option<u8>) -> String {
	let (pct, text, color) = match percentage {
		Some(p) => {
			let clamped = p.min(100) as f32;
			let hue = (clamped / 100.0) * 120.0;
			(clamped, format!("{}%", p), hsv_to_rgb(hue, 1.0, 0.9))
		}
		None => (0.0, "N/A".to_string(), "#555555".to_string()),
	};
	let bar_width = (pct / 100.0) * 96.0;
	let svg = format!(
		"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 144 144\" width=\"144\" height=\"144\">\
            <rect x=\"15\" y=\"45\" width=\"108\" height=\"54\" rx=\"8\" ry=\"8\" fill=\"none\" stroke=\"#FFFFFF\" stroke-width=\"6\"/>\
            <path d=\"M 123 60 C 129 60 129 84 123 84\" fill=\"#FFFFFF\"/>\
            <rect x=\"21\" y=\"51\" width=\"{}\" height=\"42\" rx=\"4\" ry=\"4\" fill=\"{}\"/>\
            <text x=\"69\" y=\"72\" font-family=\"sans-serif\" font-size=\"24\" font-weight=\"bold\" fill=\"#FFFFFF\" stroke=\"#000000\" stroke-width=\"2\" stroke-linejoin=\"round\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\
        </svg>",
		bar_width, color, text
	);
	format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg))
}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	{
		use simplelog::*;
		if let Err(error) = TermLogger::init(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {}", error);
		}
	}
	tokio::spawn(async {
		loop {
			let mut current_capacity: Option<u8> = None;
			if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
				for entry in entries.flatten() {
					let name = entry.file_name().to_string_lossy().into_owned();
					if name.contains("hidpp_battery") || name.contains("mouse") {
						let cap_path = entry.path().join("capacity");
						if let Ok(cap) = std::fs::read_to_string(cap_path) {
							if let Ok(cap_num) = cap.trim().parse::<u8>() {
								current_capacity = Some(cap_num);
							}
							break;
						}
					}
				}
			}
			let svg_data = generate_battery_svg(current_capacity);
			for instance in visible_instances(BatteryAction::UUID).await {
				let _ = instance.set_image(Some(svg_data.clone()), None).await;
			}
		}
	});
	register_action(BatteryAction).await;
	run(std::env::args().collect()).await
}

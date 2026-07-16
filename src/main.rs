use std::collections::HashMap;

use openaction::*;

struct BatteryAction;
#[async_trait]
impl Action for BatteryAction {
	const UUID: ActionUuid = "dev.lyzev.oag.battery";
	type Settings = HashMap<String, String>;
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
		let mut mouse_battery = "N/A".to_string();
		loop {
			if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
				for entry in entries.flatten() {
					let name = entry.file_name().to_string_lossy().into_owned();
					if name.contains("hidpp_battery") || name.contains("mouse") {
						let cap_path = entry.path().join("capacity");
						if let Ok(cap) = std::fs::read_to_string(cap_path) {
							mouse_battery = format!("{}%", cap.trim());
							break;
						}
					}
				}
			}
			for instance in visible_instances(BatteryAction::UUID).await {
				let _ = instance.set_title(Some(mouse_battery.clone()), Some(1)).await;
			}
		}
	});

	register_action(BatteryAction).await;

	run(std::env::args().collect()).await
}

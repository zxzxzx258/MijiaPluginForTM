use cosmic_applet_monitor::{config, miio};

#[tokio::main]
async fn main() {
    let (config, error) = config::load();
    if let Some(error) = error {
        eprintln!("configuration error: {error}");
        std::process::exit(2);
    }
    if config.mijia.is_empty() {
        eprintln!("MiJia is not configured");
        std::process::exit(2);
    }
    let mut failed = false;
    for device in config.mijia {
        match miio::read_power(
            device.ip,
            &device.token,
            &device.model,
            device.power_siid,
            device.power_piid,
        )
        .await
        {
            Ok(reading) => println!("{}: {:.1} W", device.name, reading.watts),
            Err(error) => {
                eprintln!("{}: {error}", device.name);
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}

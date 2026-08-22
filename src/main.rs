const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();
    tracing::info!(%VERSION, "starting COSMIC monitor applet");
    cosmic_applet_monitor::run()
}

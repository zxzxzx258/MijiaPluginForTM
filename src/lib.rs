mod app;
pub mod config;
pub mod metrics;
pub mod miio;

pub fn run() -> cosmic::iced::Result {
    app::run()
}

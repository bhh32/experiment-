//! Rust Writer — A modern professional-grade word processor
//!
//! Built with pure Rust and libcosmic (COSMIC desktop toolkit).

mod app;
mod config;
mod messages;

fn main() -> cosmic::iced::Result {
    env_logger::init();
    log::info!("Starting Rust Writer");

    let settings = cosmic::app::Settings::default();
    cosmic::app::run::<app::RustWriter>(settings, ())
}

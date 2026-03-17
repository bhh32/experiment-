mod config;
mod localize;
mod logic;
mod notifications;
mod tailscale_api;
mod window;

use crate::window::Window;

fn main() -> cosmic::iced::Result {
    localize::localize();
    cosmic::applet::run::<Window>(())?;

    Ok(())
}

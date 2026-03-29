pub mod api;
pub mod app;
pub mod components;
pub mod pages;

use leptos::prelude::*;

pub fn main() {
    mount_to_body(app::App);
}

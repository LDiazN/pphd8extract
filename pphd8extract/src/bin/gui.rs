#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

// Local imports
extern crate pphd8extract;
mod gui_app;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 640.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "PPHD8 Extract",
        options,
        Box::new(|_cc| {
            // This gives us image support:

            Box::<gui_app::App>::default()
        }),
    )
}

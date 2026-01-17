#![windows_subsystem = "windows"]
//! VoxMagic - Creative AI Speech-to-Text

mod app;
mod audio;
mod config;
mod transcriber;
mod settings;

use app::VoxMagicApp;
use eframe::egui;
use settings::AppSettings;

fn main() -> eframe::Result<()> {
    let settings = AppSettings::load();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([500.0, 600.0])
        .with_min_inner_size([400.0, 450.0])
        .with_title("🎙️ VoxMagic")
        .with_icon(load_icon());

    if settings.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    eframe::run_native(
        "VoxMagic",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(VoxMagicApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Embed the icon so it's always available in the standalone EXE
    let icon_data = include_bytes!("../VoxMagicLogo.png");
    
    if let Ok(image) = image::load_from_memory(icon_data) {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        return egui::IconData {
            rgba: image.into_raw(),
            width,
            height,
        };
    }

    // Fallback to purple square if logo invalid
    let mut rgba = vec![0; 16 * 16 * 4];
    for i in 0..16*16 {
        rgba[i*4] = 139;    // R (Purple accent)
        rgba[i*4+1] = 92;   // G
        rgba[i*4+2] = 246;  // B
        rgba[i*4+3] = 255;  // A
    }
    egui::IconData { rgba, width: 16, height: 16 }
}

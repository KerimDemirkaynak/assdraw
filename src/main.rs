#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code, unused_imports)]

mod app;
mod canvas;
mod engine;
mod export;
mod i18n;
mod library;
mod settings;
mod ui;

use app::AssDrawApp;

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsunb.ttf",
        "C:\\Windows\\Fonts\\msyhl.ttc",
        "C:\\Windows\\Fonts\\segoeui.ttf",
    ];

    for path in &candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "system_fallback".to_owned(),
                egui::FontData::from_owned(bytes),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("system_fallback".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("system_fallback".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn load_app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/tsukasa.ico");
    if let Ok(img) = image::load_from_memory(bytes) {
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([900.0, 560.0])
        .with_title("ASSDraw3");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let opts = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "ASSDraw3",
        opts,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(AssDrawApp::new(cc)))
        }),
    )
}

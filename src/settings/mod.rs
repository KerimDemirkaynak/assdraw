//! Persistent settings for ASSDraw-rs (mirrors settings.hpp/cpp).
//! Full appearance, colors, alphas, behaviors, and language support.

use crate::engine::{Color, ShapeStyle};
use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // Language & Theme
    pub language: Language,
    pub theme_dark: bool,
    /// Follow the operating system's light/dark setting (overrides `theme_dark`).
    #[serde(default)]
    pub theme_system: bool,
    /// Do not show the welcome screen at startup.
    #[serde(default)]
    pub no_splash: bool,

    // Appearance - Colors
    pub canvas_bg: Color,
    pub shape_fill: Color,
    pub shape_outline: Color,
    pub shape_preview: Color,
    pub guideline_color: Color,
    pub point_main: Color,
    pub point_control: Color,
    pub point_selected: Color,
    pub origin_color: Color,
    pub origin_cross_size: u32,
    pub ruler_h_color: Color,
    pub ruler_v_color: Color,
    pub library_bg: Color,
    pub library_shape_color: Color,
    /// Thin editor outline drawn along the shape (the original's "Outline" setting).
    #[serde(default = "default_black")]
    pub editor_outline: Color,
    #[serde(default = "default_255")]
    pub editor_outline_opacity: u8,

    // Appearance - Visibility & Dimensions
    pub show_grid: bool,
    pub show_points: bool,
    pub show_fill: bool,
    pub show_rulers: bool,
    pub fill_opacity: u8,
    pub border_width: f64,

    // Appearance - Opacities (matching ASSDraw3 settings dialog)
    #[serde(default = "default_255")]
    pub preview_opacity: u8,
    #[serde(default = "default_255")]
    pub outline_opacity: u8,
    #[serde(default = "default_255")]
    pub guideline_opacity: u8,
    #[serde(default = "default_128")]
    pub point_opacity: u8,
    #[serde(default = "default_128")]
    pub control_point_opacity: u8,
    #[serde(default = "default_255")]
    pub selected_point_opacity: u8,

    // Behaviors
    pub auto_bezier: bool,
    pub bezier_smoothness: f64,
    pub capitalize_cmds: bool,
    pub auto_ask_img_opacity: bool,
    pub bg_alpha_default: u8,
    pub parse_spc: bool,
    pub confirm_quit: bool,
    pub use_multicolor: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::detect(),
            theme_dark: true,
            theme_system: false,
            no_splash: false,

            canvas_bg: Color::from_rgb(25, 25, 25),
            shape_fill: Color::from_rgb(0, 0, 255),
            shape_outline: Color::BLACK,
            shape_preview: Color::from_rgb(0, 0, 255),
            guideline_color: Color::from_rgb(150, 154, 160),
            point_main: Color::from_rgb(255, 112, 112),
            point_control: Color::from_rgb(94, 220, 138),
            point_selected: Color::from_rgb(104, 190, 255),
            origin_color: Color::from_rgb(255, 112, 112),
            origin_cross_size: 2,
            ruler_h_color: Color::from_rgb(181, 205, 240),
            ruler_v_color: Color::from_rgb(240, 178, 190),
            library_bg: Color::from_rgb(51, 51, 51),
            library_shape_color: Color::from_rgb(100, 181, 246),
            editor_outline: Color::from_rgb(200, 200, 200),
            editor_outline_opacity: 255,

            show_grid: false,
            show_points: true,
            show_fill: true,
            show_rulers: true,
            fill_opacity: 255,
            border_width: 0.0,

            preview_opacity: 255,
            outline_opacity: 255,
            guideline_opacity: 255,
            point_opacity: 128,
            control_point_opacity: 128,
            selected_point_opacity: 255,

            auto_bezier: true,
            bezier_smoothness: 0.35,
            capitalize_cmds: false,
            auto_ask_img_opacity: true,
            bg_alpha_default: 128,
            parse_spc: true,
            confirm_quit: false,
            use_multicolor: true,
        }
    }
}

const fn default_black() -> Color {
    Color::BLACK
}

const fn default_255() -> u8 {
    255
}

const fn default_128() -> u8 {
    128
}

impl AppSettings {
    /// The fill/outline style that is written into (and read from) ASS override tags.
    pub fn shape_style(&self) -> ShapeStyle {
        ShapeStyle {
            fill_enabled: self.show_fill,
            fill: self.shape_fill,
            fill_opacity: self.fill_opacity,
            outline_enabled: self.border_width > 0.0 && self.outline_opacity > 0,
            outline_width: self.border_width,
            outline: self.shape_outline,
            outline_opacity: self.outline_opacity,
        }
    }

    pub fn apply_shape_style(&mut self, style: &ShapeStyle) {
        self.show_fill = style.fill_enabled;
        self.shape_fill = style.fill;
        self.fill_opacity = style.fill_opacity;
        self.shape_outline = style.outline;
        self.outline_opacity = style.outline_opacity;
        self.border_width = if style.outline_enabled { style.outline_width } else { 0.0 };
    }

    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("assdraw-rs").join("settings.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(s) = serde_json::from_str(&data) {
                    return s;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }
}

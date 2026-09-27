use eframe::egui::{self, Color32};

pub const BG: Color32 = Color32::from_rgb(248, 248, 255);
pub const PANEL_BG: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT: Color32 = Color32::from_rgb(30, 30, 30);
pub const TEXT_DIM: Color32 = Color32::from_rgb(120, 120, 120);
pub const BORDER_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
pub const ACCENT: Color32 = Color32::from_rgb(200, 30, 30);

pub const OK: Color32 = Color32::from_rgb(50, 140, 70);
pub const ERR: Color32 = Color32::from_rgb(140, 56, 50);
pub const WARN: Color32 = Color32::from_rgb(166, 136, 60);

pub const IDLE: Color32 = Color32::from_rgb(160, 160, 160);
pub const WORKING: Color32 = Color32::from_rgb(60, 120, 200);
pub const REVIEW: Color32 = Color32::from_rgb(220, 170, 60);
pub const DONE: Color32 = Color32::from_rgb(80, 160, 80);
pub const BANNED: Color32 = Color32::from_rgb(200, 60, 60);
pub const OFFLINE: Color32 = Color32::from_rgb(100, 100, 100);

pub const BORDER_RADIUS: egui::CornerRadius = egui::CornerRadius::same(4);

// Buttons
pub const BUTTON_INACTIVE_BG: Color32 = Color32::from_rgb(220, 220, 220);
pub const BUTTON_HOVER_BG: Color32 = Color32::from_rgb(200, 200, 200);


pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = PANEL_BG;
    visuals.faint_bg_color = BORDER_COLOR;
    visuals.override_text_color = Some(TEXT);
    visuals.window_corner_radius = BORDER_RADIUS;
    visuals.menu_corner_radius = BORDER_RADIUS;

    // buttons
    visuals.widgets.inactive.weak_bg_fill = BUTTON_INACTIVE_BG;
    visuals.widgets.inactive.bg_stroke.color = BORDER_COLOR;
    visuals.widgets.hovered.weak_bg_fill = BUTTON_HOVER_BG;
    visuals.widgets.active.weak_bg_fill = BUTTON_INACTIVE_BG;

    ctx.set_visuals(visuals);
}

pub fn get_window_bg_color() -> [f32; 4] {
    [BG.r() as f32, BG.g() as f32, BG.b() as f32, BG.a() as f32]
}
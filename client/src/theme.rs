use eframe::egui::{self, Color32};

pub const BG: Color32 = Color32::from_rgb(255, 255, 255);
pub const PANEL_BG: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT: Color32 = Color32::from_rgb(23, 32, 51);
pub const TEXT_ON_ACCENT: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_DIM: Color32 = Color32::from_rgb(139, 149, 167);
pub const BORDER_COLOR: Color32 = Color32::from_rgb(201, 208, 219);
pub const ACCENT: Color32 = Color32::from_rgb(234, 65, 65);
pub const CARD_BG: Color32 = Color32::from_rgb(248,250, 252);

pub const ERR: Color32 = Color32::from_rgb(230, 65, 65);

pub const IDLE: Color32 = Color32::from_rgb(241, 241, 241);
pub const IDLE_DOT: Color32 = Color32::from_rgb(101, 101, 101);
pub const WORKING: Color32 = Color32::from_rgb(234, 247, 240);
pub const WORKING_FG: Color32 = Color32::from_rgb(27, 122, 74);
pub const WORKING_DOT: Color32 = Color32::from_rgb(36, 166, 101);
pub const REVIEW: Color32 = Color32::from_rgb(247, 245, 234);
pub const REVIEW_FG: Color32 = Color32::from_rgb(122, 106, 27);
pub const REVIEW_DOT: Color32 = Color32::from_rgb(166, 144, 36);
pub const DONE: Color32 = Color32::from_rgb(234, 241, 247);
pub const DONE_FG: Color32 = Color32::from_rgb(27, 75, 122);
pub const DONE_DOT: Color32 = Color32::from_rgb(36, 101, 166);
pub const BANNED: Color32 = Color32::from_rgb(247, 234, 234);
pub const BANNED_FG: Color32 = Color32::from_rgb(122, 27, 27);
pub const BANNED_DOT: Color32 = Color32::from_rgb(166, 36, 36);


pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = PANEL_BG;
    visuals.faint_bg_color = BORDER_COLOR;
    visuals.override_text_color = Some(TEXT);

    // buttons
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);

    ctx.set_visuals(visuals);
}

pub fn get_window_bg_color() -> [f32; 4] {
    [BG.r() as f32, BG.g() as f32, BG.b() as f32, BG.a() as f32]
}
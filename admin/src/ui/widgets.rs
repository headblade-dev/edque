use eframe::egui::{Color32, Frame, Margin, Sense, Ui, Vec2};
use shared::{theme, widgets};

///
/// Draws a status badge (pill-style)
///
pub fn add_status(ui: &mut Ui, bg: Color32, fg: Color32, dot_color: Color32, text: &str) {
    Frame::new().show(ui, |ui| {
        ui.set_width(124.0);

        Frame::new()
            .corner_radius(14)
            .inner_margin(Margin::symmetric(10, 6))
            .fill(bg)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let (dot, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());

                    ui.painter().rect_filled(dot, 4.0, dot_color);
                    ui.add_space(6.0);

                    widgets::rich_label(ui, text, 13, fg, 500)
                })
            });
    });
}

///
/// Draws a separator. Use this because it has a 1-pixel width.
///
pub fn separator(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::THIN_LINE);
}

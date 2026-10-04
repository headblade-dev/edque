use crate::{ClientApp, ClientField, ClientInfo};
use eframe::egui::{
    Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Layout, Margin, Response,
    Sense, Stroke, Ui, Vec2,
};
use shared::theme;
use std::time::{SystemTime, UNIX_EPOCH};

impl ClientApp {
    /// Draws a `TextEdit` with `Label` from left side
    pub fn labeled_input(&mut self, ui: &mut Ui, label: &str, hint: &str, field: ClientField) {
        let (buf, focus_buf) = match field {
            ClientField::CompID => (&mut self.comp_id_input, &mut self.comp_id_input_focused),
            ClientField::Server => (&mut self.server_input, &mut self.server_input_focused),
        };

        Frame::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(64.0);

            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                shared::widgets::rich_label(ui, label, 15, theme::TEXT, 600);

                let (offset, stroke) = if *focus_buf {
                    (1, Stroke::new(2.0, theme::ACCENT))
                } else {
                    (0, Stroke::new(1.0, theme::BORDER_COLOR))
                };

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let input = &mut shared::widgets::Input {
                        dims: (308.0, 16.0),
                        margin: Margin::same(16 - offset as i8),
                        stroke,
                        buf: buf.to_string(),
                        hint: hint.to_string(),
                        ..Default::default()
                    };
                    *focus_buf = shared::widgets::add_input(ui, input).has_focus();
                });
            });
        });
    }

    pub fn error_label(&mut self, ui: &mut Ui) {
        if let Some(e) = self.error.clone() {
            Frame::new().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());

                ui.vertical_centered(|ui| {
                    shared::widgets::rich_label(ui, &e, 12, theme::ERR, 500);
                });
            });
        };
    }

    ///
    /// Draws a time-info card, used in `Status::Working` screen
    ///
    /// `card_of` - (cards_count, card_index)
    ///
    pub fn info_card(
        &mut self,
        ui: &mut Ui,
        timestamp: Option<u64>,
        info: ClientInfo,
        card_of: (u8, u8),
    ) {
        if card_of.1 >= card_of.0 {
            return;
        };
        Frame::new()
            .corner_radius(8)
            .fill(theme::CARD_BG)
            .inner_margin(Margin::ZERO)
            .stroke(Stroke::new(1.0, theme::BORDER_COLOR))
            .show(ui, |ui| {
                let gap = 24.0;
                let remain_cards = (card_of.0 - card_of.1) as f32;
                let card_width = ui.available_width() / remain_cards
                    - gap * ((remain_cards - 1.0) / remain_cards);

                ui.set_height(149.0);
                ui.set_width(card_width);

                ui.vertical_centered(|ui| {
                    ui.add_space(53.5);

                    let text = match info {
                        ClientInfo::StartedAt => "Начало",
                        ClientInfo::Passed => "Прошло",
                    };

                    shared::widgets::rich_label(ui, text, 12, theme::TEXT_DIM, 400);

                    ui.add_space(4.0);

                    let mut time = String::new();

                    if let Some(ts) = timestamp {
                        // If `Passed Time` - calc duration, else just format timestamp
                        if info == ClientInfo::Passed {
                            let current_ts = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            time = shared::format_duration(ts, current_ts);
                        } else {
                            time = shared::format_time(Some(ts), false);
                        }
                    } else {
                        self.error =
                            Some("Ошибка получения временной отметки от сервера".to_string());
                    }

                    shared::widgets::rich_label(ui, &time, 18, theme::TEXT, 500);
                });
            });
    }

    ///
    /// Draws a header of ClientApp
    ///
    pub fn add_header(
        &mut self,
        ui: &mut Ui,
        status_bg: Option<Color32>,
        status_fg: Option<Color32>,
        status_dot: Option<Color32>,
        status_text: Option<&str>,
    ) {
        Frame::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(64.0);

            ui.horizontal(|ui| {
                shared::widgets::rich_label(ui, "edque - клиент", 32, theme::TEXT, 600);

                if let (Some(status_bg), Some(status_dot)) = (status_bg, status_dot) {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        Frame::new()
                            .fill(status_bg)
                            .corner_radius(14)
                            .inner_margin(Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.set_height(16.0);

                                if let (Some(status_text), Some(status_fg)) =
                                    (status_text, status_fg)
                                {
                                    shared::widgets::rich_label(
                                        ui,
                                        status_text,
                                        13,
                                        status_fg,
                                        400,
                                    );
                                }

                                Frame::new()
                                    .fill(status_dot)
                                    .corner_radius(4)
                                    .show(ui, |ui| {
                                        ui.set_width(8.0);
                                        ui.set_height(8.0);
                                    })
                            });

                        ui.add_space(10.0);

                        if let Some(id) = self.comp_id {
                            shared::widgets::rich_label(
                                ui,
                                &format!("Рабочий стол #{}", id),
                                13,
                                theme::TEXT_DIM,
                                500,
                            );
                        };
                    });
                }
            });
        });
    }
}
/// Draws a button with adaptive width
pub fn add_button(
    ui: &mut Ui,
    desired_size: Vec2,
    bg: Color32,
    fg: Color32,
    text: &str,
    font_size: f32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);

    // center text
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::new(font_size, FontFamily::Name("Inter-SemiBold".into())),
        fg,
    );
    response
}

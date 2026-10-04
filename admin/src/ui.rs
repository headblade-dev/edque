use crate::AdminApp;
use eframe::egui::{
    Align, Align2, Color32, FontFamily, FontId, Frame, Layout, Margin, Sense, Stroke, StrokeKind,
    TextEdit, Ui, Vec2,
};
use shared::{theme, Status::*};

pub mod widgets;

impl AdminApp {
    ///
    /// Toggles selection for `comp_id` in `self.selected_comps`. Used in checkboxes.
    ///
    pub fn toggle_select(&mut self, comp_id: u8) {
        if let Some(pos) = self.selected_comps.iter().position(|&x| x == comp_id) {
            self.selected_comps.remove(pos);
        } else {
            self.selected_comps.push(comp_id);
        }
    }

    ///
    /// Draws a row which shows computer statistics and controls
    ///
    pub fn comp_row(&mut self, ui: &mut Ui, comp_id: u8) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let checked = self.selected_comps.contains(&comp_id);

            let comp_state = self.comps.iter().find(|c| c.comp_id == comp_id);
            let status = comp_state.map(|c| c.status.clone());
            let started_at = comp_state.and_then(|c| c.started_at);
            let finished_at = comp_state.and_then(|c| c.finished_at);
            let score = comp_state.and_then(|c| c.score);

            let started_at = shared::format_time(started_at, false);
            let finished_at = shared::format_time(finished_at, false);

            let (bg, stroke_color, checkbox_color) = match checked {
                false => (theme::BG, theme::THIN_LINE, theme::TEXT_DIM),
                true => (theme::ACCENT_BG, theme::ACCENT_DIM, theme::ACCENT),
            };

            #[rustfmt::skip]
            let (status_bg, status_fg, status_dot, status_text, buttons) = match status {
                Some(Idle) => (
                    theme::IDLE, theme::IDLE_FG, theme::IDLE_DOT,
                    "Простаивает", &["ban"] as &[&str],
                ),
                Some(Working) => (
                    theme::WORKING, theme::WORKING_FG, theme::WORKING_DOT,
                    "В работе", &["ban", "reset"] as &[&str],
                ),
                Some(ReviewRequired) => (
                    theme::REVIEW, theme::REVIEW_FG, theme::REVIEW_DOT,
                    "Ждёт проверки", &["ban", "resume"] as &[&str],
                ),
                Some(Done) => (
                    theme::DONE, theme::DONE_FG, theme::DONE_DOT,
                    "Проверен", &["ban", "resume"] as &[&str],
                ),
                Some(Banned) => (
                    theme::BANNED, theme::BANNED_FG, theme::BANNED_DOT,
                    "Блокировка", &["reset"] as &[&str],
                ),
                _ => return,
            };

            // -----------------
            // Main Frame of row
            // -----------------
            Frame::new()
                .fill(bg)
                .stroke(Stroke::new(1.0, stroke_color))
                .corner_radius(24)
                .inner_margin(Margin {
                    top: 9,
                    bottom: 9,
                    right: 15,
                    left: 12,
                })
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(30.0);

                    ui.horizontal_centered(|ui| {
                        // Checkbox
                        self.checkbox(ui, comp_id, checked, checkbox_color);
                        ui.add_space(10.0);

                        // Status badge
                        widgets::add_status(ui, status_bg, status_fg, status_dot, status_text);
                        ui.add_space(10.0);

                        // CompID
                        #[rustfmt::skip]
                        shared::widgets::rich_label_sized(ui,
                            (24.0, 17.0),
                            &comp_id.to_string(),
                            14, theme::TEXT, 500
                        );
                        ui.add_space(10.0);

                        // Started at
                        #[rustfmt::skip]
                        shared::widgets::rich_label_sized(ui,
                            (40.0, 17.0),
                            &started_at,
                            14, theme::TEXT, 500
                        );
                        ui.add_space(10.0);

                        // Finished at
                        #[rustfmt::skip]
                        shared::widgets::rich_label_sized(ui,
                            (40.0, 17.0),
                            &finished_at,
                            14, theme::TEXT, 500
                        );
                        ui.add_space(10.0);

                        // Results
                        self.results_input(ui, comp_id, status, score);

                        // Buttons
                        if !buttons.is_empty() {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                for &button in buttons {
                                    self.action_button(ui, comp_id, button);
                                    ui.add_space(10.0);
                                }
                            });
                        }
                    });
                });
        });
    }

    ///
    /// Draws a control button for computer (ban, reset or resume)
    ///
    fn action_button(&mut self, ui: &mut Ui, comp_id: u8, text: &str) {
        let (btn_bg, btn_fg, btn_width, btn_text) = match text {
            "ban" => (theme::ACCENT_BG, theme::ACCENT_DIM, 42.0, "БАН"),
            "reset" => (theme::CARD_BG, theme::TEXT_DIM, 50.0, "СБРОС"),
            "resume" => (theme::REVIEW, theme::REVIEW_FG, 66.0, "ВЕРНУТЬ"),
            _ => (Color32::WHITE, Color32::BLACK, 42.0, "N/A"),
        };

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(btn_width, 22.0), Sense::click());

        ui.painter().rect_filled(rect, 11, btn_bg);
        ui.painter()
            .rect_stroke(rect, 11, Stroke::new(1.0, btn_fg), StrokeKind::Outside);

        #[rustfmt::skip]
        ui.painter().text(
            rect.center(), Align2::CENTER_CENTER,
            btn_text,
            FontId::new(10.0, FontFamily::Name("Inter-SemiBold".into())),
            btn_fg,
        );

        if resp.clicked() {
            match text {
                "ban" => self.ban(comp_id),
                "reset" => self.reset(comp_id),
                "resume" => self.resume(comp_id),
                _ => {}
            }
        }
    }

    ///
    /// Adds a `TextEdit` for results input
    ///
    fn results_input(
        &mut self,
        ui: &mut Ui,
        comp_id: u8,
        status: Option<shared::Status>,
        score: Option<u16>,
    ) {
        if status == Some(ReviewRequired) || status == Some(Done) {
            ui.add_space(12.0);
            shared::widgets::rich_label_sized(ui, (45.0, 17.0), "Баллы", 14, theme::TEXT, 500);
            ui.add_space(10.0);

            Frame::new()
                .fill(theme::BG)
                .stroke(Stroke::new(1.0, theme::TEXT_DIM))
                .inner_margin(Margin::symmetric(4, 2))
                .corner_radius(6)
                .show(ui, |ui| {
                    ui.set_width(32.0);
                    ui.set_height(18.0);

                    let buf = self
                        .score_bufs
                        .entry(comp_id)
                        .or_insert_with(|| score.map(shared::format_score).unwrap_or_default());

                    let response = ui.add(
                        TextEdit::singleline(buf)
                            .desired_width(60.0)
                            .frame(Frame::NONE),
                    );

                    if response.lost_focus() {
                        if let Some(score) = shared::parse_score(buf) {
                            self.set_score(comp_id, score);
                        } else {
                            *buf = score.map(shared::format_score).unwrap_or_default();
                        }
                    }
                });
        }
    }

    ///
    /// Draws a checkbox for computers selection
    ///
    fn checkbox(&mut self, ui: &mut Ui, comp_id: u8, checked: bool, color: Color32) {
        let (circle, resp_circle) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), Sense::click());

        // Draw checkbox shape depending on its state
        if checked {
            ui.painter().rect_filled(circle, 8, color);
        } else {
            ui.painter()
                .rect_stroke(circle, 8, Stroke::new(1.0, color), StrokeKind::Inside);
        }

        if resp_circle.clicked() {
            self.toggle_select(comp_id);
        }

        // Draw ✓ marker if the checkbox is checked
        if checked {
            ui.painter().text(
                circle.center(),
                Align2::CENTER_CENTER,
                "✓",
                FontId::new(14.0, FontFamily::Name("Inter-SemiBold".into())),
                theme::TEXT_ON_ACCENT,
            );
        }
    }

    ///
    /// Draws a edque-admin header
    ///
    pub fn header(&mut self, ui: &mut Ui) {
        Frame::new()
            .inner_margin(Margin::symmetric(16, 13))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(22.0);

                ui.horizontal_centered(|ui| {
                    shared::widgets::rich_label(
                        ui,
                        "edque - панель управления",
                        20,
                        theme::TEXT,
                        600,
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        shared::widgets::rich_label(
                            ui,
                            &format!("Компьютеров подключено: {}", self.comps.len()),
                            13,
                            theme::TEXT,
                            400,
                        );
                    });
                });
            });
    }

    ///
    /// Draws a edque-admin footer
    ///
    pub fn footer(&mut self, ui: &mut Ui) {
        Frame::new().inner_margin(6).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            ui.horizontal_centered(|ui| {
                // Report button
                {
                    let button = shared::widgets::Button {
                        dims: (160.0, 36.0),
                        text: "Начать".to_string(),
                        size: 13,
                        ..Default::default()
                    };
                    if shared::widgets::add_button(ui, button).clicked() {
                        self.save_report();
                    }
                }
                ui.add_space(27.0);

                // Address input
                shared::widgets::rich_label(ui, "Адрес сервера: ", 13, theme::TEXT_DIM, 400);
                ui.add_space(8.0);

                let mut input = shared::widgets::Input {
                    dims: (160.0, 14.0),
                    radius: 6,
                    margin: Margin::symmetric(8, 4),
                    buf: self.server_input.to_string(),
                    hint: "server.example.com:3000".to_string(),
                    hint_size: 12,
                    ..Default::default()
                };
                shared::widgets::add_input(ui, &mut input);
                ui.add_space(12.0);

                // Address apply button
                {
                    let button = shared::widgets::Button {
                        dims: (97.0, 32.0),
                        bg: theme::ACCENT_BG,
                        fg: theme::ACCENT_DIM,
                        stroke_color: Some(theme::ACCENT_DIM),
                        text: "Применить".to_string(),
                        size: 13,
                        ..Default::default()
                    };
                    if shared::widgets::add_button(ui, button).clicked() {
                        self.set_server();
                    }
                }
            });
        });
    }
}

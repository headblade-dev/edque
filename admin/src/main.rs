use std::{collections::HashMap, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use eframe::egui;
use shared::{CompState, ReportRequest, ScoreRequest, Status, theme};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    // create window structure
    let app = AdminApp::new();

    // window parameters (size, pos, icon, etc.)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 832.0])
            .with_resizable(false),
        ..Default::default()
    };



    // start window
    eframe::run_native(
        "edque - admin",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "Inter".to_owned(),
                Arc::new(egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter.ttf")))
            );
            fonts.font_data.insert(
                "Inter-SemiBold".to_owned(),
                Arc::new(egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")))
            );
            fonts.font_data.insert(
                "Inter-Bold".to_owned(),
                Arc::new(egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Bold.ttf")))
            );

            fonts.families.insert(
                egui::FontFamily::Name("Inter".into()),
                vec!["Inter".to_owned()]
            );
            fonts.families.insert(
                egui::FontFamily::Name("Inter-SemiBold".into()),
                vec!["Inter-SemiBold".to_owned()]
            );
            fonts.families.insert(
                egui::FontFamily::Name("Inter-Bold".into()),
                vec!["Inter-Bold".to_owned()]
            );

            cc.egui_ctx.set_fonts(fonts);
            theme::apply(&cc.egui_ctx);

            Ok(Box::new(app))
        })
    )
}

struct AdminApp {
    http: reqwest::blocking::Client,
    score_bufs: HashMap<u8, String>,
    comps: Vec<CompState>,
    error: Option<String>,
    last_poll: Option<Instant>,
    selected_comps: Vec<u8>,
    server: Option<String>,
    server_input: String,
}

impl AdminApp {
    // Constructor
    // creates http-client and self structure
    fn new() -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            score_bufs: HashMap::new(),
            comps: Vec::new(),
            last_poll: None,
            error: None,
            selected_comps: Vec::new(),
            server: None,
            server_input: String::new(),
        }
    }

    fn poll(&mut self) {
        if let None = self.server {
            return;
        }

        if let Some(t) = self.last_poll {
            if t.elapsed() < Duration::from_secs(1) {
                return;
            }
        }

        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/comps", server);

        match self.http.get(&url).send() {
            Ok(resp) => match resp.json::<Vec<CompState>>() {
                Ok(s) => {
                    self.error = None;
                    self.comps = s;
                }
                Err(e) => {
                    self.error = Some(format!("Ошибка обработки информации о компьютерах от сервера: {e}"));
                }
            }
            Err(_) => {
                self.error = Some(format!("Ошибка подключения к серверу"));
            }
        }
    }

    fn set_score(&mut self, comp_id: u8, score: u16) {
        if let None = self.server {
            return;
        }
        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/comps/{}/score", server, comp_id);
        let request = ScoreRequest {
            score,
        };

        match self.http.post(&url).json(&request).send() {
            Ok(resp) => match resp.json::<CompState>() {
                Ok(s) => {
                    if let Some(entry) = self.comps.iter_mut().find(|c| c.comp_id == comp_id) {
                        self.error = None;
                        entry.score = s.score;
                        return;
                    } else {
                        self.error = Some(format!("Компьютер #{} не найден", comp_id));
                    }
                }
                Err(e) => {
                    self.error = Some(format!("Ошибка обработки информации с сервера: {e}"));
                }
            }
            Err(_) => {
                self.error = Some(format!("Ошибка отправки результатов проверки"));
            }
        }
    }

    fn reset(&mut self, comp_id: u8) {
        if let None = self.server {
            return;
        }
        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/comps/{}/reset", server, comp_id);

        match self.http.post(&url).send() {
            Ok(resp) => match resp.json::<CompState>() {
                Ok(s) => {
                    if let Some(entry) = self.comps.iter_mut().find(|c| c.comp_id == comp_id) {
                        *entry = s;
                        self.error = None;
                        return;
                    }
                    self.error = Some(format!("Компьютер #{} не найден", comp_id));
                }
                Err(e) => {
                    self.error = Some(format!("Ошибка обработки информации с сервера: {e}"));
                }
            }
            Err(_) => {
                self.error = Some(format!("Ошибка отправки запроса о сбросе компьютера"));
            }
        }

        self.score_bufs.remove(&comp_id);
    }

    fn ban(&mut self, comp_id: u8) {
        if let None = self.server {
            return;
        }
        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/comps/{}/ban", server, comp_id);

        match self.http.post(&url).send() {
            Ok(resp) => match resp.json::<CompState>() {
                Ok(s) => {
                    if let Some(entry) = self.comps.iter_mut().find(|c| c.comp_id == comp_id) {
                        *entry = s;
                        self.error = None;
                        return;
                    }
                    self.error = Some(format!("Компьютер #{} не найден", comp_id));
                }
                Err(e) => {
                    self.error = Some(format!("Ошибка обработки информации с сервера: {e}"));
                }
            }
            Err(_) => {
                self.error = Some(format!("Ошибка отправки запроса о блокировке компьютера"));
            }
        }

        self.score_bufs.remove(&comp_id);
    }

    fn resume(&mut self, comp_id: u8) {
        if let None = self.server {
            return;
        }
        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/comps/{}/resume", server, comp_id);

        match self.http.post(&url).send() {
            Ok(resp) => match resp.json::<CompState>() {
                Ok(s) => {
                    if let Some(entry) = self.comps.iter_mut().find(|c| c.comp_id == comp_id) {
                        *entry = s;
                        self.error = None;
                        return;
                    }
                    self.error = Some(format!("computer #{comp_id} not found"));
                }
                Err(e) => {
                    self.error = Some(format!("parse: {e}"));
                }
            }
            Err(_) => {
                self.error = Some(format!("Ошибка отправки запроса о возобновлении компьютера"));
            }
        }
    }

    fn save_report(&mut self){
        if let None = self.server {
            self.error = Some("Ошибка подключения к серверу".to_string());
            return;
        }

        if self.selected_comps.is_empty() {
            self.error = Some("Для сохранения отчёта сначала нужно выбрать компьютеры".to_string());
            return;
        }

        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/report", server);
        let body = ReportRequest {
            comp_ids: self.selected_comps.clone(),
        };

        let resp = self.http.post(&url).json(&body).send()
            .map_err(|e| self.error = Some(format!("Ошибка при запросе отчёта с сервера: {e}")));
        
        if let Ok(resp) = resp {
            let csv = resp.text()
                .map_err(|e| self.error = Some(format!("Ошибка при чтении отчёта, полученного с сервера: {e}")));
            
            if let Ok(csv) = csv {
                let timestamp = shared::format_time(
                    Some(SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)),
                    true
                );

                let home = std::env::var("HOME").expect("HOME not set");

                let path = format!("{}/.local/share/edque/edque_report_{timestamp}.csv", home);

                let _ = std::fs::write(&path, csv)
                    .map_err(|e| self.error = Some(format!("Не удалось записать отчёт в файл: {e}")));

                
                self.error = Some(format!("Отчёт успешно сохранён в {path}"));
            }
        }
    }

    fn toggle_select(&mut self, comp_id: u8) {
        if let Some(pos) = self.selected_comps.iter().position(|&x| x == comp_id) {
            self.selected_comps.remove(pos);
        } else {
            self.selected_comps.push(comp_id);
        }
    }

    fn set_server(&mut self) {
        self.server = Some(self.server_input.clone());
    }

    fn separator(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 1.0), 
            egui::Sense::hover()
        );
        ui.painter().rect_filled(rect, 0.0, theme::THIN_LINE);
    }

    fn comp_row(&mut self, ui: &mut egui::Ui, comp_id: u8) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let checked = self.selected_comps.contains(&comp_id);

            let comp_state = self.comps.iter().find(|c| c.comp_id == comp_id);
            let status = comp_state.map(|c| c.status.clone());
            let started_at = comp_state.map(|c| c.started_at);
            let finished_at = comp_state.map(|c| c.finished_at);
            let score = comp_state.map(|c| c.score);
            
            if let (Some(started_at), Some(finished_at), Some(score)) = (started_at, finished_at, score) {
                let started_at = shared::format_time(started_at, false);
                let finished_at = shared::format_time(finished_at, false);

                let (
                    bg, 
                    stroke_color,
                    checkbox_color
                ) = match checked {
                    false => {
                        (
                            theme::BG,
                            theme::THIN_LINE,
                            theme::TEXT_DIM
                        )
                    }
                    true => {
                        (
                            theme::ACCENT_BG,
                            theme::ACCENT_DIM,
                            theme::ACCENT
                        )
                    }
                };

                let (status_bg, status_fg, status_dot, status_text, status_offset, buttons) = match status {
                    Some(Status::Idle) => {
                        (
                            theme::IDLE,
                            theme::IDLE_FG,
                            theme::IDLE_DOT,
                            "Простаивает",
                            Some(7.0),
                            vec!["ban"]
                        )
                    }
                    Some(Status::Working) => {
                        (
                            theme::WORKING,
                            theme::WORKING_FG,
                            theme::WORKING_DOT,
                            "В работе",
                            Some(33.0),
                            vec!["ban", "reset"]
                        )
                    }
                    Some(Status::ReviewRequired) => {
                        (
                            theme::REVIEW,
                            theme::REVIEW_FG,
                            theme::REVIEW_DOT,
                            "Ждёт проверки",
                            None,
                            vec!["ban", "resume"]
                        )
                    }
                    Some(Status::Done) => {
                        (
                            theme::DONE,
                            theme::DONE_FG,
                            theme::DONE_DOT,
                            "Проверен",
                            Some(27.0),
                            vec!["ban", "resume"]
                        )
                    }
                    Some(Status::Banned) => {
                        (
                            theme::BANNED,
                            theme::BANNED_FG,
                            theme::BANNED_DOT,
                            "Блокировка",
                            Some(13.0),
                            vec!["reset"]
                        )
                    }
                    _ => {
                        return
                    }
                };

                egui::Frame::new()
                .fill(bg)
                .stroke(egui::Stroke::new(1.0, stroke_color))
                .corner_radius(24)
                .inner_margin(egui::Margin{
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
                        {
                            let (circle, resp_circle) = ui.allocate_exact_size(
                                egui::Vec2::new(16.0, 16.0), 
                                egui::Sense::click()
                            );

                            if !checked {
                                ui.painter().rect_stroke(
                                    circle,
                                    8,
                                    egui::Stroke::new(1.0, checkbox_color),
                                    egui::StrokeKind::Inside
                                );
                            } else {
                                ui.painter().rect_filled(
                                    circle,
                                    8,
                                    checkbox_color
                                );
                            }

                            if resp_circle.clicked() {
                                self.toggle_select(comp_id);
                            }

                            if checked {
                                ui.painter().text(
                                    circle.center(), 
                                    egui::Align2::CENTER_CENTER, 
                                    "✓", 
                                    egui::FontId::new(14.0, egui::FontFamily::Name("Inter-SemiBold".into())), 
                                    theme::TEXT_ON_ACCENT
                                );
                            }
                        }
                        ui.add_space(10.0);

                        // Pill
                        {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            egui::Frame::new()
                            .corner_radius(14)
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .fill(status_bg)
                            .show(ui, |ui| {
                                ui.horizontal_centered(|ui| {
                                    let (dot, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());

                                    ui.painter().rect_filled(dot, 4.0, status_dot);

                                    ui.add_space(6.0);

                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(status_text)
                                                .family(egui::FontFamily::Name("Inter".into()))
                                                .size(13.0)
                                                .color(status_fg)
                                                .variation("wght", 500.0)
                                        ).selectable(false)
                                    )
                                })
                            });
                        }
                        ui.add_space(10.0);
                        if let Some(status_offset) = status_offset {
                            ui.add_space(status_offset + 10.0);
                        }

                        // CompID
                        {
                            ui.add_sized(
                                egui::Vec2::new(24.0, 17.0),
                                egui::Label::new(
                                    egui::RichText::new(comp_id.to_string())
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .color(theme::TEXT)
                                        .size(14.0)
                                        .variation("wght", 500.0)
                                ).selectable(false)
                            );
                        }
                        ui.add_space(10.0);

                        // Time
                        {
                            ui.add_sized(
                                egui::Vec2::new(40.0, 17.0),
                                egui::Label::new(
                                    egui::RichText::new(started_at)
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .size(14.0)
                                        .color(theme::TEXT)
                                        .variation("wght", 500.0)
                                ).selectable(false)
                            );
                            ui.add_space(10.0);
                            ui.add_sized(
                                egui::Vec2::new(40.0, 17.0),
                                egui::Label::new(
                                    egui::RichText::new(finished_at)
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .size(14.0)
                                        .color(theme::TEXT)
                                        .variation("wght", 500.0)
                                ).selectable(false)
                            );
                        }
                        ui.add_space(10.0);


                        // Results
                        if status == Some(Status::ReviewRequired) || status == Some(Status::Done) {
                            ui.add_space(12.0);
                            ui.add_sized(
                                egui::Vec2::new(45.0, 17.0),
                                egui::Label::new(
                                    egui::RichText::new("Баллы")
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .size(14.0)
                                        .color(theme::TEXT)
                                        .variation("wght", 500.0)
                                ).selectable(false)
                            );
                            ui.add_space(10.0);

                            egui::Frame::new()
                                .fill(theme::BG)
                                .stroke(egui::Stroke::new(1.0, theme::TEXT_DIM))
                                .inner_margin(egui::Margin::symmetric(4, 2))
                                .corner_radius(6)

                                .show(ui, |ui| {
                                    ui.set_width(32.0);
                                    ui.set_height(18.0);

                                    let buf = self.score_bufs
                                        .entry(comp_id)
                                        .or_insert_with(|| score.map(shared::format_score).unwrap_or_default());

                                    let response = ui.add (
                                        egui::TextEdit::singleline(buf).desired_width(60.0)
                                            .frame(egui::Frame::NONE)
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
                        
                        // Buttons
                        if !buttons.is_empty() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                for button in buttons {
                                    let (btn_bg, btn_fg, btn_width, btn_text) = match button {
                                        "ban" => (theme::ACCENT_BG, theme::ACCENT_DIM, 42.0, "БАН"),
                                        "reset" => (theme::CARD_BG, theme::TEXT_DIM, 50.0, "СБРОС"),
                                        "resume" => (theme::REVIEW, theme::REVIEW_FG, 66.0, "ВЕРНУТЬ"),
                                        _ => (egui::Color32::WHITE, egui::Color32::BLACK, 42.0, "N/A")
                                    };

                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::Vec2::new(btn_width, 22.0),
                                        egui::Sense::click()
                                    );

                                    ui.painter().rect_filled(rect, 11, btn_bg);
                                    ui.painter().rect_stroke(rect, 11, egui::Stroke::new(1.0, btn_fg), egui::StrokeKind::Outside);

                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        btn_text,
                                        egui::FontId::new(10.0, egui::FontFamily::Name("Inter-SemiBold".into())),
                                        btn_fg
                                    );

                                    if resp.clicked() {
                                        match button {
                                            "ban" => self.ban(comp_id),
                                            "reset" => self.reset(comp_id),
                                            "resume" => self.resume(comp_id),
                                            _ => {}
                                        }
                                    }

                                    ui.add_space(10.0);
                                }
                            });
                        }
                    });
                });
            } else {
                self.error = Some("Ошибка данных сервера".to_string());
            }
        });
    }
}

impl eframe::App for AdminApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().request_repaint_after(Duration::from_secs(1));

        egui::Frame::new()
            .fill(theme::BG)
            .corner_radius(egui::CornerRadius{
                nw: 24,
                ne: 24,
                sw: 14,
                se: 14,
            })
            .inner_margin(egui::Margin::ZERO)
            
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                // ------
                // Header
                // ------
                egui::Frame::new().inner_margin(egui::Margin::symmetric(16, 13)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(22.0);

                    ui.horizontal_centered(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("edque - панель управления")
                                    .family(egui::FontFamily::Name("Inter".into()))
                                    .size(20.0)
                                    .color(theme::TEXT)
                                    .variation("wght", 600.0)
                            ).selectable(false)
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!("Компьютеров подключено: {}", self.comps.len()))
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .size(13.0)
                                        .color(theme::TEXT)
                                        .variation("wght", 400.0)
                                ).selectable(false)
                            );
                        });
                    });
                });
                self.separator(ui);
                
                // ----
                // Body
                // ----
                egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(14, 9))
                
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height() - 48.0 - 32.0); // all - footer - error_line

                    ui.vertical_centered(|ui| {
                        for (idx, comp_state) in self.comps.clone().into_iter().enumerate() {
                            let comp_id = comp_state.comp_id;
                            if idx > 0 { ui.add_space(10.0); }
                            self.comp_row(ui, comp_id)
                        }
                    })
                });

                // ----------
                // Error line
                // ----------
                egui::Frame::new().inner_margin(9).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height() - 48.0); // all - footer

                    if let Some(e) = self.error.clone() {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(e)
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .color(theme::ERR)
                                        .size(12.0)
                                        .variation("wght", 500.0)
                                ).selectable(true)
                            )
                        });
                    };
                });
                self.separator(ui);

                // ------
                // Footer
                // ------
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    egui::Frame::new().inner_margin(6).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(ui.available_height());

                        ui.horizontal_centered(|ui| {
                            {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::Vec2::new(160.0, 36.0), 
                                    egui::Sense::click()
                                );

                                ui.painter().rect_filled(rect, 8, theme::ACCENT);

                                ui.painter().text(
                                    rect.center(), 
                                    egui::Align2::CENTER_CENTER, 
                                    "Сохранить отчёт", 
                                    egui::FontId::new(13.0, egui::FontFamily::Name("Inter-SemiBold".into())), 
                                    theme::TEXT_ON_ACCENT
                                );

                                if resp.clicked() {
                                    self.save_report();
                                }
                            }
                            ui.add_space(27.0);

                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new("Адрес сервера: ")
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .size(13.0)
                                        .color(theme::TEXT_DIM)
                                ).selectable(false)
                            );
                            ui.add_space(8.0);

                            egui::Frame::new()
                            .corner_radius(6)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .stroke(
                                egui::Stroke::new(1.0, theme::TEXT_DIM)
                            )
                            .show(ui, |ui| {
                                ui.set_width(160.0);
                                ui.add_sized(
                                    egui::Vec2::new(160.0, 14.0), 
                                    egui::TextEdit::singleline(&mut self.server_input)
                                        .hint_text(
                                            egui::RichText::new("server.example.com:3000")
                                                .family(egui::FontFamily::Name("Inter".into()))
                                                .size(12.0)
                                                .color(theme::TEXT_DIM)
                                        )
                                        .frame(egui::Frame::NONE)
                                );
                            });
                            ui.add_space(12.0);
                            
                            {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::Vec2::new(97.0, 36.0), 
                                    egui::Sense::click()
                                );

                                ui.painter().rect_filled(rect, 8, theme::ACCENT_BG);
                                ui.painter().rect_stroke(
                                    rect, 
                                    8, 
                                    egui::Stroke::new(1.0, theme::ACCENT_DIM),
                                    egui::StrokeKind::Outside
                                );

                                ui.painter().text(
                                    rect.center(), 
                                    egui::Align2::CENTER_CENTER, 
                                    "Применить", 
                                    egui::FontId::new(13.0, egui::FontFamily::Name("Inter-SemiBold".into())), 
                                    theme::ACCENT_DIM
                                );

                                if resp.clicked() {
                                    self.set_server();
                                }
                            }
                        });
                    });
                });   
            });
        });
    }
}
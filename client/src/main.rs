use eframe::egui;
use std::time::{Duration, Instant};
use shared::{CompState, RegisterRequest, Status};

const SERVER: &str = "http://127.0.0.1:3000";

fn main() -> eframe::Result<()> {
    // create window structure
    let app = ClientApp::new();

    // window parameters (size, pos, icon, etc.)
    let options = eframe::NativeOptions::default();

    // start window
    eframe::run_native(
        "edque client", 
        options, 
        Box::new(|_cc| Ok(Box::new(app)))
    )
}

struct ClientApp {
    http: reqwest::blocking::Client,
    comp_id: Option<u8>,
    input: String,
    state: Option<CompState>,
    error: Option<String>,
    last_poll: Option<Instant>,
}

impl ClientApp {
    fn new() -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            http,
            comp_id: None,
            input: String::new(),
            state: None,
            error: None,
            last_poll: None,
        }
    }

    fn register(&mut self, set_id: u8) -> bool {
        let url = format!("{SERVER}/api/register");
        let request = RegisterRequest {
            comp_id: set_id,
            hostname: std::env::var("HOSTNAME").ok(),
        };

        match self.http.post(&url).json(&request).send() {
            Ok(resp) => match resp.json::<CompState>() {
                Ok(s) => {
                    self.state = Some(s);
                    self.comp_id = Some(set_id);
                    self.error = None;
                    true
                }
                Err(e) => {
                    self.error = Some(format!("parse: {e}"));
                    false
                }
            }
            Err(e) => {
                self.error = Some(format!("register: {e}"));
                false
            }
        }
    }

    fn poll(&mut self) {
        let Some(id) = self.comp_id else { return };

        if let Some(t) = self.last_poll {
            if t.elapsed() < Duration::from_secs(1) {
                return;
            }
        }

        self.last_poll = Some(Instant::now());

        let url = format!("{SERVER}/api/comps/{id}");
        match self.http.get(&url).send() {
            Ok(response) => match response.json::<CompState>() {
                Ok(s) => {
                    self.state = Some(s);
                    self.error = None;
                }
                Err(e) => { self.error = Some(format!("parse: {e}")) }
            }
            Err(e) => { self.error = Some(format!("poll: {e}")) }
        }
    }

    fn post_action(&mut self, action: &str) {
        let Some(id) = self.comp_id else { return };

        let url = format!("{SERVER}/api/comps/{id}/{action}");
        match self.http.post(&url).send() {
            Ok(response) => {
                if !response.status().is_success() {
                    self.error = Some(format!("{}: HTTP {}", action, response.status()));
                }
                if let Ok(s) = response.json::<CompState>() {
                    self.state = Some(s);
                    self.error = None;
                }
            }
            Err(e) => self.error = Some(format!("{}: {}", action, e)),
        }
    }
}

impl eframe::App for ClientApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().request_repaint_after(Duration::from_secs(1));

        ui.vertical_centered(|ui| {
            if let Some(err) = self.error.clone() {
                ui.colored_label(egui::Color32::RED, &err);
                eprintln!("{err}");
                ui.add_space(10.0);
            }

            ui.add_space(40.0);

            if self.comp_id.is_none() {
                ui.heading("Номер рабочего стола");
                ui.add_space(20.0);

                ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .desired_width(80.0)
                        .hint_text("5"),
                );

                ui.add_space(20.0);
                if ui.button("Подключиться").clicked() {
                    if let Ok(id) = self.input.trim().parse::<u8>() {
                        self.register(id);
                    } else {
                        self.error = Some("Введите число".into());
                    }
                }
            } else {
                ui.heading(format!("Номер стола: {}", self.comp_id.unwrap()));
                ui.add_space(20.0);

                let status = self.state.as_ref().map(|s| s.status.clone());
                match status {
                    Some(Status::Idle) | None => {
                        if ui.add_sized(
                            [200.0, 60.0], 
                            egui::Button::new("Начать")
                        ).clicked() {
                            self.post_action("start");
                        }
                    }
                    Some(Status::Working) => {
                        if ui.add_sized(
                            [200.0, 60.0], 
                            egui::Button::new("Завершить")
                        ).clicked() {
                            self.post_action("finish");
                        }
                    }
                    Some(Status::ReviewRequired) => {
                        ui.label(egui::RichText::new("Ожидайте результатов проверки").size(24.0));
                    }
                    Some(Status::Offline) => {
                        ui.label(egui::RichText::new("Оффлайн").size(24.0));
                    }
                    Some(Status::Done) => {
                        let score_text = self.state.as_ref()
                            .and_then(|s| s.score)
                            .map(shared::format_score)
                            .unwrap_or_else(|| "-".into());

                        let duration_text = self.state.as_ref()
                            .and_then(|s| match (s.started_at, s.finished_at) {
                                (Some(a), Some(b)) => Some(shared::format_duration(a, b)),
                                _ => None,
                            })
                            .unwrap_or_else(|| "-".into());

                        ui.label(egui::RichText::new("Результаты:").size(24.0));
                        ui.label(
                            egui::RichText::new(format!("{} баллов, {}", score_text, duration_text))
                            .size(24.0)
                        );
                    }
                    Some(Status::Banned) => {
                        ui.colored_label(
                            egui::Color32::RED,
                            egui::RichText::new(
                                format!("ЗАБЛОКИРОВАН")
                            ).size(48.0)
                        );
                    }
                }
            }
        });
    }
}
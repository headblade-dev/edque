use eframe::egui;
use shared::{theme, CompState, RegisterRequest, Status};
use std::time::{Duration, Instant};

mod ui;

enum ClientField {
    CompID,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientInfo {
    StartedAt,
    Passed,
}

fn main() -> eframe::Result<()> {
    // create window structure
    let app = ClientApp::new();

    // window parameters (size, pos, icon, etc.)
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/Icon.png"))
        .expect("Icon must be valid PNG");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 373.0])
            .with_resizable(false)
            .with_icon(icon),
        ..Default::default()
    };

    // start window
    eframe::run_native(
        "edque client",
        options,
        Box::new(|cc| {
            theme::add_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

struct ClientApp {
    http: reqwest::blocking::Client,
    comp_id: Option<u8>,
    comp_id_input: String,
    comp_id_input_focused: bool,
    state: Option<CompState>,
    error: Option<String>,
    last_poll: Option<Instant>,
    server: Option<String>,
    server_input: String,
    server_input_focused: bool,
}

impl ClientApp {
    ///
    /// Creates HTTP-client and constructs ClientApp structure
    ///
    fn new() -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            http,
            comp_id: None,
            comp_id_input: String::new(),
            comp_id_input_focused: false,
            state: None,
            error: None,
            last_poll: None,
            server: None,
            server_input: String::new(),
            server_input_focused: false,
        }
    }

    ///
    /// Sets server from buffer
    ///
    fn set_server(&mut self) {
        if !self.server_input.is_empty() {
            self.server = Some(self.server_input.clone());
        }
    }

    ///
    /// Sends POST request to register computer on server
    ///
    fn register(&mut self, set_id: u8) -> bool {
        let Some(server) = self.server.clone() else {
            return false;
        };

        let url = format!("http://{}/api/register", server);
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
                    self.error = Some(format!(
                        "Ошибка обработки информации, полученной с сервера (REGISTER): {e}"
                    ));
                    false
                }
            },
            Err(_) => {
                self.error = Some(format!("Ошибка подключения к серверу"));
                false
            }
        }
    }

    ///
    /// GETs `CompState` for local computer from server with interval
    ///
    fn poll(&mut self) {
        let Some(server) = self.server.clone() else {
            return;
        };

        let Some(id) = self.comp_id else { return };

        if let Some(t) = self.last_poll {
            if t.elapsed() < Duration::from_millis(500) {
                return;
            }
        }

        self.last_poll = Some(Instant::now());

        let url = format!("http://{}/api/comps/{id}", server);
        match self.http.get(&url).send() {
            Ok(response) => match response.json::<CompState>() {
                Ok(s) => {
                    self.state = Some(s);
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(format!(
                        "Ошибка обработки информации, полученной с сервера (POLL): {e}"
                    ))
                }
            },
            Err(_) => self.error = Some(format!("Ошибка подключения к серверу")),
        }
    }

    ///
    /// Formats and send POST requests
    ///
    fn post_action(&mut self, action: &str) {
        let Some(server) = self.server.clone() else {
            return;
        };

        let Some(id) = self.comp_id else { return };

        let url = format!("http://{server}/api/comps/{id}/{action}");
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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::get_window_bg_color()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().request_repaint_after(Duration::from_secs(1));

        // -----------------
        // Connection screen
        // -----------------
        if self.comp_id.is_none() | self.server.is_none() {
            egui::Frame::new()
                .fill(theme::BG)
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin {
                    left: 32,
                    right: 32,
                    top: 32,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height() - 24.0);

                    ui.vertical(|ui| {
                        self.add_header(ui, None, None, None, None);

                        ui.add_space(16.0);

                        self.labeled_input(ui, "Номер ПК", "Например, 3", ClientField::CompID);

                        ui.add_space(16.0);

                        self.labeled_input(
                            ui,
                            "Адрес сервера",
                            "server.example.com:3000",
                            ClientField::Server,
                        );

                        ui.add_space(16.0);

                        let response = ui::add_button(
                            ui,
                            egui::vec2(ui.available_width(), 64.0),
                            theme::ACCENT,
                            theme::TEXT_ON_ACCENT,
                            "Подключиться",
                            16.0,
                        );

                        if response.clicked() {
                            self.set_server();
                            if let Ok(id) = self.comp_id_input.trim().parse::<u8>() {
                                if !self.register(id) {
                                    self.error = Some("Ошибка подключения к серверу".into());
                                }
                            } else {
                                self.error =
                                    Some("Номер ПК должен являться целым числом > 0".into());
                            }
                        }
                    });
                });

            self.error_label(ui);
        } else {
            let status = self.state.as_ref().map(|s| s.status.clone());
            let started_at = self.state.as_ref().and_then(|s| s.started_at);
            let finished_at = self.state.as_ref().and_then(|s| s.finished_at);
            let score = self.state.as_ref().and_then(|s| s.score);
            match status {
                // -----------
                // Idle screen
                // -----------
                Some(Status::Idle) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 32,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            self.add_header(
                                ui,
                                Some(theme::IDLE),
                                None,
                                Some(theme::IDLE_DOT),
                                None,
                            );

                            egui::Frame::new().show(ui, |ui| {
                                ui.set_height(ui.available_height() - 16.0 - 64.0 - 32.0);
                                ui.set_width(ui.available_width());

                                ui.vertical_centered(|ui| {
                                    ui.add_space(46.5);

                                    #[rustfmt::skip]
                                shared::widgets::rich_label(ui, 
                                    "Готов к работе?", 
                                    24, theme::TEXT, 600
                                );
                                    ui.add_space(5.0);

                                    #[rustfmt::skip]
                                shared::widgets::rich_label(ui, 
                                    "Запусти машины, но не входи в аккаунт!", 
                                    16, theme::TEXT, 400
                                );
                                });
                            });

                            ui.add_space(16.0);

                            #[rustfmt::skip]
                        let response = shared::widgets::add_button(ui, 
                            (ui.available_width(), 64.0), 8, 
                            theme::ACCENT, theme::TEXT_ON_ACCENT, None, 
                            "Начать", "Inter-SemiBold", 16);

                            if response.clicked() {
                                self.post_action("start");
                            }
                        });

                    self.error_label(ui);
                }

                // --------------
                // Working screen
                // --------------
                Some(Status::Working) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(egui::CornerRadius::same(14))
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 32,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            #[rustfmt::skip]
                        self.add_header(ui, 
                            Some(theme::WORKING), Some(theme::WORKING_FG), 
                            Some(theme::WORKING_DOT), Some("В работе")
                        );

                            egui::Frame::new().show(ui, |ui| {
                                ui.set_height(ui.available_height() - 16.0 - 64.0 - 32.0);
                                ui.set_width(ui.available_width());

                                ui.horizontal_centered(|ui| {
                                    self.info_card(ui, started_at, ClientInfo::StartedAt, (2, 0));
                                    ui.add_space(12.0);
                                    self.info_card(ui, started_at, ClientInfo::Passed, (2, 1));
                                });
                            });

                            ui.add_space(16.0);

                            #[rustfmt::skip]
                        let response = shared::widgets::add_button(ui, 
                            (ui.available_width(), 64.0), 8, 
                            theme::ACCENT, theme::TEXT_ON_ACCENT, None, 
                            "Завершить", "Inter-SemiBold", 16);

                            if response.clicked() {
                                self.post_action("finish");
                            }
                        });

                    self.error_label(ui);
                }

                // -------------
                // Review screen
                // -------------
                Some(Status::ReviewRequired) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 32,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            #[rustfmt::skip]
                        self.add_header(ui, 
                            Some(theme::REVIEW), Some(theme::REVIEW_FG), 
                            Some(theme::REVIEW_DOT), Some("На проверке")
                        );

                            egui::Frame::new().show(ui, |ui| {
                                ui.set_height(ui.available_height() - 16.0 - 32.0);
                                ui.set_width(ui.available_width());

                                ui.vertical_centered(|ui| {
                                    egui::Frame::new().inner_margin(egui::Margin::ZERO).show(
                                        ui,
                                        |ui| {
                                            ui.set_height(ui.available_height());
                                            ui.set_width(ui.available_width());

                                            ui.add_space(100.0);

                                            #[rustfmt::skip]
                                    shared::widgets::rich_label(ui, 
                                        "Ожидайте результатов проверки", 
                                        24, theme::TEXT, 600
                                    );

                                            ui.add_space(116.0);
                                        },
                                    );
                                });
                            });
                        });

                    self.error_label(ui);
                }

                // -----------
                // Done screen
                // -----------
                Some(Status::Done) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 32,
                            bottom: 0,
                        })
                    
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(ui.available_height() - 16.0 - 32.0);

                        #[rustfmt::skip]
                        self.add_header(ui, 
                            Some(theme::DONE), Some(theme::DONE_FG), 
                            Some(theme::DONE_DOT), Some("Проверено")
                        );

                        ui.add_space(89.0);

                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(49.0);
                            ui.vertical_centered(|ui| {
                                egui::Frame::new().show(ui, |ui| {
                                    ui.set_width(196.0);

                                    ui.horizontal(|ui| {
                                        #[rustfmt::skip]
                                        shared::widgets::rich_label(ui, 
                                            "Результат:", 
                                            24, theme::TEXT, 500
                                        );
                                        ui.add_space(8.0);

                                        if let Some(score) = score {
                                            let score_text = shared::format_score(score);

                                            #[rustfmt::skip]
                                            shared::widgets::rich_label(ui, 
                                                &score_text, 
                                                24, theme::TEXT, 700
                                            );
                                        } else {
                                            self.error = Some("Ошибка получения результатов проверки с сервера".to_string());
                                        }
                                    });
                                });  
                            
                                egui::Frame::new().show(ui, |ui| {
                                    ui.set_width(83.0);

                                    ui.horizontal(|ui| {
                                        #[rustfmt::skip]
                                        shared::widgets::rich_label(ui, 
                                            "Время:", 
                                            12, theme::TEXT_DIM, 400
                                        );
                                        ui.add_space(4.0);

                                        if let (Some(started_at), Some(finished_at)) = (started_at, finished_at) {
                                            let duration = shared::format_duration(started_at, finished_at);

                                            #[rustfmt::skip]
                                            shared::widgets::rich_label(ui, 
                                                &duration, 
                                                12, theme::TEXT_DIM, 500
                                            );
                                        } else {
                                            self.error = Some("Ошибка получения данных о времени с сервера".to_string());
                                        }
                                    });
                                });
                            });
                        });

                        ui.add_space(89.0);
                    });

                    self.error_label(ui);
                }

                // -------------
                // Banned screen
                // -------------
                Some(Status::Banned) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 32,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            self.add_header(
                                ui,
                                Some(theme::BANNED),
                                Some(theme::BANNED_FG),
                                Some(theme::BANNED_DOT),
                                Some("BAN"),
                            );

                            ui.add_space(90.5);

                            ui.vertical_centered(|ui| {
                                #[rustfmt::skip]
                            shared::widgets::rich_label(ui, 
                                "ЗАБЛОКИРОВАН", 
                                40, theme::ERR, 600
                            );
                            });

                            ui.add_space(90.5);
                        });

                    self.error_label(ui);
                }

                _ => {}
            }
        }
    }
}

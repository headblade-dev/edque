use eframe::egui;
use std::{
    sync::Arc, time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use shared::{CompState, RegisterRequest, Status, theme};




enum ClientField {
    CompID,
    Server
}

fn main() -> eframe::Result<()> {
    // create window structure
    let app = ClientApp::new();

    // window parameters (size, pos, icon, etc.)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 373.0]),
        ..Default::default()
    };

    // start window
    eframe::run_native(
        "edque client", 
        options, 
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "Inter".to_owned(), 
                Arc::new(egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter.ttf"))),
            );
            fonts.font_data.insert(
                "Inter-SemiBold".to_owned(), 
                Arc::new(egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf"))),
            );
            fonts.font_data.insert(
                "Inter-Bold".to_owned(), 
                Arc::new(egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Bold.ttf"))),
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

    fn set_server(&mut self) {
        if !self.server_input.is_empty() {
            self.server = Some(self.server_input.clone());
        }
    }

    fn register(&mut self, set_id: u8) -> bool {
        let Some(server) = self.server.clone() else { return false; };

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
                    self.error = Some(format!("Ошибка обработки информации, полученной с сервера (REGISTER): {e}"));
                    false
                }
            }
            Err(_) => {
                self.error = Some(format!("Ошибка подключения к серверу"));
                false
            }
        }
    }

    fn poll(&mut self) {
        let Some(server) = self.server.clone() else { return; };

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
                Err(e) => { self.error = Some(format!("Ошибка обработки информации, полученной с сервера (POLL): {e}")) }
            }
            Err(_) => { self.error = Some(format!("Ошибка подключения к серверу")) }
        }
    }

    fn post_action(&mut self, action: &str) {
        let Some(server) = self.server.clone() else { return };

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

    fn labeled_input(&mut self, ui: &mut egui::Ui, label: &str, hint: &str, field: ClientField) {
        let (buf, focus_buf) = match field {
            ClientField::CompID => (&mut self.comp_id_input, &mut self.comp_id_input_focused),
            ClientField::Server => (&mut self.server_input, &mut self.server_input_focused)
        };

        egui::Frame::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(64.0);
    
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(label)
                        .family(egui::FontFamily::Name("Inter".into()))
                        .size(15.0)
                        .color(theme::TEXT)
                        .variation("wght", 600.0)
                    ).selectable(false)
                );
    
                let (offset, stroke) = if *focus_buf {
                    (1, egui::Stroke::new(2.0, theme::ACCENT))
                } else {
                    (0, egui::Stroke::new(1.0, theme::BORDER_COLOR))
                };
    
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::new()
                    .corner_radius(8)
                    .stroke(stroke)
                    .inner_margin(egui::Margin::same(16 - offset as i8))
    
                    .show(ui, |ui| {
                        ui.set_width(308.0);
                        if ui.add(
                            egui::TextEdit::singleline(buf)
                                .font(egui::FontId::new(
                                    16.0,
                                    egui::FontFamily::Name("Inter".into())
                                ))
                                .hint_text(
                                    egui::RichText::new(hint)
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .size(16.0)
                                        .color(theme::TEXT_DIM)
                                )
                                .frame(egui::Frame::NONE)
                                .desired_width(308.0)
                        ).has_focus() {
                            *focus_buf = true;
                        } else {
                            *focus_buf = false;
                        }
                    });
                });
            });
        });
    }
    
    fn add_header(
        &mut self, 
        ui: &mut egui::Ui,
        status_bg: Option<egui::Color32>,
        status_fg: Option<egui::Color32>,
        status_dot: Option<egui::Color32>,
        status_text: Option<&str>,
    ) {
        egui::Frame::new().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(64.0);
        
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new("edque - клиент")
                                .family(egui::FontFamily::Name("Inter".into()))
                                .size(32.0)
                                .color(theme::TEXT)
                                .variation("wght", 600.0)
                        ).selectable(false)
                    );
                    
                    if let (Some(status_bg), Some(status_dot)) = (status_bg, status_dot) {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            egui::Frame::new()
                                .fill(status_bg)
                                .corner_radius(14)
                                .inner_margin(egui::Margin::symmetric(10, 6))
        
                            .show(ui, |ui| {
                                ui.set_height(16.0);
        
                                
                                if let (Some(status_text), Some(status_fg)) = (status_text, status_fg) {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(status_text)
                                                .family(egui::FontFamily::Name("Inter".into()))
                                                .size(13.0)
                                                .color(status_fg)
                                        ).selectable(false)
                                    );
                                }
        
                                egui::Frame::new()
                                    .fill(status_dot)
                                    .corner_radius(4)
        
                                    .show(ui, |ui| {
                                        ui.set_width(8.0);
                                        ui.set_height(8.0);
                                    })
                            });
        
                            ui.add_space(10.0);
        
                            if let Some(id) = self.comp_id {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(format!("Рабочий стол #{}", id))
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(13.0)
                                            .color(theme::TEXT_DIM)
                                            .variation("wght", 500.0)  
                                    ).selectable(false)
                                );
                            };
                        });  
                    }
                });
        });
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
                .inner_margin(
                    egui::Margin {
                        left: 32,
                        right: 32,
                        top: 32,
                        bottom: 0,
                    }
                )
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height() - 24.0);

                ui.vertical(|ui| {
                    self.add_header(ui, None, None, None, None);

                    ui.add_space(16.0);

                    self.labeled_input(ui, "Номер ПК", "Например, 3", ClientField::CompID);

                    ui.add_space(16.0);

                    self.labeled_input(ui, "Адрес сервера", "server.example.com:3000", ClientField::Server);

                    ui.add_space(16.0);

                    let response = add_button(
                        ui,
                        egui::vec2(ui.available_width(), 64.0),
                        theme::ACCENT,
                        theme::TEXT_ON_ACCENT,
                        "Подключиться",
                        16.0
                    );

                    if response.clicked() {
                        self.set_server();
                        if let Ok(id) = self.comp_id_input.trim().parse::<u8>() {
                            if !self.register(id) {
                                self.error = Some("Ошибка подключения к серверу".into());
                            }
                        } else {
                            self.error = Some("Номер ПК должен являться целым числом > 0".into());
                        }
                    }
                });
            });

            if let Some(e) = self.error.clone() {
                egui::Frame::new().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());

                    ui.vertical_centered(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(e)
                                    .family(egui::FontFamily::Name("Inter".into()))
                                    .size(12.0)
                                    .color(theme::ERR)
                            ).selectable(true)
                        );
                    });
                });
            };
        } else {
            let status = self.state.as_ref().map(|s| s.status.clone());
            let started_at = self.state.as_ref().map(|s| s.started_at);
            let finished_at = self.state.as_ref().map(|s| s.finished_at);
            let score = self.state.as_ref().map(|s| s.score);
            match status {
                // -----------
                // Idle screen
                // -----------
                Some(Status::Idle) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(
                            egui::Margin {
                                left: 32,
                                right: 32,
                                top: 32,
                                bottom: 0,
                            }
                        )
                    .show(ui, |ui| {
                        self.add_header(ui, Some(theme::IDLE), None, Some(theme::IDLE_DOT), None);

                        egui::Frame::new().show(ui, |ui| {
                            ui.set_height(ui.available_height() - 16.0 - 64.0 - 32.0);
                            ui.set_width(ui.available_width());

                            ui.vertical_centered(|ui| {
                                ui.add_space(46.5);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("Готов к работе?")
                                            .family(egui::FontFamily::Name("Inter-SemiBold".into()))
                                            .size(24.0)
                                            .color(theme::TEXT)
                                    ).selectable(false)
                                );
                                ui.add_space(5.0);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("Запусти машины, но не входи в аккаунт!")
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(16.0)
                                            .color(theme::TEXT)
                                    ).selectable(false)
                                );
                            });

                            
                        });

                        ui.add_space(16.0);

                        let response = add_button(
                            ui,
                            egui::vec2(ui.available_width(), 64.0),
                            theme::ACCENT,
                            theme::TEXT_ON_ACCENT,
                            "Начать",
                            16.0
                        );

                        if response.clicked() {
                            self.post_action("start");
                        }
                    });

                    if let Some(e) = self.error.clone() {
                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());

                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(e)
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(12.0)
                                            .color(theme::ERR)
                                    ).selectable(true)
                                );
                            });
                        });
                    };
                }

                // --------------
                // Working screen
                // --------------
                Some(Status::Working) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(egui::CornerRadius::same(14))
                        .inner_margin(
                            egui::Margin {
                                left: 32,
                                right: 32,
                                top: 32,
                                bottom: 0,
                            }
                        )
                    .show(ui, |ui| {
                        self.add_header(
                            ui, 
                            Some(theme::WORKING), 
                            Some(theme::WORKING_FG), 
                            Some(theme::WORKING_DOT), 
                            Some("В работе")
                        );

                        egui::Frame::new().show(ui, |ui| {
                            ui.set_height(ui.available_height() - 16.0 - 64.0 - 32.0);
                            ui.set_width(ui.available_width());

                            ui.horizontal_centered(|ui| {
                                egui::Frame::new()
                                .corner_radius(8)
                                .fill(theme::CARD_BG)
                                .inner_margin(egui::Margin::ZERO)
                                .stroke(egui::Stroke::new(1.0, theme::BORDER_COLOR))

                                .show(ui, |ui| {
                                    ui.set_height(149.0);
                                    ui.set_width(ui.available_width() / 2.0 - 12.0);
                                    
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(53.5);
                                        
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("Начало")
                                                    .family(egui::FontFamily::Name("Inter".into()))
                                                    .size(12.0)
                                                    .color(theme::TEXT_DIM)
                                            ).selectable(false)
                                        );
                                        ui.add_space(4.0);

                                        let mut started_time = String::new();

                                        if let Some(started_at) = started_at {
                                            started_time = shared::format_time(started_at);
                                        } else {
                                            self.error = Some("Ошибка получения времени старта от сервера".to_string());
                                        }

                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(format!("{}", started_time))
                                                    .family(egui::FontFamily::Name("Inter".into()))
                                                    .size(18.0)
                                                    .color(theme::TEXT)
                                                    .variation("wght", 500.0)
                                            ).selectable(false)
                                        );
                                    });
                                });

                                ui.add_space(12.0);
                                
                                egui::Frame::new()
                                .corner_radius(8)
                                .fill(theme::CARD_BG)
                                .inner_margin(egui::Margin::ZERO)
                                .stroke(egui::Stroke::new(1.0, theme::BORDER_COLOR))

                                .show(ui, |ui| {
                                    ui.set_height(149.0);
                                    ui.set_width(ui.available_width());
                                    
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(53.5);
                                        
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("Прошло")
                                                    .family(egui::FontFamily::Name("Inter".into()))
                                                    .size(12.0)
                                                    .color(theme::TEXT_DIM)
                                            ).selectable(false)
                                        );
                                        ui.add_space(4.0);

                                        let mut passed_time = String::new();

                                        if let Some(Some(started_at)) = started_at {

                                            let current_time = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .map(|d| d.as_secs())
                                                .unwrap_or(0);

                                            passed_time = shared::format_duration(started_at, current_time);
                                        } else {
                                            self.error = Some("Ошибка получения времени старта от сервера".to_string());
                                        }

                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(format!("{}", passed_time))
                                                    .family(egui::FontFamily::Name("Inter".into()))
                                                    .size(18.0)
                                                    .color(theme::TEXT)
                                                    .variation("wght", 500.0)
                                            ).selectable(false)
                                        );
                                    });
                                });
                            });
                        });
                        
                        ui.add_space(16.0);

                        let response = add_button(
                            ui,
                            egui::vec2(ui.available_width(), 64.0),
                            theme::ACCENT,
                            theme::TEXT_ON_ACCENT,
                            "Завершить",
                            16.0
                        );

                        if response.clicked() {
                            self.post_action("finish");
                        }
                    });

                    if let Some(e) = self.error.clone() {
                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());

                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(e)
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(12.0)
                                            .color(theme::ERR)
                                    ).selectable(true)
                                );
                            });
                        });
                    };
                }

                Some(Status::ReviewRequired) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(
                            egui::Margin {
                                left: 32,
                                right: 32,
                                top: 32,
                                bottom: 0,
                            }
                        )
                    .show(ui, |ui| {
                        self.add_header(
                            ui, 
                            Some(theme::REVIEW), 
                            Some(theme::REVIEW_FG), 
                            Some(theme::REVIEW_DOT), 
                            Some("На проверке")
                        );

                        egui::Frame::new().show(ui, |ui| {
                            ui.set_height(ui.available_height() - 16.0 - 32.0);
                            ui.set_width(ui.available_width());

                            ui.vertical_centered(|ui| {
                                egui::Frame::new()
                                .inner_margin(egui::Margin::ZERO)

                                .show(ui, |ui| {
                                    ui.set_height(ui.available_height());
                                    ui.set_width(ui.available_width());
                                    
                                    ui.add_space(100.0);

                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new("Ожидайте результатов проверки")
                                                .family(egui::FontFamily::Name("Inter".into()))
                                                .size(24.0)
                                                .color(theme::TEXT)
                                                .variation("wght", 600.0)
                                        )
                                    );

                                    ui.add_space(116.0);
                                });
                            });
                        });
                    });

                    if let Some(e) = self.error.clone() {
                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());

                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(e)
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(12.0)
                                            .color(theme::ERR)
                                    ).selectable(true)
                                );
                            });
                        });
                    };
                }

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

                        self.add_header(
                            ui, 
                            Some(theme::DONE), 
                            Some(theme::DONE_FG), 
                            Some(theme::DONE_DOT), 
                            Some("Проверено")
                        );

                        ui.add_space(89.0);

                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(49.0);
                            ui.vertical_centered(|ui| {
                                egui::Frame::new().show(ui, |ui| {
                                    ui.set_width(196.0);

                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("Результат:")
                                                    .family(egui::FontFamily::Name("Inter".into()))
                                                    .color(theme::TEXT)
                                                    .size(24.0)
                                                    .variation("wght", 500.0)
                                            ).selectable(false)
                                        );
                                        ui.add_space(8.0);

                                        if let Some(Some(score)) = score {

                                            let score = shared::format_score(score);
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(score)
                                                        .family(egui::FontFamily::Name("Inter".into()))
                                                        .color(theme::TEXT)
                                                        .size(24.0)
                                                        .variation("wght", 700.0)
                                                ).selectable(false)
                                            );
                                        } else {
                                            self.error = Some("Ошибка получения результатов проверки с сервера".to_string());
                                        }
                                    });
                                });  
                            
                                egui::Frame::new().show(ui, |ui| {
                                    ui.set_width(83.0);

                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("Время:")
                                                    .family(egui::FontFamily::Name("Inter".into()))
                                                    .color(theme::TEXT_DIM)
                                                    .size(12.0)
                                                    .variation("wght", 400.0)
                                            ).selectable(false)
                                        );
                                        ui.add_space(4.0);

                                        if let (Some(Some(started_at)), Some(Some(finished_at))) = (started_at, finished_at) {
                                            let duration = shared::format_duration(started_at, finished_at);

                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(duration)
                                                        .family(egui::FontFamily::Name("Inter".into()))
                                                        .color(theme::TEXT_DIM)
                                                        .size(12.0)
                                                        .variation("wght", 500.0)
                                                ).selectable(false)
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

                    if let Some(e) = self.error.clone() {
                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());

                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(e)
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(12.0)
                                            .color(theme::ERR)
                                    ).selectable(true)
                                );
                            });
                        });
                    };
                }

                Some(Status::Banned) => {
                    egui::Frame::new()
                        .fill(theme::BG)
                        .corner_radius(14.0)
                        .inner_margin(egui::Margin{
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
                            Some("BAN")
                        );

                        ui.add_space(90.5);

                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new("ЗАБЛОКИРОВАН")
                                        .size(40.0)
                                        .family(egui::FontFamily::Name("Inter".into()))
                                        .color(theme::ERR)
                                        .variation("wght", 600.0)
                                ).selectable(false)
                            );
                        });

                        ui.add_space(90.5);
                    });

                    if let Some(e) = self.error.clone() {
                        egui::Frame::new().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());

                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(e)
                                            .family(egui::FontFamily::Name("Inter".into()))
                                            .size(12.0)
                                            .color(theme::ERR)
                                    ).selectable(true)
                                );
                            });
                        });
                    };
                }

                _ => {}
            }
        }
    }
}

fn add_button(ui: &mut egui::Ui, desired_size: egui::Vec2, bg: egui::Color32, fg: egui::Color32, text: &str, font_size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    ui.painter().rect_filled(rect, egui::CornerRadius::same(8), bg);

    // center text
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::new(font_size, egui::FontFamily::Name("Inter-SemiBold".into())),
        fg
    );
    response
}
use eframe::egui;
use shared::{CompState, ReportRequest, theme};
use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use ui::widgets;

mod post;
mod ui;

fn main() -> eframe::Result<()> {
    // create window structure
    let app = AdminApp::new();
    // window parameters (size, pos, icon, etc.)
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/Icon.png"))
        .expect("Icon must be valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 832.0])
            .with_resizable(false)
            .with_icon(icon),
        ..Default::default()
    };
    // start window
    eframe::run_native(
        "edque - admin",
        options,
        Box::new(|cc| {
            theme::add_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);

            Ok(Box::new(app))
        }),
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
    /// Creates http-client and AdminApp structure
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
    /// Send GET /api/comps with interval
    fn poll(&mut self) {
        if self.server.is_none() {
            return;
        }
        if let Some(t) = self.last_poll
            && t.elapsed() < Duration::from_secs(1)
        {
            return;
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
                    self.error = Some(format!(
                        "Ошибка обработки информации о компьютерах от сервера: {e}"
                    ));
                }
            },
            Err(_) => {
                self.error = Some("Ошибка подключения к серверу".to_string());
            }
        }
    }
    /// Saves selected computers to .CSV report
    /// Gets .CSV from server and saves it to `~/.local/share/edque` folder
    fn save_report(&mut self) {
        if self.server.is_none() {
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
        let resp =
            self.http.post(&url).json(&body).send().map_err(|e| {
                self.error = Some(format!("Ошибка при запросе отчёта с сервера: {e}"))
            });
        if let Ok(resp) = resp {
            let csv = resp.text().map_err(|e| {
                self.error = Some(format!(
                    "Ошибка при чтении отчёта, полученного с сервера: {e}"
                ))
            });
            if let Ok(csv) = csv {
                let timestamp = shared::format_time(
                    Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    ),
                    true,
                );
                let home = std::env::var("HOME").expect("HOME not set");
                let path = format!("{}/.local/share/edque/edque_report_{timestamp}.csv", home);
                let _ = std::fs::write(&path, csv).map_err(|e| {
                    self.error = Some(format!("Не удалось записать отчёт в файл: {e}"))
                });
                self.error = Some(format!("Отчёт успешно сохранён в {path}"));
            }
        }
    }
    /// Sets server address from buffer
    fn set_server(&mut self) {
        self.server = Some(self.server_input.clone());
    }
}

impl eframe::App for AdminApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().request_repaint_after(Duration::from_secs(1));
        // Main Frame
        egui::Frame::new()
            .fill(theme::BG)
            .corner_radius(egui::CornerRadius {
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
                    self.header(ui);
                    widgets::separator(ui);
                    // ----
                    // Body
                    // ----
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(14, 9))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height() - 48.0 - 32.0); // all - footer - error_line

                            ui.vertical_centered(|ui| {
                                for (idx, comp_state) in self.comps.clone().into_iter().enumerate()
                                {
                                    let comp_id = comp_state.comp_id;
                                    if idx > 0 {
                                        ui.add_space(10.0);
                                    }
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
                                shared::widgets::rich_label(ui, &e, 12, theme::ERR, 500);
                            });
                        };
                    });
                    widgets::separator(ui);
                    // ------
                    // Footer
                    // ------
                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;

                        self.footer(ui);
                    });
                });
            });
    }
}

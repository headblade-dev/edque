use std::{collections::HashMap, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use eframe::egui;
use shared::{CompState, ReportRequest, ScoreRequest, Status, format_time};

fn main() -> eframe::Result<()> {
    // create window structure
    let app = AdminApp::new();

    // window parameters (size, pos, icon, etc.)
    let options = eframe::NativeOptions::default();

    // start window
    eframe::run_native(
        "edque - admin",
        options,
        Box::new(|_cc| Ok(Box::new(app)))
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

enum Action {
    SetScore(u8, u16),
    Ban(u8),
    Resume(u8),
    Reset(u8),
    ToggleSelect(u8),
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
                    self.error = Some(format!("parse: {e}"));
                }
            }
            Err(e) => {
                self.error = Some(format!("get list: {e}"));
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
                        self.error = Some(format!("computer #{comp_id} not found"));
                    }
                }
                Err(e) => {
                    self.error = Some(format!("parse: {e}"));
                }
            }
            Err(e) => {
                self.error = Some(format!("score: {e}"));
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
                    self.error = Some(format!("computer #{comp_id} not found"));
                }
                Err(e) => {
                    self.error = Some(format!("parse: {e}"));
                }
            }
            Err(e) => {
                self.error = Some(format!("reset: {e}"));
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
                    self.error = Some(format!("computer #{comp_id} not found"));
                }
                Err(e) => {
                    self.error = Some(format!("parse: {e}"));
                }
            }
            Err(e) => {
                self.error = Some(format!("ban: {e}"));
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
            Err(e) => {
                self.error = Some(format!("resume: {e}"));
            }
        }
    }

    fn save_report(&mut self) -> Result<(), String> {
        if let None = self.server {
            self.error = Some("report: не подключен к серверу".to_string());
            return Err("не подключен к серверу".into());
        }

        if self.selected_comps.is_empty() {
            return Err("ничего не выбрано".into())
        }

        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/report", server);
        let body = ReportRequest {
            comp_ids: self.selected_comps.clone(),
        };

        let resp = self.http.post(&url).json(&body).send()
            .map_err(|e| format!("request: {e}"))?;

        let csv = resp.text()
            .map_err(|e| format!("read: {e}"))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let home = std::env::var("HOME").expect("HOME not set");

        let path = format!("{}/.local/share/edque/edque_report_{timestamp}.csv", home);

        std::fs::write(&path, csv)
            .map_err(|e| format!("write: {e}"))?;

        self.error = Some(format!("сохранено в {path}"));
        Ok(())
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
}

impl eframe::App for AdminApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().request_repaint_after(Duration::from_secs(1));
        
        ui.heading("edgui - Панель управления");
        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
        }
        ui.separator();

        let mut action: Option<Action> = None;

        let footer_height = 40.0;

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - footer_height)
            .show(ui, |ui| {
                for comp in &self.comps {
                    ui.horizontal(|ui| {
                        let start = format_time(comp.started_at);
                        let finish = format_time(comp.finished_at);

                        let mut checked: bool = self.selected_comps.contains(&comp.comp_id);

                        if ui.checkbox(&mut checked, "").changed() {
                            action = Some(Action::ToggleSelect(comp.comp_id));
                        }
                        
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(28.0, 28.0),
                            egui::Sense::hover()
                        );
                        ui.painter().circle_filled(rect.center(), 14.0, status_color(&comp.status));
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            status_icon(&comp.status),
                            egui::FontId::proportional(16.0),
                            status_icon_color(&comp.status)
                        );
                        ui.separator();
                        ui.label(format!("№{}", comp.comp_id));
                        ui.separator();

                        match comp.status {
                            Status::Offline => {
                                ui.label("Отключен");
                            }
                            Status::Idle => {
                                ui.label("Простаивает");
                                ui.separator();
                                
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Бан").clicked() {
                                        action = Some(Action::Ban(comp.comp_id));
                                    }
                                    ui.separator();
                                });
                            }
                            Status::Working => {
                                ui.label(format!("{}", start));
                                ui.separator();
                                ui.label(format!("{}", finish));
                                ui.separator();
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Бан").clicked() {
                                        action = Some(Action::Ban(comp.comp_id));
                                    }
                                    if ui.button("Сброс").clicked() {
                                        action = Some(Action::Reset(comp.comp_id));
                                    }
                                    ui.separator();
                                });
                            }
                            Status::ReviewRequired | Status::Done => {
                                ui.label(format!("{}", start));
                                ui.separator();
                                ui.label(format!("{}", finish));
                                ui.separator();

                                ui.label("Результат:");

                                let buf = self.score_bufs
                                    .entry(comp.comp_id)
                                    .or_insert_with(|| comp.score.map(shared::format_score).unwrap_or_default());

                                let response = ui.add (
                                    egui::TextEdit::singleline(buf).desired_width(60.0)
                                );
                                
                                if response.lost_focus() {
                                    println!("LOST FOCUS, buf = {:?}", buf);
                                    if let Some(score) = shared::parse_score(buf) {
                                        println!("PARSED = {}", score);
                                        action = Some(Action::SetScore(comp.comp_id, score));
                                    } else {
                                        println!("PARSE FAILED");
                                        *buf = comp.score.map(shared::format_score).unwrap_or_default();
                                    }
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Бан").clicked() {
                                        action = Some(Action::Ban(comp.comp_id));
                                    }
                                    if comp.status == Status::Done {
                                        if ui.button("Сброс").clicked() {
                                            action = Some(Action::Reset(comp.comp_id));
                                        }
                                    }
                                    if ui.button("Возобновить").clicked() {
                                        action = Some(Action::Resume(comp.comp_id));
                                    }
                                    ui.separator();
                                });
                            }
                            Status::Banned => {
                                ui.label("Заблокирован");
                                ui.separator();
                                
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Сброс").clicked() {
                                        action = Some(Action::Reset(comp.comp_id));
                                    }
                                    ui.separator();
                                });
                            }
                        }
                    });
                }
            }
        );
        ui.horizontal(|ui| {
            if ui.button("Сохранить отчет").clicked() {
                if let Err(e) = self.save_report() {
                    eprintln!("Failed to save CSV-report: {e}");
                }
            }
            ui.separator();

            ui.label("Сервер: ");
            ui.text_edit_singleline(&mut self.server_input);

            if ui.button("Применить").clicked() {
                self.set_server();
            }


        });

        if let Some(a) = action {
            match a {
                Action::Ban(id) => self.ban(id),
                Action::Reset(id) => self.reset(id),
                Action::Resume(id) => self.resume(id),
                Action::SetScore(id, score) => self.set_score(id, score),
                Action::ToggleSelect(id) => self.toggle_select(id),
            }
        } 
    }
}

fn status_color(status: &Status) -> egui::Color32 {
    match status {
        Status::Offline => egui::Color32::GRAY,
        Status::Idle => egui::Color32::WHITE,
        Status::Working => egui::Color32::DARK_BLUE,
        Status::ReviewRequired => egui::Color32::YELLOW,
        Status::Done => egui::Color32::DARK_GREEN,
        Status::Banned => egui::Color32::DARK_RED,
    }
}

fn status_icon(status: &Status) -> &'static str {
    match status {
        Status::Offline => "-",
        Status::Idle => "",
        Status::Working => "…",
        Status::ReviewRequired => "?",
        Status::Done => "OK",
        Status::Banned => "BAN",
    }
}

fn status_icon_color(status: &Status) -> egui::Color32 {
    match status {
        Status::Offline => egui::Color32::WHITE,
        Status::Idle => egui::Color32::BLACK,
        Status::Working => egui::Color32::WHITE,
        Status::ReviewRequired => egui::Color32::BLACK,
        Status::Done => egui::Color32::WHITE,
        Status::Banned => egui::Color32::WHITE,
    }
}

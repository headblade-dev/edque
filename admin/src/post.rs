use crate::AdminApp;
use shared::{CompState, ScoreRequest};

impl AdminApp {
    ///
    /// Sets score for given computer
    /// 
    pub fn set_score(&mut self, comp_id: u8, score: u16) {
        if let None = self.server {
            return;
        }
        let server = self.server.clone().unwrap();

        let url = format!("http://{}/api/comps/{}/score", server, comp_id);
        let request = ScoreRequest { score };

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
            },
            Err(_) => {
                self.error = Some(format!("Ошибка отправки результатов проверки"));
            }
        }
    }

    ///
    /// Resets values of given computer
    ///
    pub fn reset(&mut self, comp_id: u8) {
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
            },
            Err(_) => {
                self.error = Some(format!("Ошибка отправки запроса о сбросе компьютера"));
            }
        }

        self.score_bufs.remove(&comp_id);
    }

    ///
    /// Sends ban to given computer
    ///
    pub fn ban(&mut self, comp_id: u8) {
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
            },
            Err(_) => {
                self.error = Some(format!("Ошибка отправки запроса о блокировке компьютера"));
            }
        }

        self.score_bufs.remove(&comp_id);
    }

    ///
    /// Sets status of given computer to `Working` (if student want to resume his work)
    ///
    pub fn resume(&mut self, comp_id: u8) {
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
            },
            Err(_) => {
                self.error = Some(format!(
                    "Ошибка отправки запроса о возобновлении компьютера"
                ));
            }
        }
    }
}
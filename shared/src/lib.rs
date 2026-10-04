use serde::{Deserialize, Serialize};

pub mod theme;
pub mod widgets;

pub type Score = u16;

pub const MAX_SCORE: Score = 1000;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Idle,
    Working,
    Done,
    Offline,
    ReviewRequired,
    Banned,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CompState {
    pub comp_id: u8,
    pub status: Status,
    pub hostname: Option<String>,
    pub score: Option<Score>,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRequest {
    pub comp_id: u8,
    pub hostname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScoreRequest {
    pub score: Score,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReportRequest {
    pub comp_ids: Vec<u8>,
}

///
/// Formats score to `String` (245_u16 -> "24.5")
///
pub fn format_score(score: Score) -> String {
    format!("{}.{}", score / 10, score % 10)
}

pub fn parse_score(s: &str) -> Option<Score> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };

    let int: u16 = if int_part.is_empty() {
        0
    } else {
        int_part.parse().ok()?
    };

    let frac: u16 = match frac_part.len() {
        0 => 0,
        1 => frac_part.parse::<u16>().ok()?,
        _ => return None,
    };

    let total = int.checked_mul(10)?.checked_add(frac)?;
    if total > MAX_SCORE {
        return None;
    }
    Some(total)
}

///
/// Calculates duration from `start` to `end` and formats it to `String`
///
pub fn format_duration(start: u64, end: u64) -> String {
    let secs = end.saturating_sub(start);
    format!("{}:{:02}", secs / 60, secs % 60)
}

///
/// Formats time in seconds since Unix Epoch to `String`
///
pub fn format_time(secs: Option<u64>, print_secs: bool) -> String {
    let Some(t) = secs else {
        return "-".to_string();
    };

    let tz_offset = 4 * 3600;
    let local = t + tz_offset;

    let secs_in_day = local % 86400;
    let h = secs_in_day / 3600;
    let m = (secs_in_day % 3600) / 60;
    if print_secs {
        let s = (secs_in_day) % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", h, m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_works() {
        assert_eq!(format_score(875), "87.5");
        assert_eq!(format_score(870), "87.0");
        assert_eq!(format_score(1000), "100.0");
        assert_eq!(format_score(0), "0.0");
    }

    #[test]
    fn parse_works() {
        assert_eq!(parse_score("87.5"), Some(875));
        assert_eq!(parse_score("87"), Some(870));
        assert_eq!(parse_score("100"), Some(1000));
        assert_eq!(parse_score("0.5"), Some(5));
        assert_eq!(parse_score(".5"), Some(5));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert_eq!(parse_score("abc"), None);
        assert_eq!(parse_score("87.55"), None);
        assert_eq!(parse_score("101"), None);
        assert_eq!(parse_score("100.1"), None);
        assert_eq!(parse_score("-5"), None);
        assert_eq!(parse_score(""), None);
    }
}

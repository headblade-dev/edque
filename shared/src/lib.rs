use std::u16;

use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompState {
    pub comp_id: u8,
    pub status: Status,
    pub hostname: Option<String>,
    pub score: Option<Score>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRequest {
    pub comp_id: u8,
    pub hostname: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScoreRequest {
    pub score: Score,
}

pub fn format_score(score: Score) -> String {
    format!("{}.{}", score / 10, score % 10)
}

pub fn parse_score(s: &str) -> Option<Score> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (int_part, frac_part) = match s.split_once('.') {
        Some((i,f)) => (i, f),
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
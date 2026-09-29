mod storage;

use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, routing::{get, post},
};

use shared::{CompState, RegisterRequest, ScoreRequest, ReportRequest, Status, MAX_SCORE};

use tokio::sync::RwLock;

use std::{
    collections::HashMap, path::PathBuf, sync::{Arc}, time::{Duration, SystemTime, UNIX_EPOCH},
};

type Db = Arc<RwLock<HashMap<u8, CompState>>>;

const LISTEN_ON: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create folder: ~/.edque");

    let state_file = dir.join("state.json");
    let initial = storage::load(&state_file).unwrap_or_else(|e| {
        eprintln!("Failed to load saved state: {} - starting empty.", e);
        HashMap::new()
    });

    let db: Db = Arc::new(RwLock::new(initial));

    let db_for_save = db.clone();
    let state_file_for_save = state_file.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let snapshot = db_for_save.read().await;
            if let Err(e) = storage::save(&state_file_for_save, &snapshot) {
                eprintln!("Failed to save state: {}", e);
            }
        }
    });

    let app = Router::new()
        .route("/api/comps", get(list_comps))
        .route("/api/comps/{id}", get(get_comp))
        .route("/api/register", post(register))
        .route("/api/comps/{id}/start", post(start))
        .route("/api/comps/{id}/finish", post(finish))
        .route("/api/comps/{id}/score", post(set_score))
        .route("/api/comps/{id}/reset", post(reset))
        .route("/api/comps/{id}/ban", post(ban))
        .route("/api/comps/{id}/resume", post(resume))
        .route("/api/report", post(report))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(LISTEN_ON)
        .await
        .expect(&format!("Failed to bind to {}", LISTEN_ON));

    println!("Server listening on {}", LISTEN_ON);
    axum::serve(listener, app).await.expect("Server crashed");
}

pub fn data_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .expect("HOME is not set!");
    PathBuf::from(home).join(".local/share/edque")
}

async fn list_comps(State(db): State<Db>) -> Json<Vec<CompState>> {
    let db = db.read().await;
    let mut comps: Vec<CompState> = db.values().cloned().collect();
    comps.sort_by_key(|c| c.comp_id);
    Json(comps)
}

async fn get_comp(
    State(db): State<Db>,
    Path(comp_id): Path<u8>
) -> Result<Json<CompState>, StatusCode> {
    let db = db.read().await;
    db.get(&comp_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn register(
    State(db): State<Db>,
    Json(request): Json<RegisterRequest>
) -> Json<CompState> {
    let comp_id = request.comp_id;
    let hostname = request.hostname;

    let mut db = db.write().await;
    let entry = db.entry(comp_id).or_insert_with(|| CompState {
        comp_id: request.comp_id,
        status: Status::Idle,
        hostname: hostname.clone(),
        score: None,
        started_at: None,
        finished_at: None,
    });

    entry.hostname = hostname.clone();

    println!("POST /api/register comp_id={} hostname='{}'", comp_id, hostname.as_deref().unwrap_or("null"));

    Json(entry.clone())
}

async fn start(
    State(db): State<Db>,
    Path(comp_id): Path<u8>
) -> Result<Json<CompState>, StatusCode> {
    let mut db = db.write().await;
    let entry = db.get_mut(&comp_id).ok_or(StatusCode::NOT_FOUND)?;

    if entry.status == Status::Working {
        return Ok(Json(entry.clone()));
    }

    entry.status = Status::Working;
    entry.started_at = Some(now_ts());
    entry.finished_at = None;

    println!("POST /api/comp/{}/start", comp_id);

    Ok(Json(entry.clone()))
}

async fn finish(
    State(db): State<Db>,
    Path(comp_id): Path<u8>
) -> Result<Json<CompState>, StatusCode> {
    let mut db = db.write().await;
    let entry = db.get_mut(&comp_id).ok_or(StatusCode::NOT_FOUND)?;

    match entry.status {
        Status::Idle => return Err(StatusCode::BAD_REQUEST),
        Status::Done => return Err(StatusCode::CONFLICT),
        Status::ReviewRequired => return Ok(Json(entry.clone())),
        Status::Offline => return Err(StatusCode::BAD_REQUEST),
        _ => {}
    }

    entry.status = Status::ReviewRequired;
    entry.finished_at = Some(now_ts());

    println!("POST /api/comp/{}/finish", comp_id);

    Ok(Json(entry.clone()))
}

async fn set_score(
    State(db): State<Db>,
    Path(comp_id): Path<u8>,
    Json(request): Json<ScoreRequest>
) -> Result<Json<CompState>, StatusCode> {
    if request.score > MAX_SCORE {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let mut db = db.write().await;
    let entry = db.get_mut(&comp_id).ok_or(StatusCode::NOT_FOUND)?;

    entry.score = Some(request.score);
    entry.status = Status::Done;

    println!("POST /api/comp/{}/score score={}", comp_id, request.score);

    Ok(Json(entry.clone()))
}

async fn reset(
    State(db): State<Db>,
    Path(comp_id): Path<u8>,
) -> Result<Json<CompState>, StatusCode> {
    let mut db = db.write().await;
    let entry = db.get_mut(&comp_id).ok_or(StatusCode::NOT_FOUND)?;

    do_reset(entry);
    println!("POST /api/comp/{}/reset", comp_id);
    Ok(Json(entry.clone()))
}

async fn ban(
    State(db): State<Db>,
    Path(comp_id): Path<u8>,
) -> Result<Json<CompState>, StatusCode> {
    let mut db = db.write().await;
    let entry = db.get_mut(&comp_id).ok_or(StatusCode::NOT_FOUND)?;

    do_reset(entry);
    entry.status = Status::Banned;
    println!("POST /api/comp/{}/reset", comp_id);
    Ok(Json(entry.clone()))
}

async fn resume(
    State(db): State<Db>,
    Path(comp_id): Path<u8>,
) -> Result<Json<CompState>, StatusCode> {
    let mut db = db.write().await;
    let entry = db.get_mut(&comp_id).ok_or(StatusCode::NOT_FOUND)?;

    match entry.status {
        Status::Offline | Status::Idle | Status::Banned => return Err(StatusCode::BAD_REQUEST),
        Status::Working => return Ok(Json(entry.clone())),
        _ => ()
    }

    entry.status = Status::Working;
    println!("POST /api/comps/{}/resume", comp_id);
    Ok(Json(entry.clone()))
}

async fn report(
    State(db): State<Db>,
    Json(request): Json<ReportRequest>
) -> Result<String, StatusCode> {
    let db = db.read().await;
    let mut out = String::from("\u{FEFF}");
    out.push_str("ID,Длительность,Баллы\n");

    for comp_id in request.comp_ids {
        if let Some(entry) = db.get(&comp_id) {
            let duration = match (entry.started_at, entry.finished_at) {
                (Some(a), Some(b)) => {
                    shared::format_duration(a, b)
                }
                _ => String::new(),
            };

            let score = entry.score.map(shared::format_score).unwrap_or_default();

            out.push_str(&format!(
                "{},{},{}\n",
                entry.comp_id, duration, score
            ));
        }
    }

    Ok(out)
}

fn do_reset(entry: &mut CompState) {
    entry.status = Status::Idle;
    entry.started_at = None;
    entry.finished_at = None;
    entry.score = None;
}

fn now_ts() -> u64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs
}
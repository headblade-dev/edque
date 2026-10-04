**English** | [Русский](README.ru.md)
# edque

Virtual queue program for training the speed of completing demo exams.

## Libraries

- axum (REST API server)
- tokio (async, server only)
- serde and serde_json (state save/load from .json)
- eframe (UI)
- reqwest (HTTP requests)

## Features

- **Client:** "Start", "Finish", "In progress" screen with time, results screen, ban screen
- **Control panel:**
  - list of computers
  - colored statuses
  - start/finish time
  - score input field (applied on focus loss — click outside or Enter)
  - Reset (clears score and time)
  - Ban (client shows large red "BANNED" text, very effective)
  - Resume (for practice, in case a student wants to finish the work)
  - CSV report export (checkboxes for selecting specific computers)
- **Server endpoints:**
  - POST /api/register — register or reconnect client
  - GET /api/comps — array of computers and their data (id, hostname, start/finish time, scores)
  - GET /api/comps/:id — structure (object) of a specific computer
  - POST /api/comps/:id/start, finish, reset, ban, resume, score — state management
  - POST /api/report — CSV generation on the server

## Requirements

- Rust (stable) — install via [rustup](https://rustup.rs/)
- System libraries for eframe:

**Debian/Ubuntu**

```sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev```

**RHEL/Fedora:**

```sudo dnf install libxcb-devel libxkbcommon-devel openssl-devel```

## Build

```cargo build --release```

## Running

Binaries are built into `target/release/`:

- `edque-server` — server (can run on any machine accessible over the network)
- `edque-client` — run on student computers
- `edque-admin` — teacher's control panel

## Stored data

- **Server state:** `~/.local/share/edque/state.json` (on the machine running the server)
  Reset — by deleting the file.
- **Reports:** `~/.local/share/edque/report-TIMESTAMP.csv` (on the machine running admin)
  Keep in mind when fully uninstalling.
//! Tauri command layer — the UI's contract to the engine (ADR 0002).
//!
//! Holds the open Global Palbox in memory; every edit is in-memory until
//! `save_box`, which **backs up the original, then atomically writes**. Real
//! logic lives in `palbox_core`; this layer just marshals + owns the session.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use palbox_core::globalbox::{list_pals, pal_param_mut, read_pal_at, slot_count, PalSummary};
use palbox_core::pal::{self, PalDto};
use palbox_core::save::{read_sav, write_sav, PalSave};
use palbox_core::ue::Properties;
use serde::Serialize;
use tauri::State;

struct BoxSession {
    path: PathBuf,
    save: PalSave,
}

#[derive(Default)]
struct AppState(Mutex<Option<BoxSession>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResult {
    path: String,
    slot_count: usize,
    pals: Vec<PalSummary>,
}

/// Smoke test that the UI <-> core bridge is live.
#[tauri::command]
fn core_version() -> String {
    palbox_core::version().to_string()
}

/// Open a `GlobalPalStorage.sav`: decode, hold in memory, return the box tiles.
#[tauri::command]
fn open_box(path: String, state: State<AppState>) -> Result<OpenResult, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("read file: {e}"))?;
    let save = read_sav(&bytes)?;
    let pals = list_pals(&save);
    let slots = slot_count(&save).unwrap_or(0);
    *state.0.lock().unwrap() = Some(BoxSession { path: PathBuf::from(&path), save });
    Ok(OpenResult { path, slot_count: slots, pals })
}

/// Full editable DTO for the pal at `slot`.
#[tauri::command]
fn get_pal(slot: usize, state: State<AppState>) -> Result<PalDto, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("no box open")?;
    read_pal_at(&session.save, slot).ok_or_else(|| "no pal at slot".to_string())
}

/// Apply an edited DTO to the in-memory box; returns the freshly re-read DTO.
#[tauri::command]
fn update_pal(dto: PalDto, state: State<AppState>) -> Result<PalDto, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let sp = pal_param_mut(&mut session.save, dto.slot).ok_or("no pal at slot")?;
    apply_dto(sp, &dto);
    read_pal_at(&session.save, dto.slot).ok_or_else(|| "re-read failed".to_string())
}

/// Back up the original, then atomically write the edited box. Returns backup path.
#[tauri::command]
fn save_box(state: State<AppState>) -> Result<String, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("no box open")?;

    // 1) mandatory backup of the original before any write; failure aborts.
    let backup = backup_path(&session.path);
    if let Some(dir) = backup.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("backup dir: {e}"))?;
    }
    std::fs::copy(&session.path, &backup).map_err(|e| format!("backup failed: {e}"))?;

    // 2) atomic write: temp file, then rename over the target (no partial writes).
    let bytes = write_sav(&session.save)?;
    let tmp = session.path.with_extension("sav.tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("write temp: {e}"))?;
    std::fs::rename(&tmp, &session.path).map_err(|e| format!("atomic rename: {e}"))?;
    Ok(backup.to_string_lossy().into_owned())
}

/// Apply every implemented edit port from a DTO. (Work-suitability list setter
/// is the one field still pending — see pal.rs.)
fn apply_dto(sp: &mut Properties, dto: &PalDto) {
    pal::set_level(sp, dto.level);
    if let Some(name) = &dto.nickname {
        pal::set_nickname(sp, name);
    }
    pal::set_gender(sp, &dto.gender);
    pal::set_iv(sp, "hp", dto.ivs.hp);
    pal::set_iv(sp, "shot", dto.ivs.shot);
    pal::set_iv(sp, "defense", dto.ivs.defense);
    pal::set_soul(sp, "hp", dto.souls.hp);
    pal::set_soul(sp, "attack", dto.souls.attack);
    pal::set_soul(sp, "defense", dto.souls.defense);
    pal::set_soul(sp, "craftSpeed", dto.souls.craft_speed);
    pal::set_condensation(sp, dto.condensation);
    pal::set_lucky(sp, dto.is_lucky);
    pal::set_passives(sp, dto.passives.clone());
    pal::set_equipped_moves(sp, dto.equipped_moves.clone());
}

fn backup_path(original: &Path) -> PathBuf {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let stem = original.file_stem().and_then(|s| s.to_str()).unwrap_or("GlobalPalStorage");
    original
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("PalboxStudio-backups")
        .join(format!("{stem}.{secs}.bak"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            core_version,
            open_box,
            get_pal,
            update_pal,
            save_box
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

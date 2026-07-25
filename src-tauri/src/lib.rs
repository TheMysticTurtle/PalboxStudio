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
use palbox_core::reference::{
    PassiveOption, PassivePreset, ReferenceBundle, ReferenceDatabase, UserDatabase,
};
use palbox_core::save::{read_sav, write_sav, PalSave};
use palbox_core::ue::Properties;
use serde::Serialize;
use tauri::{path::BaseDirectory, Manager, State};

struct BoxSession {
    path: PathBuf,
    save: PalSave,
}

#[derive(Default)]
struct AppState(Mutex<Option<BoxSession>>);

struct DatabasePaths {
    reference: PathBuf,
    user: PathBuf,
}

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

/// Static 1.0 passive choices for the preset picker.
#[tauri::command]
fn list_passive_options(
    search: Option<String>,
    include_disabled: Option<bool>,
    include_unavailable: Option<bool>,
    databases: State<DatabasePaths>,
) -> Result<Vec<PassiveOption>, String> {
    ReferenceDatabase::open(&databases.reference)
        .and_then(|reference| {
            reference.list_passives(
                search.as_deref().unwrap_or(""),
                include_disabled.unwrap_or(false),
                include_unavailable.unwrap_or(false),
            )
        })
        .map_err(|error| error.to_string())
}

/// The normalized reference tables shaped for the existing UI model.
#[tauri::command]
fn get_reference_data(databases: State<DatabasePaths>) -> Result<ReferenceBundle, String> {
    ReferenceDatabase::open(&databases.reference)
        .and_then(|reference| reference.load_ui_bundle())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_passive_presets(databases: State<DatabasePaths>) -> Result<Vec<PassivePreset>, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.list_presets())
        .map_err(|error| error.to_string())
}

/// Create or replace a named preset. The core validates every passive code
/// against the 1.0 reference DB and enforces the four-slot limit.
#[tauri::command]
fn save_passive_preset(
    id: Option<i64>,
    name: String,
    passive_codes: Vec<String>,
    databases: State<DatabasePaths>,
) -> Result<PassivePreset, String> {
    let reference =
        ReferenceDatabase::open(&databases.reference).map_err(|error| error.to_string())?;
    let mut user =
        UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    user.save_preset(&reference, id, &name, &passive_codes)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_passive_preset(id: i64, databases: State<DatabasePaths>) -> Result<bool, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.delete_preset(id))
        .map_err(|error| error.to_string())
}

/// Apply a preset only to the in-memory Pal currently addressed by `slot`.
/// The preset DB never stores or mirrors that Pal's mutable save values.
#[tauri::command]
fn apply_passive_preset(
    slot: usize,
    preset_id: i64,
    box_state: State<AppState>,
    databases: State<DatabasePaths>,
) -> Result<PalDto, String> {
    let reference =
        ReferenceDatabase::open(&databases.reference).map_err(|error| error.to_string())?;
    let user = UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    let preset = user
        .get_preset(preset_id)
        .map_err(|error| error.to_string())?;
    reference
        .validate_passive_codes(&preset.passive_codes)
        .map_err(|error| error.to_string())?;

    let mut guard = box_state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let sp = pal_param_mut(&mut session.save, slot).ok_or("no pal at slot")?;
    pal::set_passives(sp, preset.passive_codes);
    read_pal_at(&session.save, slot).ok_or_else(|| "re-read failed".to_string())
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

fn database_paths(app: &tauri::App) -> Result<DatabasePaths, Box<dyn std::error::Error>> {
    let reference = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/palbox-reference.db")
    } else {
        app.path()
            .resolve("data/palbox-reference.db", BaseDirectory::Resource)?
    };
    if !reference.is_file() {
        return Err(format!("reference database is missing: {}", reference.display()).into());
    }
    // The user DB is intentionally outside the application bundle so presets
    // survive upgrades. UserDatabase initializes it from schema v1 on first use.
    let user = app
        .path()
        .resolve("palbox-user.db", BaseDirectory::AppData)?;
    Ok(DatabasePaths { reference, user })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            let paths = database_paths(app)?;
            app.manage(paths);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            core_version,
            open_box,
            get_pal,
            update_pal,
            save_box,
            get_reference_data,
            list_passive_options,
            list_passive_presets,
            save_passive_preset,
            delete_passive_preset,
            apply_passive_preset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

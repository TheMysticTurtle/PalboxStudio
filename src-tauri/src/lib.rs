//! Tauri command layer — the UI's contract to the engine (ADR 0002).
//!
//! Holds the open Global Palbox in memory; every edit is in-memory until
//! `save_box`, which **backs up the original, then atomically writes**. Real
//! logic lives in `palbox_core`; this layer just marshals + owns the session.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use palbox_core::globalbox::{
    add_initialized_pal, apply_pal_input_at, clone_pal, delete_pal, list_pal_views,
    read_pal_view_at, slot_count, PalSummaryView,
};
use palbox_core::projection::{PalInput, PalView};
use palbox_core::reference::{
    validate_passive_codes, AppPreferences, PalGroupMembership, PassiveOption, PassivePreset,
    ReferenceBundle, ReferenceCatalog, ReferenceDatabase, UserDatabase, UserGroup,
};
use palbox_core::session::SaveSession;
use serde::Serialize;
use tauri::{path::BaseDirectory, Manager, State};

#[derive(Default)]
struct AppState(Mutex<Option<SaveSession>>);

struct DatabasePaths {
    reference: PathBuf,
    user: PathBuf,
}

/// The read-only reference materialized into memory once at startup, so reference
/// commands never re-open the bundled 17 MB DB.
struct ReferenceCache {
    catalog: ReferenceCatalog,
    passive_options: Vec<PassiveOption>,
    passive_codes: HashSet<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResult {
    path: String,
    slot_count: usize,
    pals: Vec<PalSummaryView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoxSessionStatus {
    dirty: bool,
    source_state: &'static str,
    detail: Option<String>,
}

/// Smoke test that the UI <-> core bridge is live.
#[tauri::command]
fn core_version() -> String {
    palbox_core::version().to_string()
}

/// Open a `GlobalPalStorage.sav`: decode, hold in memory, return the box tiles.
#[tauri::command]
fn open_box(
    path: String,
    state: State<AppState>,
    cache: State<ReferenceCache>,
) -> Result<OpenResult, String> {
    let session = SaveSession::open(&path)?;
    let pals = list_pal_views(session.save(), &cache.catalog);
    let slots = slot_count(session.save()).unwrap_or(0);
    *state.0.lock().unwrap() = Some(session);
    Ok(OpenResult {
        path,
        slot_count: slots,
        pals,
    })
}

/// Full editable DTO for the pal at `slot`.
#[tauri::command]
fn get_pal(
    slot: usize,
    state: State<AppState>,
    cache: State<ReferenceCache>,
) -> Result<PalView, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("no box open")?;
    read_pal_view_at(session.save(), slot, &cache.catalog)
}

/// Lightweight source monitor for the UI. The fresh content hash is
/// authoritative; a watcher/poll result can only warn or block early.
#[tauri::command]
fn box_session_status(state: State<AppState>) -> Result<BoxSessionStatus, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("no box open")?;
    let (source_state, detail) = match session.source_is_current() {
        Ok(true) => ("unchanged", None),
        Ok(false) => (
            "changed",
            Some("The Global Palbox on disk no longer matches the opened copy.".to_string()),
        ),
        Err(error) => ("unavailable", Some(error)),
    };
    Ok(BoxSessionStatus {
        dirty: session.is_dirty(),
        source_state,
        detail,
    })
}

/// Apply semantic input through the headless engine and return its fresh view.
#[tauri::command]
fn update_pal(
    dto: PalInput,
    state: State<AppState>,
    cache: State<ReferenceCache>,
) -> Result<PalView, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    apply_pal_input_at(session.save_mut(), &dto, &cache.catalog)
}

/// Result of a box add/clone/delete: the refreshed tiles and the slot the UI
/// should select+reveal (the new pal for add/clone; `None` after a delete).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoxMutation {
    pals: Vec<PalSummaryView>,
    slot: Option<usize>,
}

/// Add a brand-new pal (default: the turtle CubeTurtle) to a free box slot.
#[tauri::command]
fn add_box_pal(
    species: Option<String>,
    state: State<AppState>,
    cache: State<ReferenceCache>,
) -> Result<BoxMutation, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let species = species.unwrap_or_else(|| "CubeTurtle".to_string());
    let slot = add_initialized_pal(session.save_mut(), &species, &cache.catalog)?;
    Ok(BoxMutation {
        pals: list_pal_views(session.save(), &cache.catalog),
        slot: Some(slot),
    })
}

/// Deep-copy the pal at `slot` into a free slot with a fresh identity.
#[tauri::command]
fn clone_box_pal(
    slot: usize,
    state: State<AppState>,
    cache: State<ReferenceCache>,
) -> Result<BoxMutation, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let new_slot = clone_pal(session.save_mut(), slot)?;
    Ok(BoxMutation {
        pals: list_pal_views(session.save(), &cache.catalog),
        slot: Some(new_slot),
    })
}

/// Remove the pal at `slot`, restoring the slot to a vacancy.
#[tauri::command]
fn delete_box_pal(
    slot: usize,
    state: State<AppState>,
    cache: State<ReferenceCache>,
) -> Result<BoxMutation, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    delete_pal(session.save_mut(), slot)?;
    Ok(BoxMutation {
        pals: list_pal_views(session.save(), &cache.catalog),
        slot: None,
    })
}

/// Back up the original, then atomically write the edited box. Returns backup path.
#[tauri::command]
fn save_box(state: State<AppState>) -> Result<String, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let backup = session.persist()?;
    Ok(backup.to_string_lossy().into_owned())
}

/// Static 1.0 passive choices for the preset picker.
#[tauri::command]
fn list_passive_options(
    search: Option<String>,
    include_disabled: Option<bool>,
    include_unavailable: Option<bool>,
    cache: State<ReferenceCache>,
) -> Vec<PassiveOption> {
    let query = search.unwrap_or_default().trim().to_lowercase();
    let include_disabled = include_disabled.unwrap_or(false);
    let include_unavailable = include_unavailable.unwrap_or(false);
    cache
        .passive_options
        .iter()
        .filter(|option| {
            (include_disabled || !option.disabled)
                && (include_unavailable || option.available_normal_pal)
                && (query.is_empty()
                    || option.name.to_lowercase().contains(&query)
                    || option.code.to_lowercase().contains(&query)
                    || option.description.to_lowercase().contains(&query))
        })
        .cloned()
        .collect()
}

/// The normalized reference tables shaped for the existing UI model.
#[tauri::command]
fn get_reference_data(cache: State<ReferenceCache>) -> ReferenceBundle {
    cache.catalog.bundle().clone()
}

#[tauri::command]
fn get_app_preferences(databases: State<DatabasePaths>) -> Result<AppPreferences, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.app_preferences())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_app_preferences(
    preferences: AppPreferences,
    databases: State<DatabasePaths>,
) -> Result<AppPreferences, String> {
    let mut user =
        UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    user.save_app_preferences(&preferences)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_passive_presets(databases: State<DatabasePaths>) -> Result<Vec<PassivePreset>, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.list_presets())
        .map_err(|error| error.to_string())
}

/// Create or replace a named preset. The core validates every passive code
/// against the reference DB and enforces its DB-backed slot limit.
#[tauri::command]
fn save_passive_preset(
    id: Option<i64>,
    name: String,
    passive_codes: Vec<String>,
    cache: State<ReferenceCache>,
    databases: State<DatabasePaths>,
) -> Result<PassivePreset, String> {
    let mut user =
        UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    user.save_preset(
        &cache.passive_codes,
        cache.catalog.bundle().limits.passives_max as usize,
        id,
        &name,
        &passive_codes,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_passive_preset(id: i64, databases: State<DatabasePaths>) -> Result<bool, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.delete_preset(id))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_groups(databases: State<DatabasePaths>) -> Result<Vec<UserGroup>, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.list_groups())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_group(name: String, databases: State<DatabasePaths>) -> Result<UserGroup, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.create_group(&name))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn rename_group(
    id: i64,
    name: String,
    databases: State<DatabasePaths>,
) -> Result<UserGroup, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.rename_group(id, &name))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_group(id: i64, databases: State<DatabasePaths>) -> Result<bool, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.delete_group(id))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_group_memberships(
    databases: State<DatabasePaths>,
) -> Result<Vec<PalGroupMembership>, String> {
    UserDatabase::open_or_create(&databases.user)
        .and_then(|user| user.list_group_memberships())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_pal_groups(
    instance_id: String,
    group_ids: Vec<i64>,
    databases: State<DatabasePaths>,
) -> Result<Vec<i64>, String> {
    let mut user =
        UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    user.set_pal_groups(&instance_id, &group_ids)
        .map_err(|error| error.to_string())
}

/// Apply a preset only to the in-memory Pal currently addressed by `slot`.
/// The preset DB never stores or mirrors that Pal's mutable save values.
#[tauri::command]
fn apply_passive_preset(
    slot: usize,
    preset_id: i64,
    box_state: State<AppState>,
    cache: State<ReferenceCache>,
    databases: State<DatabasePaths>,
) -> Result<PalView, String> {
    let user = UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    let preset = user
        .get_preset(preset_id)
        .map_err(|error| error.to_string())?;
    validate_passive_codes(
        &preset.passive_codes,
        &cache.passive_codes,
        cache.catalog.bundle().limits.passives_max as usize,
    )
    .map_err(|error| error.to_string())?;

    let mut guard = box_state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let mut input = read_pal_view_at(session.save(), slot, &cache.catalog)?.editable;
    input.passives = preset.passive_codes;
    apply_pal_input_at(session.save_mut(), &input, &cache.catalog)
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
        .plugin(tauri_plugin_os::init())
        .manage(AppState::default())
        .setup(|app| {
            let paths = database_paths(app)?;
            // Materialize the read-only reference DB into memory once; commands
            // then serve from RAM instead of re-opening the bundled DB per call.
            let reference = ReferenceDatabase::open(&paths.reference)?;
            let cache = ReferenceCache {
                catalog: reference.load_catalog()?,
                passive_options: reference.list_passives("", true, true)?,
                passive_codes: reference.passive_code_set()?,
            };
            drop(reference);
            app.manage(cache);
            app.manage(paths);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            core_version,
            open_box,
            get_pal,
            box_session_status,
            update_pal,
            add_box_pal,
            clone_box_pal,
            delete_box_pal,
            save_box,
            get_reference_data,
            get_app_preferences,
            save_app_preferences,
            list_passive_options,
            list_passive_presets,
            save_passive_preset,
            delete_passive_preset,
            apply_passive_preset,
            list_groups,
            create_group,
            rename_group,
            delete_group,
            list_group_memberships,
            set_pal_groups
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

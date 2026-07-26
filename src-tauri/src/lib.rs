//! Tauri command layer — the UI's contract to the engine (ADR 0002).
//!
//! Holds the open Global Palbox in memory; every edit is in-memory until
//! `save_box`, which **backs up the original, then atomically writes**. Real
//! logic lives in `palbox_core`; this layer just marshals + owns the session.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use palbox_core::globalbox::{
    add_pal, clone_pal, delete_pal, list_pals, pal_param_mut, read_pal_at, slot_count, PalSummary,
};
use palbox_core::pal::{self, PalDto};
use palbox_core::reference::{
    validate_passive_codes, PalGroupMembership, PassiveOption, PassivePreset, ReferenceBundle,
    ReferenceDatabase, UserDatabase, UserGroup,
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

/// The read-only reference materialized into memory once at startup, so reference
/// commands never re-open the bundled 17 MB DB.
struct ReferenceCache {
    bundle: ReferenceBundle,
    passive_options: Vec<PassiveOption>,
    passive_codes: HashSet<String>,
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
    *state.0.lock().unwrap() = Some(BoxSession {
        path: PathBuf::from(&path),
        save,
    });
    Ok(OpenResult {
        path,
        slot_count: slots,
        pals,
    })
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

/// Result of a box add/clone/delete: the refreshed tiles and the slot the UI
/// should select+reveal (the new pal for add/clone; `None` after a delete).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoxMutation {
    pals: Vec<PalSummary>,
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
    let slot = add_pal(&mut session.save, &species)?;
    let sp = pal_param_mut(&mut session.save, slot).ok_or("new pal has no SaveParameter")?;
    // Resolve healthy defaults from the startup cache; never query SQLite per
    // Pal. Level 1, zero IVs/souls/condensation uses the game's base HP formula.
    let base_code = species.strip_prefix("BOSS_").unwrap_or(&species);
    let species_ref = cache
        .bundle
        .species
        .iter()
        .find(|value| value.code == base_code);
    let hp_scaling = species_ref.map(|value| value.scaling.hp).unwrap_or(80) as f64;
    let alpha_rate = if species.to_uppercase().starts_with("BOSS_") {
        1.2
    } else {
        1.0
    };
    let full_hp = (500.0 + 5.0 + hp_scaling * 0.5 * alpha_rate).floor() as i64 * 1000;
    let full_food = species_ref
        .map(|value| value.max_stomach)
        .filter(|value| *value > 0)
        .unwrap_or(300) as f32;
    pal::initialize_new_pal(sp, full_hp, full_food);
    Ok(BoxMutation {
        pals: list_pals(&session.save),
        slot: Some(slot),
    })
}

/// Deep-copy the pal at `slot` into a free slot with a fresh identity.
#[tauri::command]
fn clone_box_pal(slot: usize, state: State<AppState>) -> Result<BoxMutation, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let new_slot = clone_pal(&mut session.save, slot)?;
    Ok(BoxMutation {
        pals: list_pals(&session.save),
        slot: Some(new_slot),
    })
}

/// Remove the pal at `slot`, restoring the slot to a vacancy.
#[tauri::command]
fn delete_box_pal(slot: usize, state: State<AppState>) -> Result<BoxMutation, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    delete_pal(&mut session.save, slot)?;
    Ok(BoxMutation {
        pals: list_pals(&session.save),
        slot: None,
    })
}

/// Back up the original, then atomically write the edited box. Returns backup path.
#[tauri::command]
fn save_box(state: State<AppState>) -> Result<String, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("no box open")?;

    // Encode and validate the edited payload before touching the source file.
    let bytes = write_sav(&session.save)?;
    read_sav(&bytes).map_err(|error| format!("refusing invalid encoded save: {error}"))?;

    // A byte-verified, uniquely named backup is mandatory before every write.
    // Any backup failure aborts while the original is still untouched.
    let backup = create_verified_backup(&session.path)?;

    // Write and sync a sibling temp file, verify it byte-for-byte and by
    // decoding, then atomically replace the original.
    let tmp = session.path.with_extension("sav.palboxstudio.tmp");
    let write_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)
            .map_err(|error| format!("open temp save: {error}"))?;
        file.write_all(&bytes)
            .map_err(|error| format!("write temp save: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("sync temp save: {error}"))?;
        std::fs::set_permissions(
            &tmp,
            std::fs::metadata(&session.path)
                .map_err(|error| format!("read original permissions: {error}"))?
                .permissions(),
        )
        .map_err(|error| format!("set temp permissions: {error}"))?;
        let staged = std::fs::read(&tmp).map_err(|error| format!("verify temp save: {error}"))?;
        if staged != bytes {
            return Err("temp save verification failed: bytes differ after write".to_string());
        }
        read_sav(&staged).map_err(|error| format!("temp save failed to decode: {error}"))?;
        std::fs::rename(&tmp, &session.path)
            .map_err(|error| format!("atomic save replacement failed: {error}"))
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    write_result?;
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
    cache.bundle.clone()
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
    cache: State<ReferenceCache>,
    databases: State<DatabasePaths>,
) -> Result<PassivePreset, String> {
    let mut user =
        UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    user.save_preset(&cache.passive_codes, id, &name, &passive_codes)
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
) -> Result<PalDto, String> {
    let user = UserDatabase::open_or_create(&databases.user).map_err(|error| error.to_string())?;
    let preset = user
        .get_preset(preset_id)
        .map_err(|error| error.to_string())?;
    validate_passive_codes(&preset.passive_codes, &cache.passive_codes)
        .map_err(|error| error.to_string())?;

    let mut guard = box_state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("no box open")?;
    let sp = pal_param_mut(&mut session.save, slot).ok_or("no pal at slot")?;
    pal::set_passives(sp, preset.passive_codes);
    read_pal_at(&session.save, slot).ok_or_else(|| "re-read failed".to_string())
}

/// Apply every implemented edit port from a DTO.
fn apply_dto(sp: &mut Properties, dto: &PalDto) {
    // Species first: the game derives stats/work/learnset from CharacterID.
    // Variant second so Alpha/Lucky can add or remove the BOSS_ representation.
    pal::set_species(sp, &dto.character_id);
    pal::set_variant(sp, dto.is_alpha, dto.is_lucky);
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
    pal::set_work(sp, &dto.work);
    pal::set_passives(sp, dto.passives.clone());
    pal::set_equipped_moves(sp, dto.equipped_moves.clone());
    pal::set_learned_moves(sp, dto.learned_moves.clone());
    pal::set_hp(sp, dto.hp);
    pal::set_sanity(sp, dto.sanity);
    pal::set_food(sp, dto.food);
    pal::set_friendship(sp, dto.friendship);
}

fn backup_path(original: &Path, timestamp_millis: u128, collision: usize) -> PathBuf {
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("GlobalPalStorage");
    let suffix = if collision == 0 {
        timestamp_millis.to_string()
    } else {
        format!("{timestamp_millis}-{collision}")
    };
    original
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("PalboxStudio-backups")
        .join(format!("{stem}.{suffix}.bak"))
}

fn files_match(left: &Path, right: &Path) -> std::io::Result<bool> {
    if std::fs::metadata(left)?.len() != std::fs::metadata(right)?.len() {
        return Ok(false);
    }
    let mut left = File::open(left)?;
    let mut right = File::open(right)?;
    let mut left_buffer = [0u8; 64 * 1024];
    let mut right_buffer = [0u8; 64 * 1024];
    loop {
        let left_count = left.read(&mut left_buffer)?;
        let right_count = right.read(&mut right_buffer)?;
        if left_count != right_count || left_buffer[..left_count] != right_buffer[..right_count] {
            return Ok(false);
        }
        if left_count == 0 {
            return Ok(true);
        }
    }
}

fn create_verified_backup(original: &Path) -> Result<PathBuf, String> {
    let metadata = std::fs::metadata(original)
        .map_err(|error| format!("read original for backup: {error}"))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err("refusing to back up a missing or empty Global Palbox".to_string());
    }
    let directory = original
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("PalboxStudio-backups");
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("create backup directory: {error}"))?;

    let timestamp_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("backup clock error: {error}"))?
        .as_millis();
    for collision in 0..1000 {
        let backup = backup_path(original, timestamp_millis, collision);
        let destination = match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&backup)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("create backup: {error}")),
        };
        let backup_result = (|| -> Result<(), String> {
            let mut source =
                File::open(original).map_err(|error| format!("open original: {error}"))?;
            let mut destination = destination;
            let copied = std::io::copy(&mut source, &mut destination)
                .map_err(|error| format!("copy backup: {error}"))?;
            destination
                .sync_all()
                .map_err(|error| format!("sync backup: {error}"))?;
            drop(destination);

            let verified = copied == metadata.len()
                && files_match(original, &backup)
                    .map_err(|error| format!("verify backup: {error}"))?;
            if !verified {
                return Err("backup verification failed; original was not modified".to_string());
            }
            Ok(())
        })();
        if let Err(error) = backup_result {
            let _ = std::fs::remove_file(&backup);
            return Err(error);
        }
        return Ok(backup);
    }
    Err("could not allocate a unique backup filename".to_string())
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
            // Materialize the read-only reference DB into memory once; commands
            // then serve from RAM instead of re-opening the bundled DB per call.
            let reference = ReferenceDatabase::open(&paths.reference)?;
            let cache = ReferenceCache {
                bundle: reference.load_ui_bundle()?,
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
            update_pal,
            add_box_pal,
            clone_box_pal,
            delete_box_pal,
            save_box,
            get_reference_data,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "palbox-studio-backup-test-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn verified_backups_are_exact_and_never_overwrite() {
        let root = unique_test_directory();
        std::fs::create_dir(&root).unwrap();
        let original = root.join("GlobalPalStorage.sav");
        let payload = b"representative-global-palbox-bytes";
        std::fs::write(&original, payload).unwrap();

        let first = create_verified_backup(&original).unwrap();
        let second = create_verified_backup(&original).unwrap();
        assert_ne!(first, second);
        assert_eq!(std::fs::read(&first).unwrap(), payload);
        assert_eq!(std::fs::read(&second).unwrap(), payload);
        assert_eq!(
            first.parent().unwrap().file_name().unwrap(),
            "PalboxStudio-backups"
        );

        std::fs::remove_file(first).unwrap();
        std::fs::remove_file(second).unwrap();
        std::fs::remove_dir(root.join("PalboxStudio-backups")).unwrap();
        std::fs::remove_file(original).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}

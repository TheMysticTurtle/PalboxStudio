//! Authoritative open-box session and crash-safe file lifecycle.
//!
//! The desktop shell owns dialogs and marshals commands; this module owns what
//! it means to open, mutate, and safely persist a Global Palbox. In particular,
//! a session refuses to overwrite a source file whose content changed after it
//! was opened.

use crate::save::{read_sav, write_sav, PalSave};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFingerprint {
    pub size: u64,
    pub modified_nanos: Option<u128>,
    pub sha256: [u8; 32],
}

impl SourceFingerprint {
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|error| format!("read source file: {error}"))?;
        Self::from_bytes(path, &bytes)
    }

    fn from_bytes(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("refusing an empty Global Palbox".to_string());
        }
        let metadata =
            std::fs::metadata(path).map_err(|error| format!("read source metadata: {error}"))?;
        if !metadata.is_file() {
            return Err("Global Palbox path is not a file".to_string());
        }
        let modified_nanos = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_nanos());
        let sha256 = Sha256::digest(&bytes).into();
        Ok(Self {
            size: bytes.len() as u64,
            modified_nanos,
            sha256,
        })
    }
}

/// One open Global Palbox and the source identity it is safe to replace.
pub struct SaveSession {
    path: PathBuf,
    save: PalSave,
    source: SourceFingerprint,
    dirty: bool,
}

impl SaveSession {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let bytes = std::fs::read(&path).map_err(|error| format!("read file: {error}"))?;
        // The fingerprint must describe the exact bytes we parsed. A second
        // read here could accidentally bless a write that raced with open.
        let source = SourceFingerprint::from_bytes(&path, &bytes)?;
        let save = read_sav(&bytes)?;
        Ok(Self {
            path,
            save,
            source,
            dirty: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self) -> &PalSave {
        &self.save
    }

    /// Mutable save access is an engine mutation and marks the session dirty.
    pub fn save_mut(&mut self) -> &mut PalSave {
        self.dirty = true;
        &mut self.save
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn verify_source_unchanged(&self, phase: &str) -> Result<(), String> {
        let current = SourceFingerprint::read(&self.path)?;
        if current != self.source {
            return Err(format!(
                "refusing to overwrite a Global Palbox that changed externally {phase}; reopen it and reapply the edit"
            ));
        }
        Ok(())
    }

    /// Encode, verify, back up, stage, recheck, and atomically replace the
    /// source. Returns the unique verified backup path.
    pub fn persist(&mut self) -> Result<PathBuf, String> {
        // Encoding happens before any filesystem mutation. A schema or codec
        // failure therefore cannot create a backup or touch the source.
        let bytes = write_sav(&mut self.save)?;
        read_sav(&bytes).map_err(|error| format!("refusing invalid encoded save: {error}"))?;

        self.verify_source_unchanged("since it was opened")?;
        let backup = create_verified_backup(&self.path)?;
        let temporary = temporary_path(&self.path)?;

        let write_result = (|| -> Result<(), String> {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary)
                .map_err(|error| format!("create temp save: {error}"))?;
            file.write_all(&bytes)
                .map_err(|error| format!("write temp save: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("sync temp save: {error}"))?;
            std::fs::set_permissions(
                &temporary,
                std::fs::metadata(&self.path)
                    .map_err(|error| format!("read original permissions: {error}"))?
                    .permissions(),
            )
            .map_err(|error| format!("set temp permissions: {error}"))?;

            let staged =
                std::fs::read(&temporary).map_err(|error| format!("verify temp save: {error}"))?;
            if staged != bytes {
                return Err("temp save verification failed: bytes differ after write".to_string());
            }
            read_sav(&staged).map_err(|error| format!("temp save failed to decode: {error}"))?;

            // Palworld or another editor may have written after our first
            // recheck while the backup/temp file was being produced.
            self.verify_source_unchanged("while the replacement was staged")?;
            std::fs::rename(&temporary, &self.path)
                .map_err(|error| format!("atomic save replacement failed: {error}"))
        })();
        if write_result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        write_result?;

        self.source = SourceFingerprint::read(&self.path)?;
        self.dirty = false;
        Ok(backup)
    }
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

fn temporary_path(original: &Path) -> Result<PathBuf, String> {
    let directory = original.parent().unwrap_or_else(|| Path::new("."));
    let name = original
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("GlobalPalStorage.sav");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("temp-file clock error: {error}"))?
        .as_nanos();
    Ok(directory.join(format!(
        ".{name}.palboxstudio-{}-{nonce}.tmp",
        std::process::id()
    )))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::globalbox::{list_pals, pal_param_mut, read_pal_at};
    use crate::ue;

    fn unique_test_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "palbox-studio-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn copied_fixture(label: &str) -> (PathBuf, PathBuf) {
        let root = unique_test_directory(label);
        std::fs::create_dir(&root).unwrap();
        let path = root.join("GlobalPalStorage.sav");
        std::fs::copy(crate::save::test_fixture_path(), &path).unwrap();
        (root, path)
    }

    fn remove_test_tree(root: &Path) {
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn persistence_creates_verified_backup_and_clears_dirty_state() {
        let (root, path) = copied_fixture("persist");
        let original = std::fs::read(&path).unwrap();
        let mut session = SaveSession::open(&path).unwrap();
        let slot = list_pals(session.save())[0].slot;
        crate::pal::set_level(
            pal_param_mut(session.save_mut(), slot).expect("fixture Pal"),
            80,
        );
        assert!(session.is_dirty());

        let backup = session.persist().unwrap();
        assert_eq!(std::fs::read(&backup).unwrap(), original);
        assert_ne!(std::fs::read(&path).unwrap(), original);
        assert_eq!(
            read_pal_at(session.save(), slot).unwrap().level,
            crate::limits::LEVEL_MAX
        );
        assert!(!session.is_dirty());

        remove_test_tree(&root);
    }

    #[test]
    fn verified_backups_are_exact_and_never_overwrite() {
        let (root, path) = copied_fixture("backup-uniqueness");
        let original = std::fs::read(&path).unwrap();

        let first = create_verified_backup(&path).unwrap();
        let second = create_verified_backup(&path).unwrap();

        assert_ne!(first, second);
        assert_eq!(std::fs::read(&first).unwrap(), original);
        assert_eq!(std::fs::read(&second).unwrap(), original);
        assert_eq!(
            first.parent().unwrap().file_name().unwrap(),
            "PalboxStudio-backups"
        );

        remove_test_tree(&root);
    }

    #[test]
    fn external_change_is_refused_without_backup_or_replacement() {
        let (root, path) = copied_fixture("stale");
        let mut session = SaveSession::open(&path).unwrap();
        let slot = list_pals(session.save())[0].slot;
        crate::pal::set_level(
            pal_param_mut(session.save_mut(), slot).expect("fixture Pal"),
            80,
        );

        let mut external = std::fs::read(&path).unwrap();
        let index = external.len() / 2;
        external[index] ^= 0x01;
        std::fs::write(&path, &external).unwrap();

        let error = session.persist().unwrap_err();
        assert!(error.contains("changed externally"));
        assert_eq!(std::fs::read(&path).unwrap(), external);
        assert!(!root.join("PalboxStudio-backups").exists());

        remove_test_tree(&root);
    }

    #[test]
    fn encode_failure_leaves_source_untouched_and_creates_no_backup() {
        let (root, path) = copied_fixture("encode-failure");
        let original = std::fs::read(&path).unwrap();
        let mut session = SaveSession::open(&path).unwrap();
        ue::set_prop(
            &mut session.save_mut().root.properties,
            "PalboxStudioUnknownTestProperty",
            ue::int_prop(1),
        );

        let error = session.persist().unwrap_err();
        assert!(error.contains("missing property schema"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(!root.join("PalboxStudio-backups").exists());

        remove_test_tree(&root);
    }
}

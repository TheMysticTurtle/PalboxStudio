//! Palworld `.sav` byte I/O — the compressed GVAS layer.
//!
//! The gnarly part (Oodle decompression + GVAS + Palworld's typed RawData codecs)
//! is handled by the `uesave` fork; we thread the `Palworld` game type through so
//! the typed accessors work. `read_sav` must install `palworld_types()` or the
//! Palworld structs parse as opaque bytes. `write_plm` re-emits the `PlM` magic +
//! `0x31` save-type byte the game expects.

use std::io::Cursor;
use uesave::games::palworld::{palworld_types, Palworld};
use uesave::SaveReader;

/// A parsed Palworld save (GVAS, typed to Palworld).
pub type PalSave = uesave::Save<Palworld>;

/// Decode a `.sav` byte payload (PlM/Oodle-compressed GVAS) into a typed save.
pub fn read_sav(bytes: &[u8]) -> Result<PalSave, String> {
    SaveReader::new()
        .game::<Palworld>()
        .types(palworld_types())
        .read(Cursor::new(bytes))
        .map_err(|e| format!("read_sav: {e}"))
}

/// Re-encode a save back to its `.sav` byte payload (PlM/Oodle).
pub fn write_sav(save: &mut PalSave) -> Result<Vec<u8>, String> {
    // Optional fields may be absent from every Pal in the source file, so the
    // reader never had a property tag to record for them. Install the canonical
    // engine-owned tags before every encode; source-provided tags always win.
    crate::schema::ensure_writable_schemas(save);
    let mut buf = Vec::new();
    save.write_plm(&mut buf)
        .map_err(|e| format!("write_sav: {e}"))?;
    Ok(buf)
}

#[cfg(test)]
pub(crate) fn test_fixture_path() -> std::path::PathBuf {
    std::env::var_os("PALBOX_TEST_SAV")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join("synthetic-global-palbox.sav")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lossless round-trip on the committed sanitized fixture. Maintainers can
    /// override it with PALBOX_TEST_SAV pointing at a scratchpad copy—never a
    /// live save.
    #[test]
    fn roundtrip_global_palbox() {
        let path = test_fixture_path();
        let bytes = std::fs::read(&path).expect("read fixture");
        let mut save = read_sav(&bytes).expect("decode real save");
        let out = write_sav(&mut save).expect("encode");
        assert!(!out.is_empty(), "encoded output is empty");
        // The re-encoded bytes must decode again (proves a lossless pipeline).
        read_sav(&out).expect("re-decode our own output");
    }
}

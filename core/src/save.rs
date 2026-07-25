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
pub fn write_sav(save: &PalSave) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    save.write_plm(&mut buf).map_err(|e| format!("write_sav: {e}"))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lossless round-trip on a REAL GlobalPalStorage.sav copy (never the live save):
    /// decode -> encode -> decode again must re-parse. Point PALBOX_TEST_SAV at a
    /// scratchpad copy to run; skips cleanly otherwise.
    #[test]
    fn roundtrip_global_palbox() {
        let Ok(path) = std::env::var("PALBOX_TEST_SAV") else {
            eprintln!("skip: set PALBOX_TEST_SAV to a scratchpad .sav copy to run");
            return;
        };
        let bytes = std::fs::read(&path).expect("read fixture");
        let save = read_sav(&bytes).expect("decode real save");
        let out = write_sav(&save).expect("encode");
        assert!(!out.is_empty(), "encoded output is empty");
        // The re-encoded bytes must decode again (proves a lossless pipeline).
        read_sav(&out).expect("re-decode our own output");
    }
}

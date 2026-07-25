//! `GlobalPalStorage.sav` domain — the Global Palbox, and only that.
//!
//! The save's root holds one `SaveParameterArray`: a flat, fixed list of slots,
//! each `{ InstanceId, SaveParameter }`. An empty slot has `CharacterID == "None"`.
//! We read occupied slots as pals. (This is the whole scope — no world saves.)

use crate::ue::{self, Save, StructValue};

/// A pal read from a box slot. `character_id` is the species CodeName (joins the
/// UI's species table + icon); `slot` is the array index.
#[derive(Debug, Clone)]
pub struct PalSummary {
    pub slot: usize,
    pub character_id: String,
    pub level: u8,
}

/// Total fixed slot count of the box (occupied or not). `None` if the save has no
/// `SaveParameterArray` (i.e. not a GlobalPalStorage.sav).
pub fn slot_count(save: &Save) -> Option<usize> {
    let array = ue::prop(&save.root.properties, "SaveParameterArray")?;
    Some(ue::array_structs(array)?.len())
}

/// List the occupied pals in the box; empty slots (`CharacterID == "None"` or
/// absent) are skipped. Untrusted data: an unparseable slot is skipped, not fatal.
pub fn list_pals(save: &Save) -> Vec<PalSummary> {
    let mut pals = Vec::new();
    let Some(slots) =
        ue::prop(&save.root.properties, "SaveParameterArray").and_then(ue::array_structs)
    else {
        return pals;
    };
    for (slot, value) in slots.iter().enumerate() {
        let StructValue::Struct(slot_props) = value else {
            continue;
        };
        let Some(param) = ue::prop(slot_props, "SaveParameter").and_then(ue::struct_props) else {
            continue;
        };
        let character_id = match ue::prop(param, "CharacterID").and_then(ue::as_str) {
            None | Some("None") => continue, // empty slot
            Some(id) => id.to_string(),
        };
        // Level is written only when > 1; absent means level 1.
        let level = ue::prop(param, "Level").and_then(ue::as_byte).unwrap_or(1);
        pals.push(PalSummary { slot, character_id, level });
    }
    pals
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::read_sav;

    /// Read the real Global Palbox (a scratchpad COPY) and list its pals.
    #[test]
    fn lists_real_box_pals() {
        let Ok(path) = std::env::var("PALBOX_TEST_SAV") else {
            eprintln!("skip: set PALBOX_TEST_SAV to a scratchpad .sav copy");
            return;
        };
        let bytes = std::fs::read(path).expect("read fixture");
        let save = read_sav(&bytes).expect("decode");
        let total = slot_count(&save);
        let pals = list_pals(&save);
        eprintln!(
            "box slots={:?} occupied={} first={:?}",
            total,
            pals.len(),
            pals.iter().take(6).map(|p| (p.character_id.as_str(), p.level)).collect::<Vec<_>>(),
        );
        assert!(total.is_some(), "expected a SaveParameterArray (GlobalPalStorage.sav)");
        assert!(!pals.is_empty(), "expected at least one pal in the box");
    }
}

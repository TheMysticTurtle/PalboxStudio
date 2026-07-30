//! `GlobalPalStorage.sav` domain — the Global Palbox, and only that.
//!
//! The save's root holds one `SaveParameterArray`: a flat, fixed list of slots,
//! each `{ InstanceId, SaveParameter }`. An empty slot has `CharacterID == "None"`.
//! We read occupied slots as pals. (This is the whole scope — no world saves.)

use crate::ue::{self, Save, StructValue};

/// A pal read from a box slot. `character_id` is the species CodeName (joins the
/// UI's species table + icon); `slot` is the array index.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PalSummary {
    pub slot: usize,
    pub instance_id: String,
    pub character_id: String,
    pub nickname: Option<String>,
    pub gender: String,
    pub level: u8,
    pub condensation: u8,
    pub ivs: crate::pal::Ivs,
    pub souls: crate::pal::Souls,
    /// Official Work Suitability name -> per-instance AddRank bonus.
    pub work: std::collections::BTreeMap<String, i64>,
    pub is_lucky: bool,
    pub is_alpha: bool,
    pub passives: Vec<String>,
    pub equipped_moves: Vec<String>,
    pub learned_moves: Vec<String>,
}

fn slot_instance_id(slot_props: &crate::ue::Properties) -> String {
    ue::prop(slot_props, "InstanceId")
        .and_then(ue::struct_props)
        .and_then(|value| ue::prop(value, "InstanceId"))
        .and_then(ue::as_guid)
        .map(|value| format!("{value:?}"))
        .unwrap_or_default()
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
        let dto = crate::pal::read_pal(param, slot);
        pals.push(PalSummary {
            slot,
            instance_id: slot_instance_id(slot_props),
            character_id,
            nickname: dto.nickname,
            gender: dto.gender,
            level: dto.level,
            condensation: dto.condensation,
            ivs: dto.ivs,
            souls: dto.souls,
            work: dto.work,
            is_lucky: dto.is_lucky,
            is_alpha: dto.is_alpha,
            passives: dto.passives,
            equipped_moves: dto.equipped_moves,
            learned_moves: dto.learned_moves,
        });
    }
    pals
}

/// Read one pal's full editable DTO by box slot.
pub fn read_pal_at(save: &Save, slot: usize) -> Option<crate::pal::PalDto> {
    let slots =
        ue::prop(&save.root.properties, "SaveParameterArray").and_then(ue::array_structs)?;
    let StructValue::Struct(slot_props) = slots.get(slot)? else {
        return None;
    };
    let param = ue::prop(slot_props, "SaveParameter").and_then(ue::struct_props)?;
    let mut dto = crate::pal::read_pal(param, slot);
    dto.instance_id = slot_instance_id(slot_props);
    Some(dto)
}

/// Mutable access to a pal's `SaveParameter` by slot, for edits.
pub fn pal_param_mut(save: &mut Save, slot: usize) -> Option<&mut crate::ue::Properties> {
    let slots = ue::prop_mut(&mut save.root.properties, "SaveParameterArray")
        .and_then(ue::array_structs_mut)?;
    let StructValue::Struct(slot_props) = slots.get_mut(slot)? else {
        return None;
    };
    ue::prop_mut(slot_props, "SaveParameter").and_then(ue::struct_props_mut)
}

// ---- box mutations: add / clone / delete -----------------------------------
//
// The box is a flat SaveParameterArray of up to 960 slots; an empty slot is a
// complete, valid skeleton with CharacterID "None", nil InstanceId, and
// SlotIndex -1. So: clone = deepcopy an occupied slot into an empty one with a
// fresh identity; add = claim an empty slot and give it a species; delete =
// restore a slot to a pristine vacancy.
//
// NOTE: GlobalPalStorage.sav has no authoritative Slots array (SlotIndex is
// non-authoritative and duplicated in real boxes), so an added/cloned pal will
// not appear in the in-game box until it is dragged onto an empty slot. Editing
// it here and saving is safe; the UI surfaces the "drag to a slot" caveat.

/// Whether a slot value is an empty vacancy (`CharacterID` None/absent/"").
fn is_empty_slot(sv: &StructValue) -> bool {
    let Some(props) = ue::struct_value_props(sv) else {
        return false;
    };
    let cid = ue::prop(props, "SaveParameter")
        .and_then(ue::struct_props)
        .and_then(|p| ue::prop(p, "CharacterID"))
        .and_then(ue::as_str);
    matches!(cid, None | Some("None") | Some(""))
}

/// Index of the first empty slot, if any.
fn first_empty(slots: &[StructValue]) -> Option<usize> {
    slots.iter().position(is_empty_slot)
}

fn slot_slotid<'a>(sv: &'a StructValue) -> Option<&'a crate::ue::Properties> {
    let props = ue::struct_value_props(sv)?;
    ue::prop(props, "SaveParameter")
        .and_then(ue::struct_props)
        .and_then(|p| ue::prop(p, "SlotId"))
        .and_then(ue::struct_props)
}

fn slotid_container(slotid: &crate::ue::Properties) -> Option<uesave::FGuid> {
    ue::prop(slotid, "ContainerId")
        .and_then(ue::struct_props)
        .and_then(|c| ue::prop(c, "ID"))
        .and_then(ue::as_guid)
}

/// The ContainerId shared by pals already in the box, or a fresh one if empty.
fn box_container(slots: &[StructValue]) -> uesave::FGuid {
    for sv in slots {
        if is_empty_slot(sv) {
            continue;
        }
        if let Some(g) = slot_slotid(sv).and_then(slotid_container) {
            return g;
        }
    }
    ue::new_guid()
}

/// Lowest SlotIndex not yet used within `container`.
fn next_free_slot_index(slots: &[StructValue], container: &uesave::FGuid) -> i32 {
    let mut used = std::collections::HashSet::new();
    for sv in slots {
        let Some(slotid) = slot_slotid(sv) else {
            continue;
        };
        if slotid_container(slotid) != Some(*container) {
            continue;
        }
        if let Some(i) = ue::prop(slotid, "SlotIndex").and_then(ue::as_i32) {
            if i >= 0 {
                used.insert(i);
            }
        }
    }
    let mut idx = 0;
    while used.contains(&idx) {
        idx += 1;
    }
    idx
}

/// Stamp a slot's identity: InstanceId GUID + SlotId (container + index).
fn stamp_identity(
    sv: &mut StructValue,
    instance: uesave::FGuid,
    container: uesave::FGuid,
    slot_index: i32,
) -> Result<(), String> {
    let props = ue::struct_value_props_mut(sv).ok_or("slot is not a struct")?;
    let iid = ue::prop_mut(props, "InstanceId")
        .and_then(ue::struct_props_mut)
        .ok_or("slot has no InstanceId")?;
    ue::set_prop(iid, "InstanceId", ue::guid_prop(instance));
    let param = ue::prop_mut(props, "SaveParameter")
        .and_then(ue::struct_props_mut)
        .ok_or("slot has no SaveParameter")?;
    let slotid = ue::prop_mut(param, "SlotId")
        .and_then(ue::struct_props_mut)
        .ok_or("slot has no SlotId")?;
    let container_s = ue::prop_mut(slotid, "ContainerId")
        .and_then(ue::struct_props_mut)
        .ok_or("SlotId has no ContainerId")?;
    ue::set_prop(container_s, "ID", ue::guid_prop(container));
    ue::set_prop(slotid, "SlotIndex", ue::int_prop(slot_index));
    Ok(())
}

fn slots_mut(save: &mut Save) -> Result<&mut Vec<StructValue>, String> {
    ue::prop_mut(&mut save.root.properties, "SaveParameterArray")
        .and_then(ue::array_structs_mut)
        .ok_or_else(|| "not a Global Palbox (no SaveParameterArray)".to_string())
}

/// Add a brand-new pal of `species` to the first empty slot; returns its slot.
/// The empty slot is already a clean level-1 skeleton — we claim it, give it a
/// fresh identity, and set the species/level/gender.
pub fn add_pal(save: &mut Save, species: &str) -> Result<usize, String> {
    let slots = slots_mut(save)?;
    let dst = first_empty(slots).ok_or("The Global Palbox has no free slots.")?;
    let container = box_container(slots);
    let idx = next_free_slot_index(slots, &container);
    stamp_identity(&mut slots[dst], ue::new_guid(), container, idx)?;
    let param = ue::struct_value_props_mut(&mut slots[dst])
        .and_then(|p| ue::prop_mut(p, "SaveParameter"))
        .and_then(ue::struct_props_mut)
        .ok_or("new slot has no SaveParameter")?;
    crate::pal::set_species(param, species);
    crate::pal::set_level(param, 1);
    crate::pal::set_gender(param, "Male");
    Ok(dst)
}

/// Claim and fully initialize a new slot using authoritative reference data.
pub fn add_initialized_pal(
    save: &mut Save,
    species: &str,
    reference: &crate::reference::ReferenceBundle,
) -> Result<usize, String> {
    let base_code = species
        .strip_prefix("BOSS_")
        .or_else(|| species.strip_prefix("Boss_"))
        .or_else(|| species.strip_prefix("boss_"))
        .unwrap_or(species);
    let species_ref = reference
        .species
        .iter()
        .find(|value| value.code == base_code)
        .ok_or_else(|| format!("unknown Pal species: {base_code}"))?;
    if !species_ref.palbox_selectable {
        return Err(format!(
            "{} cannot be stored in the Global Palbox",
            species_ref.name
        ));
    }

    let slot = add_pal(save, species)?;
    let hp_scaling = species_ref.scaling.hp as f64;
    let alpha_rate = if species.to_uppercase().starts_with("BOSS_") {
        1.2
    } else {
        1.0
    };
    let full_hp = (500.0 + 5.0 + hp_scaling * 0.5 * alpha_rate).floor() as i64 * 1000;
    let full_food = species_ref.max_stomach.max(1) as f32;
    crate::pal::initialize_new_pal(
        pal_param_mut(save, slot).ok_or("new Pal has no SaveParameter")?,
        full_hp,
        full_food,
    );
    Ok(slot)
}

/// Deep-copy the pal at `src` into the first empty slot with a fresh identity;
/// returns the new slot. `src` must be an occupied slot.
pub fn clone_pal(save: &mut Save, src: usize) -> Result<usize, String> {
    let slots = slots_mut(save)?;
    let source = slots.get(src).ok_or("no pal at source slot")?;
    if is_empty_slot(source) {
        return Err("source slot is empty".to_string());
    }
    let source = source.clone();
    let dst = first_empty(slots).ok_or("The Global Palbox has no free slots.")?;
    let container = box_container(slots);
    let idx = next_free_slot_index(slots, &container);
    slots[dst] = source;
    stamp_identity(&mut slots[dst], ue::new_guid(), container, idx)?;
    Ok(dst)
}

/// Restore a slot to a pristine vacancy. Prefers copying a real empty slot's
/// structure; otherwise clears the identity fields in place.
pub fn delete_pal(save: &mut Save, slot: usize) -> Result<(), String> {
    let slots = slots_mut(save)?;
    if slot >= slots.len() {
        return Err("no pal at slot".to_string());
    }
    // Prefer an actual game-produced empty slot as the template.
    if let Some(tmpl) = slots
        .iter()
        .position(|sv| is_empty_slot(sv))
        .filter(|&i| i != slot)
    {
        let blank = slots[tmpl].clone();
        slots[slot] = blank;
    } else {
        stamp_identity(&mut slots[slot], ue::nil_guid(), ue::nil_guid(), -1)?;
        let param = ue::struct_value_props_mut(&mut slots[slot])
            .and_then(|p| ue::prop_mut(p, "SaveParameter"))
            .and_then(ue::struct_props_mut)
            .ok_or("slot has no SaveParameter")?;
        crate::pal::set_species(param, "None");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::{read_sav, write_sav};

    /// Read the committed sanitized Global Palbox fixture and list its Pals.
    #[test]
    fn lists_real_box_pals() {
        let path = crate::save::test_fixture_path();
        let bytes = std::fs::read(path).expect("read fixture");
        let save = read_sav(&bytes).expect("decode");
        let total = slot_count(&save);
        let pals = list_pals(&save);
        eprintln!(
            "box slots={:?} occupied={} first={:?}",
            total,
            pals.len(),
            pals.iter()
                .take(6)
                .map(|p| (p.character_id.as_str(), p.level))
                .collect::<Vec<_>>(),
        );
        assert!(
            total.is_some(),
            "expected a SaveParameterArray (GlobalPalStorage.sav)"
        );
        assert!(!pals.is_empty(), "expected at least one pal in the box");
    }

    /// Read a full pal DTO, edit it (level -> 80), save, re-read: the edit persists.
    #[test]
    fn read_edit_save_first_pal() {
        let path = crate::save::test_fixture_path();
        let bytes = std::fs::read(path).expect("read fixture");
        let mut save = read_sav(&bytes).expect("decode");
        let slot = list_pals(&save)[0].slot;

        let before = read_pal_at(&save, slot).expect("read pal dto");
        eprintln!("full DTO: {}", serde_json::to_string(&before).unwrap());

        crate::pal::set_level(pal_param_mut(&mut save, slot).expect("mut param"), 80);
        let out = write_sav(&mut save).expect("encode");
        let reloaded = read_sav(&out).expect("re-decode");
        let after = read_pal_at(&reloaded, slot).expect("re-read pal");

        assert_eq!(after.level, 80, "level edit must survive a save round-trip");
        eprintln!(
            "edited {} level {} -> {}",
            after.character_id, before.level, after.level
        );
    }

    #[test]
    fn change_species_round_trip() {
        let path = crate::save::test_fixture_path();
        let bytes = std::fs::read(path).expect("read fixture");
        let mut save = read_sav(&bytes).expect("decode");
        let slot = list_pals(&save)[0].slot;

        let before = read_pal_at(&save, slot).expect("read pal dto");
        let was_alpha = before.character_id.to_uppercase().starts_with("BOSS_");

        crate::pal::set_species(
            pal_param_mut(&mut save, slot).expect("mut param"),
            "CubeTurtle",
        );
        let out = write_sav(&mut save).expect("encode");
        let reloaded = read_sav(&out).expect("re-decode");
        let after = read_pal_at(&reloaded, slot).expect("re-read pal");

        // Base species changed; an alpha/lucky BOSS_ prefix is preserved.
        let expected = if was_alpha {
            "BOSS_CubeTurtle"
        } else {
            "CubeTurtle"
        };
        assert_eq!(
            after.character_id, expected,
            "species edit must survive round-trip"
        );
        assert_eq!(
            after.is_alpha, was_alpha,
            "changing species must not toggle alpha"
        );
    }

    #[test]
    fn add_clone_delete_round_trip() {
        let path = crate::save::test_fixture_path();
        let bytes = std::fs::read(path).expect("read fixture");
        let mut save = read_sav(&bytes).expect("decode");
        let before = list_pals(&save).len();

        // Add a default turtle.
        let added = add_pal(&mut save, "CubeTurtle").expect("add");
        // Clone the first existing pal.
        let src = list_pals(&save)[0].slot;
        let cloned = clone_pal(&mut save, src).expect("clone");
        assert_ne!(added, cloned, "add and clone must land in different slots");

        // Persist + reload: both new pals survive with unique, non-nil InstanceIds.
        let out = write_sav(&mut save).expect("encode");
        let mut reloaded = read_sav(&out).expect("re-decode");
        let after_add = list_pals(&reloaded);
        assert_eq!(after_add.len(), before + 2, "add + clone => two more pals");
        assert_eq!(
            read_pal_at(&reloaded, added)
                .expect("added pal")
                .character_id,
            "CubeTurtle",
            "added pal is the default turtle",
        );
        // Unique InstanceIds across the box (no dup identity from clone).
        let ids = collect_instance_ids(&reloaded);
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(
            ids.len(),
            unique.len(),
            "every occupied pal has a unique InstanceId"
        );

        // Delete the cloned pal: count drops by one, slot is a vacancy again.
        delete_pal(&mut reloaded, cloned).expect("delete");
        let out2 = write_sav(&mut reloaded).expect("re-encode");
        let final_save = read_sav(&out2).expect("re-decode 2");
        let final_pals = list_pals(&final_save);
        assert_eq!(final_pals.len(), before + 1, "delete removes one pal");
        assert!(
            !final_pals.iter().any(|p| p.slot == cloned),
            "deleted slot is no longer an occupied pal",
        );
    }

    fn reference_bundle() -> crate::reference::ReferenceBundle {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("palbox-reference.db");
        crate::reference::ReferenceDatabase::open(path)
            .unwrap()
            .load_ui_bundle()
            .unwrap()
    }

    #[test]
    fn initialized_add_and_species_dependent_work_validation_live_in_core() {
        let bytes = std::fs::read(crate::save::test_fixture_path()).expect("read fixture");
        let mut save = read_sav(&bytes).expect("decode");
        let reference = reference_bundle();
        let before = list_pals(&save).len();

        let slot = add_initialized_pal(&mut save, "CubeTurtle", &reference).unwrap();
        let dto = read_pal_at(&save, slot).unwrap();
        let species = reference
            .species
            .iter()
            .find(|value| value.code == "CubeTurtle")
            .unwrap();
        let expected_hp = (500.0 + 5.0 + species.scaling.hp as f64 * 0.5).floor() as i64 * 1000;
        assert_eq!(dto.hp, expected_hp);
        assert_eq!(dto.food, species.max_stomach as f32);

        assert!(add_initialized_pal(&mut save, "NotARealPal", &reference).is_err());
        assert_eq!(
            list_pals(&save).len(),
            before + 1,
            "an invalid species must not claim a slot"
        );

        let mut invalid = dto;
        let base = species.work.get("Kindling").copied().unwrap_or(0);
        invalid.work.insert(
            "Kindling".to_string(),
            i64::from(crate::limits::WORK_SUITABILITY_MAX) - base + 1,
        );
        let before_invalid = pal_param_mut(&mut save, slot).unwrap().clone();
        let result = crate::pal::apply_dto_with_reference(
            pal_param_mut(&mut save, slot).unwrap(),
            &invalid,
            &reference,
        );
        assert!(result.is_err());
        assert_eq!(
            pal_param_mut(&mut save, slot).unwrap(),
            &before_invalid,
            "invalid totals must not partially mutate the Pal"
        );
    }

    const WORK_LIST_SCHEMA: &str = "SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList";
    const WORK_NAME_SCHEMA: &str =
        "SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.WorkSuitability";
    const WORK_RANK_SCHEMA: &str =
        "SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.Rank";

    fn remove_schemas(save: &mut Save, removed: &[&str]) {
        let mut retained = uesave::PropertySchemas::new();
        for (path, schema) in save.schemas.schemas() {
            if !removed.contains(&path.as_str()) {
                retained.record(path.clone(), schema.clone());
            }
        }
        save.schemas = retained;
    }

    fn first_fixture_pal_with_work(rank: i64) -> Save {
        let bytes = std::fs::read(crate::save::test_fixture_path()).expect("read fixture");
        let mut save = read_sav(&bytes).expect("decode");
        let slot = list_pals(&save)[0].slot;
        let mut work = std::collections::BTreeMap::new();
        work.insert("Kindling".to_string(), rank);
        crate::pal::set_work(pal_param_mut(&mut save, slot).expect("mut param"), &work)
            .expect("valid work");
        save
    }

    #[test]
    fn first_work_suitability_bonus_round_trips_without_source_schemas() {
        let mut save = first_fixture_pal_with_work(2);
        remove_schemas(
            &mut save,
            &[WORK_LIST_SCHEMA, WORK_NAME_SCHEMA, WORK_RANK_SCHEMA],
        );

        let out = write_sav(&mut save).expect("engine registers missing writable schemas");
        let reloaded = read_sav(&out).expect("re-decode");
        let pal = read_pal_at(&reloaded, list_pals(&reloaded)[0].slot).expect("re-read Pal");
        assert_eq!(pal.work.get("Kindling"), Some(&2));
    }

    #[test]
    fn reported_nested_work_schema_failure_is_reproduced_and_repaired() {
        let mut save = first_fixture_pal_with_work(3);
        remove_schemas(&mut save, &[WORK_NAME_SCHEMA, WORK_RANK_SCHEMA]);

        let mut raw = Vec::new();
        let error = save
            .write_plm(&mut raw)
            .expect_err("raw uesave write must reproduce the reported failure");
        assert_eq!(
            error.to_string(),
            format!("missing property schema for path: {WORK_NAME_SCHEMA}")
        );

        let out = write_sav(&mut save).expect("core write boundary repairs missing schemas");
        let reloaded = read_sav(&out).expect("re-decode");
        let pal = read_pal_at(&reloaded, list_pals(&reloaded)[0].slot).expect("re-read Pal");
        assert_eq!(pal.work.get("Kindling"), Some(&3));
    }

    /// InstanceId GUID strings for every occupied slot (for uniqueness checks).
    fn collect_instance_ids(save: &Save) -> Vec<String> {
        let mut ids = Vec::new();
        let Some(slots) =
            ue::prop(&save.root.properties, "SaveParameterArray").and_then(ue::array_structs)
        else {
            return ids;
        };
        for sv in slots {
            if is_empty_slot(sv) {
                continue;
            }
            if let Some(iid) = ue::struct_value_props(sv)
                .and_then(|p| ue::prop(p, "InstanceId"))
                .and_then(ue::struct_props)
                .and_then(|i| ue::prop(i, "InstanceId"))
                .and_then(ue::as_guid)
            {
                ids.push(format!("{iid:?}"));
            }
        }
        ids
    }
}

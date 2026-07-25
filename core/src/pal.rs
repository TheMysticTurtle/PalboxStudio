//! Reading and editing an individual pal's `SaveParameter` — the editable fields.
//!
//! Each field has a read (view port) and a setter (edit port), so the UI can plug
//! straight in. The engine returns **raw save values**; the UI resolves codes and
//! computes display values (stats, work totals) from its reference tables.
//!
//! Move ids in the save are `EPalWazaID::<Name>`; we strip the prefix so the codes
//! join the UI's `moves.json` (bare), and re-add it on write.

use crate::ue::{self, Properties, StructValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const WAZA: &str = "EPalWazaID::";
const GENDER: &str = "EPalGenderType::";
const WORK_PFX: &str = "EPalWorkSuitability::";

/// Work suitability: internal save key -> official UI name (13).
const WORK: [(&str, &str); 13] = [
    ("EmitFlame", "Kindling"),
    ("Watering", "Watering"),
    ("Seeding", "Planting"),
    ("GenerateElectricity", "Generating Electricity"),
    ("Handcraft", "Handiwork"),
    ("Collection", "Gathering"),
    ("Deforest", "Lumbering"),
    ("Mining", "Mining"),
    ("OilExtraction", "Crude Oil Extraction"),
    ("ProductMedicine", "Medicine Production"),
    ("Cool", "Cooling"),
    ("Transport", "Transporting"),
    ("MonsterFarm", "Farming"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Souls {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub craft_speed: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ivs {
    pub hp: u8,
    pub shot: u8,
    pub defense: u8,
}

/// One pal's editable fields, keyed by box `slot`. Raw save values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PalDto {
    pub slot: usize,
    /// Stable GUID from the containing box slot. Empty only when `read_pal` is
    /// used without the slot wrapper (tests/internal helpers).
    pub instance_id: String,
    pub character_id: String,
    pub nickname: Option<String>,
    pub gender: String,
    pub level: u8,
    pub exp: i64,
    pub condensation: u8,
    pub souls: Souls,
    pub ivs: Ivs,
    /// Official work name -> AddRank bonus (raw). UI adds the species base for the total.
    pub work: BTreeMap<String, i64>,
    pub passives: Vec<String>,
    pub equipped_moves: Vec<String>,
    pub learned_moves: Vec<String>,
    pub is_lucky: bool,
    pub is_alpha: bool,
    pub hp: i64,
    pub sanity: f32,
    pub food: f32,
    pub friendship: i32,
}

fn strip(prefix: &str, s: &str) -> String {
    s.strip_prefix(prefix).unwrap_or(s).to_string()
}

/// Read all editable fields of a pal from its `SaveParameter` properties.
pub fn read_pal(sp: &Properties, slot: usize) -> PalDto {
    let character_id = ue::prop(sp, "CharacterID").and_then(ue::as_str).unwrap_or("").to_string();
    let is_lucky = ue::prop(sp, "IsRarePal").and_then(ue::as_bool).unwrap_or(false);
    let is_alpha = character_id.to_uppercase().starts_with("BOSS_") && !is_lucky;
    let gender = ue::prop(sp, "Gender")
        .and_then(ue::as_str)
        .map(|g| strip(GENDER, g))
        .unwrap_or_else(|| "Unknown".to_string());
    let byte = |k: &str| ue::prop(sp, k).and_then(ue::as_byte);

    let mut work = BTreeMap::new();
    if let Some(list) = ue::prop(sp, "GotWorkSuitabilityAddRankList").and_then(ue::array_structs) {
        for entry in list {
            let StructValue::Struct(p) = entry else { continue };
            let Some(name) = ue::prop(p, "WorkSuitability").and_then(ue::as_str) else { continue };
            let internal = strip(WORK_PFX, name);
            let rank = ue::prop(p, "Rank").and_then(ue::as_i32).unwrap_or(0) as i64;
            if let Some((_, official)) = WORK.iter().find(|(i, _)| *i == internal) {
                work.insert(official.to_string(), rank);
            }
        }
    }

    let moves = |k: &str| {
        ue::prop(sp, k)
            .and_then(ue::enum_values)
            .map(|v| v.iter().map(|s| strip(WAZA, s)).collect::<Vec<_>>())
            .unwrap_or_default()
    };

    PalDto {
        slot,
        instance_id: String::new(),
        nickname: ue::prop(sp, "NickName").and_then(ue::as_str).map(str::to_string),
        gender,
        level: byte("Level").unwrap_or(1),
        exp: ue::prop(sp, "Exp").and_then(ue::as_i64).unwrap_or(0),
        condensation: byte("Rank").unwrap_or(0),
        souls: Souls {
            hp: byte("Rank_HP").unwrap_or(0),
            attack: byte("Rank_Attack").unwrap_or(0),
            defense: byte("Rank_Defence").unwrap_or(0),
            craft_speed: byte("Rank_CraftSpeed").unwrap_or(0),
        },
        ivs: Ivs {
            hp: byte("Talent_HP").unwrap_or(0),
            shot: byte("Talent_Shot").unwrap_or(0),
            defense: byte("Talent_Defense").unwrap_or(0),
        },
        work,
        passives: ue::prop(sp, "PassiveSkillList").and_then(ue::name_values).cloned().unwrap_or_default(),
        equipped_moves: moves("EquipWaza"),
        learned_moves: moves("MasteredWaza"),
        is_lucky,
        is_alpha,
        hp: ue::prop(sp, "Hp").and_then(ue::fixed_point64).unwrap_or(0),
        sanity: ue::prop(sp, "SanityValue").and_then(ue::as_f32).unwrap_or(100.0),
        food: ue::prop(sp, "FullStomach").and_then(ue::as_f32).unwrap_or(150.0),
        friendship: ue::prop(sp, "FriendshipPoint").and_then(ue::as_i32).unwrap_or(0),
        character_id,
    }
}

// ---- edit ports (setters over a mutable SaveParameter) ----

pub fn set_level(sp: &mut Properties, level: u8) {
    // Level is written only when > 1; absent = level 1 (matches the save format).
    if level > 1 {
        ue::set_prop(sp, "Level", ue::byte_prop(level));
    } else {
        ue::remove_prop(sp, "Level");
    }
}
pub fn set_nickname(sp: &mut Properties, name: &str) {
    ue::set_prop(sp, "NickName", ue::str_prop(name));
    if ue::prop(sp, "FilteredNickName").is_some() {
        ue::set_prop(sp, "FilteredNickName", ue::str_prop(name));
    }
}
pub fn set_gender(sp: &mut Properties, gender: &str) {
    ue::set_prop(sp, "Gender", ue::enum_prop(&format!("{GENDER}{gender}")));
}
pub fn set_iv(sp: &mut Properties, stat: &str, value: u8) {
    let key = match stat {
        "hp" => "Talent_HP",
        "shot" => "Talent_Shot",
        "defense" => "Talent_Defense",
        _ => return,
    };
    ue::set_prop(sp, key, ue::byte_prop(value));
}
pub fn set_soul(sp: &mut Properties, stat: &str, rank: u8) {
    let key = match stat {
        "hp" => "Rank_HP",
        "attack" => "Rank_Attack",
        "defense" => "Rank_Defence",
        "craftSpeed" => "Rank_CraftSpeed",
        _ => return,
    };
    if rank > 0 {
        ue::set_prop(sp, key, ue::byte_prop(rank));
    } else {
        ue::remove_prop(sp, key);
    }
}
pub fn set_condensation(sp: &mut Properties, rank: u8) {
    if rank > 0 {
        ue::set_prop(sp, "Rank", ue::byte_prop(rank));
    } else {
        ue::remove_prop(sp, "Rank");
    }
}
pub fn set_hp(sp: &mut Properties, value: i64) {
    ue::set_prop(sp, "Hp", ue::fixed_point64_prop(value.max(0)));
    // Normalize the legacy spelling so a stale duplicate cannot win on read.
    ue::remove_prop(sp, "HP");
}
pub fn set_sanity(sp: &mut Properties, value: f32) {
    ue::set_prop(sp, "SanityValue", ue::float_prop(value.clamp(0.0, 100.0)));
}
pub fn set_food(sp: &mut Properties, value: f32) {
    ue::set_prop(sp, "FullStomach", ue::float_prop(value.max(0.0)));
}
pub fn set_friendship(sp: &mut Properties, value: i32) {
    ue::set_prop(sp, "FriendshipPoint", ue::int_prop(value.clamp(-10_000, 200_000)));
}
pub fn set_lucky(sp: &mut Properties, lucky: bool) {
    if lucky {
        ue::set_prop(sp, "IsRarePal", ue::bool_prop(true));
    } else {
        ue::remove_prop(sp, "IsRarePal");
    }
}

fn set_character_id(sp: &mut Properties, value: &str) {
    let variant = match ue::prop(sp, "CharacterID") {
        Some(crate::ue::Property::Name(_)) => ue::name_prop(value),
        Some(crate::ue::Property::Enum(_)) => ue::enum_prop(value),
        _ => ue::str_prop(value),
    };
    ue::set_prop(sp, "CharacterID", variant);
}

/// Change the pal's species by rewriting `CharacterID`, mirroring PalEdit's
/// `SetType`: keep an alpha/lucky `BOSS_` prefix (the game stores the boss
/// variant for those) and preserve the property's on-disk variant (Name/Str/
/// Enum) so the save round-trips byte-for-byte. The new species' stats, work
/// suitability and natural learnset are derived by the game from `CharacterID`,
/// so no other field needs writing here.
pub fn set_species(sp: &mut Properties, code: &str) {
    let current = ue::prop(sp, "CharacterID").and_then(ue::as_str).unwrap_or("");
    let prefix = if current.to_uppercase().starts_with("BOSS_") { "BOSS_" } else { "" };
    // Strip any prefix the caller passed; we re-apply the pal's own.
    let base = code
        .strip_prefix("BOSS_")
        .or_else(|| code.strip_prefix("boss_"))
        .unwrap_or(code);
    let value = format!("{prefix}{base}");
    set_character_id(sp, &value);
}

/// Set the mutually-exclusive Alpha/Lucky variant flags and keep CharacterID in
/// the form Palworld expects: either trait adds `BOSS_`; Lucky additionally
/// writes `IsRarePal`. Clearing both removes the prefix and the rare flag.
pub fn set_variant(sp: &mut Properties, alpha: bool, lucky: bool) {
    // Lucky wins if an untrusted caller sends both true. The UI prevents that,
    // and normalizing here keeps the save representation unambiguous.
    let alpha = alpha && !lucky;
    let current = ue::prop(sp, "CharacterID").and_then(ue::as_str).unwrap_or("");
    let base = current
        .strip_prefix("BOSS_")
        .or_else(|| current.strip_prefix("Boss_"))
        .or_else(|| current.strip_prefix("boss_"))
        .unwrap_or(current);
    let value = if alpha || lucky {
        format!("BOSS_{base}")
    } else {
        base.to_string()
    };
    set_character_id(sp, &value);
    set_lucky(sp, lucky);
}
pub fn set_passives(sp: &mut Properties, codes: Vec<String>) {
    ue::set_prop(sp, "PassiveSkillList", ue::name_array_prop(codes));
}
pub fn set_equipped_moves(sp: &mut Properties, codes: Vec<String>) {
    let full = codes
        .into_iter()
        .map(|c| if c.starts_with(WAZA) { c } else { format!("{WAZA}{c}") })
        .collect();
    ue::set_prop(sp, "EquipWaza", ue::enum_array_prop(full));
}
pub fn set_learned_moves(sp: &mut Properties, codes: Vec<String>) {
    if codes.is_empty() {
        // Do not auto-fill MasteredWaza from the natural learnset. Real saves
        // commonly omit it, and absence must remain absence until the user
        // explicitly adds a non-natural move.
        ue::remove_prop(sp, "MasteredWaza");
        return;
    }
    let full = codes
        .into_iter()
        .map(|c| if c.starts_with(WAZA) { c } else { format!("{WAZA}{c}") })
        .collect();
    ue::set_prop(sp, "MasteredWaza", ue::enum_array_prop(full));
}

pub fn set_work(sp: &mut Properties, work: &BTreeMap<String, i64>) {
    // Rebuild in canonical game order and write only non-zero AddRank entries.
    // This avoids the zero-rank bloat that broke work assignment in PalEdit.
    let mut entries = Vec::new();
    for (internal, official) in WORK {
        let Some(rank) = work.get(official).copied().filter(|value| *value != 0) else {
            continue;
        };
        let mut properties = Properties::default();
        ue::set_prop(
            &mut properties,
            "WorkSuitability",
            ue::enum_prop(&format!("{WORK_PFX}{internal}")),
        );
        ue::set_prop(
            &mut properties,
            "Rank",
            ue::int_prop(rank.clamp(i32::MIN as i64, i32::MAX as i64) as i32),
        );
        entries.push(StructValue::Struct(properties));
    }
    ue::remove_prop(sp, "GotWorkSuitabilityAddRankList");
    if !entries.is_empty() {
        ue::set_prop(
            sp,
            "GotWorkSuitabilityAddRankList",
            ue::struct_array_prop(entries),
        );
    }
}

/// Healthy defaults for a newly claimed Global Palbox slot. Clearing revive
/// and sickness markers is as important as writing positive HP: otherwise the
/// game can still treat a full-HP Pal as incapacitated.
pub fn initialize_new_pal(sp: &mut Properties, hp: i64, food: f32) {
    for marker in [
        "PalReviveTimer",
        "PhysicalHealth",
        "WorkerSick",
        "HungerType",
        "FoodWithStatusEffect",
    ] {
        ue::remove_prop(sp, marker);
    }
    set_hp(sp, hp);
    set_sanity(sp, 100.0);
    set_food(sp, food);
    set_friendship(sp, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_and_lucky_keep_character_id_and_rare_flag_in_sync() {
        let mut sp = Properties::default();
        ue::set_prop(&mut sp, "CharacterID", ue::name_prop("Baphomet"));

        set_variant(&mut sp, true, false);
        assert_eq!(ue::prop(&sp, "CharacterID").and_then(ue::as_str), Some("BOSS_Baphomet"));
        assert_eq!(ue::prop(&sp, "IsRarePal").and_then(ue::as_bool), None);

        set_variant(&mut sp, false, true);
        assert_eq!(ue::prop(&sp, "CharacterID").and_then(ue::as_str), Some("BOSS_Baphomet"));
        assert_eq!(ue::prop(&sp, "IsRarePal").and_then(ue::as_bool), Some(true));

        set_variant(&mut sp, false, false);
        assert_eq!(ue::prop(&sp, "CharacterID").and_then(ue::as_str), Some("Baphomet"));
        assert_eq!(ue::prop(&sp, "IsRarePal").and_then(ue::as_bool), None);
    }

    #[test]
    fn editable_status_and_work_fields_round_trip_through_ports() {
        let mut sp = Properties::default();
        ue::set_prop(&mut sp, "HP", ue::fixed_point64_prop(1));
        ue::set_prop(&mut sp, "PalReviveTimer", ue::float_prop(30.0));
        initialize_new_pal(&mut sp, 552_000, 580.0);
        assert_eq!(ue::prop(&sp, "Hp").and_then(ue::fixed_point64), Some(552_000));
        assert!(ue::prop(&sp, "HP").is_none());
        assert!(ue::prop(&sp, "PalReviveTimer").is_none());
        assert_eq!(ue::prop(&sp, "SanityValue").and_then(ue::as_f32), Some(100.0));
        assert_eq!(ue::prop(&sp, "FullStomach").and_then(ue::as_f32), Some(580.0));

        set_friendship(&mut sp, 210_000);
        assert_eq!(
            ue::prop(&sp, "FriendshipPoint").and_then(ue::as_i32),
            Some(200_000)
        );

        let mut work = BTreeMap::new();
        work.insert("Kindling".to_string(), 2);
        work.insert("Mining".to_string(), 0);
        set_work(&mut sp, &work);
        let rows = ue::prop(&sp, "GotWorkSuitabilityAddRankList")
            .and_then(ue::array_structs)
            .unwrap();
        assert_eq!(rows.len(), 1);
    }
}

//! Reading and editing an individual pal's `SaveParameter` — the editable fields.
//!
//! Each field has a read (view port) and a setter (edit port), so the UI can plug
//! straight in. The engine returns **raw save values**; the UI resolves codes and
//! receives engine-computed display values from the same cached reference data
//! used for validation and save mutations.
//!
//! Move ids in the save are `EPalWazaID::<Name>`; we strip the prefix so the codes
//! join the UI's `moves.json` (bare), and re-add it on write.

use crate::ue::{self, Properties, StructValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const WAZA: &str = "EPalWazaID::";
const GENDER: &str = "EPalGenderType::";
const WORK_PFX: &str = "EPalWorkSuitability::";

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

/// One pal's editable fields, keyed by box `slot`. Save-backed values use the
/// editor's documented domains; save-only encodings (such as condensation's
/// stored 1..=5 rank versus its displayed 0..=4 stars) are translated here.
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
    /// Internal Work Suitability code -> save AddRank bonus (raw). This remains
    /// inside the engine; public views expose effective totals instead.
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
    let character_id = ue::prop(sp, "CharacterID")
        .and_then(ue::as_str)
        .unwrap_or("")
        .to_string();
    let is_lucky = ue::prop(sp, "IsRarePal")
        .and_then(ue::as_bool)
        .unwrap_or(false);
    let is_alpha = character_id.to_uppercase().starts_with("BOSS_") && !is_lucky;
    let gender = ue::prop(sp, "Gender")
        .and_then(ue::as_str)
        .map(|g| strip(GENDER, g))
        .unwrap_or_else(|| "Unknown".to_string());
    let byte = |k: &str| ue::prop(sp, k).and_then(ue::as_byte);

    let mut work = BTreeMap::new();
    if let Some(list) = ue::prop(sp, "GotWorkSuitabilityAddRankList").and_then(ue::array_structs) {
        for entry in list {
            let StructValue::Struct(p) = entry else {
                continue;
            };
            let Some(name) = ue::prop(p, "WorkSuitability").and_then(ue::as_str) else {
                continue;
            };
            let internal = strip(WORK_PFX, name);
            let rank = ue::prop(p, "Rank").and_then(ue::as_i32).unwrap_or(0) as i64;
            work.insert(internal, rank);
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
        nickname: ue::prop(sp, "NickName")
            .and_then(ue::as_str)
            .map(str::to_string),
        gender,
        level: byte("Level").unwrap_or(1),
        exp: ue::prop(sp, "Exp").and_then(ue::as_i64).unwrap_or(0),
        // Palworld stores condensation as rank 1..=5, but presents it as
        // 0..=4 stars. Keep that one-based encoding inside the save boundary.
        condensation: byte("Rank").unwrap_or(1).saturating_sub(1),
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
        passives: ue::prop(sp, "PassiveSkillList")
            .and_then(ue::name_values)
            .cloned()
            .unwrap_or_default(),
        equipped_moves: moves("EquipWaza"),
        learned_moves: moves("MasteredWaza"),
        is_lucky,
        is_alpha,
        hp: ue::prop(sp, "Hp").and_then(ue::fixed_point64).unwrap_or(0),
        sanity: ue::prop(sp, "SanityValue")
            .and_then(ue::as_f32)
            .unwrap_or(100.0),
        food: ue::prop(sp, "FullStomach")
            .and_then(ue::as_f32)
            .unwrap_or(150.0),
        friendship: ue::prop(sp, "FriendshipPoint")
            .and_then(ue::as_i32)
            .unwrap_or(0),
        character_id,
    }
}

// ---- edit ports (setters over a mutable SaveParameter) ----

pub fn set_level(sp: &mut Properties, level: u8, limits: &crate::reference::EditorLimits) {
    let min = limits.level_min as u8;
    let max = limits.level_max as u8;
    let level = level.clamp(min, max);
    // Level is written only when > 1; absent = level 1 (matches the save format).
    if level > min {
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
pub fn clear_nickname(sp: &mut Properties) {
    ue::remove_prop(sp, "NickName");
    ue::remove_prop(sp, "FilteredNickName");
}
pub fn set_gender(sp: &mut Properties, gender: &str) {
    ue::set_prop(sp, "Gender", ue::enum_prop(&format!("{GENDER}{gender}")));
}
pub fn set_iv(sp: &mut Properties, stat: &str, value: u8, limits: &crate::reference::EditorLimits) {
    let key = match stat {
        "hp" => "Talent_HP",
        "shot" => "Talent_Shot",
        "defense" => "Talent_Defense",
        _ => return,
    };
    ue::set_prop(
        sp,
        key,
        ue::byte_prop(value.clamp(limits.iv_min as u8, limits.iv_max as u8)),
    );
}
pub fn set_soul(
    sp: &mut Properties,
    stat: &str,
    rank: u8,
    limits: &crate::reference::EditorLimits,
) {
    let key = match stat {
        "hp" => "Rank_HP",
        "attack" => "Rank_Attack",
        "defense" => "Rank_Defence",
        "craftSpeed" => "Rank_CraftSpeed",
        _ => return,
    };
    let min = limits.soul_rank_min as u8;
    let rank = rank.clamp(min, limits.soul_rank_max as u8);
    if rank > min {
        ue::set_prop(sp, key, ue::byte_prop(rank));
    } else {
        ue::remove_prop(sp, key);
    }
}
pub fn set_condensation(sp: &mut Properties, rank: u8, limits: &crate::reference::EditorLimits) {
    // The DTO/UI value is the number of visible stars (0..=4). The save field
    // is always one-based (1..=5), including Rank=1 for an uncondensed Pal.
    let stars = rank.clamp(limits.condensation_min as u8, limits.condensation_max as u8);
    ue::set_prop(sp, "Rank", ue::byte_prop(stars + 1));
}
pub fn set_hp(sp: &mut Properties, value: i64) {
    ue::set_prop(sp, "Hp", ue::fixed_point64_prop(value.max(0)));
    // Normalize the legacy spelling so a stale duplicate cannot win on read.
    ue::remove_prop(sp, "HP");
}
pub fn set_sanity(sp: &mut Properties, value: f32, limits: &crate::reference::EditorLimits) {
    ue::set_prop(
        sp,
        "SanityValue",
        ue::float_prop(value.clamp(limits.sanity_min as f32, limits.sanity_max as f32)),
    );
}
pub fn set_food(sp: &mut Properties, value: f32) {
    ue::set_prop(sp, "FullStomach", ue::float_prop(value.max(0.0)));
}
pub fn set_friendship(sp: &mut Properties, value: i32, limits: &crate::reference::EditorLimits) {
    ue::set_prop(
        sp,
        "FriendshipPoint",
        ue::int_prop(value.clamp(limits.friendship_min as i32, limits.friendship_max as i32)),
    );
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

/// Change the pal's species by rewriting `CharacterID`: keep an alpha/lucky
/// `BOSS_` prefix (the game stores the boss
/// variant for those) and preserve the property's on-disk variant (Name/Str/
/// Enum) so the save round-trips byte-for-byte. The new species' stats, work
/// suitability and natural learnset are derived by the game from `CharacterID`,
/// so no other field needs writing here.
pub fn set_species(sp: &mut Properties, code: &str) {
    let current = ue::prop(sp, "CharacterID")
        .and_then(ue::as_str)
        .unwrap_or("");
    let prefix = if current.to_uppercase().starts_with("BOSS_") {
        "BOSS_"
    } else {
        ""
    };
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
    let current = ue::prop(sp, "CharacterID")
        .and_then(ue::as_str)
        .unwrap_or("");
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
pub fn set_passives(sp: &mut Properties, codes: Vec<String>, max: usize) {
    ue::set_prop(
        sp,
        "PassiveSkillList",
        ue::name_array_prop(codes.into_iter().take(max).collect()),
    );
}
pub fn set_equipped_moves(sp: &mut Properties, codes: Vec<String>, max: usize) {
    let full = codes
        .into_iter()
        .take(max)
        .map(|c| {
            if c.starts_with(WAZA) {
                c
            } else {
                format!("{WAZA}{c}")
            }
        })
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
        .map(|c| {
            if c.starts_with(WAZA) {
                c
            } else {
                format!("{WAZA}{c}")
            }
        })
        .collect();
    ue::set_prop(sp, "MasteredWaza", ue::enum_array_prop(full));
}

pub fn set_work(
    sp: &mut Properties,
    work: &BTreeMap<String, i64>,
    catalog: &crate::reference::ReferenceCatalog,
) -> Result<(), String> {
    for (code, rank) in work {
        if catalog.work_type(code).is_none() {
            return Err(format!("unknown Work Suitability: {code}"));
        }
        i32::try_from(*rank)
            .map_err(|_| format!("Work Suitability rank is outside IntProperty range: {rank}"))?;
    }

    // Rebuild in canonical game order and write only non-zero AddRank entries.
    // This avoids the zero-rank bloat that breaks in-game work assignment.
    let mut entries = Vec::new();
    for definition in catalog.work_types() {
        let Some(rank) = work
            .get(&definition.code)
            .copied()
            .filter(|value| *value != 0)
        else {
            continue;
        };
        let mut properties = Properties::default();
        ue::set_prop(
            &mut properties,
            "WorkSuitability",
            ue::enum_prop(&format!("{WORK_PFX}{}", definition.code)),
        );
        ue::set_prop(&mut properties, "Rank", ue::int_prop(rank as i32));
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
    Ok(())
}

fn validate_code_list(
    label: &str,
    codes: &[String],
    maximum: usize,
    known: impl Fn(&str) -> bool,
) -> Result<(), String> {
    if codes.len() > maximum {
        return Err(format!("{label} accepts at most {maximum} entries"));
    }
    let mut seen = std::collections::HashSet::new();
    for code in codes {
        if !seen.insert(code) {
            return Err(format!("{label} contains {code:?} more than once"));
        }
        if !known(code) {
            return Err(format!("unknown {label} code: {code}"));
        }
    }
    Ok(())
}

/// Apply user-facing values transactionally. Static limits, Work definitions,
/// species bases, progression tables, and calculation operands all come from
/// the validated in-memory reference catalog.
pub fn apply_input(
    sp: &mut Properties,
    input: &crate::projection::PalInput,
    catalog: &crate::reference::ReferenceCatalog,
) -> Result<(), String> {
    let limits = catalog.bundle().limits;
    let rules = catalog.bundle().calculation_rules;
    let species = catalog
        .species(&input.character_id)
        .ok_or_else(|| format!("unknown Pal species: {}", input.character_id))?;
    let current_pal = read_pal(sp, input.slot);
    let current_species = catalog.species(&current_pal.character_id);
    let species_changed =
        current_species.map(|value| value.code.as_str()) != Some(species.code.as_str());
    if !species.palbox_selectable {
        return Err(format!(
            "{} cannot be stored in the Global Palbox",
            species.name
        ));
    }
    if input.exp != current_pal.exp {
        return Err("EXP editing is not exposed by this engine contract".to_string());
    }
    match input.gender.as_str() {
        "Male" | "Female" => {}
        "Unknown" if current_pal.gender == "Unknown" => {}
        _ => return Err(format!("unsupported Pal gender: {}", input.gender)),
    }
    if !(limits.level_min..=limits.level_max).contains(&i64::from(input.level)) {
        return Err(format!(
            "level {} is outside {}..={}",
            input.level, limits.level_min, limits.level_max
        ));
    }
    for (label, value) in [
        ("HP IV", input.ivs.hp),
        ("Attack IV", input.ivs.shot),
        ("Defense IV", input.ivs.defense),
    ] {
        if !(limits.iv_min..=limits.iv_max).contains(&i64::from(value)) {
            return Err(format!(
                "{label} {value} is outside {}..={}",
                limits.iv_min, limits.iv_max
            ));
        }
    }
    for (label, value) in [
        ("HP Soul rank", input.souls.hp),
        ("Attack Soul rank", input.souls.attack),
        ("Defense Soul rank", input.souls.defense),
        ("Work Speed Soul rank", input.souls.craft_speed),
    ] {
        if !(limits.soul_rank_min..=limits.soul_rank_max).contains(&i64::from(value)) {
            return Err(format!(
                "{label} {value} is outside {}..={}",
                limits.soul_rank_min, limits.soul_rank_max
            ));
        }
    }
    if !(limits.condensation_min..=limits.condensation_max).contains(&i64::from(input.condensation))
    {
        return Err(format!(
            "condensation {} is outside {}..={}",
            input.condensation, limits.condensation_min, limits.condensation_max
        ));
    }
    if !(limits.sanity_min as f64..=limits.sanity_max as f64).contains(&input.sanity) {
        return Err(format!(
            "sanity {} is outside {}..={}",
            input.sanity, limits.sanity_min, limits.sanity_max
        ));
    }
    if !(0.0..=1.0).contains(&input.food_percent) {
        return Err(format!(
            "food percent {} is outside 0..=1",
            input.food_percent
        ));
    }
    let trust_min_rank = catalog
        .bundle()
        .friendship_ranks
        .keys()
        .copied()
        .min()
        .unwrap_or(0);
    let trust_max_rank = catalog
        .bundle()
        .friendship_ranks
        .keys()
        .copied()
        .max()
        .unwrap_or(0);
    if !(trust_min_rank..=trust_max_rank).contains(&input.trust.rank) {
        return Err(format!(
            "Trust rank {} is outside {}..={}",
            input.trust.rank, trust_min_rank, trust_max_rank
        ));
    }
    if !(0.0..=1.0).contains(&input.trust.progress) {
        return Err(format!(
            "Trust progress {} is outside 0..=1",
            input.trust.progress
        ));
    }
    let passives_max = limits.passives_max as usize;
    validate_code_list("passives", &input.passives, passives_max, |code| {
        catalog.bundle().passives.contains_key(code)
    })?;
    let equipped_moves_max = limits.equipped_moves_max as usize;
    validate_code_list(
        "equipped moves",
        &input.equipped_moves,
        equipped_moves_max,
        |code| catalog.bundle().moves.contains_key(code),
    )?;
    validate_code_list("learned moves", &input.learned_moves, usize::MAX, |code| {
        catalog.bundle().moves.contains_key(code)
    })?;

    for code in input.work.keys() {
        if catalog.work_type(code).is_none() {
            return Err(format!("unknown Work Suitability: {code}"));
        }
    }
    let work_bonus = if species_changed {
        // A species change preserves the Pal's save-backed AddRank bonuses.
        // Effective totals are re-projected against the new species base.
        for definition in catalog.work_types() {
            let base = species.work.get(&definition.code).copied().unwrap_or(0);
            let bonus = current_pal.work.get(&definition.code).copied().unwrap_or(0);
            let total = base + bonus;
            if !(limits.work_suitability_min..=limits.work_suitability_max).contains(&total) {
                return Err(format!(
                    "changing species would make {} total level {total}, outside {}..={}",
                    definition.name, limits.work_suitability_min, limits.work_suitability_max
                ));
            }
        }
        current_pal.work
    } else {
        let mut bonuses = BTreeMap::new();
        for definition in catalog.work_types() {
            let base = species.work.get(&definition.code).copied().unwrap_or(0);
            let total = input.work.get(&definition.code).copied().unwrap_or(base);
            if !(limits.work_suitability_min..=limits.work_suitability_max).contains(&total) {
                return Err(format!(
                    "{} total level {total} is outside {}..={}",
                    definition.name, limits.work_suitability_min, limits.work_suitability_max
                ));
            }
            let bonus = total - base;
            if bonus != 0 {
                bonuses.insert(definition.code.clone(), bonus);
            }
        }
        bonuses
    };

    let hp_scaled = input.hp.max(0) as f64 * rules.save_hp_scale;
    if !hp_scaled.is_finite() || hp_scaled > i64::MAX as f64 {
        return Err("HP is too large for the save representation".to_string());
    }
    let food = input.food_percent * species.max_stomach.max(0) as f64;
    let friendship = crate::projection::friendship_points(&input.trust, catalog);

    // Build on a clone so no validation/encoding failure can partially mutate
    // the in-memory Pal.
    let mut edited = sp.clone();

    set_species(&mut edited, &input.character_id);
    set_variant(&mut edited, input.is_alpha, input.is_lucky);
    set_level(&mut edited, input.level, &limits);
    match input
        .nickname
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        Some(name) => set_nickname(&mut edited, name),
        None => clear_nickname(&mut edited),
    }
    if input.gender != "Unknown" {
        set_gender(&mut edited, &input.gender);
    }
    set_iv(&mut edited, "hp", input.ivs.hp, &limits);
    set_iv(&mut edited, "shot", input.ivs.shot, &limits);
    set_iv(&mut edited, "defense", input.ivs.defense, &limits);
    set_soul(&mut edited, "hp", input.souls.hp, &limits);
    set_soul(&mut edited, "attack", input.souls.attack, &limits);
    set_soul(&mut edited, "defense", input.souls.defense, &limits);
    set_soul(&mut edited, "craftSpeed", input.souls.craft_speed, &limits);
    set_condensation(&mut edited, input.condensation, &limits);
    set_work(&mut edited, &work_bonus, catalog)?;
    set_passives(&mut edited, input.passives.clone(), passives_max);
    set_equipped_moves(
        &mut edited,
        input.equipped_moves.clone(),
        equipped_moves_max,
    );
    set_learned_moves(&mut edited, input.learned_moves.clone());
    set_hp(&mut edited, hp_scaled.round() as i64);
    set_sanity(&mut edited, input.sanity as f32, &limits);
    set_food(&mut edited, food as f32);
    set_friendship(&mut edited, friendship, &limits);

    *sp = edited;
    Ok(())
}

/// Healthy defaults for a newly claimed Global Palbox slot. Clearing revive
/// and sickness markers is as important as writing positive HP: otherwise the
/// game can still treat a full-HP Pal as incapacitated.
pub fn initialize_new_pal(
    sp: &mut Properties,
    hp: i64,
    food: f32,
    limits: &crate::reference::EditorLimits,
) {
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
    set_sanity(sp, limits.sanity_max as f32, limits);
    set_food(sp, food);
    set_friendship(sp, 0, limits);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condensation_translates_between_display_stars_and_save_rank() {
        let catalog = crate::test_reference_catalog();
        let limits = catalog.bundle().limits;
        let mut sp = Properties::default();

        // A missing Rank is treated like the game's baseline Rank=1.
        assert_eq!(read_pal(&sp, 0).condensation, 0);

        for stars in limits.condensation_min as u8..=limits.condensation_max as u8 {
            set_condensation(&mut sp, stars, &limits);
            assert_eq!(
                ue::prop(&sp, "Rank").and_then(ue::as_byte),
                Some(stars + 1),
                "{stars} displayed stars must be stored one rank higher"
            );
            assert_eq!(
                read_pal(&sp, 0).condensation,
                stars,
                "stored rank must map back to the same displayed stars"
            );
        }

        // Keep the public mutation port inside the verified game range.
        set_condensation(&mut sp, u8::MAX, &limits);
        assert_eq!(
            ue::prop(&sp, "Rank").and_then(ue::as_byte),
            Some(limits.condensation_max as u8 + 1)
        );
        assert_eq!(read_pal(&sp, 0).condensation, limits.condensation_max as u8);
    }

    #[test]
    fn alpha_and_lucky_keep_character_id_and_rare_flag_in_sync() {
        let mut sp = Properties::default();
        ue::set_prop(&mut sp, "CharacterID", ue::name_prop("Baphomet"));

        set_variant(&mut sp, true, false);
        assert_eq!(
            ue::prop(&sp, "CharacterID").and_then(ue::as_str),
            Some("BOSS_Baphomet")
        );
        assert_eq!(ue::prop(&sp, "IsRarePal").and_then(ue::as_bool), None);

        set_variant(&mut sp, false, true);
        assert_eq!(
            ue::prop(&sp, "CharacterID").and_then(ue::as_str),
            Some("BOSS_Baphomet")
        );
        assert_eq!(ue::prop(&sp, "IsRarePal").and_then(ue::as_bool), Some(true));

        set_variant(&mut sp, false, false);
        assert_eq!(
            ue::prop(&sp, "CharacterID").and_then(ue::as_str),
            Some("Baphomet")
        );
        assert_eq!(ue::prop(&sp, "IsRarePal").and_then(ue::as_bool), None);
    }

    #[test]
    fn editable_status_and_work_fields_round_trip_through_ports() {
        let catalog = crate::test_reference_catalog();
        let limits = catalog.bundle().limits;
        let mut sp = Properties::default();
        ue::set_prop(&mut sp, "HP", ue::fixed_point64_prop(1));
        ue::set_prop(&mut sp, "PalReviveTimer", ue::float_prop(30.0));
        initialize_new_pal(&mut sp, 552_000, 580.0, &limits);
        assert_eq!(
            ue::prop(&sp, "Hp").and_then(ue::fixed_point64),
            Some(552_000)
        );
        assert!(ue::prop(&sp, "HP").is_none());
        assert!(ue::prop(&sp, "PalReviveTimer").is_none());
        assert_eq!(
            ue::prop(&sp, "SanityValue").and_then(ue::as_f32),
            Some(limits.sanity_max as f32)
        );
        assert_eq!(
            ue::prop(&sp, "FullStomach").and_then(ue::as_f32),
            Some(580.0)
        );

        set_gender(&mut sp, "Female");
        assert_eq!(read_pal(&sp, 0).gender, "Female");

        set_friendship(&mut sp, i32::MAX, &limits);
        assert_eq!(
            ue::prop(&sp, "FriendshipPoint").and_then(ue::as_i32),
            Some(limits.friendship_max as i32)
        );

        let mut work = BTreeMap::new();
        work.insert("EmitFlame".to_string(), 2);
        work.insert("Mining".to_string(), 0);
        set_work(&mut sp, &work, &catalog).unwrap();
        let rows = ue::prop(&sp, "GotWorkSuitabilityAddRankList")
            .and_then(ue::array_structs)
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn work_mutation_is_canonical_validated_and_removes_an_empty_list() {
        let catalog = crate::test_reference_catalog();
        let mut sp = Properties::default();
        let mut work = BTreeMap::new();
        work.insert("Mining".to_string(), 3);
        work.insert("EmitFlame".to_string(), 2);
        set_work(&mut sp, &work, &catalog).unwrap();

        let rows = ue::prop(&sp, "GotWorkSuitabilityAddRankList")
            .and_then(ue::array_structs)
            .unwrap();
        let names = rows
            .iter()
            .filter_map(ue::struct_value_props)
            .filter_map(|row| ue::prop(row, "WorkSuitability"))
            .filter_map(ue::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "EPalWorkSuitability::EmitFlame",
                "EPalWorkSuitability::Mining"
            ]
        );

        let before_invalid = sp.clone();
        work.insert("Not a real job".to_string(), 1);
        assert!(set_work(&mut sp, &work, &catalog).is_err());
        assert_eq!(sp, before_invalid, "invalid work must not mutate the Pal");

        let all_zero = catalog
            .work_types()
            .iter()
            .map(|work_type| (work_type.code.clone(), 0))
            .collect();
        set_work(&mut sp, &all_zero, &catalog).unwrap();
        assert!(ue::prop(&sp, "GotWorkSuitabilityAddRankList").is_none());
    }

    #[test]
    fn core_setters_enforce_documented_limits() {
        let catalog = crate::test_reference_catalog();
        let limits = catalog.bundle().limits;
        let mut sp = Properties::default();
        set_level(&mut sp, u8::MAX, &limits);
        set_iv(&mut sp, "hp", u8::MAX, &limits);
        set_soul(&mut sp, "hp", u8::MAX, &limits);
        set_passives(
            &mut sp,
            (0..10).map(|value| format!("Passive{value}")).collect(),
            limits.passives_max as usize,
        );
        set_equipped_moves(
            &mut sp,
            (0..10).map(|value| format!("Move{value}")).collect(),
            limits.equipped_moves_max as usize,
        );

        assert_eq!(
            ue::prop(&sp, "Level").and_then(ue::as_byte),
            Some(limits.level_max as u8)
        );
        assert_eq!(
            ue::prop(&sp, "Talent_HP").and_then(ue::as_byte),
            Some(limits.iv_max as u8)
        );
        assert_eq!(
            ue::prop(&sp, "Rank_HP").and_then(ue::as_byte),
            Some(limits.soul_rank_max as u8)
        );
        assert_eq!(
            ue::prop(&sp, "PassiveSkillList")
                .and_then(ue::name_values)
                .map(Vec::len),
            Some(limits.passives_max as usize)
        );
        assert_eq!(
            ue::prop(&sp, "EquipWaza")
                .and_then(ue::enum_values)
                .map(Vec::len),
            Some(limits.equipped_moves_max as usize)
        );
    }
}

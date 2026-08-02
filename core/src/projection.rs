//! Engine-owned public views and semantic edit input.
//!
//! Raw save encodings stay in `pal::PalDto`. Every frontend receives the same
//! canonical projection and submits user-facing values; the engine alone
//! translates those values back to Palworld's storage representation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::pal::{Ivs, PalDto, Souls};
use crate::reference::{PartnerSkillRankRef, ReferenceCatalog, SpeciesRef};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustInput {
    pub rank: i64,
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PalInput {
    pub slot: usize,
    pub instance_id: String,
    pub character_id: String,
    pub nickname: Option<String>,
    pub gender: String,
    pub level: u8,
    pub exp: i64,
    pub condensation: u8,
    pub is_awakened: bool,
    pub souls: Souls,
    pub ivs: Ivs,
    /// Internal Work Suitability code -> desired effective level.
    pub work: BTreeMap<String, i64>,
    pub passives: Vec<String>,
    pub equipped_moves: Vec<String>,
    pub learned_moves: Vec<String>,
    pub is_lucky: bool,
    pub is_alpha: bool,
    /// User-facing whole HP. The engine applies the save fixed-point scale.
    pub hp: i64,
    pub sanity: f64,
    /// Normalized 0..=1 stomach fill.
    pub food_percent: f64,
    pub trust: TrustInput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSuitabilityView {
    pub code: String,
    pub name: String,
    pub icon: String,
    pub base_level: i64,
    pub bonus_level: i64,
    pub total_level: i64,
    pub available: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatStatsView {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustView {
    pub rank: i64,
    pub min_rank: i64,
    pub max_rank: i64,
    pub progress: f64,
    pub points: i64,
    pub rank_start_points: i64,
    pub next_rank_points: i64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpView {
    pub points: i64,
    pub level_start_points: i64,
    pub next_level_points: i64,
    pub to_next_level: i64,
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerSkillView {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub element: Option<String>,
    pub gear_name: Option<String>,
    pub technology_level: Option<i64>,
    pub level: i64,
    pub active_rank: Option<PartnerSkillRankRef>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PalProjection {
    pub species_name: String,
    pub elements: Vec<String>,
    pub max_stomach: i64,
    pub work: Vec<WorkSuitabilityView>,
    pub stats: CombatStatsView,
    pub trust: TrustView,
    pub exp: ExpView,
    pub partner_skill: Option<PartnerSkillView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PalView {
    pub editable: PalInput,
    pub projection: PalProjection,
}

fn species_for<'a>(pal: &PalDto, catalog: &'a ReferenceCatalog) -> Result<&'a SpeciesRef, String> {
    catalog
        .species(&pal.character_id)
        .ok_or_else(|| format!("unknown Pal species: {}", pal.character_id))
}

fn work_projection(
    pal: &PalDto,
    species: &SpeciesRef,
    catalog: &ReferenceCatalog,
) -> Vec<WorkSuitabilityView> {
    catalog
        .work_types()
        .iter()
        .map(|work| {
            let base_level = species.work.get(&work.code).copied().unwrap_or(0);
            let bonus_level = pal.work.get(&work.code).copied().unwrap_or(0);
            let total_level = base_level + bonus_level;
            WorkSuitabilityView {
                code: work.code.clone(),
                name: work.name.clone(),
                icon: work.icon.clone(),
                base_level,
                bonus_level,
                total_level,
                available: base_level > 0 || bonus_level != 0,
            }
        })
        .collect()
}

fn passive_percent(pal: &PalDto, effect_type: &str, catalog: &ReferenceCatalog) -> f64 {
    pal.passives
        .iter()
        .filter_map(|code| catalog.bundle().passives.get(code))
        .flat_map(|passive| &passive.effects)
        .filter(|effect| {
            effect.effect_type == effect_type
                && matches!(
                    effect.target.as_deref(),
                    None | Some("None") | Some("ToSelf") | Some("ToSelfAndTrainer")
                )
        })
        .filter_map(|effect| effect.value)
        .sum()
}

fn finish_stat(
    base: f64,
    soul_rank: u8,
    condensation: u8,
    passive_percent: f64,
    catalog: &ReferenceCatalog,
) -> i64 {
    let rules = catalog.bundle().calculation_rules;
    let soul_multiplier = 1.0 + f64::from(soul_rank) * rules.soul_bonus_percent_per_rank / 100.0;
    let condensation_multiplier =
        1.0 + f64::from(condensation) * rules.condensation_stat_bonus_percent_per_star / 100.0;
    let trained = (base * soul_multiplier * condensation_multiplier).floor();
    (trained * (1.0 + passive_percent / 100.0))
        .floor()
        .max(rules.displayed_stat_min) as i64
}

fn combat_stats(pal: &PalDto, species: &SpeciesRef, catalog: &ReferenceCatalog) -> CombatStatsView {
    let rules = catalog.bundle().calculation_rules;
    let hp_iv = 1.0 + f64::from(pal.ivs.hp) * rules.iv_stat_bonus_ratio_per_point;
    let attack_iv = 1.0 + f64::from(pal.ivs.shot) * rules.iv_stat_bonus_ratio_per_point;
    let defense_iv = 1.0 + f64::from(pal.ivs.defense) * rules.iv_stat_bonus_ratio_per_point;
    let alpha_hp_multiplier = if pal.is_alpha {
        rules.alpha_hp_multiplier
    } else {
        1.0
    };
    let level = f64::from(pal.level);
    let hp_base = (rules.hp_flat_base
        + rules.hp_per_level * level
        + species.scaling.hp as f64
            * rules.hp_scaling_factor
            * level
            * hp_iv
            * alpha_hp_multiplier)
        .floor();
    let attack_base = (rules.attack_flat_base
        + species.scaling.attack as f64 * rules.attack_scaling_factor * level * attack_iv)
        .floor();
    let defense_base = (rules.defense_flat_base
        + species.scaling.defense as f64 * rules.defense_scaling_factor * level * defense_iv)
        .floor();

    CombatStatsView {
        hp: finish_stat(
            hp_base,
            pal.souls.hp,
            pal.condensation,
            passive_percent(pal, "MaxHP", catalog),
            catalog,
        ),
        attack: finish_stat(
            attack_base,
            pal.souls.attack,
            pal.condensation,
            passive_percent(pal, "ShotAttack", catalog),
            catalog,
        ),
        defense: finish_stat(
            defense_base,
            pal.souls.defense,
            pal.condensation,
            passive_percent(pal, "Defense", catalog),
            catalog,
        ),
    }
}

fn trust_view(points: i32, catalog: &ReferenceCatalog) -> TrustView {
    let limits = catalog.bundle().limits;
    let points = i64::from(points).clamp(limits.friendship_min, limits.friendship_max);
    let ranks = &catalog.bundle().friendship_ranks;
    let min_rank = ranks.keys().copied().min().unwrap_or(0);
    let max_rank = ranks.keys().copied().max().unwrap_or(0);
    let mut rank = min_rank;
    let mut rank_start_points = *ranks.get(&min_rank).unwrap_or(&limits.friendship_min);
    for (&candidate_rank, &threshold) in ranks {
        if threshold > points {
            break;
        }
        rank = candidate_rank;
        rank_start_points = threshold;
    }
    let next = ranks
        .range((rank + 1)..)
        .next()
        .map(|(_, threshold)| *threshold)
        .unwrap_or(rank_start_points);
    let progress = if next > rank_start_points {
        (points - rank_start_points) as f64 / (next - rank_start_points) as f64
    } else {
        1.0
    }
    .clamp(0.0, 1.0);
    TrustView {
        rank,
        min_rank,
        max_rank,
        progress,
        points,
        rank_start_points,
        next_rank_points: next,
    }
}

fn exp_view(pal: &PalDto, catalog: &ReferenceCatalog) -> ExpView {
    let levels = &catalog.bundle().exp_levels;
    let start = levels
        .get(&i64::from(pal.level))
        .map(|row| row.pal_total_exp)
        .unwrap_or(pal.exp);
    let at_cap = i64::from(pal.level) >= catalog.bundle().limits.level_max;
    let next = if at_cap {
        start
    } else {
        levels
            .get(&(i64::from(pal.level) + 1))
            .map(|row| row.pal_total_exp)
            .unwrap_or(start)
    };
    let progress = if next > start {
        (pal.exp - start) as f64 / (next - start) as f64
    } else {
        1.0
    }
    .clamp(0.0, 1.0);
    ExpView {
        points: pal.exp,
        level_start_points: start,
        next_level_points: next,
        to_next_level: (next - pal.exp).max(0),
        progress,
    }
}

fn partner_skill(
    pal: &PalDto,
    species: &SpeciesRef,
    catalog: &ReferenceCatalog,
) -> Option<PartnerSkillView> {
    let partner = species.partner_skill.as_ref()?;
    let limits = catalog.bundle().limits;
    let rules = catalog.bundle().calculation_rules;
    let level = (i64::from(pal.condensation) + rules.partner_skill_level_offset).clamp(
        limits.partner_skill_level_min,
        limits.partner_skill_level_max,
    );
    Some(PartnerSkillView {
        name: partner.name.clone(),
        description: partner.description.clone(),
        category: partner.category.clone(),
        element: partner.element.clone(),
        gear_name: partner.gear_name.clone(),
        technology_level: partner.technology_level,
        level,
        active_rank: partner
            .ranks
            .iter()
            .find(|rank| rank.rank == level)
            .cloned(),
    })
}

pub fn project_pal(pal: PalDto, catalog: &ReferenceCatalog) -> Result<PalView, String> {
    let species = species_for(&pal, catalog)?;
    let work = work_projection(&pal, species, catalog);
    let trust = trust_view(pal.friendship, catalog);
    let rules = catalog.bundle().calculation_rules;
    let hp = ((pal.hp as f64) / rules.save_hp_scale).round().max(0.0) as i64;
    let food_percent = if species.max_stomach > 0 {
        (f64::from(pal.food) / species.max_stomach as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let editable = PalInput {
        slot: pal.slot,
        instance_id: pal.instance_id.clone(),
        character_id: pal.character_id.clone(),
        nickname: pal.nickname.clone(),
        gender: pal.gender.clone(),
        level: pal.level,
        exp: pal.exp,
        condensation: pal.condensation,
        is_awakened: pal.is_awakened,
        souls: pal.souls.clone(),
        ivs: pal.ivs.clone(),
        work: work
            .iter()
            .map(|value| (value.code.clone(), value.total_level))
            .collect(),
        passives: pal.passives.clone(),
        equipped_moves: pal.equipped_moves.clone(),
        learned_moves: pal.learned_moves.clone(),
        is_lucky: pal.is_lucky,
        is_alpha: pal.is_alpha,
        hp,
        sanity: f64::from(pal.sanity),
        food_percent,
        trust: TrustInput {
            rank: trust.rank,
            progress: trust.progress,
        },
    };
    let projection = PalProjection {
        species_name: species.name.clone(),
        elements: species.elements.clone(),
        max_stomach: species.max_stomach,
        stats: combat_stats(&pal, species, catalog),
        trust,
        exp: exp_view(&pal, catalog),
        partner_skill: partner_skill(&pal, species, catalog),
        work,
    };
    Ok(PalView {
        editable,
        projection,
    })
}

pub fn friendship_points(input: &TrustInput, catalog: &ReferenceCatalog) -> i32 {
    let limits = catalog.bundle().limits;
    let ranks = &catalog.bundle().friendship_ranks;
    let min_rank = ranks.keys().copied().min().unwrap_or(0);
    let max_rank = ranks.keys().copied().max().unwrap_or(0);
    let rank = input.rank.clamp(min_rank, max_rank);
    let start = *ranks.get(&rank).unwrap_or(&limits.friendship_min);
    let next = ranks
        .range((rank + 1)..)
        .next()
        .map(|(_, threshold)| *threshold)
        .unwrap_or(start);
    let progress = input.progress.clamp(0.0, 1.0);
    let points = if next > start {
        start + ((next - start) as f64 * progress).round() as i64
    } else {
        start
    }
    .clamp(limits.friendship_min, limits.friendship_max);
    i32::try_from(points).unwrap_or_else(|_| {
        if points.is_negative() {
            i32::MIN
        } else {
            i32::MAX
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_db_trust_rank_round_trips_including_negative_ranks() {
        let catalog = crate::test_reference_catalog();
        let ranks = &catalog.bundle().friendship_ranks;
        let minimum = *ranks.keys().min().unwrap();
        let maximum = *ranks.keys().max().unwrap();

        for (&rank, &points) in ranks {
            let view = trust_view(points as i32, &catalog);
            assert_eq!(view.rank, rank);
            assert_eq!(view.min_rank, minimum);
            assert_eq!(view.max_rank, maximum);
            assert_eq!(view.progress, if rank == maximum { 1.0 } else { 0.0 });
            assert_eq!(
                friendship_points(
                    &TrustInput {
                        rank,
                        progress: 0.0,
                    },
                    &catalog,
                ),
                points as i32
            );
        }
    }

    #[test]
    fn projected_view_contains_db_work_stats_exp_and_partner_progression() {
        let catalog = crate::test_reference_catalog();
        let bytes = std::fs::read(crate::save::test_fixture_path()).expect("read fixture");
        let save = crate::save::read_sav(&bytes).expect("decode fixture");
        let slot = crate::globalbox::list_pals(&save)[0].slot;
        let raw = crate::globalbox::read_pal_at(&save, slot).expect("fixture Pal");
        let view = project_pal(raw.clone(), &catalog).expect("project fixture Pal");
        let rules = catalog.bundle().calculation_rules;

        assert_eq!(view.projection.work.len(), catalog.work_types().len());
        let species = catalog.species(&raw.character_id).expect("fixture species");
        for work in &view.projection.work {
            let base = species.work.get(&work.code).copied().unwrap_or(0);
            let bonus = raw.work.get(&work.code).copied().unwrap_or(0);
            assert_eq!(
                work.available,
                base > 0 || bonus != 0,
                "availability for {} must come from species base or saved bonus",
                work.code
            );
        }
        assert!(
            view.projection.work.iter().any(|work| !work.available),
            "fixture species must exercise unavailable Work Suitabilities"
        );
        assert!(view
            .projection
            .stats
            .hp
            .ge(&(rules.displayed_stat_min as i64)));
        assert!(view
            .projection
            .stats
            .attack
            .ge(&(rules.displayed_stat_min as i64)));
        assert!(view
            .projection
            .stats
            .defense
            .ge(&(rules.displayed_stat_min as i64)));
        assert_eq!(
            view.editable.hp,
            ((raw.hp as f64) / rules.save_hp_scale).round() as i64
        );
        assert_eq!(view.projection.exp.points, raw.exp);
        if let Some(partner) = view.projection.partner_skill {
            assert_eq!(
                partner.level,
                (i64::from(raw.condensation) + rules.partner_skill_level_offset).clamp(
                    catalog.bundle().limits.partner_skill_level_min,
                    catalog.bundle().limits.partner_skill_level_max,
                )
            );
        }
    }
}

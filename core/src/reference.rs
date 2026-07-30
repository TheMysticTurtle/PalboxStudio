//! Static reference data and user-authored passive presets.
//!
//! The reference DB is read-only game data. The user DB contains only app
//! metadata (currently named passive presets); it never mirrors per-Pal state
//! from `GlobalPalStorage.sav`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};

const USER_SCHEMA: &str = include_str!("../../database/user-schema.sql");
const USER_MIGRATION_V2: &str = include_str!("../../database/migrations/user-v2-groups.sql");
const USER_MIGRATION_V3: &str = include_str!("../../database/migrations/user-v3-app-settings.sql");
const USER_MIGRATION_V4: &str =
    include_str!("../../database/migrations/user-v4-dynamic-preset-slots.sql");
const USER_SCHEMA_VERSION: i64 = 4;
const REFERENCE_SCHEMA_VERSION: &str = "4";

#[derive(Debug)]
pub enum DatabaseError {
    Sql(rusqlite::Error),
    Io(std::io::Error),
    Invalid(String),
}

impl Display for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(error) => write!(formatter, "database error: {error}"),
            Self::Io(error) => write!(formatter, "file error: {error}"),
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<rusqlite::Error> for DatabaseError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveOption {
    pub code: String,
    pub name: String,
    pub description: String,
    pub rating: i64,
    pub disabled: bool,
    pub available_normal_pal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassivePreset {
    pub id: i64,
    pub name: String,
    pub passive_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PalGroupMembership {
    pub instance_id: String,
    pub group_ids: Vec<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferences {
    pub last_box_path: String,
    pub auto_reopen: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveEffectRef {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub value: Option<f64>,
    pub target: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveRef {
    pub name: String,
    pub rating: i64,
    pub description: String,
    pub disabled: bool,
    pub available_normal_pal: bool,
    pub available_lucky_pal: bool,
    pub effects: Vec<PassiveEffectRef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRef {
    pub name: String,
    pub element: String,
    pub power: i64,
    pub category: String,
    pub skill_fruit: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementRef {
    pub name: String,
    pub color: String,
    pub icon: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkTypeRef {
    pub code: String,
    pub name: String,
    pub icon: String,
    pub sort_order: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLimits {
    pub level_min: i64,
    pub level_max: i64,
    pub iv_min: i64,
    pub iv_max: i64,
    pub work_suitability_min: i64,
    pub work_suitability_max: i64,
    pub soul_rank_min: i64,
    pub soul_rank_max: i64,
    pub condensation_min: i64,
    pub condensation_max: i64,
    pub equipped_moves_min: i64,
    pub equipped_moves_max: i64,
    pub passives_min: i64,
    pub passives_max: i64,
    pub sanity_min: i64,
    pub sanity_max: i64,
    pub friendship_min: i64,
    pub friendship_max: i64,
    pub partner_skill_level_min: i64,
    pub partner_skill_level_max: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationRules {
    pub soul_bonus_percent_per_rank: f64,
    pub condensation_stat_bonus_percent_per_star: f64,
    pub iv_stat_bonus_ratio_per_point: f64,
    pub alpha_hp_multiplier: f64,
    pub hp_flat_base: f64,
    pub hp_per_level: f64,
    pub hp_scaling_factor: f64,
    pub attack_flat_base: f64,
    pub attack_scaling_factor: f64,
    pub defense_flat_base: f64,
    pub defense_scaling_factor: f64,
    pub save_hp_scale: f64,
    pub displayed_stat_min: f64,
    pub partner_skill_level_offset: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpLevelRef {
    pub level: i64,
    pub pal_next_exp: i64,
    pub pal_total_exp: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerSkillRankRef {
    pub rank: i64,
    pub value_text: String,
    pub value_number: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalingRef {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerSkillRef {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub element: Option<String>,
    pub gear_name: Option<String>,
    pub technology_level: Option<i64>,
    pub ranks: Vec<PartnerSkillRankRef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RanchDropRef {
    pub item_code: Option<String>,
    pub item_name: String,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesRef {
    pub code: String,
    pub name: String,
    pub elements: Vec<String>,
    pub category: String,
    pub disabled: bool,
    pub palbox_selectable: bool,
    pub rarity: i64,
    pub size: String,
    pub genus: String,
    pub nocturnal: bool,
    pub alpha: bool,
    pub deck_index: i64,
    pub combi_rank: i64,
    pub capture_rate: f64,
    pub price: f64,
    pub food_amount: i64,
    pub max_stomach: i64,
    pub male_probability: f64,
    pub run_speed: i64,
    pub ride_speed: i64,
    pub scaling: ScalingRef,
    pub work: BTreeMap<String, i64>,
    pub moves: Vec<String>,
    pub passives: Vec<String>,
    pub partner_skill: Option<PartnerSkillRef>,
    pub farm_drops: Vec<RanchDropRef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaColumnRef {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub value_type: String,
    pub filterable: bool,
    pub displayable: bool,
    pub options: Vec<SchemaOptionRef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaOptionRef {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceBundle {
    pub passives: BTreeMap<String, PassiveRef>,
    pub moves: BTreeMap<String, MoveRef>,
    pub species: Vec<SpeciesRef>,
    pub species_aliases: BTreeMap<String, String>,
    pub elements: BTreeMap<String, ElementRef>,
    pub work_types: Vec<WorkTypeRef>,
    pub friendship_ranks: BTreeMap<i64, i64>,
    pub exp_levels: BTreeMap<i64, ExpLevelRef>,
    pub limits: EditorLimits,
    pub calculation_rules: CalculationRules,
    pub schema: Vec<SchemaColumnRef>,
}

/// Validated, indexed in-memory form of the generated reference database.
/// The serializable bundle is still exposed to frontends, while domain logic
/// resolves codes through these indexes instead of repeatedly scanning vectors.
pub struct ReferenceCatalog {
    bundle: ReferenceBundle,
    species_by_code: HashMap<String, usize>,
    work_by_code: HashMap<String, usize>,
}

impl ReferenceCatalog {
    pub fn new(bundle: ReferenceBundle) -> Result<Self> {
        let mut species_by_code = HashMap::with_capacity(bundle.species.len());
        for (index, species) in bundle.species.iter().enumerate() {
            if species_by_code
                .insert(species.code.clone(), index)
                .is_some()
            {
                return Err(DatabaseError::Invalid(format!(
                    "duplicate species code in reference bundle: {}",
                    species.code
                )));
            }
        }

        let mut work_by_code = HashMap::with_capacity(bundle.work_types.len());
        for (index, work) in bundle.work_types.iter().enumerate() {
            if work.code.trim().is_empty()
                || work.name.trim().is_empty()
                || work.icon.trim().is_empty()
            {
                return Err(DatabaseError::Invalid(
                    "Work Suitability rows require code, name, and icon".to_string(),
                ));
            }
            if work_by_code.insert(work.code.clone(), index).is_some() {
                return Err(DatabaseError::Invalid(format!(
                    "duplicate Work Suitability code in reference bundle: {}",
                    work.code
                )));
            }
        }
        if work_by_code.is_empty() {
            return Err(DatabaseError::Invalid(
                "reference bundle contains no Work Suitabilities".to_string(),
            ));
        }
        for species in &bundle.species {
            if species.work.len() != work_by_code.len() {
                return Err(DatabaseError::Invalid(format!(
                    "species {} has {} Work rows; expected {}",
                    species.code,
                    species.work.len(),
                    work_by_code.len()
                )));
            }
            for code in species.work.keys() {
                if !work_by_code.contains_key(code) {
                    return Err(DatabaseError::Invalid(format!(
                        "species {} references unknown Work Suitability {code}",
                        species.code
                    )));
                }
            }
        }

        let limits = bundle.limits;
        for (name, min, max) in [
            ("level", limits.level_min, limits.level_max),
            ("iv", limits.iv_min, limits.iv_max),
            ("soul rank", limits.soul_rank_min, limits.soul_rank_max),
            (
                "condensation",
                limits.condensation_min,
                limits.condensation_max,
            ),
            ("sanity", limits.sanity_min, limits.sanity_max),
        ] {
            if min < 0 || max > i64::from(u8::MAX) || max < min {
                return Err(DatabaseError::Invalid(format!(
                    "{name} limits {min}..={max} do not fit the save byte domain"
                )));
            }
        }
        for (name, min, max) in [
            (
                "Work Suitability",
                limits.work_suitability_min,
                limits.work_suitability_max,
            ),
            ("friendship", limits.friendship_min, limits.friendship_max),
            (
                "Partner Skill level",
                limits.partner_skill_level_min,
                limits.partner_skill_level_max,
            ),
        ] {
            if max < min || min < i64::from(i32::MIN) || max > i64::from(i32::MAX) {
                return Err(DatabaseError::Invalid(format!(
                    "invalid {name} limits {min}..={max}"
                )));
            }
        }
        for species in &bundle.species {
            for (code, base) in &species.work {
                if !(limits.work_suitability_min..=limits.work_suitability_max).contains(base) {
                    return Err(DatabaseError::Invalid(format!(
                        "species {} has {code} base {base}, outside {}..={}",
                        species.code, limits.work_suitability_min, limits.work_suitability_max
                    )));
                }
            }
        }
        for (name, min, max) in [
            (
                "equipped moves",
                limits.equipped_moves_min,
                limits.equipped_moves_max,
            ),
            ("passives", limits.passives_min, limits.passives_max),
        ] {
            if min < 0 || max < min {
                return Err(DatabaseError::Invalid(format!(
                    "invalid {name} limits {min}..={max}"
                )));
            }
        }
        let rules = bundle.calculation_rules;
        if ![
            rules.soul_bonus_percent_per_rank,
            rules.condensation_stat_bonus_percent_per_star,
            rules.iv_stat_bonus_ratio_per_point,
            rules.alpha_hp_multiplier,
            rules.hp_flat_base,
            rules.hp_per_level,
            rules.hp_scaling_factor,
            rules.attack_flat_base,
            rules.attack_scaling_factor,
            rules.defense_flat_base,
            rules.defense_scaling_factor,
            rules.save_hp_scale,
            rules.displayed_stat_min,
        ]
        .into_iter()
        .all(f64::is_finite)
            || rules.save_hp_scale <= 0.0
        {
            return Err(DatabaseError::Invalid(
                "reference calculation rules contain invalid numeric values".to_string(),
            ));
        }
        if bundle.exp_levels.is_empty() || bundle.friendship_ranks.is_empty() {
            return Err(DatabaseError::Invalid(
                "reference progression tables must not be empty".to_string(),
            ));
        }
        for level in limits.level_min..=limits.level_max {
            if !bundle.exp_levels.contains_key(&level) {
                return Err(DatabaseError::Invalid(format!(
                    "reference EXP table has no row for editable level {level}"
                )));
            }
        }
        let mut previous_threshold = None;
        for (&rank, &threshold) in &bundle.friendship_ranks {
            if let Some(previous) = previous_threshold {
                if threshold <= previous {
                    return Err(DatabaseError::Invalid(format!(
                        "Friendship threshold for rank {rank} is not strictly increasing"
                    )));
                }
            }
            previous_threshold = Some(threshold);
        }
        if bundle.friendship_ranks.values().next().copied() != Some(limits.friendship_min)
            || bundle.friendship_ranks.values().next_back().copied() != Some(limits.friendship_max)
        {
            return Err(DatabaseError::Invalid(
                "friendship limits must match the first and last rank thresholds".to_string(),
            ));
        }

        Ok(Self {
            bundle,
            species_by_code,
            work_by_code,
        })
    }

    pub fn bundle(&self) -> &ReferenceBundle {
        &self.bundle
    }

    pub fn species(&self, stored_code: &str) -> Option<&SpeciesRef> {
        let base = stored_code
            .strip_prefix("BOSS_")
            .or_else(|| stored_code.strip_prefix("Boss_"))
            .or_else(|| stored_code.strip_prefix("boss_"))
            .unwrap_or(stored_code);
        let canonical = self
            .bundle
            .species_aliases
            .get(base)
            .map(String::as_str)
            .unwrap_or(base);
        self.species_by_code
            .get(canonical)
            .map(|index| &self.bundle.species[*index])
    }

    pub fn work_types(&self) -> &[WorkTypeRef] {
        &self.bundle.work_types
    }

    pub fn work_type(&self, code: &str) -> Option<&WorkTypeRef> {
        self.work_by_code
            .get(code)
            .map(|index| &self.bundle.work_types[*index])
    }
}

pub struct ReferenceDatabase {
    connection: Connection,
}

impl ReferenceDatabase {
    /// Open the generated static database with SQLite's read-only flag.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let kind: String = connection.query_row(
            "SELECT value FROM metadata WHERE key = 'database_kind'",
            [],
            |row| row.get(0),
        )?;
        if kind != "palbox-reference" {
            return Err(DatabaseError::Invalid(format!(
                "expected a Palbox reference DB, found {kind:?}"
            )));
        }
        let schema_version: String = connection.query_row(
            "SELECT value FROM metadata WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )?;
        if schema_version != REFERENCE_SCHEMA_VERSION {
            return Err(DatabaseError::Invalid(format!(
                "reference DB schema v{schema_version} is incompatible with supported v{REFERENCE_SCHEMA_VERSION}"
            )));
        }
        let game_version: String = connection.query_row(
            "SELECT value FROM metadata WHERE key = 'game_version'",
            [],
            |row| row.get(0),
        )?;
        if game_version.trim().is_empty() {
            return Err(DatabaseError::Invalid(
                "reference DB has no game-version provenance".to_string(),
            ));
        }
        Ok(Self { connection })
    }

    /// Search passive choices for the preset picker.
    ///
    /// Normal UI use passes both flags as false. The flags exist so an advanced
    /// picker can deliberately expose game-defined or disabled entries later.
    pub fn list_passives(
        &self,
        search: &str,
        include_disabled: bool,
        include_unavailable: bool,
    ) -> Result<Vec<PassiveOption>> {
        let search = search.trim();
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                code, name, description, rating, disabled, available_normal_pal
            FROM passive
            WHERE (?1 = 1 OR disabled = 0)
              AND (?2 = 1 OR available_normal_pal = 1)
              AND (
                    ?3 = ''
                 OR name LIKE '%' || ?3 || '%' COLLATE NOCASE
                 OR code LIKE '%' || ?3 || '%' COLLATE NOCASE
                 OR description LIKE '%' || ?3 || '%' COLLATE NOCASE
            )
            ORDER BY rating DESC, name COLLATE NOCASE, code
            "#,
        )?;
        let rows = statement.query_map(
            params![
                i64::from(include_disabled),
                i64::from(include_unavailable),
                search
            ],
            |row| {
                Ok(PassiveOption {
                    code: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    rating: row.get(3)?,
                    disabled: row.get::<_, i64>(4)? != 0,
                    available_normal_pal: row.get::<_, i64>(5)? != 0,
                })
            },
        )?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn validate_passive_codes(&self, codes: &[String]) -> Result<()> {
        let maximum = self.connection.query_row(
            "SELECT passives_max FROM editor_limits WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )?;
        validate_passive_codes(codes, &self.passive_code_set()?, maximum as usize)
    }

    /// Every passive code, for building the in-memory cache (validate without
    /// re-querying the DB on each preset write/apply).
    pub fn passive_code_set(&self) -> Result<HashSet<String>> {
        let mut statement = self.connection.prepare("SELECT code FROM passive")?;
        let codes = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<HashSet<String>>>()?;
        Ok(codes)
    }

    /// Materialize the compact UI-facing reference bundle from normalized
    /// tables. This replaces the generated JSON as the desktop app's source.
    pub fn load_ui_bundle(&self) -> Result<ReferenceBundle> {
        let mut passives = BTreeMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT code, name, rating, description, disabled,
                   available_normal_pal, available_lucky_pal
            FROM passive
            ORDER BY code
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                PassiveRef {
                    name: row.get(1)?,
                    rating: row.get(2)?,
                    description: row.get(3)?,
                    disabled: row.get::<_, i64>(4)? != 0,
                    available_normal_pal: row.get::<_, i64>(5)? != 0,
                    available_lucky_pal: row.get::<_, i64>(6)? != 0,
                    effects: Vec::new(),
                },
            ))
        })? {
            let (code, value) = row?;
            passives.insert(code, value);
        }
        let mut statement = self.connection.prepare(
            r#"
            SELECT passive_code, type, value, target
            FROM passive_effect
            ORDER BY passive_code, position
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                PassiveEffectRef {
                    effect_type: row.get(1)?,
                    value: row.get(2)?,
                    target: row.get(3)?,
                },
            ))
        })? {
            let (code, effect) = row?;
            if let Some(passive) = passives.get_mut(&code) {
                passive.effects.push(effect);
            }
        }

        let mut moves = BTreeMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT code, name, COALESCE(element_code, ''), power,
                   COALESCE(category, ''),
                   EXISTS (
                       SELECT 1
                       FROM item
                       WHERE item.code = 'SkillCard_' || move.code
                         AND item.type_b = 'ConsumeWazaMachine'
                         AND item.disabled = 0
                   )
            FROM move
            ORDER BY code
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                MoveRef {
                    name: row.get(1)?,
                    element: row.get(2)?,
                    power: row.get(3)?,
                    category: row.get(4)?,
                    skill_fruit: row.get::<_, i64>(5)? != 0,
                },
            ))
        })? {
            let (code, value) = row?;
            moves.insert(code, value);
        }

        let mut elements = BTreeMap::new();
        let mut statement = self.connection.prepare(
            "SELECT code, name, COALESCE(color, ''), COALESCE(icon, ''), sort_order
                 FROM element ORDER BY sort_order",
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                ElementRef {
                    name: row.get(1)?,
                    color: row.get(2)?,
                    icon: row.get(3)?,
                    sort_order: row.get(4)?,
                },
            ))
        })? {
            let (code, value) = row?;
            elements.insert(code, value);
        }

        let mut work_types = Vec::new();
        let mut statement = self
            .connection
            .prepare("SELECT code, name, icon, sort_order FROM work_type ORDER BY sort_order")?;
        for row in statement.query_map([], |row| {
            Ok(WorkTypeRef {
                code: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                sort_order: row.get(3)?,
            })
        })? {
            work_types.push(row?);
        }

        let mut friendship_ranks = BTreeMap::new();
        let mut statement = self
            .connection
            .prepare("SELECT rank, required_point FROM friendship_rank ORDER BY rank")?;
        for row in
            statement.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
        {
            let (rank, required_point) = row?;
            friendship_ranks.insert(rank, required_point);
        }

        let mut exp_levels = BTreeMap::new();
        let mut statement = self
            .connection
            .prepare("SELECT level, pal_next_exp, pal_total_exp FROM exp_level ORDER BY level")?;
        for row in statement.query_map([], |row| {
            Ok(ExpLevelRef {
                level: row.get(0)?,
                pal_next_exp: row.get(1)?,
                pal_total_exp: row.get(2)?,
            })
        })? {
            let value = row?;
            exp_levels.insert(value.level, value);
        }

        let limits = self.connection.query_row(
            r#"
            SELECT
                level_min, level_max,
                iv_min, iv_max,
                work_suitability_min, work_suitability_max,
                soul_rank_min, soul_rank_max,
                condensation_min, condensation_max,
                equipped_moves_min, equipped_moves_max,
                passives_min, passives_max,
                sanity_min, sanity_max,
                friendship_min, friendship_max,
                partner_skill_level_min, partner_skill_level_max
            FROM editor_limits
            WHERE id = 1
            "#,
            [],
            |row| {
                Ok(EditorLimits {
                    level_min: row.get(0)?,
                    level_max: row.get(1)?,
                    iv_min: row.get(2)?,
                    iv_max: row.get(3)?,
                    work_suitability_min: row.get(4)?,
                    work_suitability_max: row.get(5)?,
                    soul_rank_min: row.get(6)?,
                    soul_rank_max: row.get(7)?,
                    condensation_min: row.get(8)?,
                    condensation_max: row.get(9)?,
                    equipped_moves_min: row.get(10)?,
                    equipped_moves_max: row.get(11)?,
                    passives_min: row.get(12)?,
                    passives_max: row.get(13)?,
                    sanity_min: row.get(14)?,
                    sanity_max: row.get(15)?,
                    friendship_min: row.get(16)?,
                    friendship_max: row.get(17)?,
                    partner_skill_level_min: row.get(18)?,
                    partner_skill_level_max: row.get(19)?,
                })
            },
        )?;

        let calculation_rules = self.connection.query_row(
            r#"
            SELECT
                soul_bonus_percent_per_rank,
                condensation_stat_bonus_percent_per_star,
                iv_stat_bonus_ratio_per_point,
                alpha_hp_multiplier,
                hp_flat_base,
                hp_per_level,
                hp_scaling_factor,
                attack_flat_base,
                attack_scaling_factor,
                defense_flat_base,
                defense_scaling_factor,
                save_hp_scale,
                displayed_stat_min,
                partner_skill_level_offset
            FROM calculation_rules
            WHERE id = 1
            "#,
            [],
            |row| {
                Ok(CalculationRules {
                    soul_bonus_percent_per_rank: row.get(0)?,
                    condensation_stat_bonus_percent_per_star: row.get(1)?,
                    iv_stat_bonus_ratio_per_point: row.get(2)?,
                    alpha_hp_multiplier: row.get(3)?,
                    hp_flat_base: row.get(4)?,
                    hp_per_level: row.get(5)?,
                    hp_scaling_factor: row.get(6)?,
                    attack_flat_base: row.get(7)?,
                    attack_scaling_factor: row.get(8)?,
                    defense_flat_base: row.get(9)?,
                    defense_scaling_factor: row.get(10)?,
                    save_hp_scale: row.get(11)?,
                    displayed_stat_min: row.get(12)?,
                    partner_skill_level_offset: row.get(13)?,
                })
            },
        )?;

        let mut species_aliases = BTreeMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT alias_code, canonical_code
            FROM species_alias
            ORDER BY alias_code
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (alias, canonical) = row?;
            species_aliases.insert(alias, canonical);
        }

        let mut species_elements: HashMap<String, Vec<String>> = HashMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT species_code, element_code
            FROM species_element
            ORDER BY species_code, position
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (species_code, element_code) = row?;
            species_elements
                .entry(species_code)
                .or_default()
                .push(element_code);
        }

        let mut species_work: HashMap<String, BTreeMap<String, i64>> = HashMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT sw.species_code, wt.code, sw.base_level
            FROM species_work AS sw
            JOIN work_type AS wt ON wt.code = sw.work_code
            ORDER BY sw.species_code, wt.sort_order
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })? {
            let (species_code, code, level) = row?;
            species_work
                .entry(species_code)
                .or_default()
                .insert(code, level);
        }

        let mut species_moves: HashMap<String, Vec<String>> = HashMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT species_code, move_code
            FROM species_move
            ORDER BY species_code, unlock_level, move_code
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (species_code, move_code) = row?;
            species_moves
                .entry(species_code)
                .or_default()
                .push(move_code);
        }

        let mut species_passives: HashMap<String, Vec<String>> = HashMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT species_code, passive_code
            FROM species_passive
            ORDER BY species_code, relationship, passive_code
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (species_code, passive_code) = row?;
            species_passives
                .entry(species_code)
                .or_default()
                .push(passive_code);
        }

        let mut partner_skills = HashMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT species_code, name, description, category, element_code,
                   gear_name, technology_level
            FROM partner_skill
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                PartnerSkillRef {
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    element: row.get(4)?,
                    gear_name: row.get(5)?,
                    technology_level: row.get(6)?,
                    ranks: Vec::new(),
                },
            ))
        })? {
            let (species_code, value) = row?;
            partner_skills.insert(species_code, value);
        }
        let mut statement = self.connection.prepare(
            r#"
            SELECT species_code, rank, value_text, value_number
            FROM partner_skill_rank
            ORDER BY species_code, rank
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                PartnerSkillRankRef {
                    rank: row.get(1)?,
                    value_text: row.get(2)?,
                    value_number: row.get(3)?,
                },
            ))
        })? {
            let (species_code, rank) = row?;
            if let Some(partner_skill) = partner_skills.get_mut(&species_code) {
                partner_skill.ranks.push(rank);
            }
        }

        let mut ranch_drops: HashMap<String, Vec<RanchDropRef>> = HashMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT species_code, item_code, item_name, notes
            FROM ranch_drop
            ORDER BY species_code, position
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                RanchDropRef {
                    item_code: row.get(1)?,
                    item_name: row.get(2)?,
                    notes: row.get(3)?,
                },
            ))
        })? {
            let (species_code, value) = row?;
            ranch_drops.entry(species_code).or_default().push(value);
        }

        let mut species = Vec::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                code, name, category, disabled, palbox_selectable, rarity,
                COALESCE(size, ''), COALESCE(genus, ''), nocturnal,
                is_alpha_species, paldeck_index, breeding_rank, capture_rate,
                price, food_amount, max_stomach, male_probability, run_speed,
                ride_sprint_speed, hp_scaling, attack_scaling, defense_scaling
            FROM species
            ORDER BY
                CASE WHEN paldeck_index >= 0 THEN paldeck_index ELSE 99999 END,
                name COLLATE NOCASE, code
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                SpeciesRef {
                    code: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    disabled: row.get::<_, i64>(3)? != 0,
                    palbox_selectable: row.get::<_, i64>(4)? != 0,
                    rarity: row.get(5)?,
                    size: row.get(6)?,
                    genus: row.get(7)?,
                    nocturnal: row.get::<_, i64>(8)? != 0,
                    alpha: row.get::<_, i64>(9)? != 0,
                    deck_index: row.get(10)?,
                    combi_rank: row.get(11)?,
                    capture_rate: row.get(12)?,
                    price: row.get(13)?,
                    food_amount: row.get(14)?,
                    max_stomach: row.get(15)?,
                    male_probability: row.get(16)?,
                    run_speed: row.get(17)?,
                    ride_speed: row.get(18)?,
                    scaling: ScalingRef {
                        hp: row.get(19)?,
                        attack: row.get(20)?,
                        defense: row.get(21)?,
                    },
                    elements: Vec::new(),
                    work: BTreeMap::new(),
                    moves: Vec::new(),
                    passives: Vec::new(),
                    partner_skill: None,
                    farm_drops: Vec::new(),
                },
            ))
        })? {
            let (code, mut value) = row?;
            value.elements = species_elements.remove(&code).unwrap_or_default();
            value.work = species_work.remove(&code).unwrap_or_default();
            value.moves = species_moves.remove(&code).unwrap_or_default();
            value.passives = species_passives.remove(&code).unwrap_or_default();
            value.partner_skill = partner_skills.remove(&code);
            value.farm_drops = ranch_drops.remove(&code).unwrap_or_default();
            species.push(value);
        }

        let mut schema = Vec::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT key, label, value_type, filterable, displayable
            FROM filter_field
            ORDER BY sort_order
            "#,
        )?;
        for row in statement.query_map([], |row| {
            Ok(SchemaColumnRef {
                key: row.get(0)?,
                label: row.get(1)?,
                value_type: row.get(2)?,
                filterable: row.get::<_, i64>(3)? != 0,
                displayable: row.get::<_, i64>(4)? != 0,
                options: Vec::new(),
            })
        })? {
            let mut value = row?;
            let mut option_statement = self.connection.prepare_cached(
                r#"
                SELECT value, label
                FROM filter_option
                WHERE field_key = ?1
                ORDER BY sort_order
                "#,
            )?;
            let options = option_statement
                .query_map([&value.key], |option| {
                    Ok(SchemaOptionRef {
                        value: option.get(0)?,
                        label: option.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            value.options = options;
            schema.push(value);
        }

        Ok(ReferenceBundle {
            passives,
            moves,
            species,
            species_aliases,
            elements,
            work_types,
            friendship_ranks,
            exp_levels,
            limits,
            calculation_rules,
            schema,
        })
    }

    pub fn load_catalog(&self) -> Result<ReferenceCatalog> {
        ReferenceCatalog::new(self.load_ui_bundle()?)
    }
}

/// Validate preset passive codes against a known-valid set and the DB-backed
/// slot limit. Used with the in-memory reference cache so writes/applies never
/// re-query the reference DB.
pub fn validate_passive_codes(
    codes: &[String],
    valid: &HashSet<String>,
    maximum: usize,
) -> Result<()> {
    if codes.len() > maximum {
        return Err(DatabaseError::Invalid(format!(
            "a passive preset can contain at most {} entries",
            maximum
        )));
    }
    let mut seen = HashSet::new();
    for code in codes {
        if code.trim().is_empty() {
            return Err(DatabaseError::Invalid(
                "passive codes cannot be blank".to_string(),
            ));
        }
        if !seen.insert(code) {
            return Err(DatabaseError::Invalid(format!(
                "passive {code:?} appears more than once"
            )));
        }
        if !valid.contains(code) {
            return Err(DatabaseError::Invalid(format!(
                "unknown passive code {code:?}"
            )));
        }
    }
    Ok(())
}

pub struct UserDatabase {
    connection: Connection,
}

impl UserDatabase {
    /// Open the user DB, creating the current schema or migrating an older one.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let has_schema: bool = connection.query_row(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type = 'table' AND name = 'schema_migrations'
            )
            "#,
            [],
            |row| row.get(0),
        )?;
        if !has_schema {
            connection.execute_batch(USER_SCHEMA)?;
        }
        let schema_version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if schema_version > USER_SCHEMA_VERSION {
            return Err(DatabaseError::Invalid(format!(
                "user database schema v{schema_version} is newer than supported v{USER_SCHEMA_VERSION}"
            )));
        }
        if schema_version < 2 {
            connection.execute_batch(USER_MIGRATION_V2)?;
        }
        if schema_version < 3 {
            connection.execute_batch(USER_MIGRATION_V3)?;
        }
        if schema_version < 4 {
            connection.execute_batch(USER_MIGRATION_V4)?;
        }
        let kind: String = connection.query_row(
            "SELECT value FROM metadata WHERE key = 'database_kind'",
            [],
            |row| row.get(0),
        )?;
        if kind != "palbox-user" {
            return Err(DatabaseError::Invalid(format!(
                "expected a Palbox user DB, found {kind:?}"
            )));
        }
        Ok(Self { connection })
    }

    pub fn app_preferences(&self) -> Result<AppPreferences> {
        let last_box_path = self.app_setting("last_box_path")?;
        let auto_reopen = match self.app_setting("auto_reopen")?.as_str() {
            "0" => false,
            "1" => true,
            value => {
                return Err(DatabaseError::Invalid(format!(
                    "invalid auto_reopen setting {value:?}"
                )));
            }
        };
        Ok(AppPreferences {
            last_box_path,
            auto_reopen,
        })
    }

    pub fn save_app_preferences(&mut self, preferences: &AppPreferences) -> Result<AppPreferences> {
        if preferences.last_box_path.chars().count() > 32_768 {
            return Err(DatabaseError::Invalid(
                "last Global Palbox path is too long".to_string(),
            ));
        }
        let preferences = AppPreferences {
            last_box_path: preferences.last_box_path.clone(),
            auto_reopen: preferences.auto_reopen && !preferences.last_box_path.is_empty(),
        };
        let transaction = self.connection.transaction()?;
        transaction.execute(
            r#"
            INSERT INTO app_setting(key, value)
            VALUES ('last_box_path', ?1)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
            [&preferences.last_box_path],
        )?;
        transaction.execute(
            r#"
            INSERT INTO app_setting(key, value)
            VALUES ('auto_reopen', ?1)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
            [if preferences.auto_reopen { "1" } else { "0" }],
        )?;
        transaction.commit()?;
        Ok(preferences)
    }

    fn app_setting(&self, key: &str) -> Result<String> {
        self.connection
            .query_row(
                "SELECT value FROM app_setting WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| DatabaseError::Invalid(format!("missing app setting {key:?}")))
    }

    pub fn list_presets(&self) -> Result<Vec<PassivePreset>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name FROM passive_preset ORDER BY name COLLATE NOCASE, id")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let headers = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        headers
            .into_iter()
            .map(|(id, name)| self.preset_from_header(id, name))
            .collect()
    }

    pub fn get_preset(&self, id: i64) -> Result<PassivePreset> {
        let name = self
            .connection
            .query_row(
                "SELECT name FROM passive_preset WHERE id = ?1",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| DatabaseError::Invalid(format!("preset {id} does not exist")))?;
        self.preset_from_header(id, name)
    }

    pub fn save_preset(
        &mut self,
        valid_codes: &HashSet<String>,
        maximum_passives: usize,
        id: Option<i64>,
        name: &str,
        passive_codes: &[String],
    ) -> Result<PassivePreset> {
        let name = name.trim();
        let name_length = name.chars().count();
        if !(1..=80).contains(&name_length) {
            return Err(DatabaseError::Invalid(
                "preset name must contain 1 to 80 characters".to_string(),
            ));
        }
        validate_passive_codes(passive_codes, valid_codes, maximum_passives)?;

        let transaction = self.connection.transaction()?;
        let preset_id = if let Some(id) = id {
            let changed = transaction.execute(
                "UPDATE passive_preset SET name = ?1 WHERE id = ?2",
                params![name, id],
            )?;
            if changed == 0 {
                return Err(DatabaseError::Invalid(format!(
                    "preset {id} does not exist"
                )));
            }
            id
        } else {
            transaction.execute("INSERT INTO passive_preset(name) VALUES (?1)", [name])?;
            transaction.last_insert_rowid()
        };
        transaction.execute(
            "DELETE FROM passive_preset_entry WHERE preset_id = ?1",
            [preset_id],
        )?;
        for (slot, code) in passive_codes.iter().enumerate() {
            transaction.execute(
                r#"
                INSERT INTO passive_preset_entry(preset_id, slot, passive_code)
                VALUES (?1, ?2, ?3)
                "#,
                params![preset_id, slot as i64, code],
            )?;
        }
        transaction.commit()?;
        self.get_preset(preset_id)
    }

    pub fn delete_preset(&self, id: i64) -> Result<bool> {
        Ok(self
            .connection
            .execute("DELETE FROM passive_preset WHERE id = ?1", [id])?
            > 0)
    }

    fn preset_from_header(&self, id: i64, name: String) -> Result<PassivePreset> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT passive_code
            FROM passive_preset_entry
            WHERE preset_id = ?1
            ORDER BY slot
            "#,
        )?;
        let passive_codes = statement
            .query_map([id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(PassivePreset {
            id,
            name,
            passive_codes,
        })
    }

    pub fn list_groups(&self) -> Result<Vec<UserGroup>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name FROM pal_group ORDER BY name COLLATE NOCASE, id")?;
        let groups = statement
            .query_map([], |row| {
                Ok(UserGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DatabaseError::from)?;
        Ok(groups)
    }

    pub fn create_group(&self, name: &str) -> Result<UserGroup> {
        let name = validate_group_name(name)?;
        self.connection
            .execute("INSERT INTO pal_group(name) VALUES (?1)", [name])?;
        self.get_group(self.connection.last_insert_rowid())
    }

    pub fn rename_group(&self, id: i64, name: &str) -> Result<UserGroup> {
        let name = validate_group_name(name)?;
        let changed = self.connection.execute(
            "UPDATE pal_group SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        if changed == 0 {
            return Err(DatabaseError::Invalid(format!("group {id} does not exist")));
        }
        self.get_group(id)
    }

    pub fn delete_group(&self, id: i64) -> Result<bool> {
        Ok(self
            .connection
            .execute("DELETE FROM pal_group WHERE id = ?1", [id])?
            > 0)
    }

    pub fn list_group_memberships(&self) -> Result<Vec<PalGroupMembership>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT instance_id, group_id
            FROM pal_group_member
            ORDER BY instance_id, group_id
            "#,
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut memberships = BTreeMap::<String, Vec<i64>>::new();
        for row in rows {
            let (instance_id, group_id) = row?;
            memberships.entry(instance_id).or_default().push(group_id);
        }
        Ok(memberships
            .into_iter()
            .map(|(instance_id, group_ids)| PalGroupMembership {
                instance_id,
                group_ids,
            })
            .collect())
    }

    pub fn set_pal_groups(&mut self, instance_id: &str, group_ids: &[i64]) -> Result<Vec<i64>> {
        let instance_id = instance_id.trim();
        if !(1..=128).contains(&instance_id.chars().count()) {
            return Err(DatabaseError::Invalid(
                "Pal InstanceId must contain 1 to 128 characters".to_string(),
            ));
        }
        let mut seen = HashSet::new();
        for group_id in group_ids {
            if !seen.insert(*group_id) {
                return Err(DatabaseError::Invalid(format!(
                    "group {group_id} appears more than once"
                )));
            }
        }

        let transaction = self.connection.transaction()?;
        for group_id in group_ids {
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM pal_group WHERE id = ?1)",
                [group_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(DatabaseError::Invalid(format!(
                    "group {group_id} does not exist"
                )));
            }
        }
        transaction.execute(
            "DELETE FROM pal_group_member WHERE instance_id = ?1",
            [instance_id],
        )?;
        for group_id in group_ids {
            transaction.execute(
                "INSERT INTO pal_group_member(instance_id, group_id) VALUES (?1, ?2)",
                params![instance_id, group_id],
            )?;
        }
        transaction.commit()?;
        Ok(group_ids.to_vec())
    }

    fn get_group(&self, id: i64) -> Result<UserGroup> {
        self.connection
            .query_row(
                "SELECT id, name FROM pal_group WHERE id = ?1",
                [id],
                |row| {
                    Ok(UserGroup {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| DatabaseError::Invalid(format!("group {id} does not exist")))
    }
}

fn validate_group_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if !(1..=80).contains(&name.chars().count()) {
        return Err(DatabaseError::Invalid(
            "group name must contain 1 to 80 characters".to_string(),
        ));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Barrier};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn reference_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/palbox-reference.db")
    }

    fn unique_user_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "palbox-user-test-{}-{nonce}.db",
            std::process::id()
        ))
    }

    #[test]
    fn reference_db_is_current_and_queryable() {
        let reference = ReferenceDatabase::open(reference_path()).unwrap();
        for view in [
            "v_species_summary",
            "v_species_work_suitability",
            "v_partner_skill_progression",
            "v_move_catalog",
            "v_move_effect_catalog",
            "v_passive_catalog",
            "v_passive_effect_catalog",
            "v_reference_sources",
        ] {
            let count: i64 = reference
                .connection
                .query_row(&format!("SELECT COUNT(*) FROM {view}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert!(count > 0, "{view} should be useful in a SQLite viewer");
        }
        let passives = reference.list_passives("", false, false).unwrap();
        assert!(!passives.is_empty());
        assert!(passives.iter().all(|passive| !passive.disabled));
        assert!(passives.iter().all(|passive| passive.available_normal_pal));

        let bundle = reference.load_ui_bundle().unwrap();
        assert_eq!(bundle.species.len(), 406);
        assert_eq!(
            bundle
                .species
                .iter()
                .filter(|species| species.palbox_selectable)
                .count(),
            287
        );
        assert_eq!(bundle.species_aliases.len(), 73);
        assert_eq!(
            bundle
                .species_aliases
                .get("SUMMON_DarkAlien_MAX")
                .map(String::as_str),
            Some("DarkAlien")
        );
        assert!(
            bundle
                .species
                .iter()
                .find(|species| species.code == "DarkAlien")
                .unwrap()
                .palbox_selectable
        );
        assert!(
            !bundle
                .species
                .iter()
                .find(|species| species.code == "ElecLion")
                .unwrap()
                .palbox_selectable
        );
        assert!(
            !bundle
                .species
                .iter()
                .find(|species| species.code == "RAID_YakushimaBoss002")
                .unwrap()
                .palbox_selectable
        );
        assert_eq!(bundle.moves.len(), 351);
        assert_eq!(bundle.passives.len(), 420);
        assert!(bundle.elements.values().all(|element| {
            !element.name.is_empty()
                && !element.color.is_empty()
                && !element.icon.is_empty()
                && element.sort_order >= 0
        }));
        assert!(bundle
            .work_types
            .iter()
            .all(|work| !work.code.is_empty() && !work.name.is_empty() && !work.icon.is_empty()));
        assert!(!bundle.exp_levels.is_empty());
        assert!(bundle.limits.level_max >= bundle.limits.level_min);
        assert!(bundle.calculation_rules.save_hp_scale > 0.0);
        assert!(bundle
            .passives
            .values()
            .any(|passive| !passive.effects.is_empty()));
        assert!(bundle.passives.values().all(|passive| {
            !passive.description.trim().is_empty()
                && !passive.description.contains('<')
                && !passive.description.contains('{')
        }));
        assert_eq!(
            bundle
                .moves
                .values()
                .filter(|value| value.skill_fruit)
                .count(),
            93
        );
        assert_eq!(bundle.friendship_ranks.get(&10), Some(&200_000));
        assert!(bundle
            .species
            .iter()
            .any(|species| !species.passives.is_empty()));
        assert_eq!(
            bundle
                .species
                .iter()
                .filter(|species| species.partner_skill.is_some())
                .count(),
            348
        );
        let incineram = bundle
            .species
            .iter()
            .find(|species| species.code == "Baphomet")
            .unwrap();
        assert_eq!(
            incineram.partner_skill.as_ref().unwrap().name,
            "Flameclaw Hunter"
        );
        assert!(!incineram
            .partner_skill
            .as_ref()
            .unwrap()
            .description
            .is_empty());
        let vixy = bundle
            .species
            .iter()
            .find(|species| species.name == "Vixy")
            .unwrap();
        assert_eq!(vixy.farm_drops.len(), 7);
    }

    #[test]
    fn passive_presets_are_ordered_and_limited_to_four() {
        let reference = ReferenceDatabase::open(reference_path()).unwrap();
        let maximum = reference.load_ui_bundle().unwrap().limits.passives_max as usize;
        let codes = reference
            .list_passives("", false, false)
            .unwrap()
            .into_iter()
            .take(maximum + 1)
            .map(|passive| passive.code)
            .collect::<Vec<_>>();
        let valid = reference.passive_code_set().unwrap();
        let path = unique_user_path();
        let mut user = UserDatabase::open_or_create(&path).unwrap();
        let preset = user
            .save_preset(&valid, maximum, None, "Worker", &codes[..maximum])
            .unwrap();
        assert_eq!(preset.passive_codes, codes[..maximum]);
        assert!(user
            .save_preset(&valid, maximum, Some(preset.id), "Too many", &codes)
            .is_err());
        assert_eq!(user.list_presets().unwrap().len(), 1);
        drop(user);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn groups_persist_and_memberships_replace_atomically() {
        let path = unique_user_path();
        let mut user = UserDatabase::open_or_create(&path).unwrap();
        let combat = user.create_group("Combat Team").unwrap();
        let workers = user.create_group("Base Workers").unwrap();
        assert!(user.create_group(" combat team ").is_err());

        let assigned = user
            .set_pal_groups("instance-a", &[combat.id, workers.id])
            .unwrap();
        assert_eq!(assigned, vec![combat.id, workers.id]);
        assert!(user
            .set_pal_groups("instance-a", &[combat.id, combat.id])
            .is_err());
        assert_eq!(
            user.list_group_memberships().unwrap(),
            vec![PalGroupMembership {
                instance_id: "instance-a".to_string(),
                group_ids: vec![combat.id, workers.id],
            }]
        );

        let renamed = user.rename_group(workers.id, "Ranch Crew").unwrap();
        assert_eq!(renamed.name, "Ranch Crew");
        assert!(user.delete_group(combat.id).unwrap());
        assert_eq!(
            user.list_group_memberships().unwrap()[0].group_ids,
            vec![workers.id]
        );
        drop(user);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn app_preferences_are_durable_and_normalized() {
        let path = unique_user_path();
        let mut user = UserDatabase::open_or_create(&path).unwrap();
        assert_eq!(user.app_preferences().unwrap(), AppPreferences::default());

        let saved = user
            .save_app_preferences(&AppPreferences {
                last_box_path: "C:\\Pal\\Saved\\GlobalPalStorage.sav".to_string(),
                auto_reopen: true,
            })
            .unwrap();
        assert!(saved.auto_reopen);
        drop(user);

        let mut reopened = UserDatabase::open_or_create(&path).unwrap();
        assert_eq!(reopened.app_preferences().unwrap(), saved);
        let normalized = reopened
            .save_app_preferences(&AppPreferences {
                last_box_path: String::new(),
                auto_reopen: true,
            })
            .unwrap();
        assert_eq!(normalized, AppPreferences::default());
        drop(reopened);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn concurrent_user_database_initialization_is_idempotent() {
        let path = Arc::new(unique_user_path());
        let barrier = Arc::new(Barrier::new(4));
        let workers = (0..4)
            .map(|_| {
                let path = Arc::clone(&path);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let user = UserDatabase::open_or_create(path.as_ref()).unwrap();
                    assert_eq!(user.app_preferences().unwrap(), AppPreferences::default());
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
        fs::remove_file(path.as_ref()).unwrap();
    }

    #[test]
    fn schema_v2_migration_preserves_existing_user_metadata() {
        let path = unique_user_path();
        let mut user = UserDatabase::open_or_create(&path).unwrap();
        let valid_codes = HashSet::from(["Legend".to_string()]);
        let preset = user
            .save_preset(&valid_codes, 1, None, "Favorite", &["Legend".to_string()])
            .unwrap();
        let group = user.create_group("Workers").unwrap();
        user.set_pal_groups("instance-1", &[group.id]).unwrap();
        drop(user);

        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                r#"
                DROP TABLE app_setting;
                DELETE FROM schema_migrations WHERE version >= 3;
                UPDATE metadata SET value = '2' WHERE key = 'schema_version';
                "#,
            )
            .unwrap();
        drop(connection);

        let migrated = UserDatabase::open_or_create(&path).unwrap();
        assert_eq!(migrated.list_presets().unwrap(), vec![preset]);
        assert_eq!(migrated.list_groups().unwrap(), vec![group.clone()]);
        assert_eq!(
            migrated.list_group_memberships().unwrap(),
            vec![PalGroupMembership {
                instance_id: "instance-1".to_string(),
                group_ids: vec![group.id],
            }]
        );
        assert_eq!(
            migrated.app_preferences().unwrap(),
            AppPreferences::default()
        );
        drop(migrated);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn schema_v3_migration_preserves_presets_and_removes_the_stale_slot_cap() {
        let path = unique_user_path();
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;
                CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                ) STRICT;
                CREATE TABLE metadata (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                ) STRICT;
                CREATE TABLE passive_preset (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
                    created_at TEXT NOT NULL DEFAULT '',
                    updated_at TEXT NOT NULL DEFAULT ''
                ) STRICT;
                CREATE TABLE passive_preset_entry (
                    preset_id INTEGER NOT NULL REFERENCES passive_preset(id) ON DELETE CASCADE,
                    slot INTEGER NOT NULL CHECK (slot BETWEEN 0 AND 3),
                    passive_code TEXT NOT NULL,
                    PRIMARY KEY (preset_id, slot),
                    UNIQUE (preset_id, passive_code)
                ) STRICT;
                CREATE INDEX passive_preset_entry_code_idx
                ON passive_preset_entry(passive_code);
                INSERT INTO schema_migrations(version, applied_at) VALUES
                    (1, '2026-07-25'),
                    (2, '2026-07-25'),
                    (3, '2026-07-29');
                INSERT INTO metadata(key, value) VALUES
                    ('database_kind', 'palbox-user'),
                    ('schema_version', '3'),
                    ('preset_passive_limit', '4');
                INSERT INTO passive_preset(id, name) VALUES (1, 'Workers');
                INSERT INTO passive_preset_entry(preset_id, slot, passive_code) VALUES
                    (1, 0, 'Artisan'),
                    (1, 1, 'Serious');
                "#,
            )
            .unwrap();
        drop(connection);

        let migrated = UserDatabase::open_or_create(&path).unwrap();
        assert_eq!(
            migrated.get_preset(1).unwrap().passive_codes,
            ["Artisan", "Serious"]
        );
        migrated
            .connection
            .execute(
                "INSERT INTO passive_preset_entry(preset_id, slot, passive_code)
                 VALUES (1, 4, 'Workaholic')",
                [],
            )
            .expect("the user DB no longer duplicates the current game slot cap");
        let version: String = migrated
            .connection
            .query_row(
                "SELECT value FROM metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "4");
        let stale_limit: i64 = migrated
            .connection
            .query_row(
                "SELECT COUNT(*) FROM metadata WHERE key = 'preset_passive_limit'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stale_limit, 0);
        drop(migrated);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn schema_v1_user_database_migrates_to_current() {
        let path = unique_user_path();
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                r#"
                CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                ) STRICT;
                CREATE TABLE metadata (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                ) STRICT;
                CREATE TABLE passive_preset (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
                    created_at TEXT NOT NULL DEFAULT '',
                    updated_at TEXT NOT NULL DEFAULT ''
                ) STRICT;
                CREATE TABLE passive_preset_entry (
                    preset_id INTEGER NOT NULL REFERENCES passive_preset(id) ON DELETE CASCADE,
                    slot INTEGER NOT NULL CHECK (slot BETWEEN 0 AND 3),
                    passive_code TEXT NOT NULL,
                    PRIMARY KEY (preset_id, slot),
                    UNIQUE (preset_id, passive_code)
                ) STRICT;
                INSERT INTO schema_migrations(version, applied_at)
                VALUES (1, '2026-07-25');
                INSERT INTO metadata(key, value) VALUES
                    ('database_kind', 'palbox-user'),
                    ('schema_version', '1');
                "#,
            )
            .unwrap();
        drop(connection);

        let user = UserDatabase::open_or_create(&path).unwrap();
        assert_eq!(user.list_groups().unwrap(), Vec::<UserGroup>::new());
        let version: String = user
            .connection
            .query_row(
                "SELECT value FROM metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "4");
        assert_eq!(user.app_preferences().unwrap(), AppPreferences::default());
        drop(user);
        fs::remove_file(path).unwrap();
    }
}

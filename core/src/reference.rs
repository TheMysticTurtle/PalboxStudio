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
const USER_SCHEMA_VERSION: i64 = 3;

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
    pub color: String,
    pub icon: String,
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
    pub values: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceBundle {
    pub passives: BTreeMap<String, PassiveRef>,
    pub moves: BTreeMap<String, MoveRef>,
    pub species: Vec<SpeciesRef>,
    pub species_aliases: BTreeMap<String, String>,
    pub elements: BTreeMap<String, ElementRef>,
    pub friendship_ranks: BTreeMap<i64, i64>,
    pub schema: Vec<SchemaColumnRef>,
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
        let game_version: String = connection.query_row(
            "SELECT value FROM metadata WHERE key = 'game_version'",
            [],
            |row| row.get(0),
        )?;
        if game_version != "Palworld 1.0" {
            return Err(DatabaseError::Invalid(format!(
                "reference DB targets {game_version}, not Palworld 1.0"
            )));
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
        validate_passive_codes(codes, &self.passive_code_set()?)
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
            "SELECT code, COALESCE(color, ''), COALESCE(icon, '')
                 FROM element ORDER BY sort_order",
        )?;
        for row in statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                ElementRef {
                    color: row.get(1)?,
                    icon: row.get(2)?,
                },
            ))
        })? {
            let (code, value) = row?;
            elements.insert(code, value);
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
            SELECT sw.species_code, wt.name, sw.base_level
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
            let (species_code, name, level) = row?;
            species_work
                .entry(species_code)
                .or_default()
                .insert(name, level);
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
                },
            ))
        })? {
            let (species_code, value) = row?;
            partner_skills.insert(species_code, value);
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
                values: None,
            })
        })? {
            let mut value = row?;
            let mut option_statement = self.connection.prepare_cached(
                r#"
                SELECT value
                FROM filter_option
                WHERE field_key = ?1
                ORDER BY sort_order
                "#,
            )?;
            let values = option_statement
                .query_map([&value.key], |option| option.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            if !values.is_empty() {
                value.values = Some(values);
            }
            schema.push(value);
        }

        Ok(ReferenceBundle {
            passives,
            moves,
            species,
            species_aliases,
            elements,
            friendship_ranks,
            schema,
        })
    }
}

/// Validate preset passive codes against a known-valid set (<=4, unique,
/// non-blank, all present). Used with the in-memory reference cache so preset
/// writes/applies never re-query the reference DB.
pub fn validate_passive_codes(codes: &[String], valid: &HashSet<String>) -> Result<()> {
    if codes.len() > crate::limits::PASSIVES_MAX {
        return Err(DatabaseError::Invalid(format!(
            "a passive preset can contain at most {} entries",
            crate::limits::PASSIVES_MAX
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
        validate_passive_codes(passive_codes, valid_codes)?;

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
        assert!(bundle
            .elements
            .values()
            .all(|element| !element.color.is_empty() && !element.icon.is_empty()));
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
        let codes = reference
            .list_passives("", false, false)
            .unwrap()
            .into_iter()
            .take(5)
            .map(|passive| passive.code)
            .collect::<Vec<_>>();
        let valid = reference.passive_code_set().unwrap();
        let path = unique_user_path();
        let mut user = UserDatabase::open_or_create(&path).unwrap();
        let preset = user
            .save_preset(&valid, None, "Worker", &codes[..4])
            .unwrap();
        assert_eq!(preset.passive_codes, codes[..4]);
        assert!(user
            .save_preset(&valid, Some(preset.id), "Too many", &codes)
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
            .save_preset(&valid_codes, None, "Favorite", &["Legend".to_string()])
            .unwrap();
        let group = user.create_group("Workers").unwrap();
        user.set_pal_groups("instance-1", &[group.id]).unwrap();
        drop(user);

        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                r#"
                DROP TABLE app_setting;
                DELETE FROM schema_migrations WHERE version = 3;
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
        assert_eq!(version, "3");
        assert_eq!(user.app_preferences().unwrap(), AppPreferences::default());
        drop(user);
        fs::remove_file(path).unwrap();
    }
}

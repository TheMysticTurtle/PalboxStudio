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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveRef {
    pub name: String,
    pub rating: i64,
    pub description: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRef {
    pub name: String,
    pub element: String,
    pub power: i64,
    pub category: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementRef {
    pub color: String,
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
    pub elements: BTreeMap<String, ElementRef>,
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
        if codes.len() > crate::limits::PASSIVES_MAX {
            return Err(DatabaseError::Invalid(format!(
                "a passive preset can contain at most {} entries",
                crate::limits::PASSIVES_MAX
            )));
        }
        let mut seen = HashSet::new();
        let mut exists = self
            .connection
            .prepare_cached("SELECT EXISTS(SELECT 1 FROM passive WHERE code = ?1)")?;
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
            if !exists.query_row([code], |row| row.get::<_, bool>(0))? {
                return Err(DatabaseError::Invalid(format!(
                    "unknown passive code {code:?}"
                )));
            }
        }
        Ok(())
    }

    /// Materialize the compact UI-facing reference bundle from normalized
    /// tables. This replaces the generated JSON as the desktop app's source.
    pub fn load_ui_bundle(&self) -> Result<ReferenceBundle> {
        let mut passives = BTreeMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT code, name, rating, description
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
                },
            ))
        })? {
            let (code, value) = row?;
            passives.insert(code, value);
        }

        let mut moves = BTreeMap::new();
        let mut statement = self.connection.prepare(
            r#"
            SELECT code, name, COALESCE(element_code, ''), power,
                   COALESCE(category, '')
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
                },
            ))
        })? {
            let (code, value) = row?;
            moves.insert(code, value);
        }

        let mut elements = BTreeMap::new();
        let mut statement = self
            .connection
            .prepare("SELECT code, COALESCE(color, '') FROM element ORDER BY sort_order")?;
        for row in statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, ElementRef { color: row.get(1)? }))
        })? {
            let (code, value) = row?;
            elements.insert(code, value);
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
                code, name, category, disabled, rarity, COALESCE(size, ''),
                COALESCE(genus, ''), nocturnal, is_alpha_species, paldeck_index,
                breeding_rank, capture_rate, price, food_amount, max_stomach,
                male_probability, run_speed, ride_sprint_speed, hp_scaling,
                attack_scaling, defense_scaling
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
                    rarity: row.get(4)?,
                    size: row.get(5)?,
                    genus: row.get(6)?,
                    nocturnal: row.get::<_, i64>(7)? != 0,
                    alpha: row.get::<_, i64>(8)? != 0,
                    deck_index: row.get(9)?,
                    combi_rank: row.get(10)?,
                    capture_rate: row.get(11)?,
                    price: row.get(12)?,
                    food_amount: row.get(13)?,
                    max_stomach: row.get(14)?,
                    male_probability: row.get(15)?,
                    run_speed: row.get(16)?,
                    ride_speed: row.get(17)?,
                    scaling: ScalingRef {
                        hp: row.get(18)?,
                        attack: row.get(19)?,
                        defense: row.get(20)?,
                    },
                    elements: Vec::new(),
                    work: BTreeMap::new(),
                    moves: Vec::new(),
                    partner_skill: None,
                    farm_drops: Vec::new(),
                },
            ))
        })? {
            let (code, mut value) = row?;
            value.elements = species_elements.remove(&code).unwrap_or_default();
            value.work = species_work.remove(&code).unwrap_or_default();
            value.moves = species_moves.remove(&code).unwrap_or_default();
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
            elements,
            schema,
        })
    }
}

pub struct UserDatabase {
    connection: Connection,
}

impl UserDatabase {
    /// Open the user DB, creating schema v1 when the file does not yet exist.
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
        reference: &ReferenceDatabase,
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
        reference.validate_passive_codes(passive_codes)?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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
        assert_eq!(bundle.moves.len(), 351);
        assert_eq!(bundle.passives.len(), 420);
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
        let path = unique_user_path();
        let mut user = UserDatabase::open_or_create(&path).unwrap();
        let preset = user
            .save_preset(&reference, None, "Worker", &codes[..4])
            .unwrap();
        assert_eq!(preset.passive_codes, codes[..4]);
        assert!(user
            .save_preset(&reference, Some(preset.id), "Too many", &codes)
            .is_err());
        assert_eq!(user.list_presets().unwrap().len(), 1);
        drop(user);
        fs::remove_file(path).unwrap();
    }
}

PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
    version     INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
) STRICT;

CREATE TABLE metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

CREATE TABLE data_source (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    kind         TEXT NOT NULL,
    url          TEXT,
    version      TEXT,
    revision     TEXT,
    retrieved_at TEXT,
    sha256       TEXT,
    notes        TEXT
) STRICT;

CREATE TABLE element (
    code       TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    color      TEXT,
    icon       TEXT,
    sort_order INTEGER NOT NULL UNIQUE
) STRICT;

CREATE TABLE species (
    code                       TEXT PRIMARY KEY,
    name                       TEXT NOT NULL,
    tribe                      TEXT,
    paldeck_index              INTEGER NOT NULL,
    category                   TEXT NOT NULL CHECK (category IN ('Natural', 'TowerBoss', 'Unobtainable')),
    size                       TEXT,
    rarity                     INTEGER NOT NULL,
    genus                      TEXT,
    weapon                     TEXT,
    weapon_equip               INTEGER NOT NULL CHECK (weapon_equip IN (0, 1)),
    hp_scaling                 INTEGER NOT NULL,
    attack_scaling             INTEGER NOT NULL,
    defense_scaling            INTEGER NOT NULL,
    friendship_hp              REAL NOT NULL,
    friendship_attack          REAL NOT NULL,
    friendship_defense         REAL NOT NULL,
    friendship_craft_speed     REAL NOT NULL,
    enemy_max_hp_rate          REAL NOT NULL,
    enemy_receive_damage_rate  REAL NOT NULL,
    enemy_inflict_damage_rate  REAL NOT NULL,
    capture_rate               REAL NOT NULL,
    exp_ratio                  REAL NOT NULL,
    price                      REAL NOT NULL,
    slow_walk_speed            INTEGER NOT NULL,
    walk_speed                 INTEGER NOT NULL,
    run_speed                  INTEGER NOT NULL,
    ride_sprint_speed          INTEGER NOT NULL,
    transport_speed            INTEGER NOT NULL,
    is_alpha_species           INTEGER NOT NULL CHECK (is_alpha_species IN (0, 1)),
    is_tower_boss              INTEGER NOT NULL CHECK (is_tower_boss IN (0, 1)),
    is_raid_boss               INTEGER NOT NULL CHECK (is_raid_boss IN (0, 1)),
    is_predator                INTEGER NOT NULL CHECK (is_predator IN (0, 1)),
    nocturnal                  INTEGER NOT NULL CHECK (nocturnal IN (0, 1)),
    edible                     INTEGER NOT NULL CHECK (edible IN (0, 1)),
    max_stomach                INTEGER NOT NULL,
    food_amount                INTEGER NOT NULL,
    biological_grade           INTEGER NOT NULL,
    stamina                    INTEGER NOT NULL,
    male_probability           REAL NOT NULL,
    breeding_rank              INTEGER NOT NULL,
    disabled                   INTEGER NOT NULL CHECK (disabled IN (0, 1)),
    palbox_selectable          INTEGER NOT NULL CHECK (palbox_selectable IN (0, 1)),
    icon                       TEXT,
    source_id                  INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE INDEX species_name_idx ON species(name COLLATE NOCASE);
CREATE INDEX species_paldeck_idx ON species(paldeck_index);
CREATE INDEX species_category_idx ON species(category);
CREATE INDEX species_breeding_rank_idx ON species(breeding_rank);
CREATE INDEX species_palbox_selectable_idx ON species(palbox_selectable);

CREATE TABLE species_alias (
    alias_code      TEXT PRIMARY KEY REFERENCES species(code) ON DELETE CASCADE,
    canonical_code  TEXT NOT NULL REFERENCES species(code) ON DELETE CASCADE,
    reason          TEXT NOT NULL,
    CHECK (alias_code <> canonical_code)
) STRICT;

CREATE INDEX species_alias_canonical_idx ON species_alias(canonical_code);

CREATE TABLE species_element (
    species_code TEXT NOT NULL REFERENCES species(code) ON DELETE CASCADE,
    element_code TEXT NOT NULL REFERENCES element(code),
    position     INTEGER NOT NULL CHECK (position BETWEEN 0 AND 1),
    PRIMARY KEY (species_code, position),
    UNIQUE (species_code, element_code)
) STRICT;

CREATE INDEX species_element_element_idx ON species_element(element_code);

CREATE TABLE work_type (
    code       TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    icon       TEXT,
    sort_order INTEGER NOT NULL UNIQUE
) STRICT;

CREATE TABLE species_work (
    species_code TEXT NOT NULL REFERENCES species(code) ON DELETE CASCADE,
    work_code    TEXT NOT NULL REFERENCES work_type(code),
    base_level   INTEGER NOT NULL CHECK (base_level BETWEEN 0 AND 10),
    PRIMARY KEY (species_code, work_code)
) STRICT;

CREATE INDEX species_work_filter_idx ON species_work(work_code, base_level);

CREATE TABLE move (
    code        TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    element_code TEXT REFERENCES element(code),
    category    TEXT,
    power       INTEGER NOT NULL,
    min_range   INTEGER,
    max_range   INTEGER,
    cooldown    REAL,
    synthesized INTEGER NOT NULL DEFAULT 0 CHECK (synthesized IN (0, 1)),
    source_id   INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE INDEX move_name_idx ON move(name COLLATE NOCASE);
CREATE INDEX move_element_idx ON move(element_code);
CREATE INDEX move_power_idx ON move(power);

CREATE TABLE move_effect (
    move_code TEXT NOT NULL REFERENCES move(code) ON DELETE CASCADE,
    position  INTEGER NOT NULL,
    type      TEXT NOT NULL,
    value     REAL,
    value_ex  REAL,
    PRIMARY KEY (move_code, position)
) STRICT;

CREATE TABLE species_move (
    species_code TEXT NOT NULL REFERENCES species(code) ON DELETE CASCADE,
    move_code    TEXT NOT NULL REFERENCES move(code),
    unlock_level INTEGER NOT NULL CHECK (unlock_level >= 0),
    source_type  TEXT NOT NULL CHECK (source_type IN ('level', 'innate', 'raid', 'other')),
    PRIMARY KEY (species_code, move_code, source_type)
) STRICT;

CREATE INDEX species_move_move_idx ON species_move(move_code);
CREATE INDEX species_move_level_idx ON species_move(species_code, unlock_level);

CREATE TABLE passive (
    code                 TEXT PRIMARY KEY,
    name                 TEXT NOT NULL,
    description          TEXT NOT NULL,
    rating               INTEGER NOT NULL CHECK (rating BETWEEN -3 AND 5),
    invoke_active_party  INTEGER NOT NULL CHECK (invoke_active_party IN (0, 1)),
    invoke_worker        INTEGER NOT NULL CHECK (invoke_worker IN (0, 1)),
    invoke_riding        INTEGER NOT NULL CHECK (invoke_riding IN (0, 1)),
    invoke_reserve       INTEGER NOT NULL CHECK (invoke_reserve IN (0, 1)),
    invoke_in_party      INTEGER NOT NULL CHECK (invoke_in_party IN (0, 1)),
    invoke_always        INTEGER NOT NULL CHECK (invoke_always IN (0, 1)),
    invoke_in_base       INTEGER NOT NULL CHECK (invoke_in_base IN (0, 1)),
    available_normal_pal INTEGER NOT NULL CHECK (available_normal_pal IN (0, 1)),
    available_lucky_pal  INTEGER NOT NULL CHECK (available_lucky_pal IN (0, 1)),
    available_shot_weapon INTEGER NOT NULL CHECK (available_shot_weapon IN (0, 1)),
    available_melee_weapon INTEGER NOT NULL CHECK (available_melee_weapon IN (0, 1)),
    available_armor      INTEGER NOT NULL CHECK (available_armor IN (0, 1)),
    available_accessory  INTEGER NOT NULL CHECK (available_accessory IN (0, 1)),
    disabled             INTEGER NOT NULL CHECK (disabled IN (0, 1)),
    source_id            INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE INDEX passive_name_idx ON passive(name COLLATE NOCASE);
CREATE INDEX passive_rating_idx ON passive(rating);
CREATE INDEX passive_rollable_idx ON passive(available_normal_pal, disabled);

CREATE TABLE passive_effect (
    passive_code TEXT NOT NULL REFERENCES passive(code) ON DELETE CASCADE,
    position     INTEGER NOT NULL,
    type         TEXT NOT NULL,
    value        REAL,
    target       TEXT,
    PRIMARY KEY (passive_code, position)
) STRICT;

CREATE TABLE species_passive (
    species_code TEXT NOT NULL REFERENCES species(code) ON DELETE CASCADE,
    passive_code TEXT NOT NULL REFERENCES passive(code),
    relationship TEXT NOT NULL CHECK (relationship IN ('innate', 'exclusive', 'other')),
    PRIMARY KEY (species_code, passive_code, relationship)
) STRICT;

CREATE INDEX species_passive_passive_idx ON species_passive(passive_code);

CREATE TABLE item (
    code            TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    item_group      TEXT,
    type_a          TEXT,
    type_b          TEXT,
    rank            INTEGER,
    rarity          INTEGER,
    max_stack_count INTEGER,
    weight          REAL,
    price           INTEGER,
    sort_id         INTEGER,
    icon            TEXT,
    disabled        INTEGER NOT NULL CHECK (disabled IN (0, 1)),
    source_id       INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE INDEX item_name_idx ON item(name COLLATE NOCASE);
CREATE INDEX item_type_idx ON item(type_a, type_b);

CREATE TABLE partner_skill (
    species_code     TEXT PRIMARY KEY REFERENCES species(code) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    description      TEXT NOT NULL,
    category         TEXT,
    element_code     TEXT REFERENCES element(code),
    gear_item_slug   TEXT,
    gear_name        TEXT,
    technology_level INTEGER,
    source_id        INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE INDEX partner_skill_name_idx ON partner_skill(name COLLATE NOCASE);
CREATE INDEX partner_skill_category_idx ON partner_skill(category);

CREATE TABLE partner_skill_rank (
    species_code TEXT NOT NULL REFERENCES partner_skill(species_code) ON DELETE CASCADE,
    rank         INTEGER NOT NULL CHECK (rank BETWEEN 1 AND 5),
    value_text   TEXT NOT NULL,
    value_number REAL,
    PRIMARY KEY (species_code, rank)
) STRICT;

CREATE TABLE ranch_drop (
    species_code TEXT NOT NULL REFERENCES species(code) ON DELETE CASCADE,
    position     INTEGER NOT NULL,
    item_code    TEXT REFERENCES item(code),
    item_name    TEXT NOT NULL,
    notes        TEXT,
    source_id    INTEGER NOT NULL REFERENCES data_source(id),
    PRIMARY KEY (species_code, position)
) STRICT;

CREATE INDEX ranch_drop_item_idx ON ranch_drop(item_code);
CREATE INDEX ranch_drop_name_idx ON ranch_drop(item_name COLLATE NOCASE);

CREATE TABLE exp_level (
    level           INTEGER PRIMARY KEY CHECK (level >= 1),
    drop_exp        INTEGER NOT NULL,
    next_exp        INTEGER NOT NULL,
    pal_next_exp    INTEGER NOT NULL,
    total_exp       INTEGER NOT NULL,
    pal_total_exp   INTEGER NOT NULL,
    build_exp       INTEGER NOT NULL,
    craft_exp       INTEGER NOT NULL,
    pal_build_exp   INTEGER NOT NULL,
    pal_craft_exp   INTEGER NOT NULL,
    source_id       INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE TABLE friendship_rank (
    code           TEXT PRIMARY KEY,
    rank           INTEGER NOT NULL UNIQUE,
    required_point INTEGER NOT NULL,
    source_id      INTEGER NOT NULL REFERENCES data_source(id)
) STRICT;

CREATE TABLE localization (
    entity_type TEXT NOT NULL,
    entity_code TEXT NOT NULL,
    locale      TEXT NOT NULL,
    name        TEXT,
    description TEXT,
    PRIMARY KEY (entity_type, entity_code, locale)
) STRICT;

CREATE INDEX localization_name_idx ON localization(locale, name COLLATE NOCASE);

CREATE TABLE filter_field (
    key             TEXT PRIMARY KEY,
    label           TEXT NOT NULL,
    value_type      TEXT NOT NULL CHECK (value_type IN ('text', 'enum', 'multi', 'number', 'bool', 'relation')),
    source_relation TEXT NOT NULL,
    source_column   TEXT,
    filterable      INTEGER NOT NULL CHECK (filterable IN (0, 1)),
    displayable     INTEGER NOT NULL CHECK (displayable IN (0, 1)),
    sort_order      INTEGER NOT NULL UNIQUE,
    operator_hint   TEXT,
    notes           TEXT
) STRICT;

CREATE TABLE filter_option (
    field_key  TEXT NOT NULL REFERENCES filter_field(key) ON DELETE CASCADE,
    value      TEXT NOT NULL,
    label      TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (field_key, value)
) STRICT;

CREATE TABLE data_quality_issue (
    id          INTEGER PRIMARY KEY,
    severity    TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'error')),
    entity_type TEXT,
    entity_code TEXT,
    field       TEXT,
    message     TEXT NOT NULL,
    source_id   INTEGER REFERENCES data_source(id)
) STRICT;

CREATE VIEW v_species_summary AS
SELECT
    s.code,
    s.name,
    s.paldeck_index,
    s.category,
    s.rarity,
    s.size,
    s.genus,
    s.nocturnal,
    s.disabled,
    s.palbox_selectable,
    s.hp_scaling,
    s.attack_scaling,
    s.defense_scaling,
    s.breeding_rank,
    GROUP_CONCAT(se.element_code, '|') AS elements
FROM species AS s
LEFT JOIN species_element AS se ON se.species_code = s.code
GROUP BY s.code;

INSERT INTO schema_migrations(version, applied_at)
VALUES (2, '2026-07-25');

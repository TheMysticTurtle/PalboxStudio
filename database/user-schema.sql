PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- App-owned preferences belong in the durable user database, not the webview
-- profile. Values are interpreted and validated by palbox-core.
CREATE TABLE IF NOT EXISTS app_setting (
    key        TEXT PRIMARY KEY CHECK (length(trim(key)) BETWEEN 1 AND 80),
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) WITHOUT ROWID, STRICT;

-- Passive presets are user-authored app metadata. Passive codes are validated
-- against the separate read-only reference database by the application layer.
CREATE TABLE IF NOT EXISTS passive_preset (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

-- slot preserves display/apply order. The current maximum comes from the
-- reference DB and is enforced by palbox-core, so game updates do not require
-- reshaping this durable user database.
CREATE TABLE IF NOT EXISTS passive_preset_entry (
    preset_id   INTEGER NOT NULL REFERENCES passive_preset(id) ON DELETE CASCADE,
    slot        INTEGER NOT NULL CHECK (slot >= 0),
    passive_code TEXT NOT NULL CHECK (length(trim(passive_code)) > 0),
    PRIMARY KEY (preset_id, slot),
    UNIQUE (preset_id, passive_code)
) STRICT;

CREATE INDEX IF NOT EXISTS passive_preset_entry_code_idx
ON passive_preset_entry(passive_code);

CREATE TRIGGER IF NOT EXISTS passive_preset_touch_name
AFTER UPDATE OF name ON passive_preset
WHEN NEW.name <> OLD.name
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS passive_preset_touch_updated_at
AFTER INSERT ON passive_preset_entry
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.preset_id;
END;

CREATE TRIGGER IF NOT EXISTS passive_preset_touch_updated_at_update
AFTER UPDATE ON passive_preset_entry
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.preset_id;
END;

CREATE TRIGGER IF NOT EXISTS passive_preset_touch_updated_at_delete
AFTER DELETE ON passive_preset_entry
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = OLD.preset_id;
END;

-- User-authored groups are app metadata keyed to a Pal's stable InstanceId.
-- Membership never enters GlobalPalStorage.sav, and one Pal may belong to
-- any number of groups.
CREATE TABLE IF NOT EXISTS pal_group (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE IF NOT EXISTS pal_group_member (
    instance_id TEXT NOT NULL CHECK (length(trim(instance_id)) BETWEEN 1 AND 128),
    group_id    INTEGER NOT NULL REFERENCES pal_group(id) ON DELETE CASCADE,
    PRIMARY KEY (instance_id, group_id)
) WITHOUT ROWID, STRICT;

CREATE INDEX IF NOT EXISTS pal_group_member_group_idx
ON pal_group_member(group_id, instance_id);

CREATE TRIGGER IF NOT EXISTS pal_group_touch_name
AFTER UPDATE OF name ON pal_group
WHEN NEW.name <> OLD.name
BEGIN
    UPDATE pal_group
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.id;
END;

INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES
    (1, '2026-07-25'),
    (2, '2026-07-25'),
    (3, '2026-07-29'),
    (4, '2026-07-30'),
    (5, '2026-08-02');

INSERT OR IGNORE INTO metadata(key, value) VALUES
    ('database_kind', 'palbox-user'),
    ('schema_version', '5');

INSERT OR IGNORE INTO app_setting(key, value) VALUES
    ('last_box_path', ''),
    ('auto_reopen', '0'),
    ('max_hp', '1'),
    ('max_sanity', '1'),
    ('max_food', '1'),
    ('max_trust', '0');

COMMIT;

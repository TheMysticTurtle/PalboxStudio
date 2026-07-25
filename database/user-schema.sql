PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
    version     INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
) STRICT;

CREATE TABLE metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- Passive presets are user-authored app metadata. Passive codes are validated
-- against the separate read-only reference database by the application layer.
CREATE TABLE passive_preset (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

-- slot is both the display/apply order and the hard database-level limit:
-- only slots 0..3 can exist, so a preset can never contain more than four.
CREATE TABLE passive_preset_entry (
    preset_id   INTEGER NOT NULL REFERENCES passive_preset(id) ON DELETE CASCADE,
    slot        INTEGER NOT NULL CHECK (slot BETWEEN 0 AND 3),
    passive_code TEXT NOT NULL CHECK (length(trim(passive_code)) > 0),
    PRIMARY KEY (preset_id, slot),
    UNIQUE (preset_id, passive_code)
) STRICT;

CREATE INDEX passive_preset_entry_code_idx
ON passive_preset_entry(passive_code);

CREATE TRIGGER passive_preset_touch_name
AFTER UPDATE OF name ON passive_preset
WHEN NEW.name <> OLD.name
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.id;
END;

CREATE TRIGGER passive_preset_touch_updated_at
AFTER INSERT ON passive_preset_entry
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.preset_id;
END;

CREATE TRIGGER passive_preset_touch_updated_at_update
AFTER UPDATE ON passive_preset_entry
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.preset_id;
END;

CREATE TRIGGER passive_preset_touch_updated_at_delete
AFTER DELETE ON passive_preset_entry
BEGIN
    UPDATE passive_preset
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = OLD.preset_id;
END;

INSERT INTO schema_migrations(version, applied_at)
VALUES (1, '2026-07-25');

INSERT INTO metadata(key, value) VALUES
    ('database_kind', 'palbox-user'),
    ('schema_version', '1');

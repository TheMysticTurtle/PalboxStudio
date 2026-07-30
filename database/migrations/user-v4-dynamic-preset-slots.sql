PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

-- The number of Pal passive slots is patch-sensitive reference data. The user
-- database preserves ordered preset entries but deliberately does not duplicate
-- the current game limit; palbox-core validates it against editor_limits.
DROP TRIGGER IF EXISTS passive_preset_touch_updated_at;
DROP TRIGGER IF EXISTS passive_preset_touch_updated_at_update;
DROP TRIGGER IF EXISTS passive_preset_touch_updated_at_delete;
DROP INDEX IF EXISTS passive_preset_entry_code_idx;

ALTER TABLE passive_preset_entry RENAME TO passive_preset_entry_v3;

CREATE TABLE passive_preset_entry (
    preset_id    INTEGER NOT NULL REFERENCES passive_preset(id) ON DELETE CASCADE,
    slot         INTEGER NOT NULL CHECK (slot >= 0),
    passive_code TEXT NOT NULL CHECK (length(trim(passive_code)) > 0),
    PRIMARY KEY (preset_id, slot),
    UNIQUE (preset_id, passive_code)
) STRICT;

INSERT INTO passive_preset_entry(preset_id, slot, passive_code)
SELECT preset_id, slot, passive_code
FROM passive_preset_entry_v3
ORDER BY preset_id, slot;

DROP TABLE passive_preset_entry_v3;

CREATE INDEX passive_preset_entry_code_idx
ON passive_preset_entry(passive_code);

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

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (4, '2026-07-30');

UPDATE metadata SET value = '4' WHERE key = 'schema_version';
DELETE FROM metadata WHERE key = 'preset_passive_limit';

COMMIT;

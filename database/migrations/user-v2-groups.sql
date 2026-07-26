PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

CREATE TABLE pal_group (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE
               CHECK (length(trim(name)) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE pal_group_member (
    instance_id TEXT NOT NULL CHECK (length(trim(instance_id)) BETWEEN 1 AND 128),
    group_id    INTEGER NOT NULL REFERENCES pal_group(id) ON DELETE CASCADE,
    PRIMARY KEY (instance_id, group_id)
) WITHOUT ROWID, STRICT;

CREATE INDEX pal_group_member_group_idx
ON pal_group_member(group_id, instance_id);

CREATE TRIGGER pal_group_touch_name
AFTER UPDATE OF name ON pal_group
WHEN NEW.name <> OLD.name
BEGIN
    UPDATE pal_group
       SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
     WHERE id = NEW.id;
END;

INSERT INTO schema_migrations(version, applied_at)
VALUES (2, '2026-07-25');

UPDATE metadata SET value = '2' WHERE key = 'schema_version';

COMMIT;

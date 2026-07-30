PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

CREATE TABLE IF NOT EXISTS app_setting (
    key        TEXT PRIMARY KEY CHECK (length(trim(key)) BETWEEN 1 AND 80),
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) WITHOUT ROWID, STRICT;

INSERT OR IGNORE INTO app_setting(key, value) VALUES
    ('last_box_path', ''),
    ('auto_reopen', '0');

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (3, '2026-07-29');

UPDATE metadata SET value = '3' WHERE key = 'schema_version';

COMMIT;

PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

INSERT OR IGNORE INTO app_setting(key, value) VALUES
    ('max_hp', '1'),
    ('max_sanity', '1'),
    ('max_food', '1'),
    ('max_trust', '0');

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (5, '2026-08-02');

UPDATE metadata SET value = '5' WHERE key = 'schema_version';

COMMIT;

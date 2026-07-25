# Retained reference sources

These files are evidence for static facts that are not present in the local
Palworld Save Pal game-data dump. They are deliberately committed instead of
being treated as temporary scrape output, so a database build can always be
reproduced and reviewed without relying on a live website.

## 2026-07-25 snapshots

| File | Source | Purpose |
| --- | --- | --- |
| `palworld-tools-partner-skills-2026-07-25.html` | <https://www.palworld.tools/partner-skills> | Exact source page for Partner Skill names, descriptions, categories, gear, technology levels, and rank values. The page identifies itself as Palworld 1.0 build `24088745`, updated July 13, 2026. |
| `palworld-tools-partner-skills-2026-07-25.parsed.json` | Derived by `scripts/scrape_partner_skills.py` | Reviewable normalized form of the 287 Partner Skill cards. |
| `palworld-wiki-farming-table-2026-07-25.json` | <https://palworld.wiki.gg/wiki/Template:Farming_Table> through the MediaWiki API | Exact source wikitext for Ranch product relationships. |
| `palworld-wiki-farming-table-2026-07-25.parsed.json` | Derived by `scripts/scrape_ranch_drops.py` | Reviewable normalized form of the Ranch table. |
| `palworld-wiki-ranch-2026-07-25.json` | <https://palworld.wiki.gg/wiki/Ranch> through the MediaWiki API | Ranch page snapshot retained for surrounding mechanics/context. |

Each parsed file records the SHA-256 digest of its raw input. The database
builder verifies that digest before importing any derived records.

## Authority rules

1. The local `oMaN-Rod/palworld-save-pal` 1.0 dump is authoritative for
   internal codes and every static field it exposes.
2. `palworld.tools` supplies Partner Skill facts absent from that dump.
3. Palworld Wiki supplies Ranch product relationships absent from the dump.
4. External rows must resolve to a current game-data code. Ambiguous, stale, or
   unmatched rows are skipped and written to `data_quality_issue`.
5. Current game localization wins when an external source uses an older item
   label. The source label remains reviewable in the retained snapshot.

To refresh these sources, save the raw responses here first, run the matching
scraper, review the diff, and then rebuild the databases.

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

## Project-verified engine rules

`palbox-verified-engine-rules-2026-07-30.json` is the reviewable build input for
patch-sensitive editing limits, calculation coefficients, and the bundled Work
Suitability asset identities. These values are loaded into the generated
reference database; neither the Rust engine nor a frontend carries a second
runtime copy. Replace values in this source only after updating the verification
notes in `SPECS-1.0.md` / `SAVE-FORMAT.md` or after a future game-data extractor
can supply them directly.

## Authority rules

1. The local `oMaN-Rod/palworld-save-pal` 1.0 dump is authoritative for
   internal codes and every static field it exposes.
2. `palworld.tools` supplies Partner Skill facts absent from that dump.
3. Palworld Wiki supplies Ranch product relationships absent from the dump.
4. External rows must resolve to a current game-data code. Ambiguous, stale, or
   unmatched rows are skipped and written to `data_quality_issue`.
5. Current game localization wins when an external source uses an older item
   label. The source label remains reviewable in the retained snapshot.
6. Project-verified rules supplement facts not exposed by the current game-data
   dump and remain explicit, reviewable inputs to the generated database.

To refresh these sources, save the raw responses here first, run the matching
scraper, review the diff, and then rebuild the databases.

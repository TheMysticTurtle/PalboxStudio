#!/usr/bin/env python3
"""Normalize Palworld Wiki's 1.0 Ranch table into a reviewable JSON snapshot.

The raw MediaWiki API response is kept in data/reference-sources. This script
extracts only the table's factual Pal/item relationships; build_reference_db.py
then cross-checks them against both the local 1.0 game dump and the current
Partner Skill descriptions.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


REPO = Path(__file__).resolve().parents[1]
SOURCE_URL = (
    "https://palworld.wiki.gg/api.php?"
    + urllib.parse.urlencode(
        {
            "action": "parse",
            "page": "Template:Farming Table",
            "prop": "wikitext",
            "format": "json",
            "formatversion": "2",
        }
    )
)
DEFAULT_INPUT = REPO / "data" / "reference-sources" / "palworld-wiki-farming-table-2026-07-25.json"
DEFAULT_OUTPUT = (
    REPO / "data" / "reference-sources" / "palworld-wiki-farming-table-2026-07-25.parsed.json"
)
ICON_PATTERN = re.compile(r"\{\{i\|([^}|]+)")


def download() -> bytes:
    request = urllib.request.Request(
        SOURCE_URL,
        headers={"User-Agent": "PalboxStudioReferenceBuilder/0.1 (+local data verification)"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def parse_table(raw: bytes) -> list[dict[str, object]]:
    payload = json.loads(raw)
    wikitext = payload["parse"]["wikitext"]
    rows: list[dict[str, object]] = []
    for block in re.split(r"\n\|-\s*\n", wikitext)[1:]:
        cells = [
            line[1:].strip()
            for line in block.splitlines()
            if line.startswith("|") and not line.startswith("|}")
        ]
        if len(cells) < 4:
            continue
        pal_cell, paldeck_no, drop_cell, food_cell = cells[:4]
        pal_match = ICON_PATTERN.search(pal_cell)
        if pal_match is None:
            continue
        item_names = ICON_PATTERN.findall(drop_cell)
        if not item_names:
            continue
        rows.append(
            {
                "species_name": pal_match.group(1).strip(),
                "paldeck_no": paldeck_no.strip(),
                "item_names": [name.strip() for name in item_names],
                "drop_cell": drop_cell,
                "food": int(food_cell) if food_cell.isdigit() else None,
                "notes": " ".join(re.findall(r"<ref>(.*?)</ref>", pal_cell)),
            }
        )
    if len(rows) < 20:
        raise ValueError(f"unexpectedly small Farming Table: {len(rows)} rows")
    return rows


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument(
        "--download",
        action="store_true",
        help="Download the live API response instead of using --input",
    )
    args = parser.parse_args()

    normalized_at = datetime.now(timezone.utc).isoformat(timespec="seconds")
    if args.download:
        raw = download()
        retrieved_at = normalized_at
    else:
        raw = args.input.read_bytes()
        retrieved_at = datetime.fromtimestamp(
            args.input.stat().st_mtime, timezone.utc
        ).isoformat(timespec="seconds")
    rows = parse_table(raw)
    payload = {
        "_meta": {
            "source_url": SOURCE_URL,
            "retrieved_at": retrieved_at,
            "normalized_at": normalized_at,
            "raw_sha256": hashlib.sha256(raw).hexdigest(),
            "claimed_game_version": "Palworld 1.0",
            "record_count": len(rows),
            "generator": "scripts/scrape_ranch_drops.py",
        },
        "ranch_rows": rows,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"ranch rows: {len(rows)}")
    print(f"output: {args.output}")


if __name__ == "__main__":
    main()

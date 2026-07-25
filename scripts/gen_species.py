#!/usr/bin/env python3
"""Generate the Palbox Studio reference dataset (static game data).

Source of truth: the actively-maintained Palworld Save Pal dumps
(`PalEdit/psp-reference/data/json/`) + PalEdit's en-GB display names. This script
is the ONLY way the files under `ui/static/data/` should change — never hand-edit
the output; edit this generator and re-run so every change is reproducible and
reviewable in git.

Emits (each with a `_meta` provenance block):
  species.json   one row per box-storable pal (is_pal == true; humans/NPCs and
                 Quest_ duplicates excluded). The main filter/display table.
  moves.json     move code -> { name, element, power, category }.
  passives.json  passive code -> { name, rating, description }.
  elements.json  element -> { color }.
  schema.json    self-describing column metadata for `species` (drives the UI
                 filters + result columns).

Known gaps (not present in the dumps — need a wiki scrape later): a pal's
Partner Skill (special ability) and ranch/farm drops. Columns are stubbed
(`partnerSkill: null`, `farmDrops: []`) so adding them later is non-breaking.

Run:  python scripts/gen_species.py     (from the PalboxStudio repo root)
"""
from __future__ import annotations
import json, os, datetime, re

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
PSP = os.path.join(REPO, "..", "PalEdit", "psp-reference", "data", "json")
ENGB = os.path.join(REPO, "..", "PalEdit", "palworld_pal_edit", "resources", "data", "en-GB")
OUTDIR = os.path.join(REPO, "ui", "static", "data")

ELEMENT = {"Normal": "Neutral", "Fire": "Fire", "Water": "Water", "Electricity": "Electric",
           "Leaf": "Grass", "Ice": "Ice", "Earth": "Ground", "Dark": "Dark", "Dragon": "Dragon"}
WORK = {"EmitFlame": "Kindling", "Watering": "Watering", "Seeding": "Planting",
        "GenerateElectricity": "Generating Electricity", "Handcraft": "Handiwork",
        "Collection": "Gathering", "Deforest": "Lumbering", "Mining": "Mining",
        "OilExtraction": "Oil Extraction", "ProductMedicine": "Medicine Production",
        "Cool": "Cooling", "Transport": "Transporting", "MonsterFarm": "Farming"}
WAZA = "EPalWazaID::"

SCHEMA = [
    {"key": "name", "label": "Name", "type": "text", "filterable": True, "displayable": True},
    {"key": "elements", "label": "Element", "type": "multi", "filterable": True, "displayable": True},
    {"key": "category", "label": "Category", "type": "enum", "filterable": True, "displayable": True,
     "values": ["Natural", "TowerBoss", "Unobtainable"]},
    {"key": "rarity", "label": "Rarity", "type": "number", "filterable": True, "displayable": True},
    {"key": "size", "label": "Size", "type": "enum", "filterable": True, "displayable": True},
    {"key": "genus", "label": "Genus", "type": "enum", "filterable": True, "displayable": True},
    {"key": "nocturnal", "label": "Nocturnal", "type": "bool", "filterable": True, "displayable": True},
    {"key": "work", "label": "Work Suitability", "type": "map", "filterable": True, "displayable": True},
    {"key": "moves", "label": "Learnset", "type": "multi", "filterable": True, "displayable": False},
    {"key": "deckIndex", "label": "Paldeck No.", "type": "number", "filterable": False, "displayable": True},
    {"key": "combiRank", "label": "Breeding Rank", "type": "number", "filterable": True, "displayable": True},
    {"key": "scaling", "label": "Base Scaling", "type": "map", "filterable": False, "displayable": True},
    {"key": "captureRate", "label": "Capture Rate", "type": "number", "filterable": False, "displayable": True},
    {"key": "price", "label": "Gold Value", "type": "number", "filterable": False, "displayable": True},
    {"key": "partnerSkill", "label": "Partner Skill", "type": "text", "filterable": False, "displayable": True},
    {"key": "farmDrops", "label": "Ranch Drops", "type": "multi", "filterable": True, "displayable": True},
]


def load(p):
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def write(name, meta_extra, body):
    meta = {"generated": datetime.date.today().isoformat(),
            "source": "oMaN-Rod/palworld-save-pal dumps + PalEdit en-GB names",
            "generator": "scripts/gen_species.py",
            "note": "Do not hand-edit; re-run the generator."}
    meta.update(meta_extra)
    payload = {"_meta": meta}
    payload.update(body)
    os.makedirs(OUTDIR, exist_ok=True)
    with open(os.path.join(OUTDIR, name), "w", encoding="utf-8") as f:
        json.dump(payload, f, ensure_ascii=True, indent=1)


def prettify(code):
    return re.sub(r"(?<=[a-z])(?=[A-Z])", " ", code).replace("_", " ").strip()


def category(code, r):
    if code.startswith("GYM_") or r.get("is_tower_boss"):
        return "TowerBoss"
    if code.startswith(("RAID_", "SUMMON_", "PREDATOR_")) or r.get("is_raid_boss") or r.get("predator"):
        return "Unobtainable"
    if (r.get("pal_deck_index", -1) or -1) >= 0:
        return "Natural"
    return "Unobtainable"


def main():
    pals = load(os.path.join(PSP, "pals.json"))
    names = load(os.path.join(ENGB, "pals.json"))
    psp_moves = load(os.path.join(PSP, "active_skills.json"))
    psp_pass = load(os.path.join(PSP, "passive_skills.json"))
    psp_el = load(os.path.join(PSP, "elements.json"))
    en_atk = load(os.path.join(ENGB, "attacks.json"))
    en_pass = load(os.path.join(ENGB, "passives.json"))

    # --- elements lookup ---
    elements = {ELEMENT.get(k, k): {"color": v.get("color", "")} for k, v in psp_el.items()}

    # --- moves lookup (keyed by bare code, matching a species' skill_set) ---
    moves = {}
    for full, r in psp_moves.items():
        bare = full[len(WAZA):] if full.startswith(WAZA) else full
        name = en_atk.get(full) or en_atk.get(bare) or prettify(bare)
        moves[bare] = {"name": name, "element": ELEMENT.get(r.get("element", ""), r.get("element", "")),
                       "power": r.get("power", 0), "category": r.get("type", "")}

    # --- passives lookup ---
    passives = {}
    for code, r in psp_pass.items():
        en = en_pass.get(code, {})
        passives[code] = {"name": en.get("Name") or prettify(code),
                          "rating": r.get("rank", 0),
                          "description": (en.get("Description") or "").strip()}

    # --- species table ---
    rows, skipped_human, skipped_dupe = [], 0, 0
    for code, r in pals.items():
        if not r.get("is_pal"):
            skipped_human += 1
            continue
        if code.startswith("Quest_"):  # duplicate quest instances of a base pal
            skipped_dupe += 1
            continue
        sc = r.get("scaling", {}) or {}
        work = {k: v for k, v in (r.get("work_suitability", {}) or {}).items()}
        rows.append({
            "code": code,
            "name": names.get(code, code),
            "elements": [ELEMENT.get(e, e) for e in r.get("element_types", []) if e and e != "None"],
            "category": category(code, r),
            "disabled": bool(r.get("disabled")),
            "rarity": r.get("rarity", 0),
            "size": r.get("size", ""),
            "genus": r.get("genus_category", ""),
            "nocturnal": bool(r.get("nocturnal")),
            "alpha": bool(r.get("is_boss")),
            "deckIndex": r.get("pal_deck_index", -1),
            "combiRank": r.get("combi_rank", 0),
            "captureRate": r.get("capture_rate_correct", 0),
            "price": r.get("price", 0),
            "foodAmount": r.get("food_amount", 0),
            "maxStomach": r.get("max_full_stomach", 0),
            "maleProbability": r.get("male_probability", 50),
            "runSpeed": r.get("run_speed", 0),
            "rideSpeed": r.get("ride_sprint_speed", 0),
            "scaling": {"hp": sc.get("hp", 0), "attack": sc.get("attack", 0), "defense": sc.get("defense", 0)},
            "work": {WORK[k]: work.get(k, 0) for k in WORK},
            "moves": list((r.get("skill_set", {}) or {}).keys()),
            "partnerSkill": None,  # GAP: not in dumps; needs a wiki scrape
            "farmDrops": [],       # GAP: not in dumps; needs a wiki scrape
        })
    rows.sort(key=lambda x: (x["deckIndex"] if x["deckIndex"] >= 0 else 99999, x["name"]))

    # Referential integrity: synthesize entries for boss/gym-only unique moves that a
    # species lists but the standard skill DB doesn't define, so no learnset code dangles.
    for m in {mv for r in rows for mv in r["moves"]}:
        if m not in moves:
            nm = re.sub(r"^Unique_[A-Za-z0-9]+_", "", m)
            moves[m] = {"name": en_atk.get(WAZA + m) or prettify(nm), "element": "",
                        "power": 0, "category": "Unique"}

    # --- validation ---
    dangling_moves = sorted({m for r in rows for m in r["moves"] if m not in moves})
    unknown_elements = sorted({e for r in rows for e in r["elements"] if e not in elements})

    # --- write ---
    cats = {}
    for r in rows:
        cats[r["category"]] = cats.get(r["category"], 0) + 1
    write("elements.json", {"count": len(elements)}, {"elements": elements})
    write("moves.json", {"count": len(moves)}, {"moves": moves})
    write("passives.json", {"count": len(passives)}, {"passives": passives})
    write("schema.json", {"of": "species"}, {"schema": SCHEMA})
    write("species.json", {"count": len(rows), "byCategory": cats,
                           "note": "is_pal == true only; humans/NPCs + Quest_ dupes excluded."},
          {"species": rows})

    print(f"species: {len(rows)}  (excluded humans {skipped_human}, quest-dupes {skipped_dupe})")
    print(f"  byCategory: {cats}")
    print(f"moves: {len(moves)}  passives: {len(passives)}  elements: {len(elements)}")
    print(f"VALIDATION  dangling moves: {len(dangling_moves)}  unknown elements: {len(unknown_elements)}")
    if dangling_moves:
        print("  dangling move codes:", dangling_moves[:12], "..." if len(dangling_moves) > 12 else "")
    if unknown_elements:
        print("  unknown elements:", unknown_elements)


if __name__ == "__main__":
    main()

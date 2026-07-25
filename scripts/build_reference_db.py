#!/usr/bin/env python3
"""Build Palbox Studio's static reference and user-template SQLite databases.

Static game facts come from the current local Palworld Save Pal data dump.
Retained web snapshots supplement only facts that dump does not contain:
Partner Skills and Ranch products. Per-Pal state never belongs in either DB.

Run from the repository root:
    python scripts/build_reference_db.py
    python scripts/build_reference_db.py --check
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sqlite3
import tempfile
from pathlib import Path
from typing import Any, Iterable


REPO = Path(__file__).resolve().parents[1]
PSP_REPO = (REPO / ".." / "PalEdit" / "psp-reference").resolve()
PSP_JSON = PSP_REPO / "data" / "json"
SOURCE_DIR = REPO / "data" / "reference-sources"
REFERENCE_SCHEMA = REPO / "database" / "reference-schema.sql"
USER_SCHEMA = REPO / "database" / "user-schema.sql"
DEFAULT_REFERENCE_DB = REPO / "data" / "palbox-reference.db"
DEFAULT_USER_DB = REPO / "data" / "palbox-user.template.db"
WAZA_PREFIX = "EPalWazaID::"

ELEMENT_CODES = {
    "Normal": "Neutral",
    "Fire": "Fire",
    "Water": "Water",
    "Electricity": "Electric",
    "Leaf": "Grass",
    "Ice": "Ice",
    "Earth": "Ground",
    "Dark": "Dark",
    "Dragon": "Dragon",
    "None": None,
    "": None,
    None: None,
}

WORK_ORDER = [
    "EmitFlame",
    "Watering",
    "Seeding",
    "GenerateElectricity",
    "Handcraft",
    "Collection",
    "Deforest",
    "Mining",
    "OilExtraction",
    "ProductMedicine",
    "Cool",
    "Transport",
    "MonsterFarm",
]

# Explicit mappings are safer than display-name guessing where the current
# game has aliases or multiple items with the same localized name.
RANCH_ITEM_CODES = {
    "Wool": "Wool",
    "Egg": "Egg",
    "Arrow": "Arrow",
    "Bone": "Bone",
    "Gold Coin": "Money",
    "Pal Sphere": "PalSphere",
    "Mega Sphere": "PalSphere_Mega",
    "Giga Sphere": "PalSphere_Giga",
    "Hyper Sphere": "PalSphere_Tera",
    "Venom Gland": "Venom",
    "Flame Organ": "FireOrgan",
    "Ice Organ": "IceOrgan",
    "Red Berries": "Berries",
    "Cotton Candy": "Sweet",
    "Caramel Cotton Candy": "Sweet_Caramel",
    "Milk": "Milk",
    "Electric Organ": "ElectricOrgan",
    "Pal Fluids": "PalFluid",
    "Honey": "Honey",
    "Tomato Seeds": "TomatoSeeds",
    "Wheat Seeds": "WheatSeeds",
    "Potato Seeds": "PotatoSeeds",
    "Carrot Seeds": "CarrotSeeds",
    "Onion Seeds": "OnionSeeds",
    "Lettuce Seeds": "LettuceSeeds",
    "Leather": "Leather",
    "High Quality Cloth": "Cloth2",
    "High Quality Pal Oil": "PalOil",
    "Mushroom": "Mushroom",
    "Cavern Mushroom": "CaveMushroom",
}


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def bundle_sha256(paths: Iterable[Path]) -> str:
    digest = hashlib.sha256()
    for path in sorted(paths, key=lambda value: value.as_posix()):
        relative = path.relative_to(PSP_REPO).as_posix().encode()
        digest.update(len(relative).to_bytes(4, "big"))
        digest.update(relative)
        contents = path.read_bytes()
        digest.update(len(contents).to_bytes(8, "big"))
        digest.update(contents)
    return digest.hexdigest()


def git_revision(repo: Path) -> str | None:
    head = (repo / ".git" / "HEAD").read_text(encoding="utf-8").strip()
    if head.startswith("ref: "):
        ref_path = repo / ".git" / head[5:]
        if ref_path.exists():
            return ref_path.read_text(encoding="utf-8").strip()
        packed = repo / ".git" / "packed-refs"
        if packed.exists():
            for line in packed.read_text(encoding="utf-8").splitlines():
                if line and not line.startswith("#") and line.endswith(" " + head[5:]):
                    return line.split(" ", 1)[0]
        return None
    return head


def psp_version() -> str:
    workspace = (PSP_REPO / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'(?m)^version\s*=\s*"([^"]+)"', workspace)
    return match.group(1) if match else "unknown"


def latest(pattern: str) -> Path:
    matches = sorted(SOURCE_DIR.glob(pattern))
    if not matches:
        raise FileNotFoundError(f"no retained source matches {pattern!r}")
    return matches[-1]


def prettify(code: str) -> str:
    value = code.removeprefix(WAZA_PREFIX).replace("_", " ")
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", " ", value).strip()


def synthesized_move_name(code: str) -> str:
    # Unique move codes conventionally prefix the owning species. Removing that
    # first owner token produces the same readable fallback used by the legacy
    # JSON generator while preserving remaining qualifiers such as GYM or Green.
    without_owner = re.sub(r"^Unique_[A-Za-z0-9]+_", "", code)
    return prettify(without_owner)


def format_effect_value(value: Any) -> str:
    number = as_float(value)
    if number.is_integer():
        return str(abs(int(number)))
    return f"{abs(number):.1f}".rstrip("0").rstrip(".")


def signed_percent(label: str, value: Any) -> str:
    number = as_float(value)
    sign = "+" if number >= 0 else "-"
    return f"{label} {sign}{format_effect_value(number)}%."


def passive_effect_sentence(
    code: str,
    name: str,
    effect: dict[str, Any],
) -> str | None:
    effect_type = str(effect.get("type") or "None")
    value = as_float(effect.get("value"))
    amount = format_effect_value(value)

    direct_percent = {
        "ShotAttack": "Attack",
        "Defense": "Defense",
        "MaxHP": "Max HP",
        "CraftSpeed": "Work speed",
        "MoveSpeed": "Movement speed",
        "MaxInventoryWeight": "Carrying capacity",
        "PalExp_Increase": "Pal experience gained",
        "PalSP_Increase": "Pal stamina",
        "JumpPower_Increase": "Jump power",
        "BreedSpeed": "Breeding speed",
        "PalEggHatchingSpeed": "Egg hatching speed",
        "Logging": "Logging efficiency",
        "Mining": "Mining efficiency",
    }
    if effect_type in direct_percent:
        return signed_percent(direct_percent[effect_type], value)

    if effect_type.startswith("ElementBoost_"):
        raw_element = effect_type.removeprefix("ElementBoost_")
        element = {
            "Normal": "Neutral",
            "Leaf": "Grass",
            "Earth": "Ground",
            "Electricity": "Electric",
        }.get(raw_element, raw_element)
        return signed_percent(f"{element} damage", value)

    if effect_type.startswith("ElementResist_"):
        raw_element = effect_type.removeprefix("ElementResist_")
        element = {
            "Normal": "Neutral",
            "Leaf": "Grass",
            "Earth": "Ground",
            "Electricity": "Electric",
        }.get(raw_element, raw_element)
        return signed_percent(f"{element} resistance", value)

    if effect_type == "FullStomatch_Decrease":
        direction = "faster" if value > 0 else "slower"
        return f"Hunger drains {amount}% {direction}."
    if effect_type == "Sanity_Decrease":
        direction = "faster" if value > 0 else "slower"
        return f"SAN drains {amount}% {direction}."
    if effect_type == "ActiveSkillCoolTime_Decrease":
        direction = "faster" if value > 0 else "slower"
        return f"Active skills recharge {amount}% {direction}."
    if effect_type == "LifeSteal":
        return f"Restores {amount}% of damage dealt as HP."
    if effect_type == "Nocturnal":
        return "Can work through the night."
    if effect_type == "NonKilling":
        return "Attacks cannot reduce an enemy below 1 HP."
    if effect_type == "JumpCount_Increase":
        return f"Adds {amount} midair jump{'s' if abs(value) != 1 else ''}."
    if effect_type == "TemperatureResist_Cold":
        return f"Cold resistance +{amount}."
    if effect_type == "TemperatureResist_Heat":
        return f"Heat resistance +{amount}."
    if effect_type == "ShopBuyPrice_Money_Increase":
        return signed_percent("Shop purchase prices", value)
    if effect_type == "ShopSellPrice_Money_Increase":
        return signed_percent("Shop selling prices", value)
    if effect_type == "CollectItem":
        item = code.removeprefix("CollectItem_").replace("_", " ")
        return f"Collects {item.lower()} while assigned."

    if effect_type != "None":
        return signed_percent(prettify(effect_type), value)

    # Some game effects are exported as `None`; their stable codes still
    # provide enough meaning for a useful plain-English summary.
    if code.startswith("SwimSpeed_"):
        return signed_percent("Swimming speed", value)
    if code.startswith("AirDash_"):
        return f"Adds {amount} aerial dash{'es' if abs(value) != 1 else ''}."
    if code.startswith("CaptureLevel_"):
        return f"Capture strength +{amount}."
    if code.startswith("RideJumpCount_"):
        return f"Adds {amount} mounted jump{'s' if abs(value) != 1 else ''}."
    if code.startswith("WorkSuitabilityAddRank_MonsterFarm"):
        return f"Farming suitability +{amount}."
    if code.startswith("WoodDrop_Boost"):
        return signed_percent("Logging yield", value)
    if code.startswith("StonDrop_Boost"):
        return signed_percent("Mining yield", value)
    if code == "StonWoodDrop_Boost":
        return signed_percent("Logging and mining yield", value)
    if code.startswith("SelfDeathAddItemDrop"):
        return signed_percent("Items dropped when defeated", value)
    if code == "FriendshipPoint_Increase_EquipSkill":
        return signed_percent("Trust gained by party Pals", value)
    if code == "ReloadSpeedUp_Passive":
        return signed_percent("Player reload speed", value)
    if code == "PlayerSP_DecreaseRate_Passive":
        direction = "faster" if value > 0 else "slower"
        return f"Player stamina drains {amount}% {direction}."
    if code == "MutationPal_ExplosionResist":
        return "Immune to explosion damage."
    if code == "MutationPal_Mutant":
        return "Pal and player health regeneration +50%. Immune to poison and burn."
    if code == "MutationPal_Immortal":
        return "Grants the Immortality effect."
    if code == "NightOwl":
        return "Can work through the night."
    if code.startswith("WorldTree_"):
        return "World Tree resources no longer vanish when approached."
    if "Immunity" in name:
        return f"Grants {name.lower()}."
    return None


def plain_passive_description(
    code: str,
    name: str,
    effects: list[dict[str, Any]],
) -> str:
    sentences: list[str] = []
    for effect in effects:
        sentence = passive_effect_sentence(code, name, effect)
        if sentence and sentence not in sentences:
            sentences.append(sentence)
    if not sentences:
        sentences.append(f"Grants the {name} effect.")
    return " ".join(sentences)


def as_int(value: Any, default: int = 0) -> int:
    return default if value is None else int(value)


def as_float(value: Any, default: float = 0.0) -> float:
    return default if value is None else float(value)


def as_bool(value: Any) -> int:
    return 1 if value else 0


def classify_species(code: str, row: dict[str, Any]) -> str:
    if code.startswith("GYM_") or row.get("is_tower_boss"):
        return "TowerBoss"
    if (
        code.startswith(("RAID_", "SUMMON_", "PREDATOR_"))
        or code.endswith("_Oilrig")
        or code == "WorldTreeDragon"
        or row.get("is_raid_boss")
        or row.get("predator")
    ):
        return "Unobtainable"
    if as_int(row.get("pal_deck_index"), -1) >= 0:
        return "Natural"
    return "Unobtainable"


def english_name(localization: dict[str, Any], code: str) -> str:
    value = localization.get(code)
    if isinstance(value, dict):
        return value.get("localized_name") or prettify(code)
    if isinstance(value, str):
        return value
    return prettify(code)


def verify_derived_source(path: Path, raw_path: Path, hash_key: str) -> dict[str, Any]:
    payload = load_json(path)
    expected = payload["_meta"][hash_key].lower()
    actual = sha256_file(raw_path)
    if expected != actual:
        raise ValueError(
            f"{path.name} expects {raw_path.name} SHA-256 {expected}, found {actual}"
        )
    return payload


def add_source(
    connection: sqlite3.Connection,
    *,
    name: str,
    kind: str,
    url: str | None = None,
    version: str | None = None,
    revision: str | None = None,
    retrieved_at: str | None = None,
    sha256: str | None = None,
    notes: str | None = None,
) -> int:
    cursor = connection.execute(
        """
        INSERT INTO data_source(
            name, kind, url, version, revision, retrieved_at, sha256, notes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (name, kind, url, version, revision, retrieved_at, sha256, notes),
    )
    return int(cursor.lastrowid)


def add_issue(
    connection: sqlite3.Connection,
    severity: str,
    message: str,
    *,
    entity_type: str | None = None,
    entity_code: str | None = None,
    field: str | None = None,
    source_id: int | None = None,
) -> None:
    connection.execute(
        """
        INSERT INTO data_quality_issue(
            severity, entity_type, entity_code, field, message, source_id
        ) VALUES (?, ?, ?, ?, ?, ?)
        """,
        (severity, entity_type, entity_code, field, message, source_id),
    )


def create_connection(schema_path: Path, destination: Path) -> tuple[sqlite3.Connection, Path]:
    destination.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temp_name = tempfile.mkstemp(
        prefix=destination.stem + "-", suffix=".tmp", dir=destination.parent
    )
    os.close(descriptor)
    temp_path = Path(temp_name)
    connection = sqlite3.connect(temp_path)
    connection.execute("PRAGMA foreign_keys = ON")
    connection.executescript(schema_path.read_text(encoding="utf-8"))
    return connection, temp_path


def install_database(
    connection: sqlite3.Connection, temp_path: Path, destination: Path
) -> None:
    connection.commit()
    connection.execute("ANALYZE")
    connection.execute("VACUUM")
    connection.close()
    os.replace(temp_path, destination)


def build_reference(destination: Path) -> dict[str, int]:
    required = [
        PSP_JSON / "elements.json",
        PSP_JSON / "pals.json",
        PSP_JSON / "active_skills.json",
        PSP_JSON / "passive_skills.json",
        PSP_JSON / "items.json",
        PSP_JSON / "exp.json",
        PSP_JSON / "friendship.json",
    ]
    required.extend(sorted((PSP_JSON / "l10n").glob("*/*.json")))
    missing = [path for path in required if not path.exists()]
    if missing:
        raise FileNotFoundError(f"missing source files: {missing}")

    partner_parsed = latest("palworld-tools-partner-skills-*.parsed.json")
    partner_raw = SOURCE_DIR / partner_parsed.name.replace(".parsed.json", ".html")
    partner_payload = verify_derived_source(
        partner_parsed, partner_raw, "html_sha256"
    )
    ranch_parsed = latest("palworld-wiki-farming-table-*.parsed.json")
    ranch_raw = SOURCE_DIR / ranch_parsed.name.replace(".parsed.json", ".json")
    ranch_payload = verify_derived_source(ranch_parsed, ranch_raw, "raw_sha256")

    elements = load_json(PSP_JSON / "elements.json")
    pals = load_json(PSP_JSON / "pals.json")
    moves = load_json(PSP_JSON / "active_skills.json")
    passives = load_json(PSP_JSON / "passive_skills.json")
    items = load_json(PSP_JSON / "items.json")
    exp = load_json(PSP_JSON / "exp.json")
    friendship = load_json(PSP_JSON / "friendship.json")
    en_dir = PSP_JSON / "l10n" / "en"
    en_pals = load_json(en_dir / "pals.json")
    en_moves = load_json(en_dir / "active_skills.json")
    en_passives = load_json(en_dir / "passive_skills.json")
    en_items = load_json(en_dir / "items.json")
    en_elements = load_json(en_dir / "elements.json")
    en_work = load_json(en_dir / "work_suitability.json")

    connection, temp_path = create_connection(REFERENCE_SCHEMA, destination)
    try:
        # Tie generated metadata to retained input snapshots, not wall-clock build
        # time, so rebuilding unchanged sources produces the same DB contents.
        generated_at = max(
            partner_payload["_meta"]["retrieved_at"],
            ranch_payload["_meta"]["retrieved_at"],
        )
        connection.executemany(
            "INSERT INTO metadata(key, value) VALUES (?, ?)",
            [
                ("database_kind", "palbox-reference"),
                ("schema_version", "1"),
                ("game_version", "Palworld 1.0"),
                ("generated_at", generated_at),
                ("generator", "scripts/build_reference_db.py"),
                ("per_pal_state_included", "false"),
            ],
        )
        psp_source = add_source(
            connection,
            name="Palworld Save Pal game-data dump",
            kind="game-data-extract",
            url="https://github.com/oMaN-Rod/palworld-save-pal",
            version=psp_version(),
            revision=git_revision(PSP_REPO),
            sha256=bundle_sha256(required),
            notes="Authoritative for internal codes and static fields exposed by the dump.",
        )
        partner_source = add_source(
            connection,
            name="palworld.tools Partner Skills",
            kind="retained-web-snapshot",
            url=partner_payload["_meta"]["source_url"],
            version=partner_payload["_meta"].get("claimed_game_version"),
            revision=str(partner_payload["_meta"].get("claimed_game_build") or ""),
            retrieved_at=partner_payload["_meta"].get("retrieved_at"),
            sha256=partner_payload["_meta"]["html_sha256"],
            notes=f"Raw snapshot: {partner_raw.relative_to(REPO).as_posix()}",
        )
        ranch_source = add_source(
            connection,
            name="Palworld Wiki Farming Table",
            kind="retained-web-snapshot",
            url=ranch_payload["_meta"]["source_url"],
            version=ranch_payload["_meta"].get("claimed_game_version"),
            retrieved_at=ranch_payload["_meta"].get("retrieved_at"),
            sha256=ranch_payload["_meta"]["raw_sha256"],
            notes=f"Raw snapshot: {ranch_raw.relative_to(REPO).as_posix()}",
        )

        for position, (raw_code, row) in enumerate(elements.items()):
            code = ELEMENT_CODES[raw_code]
            localized = en_elements.get(raw_code, {})
            connection.execute(
                """
                INSERT INTO element(code, name, color, icon, sort_order)
                VALUES (?, ?, ?, ?, ?)
                """,
                (
                    code,
                    localized.get("localized_name") or code,
                    row.get("color"),
                    row.get("icon"),
                    position,
                ),
            )

        for position, work_code in enumerate(WORK_ORDER):
            localized = en_work.get(work_code, {})
            connection.execute(
                """
                INSERT INTO work_type(code, name, icon, sort_order)
                VALUES (?, ?, NULL, ?)
                """,
                (
                    work_code,
                    localized.get("localized_name") or prettify(work_code),
                    position,
                ),
            )

        species_rows = {
            code: row
            for code, row in pals.items()
            if row.get("is_pal") and not code.startswith("Quest_")
        }
        for code, row in species_rows.items():
            scaling = row.get("scaling") or {}
            connection.execute(
                """
                INSERT INTO species(
                    code, name, tribe, paldeck_index, category, size, rarity,
                    genus, weapon, weapon_equip, hp_scaling, attack_scaling,
                    defense_scaling, friendship_hp, friendship_attack,
                    friendship_defense, friendship_craft_speed, enemy_max_hp_rate,
                    enemy_receive_damage_rate, enemy_inflict_damage_rate,
                    capture_rate, exp_ratio, price, slow_walk_speed, walk_speed,
                    run_speed, ride_sprint_speed, transport_speed, is_alpha_species,
                    is_tower_boss, is_raid_boss, is_predator, nocturnal, edible,
                    max_stomach, food_amount, biological_grade, stamina,
                    male_probability, breeding_rank, disabled, icon, source_id
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                    ?, ?, ?
                )
                """,
                (
                    code,
                    english_name(en_pals, code),
                    row.get("tribe"),
                    as_int(row.get("pal_deck_index"), -1),
                    classify_species(code, row),
                    row.get("size"),
                    as_int(row.get("rarity")),
                    row.get("genus_category"),
                    row.get("weapon"),
                    as_bool(row.get("weapon_equip")),
                    as_int(scaling.get("hp")),
                    as_int(scaling.get("attack")),
                    as_int(scaling.get("defense")),
                    as_float(row.get("friendship_hp")),
                    as_float(row.get("friendship_shotattack")),
                    as_float(row.get("friendship_defense")),
                    as_float(row.get("friendship_craftspeed")),
                    as_float(row.get("enemy_max_hp_rate")),
                    as_float(row.get("enemy_receive_damage_rate")),
                    as_float(row.get("enemy_inflict_damage_rate")),
                    as_float(row.get("capture_rate_correct")),
                    as_float(row.get("exp_ratio")),
                    as_float(row.get("price")),
                    as_int(row.get("slow_walk_speed")),
                    as_int(row.get("walk_speed")),
                    as_int(row.get("run_speed")),
                    as_int(row.get("ride_sprint_speed")),
                    as_int(row.get("transport_speed")),
                    as_bool(row.get("is_boss")),
                    as_bool(row.get("is_tower_boss")),
                    as_bool(row.get("is_raid_boss")),
                    as_bool(row.get("predator")),
                    as_bool(row.get("nocturnal")),
                    as_bool(row.get("edible")),
                    as_int(row.get("max_full_stomach")),
                    as_int(row.get("food_amount")),
                    as_int(row.get("biological_grade")),
                    as_int(row.get("stamina")),
                    as_float(row.get("male_probability"), 50),
                    as_int(row.get("combi_rank")),
                    as_bool(row.get("disabled")),
                    row.get("icon"),
                    psp_source,
                ),
            )
            for position, raw_element in enumerate(
                value
                for value in (row.get("element_types") or [])
                if value not in ("", "None", None)
            ):
                connection.execute(
                    """
                    INSERT INTO species_element(species_code, element_code, position)
                    VALUES (?, ?, ?)
                    """,
                    (code, ELEMENT_CODES[raw_element], position),
                )
            suitability = row.get("work_suitability") or {}
            connection.executemany(
                """
                INSERT INTO species_work(species_code, work_code, base_level)
                VALUES (?, ?, ?)
                """,
                [
                    (code, work_code, as_int(suitability.get(work_code)))
                    for work_code in WORK_ORDER
                ],
            )

        move_rows: dict[str, dict[str, Any]] = {}
        for full_code, row in moves.items():
            code = full_code.removeprefix(WAZA_PREFIX)
            move_rows[code] = row
            localized = en_moves.get(full_code) or en_moves.get(code) or {}
            connection.execute(
                """
                INSERT INTO move(
                    code, name, element_code, category, power, min_range,
                    max_range, cooldown, synthesized, source_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?)
                """,
                (
                    code,
                    localized.get("localized_name") or prettify(code),
                    ELEMENT_CODES.get(row.get("element")),
                    row.get("type"),
                    as_int(row.get("power")),
                    row.get("min_range"),
                    row.get("max_range"),
                    row.get("cool_time"),
                    psp_source,
                ),
            )
            for position, effect in enumerate(row.get("effects") or []):
                connection.execute(
                    """
                    INSERT INTO move_effect(
                        move_code, position, type, value, value_ex
                    ) VALUES (?, ?, ?, ?, ?)
                    """,
                    (
                        code,
                        position,
                        str(effect.get("type") or "None"),
                        effect.get("value"),
                        effect.get("value_ex"),
                    ),
                )

        referenced_moves = {
            move_code
            for row in species_rows.values()
            for move_code in (row.get("skill_set") or {})
        }
        for code in sorted(referenced_moves - move_rows.keys()):
            localized = (
                en_moves.get(WAZA_PREFIX + code) or en_moves.get(code) or {}
            )
            connection.execute(
                """
                INSERT INTO move(
                    code, name, element_code, category, power, min_range,
                    max_range, cooldown, synthesized, source_id
                ) VALUES (?, ?, NULL, 'Unique', 0, NULL, NULL, NULL, 1, ?)
                """,
                (
                    code,
                    localized.get("localized_name") or synthesized_move_name(code),
                    psp_source,
                ),
            )
            add_issue(
                connection,
                "warning",
                "Move is referenced by a species learnset but absent from active_skills.json; placeholder fields were synthesized.",
                entity_type="move",
                entity_code=code,
                source_id=psp_source,
            )

        for species_code, row in species_rows.items():
            for move_code, level in (row.get("skill_set") or {}).items():
                connection.execute(
                    """
                    INSERT INTO species_move(
                        species_code, move_code, unlock_level, source_type
                    ) VALUES (?, ?, ?, 'level')
                    """,
                    (species_code, move_code, as_int(level)),
                )

        for code, row in passives.items():
            localized = en_passives.get(code) or {}
            passive_name = localized.get("localized_name") or prettify(code)
            effects = row.get("effects") or []
            connection.execute(
                """
                INSERT INTO passive(
                    code, name, description, rating, invoke_active_party,
                    invoke_worker, invoke_riding, invoke_reserve, invoke_in_party,
                    invoke_always, invoke_in_base, available_normal_pal,
                    available_lucky_pal, available_shot_weapon,
                    available_melee_weapon, available_armor, available_accessory,
                    disabled, source_id
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                )
                """,
                (
                    code,
                    passive_name,
                    plain_passive_description(code, passive_name, effects),
                    as_int(row.get("rank")),
                    as_bool(row.get("invoke_active_party")),
                    as_bool(row.get("invoke_worker")),
                    as_bool(row.get("invoke_riding")),
                    as_bool(row.get("invoke_reserve")),
                    as_bool(row.get("invoke_in_party")),
                    as_bool(row.get("invoke_always")),
                    as_bool(row.get("invoke_in_base")),
                    as_bool(row.get("add_pal")),
                    as_bool(row.get("add_rare_pal")),
                    as_bool(row.get("add_shot_weapon")),
                    as_bool(row.get("add_melee_weapon")),
                    as_bool(row.get("add_armor")),
                    as_bool(row.get("add_accessory")),
                    as_bool(row.get("disabled")),
                    psp_source,
                ),
            )
            for position, effect in enumerate(effects):
                connection.execute(
                    """
                    INSERT INTO passive_effect(
                        passive_code, position, type, value, target
                    ) VALUES (?, ?, ?, ?, ?)
                    """,
                    (
                        code,
                        position,
                        str(effect.get("type") or "None"),
                        effect.get("value"),
                        effect.get("target"),
                    ),
                )

        for species_code, row in species_rows.items():
            for passive_code in row.get("passive_skills") or []:
                if passive_code not in passives:
                    add_issue(
                        connection,
                        "error",
                        "Innate passive is absent from passive_skills.json and was not imported.",
                        entity_type="species",
                        entity_code=species_code,
                        field="passive_skills",
                        source_id=psp_source,
                    )
                    continue
                connection.execute(
                    """
                    INSERT INTO species_passive(
                        species_code, passive_code, relationship
                    ) VALUES (?, ?, 'innate')
                    """,
                    (species_code, passive_code),
                )

        for code, row in items.items():
            localized = en_items.get(code) or {}
            connection.execute(
                """
                INSERT INTO item(
                    code, name, item_group, type_a, type_b, rank, rarity,
                    max_stack_count, weight, price, sort_id, icon, disabled,
                    source_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    code,
                    localized.get("localized_name") or prettify(code),
                    row.get("group"),
                    row.get("type_a"),
                    row.get("type_b"),
                    row.get("rank"),
                    row.get("rarity"),
                    row.get("max_stack_count"),
                    row.get("weight"),
                    row.get("price"),
                    row.get("sort_id"),
                    row.get("icon"),
                    as_bool(row.get("disabled")),
                    psp_source,
                ),
            )

        for level, row in exp.items():
            connection.execute(
                """
                INSERT INTO exp_level(
                    level, drop_exp, next_exp, pal_next_exp, total_exp,
                    pal_total_exp, build_exp, craft_exp, pal_build_exp,
                    pal_craft_exp, source_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    int(level),
                    as_int(row.get("DropEXP")),
                    as_int(row.get("NextEXP")),
                    as_int(row.get("PalNextEXP")),
                    as_int(row.get("TotalEXP")),
                    as_int(row.get("PalTotalEXP")),
                    as_int(row.get("BuildEXP")),
                    as_int(row.get("CraftEXP")),
                    as_int(row.get("PalBuildEXP")),
                    as_int(row.get("PalCraftEXP")),
                    psp_source,
                ),
            )

        for code, row in friendship.items():
            connection.execute(
                """
                INSERT INTO friendship_rank(code, rank, required_point, source_id)
                VALUES (?, ?, ?, ?)
                """,
                (
                    code,
                    as_int(row.get("rank")),
                    as_int(row.get("required_point")),
                    psp_source,
                ),
            )

        current_species_by_name: dict[str, list[str]] = {}
        for current_code in species_rows:
            current_species_by_name.setdefault(
                english_name(en_pals, current_code).casefold(), []
            ).append(current_code)

        partner_by_name: dict[str, dict[str, Any]] = {}
        resolved_partner_codes: set[str] = set()
        for row in partner_payload["partner_skills"]:
            species_code = row["internal_code"]
            parsed_code_name = (
                english_name(en_pals, species_code)
                if species_code in species_rows
                else None
            )
            if (
                parsed_code_name is None
                or parsed_code_name.casefold() != row["species_name"].casefold()
            ):
                name_matches = current_species_by_name.get(
                    row["species_name"].casefold(), []
                )
                if len(name_matches) == 1:
                    corrected_code = name_matches[0]
                    add_issue(
                        connection,
                        "warning",
                        f"Corrected source-page image code {species_code!r} to {corrected_code!r} using the unique current species name.",
                        entity_type="partner_skill",
                        entity_code=corrected_code,
                        field="species_code",
                        source_id=partner_source,
                    )
                    species_code = corrected_code
            if species_code not in species_rows:
                add_issue(
                    connection,
                    "warning",
                    "Partner Skill row did not resolve to a storable game-data species.",
                    entity_type="partner_skill",
                    entity_code=species_code,
                    source_id=partner_source,
                )
                continue
            current_name = english_name(en_pals, species_code)
            if row["species_name"].casefold() != current_name.casefold():
                add_issue(
                    connection,
                    "info",
                    f"External species label {row['species_name']!r} differs from current game localization {current_name!r}; internal code matched.",
                    entity_type="species",
                    entity_code=species_code,
                    field="name",
                    source_id=partner_source,
                )
            description = (row.get("description") or "").strip()
            if not description:
                add_issue(
                    connection,
                    "warning",
                    "Current Partner Skill source provides no description.",
                    entity_type="partner_skill",
                    entity_code=species_code,
                    field="description",
                    source_id=partner_source,
                )
            connection.execute(
                """
                INSERT INTO partner_skill(
                    species_code, name, description, category, element_code,
                    gear_item_slug, gear_name, technology_level, source_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    species_code,
                    row["partner_skill_name"],
                    description,
                    row.get("category"),
                    ELEMENT_CODES.get(row.get("element"), row.get("element")),
                    row.get("gear_item_slug"),
                    row.get("gear_name"),
                    row.get("technology_level"),
                    partner_source,
                ),
            )
            for rank, value in enumerate(row.get("rank_values") or [], start=1):
                try:
                    numeric_value = float(value)
                except (TypeError, ValueError):
                    numeric_value = None
                connection.execute(
                    """
                    INSERT INTO partner_skill_rank(
                        species_code, rank, value_text, value_number
                    ) VALUES (?, ?, ?, ?)
                    """,
                    (species_code, rank, str(value), numeric_value),
                )
            resolved_partner = dict(row)
            resolved_partner["internal_code"] = species_code
            partner_by_name[row["species_name"].casefold()] = resolved_partner
            resolved_partner_codes.add(species_code)

        # Engine encounter/appearance variants deliberately reuse a base species'
        # localized name and tribe but are not separate cards on the source page
        # (flower Gumoss, Oil Rig, Predator, Raid, and quest variants). Inherit only
        # when current name+tribe identity resolves to one verified source row.
        for species_code, species_row in species_rows.items():
            if species_code in resolved_partner_codes:
                continue
            species_name = english_name(en_pals, species_code)
            matches = [
                source_code
                for source_code in resolved_partner_codes
                if english_name(en_pals, source_code).casefold()
                == species_name.casefold()
                and species_rows[source_code].get("tribe") == species_row.get("tribe")
            ]
            if len(matches) != 1:
                continue
            source_code = matches[0]
            connection.execute(
                """
                INSERT INTO partner_skill(
                    species_code, name, description, category, element_code,
                    gear_item_slug, gear_name, technology_level, source_id
                )
                SELECT ?, name, description, category, element_code,
                       gear_item_slug, gear_name, technology_level, source_id
                FROM partner_skill
                WHERE species_code = ?
                """,
                (species_code, source_code),
            )
            connection.execute(
                """
                INSERT INTO partner_skill_rank(
                    species_code, rank, value_text, value_number
                )
                SELECT ?, rank, value_text, value_number
                FROM partner_skill_rank
                WHERE species_code = ?
                """,
                (species_code, source_code),
            )
            add_issue(
                connection,
                "info",
                f"Inherited Partner Skill from same-name, same-tribe engine variant {source_code!r}.",
                entity_type="partner_skill",
                entity_code=species_code,
                field="species_code",
                source_id=partner_source,
            )
            resolved_partner_codes.add(species_code)

        for ranch_row in ranch_payload["ranch_rows"]:
            external_name = ranch_row["species_name"]
            partner = partner_by_name.get(external_name.casefold())
            if partner is None:
                add_issue(
                    connection,
                    "warning",
                    "Ranch row could not be resolved through the current Partner Skill source.",
                    entity_type="ranch_drop",
                    entity_code=external_name,
                    source_id=ranch_source,
                )
                continue
            species_code = partner["internal_code"]
            source_deck = re.match(r"\d+", str(ranch_row.get("paldeck_no") or ""))
            source_deck_number = int(source_deck.group()) if source_deck else -1
            current_deck = as_int(species_rows[species_code].get("pal_deck_index"), -1)
            if source_deck_number != current_deck:
                add_issue(
                    connection,
                    "warning",
                    f"Skipped stale Ranch row with Paldeck {ranch_row.get('paldeck_no')}; current game data is {current_deck}.",
                    entity_type="ranch_drop",
                    entity_code=species_code,
                    field="paldeck_no",
                    source_id=ranch_source,
                )
                continue
            for position, external_item_name in enumerate(ranch_row["item_names"]):
                item_code = RANCH_ITEM_CODES.get(external_item_name)
                if item_code not in items:
                    add_issue(
                        connection,
                        "error",
                        f"Ranch product {external_item_name!r} has no verified current item-code mapping.",
                        entity_type="ranch_drop",
                        entity_code=species_code,
                        field="item_code",
                        source_id=ranch_source,
                    )
                    item_code = None
                    current_item_name = external_item_name
                else:
                    current_item_name = english_name(en_items, item_code)
                    if external_item_name.casefold() != current_item_name.casefold():
                        add_issue(
                            connection,
                            "info",
                            f"External item label {external_item_name!r} maps to current game localization {current_item_name!r}.",
                            entity_type="item",
                            entity_code=item_code,
                            field="name",
                            source_id=ranch_source,
                        )
                connection.execute(
                    """
                    INSERT INTO ranch_drop(
                        species_code, position, item_code, item_name, notes, source_id
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    (
                        species_code,
                        position,
                        item_code,
                        current_item_name,
                        ranch_row.get("notes") or None,
                        ranch_source,
                    ),
                )

        valid_codes = {
            "species": set(species_rows),
            "move": set(move_rows) | referenced_moves,
            "passive": set(passives),
            "item": set(items),
            "element": {value for value in ELEMENT_CODES.values() if value},
            "work_type": set(WORK_ORDER),
        }
        localization_files = {
            "pals.json": ("species", lambda code: code),
            "active_skills.json": (
                "move",
                lambda code: code.removeprefix(WAZA_PREFIX),
            ),
            "passive_skills.json": ("passive", lambda code: code),
            "items.json": ("item", lambda code: code),
            "elements.json": ("element", lambda code: ELEMENT_CODES.get(code)),
            "work_suitability.json": ("work_type", lambda code: code),
        }
        for locale_dir in sorted(path for path in (PSP_JSON / "l10n").iterdir() if path.is_dir()):
            locale = locale_dir.name
            for filename, (entity_type, transform) in localization_files.items():
                path = locale_dir / filename
                if not path.exists():
                    continue
                for raw_code, row in load_json(path).items():
                    entity_code = transform(raw_code)
                    if entity_code not in valid_codes[entity_type]:
                        continue
                    if isinstance(row, dict):
                        name = row.get("localized_name")
                        description = row.get("description")
                    else:
                        name = str(row)
                        description = None
                    connection.execute(
                        """
                        INSERT OR REPLACE INTO localization(
                            entity_type, entity_code, locale, name, description
                        ) VALUES (?, ?, ?, ?, ?)
                        """,
                        (entity_type, entity_code, locale, name, description),
                    )

        filter_fields = [
            ("name", "Name", "text", "species", "name", 1, 1, 10, "contains", None),
            ("elements", "Element", "relation", "species_element", "element_code", 1, 1, 20, "any", None),
            ("category", "Category", "enum", "species", "category", 1, 1, 30, "equals", None),
            ("rarity", "Rarity", "number", "species", "rarity", 1, 1, 40, "range", None),
            ("size", "Size", "enum", "species", "size", 1, 1, 50, "equals", None),
            ("genus", "Genus", "enum", "species", "genus", 1, 1, 60, "equals", None),
            ("nocturnal", "Nocturnal", "bool", "species", "nocturnal", 1, 1, 70, "equals", None),
            ("disabled", "Disabled", "bool", "species", "disabled", 1, 0, 80, "equals", None),
            ("work", "Work Suitability", "relation", "species_work", "base_level", 1, 1, 90, "minimum", "Pair work_code with minimum base_level."),
            ("moves", "Learnset", "relation", "species_move", "move_code", 1, 0, 100, "any", None),
            ("innate_passives", "Innate Passives", "relation", "species_passive", "passive_code", 1, 0, 110, "any", None),
            ("paldeck_index", "Paldeck No.", "number", "species", "paldeck_index", 1, 1, 120, "range", None),
            ("breeding_rank", "Breeding Rank", "number", "species", "breeding_rank", 1, 1, 130, "range", None),
            ("hp_scaling", "Base HP", "number", "species", "hp_scaling", 1, 1, 140, "range", None),
            ("attack_scaling", "Base Attack", "number", "species", "attack_scaling", 1, 1, 150, "range", None),
            ("defense_scaling", "Base Defense", "number", "species", "defense_scaling", 1, 1, 160, "range", None),
            ("capture_rate", "Capture Rate", "number", "species", "capture_rate", 1, 1, 170, "range", None),
            ("price", "Gold Value", "number", "species", "price", 1, 1, 180, "range", None),
            ("partner_skill", "Partner Skill", "relation", "partner_skill", "name", 1, 1, 190, "contains", None),
            ("partner_category", "Partner Skill Type", "relation", "partner_skill", "category", 1, 1, 200, "equals", None),
            ("ranch_drops", "Ranch Products", "relation", "ranch_drop", "item_code", 1, 1, 210, "any", None),
        ]
        connection.executemany(
            """
            INSERT INTO filter_field(
                key, label, value_type, source_relation, source_column,
                filterable, displayable, sort_order, operator_hint, notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            filter_fields,
        )
        option_values = {
            "elements": connection.execute(
                "SELECT code, name, sort_order FROM element ORDER BY sort_order"
            ).fetchall(),
            "category": [
                ("Natural", "Natural", 0),
                ("TowerBoss", "Tower Boss", 1),
                ("Unobtainable", "Unobtainable", 2),
            ],
            "size": [
                (value, value, position)
                for position, (value,) in enumerate(
                    connection.execute(
                        "SELECT DISTINCT size FROM species WHERE size IS NOT NULL ORDER BY size"
                    )
                )
            ],
            "genus": [
                (value, value, position)
                for position, (value,) in enumerate(
                    connection.execute(
                        "SELECT DISTINCT genus FROM species WHERE genus IS NOT NULL ORDER BY genus"
                    )
                )
            ],
            "work": connection.execute(
                "SELECT code, name, sort_order FROM work_type ORDER BY sort_order"
            ).fetchall(),
            "partner_category": [
                (value, value, position)
                for position, (value,) in enumerate(
                    connection.execute(
                        "SELECT DISTINCT category FROM partner_skill WHERE category IS NOT NULL ORDER BY category"
                    )
                )
            ],
        }
        for field_key, rows in option_values.items():
            connection.executemany(
                """
                INSERT INTO filter_option(field_key, value, label, sort_order)
                VALUES (?, ?, ?, ?)
                """,
                [(field_key, value, label, order) for value, label, order in rows],
            )

        integrity = connection.execute("PRAGMA integrity_check").fetchone()[0]
        foreign_keys = connection.execute("PRAGMA foreign_key_check").fetchall()
        errors = connection.execute(
            "SELECT COUNT(*) FROM data_quality_issue WHERE severity = 'error'"
        ).fetchone()[0]
        if integrity != "ok" or foreign_keys or errors:
            raise ValueError(
                f"database validation failed: integrity={integrity}, "
                f"foreign_keys={len(foreign_keys)}, quality_errors={errors}"
            )
        counts = {
            table: connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
            for table in (
                "species",
                "move",
                "passive",
                "item",
                "partner_skill",
                "ranch_drop",
                "localization",
                "data_quality_issue",
            )
        }
        install_database(connection, temp_path, destination)
        return counts
    except Exception:
        connection.close()
        if temp_path.exists():
            temp_path.unlink()
        raise


def build_user_template(destination: Path) -> None:
    connection, temp_path = create_connection(USER_SCHEMA, destination)
    try:
        connection.executemany(
            "INSERT OR REPLACE INTO metadata(key, value) VALUES (?, ?)",
            [
                ("generated_at", "2026-07-25T00:00:00+00:00"),
                ("generator", "scripts/build_reference_db.py"),
                ("preset_passive_limit", "4"),
                ("per_pal_state_included", "false"),
            ],
        )
        integrity = connection.execute("PRAGMA integrity_check").fetchone()[0]
        if integrity != "ok":
            raise ValueError(f"user template integrity check failed: {integrity}")
        install_database(connection, temp_path, destination)
    except Exception:
        connection.close()
        if temp_path.exists():
            temp_path.unlink()
        raise


def validate_installed(reference_path: Path, user_path: Path) -> None:
    expected_counts = {
        "species": 406,
        "partner_skill": 348,
    }
    with sqlite3.connect(f"file:{reference_path.as_posix()}?mode=ro", uri=True) as connection:
        if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise ValueError("reference DB integrity_check failed")
        if connection.execute("PRAGMA foreign_key_check").fetchall():
            raise ValueError("reference DB foreign_key_check failed")
        for table, expected in expected_counts.items():
            actual = connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
            if actual != expected:
                raise ValueError(f"{table}: expected {expected}, found {actual}")
        unresolved_ranch = connection.execute(
            "SELECT COUNT(*) FROM ranch_drop WHERE item_code IS NULL"
        ).fetchone()[0]
        if unresolved_ranch:
            raise ValueError(f"{unresolved_ranch} Ranch products have no item code")
    with sqlite3.connect(f"file:{user_path.as_posix()}?mode=ro", uri=True) as source:
        if source.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise ValueError("user template integrity_check failed")
        connection = sqlite3.connect(":memory:")
        source.backup(connection)
        connection.execute("PRAGMA foreign_keys = ON")
        connection.execute(
            "INSERT INTO passive_preset(name) VALUES ('Validation Preset')"
        )
        preset_id = connection.execute(
            "SELECT id FROM passive_preset WHERE name = 'Validation Preset'"
        ).fetchone()[0]
        connection.executemany(
            """
            INSERT INTO passive_preset_entry(preset_id, slot, passive_code)
            VALUES (?, ?, ?)
            """,
            [(preset_id, slot, f"Validation_{slot}") for slot in range(4)],
        )
        try:
            connection.execute(
                """
                INSERT INTO passive_preset_entry(preset_id, slot, passive_code)
                VALUES (?, 4, 'Validation_4')
                """,
                (preset_id,),
            )
        except sqlite3.IntegrityError:
            pass
        else:
            raise ValueError("user DB accepted a fifth passive preset slot")
        finally:
            connection.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-output", type=Path, default=DEFAULT_REFERENCE_DB)
    parser.add_argument("--user-output", type=Path, default=DEFAULT_USER_DB)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Run strict installed-database validation after building.",
    )
    args = parser.parse_args()

    counts = build_reference(args.reference_output.resolve())
    build_user_template(args.user_output.resolve())
    if args.check:
        validate_installed(args.reference_output.resolve(), args.user_output.resolve())
    print(f"reference DB: {args.reference_output.resolve()}")
    print(f"user template DB: {args.user_output.resolve()}")
    print("counts: " + ", ".join(f"{key}={value}" for key, value in counts.items()))
    if args.check:
        print("validation: ok")


if __name__ == "__main__":
    main()

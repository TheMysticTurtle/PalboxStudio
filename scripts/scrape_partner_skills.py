#!/usr/bin/env python3
"""Scrape Palworld 1.0 Partner Skill facts into a reviewable JSON snapshot.

The scraper intentionally uses only the Python standard library. The generated
snapshot is an input to build_reference_db.py; the SQLite build itself remains
offline and reproducible.

Primary external source:
  https://www.palworld.tools/partner-skills

That page identifies its data as extracted from Palworld 1.0 build 24088745 and
updated July 13, 2026. The database builder records that claim as provenance and
cross-checks species identities against the local PSP 1.2.0 game-data snapshot.

Examples:
  python scripts/scrape_partner_skills.py
  python scripts/scrape_partner_skills.py --html C:/tmp/partner-skills.html
"""
from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import urllib.request
from dataclasses import dataclass, field
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from typing import Iterable


REPO = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = (
    REPO
    / "data"
    / "reference-sources"
    / f"palworld-tools-partner-skills-{datetime.now(timezone.utc).date().isoformat()}.parsed.json"
)
SOURCE_URL = "https://www.palworld.tools/partner-skills"
ELEMENTS = {"Neutral", "Fire", "Water", "Grass", "Electric", "Ice", "Ground", "Dark", "Dragon"}
CATEGORIES = {"Ride", "Glide", "Combat", "Gather", "Base", "Other"}
VOID_TAGS = {
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link",
    "meta", "param", "source", "track", "wbr",
}


@dataclass
class Node:
    tag: str
    attrs: dict[str, str] = field(default_factory=dict)
    children: list["Node | str"] = field(default_factory=list)
    parent: "Node | None" = None

    def descendants(self, tag: str | None = None) -> Iterable["Node"]:
        for child in self.children:
            if not isinstance(child, Node):
                continue
            if tag is None or child.tag == tag:
                yield child
            yield from child.descendants(tag)

    def text(self) -> str:
        chunks: list[str] = []

        def collect(node: "Node") -> None:
            for child in node.children:
                if isinstance(child, str):
                    chunks.append(child)
                else:
                    collect(child)

        collect(self)
        return " ".join(" ".join(chunks).split())

    def classes(self) -> set[str]:
        return set(self.attrs.get("class", "").split())


class TreeParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.root = Node("document")
        self.stack = [self.root]

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        node = Node(tag, {key: value or "" for key, value in attrs}, parent=self.stack[-1])
        self.stack[-1].children.append(node)
        if tag not in VOID_TAGS:
            self.stack.append(node)

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.handle_starttag(tag, attrs)
        if tag not in VOID_TAGS:
            self.stack.pop()

    def handle_endtag(self, tag: str) -> None:
        for index in range(len(self.stack) - 1, 0, -1):
            if self.stack[index].tag == tag:
                del self.stack[index:]
                return

    def handle_data(self, data: str) -> None:
        if data:
            self.stack[-1].children.append(data)


def first(nodes: Iterable[Node], predicate) -> Node | None:
    return next((node for node in nodes if predicate(node)), None)


def clean_text(value: str) -> str:
    return " ".join(html.unescape(value).replace("\u2019", "'").split())


def parse_card(card: Node) -> dict[str, object]:
    pal_link = first(
        card.descendants("a"),
        lambda node: node.attrs.get("href", "").startswith("/pals/") and bool(node.attrs.get("title")),
    )
    if pal_link is None:
        raise ValueError("partner-skill card has no titled /pals/ link")

    pal_image = first(
        card.descendants("img"),
        lambda node: node.attrs.get("src", "").startswith("/img/pals/"),
    )
    skill_name = first(
        card.descendants("span"),
        lambda node: "font-extrabold" in node.classes(),
    )
    description = first(card.descendants("p"), lambda _node: True)
    if pal_image is None or skill_name is None:
        raise ValueError(f"incomplete partner-skill card for {pal_link.attrs.get('title')}")

    element_node = first(
        card.descendants("img"),
        lambda node: node.attrs.get("title") in ELEMENTS,
    )
    category_node = first(
        card.descendants("span"),
        lambda node: node.text() in CATEGORIES,
    )
    gear_link = first(
        card.descendants("a"),
        lambda node: node.attrs.get("href", "").startswith("/items/"),
    )

    gear_name: str | None = None
    technology_level: int | None = None
    gear_item_slug: str | None = None
    if gear_link is not None:
        gear_item_slug = gear_link.attrs["href"].removeprefix("/items/")
        gear_name_node = first(
            gear_link.descendants("span"),
            lambda node: "font-semibold" in node.classes(),
        )
        gear_name = clean_text(gear_name_node.text()) if gear_name_node else None
        tech_match = re.search(r"\bTech\s+(\d+)\b", gear_link.text())
        technology_level = int(tech_match.group(1)) if tech_match else None

    ranks: list[str] = []
    rank_heading = first(
        card.descendants("div"),
        lambda node: node.text() == "Effect by rank",
    )
    if rank_heading is not None and rank_heading.parent is not None:
        rank_nodes = [
            node
            for node in rank_heading.parent.descendants("span")
            if "tabular-nums" in node.classes() and "flex-1" in node.classes()
        ]
        ranks = [clean_text(node.text()) for node in rank_nodes]

    image_path = pal_image.attrs["src"]
    internal_code = Path(image_path).stem
    return {
        "species_slug": pal_link.attrs["href"].removeprefix("/pals/"),
        "species_name": clean_text(pal_link.attrs["title"]),
        "internal_code": internal_code,
        "partner_skill_name": clean_text(skill_name.text()),
        "description": clean_text(description.text()) if description else "",
        "element": element_node.attrs.get("title") if element_node else None,
        "category": category_node.text() if category_node else None,
        "gear_item_slug": gear_item_slug,
        "gear_name": gear_name,
        "technology_level": technology_level,
        "rank_values": ranks,
    }


def parse_page(raw: bytes) -> list[dict[str, object]]:
    parser = TreeParser()
    parser.feed(raw.decode("utf-8"))
    cards = [
        node
        for node in parser.root.descendants("div")
        if {"rounded-[10px]", "border-line", "bg-bg-2"}.issubset(node.classes())
        and first(
            node.descendants("a"),
            lambda link: link.attrs.get("href", "").startswith("/pals/")
            and bool(link.attrs.get("title")),
        )
    ]
    parsed = [parse_card(card) for card in cards]
    unique = {row["species_slug"] for row in parsed}
    if len(parsed) != len(unique):
        raise ValueError(f"duplicate species cards: rows={len(parsed)} unique={len(unique)}")
    if len(parsed) < 250:
        raise ValueError(f"unexpectedly small partner-skill page: {len(parsed)} cards")
    return parsed


def download() -> bytes:
    request = urllib.request.Request(
        SOURCE_URL,
        headers={
            "User-Agent": "PalboxStudioReferenceBuilder/0.1 (+local data verification)",
            "Accept": "text/html,application/xhtml+xml",
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--html", type=Path, help="Parse an already-downloaded HTML page")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    normalized_at = datetime.now(timezone.utc).isoformat(timespec="seconds")
    if args.html:
        raw = args.html.read_bytes()
        retrieved_at = datetime.fromtimestamp(
            args.html.stat().st_mtime, timezone.utc
        ).isoformat(timespec="seconds")
    else:
        raw = download()
        retrieved_at = normalized_at
    rows = parse_page(raw)
    page_text = raw.decode("utf-8", errors="replace")
    build_match = re.search(r"build\s+(\d+)", page_text, re.IGNORECASE)
    updated_match = re.search(r"updated\s+([A-Z][a-z]+\s+\d{1,2},\s+20\d{2})", page_text)

    payload = {
        "_meta": {
            "source_url": SOURCE_URL,
            "retrieved_at": retrieved_at,
            "normalized_at": normalized_at,
            "html_sha256": hashlib.sha256(raw).hexdigest(),
            "claimed_game_version": "Palworld 1.0",
            "claimed_game_build": build_match.group(1) if build_match else None,
            "claimed_updated": updated_match.group(1) if updated_match else None,
            "record_count": len(rows),
            "generator": "scripts/scrape_partner_skills.py",
            "notes": "Derived factual fields only; the exact source HTML is retained beside this file.",
        },
        "partner_skills": rows,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"partner skills: {len(rows)}")
    print(f"output: {args.output}")
    print(f"html sha256: {payload['_meta']['html_sha256']}")


if __name__ == "__main__":
    main()

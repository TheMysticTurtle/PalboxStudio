import assert from "node:assert/strict";
import test from "node:test";
import {
  speciesMatches,
  toggleOnly,
} from "../src/lib/data/speciesFilter.ts";
import { matchesAllGroups } from "../src/lib/data/groupFilter.ts";
import { moveSkill } from "../src/lib/data/moveSlots.ts";
import {
  DEFAULT_PASSIVE_SCOPE,
  passiveMatches,
} from "../src/lib/data/passiveFilter.ts";
import {
  parseBoxPreferences,
  shouldMigrateLegacyBoxPreferences,
} from "../src/lib/data/boxPreferences.ts";
import { classifySourceConflict } from "../src/lib/data/sourceMonitor.ts";

function filter(overrides = {}) {
  return {
    search: "",
    elements: new Set(),
    work: new Set(),
    rideable: false,
    ranchDrops: new Set(),
    categories: new Set(),
    ...overrides,
  };
}

const dualElementWorker = {
  code: "TestPal",
  name: "Test Pal",
  elements: ["Fire", "Dark"],
  category: "Natural",
  work: { Kindling: 3, Handiwork: 2 },
  partnerSkill: { gearName: "Test Pal Saddle" },
  farmDrops: [
    { itemName: "Leather" },
    { itemName: "Horn" },
  ],
};

test("multi-select facets require every selected value", () => {
  assert.equal(
    speciesMatches(dualElementWorker, filter({
      elements: new Set(["Fire", "Dark"]),
      work: new Set(["Kindling", "Handiwork"]),
      ranchDrops: new Set(["Leather", "Horn"]),
      rideable: true,
    })),
    true,
  );

  assert.equal(
    speciesMatches(dualElementWorker, filter({ elements: new Set(["Fire", "Water"]) })),
    false,
  );
  assert.equal(
    speciesMatches(dualElementWorker, filter({ work: new Set(["Kindling", "Mining"]) })),
    false,
  );
  assert.equal(
    speciesMatches(dualElementWorker, filter({ ranchDrops: new Set(["Leather", "Wool"]) })),
    false,
  );
});

test("facet groups also intersect with one another", () => {
  assert.equal(
    speciesMatches(dualElementWorker, filter({
      search: "test",
      elements: new Set(["Fire"]),
      work: new Set(["Handiwork"]),
      categories: new Set(["Natural"]),
    })),
    true,
  );
  assert.equal(
    speciesMatches(dualElementWorker, filter({
      search: "test",
      elements: new Set(["Fire"]),
      work: new Set(["Handiwork"]),
      categories: new Set(["TowerBoss"]),
    })),
    false,
  );
});

test("category selection is mutually exclusive and toggleable", () => {
  const natural = toggleOnly(new Set(), "Natural");
  assert.deepEqual([...natural], ["Natural"]);
  assert.deepEqual([...toggleOnly(natural, "TowerBoss")], ["TowerBoss"]);
  assert.deepEqual([...toggleOnly(natural, "Natural")], []);
});

test("bench moves can be dragged into any open active slot", () => {
  const result = moveSkill(
    { active: ["FireBall"], bench: ["WindCutter", "DarkLaser"] },
    { code: "DarkLaser", list: "bench", index: 1 },
    "active",
    1,
    3,
  );
  assert.deepEqual(result.active, ["FireBall", "DarkLaser"]);
  assert.deepEqual(result.bench, ["WindCutter"]);
  assert.equal(result.displaced, null);
});

test("dragging one active skill onto another swaps their slots", () => {
  const result = moveSkill(
    { active: ["FireBall", "WindCutter", "DarkLaser"], bench: ["StoneBlast"] },
    { code: "DarkLaser", list: "active", index: 2 },
    "active",
    0,
    3,
  );
  assert.deepEqual(result.active, ["DarkLaser", "WindCutter", "FireBall"]);
  assert.deepEqual(result.bench, ["StoneBlast"]);
});

test("dropping a bench move onto an occupied active slot swaps the two", () => {
  const result = moveSkill(
    { active: ["FireBall", "WindCutter", "DarkLaser"], bench: ["StoneBlast"] },
    { code: "StoneBlast", list: "bench", index: 0 },
    "active",
    1,
    3,
  );
  assert.deepEqual(result.active, ["FireBall", "StoneBlast", "DarkLaser"]);
  assert.deepEqual(result.bench, ["WindCutter"]);
  assert.equal(result.displaced, null);
});

test("group filters require membership in every selected group", () => {
  assert.equal(matchesAllGroups([1, 2, 3], [1, 3]), true);
  assert.equal(matchesAllGroups([1, 2], [1, 3]), false);
  assert.equal(matchesAllGroups([], []), true);
});

test("every passive picker defaults to the full enabled catalog", () => {
  const lunker = {
    name: "Lunker",
    description: "Water attack damage increases.",
    rating: 3,
    effects: [],
    availableNormalPal: false,
    availableLuckyPal: false,
    disabled: false,
  };

  assert.equal(DEFAULT_PASSIVE_SCOPE, "all");
  assert.equal(
    passiveMatches("Nushi", lunker, "", DEFAULT_PASSIVE_SCOPE, "all", "all", false, new Set()),
    true,
  );
  assert.equal(
    passiveMatches("Nushi", lunker, "", "species", "all", "all", false, new Set()),
    false,
  );
});

test("last-box preferences are version-safe and reject malformed storage", () => {
  assert.deepEqual(parseBoxPreferences(null), {
    lastBoxPath: "",
    autoReopen: false,
    maxHp: true,
    maxSanity: true,
    maxFood: true,
    maxTrust: false,
  });
  assert.deepEqual(parseBoxPreferences("{not-json"), {
    lastBoxPath: "",
    autoReopen: false,
    maxHp: true,
    maxSanity: true,
    maxFood: true,
    maxTrust: false,
  });
  assert.deepEqual(parseBoxPreferences(JSON.stringify({
    lastBoxPath: "C:\\Pal\\GlobalPalStorage.sav",
    autoReopen: true,
    ignoredFutureField: 42,
  })), {
    lastBoxPath: "C:\\Pal\\GlobalPalStorage.sav",
    autoReopen: true,
    maxHp: true,
    maxSanity: true,
    maxFood: true,
    maxTrust: false,
  });
});

test("only an empty database preference imports the legacy remembered box", () => {
  assert.equal(
    shouldMigrateLegacyBoxPreferences(
      { lastBoxPath: "", autoReopen: false, maxHp: true, maxSanity: true, maxFood: true, maxTrust: false },
      { lastBoxPath: "/tmp/GlobalPalStorage.sav", autoReopen: true, maxHp: true, maxSanity: true, maxFood: true, maxTrust: false },
    ),
    true,
  );
  assert.equal(
    shouldMigrateLegacyBoxPreferences(
      { lastBoxPath: "D:\\Pal\\GlobalPalStorage.sav", autoReopen: false, maxHp: true, maxSanity: true, maxFood: true, maxTrust: false },
      { lastBoxPath: "/tmp/stale.sav", autoReopen: true, maxHp: true, maxSanity: true, maxFood: true, maxTrust: false },
    ),
    false,
  );
});

test("source monitoring distinguishes ordinary and immediate post-save conflicts", () => {
  assert.equal(classifySourceConflict("unchanged", 1_000, 2_000), "");
  assert.equal(classifySourceConflict("changed", 0, 2_000), "external");
  assert.equal(classifySourceConflict("unavailable", 1_000, 31_000), "post-save");
  assert.equal(classifySourceConflict("changed", 1_000, 31_001), "external");
});

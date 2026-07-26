import assert from "node:assert/strict";
import test from "node:test";
import {
  speciesMatches,
  toggleOnly,
} from "../src/lib/data/speciesFilter.ts";
import { soulBonusPercent } from "../src/lib/data/constants.ts";
import { matchesAllGroups } from "../src/lib/data/groupFilter.ts";
import { moveSkill } from "../src/lib/data/moveSlots.ts";
import {
  DEFAULT_PASSIVE_SCOPE,
  passiveMatches,
} from "../src/lib/data/passiveFilter.ts";

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

test("Pal Soul percentages share the rank-20, 60-percent cap", () => {
  assert.equal(soulBonusPercent(0), 0);
  assert.equal(soulBonusPercent(10), 30);
  assert.equal(soulBonusPercent(20), 60);
  assert.equal(soulBonusPercent(255), 60);
});

test("bench moves can be dragged into any open active slot", () => {
  const result = moveSkill(
    { active: ["FireBall"], bench: ["WindCutter", "DarkLaser"] },
    { code: "DarkLaser", list: "bench", index: 1 },
    "active",
    1,
  );
  assert.deepEqual(result.active, ["FireBall", "DarkLaser"]);
  assert.deepEqual(result.bench, ["WindCutter"]);
  assert.equal(result.displaced, null);
});

test("active skills reorder without duplicating or losing a move", () => {
  const result = moveSkill(
    { active: ["FireBall", "WindCutter", "DarkLaser"], bench: ["StoneBlast"] },
    { code: "DarkLaser", list: "active", index: 2 },
    "active",
    0,
  );
  assert.deepEqual(result.active, ["DarkLaser", "FireBall", "WindCutter"]);
  assert.deepEqual(result.bench, ["StoneBlast"]);
});

test("dropping onto a full active set returns the displaced third skill to the bench", () => {
  const result = moveSkill(
    { active: ["FireBall", "WindCutter", "DarkLaser"], bench: ["StoneBlast"] },
    { code: "StoneBlast", list: "bench", index: 0 },
    "active",
    1,
  );
  assert.deepEqual(result.active, ["FireBall", "StoneBlast", "WindCutter"]);
  assert.deepEqual(result.bench, ["DarkLaser"]);
  assert.equal(result.displaced, "DarkLaser");
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

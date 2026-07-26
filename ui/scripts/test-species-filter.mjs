import assert from "node:assert/strict";
import test from "node:test";
import {
  speciesMatches,
  toggleOnly,
} from "../src/lib/data/speciesFilter.ts";
import { soulBonusPercent } from "../src/lib/data/constants.ts";

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

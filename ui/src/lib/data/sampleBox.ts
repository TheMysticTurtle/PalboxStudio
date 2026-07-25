import type { BoxPal } from "./types";

// Placeholder box contents (real species CodeNames → real icons) until the engine
// loads a save. Only global-box-storable pals — no humans/NPCs.
export const sampleBox: BoxPal[] = [
  { instanceId: "b1", species: "Baphomet", name: "Incineram", level: 38, elements: ["Fire", "Dark"], groups: ["Combat Team"] },
  { instanceId: "b2", species: "BlackMetalDragon", name: "Astegon", level: 42, elements: ["Dark", "Dragon"], alpha: true, groups: ["Combat Team"] },
  { instanceId: "b3", species: "BlackGriffon", name: "Shadowbeak", level: 45, elements: ["Dark"], lucky: true, groups: ["Combat Team"] },
  { instanceId: "b4", species: "BlackCentaur", name: "Necromus", level: 50, elements: ["Dark"], alpha: true },
  { instanceId: "b5", species: "BirdDragon", name: "Vanwyrm", level: 30, elements: ["Fire", "Dark"] },
  { instanceId: "b6", species: "Anubis", name: "Anubis", level: 47, elements: ["Ground"], groups: ["Base Crew"] },
  { instanceId: "b7", species: "BerryGoat", name: "Caprity", level: 22, elements: ["Grass"], groups: ["Base Crew"] },
  { instanceId: "b8", species: "Alpaca", name: "Melpaca", level: 15, elements: ["Neutral"], groups: ["Base Crew"] },
  { instanceId: "b9", species: "Bastet", name: "Mau", level: 12, elements: ["Dark"], lucky: true },
  { instanceId: "b10", species: "Bastet_Ice", name: "Mau Cryst", level: 18, elements: ["Ice"] },
  { instanceId: "b11", species: "AmaterasuWolf", name: "Kitsun", level: 27, elements: ["Fire"] },
  { instanceId: "b12", species: "Baphomet_Dark", name: "Incineram Noct", level: 33, elements: ["Dark"] },
];

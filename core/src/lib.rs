//! Palbox Studio core engine (headless).
//!
//! The Palworld 1.0 Global Pal box domain model, save load/write, and mutations.
//! This crate MUST stay usable without any UI (CLI, tests) and MUST NOT depend on
//! Tauri or the frontend — that separation is the whole point (see docs/DIRECTION.md).
//!
//! Value ranges here are the verified 1.0 numbers from docs/SPECS-1.0.md — deliberately
//! NOT the stale pre-1.0 values our old PalEdit fork used.

pub mod globalbox;
pub mod pal;
pub mod save;
pub mod ue;

/// Core crate version — surfaced to the UI as a smoke test that the bridge is wired up.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Verified Palworld 1.0 editing limits. One source of truth shared by the UI and the
/// (future) mutation layer, so a stepper's cap and a setter's clamp can never disagree.
pub mod limits {
    /// Pal level (game cap; exp table has headroom to 100).
    pub const LEVEL_MIN: u8 = 1;
    pub const LEVEL_MAX: u8 = 80;

    /// IV / breeding talents (HP / Shot / Defense), game-displayed range.
    /// The save stores a raw byte (0..=255); an "unrestricted" mode may expose that later.
    pub const IV_MIN: u8 = 0;
    pub const IV_MAX: u8 = 100;

    /// Work Suitability level. 1.0 raised this to 10; the editor lets any job reach 10.
    pub const WORK_SUITABILITY_MIN: u8 = 0;
    pub const WORK_SUITABILITY_MAX: u8 = 10;

    /// Pal Souls rank per stat (Statue of Power): +3%/rank, +30% max.
    pub const SOULS_RANK_MIN: u8 = 0;
    pub const SOULS_RANK_MAX: u8 = 10;

    /// Condensation rank, shown as 0..=4 gold stars.
    pub const CONDENSATION_MIN: u8 = 0;
    pub const CONDENSATION_MAX: u8 = 4;

    /// Equipped Active Skills (moves) per pal.
    pub const EQUIPPED_MOVES_MAX: usize = 3;

    /// Passive skills per pal.
    pub const PASSIVES_MAX: usize = 4;

    /// GlobalPalStorage.sav fixed slot count (empty slot = CharacterID "None").
    pub const GLOBAL_BOX_SLOTS: usize = 960;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_present() {
        assert!(!version().is_empty());
    }

    #[test]
    fn one_zero_limits_match_specs() {
        assert_eq!(limits::LEVEL_MAX, 80);
        assert_eq!(limits::WORK_SUITABILITY_MAX, 10);
        assert_eq!(limits::SOULS_RANK_MAX, 10);
        assert_eq!(limits::CONDENSATION_MAX, 4);
        assert_eq!(limits::GLOBAL_BOX_SLOTS, 960);
    }
}

//! Palbox Studio core engine (headless).
//!
//! The Palworld 1.0 Global Pal box domain model, save load/write, and mutations.
//! This crate MUST stay usable without any UI (CLI, tests) and MUST NOT depend on
//! Tauri or the frontend — that separation is the whole point (see docs/DIRECTION.md).
//!
//! Patch-sensitive facts and editor ranges are loaded from the generated
//! reference database; binary save-format behavior remains in this crate.

pub mod globalbox;
pub mod pal;
pub mod projection;
pub mod reference;
pub mod save;
pub mod schema;
pub mod session;
pub mod ue;

/// Core crate version — surfaced to the UI as a smoke test that the bridge is wired up.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
pub(crate) fn test_reference_catalog() -> reference::ReferenceCatalog {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data")
        .join("palbox-reference.db");
    reference::ReferenceDatabase::open(path)
        .expect("open committed reference DB")
        .load_catalog()
        .expect("load validated reference catalog")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_present() {
        assert!(!version().is_empty());
    }
}

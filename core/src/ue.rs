//! `uesave` types specialized to Palworld, plus the minimal typed accessors we
//! need. `uesave` is generic over the game; Palworld's typed structs only exist
//! when the `Palworld` game type is threaded through, so we alias the
//! game-parameterized types once and everything else refers to `crate::ue::*`.
//!
//! Scope: these are generic GVAS accessors; the only save we ever touch is
//! `GlobalPalStorage.sav` (see globalbox.rs). No world/Level.sav.

use uesave::games::palworld::Palworld;
use uesave::PropertyKey;

pub type Arch = uesave::SaveGameArchiveType<Palworld>;
pub type Save = uesave::Save<Palworld>;
pub type Property = uesave::Property<Arch>;
pub type Properties = uesave::Properties<Arch>;
pub type StructValue = uesave::StructValue<Arch>;
pub type ValueVec = uesave::ValueVec<Arch>;

/// Look up a property by name in a properties map.
pub fn prop<'a>(props: &'a Properties, key: &str) -> Option<&'a Property> {
    props.0.get(&PropertyKey::from(key))
}

/// Text of a `Str` / `Name` / `Enum` property.
pub fn as_str(p: &Property) -> Option<&str> {
    match p {
        Property::Str(text) | Property::Name(text) | Property::Enum(text) => Some(text),
        _ => None,
    }
}

/// Value of a `Byte` property (e.g. a pal's Level).
pub fn as_byte(p: &Property) -> Option<u8> {
    match p {
        Property::Byte(uesave::Byte::Byte(v)) => Some(*v),
        _ => None,
    }
}

/// Nested properties of a user `Struct` property.
pub fn struct_props(p: &Property) -> Option<&Properties> {
    match p {
        Property::Struct(StructValue::Struct(props)) => Some(props),
        _ => None,
    }
}

/// The `Vec<StructValue>` of an array-of-structs property.
pub fn array_structs(p: &Property) -> Option<&Vec<StructValue>> {
    match p {
        Property::Array(ValueVec::Struct(values)) => Some(values),
        _ => None,
    }
}

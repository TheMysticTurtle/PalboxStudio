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

pub fn as_bool(p: &Property) -> Option<bool> {
    match p {
        Property::Bool(v) => Some(*v),
        _ => None,
    }
}
pub fn as_i32(p: &Property) -> Option<i32> {
    match p {
        Property::Int(v) => Some(*v),
        _ => None,
    }
}
pub fn as_i64(p: &Property) -> Option<i64> {
    match p {
        Property::Int64(v) => Some(*v),
        Property::Int(v) => Some(*v as i64),
        _ => None,
    }
}
pub fn as_f32(p: &Property) -> Option<f32> {
    match p {
        Property::Float(uesave::Float(v)) => Some(*v),
        _ => None,
    }
}
/// Array of enum strings (e.g. EquipWaza / MasteredWaza move ids).
pub fn enum_values(p: &Property) -> Option<&Vec<String>> {
    match p {
        Property::Array(ValueVec::Enum(v)) => Some(v),
        _ => None,
    }
}
/// Array of Name strings (e.g. PassiveSkillList passive codes).
pub fn name_values(p: &Property) -> Option<&Vec<String>> {
    match p {
        Property::Array(ValueVec::Name(v)) => Some(v),
        _ => None,
    }
}
/// Mutable nested properties of a user Struct property.
pub fn struct_props_mut(p: &mut Property) -> Option<&mut Properties> {
    match p {
        Property::Struct(StructValue::Struct(props)) => Some(props),
        _ => None,
    }
}
/// A `FixedPoint64` stat field: the bare struct `{ Value: Int64(n) }` (e.g. Hp).
pub fn fixed_point64(p: &Property) -> Option<i64> {
    let inner = struct_props(p)?;
    as_i64(inner.0.get(&PropertyKey::from("Value"))?)
}

// ---- write-side ports: property constructors + set/remove ----
pub fn str_prop(v: &str) -> Property {
    Property::Str(v.to_string())
}
pub fn name_prop(v: &str) -> Property {
    Property::Name(v.to_string())
}
pub fn enum_prop(v: &str) -> Property {
    Property::Enum(v.to_string())
}
pub fn bool_prop(v: bool) -> Property {
    Property::Bool(v)
}
pub fn int_prop(v: i32) -> Property {
    Property::Int(v)
}
pub fn int64_prop(v: i64) -> Property {
    Property::Int64(v)
}
pub fn float_prop(v: f32) -> Property {
    Property::Float(uesave::Float(v))
}
pub fn byte_prop(v: u8) -> Property {
    Property::Byte(uesave::Byte::Byte(v))
}
pub fn name_array_prop(values: Vec<String>) -> Property {
    Property::Array(ValueVec::Name(values))
}
pub fn enum_array_prop(values: Vec<String>) -> Property {
    Property::Array(ValueVec::Enum(values))
}

/// Mutable property lookup by name.
pub fn prop_mut<'a>(props: &'a mut Properties, key: &str) -> Option<&'a mut Property> {
    props.0.get_mut(&PropertyKey::from(key))
}
/// Mutable `Vec<StructValue>` of an array-of-structs property.
pub fn array_structs_mut(p: &mut Property) -> Option<&mut Vec<StructValue>> {
    match p {
        Property::Array(ValueVec::Struct(values)) => Some(values),
        _ => None,
    }
}

/// Insert or overwrite a property by name (keeps position if it already exists).
pub fn set_prop(props: &mut Properties, key: &str, value: Property) {
    props.0.insert(PropertyKey::from(key), value);
}
/// Remove a property by name (to leave a field at its save-default / absent state).
pub fn remove_prop(props: &mut Properties, key: &str) {
    props.0.shift_remove(&PropertyKey::from(key));
}

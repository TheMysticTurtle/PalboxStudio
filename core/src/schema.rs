//! Canonical property schemas for fields Palbox Studio is allowed to create.
//!
//! `uesave` separates property values from the Unreal property tags that describe
//! how those values are serialized. Tags discovered while reading the source save
//! are retained in `Save::schemas`, but an optional field that was absent from
//! every Pal has no discovered tag. Creating that field without registering its
//! schema makes encoding fail with `MissingPropertySchema`.
//!
//! Keep this registry at the engine write boundary. It is deliberately
//! insert-only: a schema read from the user's save is always authoritative.

use crate::save::PalSave;
use uesave::{
    FGuid, PropertySchemas, PropertyTagDataPartial, PropertyTagPartial, PropertyType, StructType,
};

const BOX: &str = "SaveParameterArray";
const PAL: &str = "SaveParameterArray.SaveParameter";

fn tag(data: PropertyTagDataPartial) -> PropertyTagPartial {
    PropertyTagPartial { id: None, data }
}

fn scalar(kind: PropertyType) -> PropertyTagDataPartial {
    PropertyTagDataPartial::Other(kind)
}

fn byte() -> PropertyTagDataPartial {
    PropertyTagDataPartial::Byte(None)
}

fn enumeration(name: &str) -> PropertyTagDataPartial {
    PropertyTagDataPartial::Enum(name.to_string(), None)
}

fn structure(name: &str) -> PropertyTagDataPartial {
    PropertyTagDataPartial::Struct {
        struct_type: StructType::Struct(Some(name.to_string())),
        id: FGuid::nil(),
    }
}

fn guid() -> PropertyTagDataPartial {
    PropertyTagDataPartial::Struct {
        struct_type: StructType::Guid,
        id: FGuid::nil(),
    }
}

fn array(element: PropertyTagDataPartial) -> PropertyTagDataPartial {
    PropertyTagDataPartial::Array(Box::new(element))
}

fn record_missing(schemas: &mut PropertySchemas, path: &str, data: PropertyTagDataPartial) -> bool {
    if schemas.get(path).is_some() {
        return false;
    }
    schemas.record(path.to_string(), tag(data));
    true
}

/// Ensure every schema for a property the current engine may create is present.
///
/// Returns the number of schemas added. Existing source schemas are never
/// overwritten, including unusual-but-valid property representations.
pub fn ensure_writable_schemas(save: &mut PalSave) -> usize {
    let schemas = &mut save.schemas;
    let mut added = 0;

    macro_rules! ensure {
        ($path:expr, $data:expr) => {
            added += usize::from(record_missing(schemas, $path, $data));
        };
    }

    // Global box and slot identity fields used by add / clone / delete.
    ensure!(BOX, array(structure("PalGlobalPalStorageSaveParameter")));
    ensure!("SaveParameterArray.InstanceId", structure("PalInstanceID"));
    ensure!("SaveParameterArray.InstanceId.InstanceId", guid());
    ensure!(PAL, structure("PalIndividualCharacterSaveParameter"));
    ensure!(
        "SaveParameterArray.SaveParameter.SlotId",
        structure("PalCharacterSlotId")
    );
    ensure!(
        "SaveParameterArray.SaveParameter.SlotId.ContainerId",
        structure("PalContainerId")
    );
    ensure!(
        "SaveParameterArray.SaveParameter.SlotId.ContainerId.ID",
        guid()
    );
    ensure!(
        "SaveParameterArray.SaveParameter.SlotId.SlotIndex",
        scalar(PropertyType::IntProperty)
    );

    // Scalar and enum properties written by the Pal mutation ports.
    ensure!(
        "SaveParameterArray.SaveParameter.CharacterID",
        scalar(PropertyType::NameProperty)
    );
    ensure!("SaveParameterArray.SaveParameter.Level", byte());
    ensure!(
        "SaveParameterArray.SaveParameter.NickName",
        scalar(PropertyType::StrProperty)
    );
    ensure!(
        "SaveParameterArray.SaveParameter.FilteredNickName",
        scalar(PropertyType::StrProperty)
    );
    ensure!(
        "SaveParameterArray.SaveParameter.Gender",
        enumeration("EPalGenderType")
    );
    for field in [
        "Talent_HP",
        "Talent_Shot",
        "Talent_Defense",
        "Rank_HP",
        "Rank_Attack",
        "Rank_Defence",
        "Rank_CraftSpeed",
        "Rank",
    ] {
        ensure!(&format!("{PAL}.{field}"), byte());
    }
    ensure!(
        "SaveParameterArray.SaveParameter.SanityValue",
        scalar(PropertyType::FloatProperty)
    );
    ensure!(
        "SaveParameterArray.SaveParameter.FullStomach",
        scalar(PropertyType::FloatProperty)
    );
    ensure!(
        "SaveParameterArray.SaveParameter.FriendshipPoint",
        scalar(PropertyType::IntProperty)
    );
    ensure!(
        "SaveParameterArray.SaveParameter.IsRarePal",
        scalar(PropertyType::BoolProperty)
    );

    // FixedPoint64 HP is a nested user struct.
    ensure!(
        "SaveParameterArray.SaveParameter.Hp",
        structure("FixedPoint64")
    );
    ensure!(
        "SaveParameterArray.SaveParameter.Hp.Value",
        scalar(PropertyType::Int64Property)
    );

    // Collections written by passives, move, and Work Suitability mutations.
    ensure!(
        "SaveParameterArray.SaveParameter.PassiveSkillList",
        array(scalar(PropertyType::NameProperty))
    );
    ensure!(
        "SaveParameterArray.SaveParameter.EquipWaza",
        array(enumeration(""))
    );
    ensure!(
        "SaveParameterArray.SaveParameter.MasteredWaza",
        array(enumeration(""))
    );
    ensure!(
        "SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList",
        array(structure("PalWorkSuitabilityInfo"))
    );
    ensure!(
        "SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.WorkSuitability",
        enumeration("EPalWorkSuitability")
    );
    ensure!(
        "SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.Rank",
        scalar(PropertyType::IntProperty)
    );

    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use uesave::{PropertyTagDataPartial, PropertyType};

    #[test]
    fn record_missing_is_insert_only() {
        let mut schemas = PropertySchemas::new();
        let source = tag(scalar(PropertyType::StrProperty));
        schemas.record(
            "SaveParameterArray.SaveParameter.CharacterID".to_string(),
            source.clone(),
        );

        assert!(!record_missing(
            &mut schemas,
            "SaveParameterArray.SaveParameter.CharacterID",
            scalar(PropertyType::NameProperty),
        ));
        assert_eq!(
            schemas.get("SaveParameterArray.SaveParameter.CharacterID"),
            Some(&source),
        );
    }

    #[test]
    fn suitability_schema_shapes_match_palworld() {
        let list = tag(array(structure("PalWorkSuitabilityInfo")));
        let job = tag(enumeration("EPalWorkSuitability"));
        let rank = tag(scalar(PropertyType::IntProperty));

        assert!(matches!(
            list.data,
            PropertyTagDataPartial::Array(inner)
                if matches!(*inner, PropertyTagDataPartial::Struct {
                    struct_type: StructType::Struct(Some(ref name)),
                    id,
                } if name == "PalWorkSuitabilityInfo" && id.is_nil())
        ));
        assert_eq!(
            job.data,
            PropertyTagDataPartial::Enum("EPalWorkSuitability".to_string(), None),
        );
        assert_eq!(
            rank.data,
            PropertyTagDataPartial::Other(PropertyType::IntProperty),
        );
    }
}

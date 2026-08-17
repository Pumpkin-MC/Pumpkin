use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

#[derive(Deserialize)]
struct DamageTypeEntry {
    id: u8,
}

pub fn build() -> String {
    let damage_types: BTreeMap<String, DamageTypeEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/damage_type.json").unwrap())
            .expect("Failed to parse damage_type.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("damage-types");

    let mut damage_type_enum = Enum::empty();
    let mut entries: Vec<(&String, &DamageTypeEntry)> = damage_types.iter().collect();
    entries.sort_by_key(|(_, entry)| entry.id);

    for (raw_name, _) in entries {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(raw_name)
            .replace('_', "-");
        damage_type_enum.case(name);
    }

    interface.type_def(TypeDef::new(
        "damage-type",
        TypeDefKind::Enum(damage_type_enum),
    ));
    package.interface(interface);

    package.to_string()
}

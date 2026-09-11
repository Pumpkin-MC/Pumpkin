use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let dir = std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/damage_type");
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("Missing damage_type directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("damage-types");
    let mut damage_type_enum = Enum::empty();

    for raw_name in names {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(&raw_name)
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

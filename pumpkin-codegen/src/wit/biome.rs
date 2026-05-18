use semver::Version;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let json: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&fs::read_to_string("../assets/biome.json").unwrap())
            .expect("Failed to parse biome.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("biomes");

    let mut biome_enum = Enum::empty();
    let mut biome_vec = json.keys().collect::<Vec<_>>();
    biome_vec.sort();

    for raw_name in biome_vec {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(raw_name)
            .replace('_', "-");
        biome_enum.case(name);
    }

    interface.type_def(TypeDef::new(
        "biome",
        TypeDefKind::Enum(biome_enum),
    ));
    package.interface(interface);

    package.to_string()
}

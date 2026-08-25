use semver::Version;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let dir = std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/worldgen/biome");
    let mut biome_vec: Vec<String> = fs::read_dir(dir)
        .expect("Missing worldgen/biome directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    biome_vec.sort();

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("biomes");

    let mut biome_enum = Enum::empty();

    for raw_name in biome_vec {
        let name = raw_name.replace('_', "-");
        biome_enum.case(name);
    }

    interface.type_def(TypeDef::new("biome", TypeDefKind::Enum(biome_enum)));
    package.interface(interface);

    package.to_string()
}

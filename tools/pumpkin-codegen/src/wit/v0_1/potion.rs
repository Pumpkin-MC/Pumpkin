use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

#[derive(Deserialize)]
struct PotionRaw {
    id: u8,
}

pub fn build() -> String {
    let raw_map: BTreeMap<String, PotionRaw> =
        serde_json::from_str(&fs::read_to_string("../../assets/potion.json").unwrap())
            .expect("Failed to parse potion.json");

    let mut potions: Vec<(String, PotionRaw)> = raw_map.into_iter().collect();
    potions.sort_by_key(|(_, raw)| raw.id);

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("potions");
    let mut potion_enum = Enum::empty();

    for (name, _) in potions {
        potion_enum.case(name.replace('_', "-"));
    }

    interface.type_def(TypeDef::new("potion-type", TypeDefKind::Enum(potion_enum)));
    package.interface(interface);

    package.to_string()
}

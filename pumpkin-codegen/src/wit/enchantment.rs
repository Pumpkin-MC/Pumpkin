use semver::Version;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let enchantments: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&fs::read_to_string("../assets/enchantments.json").unwrap())
            .expect("Failed to parse enchantments.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("enchantments");

    let mut enchantment_enum = Enum::empty();
    let mut enchantment_vec = enchantments.keys().collect::<Vec<_>>();
    enchantment_vec.sort();

    for raw_name in enchantment_vec {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(raw_name)
            .replace('_', "-");
        enchantment_enum.case(name);
    }

    interface.type_def(TypeDef::new(
        "enchantment",
        TypeDefKind::Enum(enchantment_enum),
    ));
    package.interface(interface);

    package.to_string()
}

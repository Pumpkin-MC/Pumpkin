use heck::ToKebabCase;
use semver::Version;
use std::{collections::BTreeMap, fs};
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let events: BTreeMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../../assets/entity_statuses.json").unwrap())
            .expect("Failed to parse entity_statuses.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("entity-statuses");

    let mut status_enum = Enum::empty();
    for (event_name, _) in events {
        status_enum.case(event_name.to_kebab_case());
    }

    interface.type_def(TypeDef::new(
        "entity-status",
        TypeDefKind::Enum(status_enum),
    ));
    package.interface(interface);

    package.to_string()
}

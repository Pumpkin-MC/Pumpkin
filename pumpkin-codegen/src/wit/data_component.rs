use semver::Version;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let data_components: BTreeMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("data-components");

    let mut component_enum = Enum::empty();
    let mut data_component_vec = data_components.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| **i);

    for (raw_name, _) in data_component_vec {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(raw_name)
            .replace('_', "-")
            .replace('/', "-");
        component_enum.case(name);
    }

    interface.type_def(TypeDef::new(
        "data-component",
        TypeDefKind::Enum(component_enum),
    ));
    package.interface(interface);

    package.to_string()
}

use semver::Version;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let screens: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/screens.json").unwrap())
            .expect("Failed to parse screens.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("screens");

    let mut screen_enum = Enum::empty();
    for screen in screens {
        screen_enum.case(screen.replace('_', "-"));
    }

    interface.type_def(TypeDef::new("screen", TypeDefKind::Enum(screen_enum)));
    package.interface(interface);

    package.to_string()
}

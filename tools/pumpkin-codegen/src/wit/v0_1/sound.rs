use semver::Version;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let sounds: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/sounds.json").unwrap())
            .expect("Failed to parse sounds.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("sounds");

    let mut category_enum = Enum::empty();
    category_enum.case("master");
    category_enum.case("music");
    category_enum.case("records");
    category_enum.case("weather");
    category_enum.case("blocks");
    category_enum.case("hostile");
    category_enum.case("neutral");
    category_enum.case("players");
    category_enum.case("ambient");
    category_enum.case("voice");
    category_enum.case("ui");

    interface.type_def(TypeDef::new(
        "sound-category",
        TypeDefKind::Enum(category_enum),
    ));

    let mut sound_enum = Enum::empty();
    for sound in sounds {
        sound_enum.case(sound.replace(['_', '.'], "-"));
    }

    interface.type_def(TypeDef::new("sound", TypeDefKind::Enum(sound_enum)));
    package.interface(interface);

    package.to_string()
}

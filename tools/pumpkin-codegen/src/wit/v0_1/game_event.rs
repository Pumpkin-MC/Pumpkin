use semver::Version;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let game_events: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_event.json").unwrap())
            .expect("Failed to parse game_event.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("game-events");

    let mut game_event_enum = Enum::empty();
    for game_event in game_events {
        let name = game_event
            .strip_prefix("minecraft:")
            .unwrap_or(&game_event)
            .replace('_', "-");
        game_event_enum.case(name);
    }

    interface.type_def(TypeDef::new(
        "game-event",
        TypeDefKind::Enum(game_event_enum),
    ));
    package.interface(interface);

    package.to_string()
}

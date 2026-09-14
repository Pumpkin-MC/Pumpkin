use semver::Version;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{
    Enum, Interface, Package, PackageName, Type, TypeDef, TypeDefKind, Variant, VariantCase,
};

pub fn build() -> String {
    let game_rules: BTreeMap<String, Value> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_rules.json").unwrap())
            .expect("Failed to parse game_rules.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("game-rules");

    let mut rule_enum = Enum::empty();
    for raw_name in game_rules.keys() {
        let name = raw_name.replace('_', "-");
        rule_enum.case(name);
    }

    interface.type_def(TypeDef::new("game-rule", TypeDefKind::Enum(rule_enum)));

    let mut value_variant = Variant::empty();
    value_variant.case(VariantCase::value("int", Type::S32));
    value_variant.case(VariantCase::value("bool", Type::Bool));

    interface.type_def(TypeDef::new(
        "game-rule-value",
        TypeDefKind::Variant(value_variant),
    ));

    package.interface(interface);

    package.to_string()
}

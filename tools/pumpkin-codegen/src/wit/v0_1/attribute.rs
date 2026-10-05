use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, Record, TypeDef, TypeDefKind};

#[derive(Deserialize)]
struct AttributeEntry {
    id: u8,
    default_value: f64,
}

pub fn build() -> String {
    let attributes: BTreeMap<String, AttributeEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/attributes.json").unwrap())
            .expect("Failed to parse attributes.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("attributes");

    let mut sorted_attributes: Vec<(String, AttributeEntry)> = attributes.into_iter().collect();
    sorted_attributes.sort_by_key(|(_, raw)| raw.id);

    let mut attribute_enum = Enum::empty();
    for (raw_name, _) in sorted_attributes {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(&raw_name)
            .replace('_', "-");
        attribute_enum.case(name);
    }

    interface.type_def(TypeDef::new("attribute", TypeDefKind::Enum(attribute_enum)));

    let mut modifier_op_enum = Enum::empty();
    modifier_op_enum.case("add");
    modifier_op_enum.case("multiply-base");
    modifier_op_enum.case("multiply-total");

    interface.type_def(TypeDef::new(
        "modifier-operation",
        TypeDefKind::Enum(modifier_op_enum),
    ));

    let modifier_record = Record::new([
        wit_encoder::Field::new("id", wit_encoder::Type::String),
        wit_encoder::Field::new("amount", wit_encoder::Type::F64),
        wit_encoder::Field::new(
            "operation",
            wit_encoder::Type::Named(wit_encoder::Ident::new("modifier-operation")),
        ),
    ]);

    interface.type_def(TypeDef::new(
        "attribute-modifier",
        TypeDefKind::Record(modifier_record),
    ));

    package.interface(interface);

    package.to_string()
}

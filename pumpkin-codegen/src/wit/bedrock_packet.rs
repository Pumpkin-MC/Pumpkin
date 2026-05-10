use crate::wit::utils::map_type;
use heck::ToKebabCase;
use semver::Version;
use std::{fs, path::Path};
use syn::{Fields, Item};
use wit_encoder::{
    Field, Interface, Package, PackageName, Record, Type as WitType, TypeDef, TypeDefKind, Variant,
    VariantCase,
};

pub fn build() -> String {
    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("bedrock-packets");

    let mut serverbound_variant = Variant::empty();
    let mut clientbound_variant = Variant::empty();

    // Process serverbound packets
    process_packets(
        "../pumpkin-protocol/src/bedrock/server",
        &mut interface,
        &mut serverbound_variant,
    );
    // Process clientbound packets
    process_packets(
        "../pumpkin-protocol/src/bedrock/client",
        &mut interface,
        &mut clientbound_variant,
    );

    interface.type_def(TypeDef::new(
        "serverbound-packet",
        TypeDefKind::Variant(serverbound_variant),
    ));
    interface.type_def(TypeDef::new(
        "clientbound-packet",
        TypeDefKind::Variant(clientbound_variant),
    ));

    package.interface(interface);
    package.to_string()
}

fn process_packets(dir: &str, interface: &mut Interface, variant: &mut Variant) {
    let paths = fs::read_dir(dir).expect("Failed to read packet directory");
    let mut sorted_paths: Vec<_> = paths
        .map(|e| e.expect("Failed to read entry").path())
        .collect();
    sorted_paths.sort();

    for path in sorted_paths {
        if path.is_dir() {
            process_packets(path.to_str().unwrap(), interface, variant);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().is_some_and(|name| name != "mod.rs")
        {
            parse_packet_file(&path, interface, variant);
        }
    }
}

fn parse_packet_file(path: &Path, interface: &mut Interface, variant: &mut Variant) {
    let content = fs::read_to_string(path).expect("Failed to read file");
    let file = syn::parse_file(&content).expect("Failed to parse file");

    for item in file.items {
        if let Item::Struct(s) = item {
            // Only process structs with #[packet] attribute
            if !s.attrs.iter().any(|attr| attr.path().is_ident("packet")) {
                continue;
            }

            let struct_name = s.ident.to_string();
            let mut fields_list = Vec::new();

            if let Fields::Named(fields) = s.fields {
                for field in fields.named {
                    let field_name = field.ident.as_ref().unwrap().to_string().to_kebab_case();
                    let field_type = map_type(&field.ty);
                    fields_list.push(Field::new(field_name, field_type));
                }
            }

            let wit_name = struct_name.to_kebab_case();
            if !fields_list.is_empty() {
                interface.type_def(TypeDef::new(
                    wit_name.clone(),
                    TypeDefKind::Record(Record::new(fields_list)),
                ));
                variant.case(VariantCase::value(
                    wit_name.clone(),
                    WitType::named(wit_name),
                ));
            } else {
                variant.case(VariantCase::empty(wit_name));
            }
        }
    }
}

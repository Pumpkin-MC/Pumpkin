use crate::wit::utils::map_type;
use heck::ToKebabCase;
use semver::Version;
use std::collections::HashSet;
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

    let mut interface = Interface::new("java-packets");

    interface.use_type("uuid", "uuid", None);

    let mut serverbound_variant = Variant::empty();
    let mut clientbound_variant = Variant::empty();
    let mut serverbound_cases = HashSet::new();
    let mut clientbound_cases = HashSet::new();

    // Process serverbound packets
    let server_states = &["config", "handshake", "login", "play", "status"];
    for state in server_states {
        process_packets(
            &format!("../pumpkin-protocol/src/java/server/{}", state),
            state,
            &mut interface,
            &mut serverbound_variant,
            &mut serverbound_cases,
        );
    }
    // Process clientbound packets
    let client_states = &["config", "login", "play", "status"];
    for state in client_states {
        process_packets(
            &format!("../pumpkin-protocol/src/java/client/{}", state),
            state,
            &mut interface,
            &mut clientbound_variant,
            &mut clientbound_cases,
        );
    }

    // Add an 'unknown' fallback variant (no payload) — raw payload is carried on the event record
    serverbound_variant.case(VariantCase::empty("unknown"));
    clientbound_variant.case(VariantCase::empty("unknown"));

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

fn process_packets(
    dir: &str,
    state: &str,
    interface: &mut Interface,
    variant: &mut Variant,
    defined_cases: &mut HashSet<String>,
) {
    let paths = fs::read_dir(dir).expect("Failed to read packet directory");
    let mut sorted_paths: Vec<_> = paths
        .map(|e| e.expect("Failed to read entry").path())
        .collect();
    sorted_paths.sort();

    for path in sorted_paths {
        if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().is_some_and(|name| name != "mod.rs")
        {
            parse_packet_file(&path, state, interface, variant, defined_cases);
        }
    }
}

fn parse_packet_file(
    path: &Path,
    state: &str,
    interface: &mut Interface,
    variant: &mut Variant,
    defined_cases: &mut HashSet<String>,
) {
    let content = fs::read_to_string(path).expect("Failed to read file");
    let file = syn::parse_file(&content).expect("Failed to parse file");

    for item in file.items {
        if let Item::Struct(s) = item {
            // Only process structs with #[java_packet] attribute
            if !s
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("java_packet"))
            {
                continue;
            }

            let struct_name = s.ident.to_string();
            let wit_name = if state == "play" {
                struct_name.to_kebab_case()
            } else {
                format!("{}-{}", state, struct_name.to_kebab_case())
            };

            if !defined_cases.insert(wit_name.clone()) {
                continue;
            }

            let mut fields_list = Vec::new();

            if let Fields::Named(fields) = s.fields {
                for field in fields.named {
                    let type_name = match &field.ty {
                        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                        syn::Type::Reference(r) => match &*r.elem {
                            syn::Type::Slice(s) => match &*s.elem {
                                syn::Type::Path(p) => {
                                    p.path.segments.last().unwrap().ident.to_string()
                                }
                                _ => String::new(),
                            },
                            syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                            _ => String::new(),
                        },
                        _ => String::new(),
                    };

                    if type_name == "DynamicRecipe" {
                        continue;
                    }

                    let field_name = field.ident.as_ref().unwrap().to_string().to_kebab_case();
                    let field_type = map_type(&field.ty);
                    fields_list.push(Field::new(field_name, field_type));
                }
            }
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

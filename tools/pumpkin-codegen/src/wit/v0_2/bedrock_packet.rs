use crate::wit::utils::{
    RustFilesFinder, collect_fields, extract_doc_comments, is_valid_helper_enum,
    is_valid_helper_struct, process_helper_enum, process_helper_struct, translate_enum_to_wit,
};
use heck::ToKebabCase;
use semver::Version;
use std::collections::HashSet;
use std::{fs, path::Path};
use syn::{Fields, Item};
use wit_encoder::{
    Interface, Package, PackageName, Record, Type as WitType, TypeDef, TypeDefKind, Variant,
    VariantCase,
};

#[derive(Clone, Copy, Debug)]
enum FileCategory {
    Server,
    Client,
    HelperTypes,
}

pub fn build() -> String {
    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 2, 0)),
    ));
    let mut interface = Interface::new("bedrock-packets");

    interface.use_type("common", "inexpressible-host-type", None);
    interface.use_type("uuid", "uuid", None);
    interface.use_type("common", "nbt-tree", None);

    let mut serverbound_variant = Variant::empty();
    let mut clientbound_variant = Variant::empty();
    let mut processed_types = HashSet::new();

    let dirs = [
        (
            Path::new("../../crates/pumpkin-protocol/src/bedrock/server"),
            FileCategory::Server,
        ),
        (
            Path::new("../../crates/pumpkin-protocol/src/bedrock/client"),
            FileCategory::Client,
        ),
        (
            Path::new("../../crates/pumpkin-protocol/src/bedrock/network_item.rs"),
            FileCategory::HelperTypes,
        ),
    ];

    let syn_items = dirs
        .iter()
        .flat_map(|(dir, category)| RustFilesFinder::new(dir).map(move |path| (path, category)))
        .flat_map(|(path, category)| {
            let contents = fs::read_to_string(path).expect("Failed to read file");
            let parsed = syn::parse_file(&contents).expect("Failed to parse file");

            parsed.items.into_iter().map(move |item| (item, category))
        })
        .filter(|(item, _)| match item {
            Item::Struct(s) => has_static_packet_attr(&s.attrs) || is_valid_helper_struct(&s),
            Item::Enum(e) => has_static_packet_attr(&e.attrs) || is_valid_helper_enum(&e),
            _ => false,
        })
        .collect::<Vec<_>>();

    let defined_types = syn_items
        .iter()
        .map(|(item, _)| match item {
            Item::Struct(s) => s.ident.to_string().to_kebab_case(),
            Item::Enum(e) => e.ident.to_string().to_kebab_case(),
            _ => unreachable!(),
        })
        .collect::<HashSet<_>>();

    for (item, category) in syn_items {
        let variant = match category {
            FileCategory::Server => &mut serverbound_variant,
            _ => &mut clientbound_variant,
        };
        match item {
            Item::Struct(s) if has_static_packet_attr(&s.attrs) => {
                process_struct(
                    s,
                    &mut interface,
                    variant,
                    &mut processed_types,
                    &defined_types,
                );
            }
            Item::Enum(e) if has_static_packet_attr(&e.attrs) => {
                process_enum(
                    e,
                    &mut interface,
                    variant,
                    &mut processed_types,
                    &defined_types,
                );
            }
            Item::Struct(s) if is_valid_helper_struct(&s) => {
                process_helper_struct(s, &mut interface, &mut processed_types, &defined_types);
            }
            Item::Enum(e) if is_valid_helper_enum(&e) => {
                process_helper_enum(e, &mut interface, &mut processed_types, &defined_types);
            }
            _ => unreachable!(),
        }
    }

    // Add an 'unknown' fallback variant (no payload)
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

#[inline]
#[must_use]
/// Check for `#[packet]` attribute
fn has_static_packet_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("packet"))
}

fn process_struct(
    s: syn::ItemStruct,
    interface: &mut Interface,
    variant: &mut Variant,
    processed_types: &mut HashSet<String>,
    defined_types: &HashSet<String>,
) {
    let wit_name = s.ident.to_string().to_kebab_case();
    if !processed_types.insert(wit_name.clone()) {
        panic!("Encountered duplicate type: {}", wit_name);
    }

    let fields_list = match s.fields {
        Fields::Named(fields) => collect_fields(&fields.named, defined_types),
        _ => Vec::new(),
    };

    let docs = extract_doc_comments(&s.attrs);

    let mut case = if !fields_list.is_empty() {
        let mut def = TypeDef::new(
            wit_name.clone(),
            TypeDefKind::Record(Record::new(fields_list)),
        );
        def.set_docs(docs.clone());
        interface.type_def(def);

        VariantCase::value(wit_name.clone(), WitType::named(wit_name))
    } else {
        VariantCase::empty(wit_name)
    };

    case.set_docs(docs);
    variant.case(case);
}

fn process_enum(
    e: syn::ItemEnum,
    interface: &mut Interface,
    variant: &mut Variant,
    processed_types: &mut HashSet<String>,
    defined_types: &HashSet<String>,
) {
    let enum_wit_name = e.ident.to_string().to_kebab_case();
    if !processed_types.insert(enum_wit_name.clone()) {
        panic!("Encountered duplicate type: {}", enum_wit_name);
    }

    let mut case = VariantCase::value(enum_wit_name.clone(), WitType::named(enum_wit_name.clone()));
    case.set_docs(extract_doc_comments(&e.attrs));
    variant.case(case);

    let (main_def, helper_defs) = translate_enum_to_wit(e, enum_wit_name, defined_types);
    for def in helper_defs {
        if !processed_types.insert(def.name().to_string()) {
            panic!("Encountered duplicate type: {}", def.name());
        }
        interface.type_def(def);
    }
    interface.type_def(main_def);
}

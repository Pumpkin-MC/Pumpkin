use crate::wit::utils::map_type;
use heck::ToKebabCase;
use semver::Version;
use std::{fs, path::Path};
use syn::{Fields, Item};
use wit_encoder::{
    Enum, EnumCase, Field, Interface, Package, PackageName, Record, Type as WitType, TypeDef,
    TypeDefKind, Variant, VariantCase,
};

pub fn build() -> String {
    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("bedrock-packets");

    interface.use_type("uuid", "uuid", None);

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

#[inline]
fn register_wit_type(
    wit_name: String,
    fields_list: Vec<Field>,
    interface: &mut Interface,
    variant: &mut Variant,
    wit_sub_name: Option<String>,
) {
    if !fields_list.is_empty() {
        let name = if let Some(sub_name) = &wit_sub_name {
            format!("{}-{}", sub_name, wit_name)
        } else {
            wit_name
        };
        interface.type_def(TypeDef::new(
            name.clone(),
            TypeDefKind::Record(Record::new(fields_list)),
        ));
        variant.case(VariantCase::value(name.clone(), WitType::named(name)));
    } else {
        variant.case(VariantCase::empty(wit_name));
    }
}

#[inline]
#[must_use]
/// Check for `#[packet]` attribute
fn has_static_packet_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("packet"))
}

fn collect_types(
    fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<WitType> {
    fields
        .into_iter()
        .map(|field| map_type(&field.ty))
        .collect()
}

fn collect_fields(
    named_fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<Field> {
    named_fields
        .into_iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_string().to_kebab_case();
            let field_type = map_type(&field.ty);
            Field::new(field_name, field_type)
        })
        .collect()
}

fn process_struct(s: syn::ItemStruct, interface: &mut Interface, variant: &mut Variant) {
    let wit_name = s.ident.to_string().to_kebab_case();

    let fields_list = match s.fields {
        Fields::Named(fields) => collect_fields(fields.named),
        _ => Vec::new(),
    };

    register_wit_type(wit_name, fields_list, interface, variant, None);
}

fn process_enum(e: syn::ItemEnum, interface: &mut Interface, variant: &mut Variant) {
    let enum_wit_name = e.ident.to_string().to_kebab_case();
    let mut cases = Vec::new();

    for v in e.variants {
        let variant_wit_name = v.ident.to_string().to_kebab_case();

        match v.fields {
            Fields::Named(fields) => {
                let fields_list = collect_fields(fields.named);
                register_wit_type(
                    variant_wit_name,
                    fields_list,
                    interface,
                    variant,
                    Some(enum_wit_name.clone()),
                );
            }

            Fields::Unnamed(fields) => {
                let types = collect_types(fields.unnamed);
                match types.len() {
                    0 => cases.push(VariantCase::empty(variant_wit_name)),
                    1 => cases.push(VariantCase::value(
                        variant_wit_name,
                        types.into_iter().next().unwrap(),
                    )),
                    _ => cases.push(VariantCase::value(variant_wit_name, WitType::tuple(types))),
                }
            }

            Fields::Unit => {
                cases.push(VariantCase::empty(variant_wit_name));
            }
        }
    }

    variant.case(VariantCase::value(
        enum_wit_name.clone(),
        WitType::named(enum_wit_name.clone()),
    ));

    // define weather to use Enum or Variant
    let all_empty = cases.iter().all(|c| c.type_().is_none());
    if all_empty {
        interface.type_def(TypeDef::new(
            enum_wit_name,
            TypeDefKind::Enum(Enum::from_iter(
                cases.into_iter().map(|c| EnumCase::new(c.name().clone())),
            )),
        ));
    } else {
        interface.type_def(TypeDef::new(
            enum_wit_name,
            TypeDefKind::Variant(Variant::from(cases)),
        ));
    }
}

fn parse_packet_file(path: &Path, interface: &mut Interface, variant: &mut Variant) {
    let content = fs::read_to_string(path).expect("Failed to read file");
    let file = syn::parse_file(&content).expect("Failed to parse file");

    for item in file.items {
        match item {
            Item::Struct(s) if has_static_packet_attr(&s.attrs) => {
                process_struct(s, interface, variant);
            }
            Item::Enum(e) if has_static_packet_attr(&e.attrs) => {
                process_enum(e, interface, variant);
            }
            _ => {}
        }
    }
}

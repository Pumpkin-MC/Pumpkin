use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use syn::{Attribute, Expr, Fields, Item, LitInt, Type};

#[derive(Debug, Clone)]
struct PacketConst {
    direction: &'static str,
    const_name: String,
    phase: String,
    name: String,
    variant: String,
}

#[derive(Debug, Clone)]
struct PacketSchemaField {
    name: String,
    kind_expr: String,
}

#[derive(Debug, Clone)]
struct PacketSchemaDef {
    variant: String,
    fields: Vec<PacketSchemaField>,
}

fn to_camel(input: &str) -> String {
    input
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = String::new();
                    out.push(first.to_ascii_uppercase());
                    out.push_str(&chars.as_str().to_ascii_lowercase());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn parse_packet_consts(source: &str) -> Vec<PacketConst> {
    let mut packets = Vec::new();
    let mut direction: Option<&'static str> = None;

    for line in source.lines() {
        let line = line.trim();
        if line == "pub mod serverbound {" {
            direction = Some("serverbound");
            continue;
        }
        if line == "pub mod clientbound {" {
            direction = Some("clientbound");
            continue;
        }
        if line == "}" {
            direction = None;
            continue;
        }

        if let Some(active_dir) = direction
            && let Some(rest) = line.strip_prefix("pub const ")
            && let Some((const_name, _)) = rest.split_once(':')
            && let Some((phase, packet_name)) = const_name.split_once('_')
        {
            let direction_title = to_camel(active_dir);
            let variant = format!(
                "{}{}{}",
                direction_title,
                to_camel(phase),
                to_camel(packet_name)
            );

            packets.push(PacketConst {
                direction: active_dir,
                const_name: const_name.to_string(),
                phase: phase.to_ascii_lowercase(),
                name: packet_name.to_ascii_lowercase(),
                variant,
            });
        }
    }

    packets
}

fn generate_packet_keys(packets: &[PacketConst]) -> String {
    let mut out = String::new();

    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n");
    out.push_str("pub enum JavaPacketKey {\n");
    for packet in packets {
        out.push_str("    ");
        out.push_str(&packet.variant);
        out.push_str(",\n");
    }
    out.push_str("}\n\n");

    out.push_str("impl JavaPacketKey {\n");
    out.push_str("    pub fn direction(&self) -> crate::packet::PacketDirection {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        let dir = if packet.direction == "serverbound" {
            "crate::packet::PacketDirection::Serverbound"
        } else {
            "crate::packet::PacketDirection::Clientbound"
        };
        out.push_str(&format!(
            "            Self::{} => {},\n",
            packet.variant, dir
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn phase(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        out.push_str(&format!(
            "            Self::{} => \"{}\",\n",
            packet.variant, packet.phase
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn name(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        out.push_str(&format!(
            "            Self::{} => \"{}\",\n",
            packet.variant, packet.name
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn packet_id(&self) -> &'static crate::packet_ids_full::PacketId {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        out.push_str(&format!(
            "            Self::{} => &crate::packet_ids_full::{}::{},\n",
            packet.variant, packet.direction, packet.const_name
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn id_for_version(&self, version: pumpkin_util::version::MinecraftVersion) -> i32 {\n");
    out.push_str("        self.packet_id().to_id(version)\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn from_parts(\n");
    out.push_str("        direction: crate::packet::PacketDirection,\n");
    out.push_str("        phase: &str,\n");
    out.push_str("        name: &str,\n");
    out.push_str("    ) -> Option<Self> {\n");
    out.push_str("        match (direction, phase, name) {\n");
    for packet in packets {
        let dir = if packet.direction == "serverbound" {
            "crate::packet::PacketDirection::Serverbound"
        } else {
            "crate::packet::PacketDirection::Clientbound"
        };
        out.push_str(&format!(
            "            ({}, \"{}\", \"{}\") => Some(Self::{}),\n",
            dir, packet.phase, packet.name, packet.variant
        ));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn all() -> &'static [Self] {\n");
    out.push_str("        &[\n");
    for packet in packets {
        out.push_str(&format!("            Self::{},\n", packet.variant));
    }
    out.push_str("        ]\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn packet_direction_from_protocol_path(path: &Path) -> Option<&'static str> {
    path.components().find_map(|component| {
        let value = component.as_os_str().to_str()?;
        match value {
            "server" => Some("serverbound"),
            "client" => Some("clientbound"),
            _ => None,
        }
    })
}

fn java_packet_const_name(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("java_packet") {
            return None;
        }

        let expr = attr.parse_args::<Expr>().ok()?;
        match expr {
            Expr::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            _ => None,
        }
    })
}

fn type_to_schema_kind(ty: &Type, is_last: bool) -> Option<String> {
    match ty {
        Type::Reference(reference) => type_to_schema_kind(reference.elem.as_ref(), is_last),
        Type::Slice(slice) => {
            let Type::Path(path) = slice.elem.as_ref() else {
                return None;
            };
            if path.path.is_ident("u8") && is_last {
                Some("FieldType::RemainingBytes".to_string())
            } else {
                None
            }
        }
        Type::Array(array) => {
            let Type::Path(path) = array.elem.as_ref() else {
                return None;
            };
            if !path.path.is_ident("u8") {
                return None;
            }

            let Expr::Lit(expr_lit) = &array.len else {
                return None;
            };
            let syn::Lit::Int(len) = &expr_lit.lit else {
                return None;
            };
            let len = parse_array_len(len)?;
            Some(format!("FieldType::Bytes {{ len: {len} }}"))
        }
        Type::Path(path) => {
            let segment = path.path.segments.last()?;
            match segment.ident.to_string().as_str() {
                "u8" => Some("FieldType::U8".to_string()),
                "bool" => Some("FieldType::Bool".to_string()),
                "u16" => Some("FieldType::U16".to_string()),
                "i8" => Some("FieldType::I8".to_string()),
                "i16" => Some("FieldType::I16".to_string()),
                "i32" => Some("FieldType::I32".to_string()),
                "i64" => Some("FieldType::I64".to_string()),
                "f32" => Some("FieldType::F32".to_string()),
                "f64" => Some("FieldType::F64".to_string()),
                "str" | "String" | "ResourceLocation" => {
                    Some("FieldType::String { max_len: 32767 }".to_string())
                }
                "VarInt" => Some("FieldType::VarInt".to_string()),
                "VarLong" => Some("FieldType::VarLong".to_string()),
                "Uuid" => Some("FieldType::UuidBytes".to_string()),
                "BlockPos" => Some("FieldType::BlockPos".to_string()),
                "Vector3" => {
                    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                        return None;
                    };
                    let syn::GenericArgument::Type(inner_ty) = args.args.first()? else {
                        return None;
                    };
                    match inner_ty {
                        Type::Path(inner) => {
                            match inner.path.segments.last()?.ident.to_string().as_str() {
                                "f32" => Some("FieldType::Vec3F32".to_string()),
                                "f64" => Some("FieldType::Vec3F64".to_string()),
                                "i16" => Some("FieldType::Vec3I16".to_string()),
                                "i32" => Some("FieldType::Vec3I32".to_string()),
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                "Box" => {
                    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                        return None;
                    };
                    let syn::GenericArgument::Type(inner_ty) = args.args.first()? else {
                        return None;
                    };
                    match inner_ty {
                        Type::Slice(slice) => {
                            let Type::Path(inner) = slice.elem.as_ref() else {
                                return None;
                            };
                            if inner.path.is_ident("u8") && is_last {
                                Some("FieldType::RemainingBytes".to_string())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
                "Option" => {
                    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                        return None;
                    };
                    let syn::GenericArgument::Type(inner_ty) = args.args.first()? else {
                        return None;
                    };
                    let inner = type_to_schema_kind(inner_ty, is_last)?;
                    Some(format!("FieldType::Optional(Box::new({inner}))"))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_array_len(len: &LitInt) -> Option<usize> {
    len.base10_parse().ok()
}

fn parse_packet_schema_file(
    path: &Path,
    packet_by_const: &BTreeMap<String, PacketConst>,
) -> Vec<PacketSchemaDef> {
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(file) = syn::parse_file(&source) else {
        return Vec::new();
    };

    let mut out = Vec::new();

    let Some(direction) = packet_direction_from_protocol_path(path) else {
        return Vec::new();
    };

    for item in file.items {
        let Item::Struct(item_struct) = item else {
            continue;
        };

        let Some(const_name) = java_packet_const_name(&item_struct.attrs) else {
            continue;
        };
        let packet_lookup = format!("{direction}:{const_name}");
        let Some(packet) = packet_by_const.get(&packet_lookup) else {
            continue;
        };

        let fields = match &item_struct.fields {
            Fields::Unit => Vec::new(),
            Fields::Named(named) => {
                let mut fields = Vec::new();
                let mut supported = true;

                let field_count = named.named.len();
                for (index, field) in named.named.iter().enumerate() {
                    let Some(ident) = &field.ident else {
                        supported = false;
                        break;
                    };
                    let Some(kind_expr) = type_to_schema_kind(&field.ty, index + 1 == field_count)
                    else {
                        supported = false;
                        break;
                    };

                    fields.push(PacketSchemaField {
                        name: ident.to_string(),
                        kind_expr,
                    });
                }

                if !supported {
                    continue;
                }

                fields
            }
            Fields::Unnamed(_) => continue,
        };

        out.push(PacketSchemaDef {
            variant: packet.variant.clone(),
            fields,
        });
    }

    out
}

fn generate_packet_schemas(
    manifest_dir: &Path,
    packet_by_const: &BTreeMap<String, PacketConst>,
) -> String {
    let protocol_dir = manifest_dir.join("../pumpkin-protocol/src/java");
    let mut files = Vec::new();
    collect_rust_files(&protocol_dir, &mut files);
    files.sort();

    let mut schemas = Vec::new();
    for file in files {
        schemas.extend(parse_packet_schema_file(&file, packet_by_const));
    }

    let mut out = String::new();
    out.push_str("fn generated_java_packet_schema_registry() -> JavaPacketSchemaRegistry {\n");
    out.push_str("    let mut registry = JavaPacketSchemaRegistry::default();\n");
    for schema in &schemas {
        out.push_str("    registry.register(\n");
        out.push_str("        JavaPacketKey::");
        out.push_str(&schema.variant);
        out.push_str(",\n");
        out.push_str("        PacketSchema::new()");
        for field in &schema.fields {
            out.push_str(".field(");
            out.push_str(&format!("{:?}", field.name));
            out.push_str(", ");
            out.push_str(&field.kind_expr);
            out.push(')');
        }
        out.push_str(",\n");
        out.push_str("    );\n");
    }
    out.push_str("    registry\n");
    out.push_str("}\n");

    out
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let packet_generated = manifest_dir
        .join("../pumpkin-data/src/generated/packet.rs")
        .canonicalize()
        .expect("canonicalize generated packet.rs");

    println!("cargo:rerun-if-changed={}", packet_generated.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("../pumpkin-protocol/src/java").display()
    );

    let content = fs::read_to_string(&packet_generated).expect("read generated packet table");
    let packets = parse_packet_consts(&content);
    let packet_by_const = packets
        .iter()
        .cloned()
        .map(|packet| {
            (
                format!("{}:{}", packet.direction, packet.const_name),
                packet,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let generated_keys = generate_packet_keys(&packets);
    let generated_schemas = generate_packet_schemas(&manifest_dir, &packet_by_const);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(
        out_dir.join("generated_java_packet_keys.rs"),
        generated_keys,
    )
    .expect("write generated_java_packet_keys.rs");
    fs::write(
        out_dir.join("generated_java_packet_schemas.rs"),
        generated_schemas,
    )
    .expect("write generated_java_packet_schemas.rs");
}

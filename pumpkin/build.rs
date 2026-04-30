use std::{
    collections::BTreeMap,
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use syn::{Attribute, Expr, Fields, Item, LitInt, Type};

#[derive(Debug, Clone)]
struct PacketConst {
    direction: &'static str,
    const_name: String,
    phase: String,
    name: String,
}

#[derive(Debug, Clone)]
struct PacketSchemaField {
    name: String,
    kind_expr: String,
}

#[derive(Debug, Clone)]
struct PacketSchemaDef {
    direction: &'static str,
    phase: String,
    name: String,
    fields: Vec<PacketSchemaField>,
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
            packets.push(PacketConst {
                direction: active_dir,
                const_name: const_name.to_string(),
                phase: phase.to_ascii_lowercase(),
                name: packet_name.to_ascii_lowercase(),
            });
        }
    }

    packets
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

fn parse_array_len(len: &LitInt) -> Option<usize> {
    len.base10_parse().ok()
}

fn type_to_schema_kind(ty: &Type, is_last: bool) -> Option<String> {
    match ty {
        Type::Reference(reference) => type_to_schema_kind(reference.elem.as_ref(), is_last),
        Type::Slice(slice) => {
            let Type::Path(path) = slice.elem.as_ref() else {
                return None;
            };
            (path.path.is_ident("u8") && is_last)
                .then(|| "LocalFieldType::RemainingBytes".to_string())
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
            Some(format!("LocalFieldType::Bytes {{ len: {len} }}"))
        }
        Type::Path(path) => {
            let segment = path.path.segments.last()?;
            match segment.ident.to_string().as_str() {
                "u8" => Some("LocalFieldType::U8".to_string()),
                "bool" => Some("LocalFieldType::Bool".to_string()),
                "u16" => Some("LocalFieldType::U16".to_string()),
                "i8" => Some("LocalFieldType::I8".to_string()),
                "i16" => Some("LocalFieldType::I16".to_string()),
                "i32" => Some("LocalFieldType::I32".to_string()),
                "i64" => Some("LocalFieldType::I64".to_string()),
                "f32" => Some("LocalFieldType::F32".to_string()),
                "f64" => Some("LocalFieldType::F64".to_string()),
                "str" | "String" | "ResourceLocation" => {
                    Some("LocalFieldType::String { max_len: 32767 }".to_string())
                }
                "VarInt" => Some("LocalFieldType::VarInt".to_string()),
                "VarLong" => Some("LocalFieldType::VarLong".to_string()),
                "Uuid" => Some("LocalFieldType::UuidBytes".to_string()),
                "BlockPos" => Some("LocalFieldType::BlockPos".to_string()),
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
                                "f32" => Some("LocalFieldType::Vec3F32".to_string()),
                                "f64" => Some("LocalFieldType::Vec3F64".to_string()),
                                "i16" => Some("LocalFieldType::Vec3I16".to_string()),
                                "i32" => Some("LocalFieldType::Vec3I32".to_string()),
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
                            (inner.path.is_ident("u8") && is_last)
                                .then(|| "LocalFieldType::RemainingBytes".to_string())
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
                    Some(format!("LocalFieldType::Optional(Box::new({inner}))"))
                }
                _ => None,
            }
        }
        _ => None,
    }
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
        return out;
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
            direction: packet.direction,
            phase: packet.phase.clone(),
            name: packet.name.clone(),
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
    out.push_str(
        "#[allow(clippy::too_many_lines)]\nfn generated_java_packet_schema_registry() -> BTreeMap<String, LocalPacketSchema> {\n",
    );
    out.push_str("    let mut registry = BTreeMap::new();\n");
    for schema in &schemas {
        let direction = if schema.direction == "serverbound" {
            "PacketDirection::Serverbound"
        } else {
            "PacketDirection::Clientbound"
        };

        out.push_str("    registry.insert(\n");
        let _ = writeln!(
            out,
            "        schema_key({}, {:?}, {:?}),\n",
            direction, schema.phase, schema.name
        );
        out.push_str("        LocalPacketSchema::new()");
        for field in &schema.fields {
            out.push_str(".field(");
            let _ = write!(out, "{:?}", field.name);
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

fn git_output(manifest_dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(manifest_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let stdout = stdout.trim();
    (!stdout.is_empty()).then(|| stdout.to_string())
}

fn write_build_info(manifest_dir: &Path, out_dir: &Path) {
    let git_hash = git_output(manifest_dir, &["rev-parse", "--short=10", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());
    let git_hash_full =
        git_output(manifest_dir, &["rev-parse", "HEAD"]).unwrap_or_else(|| git_hash.clone());

    let build_info = format!(
        "pub const GIT_HASH: &str = {git_hash:?};\npub const GIT_HASH_FULL: &str = {git_hash_full:?};\n"
    );

    fs::write(out_dir.join("build_info.rs"), build_info).expect("write build_info.rs");

    let git_head = manifest_dir.join("../.git/HEAD");
    println!("cargo:rerun-if-changed={}", git_head.display());
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    write_build_info(&manifest_dir, &out_dir);

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

    let generated = generate_packet_schemas(&manifest_dir, &packet_by_const);
    fs::write(out_dir.join("generated_java_packet_schemas.rs"), generated)
        .expect("write generated_java_packet_schemas.rs");
}

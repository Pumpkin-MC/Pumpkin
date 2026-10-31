use heck::ToKebabCase;
use quote::ToTokens;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use syn::{Fields, GenericArgument, PathArguments, Type, TypePath};
use wit_encoder::{
    Enum, EnumCase, Field, Interface, Record, Type as WitType, TypeDef, TypeDefKind, Variant,
    VariantCase,
};

pub fn map_type(ty: &Type, defined_types: &HashSet<String>) -> (WitType, bool) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            let ident_str = last_segment.ident.to_string();
            match ident_str.as_str() {
                "String" | "TextComponent" | "str" | "Identifier" | "ResourceLocation"
                | "LpVector3d" => (WitType::String, true),
                "Uuid" => (WitType::Named("uuid".into()), true),
                "i32" | "VarInt" | "u32" | "VarUInt" | "usize" | "u24" => (WitType::S32, true),
                "i64" | "u64" | "VarLong" | "VarULong" => (WitType::S64, true),
                "bool" => (WitType::Bool, true),
                "f32" => (WitType::F32, true),
                "f64" => (WitType::F64, true),
                "u8" => (WitType::U8, true),
                "i8" => (WitType::S8, true),
                "u16" | "i16" => (WitType::S32, true),
                "Bytes" | "BoxedU8Slice" | "ChunkData" => (WitType::list(WitType::U8), true),
                "BitSet" | "Bitset" => (WitType::list(WitType::S64), true),
                "NbtCompound" | "Nbt" => (WitType::Named("nbt-tree".into()), true),
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        let (inner, ok) = map_type(inner_ty, defined_types);
                        return (WitType::option(inner), ok);
                    }
                    (WitType::Named("inexpressible-host-type".into()), false)
                }
                "Box" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        if let Type::Slice(ts) = inner_ty {
                            if let Type::Path(tp) = &*ts.elem
                                && tp.path.segments.last().unwrap().ident == "u8"
                            {
                                return (WitType::list(WitType::U8), true);
                            }
                            let (inner, ok) = map_type(&ts.elem, defined_types);
                            return (WitType::list(inner), ok);
                        }
                        if let Type::Path(tp) = inner_ty
                            && tp.path.segments.last().unwrap().ident == "str"
                        {
                            return (WitType::String, true);
                        }
                        return map_type(inner_ty, defined_types);
                    }
                    (WitType::Named("inexpressible-host-type".into()), false)
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        if let Type::Path(tp) = inner_ty
                            && tp.path.segments.last().unwrap().ident == "u8"
                        {
                            return (WitType::list(WitType::U8), true);
                        }
                        let (inner, ok) = map_type(inner_ty, defined_types);
                        return (WitType::list(inner), ok);
                    }
                    (WitType::Named("inexpressible-host-type".into()), false)
                }
                "Cow" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        for arg in &args.args {
                            if let GenericArgument::Type(inner_ty) = arg {
                                if let Type::Path(tp) = inner_ty
                                    && tp.path.segments.last().unwrap().ident == "str"
                                {
                                    return (WitType::String, true);
                                }
                                return map_type(inner_ty, defined_types);
                            }
                        }
                    }
                    (WitType::Named("inexpressible-host-type".into()), false)
                }
                "NonZero" | "EnumAsStr" | "IdOr" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return map_type(inner_ty, defined_types);
                    }
                    (WitType::Named("inexpressible-host-type".into()), false)
                }
                "HashMap" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        let type_args: Vec<_> = args
                            .args
                            .iter()
                            .filter_map(|a| match a {
                                GenericArgument::Type(t) => Some(t),
                                _ => None,
                            })
                            .collect();
                        if type_args.len() == 2 {
                            let (t0, ok0) = map_type(type_args[0], defined_types);
                            let (t1, ok1) = map_type(type_args[1], defined_types);
                            return (WitType::list(WitType::tuple(vec![t0, t1])), ok0 && ok1);
                        }
                    }
                    (WitType::Named("inexpressible-host-type".into()), false)
                }
                "Vector3" | "BlockPos" => {
                    if ident_str == "Vector3" {
                        (
                            WitType::tuple(vec![WitType::F64, WitType::F64, WitType::F64]),
                            true,
                        )
                    } else {
                        (
                            WitType::tuple(vec![WitType::S32, WitType::S32, WitType::S32]),
                            true,
                        )
                    }
                }
                "Vector2" => (WitType::tuple(vec![WitType::F64, WitType::F64]), true),
                _ => {
                    let kebab = ident_str.to_kebab_case();
                    if defined_types.contains(&kebab) {
                        (WitType::named(kebab), true)
                    } else {
                        (WitType::Named("inexpressible-host-type".into()), false)
                    }
                }
            }
        }
        Type::Reference(tr) => map_type(&tr.elem, defined_types),
        Type::Slice(ts) => {
            if let Type::Path(tp) = &*ts.elem
                && tp.path.segments.last().unwrap().ident == "u8"
            {
                (WitType::list(WitType::U8), true)
            } else {
                let (inner, ok) = map_type(&ts.elem, defined_types);
                (WitType::list(inner), ok)
            }
        }
        Type::Array(ta) => {
            if let Type::Path(tp) = &*ta.elem
                && tp.path.segments.last().unwrap().ident == "u8"
            {
                (WitType::list(WitType::U8), true)
            } else {
                let (inner, ok) = map_type(&ta.elem, defined_types);
                (WitType::list(inner), ok)
            }
        }
        Type::Tuple(tt) => {
            if tt.elems.is_empty() {
                (WitType::String, true)
            } else {
                let mut elems = Vec::with_capacity(tt.elems.len());
                let mut all_ok = true;
                for e in &tt.elems {
                    let (mapped, ok) = map_type(e, defined_types);
                    elems.push(mapped);
                    all_ok &= ok;
                }
                (WitType::tuple(elems), all_ok)
            }
        }
        Type::Paren(tp) => map_type(&tp.elem, defined_types),
        Type::Group(tg) => map_type(&tg.elem, defined_types),
        _ => (WitType::Named("inexpressible-host-type".into()), false),
    }
}

pub fn collect_fields(
    named_fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    defined_types: &HashSet<String>,
) -> Vec<Field> {
    named_fields
        .into_iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_string().to_kebab_case();
            let (field_type, is_ok) = map_type(&field.ty, defined_types);

            let mut docs = extract_doc_comments(&field.attrs);
            if !is_ok {
                docs = Some(format!(
                    "{}WARNING: Could not fully translate type `{}` to WIT",
                    docs.map_or_default(|str| str + &"\n\n"),
                    field.ty.to_token_stream()
                ));
            }

            let mut wit_field = Field::new(field_name, field_type);
            wit_field.set_docs(docs);
            wit_field
        })
        .collect()
}

pub fn translate_enum_to_wit(
    e: syn::ItemEnum,
    enum_wit_name: String,
    defined_types: &HashSet<String>,
) -> (TypeDef, Vec<TypeDef>) {
    let case_names = e
        .variants
        .iter()
        .map(|v| v.ident.to_string().to_kebab_case())
        .collect::<Vec<_>>();

    if e.variants.iter().all(|v| matches!(v.fields, Fields::Unit)) {
        let mut main_def = TypeDef::new(
            enum_wit_name,
            TypeDefKind::Enum(Enum::from_iter(
                e.variants
                    .iter()
                    .zip(case_names.into_iter())
                    .map(|(v, name)| {
                        let mut case = EnumCase::new(name);
                        case.set_docs(extract_doc_comments(&v.attrs));
                        case
                    }),
            )),
        );

        main_def.set_docs(extract_doc_comments(&e.attrs));

        (main_def, Vec::new())
    } else {
        let helper_types = e
            .variants
            .iter()
            .zip(case_names.iter())
            .filter_map(|(v, name)| match &v.fields {
                Fields::Named(fields) => Some((fields, name)),
                _ => None,
            })
            .map(|(fields, name)| {
                let fields_list = collect_fields(&fields.named, defined_types);
                let mut def = TypeDef::new(
                    format!("{enum_wit_name}-{name}"),
                    TypeDefKind::Record(Record::new(fields_list)),
                );
                def.set_docs(Some(format!(
                    "Generated; extracted from variant arm. See `{enum_wit_name}.{name}` for details."
                )));
                def
            })
            .collect::<Vec<_>>();

        let case_fields = e
            .variants
            .iter()
            .zip(case_names.into_iter())
            .map(|(v, variant_wit_name)| {
                let (mut case, warning) = match &v.fields {
                    Fields::Named(fields) => {
                        let sub_record_name = format!("{}-{}", enum_wit_name, variant_wit_name);
                        let fields_list = collect_fields(&fields.named, defined_types);
                        (
                            VariantCase::value(variant_wit_name, WitType::named(sub_record_name)),
                            None,
                        )
                    }
                    Fields::Unnamed(fields) => match fields.unnamed.len() {
                        0 => (VariantCase::empty(variant_wit_name), None),
                        1 => {
                            let (ty, is_ok) = map_type(&fields.unnamed[0].ty, defined_types);
                            let warning = (!is_ok).then(|| {
                                format!(
                                    "WARNING: Could not fully translate type `{}` to WIT",
                                    fields.unnamed[0].ty.to_token_stream()
                                )
                            });
                            let case = VariantCase::value(variant_wit_name, ty);
                            (case, warning)
                        }
                        _ => {
                            let types: Vec<_> = fields
                                .unnamed
                                .iter()
                                .map(|f| map_type(&f.ty, defined_types))
                                .collect();
                            let warning = types.iter().any(|(_, is_ok)| !is_ok).then(|| {
                                format!(
                                    "WARNING: Could not fully translate some types to WIT: {}",
                                    fields
                                        .unnamed
                                        .iter()
                                        .zip(types.iter())
                                        .filter(|(_, (_, is_ok))| !is_ok)
                                        .map(|(rust, _)| rust.to_token_stream().to_string())
                                        .fold(String::new(), |accum, str| accum + "\n - " + &str)
                                )
                            });
                            let case = VariantCase::value(
                                variant_wit_name,
                                WitType::tuple(types.into_iter().map(|(t, _)| t)),
                            );
                            (case, warning)
                        }
                    },
                    Fields::Unit => (VariantCase::empty(variant_wit_name), None),
                };

                case.set_docs(
                    extract_doc_comments(&v.attrs)
                        .map(|str| {
                            if let Some(warning) = &warning {
                                str + &warning
                            } else {
                                str
                            }
                        })
                        .or(warning),
                );
                case
            })
            .collect::<Vec<_>>();

        let mut main_def = TypeDef::new(
            enum_wit_name.clone(),
            TypeDefKind::Variant(Variant::from(case_fields)),
        );
        main_def.set_docs(extract_doc_comments(&e.attrs));
        (main_def, helper_types)
    }
}

pub fn is_valid_helper_struct(s: &syn::ItemStruct) -> bool {
    matches!(s.vis, syn::Visibility::Public(_))
        && matches!(s.fields, Fields::Named(_))
        && s.generics.type_params().next().is_none()
        && s.generics.const_params().next().is_none()
}

pub fn is_valid_helper_enum(e: &syn::ItemEnum) -> bool {
    matches!(e.vis, syn::Visibility::Public(_))
        && e.generics.type_params().next().is_none()
        && e.generics.const_params().next().is_none()
}

pub fn process_helper_struct(
    s: syn::ItemStruct,
    interface: &mut Interface,
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

    if !fields_list.is_empty() {
        let mut def = TypeDef::new(wit_name, TypeDefKind::Record(Record::new(fields_list)));
        def.set_docs(extract_doc_comments(&s.attrs));
        interface.type_def(def);
    }
}

pub fn process_helper_enum(
    e: syn::ItemEnum,
    interface: &mut Interface,
    processed_types: &mut HashSet<String>,
    defined_types: &HashSet<String>,
) {
    let enum_wit_name = e.ident.to_string().to_kebab_case();
    if !processed_types.insert(enum_wit_name.clone()) {
        panic!("Encountered duplicate type: {}", enum_wit_name);
    }

    let (main_def, helper_defs) = translate_enum_to_wit(e, enum_wit_name, defined_types);
    for def in helper_defs {
        if !processed_types.insert(def.name().to_string()) {
            panic!("Encountered duplicate type: {}", def.name());
        }
        interface.type_def(def);
    }
    interface.type_def(main_def);
}

pub struct RustFilesFinder {
    stack: Vec<PathBuf>,
}

impl RustFilesFinder {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            stack: vec![root.as_ref().to_path_buf()],
        }
    }
}

impl Iterator for RustFilesFinder {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    let mut paths = entries.flatten().map(|e| e.path()).collect::<Vec<_>>();
                    paths.sort();
                    self.stack.extend(paths.into_iter().rev());
                }
            } else if path.is_file()
                && path.extension().is_some_and(|ext| ext == "rs")
                && path.file_name().is_some_and(|name| name != "mod.rs")
            {
                return Some(path);
            }
        }
        None
    }
}

/// Extracts doc comment lines from a slice of syn attributes, if any.
pub fn extract_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    let val = lit_str.value();
                    let line = val.strip_prefix(' ').unwrap_or(&val);
                    lines.push(line.to_string());
                }
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

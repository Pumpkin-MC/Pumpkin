use heck::ToKebabCase;
use std::collections::HashSet;
use syn::{GenericArgument, PathArguments, Type, TypePath};
use wit_encoder::Type as WitType;

pub fn map_type(ty: &Type) -> WitType {
    map_type_with_defined(ty, None)
}

pub fn map_type_with_defined(ty: &Type, defined_types: Option<&HashSet<String>>) -> WitType {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            let ident_str = last_segment.ident.to_string();
            match ident_str.as_str() {
                "String" | "TextComponent" | "str" | "Identifier" | "ResourceLocation"
                | "LpVector3d" => WitType::String,
                "Uuid" => WitType::Named("uuid".into()),
                "i32" | "VarInt" | "u32" | "VarUInt" | "usize" | "u24" => WitType::S32,
                "i64" | "u64" | "VarLong" | "VarULong" => WitType::S64,
                "bool" => WitType::Bool,
                "f32" => WitType::F32,
                "f64" => WitType::F64,
                "u8" | "i8" => WitType::U8,
                "u16" | "i16" => WitType::S32,
                "Bytes" | "BoxedU8Slice" | "ChunkData" => WitType::list(WitType::U8),
                "BitSet" | "Bitset" => WitType::list(WitType::S64),
                "NbtCompound" | "Nbt" | "DynamicRecipe" => WitType::String,
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return WitType::option(map_type_with_defined(inner_ty, defined_types));
                    }
                    WitType::String
                }
                "Box" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        if let Type::Slice(ts) = inner_ty {
                            if let Type::Path(tp) = &*ts.elem
                                && tp.path.segments.last().unwrap().ident == "u8"
                            {
                                return WitType::list(WitType::U8);
                            }
                            return WitType::list(map_type_with_defined(&ts.elem, defined_types));
                        }
                        if let Type::Path(tp) = inner_ty
                            && tp.path.segments.last().unwrap().ident == "str"
                        {
                            return WitType::String;
                        }
                        return map_type_with_defined(inner_ty, defined_types);
                    }
                    WitType::String
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        if let Type::Path(tp) = inner_ty
                            && tp.path.segments.last().unwrap().ident == "u8"
                        {
                            return WitType::list(WitType::U8);
                        }
                        return WitType::list(map_type_with_defined(inner_ty, defined_types));
                    }
                    WitType::String
                }
                "Cow" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        for arg in &args.args {
                            if let GenericArgument::Type(inner_ty) = arg {
                                if let Type::Path(tp) = inner_ty
                                    && tp.path.segments.last().unwrap().ident == "str"
                                {
                                    return WitType::String;
                                }
                                return map_type_with_defined(inner_ty, defined_types);
                            }
                        }
                    }
                    WitType::String
                }
                "NonZero" | "EnumAsStr" | "IdOr" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return map_type_with_defined(inner_ty, defined_types);
                    }
                    WitType::String
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
                            return WitType::list(WitType::tuple(vec![
                                map_type_with_defined(type_args[0], defined_types),
                                map_type_with_defined(type_args[1], defined_types),
                            ]));
                        }
                    }
                    WitType::String
                }
                "Vector3" | "BlockPos" => {
                    if ident_str == "Vector3" {
                        WitType::tuple(vec![WitType::F64, WitType::F64, WitType::F64])
                    } else {
                        WitType::tuple(vec![WitType::S32, WitType::S32, WitType::S32])
                    }
                }
                "Vector2" => WitType::tuple(vec![WitType::F64, WitType::F64]),
                _ => {
                    let kebab = ident_str.to_kebab_case();
                    if let Some(defined) = defined_types {
                        if defined.contains(&kebab) {
                            WitType::named(kebab)
                        } else {
                            WitType::String
                        }
                    } else {
                        WitType::named(kebab)
                    }
                }
            }
        }
        Type::Reference(tr) => map_type_with_defined(&tr.elem, defined_types),
        Type::Slice(ts) => {
            if let Type::Path(tp) = &*ts.elem
                && tp.path.segments.last().unwrap().ident == "u8"
            {
                WitType::list(WitType::U8)
            } else {
                WitType::list(map_type_with_defined(&ts.elem, defined_types))
            }
        }
        Type::Array(ta) => {
            if let Type::Path(tp) = &*ta.elem
                && tp.path.segments.last().unwrap().ident == "u8"
            {
                WitType::list(WitType::U8)
            } else {
                WitType::list(map_type_with_defined(&ta.elem, defined_types))
            }
        }
        Type::Tuple(tt) => {
            if tt.elems.is_empty() {
                WitType::String
            } else {
                WitType::tuple(
                    tt.elems
                        .iter()
                        .map(|e| map_type_with_defined(e, defined_types))
                        .collect::<Vec<_>>(),
                )
            }
        }
        Type::Paren(tp) => map_type_with_defined(&tp.elem, defined_types),
        Type::Group(tg) => map_type_with_defined(&tg.elem, defined_types),
        _ => WitType::String,
    }
}

use syn::{Type, TypePath};
use wit_encoder::Type as WitType;

pub fn map_type(ty: &Type) -> WitType {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            let ident_str = last_segment.ident.to_string();
            match ident_str.as_str() {
                "String" | "TextComponent" => WitType::String,
                "Uuid" => WitType::Named("uuid".into()),
                "i32" | "VarInt" | "u32" | "VarUInt" | "usize" => WitType::S32,
                "i64" | "u64" => WitType::S64,
                "bool" => WitType::Bool,
                "f32" => WitType::F32,
                "f64" => WitType::F64,
                "u8" | "i8" => WitType::U8,
                "u16" | "i16" => WitType::S32,
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return WitType::option(map_type(inner_ty));
                    }
                    WitType::String
                }
                "Box" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return map_type(inner_ty);
                        }
                    WitType::String
                }
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        // Check if the inner type is u8
                        if let Type::Path(tp) = inner_ty
                            && tp.path.segments.last().unwrap().ident == "u8"
                        {
                            return WitType::list(WitType::U8);
                        }
                        return WitType::list(map_type(inner_ty));
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
                _ => WitType::String,
            }
        }
        Type::Reference(tr) => map_type(&tr.elem),
        Type::Slice(ts) => {
            if let Type::Path(tp) = &*ts.elem
                && tp.path.segments.last().unwrap().ident == "u8"
            {
                WitType::list(WitType::U8)
            } else {
                WitType::list(map_type(&ts.elem))
            }
        }
        _ => WitType::String,
    }
}

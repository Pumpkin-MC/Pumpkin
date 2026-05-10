use syn::{Type, TypePath};
use wit_encoder::Type as WitType;

pub fn map_type(ty: &Type) -> WitType {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            let ident_str = last_segment.ident.to_string();
            match ident_str.as_str() {
                "String" | "Uuid" | "TextComponent" => WitType::String,
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
                "Vec" | "Box" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        return WitType::list(map_type(inner_ty));
                    }
                    WitType::String
                }
                "Vector3" | "BlockPos" => {
                    // Map to tuple or common record
                    // For now, let's use a tuple of floats for Vector3 and ints for BlockPos
                    if ident_str == "Vector3" {
                        WitType::tuple(vec![WitType::F64, WitType::F64, WitType::F64])
                    } else {
                        WitType::tuple(vec![WitType::S32, WitType::S32, WitType::S32])
                    }
                }
                _ => WitType::String, // Fallback
            }
        }
        _ => WitType::String, // Fallback
    }
}

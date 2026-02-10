use quote::{ToTokens, format_ident, quote};
use syn::Ident;

/// Represents a specific version of the Minecraft Java Edition protocol.
/// from pumpkin_util::version::MinecraftVersion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum MinecraftVersion {
    /// 1.7.2: The Update That Changed The World
    V_1_7_2,
    V_1_7_6,
    /// 1.8: The Bountiful Update
    V_1_8,
    /// 1.9: The Combat Update
    V_1_9,
    V_1_9_1,
    V_1_9_2,
    V_1_9_3,
    /// 1.10: The Frostburn Update
    V_1_10,
    /// 1.11: The Exploration Update
    V_1_11,
    V_1_11_1,
    /// 1.12: The World of Color Update
    V_1_12,
    V_1_12_1,
    V_1_12_2,
    /// 1.13: Update Aquatic
    V_1_13,
    V_1_13_1,
    V_1_13_2,
    /// 1.14: Village & Pillage
    V_1_14,
    V_1_14_1,
    V_1_14_2,
    V_1_14_3,
    V_1_14_4,
    /// 1.15: Buzzy Bees
    V_1_15,
    V_1_15_1,
    V_1_15_2,
    /// 1.16: Nether Update
    V_1_16,
    V_1_16_1,
    V_1_16_2,
    V_1_16_3,
    V_1_16_4,
    /// 1.17: Caves & Cliffs: Part I
    V_1_17,
    V_1_17_1,
    /// 1.18: Caves & Cliffs: Part II
    V_1_18,
    V_1_18_2,
    /// 1.19: The Wild Update
    V_1_19,
    V_1_19_1,
    V_1_19_3,
    V_1_19_4,
    /// 1.20: Trails & Tales
    V_1_20,
    V_1_20_2,
    V_1_20_3,
    /// 1.20.5: Armored Paws
    V_1_20_5,
    /// 1.21: Tricky Trials
    V_1_21,
    V_1_21_2,
    V_1_21_4,
    /// 1.21.5: Bundles of Bravery
    V_1_21_5,
    V_1_21_6,
    V_1_21_7,
    V_1_21_9,
    V_1_21_11,
}

impl MinecraftVersion {
    pub fn to_field_ident(self) -> Ident {
        match self {
            Self::V_1_7_2 => format_ident!("v1_7_2"),
            Self::V_1_7_6 => format_ident!("v1_7_6"),
            Self::V_1_8 => format_ident!("v1_8"),
            Self::V_1_9 => format_ident!("v1_9"),
            Self::V_1_9_1 => format_ident!("v1_9_1"),
            Self::V_1_9_2 => format_ident!("v1_9_2"),
            Self::V_1_9_3 => format_ident!("v1_9_3"),
            Self::V_1_10 => format_ident!("v1_10"),
            Self::V_1_11 => format_ident!("v1_11"),
            Self::V_1_11_1 => format_ident!("v1_11_1"),
            Self::V_1_12 => format_ident!("v1_12"),
            Self::V_1_12_1 => format_ident!("v1_12_1"),
            Self::V_1_12_2 => format_ident!("v1_12_2"),
            Self::V_1_13 => format_ident!("v1_13"),
            Self::V_1_13_1 => format_ident!("v1_13_1"),
            Self::V_1_13_2 => format_ident!("v1_13_2"),
            Self::V_1_14 => format_ident!("v1_14"),
            Self::V_1_14_1 => format_ident!("v1_14_1"),
            Self::V_1_14_2 => format_ident!("v1_14_2"),
            Self::V_1_14_3 => format_ident!("v1_14_3"),
            Self::V_1_14_4 => format_ident!("v1_14_4"),
            Self::V_1_15 => format_ident!("v1_15"),
            Self::V_1_15_1 => format_ident!("v1_15_1"),
            Self::V_1_15_2 => format_ident!("v1_15_2"),
            Self::V_1_16 => format_ident!("v1_16"),
            Self::V_1_16_1 => format_ident!("v1_16_1"),
            Self::V_1_16_2 => format_ident!("v1_16_2"),
            Self::V_1_16_3 => format_ident!("v1_16_3"),
            Self::V_1_16_4 => format_ident!("v1_16_4"),
            Self::V_1_17 => format_ident!("v1_17"),
            Self::V_1_17_1 => format_ident!("v1_17_1"),
            Self::V_1_18 => format_ident!("v1_18"),
            Self::V_1_18_2 => format_ident!("v1_18_2"),
            Self::V_1_19 => format_ident!("v1_19"),
            Self::V_1_19_1 => format_ident!("v1_19_1"),
            Self::V_1_19_3 => format_ident!("v1_19_3"),
            Self::V_1_19_4 => format_ident!("v1_19_4"),
            Self::V_1_20 => format_ident!("v1_20"),
            Self::V_1_20_2 => format_ident!("v1_20_2"),
            Self::V_1_20_3 => format_ident!("v1_20_3"),
            Self::V_1_20_5 => format_ident!("v1_20_5"),
            Self::V_1_21 => format_ident!("v1_21"),
            Self::V_1_21_2 => format_ident!("v1_21_2"),
            Self::V_1_21_4 => format_ident!("v1_21_4"),
            Self::V_1_21_5 => format_ident!("v1_21_5"),
            Self::V_1_21_6 => format_ident!("v1_21_6"),
            Self::V_1_21_7 => format_ident!("v1_21_7"),
            Self::V_1_21_9 => format_ident!("v1_21_9"),
            Self::V_1_21_11 => format_ident!("v1_21_11"),
        }
    }
}

impl ToTokens for MinecraftVersion {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Self::V_1_7_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_7_2 },
            Self::V_1_7_6 => quote! { pumpkin_util::version::MinecraftVersion::V_1_7_6 },
            Self::V_1_8 => quote! { pumpkin_util::version::MinecraftVersion::V_1_8 },
            Self::V_1_9 => quote! { pumpkin_util::version::MinecraftVersion::V_1_9 },
            Self::V_1_9_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_9_1 },
            Self::V_1_9_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_9_2 },
            Self::V_1_9_3 => quote! { pumpkin_util::version::MinecraftVersion::V_1_9_3 },
            Self::V_1_10 => quote! { pumpkin_util::version::MinecraftVersion::V_1_10 },
            Self::V_1_11 => quote! { pumpkin_util::version::MinecraftVersion::V_1_11 },
            Self::V_1_11_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_11_1 },
            Self::V_1_12 => quote! { pumpkin_util::version::MinecraftVersion::V_1_12 },
            Self::V_1_12_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_12_1 },
            Self::V_1_12_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_12_2 },
            Self::V_1_13 => quote! { pumpkin_util::version::MinecraftVersion::V_1_13 },
            Self::V_1_13_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_13_1 },
            Self::V_1_13_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_13_2 },
            Self::V_1_14 => quote! { pumpkin_util::version::MinecraftVersion::V_1_14 },
            Self::V_1_14_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_14_1 },
            Self::V_1_14_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_14_2 },
            Self::V_1_14_3 => quote! { pumpkin_util::version::MinecraftVersion::V_1_14_3 },
            Self::V_1_14_4 => quote! { pumpkin_util::version::MinecraftVersion::V_1_14_4 },
            Self::V_1_15 => quote! { pumpkin_util::version::MinecraftVersion::V_1_15 },
            Self::V_1_15_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_15_1 },
            Self::V_1_15_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_15_2 },
            Self::V_1_16 => quote! { pumpkin_util::version::MinecraftVersion::V_1_16 },
            Self::V_1_16_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_16_1 },
            Self::V_1_16_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_16_2 },
            Self::V_1_16_3 => quote! { pumpkin_util::version::MinecraftVersion::V_1_16_3 },
            Self::V_1_16_4 => quote! { pumpkin_util::version::MinecraftVersion::V_1_16_4 },
            Self::V_1_17 => quote! { pumpkin_util::version::MinecraftVersion::V_1_17 },
            Self::V_1_17_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_17_1 },
            Self::V_1_18 => quote! { pumpkin_util::version::MinecraftVersion::V_1_18 },
            Self::V_1_18_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_18_2 },
            Self::V_1_19 => quote! { pumpkin_util::version::MinecraftVersion::V_1_19 },
            Self::V_1_19_1 => quote! { pumpkin_util::version::MinecraftVersion::V_1_19_1 },
            Self::V_1_19_3 => quote! { pumpkin_util::version::MinecraftVersion::V_1_19_3 },
            Self::V_1_19_4 => quote! { pumpkin_util::version::MinecraftVersion::V_1_19_4 },
            Self::V_1_20 => quote! { pumpkin_util::version::MinecraftVersion::V_1_20 },
            Self::V_1_20_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_20_2 },
            Self::V_1_20_3 => quote! { pumpkin_util::version::MinecraftVersion::V_1_20_3 },
            Self::V_1_20_5 => quote! { pumpkin_util::version::MinecraftVersion::V_1_20_5 },
            Self::V_1_21 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21 },
            Self::V_1_21_2 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_2 },
            Self::V_1_21_4 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_4 },
            Self::V_1_21_5 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_5 },
            Self::V_1_21_6 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_6 },
            Self::V_1_21_7 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_7 },
            Self::V_1_21_9 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_9 },
            Self::V_1_21_11 => quote! { pumpkin_util::version::MinecraftVersion::V_1_21_11 },
        });
    }
}

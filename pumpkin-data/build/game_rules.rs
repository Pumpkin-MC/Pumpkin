use std::{collections::HashMap, fs};

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/game_rules.json");

    let game_rules: HashMap<String, Value> =
        serde_json::from_str(&fs::read_to_string("../assets/game_rules.json").unwrap())
            .expect("Failed to parse game_rules.json");

    let mut enum_variants = TokenStream::new();
    let mut enum_variants_list = TokenStream::new();
    let mut enum_to_string = TokenStream::new();
    let mut struct_fields = TokenStream::new();
    let mut default_values = TokenStream::new();
    let mut getter_match = TokenStream::new();
    let mut mut_getter_match = TokenStream::new();
    let mut default_functions = TokenStream::new();

    for (raw_name, raw_value) in &game_rules {
        let (variant_type, field_type, default_value) = match raw_value {
            Value::Bool(b) => (quote! { Bool }, quote! { bool }, quote! { #b }),
            Value::Number(n) if n.is_i64() => {
                let i = n.as_i64().unwrap();
                (quote! { Int }, quote! { i64 }, quote! { #i })
            }
            _ => panic!("Unsupported value type for key '{raw_name}'"),
        };

        let snake_case = format_ident!("{}", raw_name.to_snake_case());
        let pascal_case = format_ident!("{}", raw_name.to_pascal_case());
        let default_fn_name = format!("default_{snake_case}");
        let default_fn_ident = format_ident!("default_{snake_case}");

        // Struct field
        struct_fields.extend(quote! {
            #[serde(rename = #raw_name)]
            #[serde(default = #default_fn_name)]
            #[serde(with = "as_string")]
            pub #snake_case: #field_type,
        });

        // Enum variant
        enum_variants.extend(quote! {
            #pascal_case,
        });

        // Enum::all()
        enum_variants_list.extend(quote! {
            Self::#pascal_case,
        });

        // Enum -> &str
        enum_to_string.extend(quote! {
            Self::#pascal_case => write!(f, #raw_name),
        });

        // Default value
        default_values.extend(quote! {
            #snake_case: #default_value,
        });

        // Getter match arms
        getter_match.extend(quote! {
            GameRule::#pascal_case => GameRuleValue::#variant_type(&self.#snake_case),
        });

        mut_getter_match.extend(quote! {
            GameRule::#pascal_case => GameRuleValue::#variant_type(&mut self.#snake_case),
        });

        // Default fn
        default_functions.extend(quote! {
            fn #default_fn_ident() -> #field_type {
                GameRuleRegistry::default().#snake_case
            }
        });
    }

    quote! {
        use std::fmt;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum GameRule {
            #enum_variants
        }

        impl GameRule {
            pub const fn all() -> &'static [Self] {
                &[
                    #enum_variants_list
                ]
            }
        }

        impl fmt::Display for GameRule {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    #enum_to_string
                }
            }
        }

        #[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
        pub struct GameRuleRegistry {
            #struct_fields
        }

        pub enum GameRuleValue<I, B> {
            Int(I),
            Bool(B),
        }

        impl<I: fmt::Display, B: fmt::Display> fmt::Display for GameRuleValue<I, B> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Self::Int(v) => write!(f, "{v}"),
                    Self::Bool(v) => write!(f, "{v}"),
                }
            }
        }

        impl GameRuleRegistry {
            pub fn get(&self, rule: &GameRule) -> GameRuleValue<&i64, &bool> {
                match rule {
                    #getter_match
                }
            }

            pub fn get_mut(&mut self, rule: &GameRule) -> GameRuleValue<&mut i64, &mut bool> {
                match rule {
                    #mut_getter_match
                }
            }
        }

        impl Default for GameRuleRegistry {
            fn default() -> Self {
                Self {
                    #default_values
                }
            }
        }

        #default_functions

        mod as_string {
            use serde::{Serialize, Deserialize, Serializer, Deserializer};
            use std::{fmt::Display, str::FromStr};

            pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
            where
                T: Display,
                S: Serializer,
            {
                serializer.serialize_str(&value.to_string())
            }

            pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
            where
                T: FromStr,
                D: Deserializer<'de>,
                <T as FromStr>::Err: Display,
            {
                let s = String::deserialize(deserializer)?;
                s.parse::<T>().map_err(serde::de::Error::custom)
            }
        }
    }
}

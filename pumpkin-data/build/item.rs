use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use std::collections::HashMap;
use syn::{Ident, LitInt, LitStr};

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub components: ItemComponents,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemComponents {
    #[serde(rename = "minecraft:item_name")]
    // TODO: TextComponent
    pub item_name: Option<String>,
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
    #[serde(rename = "minecraft:jukebox_playable")]
    pub jukebox_playable: Option<JukeboxPlayable>,
    #[serde(rename = "minecraft:damage")]
    pub damage: Option<u16>,
    #[serde(rename = "minecraft:max_damage")]
    pub max_damage: Option<u16>,
    #[serde(rename = "minecraft:attribute_modifiers")]
    pub attribute_modifiers: Option<AttributeModifiers>,
}

impl ToTokens for ItemComponents {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let max_stack_size = LitInt::new(&self.max_stack_size.to_string(), Span::call_site());
        let jukebox_playable = match &self.jukebox_playable {
            Some(playable) => {
                let song = LitStr::new(&playable.song, Span::call_site());
                quote! { Some(JukeboxPlayable { song: #song }) }
            }
            None => quote! { None },
        };

        let item_name = match &self.item_name {
            Some(d) => {
                let item_name = LitStr::new(d, Span::call_site());
                quote! { Some(#item_name) }
            }
            None => quote! { None },
        };

        let damage = match self.damage {
            Some(d) => {
                let damage_lit = LitInt::new(&d.to_string(), Span::call_site());
                quote! { Some(#damage_lit) }
            }
            None => quote! { None },
        };

        let max_damage = match self.max_damage {
            Some(md) => {
                let max_damage_lit = LitInt::new(&md.to_string(), Span::call_site());
                quote! { Some(#max_damage_lit) }
            }
            None => quote! { None },
        };

        let attribute_modifiers = match &self.attribute_modifiers {
            Some(modifiers) => {
                let modifier_code = modifiers.modifiers.iter().map(|modifier| {
                    let r#type = LitStr::new(&modifier.r#type, Span::call_site());
                    let id = LitStr::new(&modifier.id, Span::call_site());
                    let amount = modifier.amount;
                    let operation =
                        Ident::new(&format!("{:?}", modifier.operation), Span::call_site());
                    let slot = LitStr::new(&modifier.slot, Span::call_site());

                    quote! {
                        Modifier {
                            r#type: #r#type,
                            id: #id,
                            amount: #amount,
                            operation: Operation::#operation,
                            slot: #slot,
                        }
                    }
                });
                quote! { Some(AttributeModifiers { modifiers: &[#(#modifier_code),*] }) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            ItemComponents {
                item_name: #item_name,
                max_stack_size: #max_stack_size,
                jukebox_playable: #jukebox_playable,
                damage: #damage,
                max_damage: #max_damage,
                attribute_modifiers: #attribute_modifiers,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct JukeboxPlayable {
    pub song: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AttributeModifiers {
    pub modifiers: Vec<Modifier>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Modifier {
    pub r#type: String,
    pub id: String,
    pub amount: f64,
    pub operation: Operation,
    // TODO: Make this an enum
    pub slot: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/items.json");

    let items: HashMap<String, Item> =
        serde_json::from_str(include_str!("../../assets/items.json"))
            .expect("Failed to parse items.json");

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();

    let mut constants = TokenStream::new();

    for (name, item) in items {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());

        let components = &item.components;
        let components_tokens = components.to_token_stream();

        let id_lit = LitInt::new(&item.id.to_string(), proc_macro2::Span::call_site());

        constants.extend(quote! {
            pub const #const_ident: Item = Item {
                id: #id_lit,
                components: #components_tokens
            };
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(Self::#const_ident),
        });

        type_from_name.extend(quote! {
            #name => Some(Self::#const_ident),
        });
    }

    quote! {
        use pumpkin_util::text::TextComponent;

        #[derive(Clone, Copy, Debug)]
        pub struct Item {
            pub id: u16,
            pub components: ItemComponents,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ItemComponents {
            pub item_name: Option<&'static str>,
            pub max_stack_size: u8,
            pub jukebox_playable: Option<JukeboxPlayable>,
            pub damage: Option<u16>,
            pub max_damage: Option<u16>,
            pub attribute_modifiers: Option<AttributeModifiers>,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct JukeboxPlayable {
            pub song: &'static str,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct AttributeModifiers {
            pub modifiers: &'static [Modifier],
        }

        #[derive(Clone, Copy, Debug)]
        pub struct Modifier {
            pub r#type: &'static str,
            pub id: &'static str,
            pub amount: f64,
            pub operation: Operation,
            // TODO: Make this an enum
            pub slot: &'static str,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Operation {
            AddValue,
            AddMultipliedBase,
            AddMultipliedTotal,
        }

        impl Item {
            #constants

            pub fn translated_name(&self) -> TextComponent {
                serde_json::from_str(self.components.item_name.unwrap()).expect("Could not parse item name.")
            }

            #[doc = r" Try to parse a Item from a resource location string"]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }
            #[doc = r" Try to parse a Item from a raw id"]
            pub const fn from_id(id: u16) -> Option<Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }

        }
    }
}

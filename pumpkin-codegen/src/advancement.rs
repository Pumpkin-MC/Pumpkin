use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::text::TextComponent;
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Deserializer, Serialize};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextContent::Translate;

#[derive(Deserialize,Default)]
pub struct Advancement {
    pub parent : Option<ResourceLocation>,
    #[serde(default)]
    pub display : Option<AdvancementDisplay>,
    // pub criteria : Vec<AdvancementCriterion>,
    //pub rewards : AdvancementRewards,
    #[serde(default,rename="sends_telemetry_event")]
    pub sends_telemetry: bool,
}

#[derive(Deserialize)]
struct DisplayIcon {
    id: ResourceLocation,
    #[serde(default)]
    count: Option<u8>,
}

fn deserialize_icon_id<'de, D>(deserializer: D) -> Result<ResourceLocation, D::Error> where D: Deserializer<'de>,{
    let icon = DisplayIcon::deserialize(deserializer)?;
    Ok(icon.id)
}

#[derive(Deserialize)]
pub struct AdvancementDisplay {
    pub title: TextComponent,
    pub description: TextComponent,
    #[serde(rename = "icon", deserialize_with = "deserialize_icon_id")]
    pub item_icon: ResourceLocation,
    #[serde(default, rename = "frame")]
    pub frame_type: FrameType,
    #[serde(default)]
    pub flags: i32,
    #[serde(default, rename = "background")]
    pub background_texture: Option<ResourceLocation>,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

impl AdvancementDisplay {
    #[must_use]
    pub fn new(
        title: TextComponent,
        description: TextComponent,
        item_icon: ResourceLocation,
        frame_type: FrameType,
        flags: i32,
        background_texture: Option<ResourceLocation>,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            title,
            description,
            item_icon,
            frame_type,
            flags,
            background_texture,
            x,
            y,
        }
    }
}

fn as_optional_translate(text: &TextComponent) -> TokenStream{
    match text {
        Some(name) => {
            let Translate { translate, with: _ } = name.0.content.as_ref() else { panic!() };
            quote! { TextComponent::translate(#translate,[]) }
        }
        None => quote! { None }
    }
}

impl ToTokens for AdvancementDisplay {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item_icon = &self.item_icon;
        let frame_type = &self.frame_type;
        let flags = self.flags;
        let background_texture = &self.background_texture;
        let x = self.x;
        let y = self.y;
        let title = as_optional_translate(&self.title);
        let description = as_optional_translate(&self.description);

        tokens.extend(quote! {
            AdvancementDisplay {
                title: #title,
                description: #description,
                item_icon: #item_icon,
                frame_type: #frame_type,
                flags: #flags,
                background_texture: #background_texture,
                x: #x,
                y: #y,
            }
        });
    }
}

#[derive(Clone, Copy, Deserialize,Serialize,Default)]
#[repr(i32)]
pub enum FrameType {
    #[default]
    Task = 0,
    Challenge = 1,
    Goal = 2,
}

impl ToTokens for FrameType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            FrameType::Task => quote! { FrameType::Task },
            FrameType::Challenge => quote! { FrameType::Challenge },
            FrameType::Goal => quote! { FrameType::Goal },
        };
        tokens.extend(t);
    }
}


#[derive(Serialize)]
pub struct AdvancementProgress<'a> {
    pub id: ResourceLocation,
    pub progress: &'a [Criteria],
}

impl<'a> AdvancementProgress<'a> {
    #[must_use]
    pub fn new(id: ResourceLocation, progress: &'a [Criteria]) -> Self {
        Self { id, progress }
    }
}

#[derive(Serialize)]
pub struct Criteria {
    pub criterion_id: ResourceLocation,
    pub achieve_date: Option<i64>,
}

impl Criteria {
    #[must_use]
    pub fn new(criterion_id: ResourceLocation, achieve_date: Option<i64>) -> Self {
        Self {
            criterion_id,
            achieve_date,
        }
    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/advancements.json");

    let advancements: BTreeMap<String, Advancement> =
        serde_json::from_str(&fs::read_to_string("../assets/advancements.json").unwrap())
            .expect("Failed to parse advancements.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut minecraft_name_to_type = TokenStream::new();

    for (minecraft_name, advancement) in advancements {
        let raw_name = minecraft_name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());

        let parent = match &advancement.parent {
            Some(parent) => quote! { Some(#parent) },
            None => quote! { None }
        };
        let send_telemetry = advancement.sends_telemetry;
        let display = advancement.display;
        variants.extend([quote! {
            pub const #format_name: Self = Self {
                id: #raw_name,
                parent : #parent,
                send_telemetry : #send_telemetry,
                display_name : #display,
            };
        }]);
        name_to_type.extend(quote! { #raw_name => Some(&Self::#format_name), });
        minecraft_name_to_type.extend(quote! { #minecraft_name => Some(&Self::#format_name), });
    }



    quote! {
        use pumpkin_util::text::TextComponent;
        use crate::item_stack::ItemStack;

        pub struct Advancement {
            pub id : &'static str,
            pub parent : Option<&'static str>,
            pub send_telemetry : bool,
            pub display : Option<TextComponent>
        }

        impl Advancement {
            #variants

            pub fn from_name(name: &str) -> Option<&'static Self> {
                    match name {
                        #name_to_type
                        _ => None
                    }
                }

            pub fn from_minecraft_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
        }

        #[derive(Serialize)]
        pub struct AdvancementDisplay {
            pub title: TextComponent,
            pub description: TextComponent,
            pub icon: ItemStack,
            pub frame_type: FrameType,
            pub flags: i32,
            pub background_texture: Option<ResourceLocation>,
            pub x: f32,
            pub y: f32,
        }
    }
}

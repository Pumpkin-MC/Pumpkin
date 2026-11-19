use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use pumpkin_util::math::float_provider::FloatProvider;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct SulfurCubeArchetypeJson {
    items: String,
    attribute_modifiers: Vec<AttributeModifierJson>,
    #[serde(default)]
    buoyant: bool,
    explosion: Option<ExplosionJson>,
    contact_damage: Option<ContactDamageJson>,
    knockback_modifiers: KnockbackModifiersJson,
    sound_settings: SoundSettingsJson,
}

#[derive(Deserialize)]
struct AttributeModifierJson {
    amount: f64,
    attribute: String,
    id: String,
    operation: String,
}

#[derive(Deserialize)]
struct ExplosionJson {
    power: i32,
    causes_fire: bool,
    fuse: i32,
}

#[derive(Deserialize)]
struct ContactDamageJson {
    amount: FloatProvider,
    damage_type: String,
    attribute_to_source: bool,
}

#[derive(Deserialize)]
struct KnockbackModifiersJson {
    horizontal_power: f32,
    vertical_power: f32,
}

#[derive(Deserialize)]
struct SoundSettingsJson {
    hit_sound: String,
    push_sound: String,
    push_sound_impulse_threshold: f32,
    push_sound_cooldown: f32,
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/sulfur_cube_archetype");

    let mut archetypes: BTreeMap<String, SulfurCubeArchetypeJson> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read sulfur_cube_archetype dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read sulfur_cube_archetype file");
        let json: SulfurCubeArchetypeJson =
            serde_json::from_str(&content).expect("parse sulfur_cube_archetype JSON");

        archetypes.insert(stem, json);
    }

    let mut const_defs = Vec::new();
    let mut all_archetypes = Vec::new();

    for (name, json) in &archetypes {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());
        all_archetypes.push(quote! { &Self::#const_ident });

        let items = json
            .items
            .strip_prefix('#')
            .expect("sulfur cube archetype items must be a tag");
        let items_ident = format_ident!("{}", items.replace([':', '/'], "_").to_uppercase());
        let items = quote! { &crate::tag::Item::#items_ident };

        let modifiers = json.attribute_modifiers.iter().map(|modifier| {
            let attribute = modifier
                .attribute
                .strip_prefix("minecraft:")
                .unwrap_or(&modifier.attribute);
            let attribute_ident = format_ident!("{}", attribute.to_shouty_snake_case());
            let operation_ident = match modifier.operation.as_str() {
                "add_value" => format_ident!("AddValue"),
                "add_multiplied_base" => format_ident!("AddMultipliedBase"),
                "add_multiplied_total" => format_ident!("AddMultipliedTotal"),
                other => panic!("unsupported sulfur cube attribute modifier operation {other}"),
            };
            let amount = modifier.amount;
            let id = &modifier.id;
            quote! {
                SulfurCubeAttributeModifier {
                    attribute: &Attributes::#attribute_ident,
                    id: #id,
                    amount: #amount,
                    operation: Operation::#operation_ident,
                }
            }
        });

        let explosion = match &json.explosion {
            Some(explosion) => {
                let power = explosion.power;
                let causes_fire = explosion.causes_fire;
                let fuse = explosion.fuse;
                quote! {
                    Some(SulfurCubeExplosion {
                        power: #power,
                        causes_fire: #causes_fire,
                        fuse: #fuse,
                    })
                }
            }
            None => quote! { None },
        };

        let contact_damage = match &json.contact_damage {
            Some(contact_damage) => {
                let damage_type = contact_damage
                    .damage_type
                    .strip_prefix("minecraft:")
                    .unwrap_or(&contact_damage.damage_type);
                let damage_type_ident = format_ident!("{}", damage_type.to_shouty_snake_case());
                let amount = &contact_damage.amount;
                let attribute_to_source = contact_damage.attribute_to_source;
                quote! {
                    Some(SulfurCubeContactDamage {
                        damage_type: &DamageType::#damage_type_ident,
                        amount: #amount,
                        attribute_to_source: #attribute_to_source,
                    })
                }
            }
            None => quote! { None },
        };

        let knockback_modifiers = &json.knockback_modifiers;
        let horizontal_power = knockback_modifiers.horizontal_power;
        let vertical_power = knockback_modifiers.vertical_power;

        let sound_settings = &json.sound_settings;
        let hit_sound = sound_settings
            .hit_sound
            .strip_prefix("minecraft:")
            .unwrap_or(&sound_settings.hit_sound)
            .to_pascal_case();
        let hit_sound = format_ident!("{}", hit_sound);
        let push_sound = sound_settings
            .push_sound
            .strip_prefix("minecraft:")
            .unwrap_or(&sound_settings.push_sound)
            .to_pascal_case();
        let push_sound = format_ident!("{}", push_sound);
        let push_sound_impulse_threshold = sound_settings.push_sound_impulse_threshold;
        let push_sound_cooldown = sound_settings.push_sound_cooldown;

        let buoyant = json.buoyant;

        const_defs.push(quote! {
            pub const #const_ident: Self = Self {
                name: #name,
                items: #items,
                attribute_modifiers: &[#(#modifiers),*],
                buoyant: #buoyant,
                explosion: #explosion,
                contact_damage: #contact_damage,
                knockback_modifiers: SulfurCubeKnockbackModifiers {
                    horizontal_power: #horizontal_power,
                    vertical_power: #vertical_power,
                },
                sound_settings: SulfurCubeSoundSettings {
                    hit_sound: Sound::#hit_sound,
                    push_sound: Sound::#push_sound,
                    push_sound_impulse_threshold: #push_sound_impulse_threshold,
                    push_sound_cooldown: #push_sound_cooldown,
                },
            };
        });
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        use crate::attributes::Attributes;
        use crate::damage::DamageType;
        use crate::data_component_impl::Operation;
        use crate::sound::Sound;
        use crate::tag::Tag;
        use pumpkin_util::math::float_provider::FloatProvider;

        pub struct SulfurCubeAttributeModifier {
            pub attribute: &'static Attributes,
            pub id: &'static str,
            pub amount: f64,
            pub operation: Operation,
        }

        pub struct SulfurCubeExplosion {
            pub power: i32,
            pub causes_fire: bool,
            pub fuse: i32,
        }

        pub struct SulfurCubeContactDamage {
            pub damage_type: &'static DamageType,
            pub amount: FloatProvider,
            pub attribute_to_source: bool,
        }

        #[derive(Clone, Copy)]
        pub struct SulfurCubeKnockbackModifiers {
            pub horizontal_power: f32,
            pub vertical_power: f32,
        }

        #[derive(Clone, Copy)]
        pub struct SulfurCubeSoundSettings {
            pub hit_sound: Sound,
            pub push_sound: Sound,
            pub push_sound_impulse_threshold: f32,
            pub push_sound_cooldown: f32,
        }

        pub struct SulfurCubeArchetype {
            pub name: &'static str,
            pub items: &'static Tag,
            pub attribute_modifiers: &'static [SulfurCubeAttributeModifier],
            pub buoyant: bool,
            pub explosion: Option<SulfurCubeExplosion>,
            pub contact_damage: Option<SulfurCubeContactDamage>,
            pub knockback_modifiers: SulfurCubeKnockbackModifiers,
            pub sound_settings: SulfurCubeSoundSettings,
        }

        impl SulfurCubeArchetype {
            #(#const_defs)*

            #[must_use]
            pub const fn all() -> &'static [&'static Self] {
                &[#(#all_archetypes),*]
            }
        }
    }
}

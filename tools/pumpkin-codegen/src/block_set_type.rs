use std::fs;

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct BlockSetTypeEntry {
    name: String,
    can_open_by_hand: bool,
    can_open_by_wind_charge: bool,
    can_button_be_activated_by_arrows: bool,
    pressure_plate_sensitivity: String,
    sound_type: String,
    door_close: String,
    door_open: String,
    trapdoor_close: String,
    trapdoor_open: String,
    pressure_plate_click_off: String,
    pressure_plate_click_on: String,
    button_click_off: String,
    button_click_on: String,
}

fn sound_ident(path: &str) -> proc_macro2::Ident {
    format_ident!("{}", path.to_pascal_case())
}

pub fn build() -> TokenStream {
    let entries: Vec<BlockSetTypeEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/block_set_types.json").unwrap())
            .expect("Failed to parse block_set_types.json");

    let mut constants = TokenStream::new();
    let mut sensitivity_variants = TokenStream::new();

    let mut seen_sensitivities = std::collections::HashSet::new();

    for entry in &entries {
        let name = &entry.name;
        let const_name = format_ident!("{}", name.to_shouty_snake_case());

        let can_open_by_hand = entry.can_open_by_hand;
        let can_open_by_wind_charge = entry.can_open_by_wind_charge;
        let can_button_be_activated_by_arrows = entry.can_button_be_activated_by_arrows;
        let sound_type = &entry.sound_type;

        let sensitivity = &entry.pressure_plate_sensitivity;
        let sensitivity_ident = format_ident!("{}", sensitivity.to_pascal_case());
        if seen_sensitivities.insert(sensitivity.clone()) {
            sensitivity_variants.extend(quote! {
                #sensitivity_ident,
            });
        }

        let door_close = sound_ident(&entry.door_close);
        let door_open = sound_ident(&entry.door_open);
        let trapdoor_close = sound_ident(&entry.trapdoor_close);
        let trapdoor_open = sound_ident(&entry.trapdoor_open);
        let pressure_plate_click_off = sound_ident(&entry.pressure_plate_click_off);
        let pressure_plate_click_on = sound_ident(&entry.pressure_plate_click_on);
        let button_click_off = sound_ident(&entry.button_click_off);
        let button_click_on = sound_ident(&entry.button_click_on);

        constants.extend(quote! {
            pub const #const_name: BlockSetType = BlockSetType {
                name: #name,
                can_open_by_hand: #can_open_by_hand,
                can_open_by_wind_charge: #can_open_by_wind_charge,
                can_button_be_activated_by_arrows: #can_button_be_activated_by_arrows,
                pressure_plate_sensitivity: PressurePlateSensitivity::#sensitivity_ident,
                sound_type: #sound_type,
                door_close: Sound::#door_close,
                door_open: Sound::#door_open,
                trapdoor_close: Sound::#trapdoor_close,
                trapdoor_open: Sound::#trapdoor_open,
                pressure_plate_click_off: Sound::#pressure_plate_click_off,
                pressure_plate_click_on: Sound::#pressure_plate_click_on,
                button_click_off: Sound::#button_click_off,
                button_click_on: Sound::#button_click_on,
            };
        });
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        use crate::sound::Sound;

        /// What kind of entity can press a pressure plate.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum PressurePlateSensitivity {
            #sensitivity_variants
        }

        /// A vanilla block set type, shared by doors, trapdoors, buttons and pressure plates.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct BlockSetType {
            pub name: &'static str,
            pub can_open_by_hand: bool,
            pub can_open_by_wind_charge: bool,
            pub can_button_be_activated_by_arrows: bool,
            pub pressure_plate_sensitivity: PressurePlateSensitivity,
            /// The vanilla `SoundType` constant name.
            pub sound_type: &'static str,
            pub door_close: Sound,
            pub door_open: Sound,
            pub trapdoor_close: Sound,
            pub trapdoor_open: Sound,
            pub pressure_plate_click_off: Sound,
            pub pressure_plate_click_on: Sound,
            pub button_click_off: Sound,
            pub button_click_on: Sound,
        }

        impl BlockSetType {
            #constants
        }
    }
}

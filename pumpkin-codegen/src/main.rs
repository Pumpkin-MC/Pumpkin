use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rayon::prelude::*;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

mod attributes;
mod biome;
mod bitsets;
mod block;
mod block_state_remap;
mod chunk_gen_settings;
mod chunk_status;
mod composter_increase_chance;
mod configured_feature;
mod damage_type;
mod data_component;
mod dimension;
mod effect;
mod enchantments;
mod entity_pose;
mod entity_status;
mod entity_type;
mod flower_pot_transformations;
mod fluid;
mod fuels;
mod game_event;
mod game_rules;
mod item;
mod item_id_remap;
mod jukebox_song;
pub mod loot;
mod message_type;
mod meta_data_type;
mod noise_parameter;
mod noise_router;
mod packet;
mod particle;
mod placed_feature;
mod potion;
mod potion_brewing;
mod recipe_remainder;
mod recipes;
mod registry;
mod scoreboard_slot;
mod screen;
mod sound;
mod sound_category;
mod spawn_egg;
mod structures;
mod tag;
mod tracked_data;
mod translations;
mod world_event;

pub const OUT_DIR: &str = "../pumpkin-data/src/generated";
pub const WORLD_OUT_DIR: &str = placed_feature::WORLD_OUT_DIR;

pub fn main() {
    type BuilderFn = fn() -> TokenStream;

    fs::create_dir_all(OUT_DIR).expect("Failed to create output directory");

    let build_functions: Vec<(BuilderFn, &str)> = vec![
        (packet::build, "packet.rs"),
        (screen::build, "screen.rs"),
        (particle::build, "particle.rs"),
        (sound::build, "sound.rs"),
        (meta_data_type::build, "meta_data_type.rs"),
        (tracked_data::build, "tracked_data.rs"),
        (chunk_status::build, "chunk_status.rs"),
        (game_event::build, "game_event.rs"),
        (game_rules::build, "game_rules.rs"),
        (registry::build, "registry.rs"),
        (dimension::build, "dimension.rs"),
        (translations::build, "translation.rs"),
        (jukebox_song::build, "jukebox_song.rs"),
        (sound_category::build, "sound_category.rs"),
        (entity_pose::build, "entity_pose.rs"),
        (scoreboard_slot::build, "scoreboard_slot.rs"),
        (world_event::build, "world_event.rs"),
        (entity_type::build, "entity_type.rs"),
        (noise_parameter::build, "noise_parameter.rs"),
        (biome::build, "biome.rs"),
        (damage_type::build, "damage_type.rs"),
        (message_type::build, "message_type.rs"),
        (spawn_egg::build, "spawn_egg.rs"),
        (block::build, "block.rs"),
        (block_state_remap::build, "block_state_remap.rs"),
        (item::build, "item.rs"),
        (item_id_remap::build, "item_id_remap.rs"),
        (structures::build, "structures.rs"),
        (chunk_gen_settings::build, "chunk_gen_settings.rs"),
        (fluid::build, "fluid.rs"),
        (entity_status::build, "entity_status.rs"),
        (tag::build, "tag.rs"),
        (noise_router::build, "noise_router.rs"),
        (
            flower_pot_transformations::build,
            "flower_pot_transformations.rs",
        ),
        (
            composter_increase_chance::build,
            "composter_increase_chance.rs",
        ),
        (recipes::build, "recipes.rs"),
        (enchantments::build, "enchantment.rs"),
        (fuels::build, "fuels.rs"),
        (data_component::build, "data_component.rs"),
        (attributes::build, "attributes.rs"),
        (effect::build, "effect.rs"),
        (potion::build, "potion.rs"),
        (potion_brewing::build, "potion_brewing.rs"),
        (recipe_remainder::build, "recipe_remainder.rs"),
    ];

    build_functions.par_iter().for_each(|(build_fn, file)| {
        println!("Parsing {}", file);

        let raw_code = build_fn().to_string();

        let header = "/* This file is generated. Do not edit manually. */\n";

        let final_code = format_code(&raw_code).map_or_else(
            |_| format!("{header}{raw_code}"),
            |formatted| format!("{header}{formatted}"),
        );

        write_generated_file(&final_code, file);
    });

    // Generate world-specific files (placed/configured features)
    let world_build_functions: Vec<(BuilderFn, &str)> = vec![
        (placed_feature::build, "placed_features_generated.rs"),
        (configured_feature::build, "configured_features_generated.rs"),
    ];

    fs::create_dir_all(WORLD_OUT_DIR).expect("Failed to create world output directory");

    world_build_functions.par_iter().for_each(|(build_fn, file)| {
        println!("Parsing {}", file);

        let raw_code = build_fn().to_string();

        let header = "/* This file is generated. Do not edit manually. */\n";

        let final_code = format_code(&raw_code).map_or_else(
            |_| format!("{header}{raw_code}"),
            |formatted| format!("{header}{formatted}"),
        );

        write_world_generated_file(&final_code, file);
    });
    println!("Done")
}

#[must_use]
pub fn array_to_tokenstream(array: &[String]) -> TokenStream {
    let variants = array.iter().map(|item| {
        let name = format_ident!("{}", item.to_pascal_case());
        quote! { #name, }
    });

    quote! {
        #(#variants)*
    }
}

pub fn write_generated_file(new_code: &str, out_file: &str) {
    let path = Path::new(OUT_DIR).join(out_file);

    if path.exists()
        && let Ok(existing_code) = fs::read_to_string(&path)
        && existing_code == new_code
    {
        return;
    }

    fs::write(&path, new_code)
        .unwrap_or_else(|_| panic!("Failed to write to file: {}", path.display()));
}

pub fn write_world_generated_file(new_code: &str, out_file: &str) {
    let path = Path::new(WORLD_OUT_DIR).join(out_file);

    if path.exists()
        && let Ok(existing_code) = fs::read_to_string(&path)
        && existing_code == new_code
    {
        return;
    }

    fs::write(&path, new_code)
        .unwrap_or_else(|_| panic!("Failed to write to file: {}", path.display()));
}

pub struct RustFmtError;

pub fn format_code(unformatted_code: &str) -> Result<String, RustFmtError> {
    let child_result = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let Ok(mut child) = child_result else {
        return Err(RustFmtError);
    };

    // Write the code to rustfmt's stdin
    if let Some(mut stdin) = child.stdin.take()
        && stdin.write_all(unformatted_code.as_bytes()).is_err()
    {
        return Err(RustFmtError);
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).map_err(|_| RustFmtError)
        }
        _ => Err(RustFmtError),
    }
}

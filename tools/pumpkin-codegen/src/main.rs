#![allow(dead_code, unused)]
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::empty_structs_with_brackets,
    clippy::semicolon_outside_block,
    clippy::unreachable,
    clippy::undocumented_unsafe_blocks,
    clippy::needless_return,
    clippy::collapsible_if
)]

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

mod advancement;
mod attributes;
mod banner_pattern;
mod bedrock_biome;
mod bedrock_creative;
mod biome;
mod bitsets;
mod block;
mod block_transformer;
mod carver;
mod cat_variant;
mod chat_type;
mod chunk_status;
mod chunk_view_lut;
mod configured_feature;
mod context_provider;
mod damage_type;
mod data_component;
mod decorated_pot_pattern;
mod dimension;
mod dye_color;
mod effect;
mod enchantment_provider;
mod enchantments;
mod entity_pose;
mod entity_status;
mod entity_type;
mod environment_attribute;
mod flower_pot_transformations;
mod fluid;
mod frog_variant;
mod game_event;
mod game_rules;
mod instrument;
mod item;
mod jukebox_song;
pub mod loot_table;
mod map_color;
mod map_decoration;
mod material_rule;
mod message_type;
mod meta_data_type;
mod mob_variant;
mod noise_parameter;
mod noise_router;
mod noise_settings;
mod packet;
mod painting_variant;
mod particle;
mod placed_feature;
mod potion;
mod potion_brewing;
mod processor_list;
mod recipes;
mod registry;
mod scoreboard_slot;
mod screen;
mod sdk;
mod sound;
mod sound_category;
mod spawn_egg;
mod statistic;
mod structure_metadata;
mod structures;
mod tag;
mod template_bytes;
mod template_pool;
mod test_instance;
mod tracked_data;
mod translations;
mod trial_spawner;
mod trim_material;
mod trim_pattern;
mod version;
mod villager;
mod wit;
mod wolf_variant;
mod world_event;

/// Output directory where all generated Rust source files are written.
pub const OUT_DIR: &str = "../../crates/pumpkin-data/src/generated";

/// Entry point for the code generator. Runs all registered builder functions in parallel
/// and writes their output to [`OUT_DIR`].
pub fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(manifest_dir).expect("Failed to set current dir to manifest dir");

    type BuilderFn = fn() -> TokenStream;

    fs::create_dir_all(OUT_DIR).expect("Failed to create output directory");

    let mut build_functions: Vec<(BuilderFn, &str)> = vec![
        (advancement::build, "advancement.rs"),
        (bedrock_biome::build, "bedrock_biome.rs"),
        (bedrock_creative::build, "bedrock_creative.rs"),
        (packet::build, "packet.rs"),
        (screen::build, "screen.rs"),
        (particle::build, "particle.rs"),
        (sound::build, "sound.rs"),
        (meta_data_type::build, "meta_data_type.rs"),
        (tracked_data::build, "tracked_data.rs"),
        (chunk_status::build, "chunk_status.rs"),
        (chunk_view_lut::build, "chunk_view_lut.rs"),
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
        (statistic::build, "statistic.rs"),
        (noise_parameter::build, "noise_parameter.rs"),
        (biome::build, "biome.rs"),
        (damage_type::build, "damage_type.rs"),
        (message_type::build, "message_type.rs"),
        (spawn_egg::build, "spawn_egg.rs"),
        (block::build, "block.rs"),
        (item::build, "item.rs"),
        (structures::build, "structures.rs"),
        (template_pool::build, "template_pool.rs"),
        (processor_list::build, "processor_list.rs"),
        (structure_metadata::build, "structure_metadata.rs"),
        (template_bytes::build, "template_bytes.rs"),
        (test_instance::build, "test_instance.rs"),
        (painting_variant::build, "painting_variant.rs"),
        (context_provider::build, "context_provider.rs"),
        (material_rule::build, "material_rule.rs"),
        (noise_settings::build, "noise_settings.rs"),
        (fluid::build, "fluid.rs"),
        (entity_status::build, "entity_status.rs"),
        (tag::build, "tag.rs"),
        (noise_router::build, "noise_router.rs"),
        (villager::build, "villager.rs"),
        (
            flower_pot_transformations::build,
            "flower_pot_transformations.rs",
        ),
        (recipes::build, "recipes.rs"),
        (enchantments::build, "enchantment.rs"),
        (data_component::build, "data_component.rs"),
        (attributes::build, "attributes.rs"),
        (environment_attribute::build, "environment_attribute.rs"),
        (effect::build, "effect.rs"),
        (potion::build, "potion.rs"),
        (potion_brewing::build, "potion_brewing.rs"),
        (placed_feature::build_enum, "placed_feature.rs"),
        (placed_feature::build, "placed_features_generated.rs"),
        (configured_feature::build_enum, "configured_feature.rs"),
        (
            configured_feature::build,
            "configured_features_generated.rs",
        ),
        (carver::build, "carver.rs"),
        (loot_table::build, "loot_table.rs"),
        (map_color::build, "map_color.rs"),
        (map_decoration::build, "map_decoration.rs"),
        (dye_color::build, "dye_color.rs"),
        (block_transformer::build, "block_transformer.rs"),
        (trial_spawner::build, "trial_spawner.rs"),
        (banner_pattern::build, "banner_pattern.rs"),
        (cat_variant::build, "cat_variant.rs"),
        (chat_type::build, "chat_type.rs"),
        (decorated_pot_pattern::build, "decorated_pot_pattern.rs"),
        (frog_variant::build, "frog_variant.rs"),
        (instrument::build, "instrument.rs"),
        (trim_material::build, "trim_material.rs"),
        (trim_pattern::build, "trim_pattern.rs"),
        (wolf_variant::build, "wolf_variant.rs"),
        (enchantment_provider::build, "enchantment_provider.rs"),
        (mob_variant::build_cow, "cow_variant.rs"),
        (mob_variant::build_cow_sound, "cow_sound_variant.rs"),
        (mob_variant::build_pig, "pig_variant.rs"),
        (mob_variant::build_pig_sound, "pig_sound_variant.rs"),
        (mob_variant::build_chicken, "chicken_variant.rs"),
        (mob_variant::build_chicken_sound, "chicken_sound_variant.rs"),
        (mob_variant::build_cat_sound, "cat_sound_variant.rs"),
        (mob_variant::build_wolf_sound, "wolf_sound_variant.rs"),
        (
            mob_variant::build_zombie_nautilus,
            "zombie_nautilus_variant.rs",
        ),
    ];

    // If any arguments are given, treat them as file-stem filters.
    // e.g. `cargo run -- chest_loot` only regenerates chest_loot.rs.
    let filters: Vec<String> = std::env::args().skip(1).collect();
    if filters.iter().any(|f| f == "wit") {
        wit::main();
        return;
    }
    let build_functions: Vec<_> = if filters.is_empty() {
        wit::main();
        sdk::main();
        build_functions
    } else {
        build_functions
            .into_iter()
            .filter(|(_, file)| {
                let stem = file.trim_end_matches(".rs");
                filters.iter().any(|f| f == stem || f == *file)
            })
            .collect()
    };

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
    println!("Done")
}

/// Converts a slice of strings into a `TokenStream` of PascalCase enum variants.
///
/// # Arguments
/// - `array` – Slice of raw name strings to convert into variant identifiers.
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

/// Writes generated source code to a file in [`OUT_DIR`], skipping the write if the
/// content is unchanged.
///
/// # Arguments
/// - `new_code` – The formatted source code string to write.
/// - `out_file` – The filename (relative to [`OUT_DIR`]) to write into.
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

/// Error returned when `rustfmt` is unavailable or fails to format code.
#[derive(Debug)]
pub struct RustFmtError;

/// Formats a Rust source string by piping it through `rustfmt`.
///
/// # Arguments
/// - `unformatted_code` – Raw Rust source code to format.
///
/// # Returns
/// The formatted source string, or `Err(RustFmtError)` if `rustfmt` is not available
/// or formatting fails.
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

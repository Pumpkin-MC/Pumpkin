use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rayon::prelude::*;
use std::{fs, io::Write, path::Path, process::Command};

mod attributes;
mod biome;
mod block;
mod chunk_status;
mod composter_increase_chance;
mod damage_type;
mod data_component;
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
pub mod loot;
mod message_type;
mod noise_parameter;
mod noise_router;
mod packet;
mod particle;
mod potion;
mod potion_brewing;
mod recipes;
mod scoreboard_slot;
mod screen;
mod sound;
mod sound_category;
mod spawn_egg;
mod tag;
mod world_event;

pub const OUT_DIR: &str = "src/generated";

pub fn main() {
    let path = Path::new(OUT_DIR);
    if !path.exists() {
        let _ = fs::create_dir(OUT_DIR);
    }
    #[allow(clippy::type_complexity)]
    let build_functions: Vec<(fn() -> TokenStream, &str)> = vec![
        (packet::build, "packet.rs"),
        (screen::build, "screen.rs"),
        (particle::build, "particle.rs"),
        (sound::build, "sound.rs"),
        (chunk_status::build, "chunk_status.rs"),
        (game_event::build, "game_event.rs"),
        (game_rules::build, "game_rules.rs"),
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
        (item::build, "item.rs"),
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
    ];

    build_functions.par_iter().for_each(|(build_fn, file)| {
        write_generated_file(build_fn(), file);
    });
}

pub fn array_to_tokenstream(array: &[String]) -> TokenStream {
    let mut variants = TokenStream::new();

    for item in array.iter() {
        let name = format_ident!("{}", item.to_pascal_case());
        variants.extend([quote! {
            #name,
        }]);
    }
    variants
}

pub fn write_generated_file(content: TokenStream, out_file: &str) {
    let path = Path::new(OUT_DIR).join(out_file);
    let code = content.to_string();

    let mut file = fs::File::create(&path).unwrap();
    if let Err(e) = file.write_all(code.as_bytes()) {
        println!("cargo::error={e}");
    }

    // Try to format the output for debugging purposes.
    // Doesn't matter if rustfmt is unavailable.
    let _ = Command::new("rustfmt").arg(&path).output();
}

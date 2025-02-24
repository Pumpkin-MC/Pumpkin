use quote::{format_ident, quote};
use std::{env, fs, path::Path, process::Command};

use heck::ToPascalCase;
use proc_macro2::TokenStream;

mod biome;
mod chunk_status;
mod damage_type;
mod entity_pose;
mod entity_type;
mod fluid;
mod game_event;
mod item;
mod message_type;
mod noise_parameter;
mod packet;
mod particle;
mod scoreboard_slot;
mod screen;
mod sound;
mod sound_category;
mod spawn_egg;
mod status_effect;
mod world_event;

pub fn main() {
    write_generated_file(packet::build(), "packet.rs");
    write_generated_file(screen::build(), "screen.rs");
    write_generated_file(particle::build(), "particle.rs");
    write_generated_file(sound::build(), "sound.rs");
    write_generated_file(chunk_status::build(), "chunk_status.rs");
    write_generated_file(game_event::build(), "game_event.rs");
    write_generated_file(sound_category::build(), "sound_category.rs");
    write_generated_file(entity_pose::build(), "entity_pose.rs");
    write_generated_file(scoreboard_slot::build(), "scoreboard_slot.rs");
    write_generated_file(world_event::build(), "world_event.rs");
    write_generated_file(entity_type::build(), "entity_type.rs");
    write_generated_file(noise_parameter::build(), "noise_parameter.rs");
    write_generated_file(biome::build(), "biome.rs");
    write_generated_file(damage_type::build(), "damage_type.rs");
    write_generated_file(message_type::build(), "message_type.rs");
    write_generated_file(spawn_egg::build(), "spawn_egg.rs");
    write_generated_file(item::build(), "item.rs");
    write_generated_file(fluid::build(), "fluid.rs");
    write_generated_file(status_effect::build(), "status_effect.rs");
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
    let out_dir = env::var_os("OUT_DIR").expect("failed to get OUT_DIR env var");
    let path = Path::new(&out_dir).join(out_file);
    let code = content.to_string();

    fs::write(&path, code).expect("Failed to write to fs");

    // Try to format the output for debugging purposes.
    // Doesn't matter if rustfmt is unavailable.
    let _ = Command::new("rustfmt").arg(path).output();
}

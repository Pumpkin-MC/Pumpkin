#![allow(clippy::too_many_lines)]
use pumpkin_data::biome::Biome;
use pumpkin_data::structures::{StructureKeys, StructureSet};
use pumpkin_data::tag::{RegistryKey, get_tag_ids, get_tag_values};
use pumpkin_data::translation::java::{
    CHAT_COORDINATES, CHAT_COORDINATES_TOOLTIP, COMMANDS_LOCATE_BIOME_NOT_FOUND,
    COMMANDS_LOCATE_BIOME_SUCCESS, COMMANDS_LOCATE_POI_NOT_FOUND, COMMANDS_LOCATE_POI_SUCCESS,
    COMMANDS_LOCATE_STRUCTURE_INVALID, COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
    COMMANDS_LOCATE_STRUCTURE_SUCCESS,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;
use std::time::Instant;

use crate::command::args::ConsumedArgs;
use crate::command::args::FindArg;
use crate::command::args::resource::biome::BiomeArgumentConsumer;
use crate::command::args::resource::poi::PoiArgumentConsumer;
use crate::command::args::resource::structure::StructureArgumentConsumer;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["locate"];
const DESCRIPTION: &str = "Locates a structure, biome or point of interest.";

struct StructureExecutor;
struct BiomeExecutor;
struct PoiExecutor;

impl CommandExecutor for StructureExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let position = sender.position().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(
                    "This command can only be executed in a world context",
                ))
            })?;
            let world = sender.world().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(
                    "This command can only be executed in a world context",
                ))
            })?;

            let source_pos = BlockPos::floored_v(position);
            let raw_structure = StructureArgumentConsumer::find_arg(args, "structure")?;

            let mut target_keys = Vec::new();
            let mut display_name = raw_structure.to_string();

            if raw_structure.starts_with('#') {
                if let Some(keys) = StructureSet::get_structures_by_tag(raw_structure) {
                    target_keys = keys;
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_STRUCTURE_INVALID,
                        COMMANDS_LOCATE_STRUCTURE_INVALID,
                        [TextComponent::text(raw_structure.to_string())],
                    )));
                }
            } else if let Some(key) = StructureKeys::from_registry_name(raw_structure) {
                target_keys.push(key);
            } else {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_STRUCTURE_INVALID,
                    COMMANDS_LOCATE_STRUCTURE_INVALID,
                    [TextComponent::text(raw_structure.to_string())],
                )));
            }

            let mut placements = Vec::new();
            for key in &target_keys {
                for set in StructureSet::ALL {
                    if set.structures.iter().any(|e| e.structure == *key) {
                        placements.push(&set.placement);
                    }
                }
            }

            if placements.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
                    COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
                    [TextComponent::text(raw_structure.to_string())],
                )));
            }

            let mut allowed_biomes_mask = [false; 256];
            for key in &target_keys {
                let struct_config = pumpkin_data::structures::Structure::get(key);
                let tag_name = struct_config
                    .biomes
                    .strip_prefix('#')
                    .unwrap_or(struct_config.biomes);
                let tag_ids = get_tag_ids(RegistryKey::WorldgenBiome, tag_name).or_else(|| {
                    get_tag_ids(RegistryKey::WorldgenBiome, &format!("minecraft:{tag_name}"))
                });
                if let Some(ids) = tag_ids {
                    for &id in ids {
                        allowed_biomes_mask[id as usize] = true;
                    }
                }
            }

            let start = Instant::now();
            let world_gen = world.level.world_gen.clone();
            let dimension = world.dimension.clone();

            let nearest_pos = tokio::task::spawn_blocking(move || {
                pumpkin_world::generation::locator::find_nearest_structure_pos(
                    &world_gen,
                    dimension,
                    source_pos,
                    &placements,
                    allowed_biomes_mask,
                )
            })
            .await
            .map_err(|e| CommandError::CommandFailed(TextComponent::text(e.to_string())))?;

            let elapsed = start.elapsed();

            let Some(found_pos) = nearest_pos else {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
                    COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
                    [TextComponent::text(raw_structure.to_string())],
                )));
            };

            let dx = found_pos.0.x - source_pos.0.x;
            let dz = found_pos.0.z - source_pos.0.z;
            let distance = ((dx * dx + dz * dz) as f64).sqrt();

            if !raw_structure.starts_with('#') && !raw_structure.contains(':') {
                display_name = format!("minecraft:{raw_structure}");
            }

            let coord_text = format_coordinates(found_pos.0.x, "~", found_pos.0.z);
            let distance_str = format!("{}", distance.floor() as i32);

            let feedback = TextComponent::translate_cross(
                COMMANDS_LOCATE_STRUCTURE_SUCCESS,
                COMMANDS_LOCATE_STRUCTURE_SUCCESS,
                [
                    TextComponent::text(display_name),
                    coord_text,
                    TextComponent::text(distance_str),
                ],
            );

            sender.send_message(feedback).await;
            tracing::info!(
                "Locating element {} took {} ms",
                raw_structure,
                elapsed.as_millis()
            );

            Ok(distance.floor() as i32)
        })
    }
}

impl CommandExecutor for BiomeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let position = sender.position().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(
                    "This command can only be executed in a world context",
                ))
            })?;
            let world = sender.world().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(
                    "This command can only be executed in a world context",
                ))
            })?;

            let source_pos = BlockPos::floored_v(position);
            let raw_biome = BiomeArgumentConsumer::find_arg(args, "biome")?;

            let mut biome_mask = [false; 256];
            let mut display_name = raw_biome.to_string();

            if let Some(tag_name) = raw_biome.strip_prefix('#') {
                let tag_ids = get_tag_ids(RegistryKey::WorldgenBiome, tag_name).or_else(|| {
                    get_tag_ids(RegistryKey::WorldgenBiome, &format!("minecraft:{tag_name}"))
                });
                if let Some(ids) = tag_ids {
                    for &id in ids {
                        biome_mask[id as usize] = true;
                    }
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_BIOME_NOT_FOUND,
                        COMMANDS_LOCATE_BIOME_NOT_FOUND,
                        [TextComponent::text(raw_biome.to_string())],
                    )));
                }
            } else {
                let biome_name = raw_biome.strip_prefix("minecraft:").unwrap_or(raw_biome);
                if let Some(biome) = Biome::from_name(biome_name) {
                    biome_mask[biome.id as usize] = true;
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_BIOME_NOT_FOUND,
                        COMMANDS_LOCATE_BIOME_NOT_FOUND,
                        [TextComponent::text(raw_biome.to_string())],
                    )));
                }
            }

            if !biome_mask.iter().any(|&v| v) {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    [TextComponent::text(raw_biome.to_string())],
                )));
            }

            let dimension = world.dimension.clone();
            let min_y = world.level.world_gen.settings.shape.min_y as i32;
            let height = world.level.world_gen.settings.shape.height as i32;
            let world_gen = world.level.world_gen.clone();

            let best_match = tokio::task::spawn_blocking(move || {
                pumpkin_world::generation::locator::find_nearest_biome(
                    &world_gen,
                    dimension,
                    source_pos,
                    &biome_mask,
                    min_y,
                    height,
                )
            })
            .await
            .map_err(|e| CommandError::CommandFailed(TextComponent::text(e.to_string())))?;

            let Some((found_pos, distance)) = best_match else {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    [TextComponent::text(raw_biome.to_string())],
                )));
            };

            if !raw_biome.starts_with('#') && !raw_biome.contains(':') {
                display_name = format!("minecraft:{raw_biome}");
            }

            let coord_text =
                format_coordinates(found_pos.0.x, &found_pos.0.y.to_string(), found_pos.0.z);
            let distance_str = format!("{}", distance.floor() as i32);

            let feedback = TextComponent::translate_cross(
                COMMANDS_LOCATE_BIOME_SUCCESS,
                COMMANDS_LOCATE_BIOME_SUCCESS,
                [
                    TextComponent::text(display_name),
                    coord_text,
                    TextComponent::text(distance_str),
                ],
            );

            sender.send_message(feedback).await;

            Ok(distance.floor() as i32)
        })
    }
}

impl CommandExecutor for PoiExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let position = sender.position().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(
                    "This command can only be executed in a world context",
                ))
            })?;
            let world = sender.world().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(
                    "This command can only be executed in a world context",
                ))
            })?;

            let source_pos = BlockPos::floored_v(position);
            let raw_poi = PoiArgumentConsumer::find_arg(args, "poi")?;

            let mut target_names = Vec::new();
            let mut display_name = raw_poi.to_string();

            if let Some(tag_name) = raw_poi.strip_prefix('#') {
                let tag_vals =
                    get_tag_values(RegistryKey::PointOfInterestType, tag_name).or_else(|| {
                        get_tag_values(
                            RegistryKey::PointOfInterestType,
                            &format!("minecraft:{tag_name}"),
                        )
                    });
                if let Some(vals) = tag_vals {
                    target_names = vals.iter().map(|&s| s.to_string()).collect();
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_POI_NOT_FOUND,
                        COMMANDS_LOCATE_POI_NOT_FOUND,
                        [TextComponent::text(raw_poi.to_string())],
                    )));
                }
            } else {
                let full_name = if raw_poi.contains(':') {
                    raw_poi.to_string()
                } else {
                    format!("minecraft:{raw_poi}")
                };
                target_names.push(full_name);
            }

            if target_names.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_POI_NOT_FOUND,
                    COMMANDS_LOCATE_POI_NOT_FOUND,
                    [TextComponent::text(raw_poi.to_string())],
                )));
            }

            let world = world.clone();
            let center = source_pos;
            let radius = 256;
            let target_names = target_names.clone();

            let results = tokio::task::spawn_blocking(move || {
                pumpkin_world::generation::locator::find_nearby_pois(
                    &world.portal_poi,
                    center,
                    radius,
                    &target_names,
                )
            })
            .await
            .map_err(|e| CommandError::CommandFailed(TextComponent::text(e.to_string())))?;

            let mut best_match: Option<(BlockPos, f64)> = None;
            for (found_pos, _poi_type) in results {
                let dx = found_pos.0.x - source_pos.0.x;
                let dz = found_pos.0.z - source_pos.0.z;
                let dist = ((dx * dx + dz * dz) as f64).sqrt();
                if best_match.as_ref().is_none_or(|&(_, d)| dist < d) {
                    best_match = Some((found_pos, dist));
                }
            }

            let Some((found_pos, distance)) = best_match else {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_POI_NOT_FOUND,
                    COMMANDS_LOCATE_POI_NOT_FOUND,
                    [TextComponent::text(raw_poi.to_string())],
                )));
            };

            if !raw_poi.starts_with('#') && !raw_poi.contains(':') {
                display_name = format!("minecraft:{raw_poi}");
            }

            let coord_text = format_coordinates(found_pos.0.x, "~", found_pos.0.z);
            let distance_str = format!("{}", distance.floor() as i32);

            let feedback = TextComponent::translate_cross(
                COMMANDS_LOCATE_POI_SUCCESS,
                COMMANDS_LOCATE_POI_SUCCESS,
                [
                    TextComponent::text(display_name),
                    coord_text,
                    TextComponent::text(distance_str),
                ],
            );

            sender.send_message(feedback).await;

            Ok(distance.floor() as i32)
        })
    }
}

fn format_coordinates(x: i32, y_str: &str, z: i32) -> TextComponent {
    let tooltip =
        TextComponent::translate_cross(CHAT_COORDINATES_TOOLTIP, CHAT_COORDINATES_TOOLTIP, []);
    let x_str = x.to_string();
    let z_str = z.to_string();
    TextComponent::wrap_in_square_brackets(TextComponent::translate_cross(
        CHAT_COORDINATES,
        CHAT_COORDINATES,
        [
            TextComponent::text(x_str.clone()),
            TextComponent::text(y_str.to_string()),
            TextComponent::text(z_str.clone()),
        ],
    ))
    .color_named(NamedColor::Green)
    .click_event(ClickEvent::SuggestCommand {
        command: Cow::from(format!("/tp @s {x_str} {y_str} {z_str}")),
    })
    .hover_event(HoverEvent::show_text(tooltip))
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("structure")
                .then(argument("structure", StructureArgumentConsumer).execute(StructureExecutor)),
        )
        .then(
            literal("biome").then(argument("biome", BiomeArgumentConsumer).execute(BiomeExecutor)),
        )
        .then(literal("poi").then(argument("poi", PoiArgumentConsumer).execute(PoiExecutor)))
}

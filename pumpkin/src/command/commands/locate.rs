#![allow(clippy::too_many_lines)]
use pumpkin_data::biome::Biome;
use pumpkin_data::dimension::Dimension;
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
use pumpkin_world::biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use pumpkin_world::generation::biome_coords;
use pumpkin_world::generation::generator::structure_finder::find_nearest_structure;
use pumpkin_world::generation::noise::router::multi_noise_sampler::{
    MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
};
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
                if let Some(keys) = get_structures_by_tag(raw_structure) {
                    target_keys = keys;
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_STRUCTURE_INVALID,
                        COMMANDS_LOCATE_STRUCTURE_INVALID,
                        [TextComponent::text(raw_structure.to_string())],
                    )));
                }
            } else {
                if let Some(key) = StructureKeys::from_registry_name(raw_structure) {
                    target_keys.push(key);
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_STRUCTURE_INVALID,
                        COMMANDS_LOCATE_STRUCTURE_INVALID,
                        [TextComponent::text(raw_structure.to_string())],
                    )));
                }
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

            let dimension = &world.dimension;
            let overworld_supplier = MultiNoiseBiomeSupplier::OVERWORLD;
            let nether_supplier = MultiNoiseBiomeSupplier::NETHER;
            let end_supplier = TheEndBiomeSupplier;

            let base_supplier: &dyn BiomeSupplier = if *dimension == Dimension::OVERWORLD {
                &overworld_supplier
            } else if *dimension == Dimension::THE_NETHER {
                &nether_supplier
            } else if *dimension == Dimension::THE_END {
                &end_supplier
            } else {
                &overworld_supplier
            };

            let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
            let mut multi_noise_sampler = MultiNoiseSampler::generate(
                &world.level.world_gen.base_router.multi_noise,
                &multi_noise_config,
            );

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
            let world_seed = world.level.seed.0 as i64;
            let nearest_pos = find_nearest_structure(
                source_pos,
                &placements,
                100,
                world_seed,
                &world.level.world_gen.global_structure_cache,
                |pos, _placement| {
                    let bx = biome_coords::from_block(pos.0.x);
                    let by = biome_coords::from_block(64);
                    let bz = biome_coords::from_block(pos.0.z);
                    let sampled_biome = base_supplier.biome(bx, by, bz, &mut multi_noise_sampler);
                    allowed_biomes_mask[sampled_biome.id as usize]
                },
            );
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

            let mut target_biome_ids = Vec::new();
            let mut display_name = raw_biome.to_string();

            if let Some(tag_name) = raw_biome.strip_prefix('#') {
                let tag_ids = get_tag_ids(RegistryKey::WorldgenBiome, tag_name).or_else(|| {
                    get_tag_ids(RegistryKey::WorldgenBiome, &format!("minecraft:{tag_name}"))
                });
                if let Some(ids) = tag_ids {
                    target_biome_ids = ids.iter().map(|&id| id as u8).collect();
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
                    target_biome_ids.push(biome.id);
                } else {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        COMMANDS_LOCATE_BIOME_NOT_FOUND,
                        COMMANDS_LOCATE_BIOME_NOT_FOUND,
                        [TextComponent::text(raw_biome.to_string())],
                    )));
                }
            }

            if target_biome_ids.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    [TextComponent::text(raw_biome.to_string())],
                )));
            }

            let dimension = &world.dimension;
            let overworld_supplier = MultiNoiseBiomeSupplier::OVERWORLD;
            let nether_supplier = MultiNoiseBiomeSupplier::NETHER;
            let end_supplier = TheEndBiomeSupplier;

            let base_supplier: &dyn BiomeSupplier = if *dimension == Dimension::OVERWORLD {
                &overworld_supplier
            } else if *dimension == Dimension::THE_NETHER {
                &nether_supplier
            } else if *dimension == Dimension::THE_END {
                &end_supplier
            } else {
                &overworld_supplier
            };

            let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
            let mut multi_noise_sampler = MultiNoiseSampler::generate(
                &world.level.world_gen.base_router.multi_noise,
                &multi_noise_config,
            );

            let px = source_pos.0.x;
            let py = source_pos.0.y;
            let pz = source_pos.0.z;

            let shape = &world.level.world_gen.settings.shape;
            let min_y = shape.min_y as i32;
            let height = shape.height as i32;
            let max_y = min_y + height - 1;

            let mut y_coords = Vec::new();
            let mut y = min_y;
            while y <= max_y {
                y_coords.push(y);
                y += 64;
            }
            y_coords.sort_by_key(|&val| (val - py).abs());

            let mut best_match: Option<(BlockPos, f64)> = None;

            for r_step in 0..=200 {
                let r = r_step * 32;
                if let Some((_, best_d)) = best_match
                    && r as f64 > best_d
                {
                    break;
                }

                let perimeter_points = get_perimeter_points(px, pz, r);
                for (x, z) in perimeter_points {
                    for &y in &y_coords {
                        let bx = biome_coords::from_block(x);
                        let by = biome_coords::from_block(y);
                        let bz = biome_coords::from_block(z);

                        let sampled_biome =
                            base_supplier.biome(bx, by, bz, &mut multi_noise_sampler);
                        if target_biome_ids.contains(&sampled_biome.id) {
                            let dx = x - px;
                            let dy = y - py;
                            let dz = z - pz;
                            let dist = ((dx * dx + dy * dy + dz * dz) as f64).sqrt();
                            if best_match.as_ref().is_none_or(|&(_, d)| dist < d) {
                                best_match = Some((BlockPos::new(x, y, z), dist));
                            }
                        }
                    }
                }
            }

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

            let results = {
                let mut storage = world.portal_poi.lock().await;
                storage.get_in_square_filtered(source_pos, 256, |poi_type| {
                    target_names.iter().any(|target| target == poi_type)
                })
            };

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

fn get_perimeter_points(px: i32, pz: i32, r: i32) -> Vec<(i32, i32)> {
    if r == 0 {
        return vec![(px, pz)];
    }

    let mut points = Vec::new();

    let mut z = pz - r;
    while z <= pz + r {
        points.push((px - r, z));
        points.push((px + r, z));
        z += 32;
    }

    let mut x = px - r + 32;
    while x <= px + r - 32 {
        points.push((x, pz - r));
        points.push((x, pz + r));
        x += 32;
    }

    points
}

fn get_structures_by_tag(tag: &str) -> Option<Vec<StructureKeys>> {
    let tag = tag.strip_prefix('#').unwrap_or(tag);
    let tag = tag.strip_prefix("minecraft:").unwrap_or(tag);
    match tag {
        "village" => Some(vec![
            StructureKeys::VillagePlains,
            StructureKeys::VillageDesert,
            StructureKeys::VillageSavanna,
            StructureKeys::VillageSnowy,
            StructureKeys::VillageTaiga,
        ]),
        "mineshaft" => Some(vec![StructureKeys::Mineshaft, StructureKeys::MineshaftMesa]),
        "shipwreck" => Some(vec![
            StructureKeys::Shipwreck,
            StructureKeys::ShipwreckBeached,
        ]),
        "ruined_portal" => Some(vec![
            StructureKeys::RuinedPortal,
            StructureKeys::RuinedPortalDesert,
            StructureKeys::RuinedPortalJungle,
            StructureKeys::RuinedPortalSwamp,
            StructureKeys::RuinedPortalMountain,
            StructureKeys::RuinedPortalOcean,
            StructureKeys::RuinedPortalNether,
        ]),
        "ocean_ruin" => Some(vec![
            StructureKeys::OceanRuinCold,
            StructureKeys::OceanRuinWarm,
        ]),
        "cats_spawn_in" => Some(vec![StructureKeys::SwampHut]),
        _ => None,
    }
}

trait StructureKeysExt {
    fn from_registry_name(name: &str) -> Option<StructureKeys>;
}

impl StructureKeysExt for StructureKeys {
    fn from_registry_name(name: &str) -> Option<Self> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        match name {
            "pillager_outpost" => Some(Self::PillagerOutpost),
            "mineshaft" => Some(Self::Mineshaft),
            "mineshaft_mesa" => Some(Self::MineshaftMesa),
            "mansion" | "woodland_mansion" => Some(Self::Mansion),
            "jungle_pyramid" | "jungle_temple" => Some(Self::JunglePyramid),
            "desert_pyramid" => Some(Self::DesertPyramid),
            "igloo" => Some(Self::Igloo),
            "shipwreck" => Some(Self::Shipwreck),
            "shipwreck_beached" => Some(Self::ShipwreckBeached),
            "swamp_hut" => Some(Self::SwampHut),
            "stronghold" => Some(Self::Stronghold),
            "monument" | "ocean_monument" => Some(Self::Monument),
            "ocean_ruin_cold" => Some(Self::OceanRuinCold),
            "ocean_ruin_warm" => Some(Self::OceanRuinWarm),
            "fortress" => Some(Self::Fortress),
            "nether_fossil" => Some(Self::NetherFossil),
            "end_city" => Some(Self::EndCity),
            "buried_treasure" => Some(Self::BuriedTreasure),
            "bastion_remnant" => Some(Self::BastionRemnant),
            "village_plains" => Some(Self::VillagePlains),
            "village_desert" => Some(Self::VillageDesert),
            "village_savanna" => Some(Self::VillageSavanna),
            "village_snowy" => Some(Self::VillageSnowy),
            "village_taiga" => Some(Self::VillageTaiga),
            "ruined_portal" => Some(Self::RuinedPortal),
            "ruined_portal_desert" => Some(Self::RuinedPortalDesert),
            "ruined_portal_jungle" => Some(Self::RuinedPortalJungle),
            "ruined_portal_swamp" => Some(Self::RuinedPortalSwamp),
            "ruined_portal_mountain" => Some(Self::RuinedPortalMountain),
            "ruined_portal_ocean" => Some(Self::RuinedPortalOcean),
            "ruined_portal_nether" => Some(Self::RuinedPortalNether),
            "ancient_city" => Some(Self::AncientCity),
            "trail_ruins" => Some(Self::TrailRuins),
            "trial_chambers" => Some(Self::TrialChambers),
            _ => None,
        }
    }
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

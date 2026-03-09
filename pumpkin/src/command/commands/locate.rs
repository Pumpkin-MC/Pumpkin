use pumpkin_data::structures::{StructurePlacementCalculator, StructureSet};
use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::generation::structure::placement::should_generate_structure;

use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["locate"];

const DESCRIPTION: &str = "Locates the closest structure, biome, or point of interest.";

const ARG_STRUCTURE: &str = "structure";
const ARG_BIOME: &str = "biome";
const ARG_POI: &str = "poi";

const MAX_SEARCH_RADIUS: i32 = 6400;

fn find_nearest_structure(
    structure_set: &StructureSet,
    seed: i64,
    center_chunk_x: i32,
    center_chunk_z: i32,
) -> Option<Vector2<i32>> {
    let calculator = StructurePlacementCalculator::new(seed);

    let mut best: Option<(Vector2<i32>, i64)> = None;

    for radius in 0..=MAX_SEARCH_RADIUS {
        for dx in -radius..=radius {
            let dz_values = if dx == -radius || dx == radius {
                (-radius..=radius).collect::<Vec<_>>()
            } else {
                vec![-radius, radius]
            };

            for dz in dz_values {
                let cx = center_chunk_x + dx;
                let cz = center_chunk_z + dz;

                if should_generate_structure(&structure_set.placement, &calculator, cx, cz) {
                    let dist_sq = (dx as i64) * (dx as i64) + (dz as i64) * (dz as i64);
                    if best.is_none() || dist_sq < best.unwrap().1 {
                        best = Some((Vector2::new(cx, cz), dist_sq));
                    }
                }
            }
        }

        // If we found something in this ring, we can stop since further rings are farther
        if best.is_some() {
            break;
        }
    }

    best.map(|(pos, _)| pos)
}

struct LocateStructureExecutor;

impl CommandExecutor for LocateStructureExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_STRUCTURE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_STRUCTURE.into())));
            };

            let clean_name = name.strip_prefix("minecraft:").unwrap_or(name);

            let structure_set = StructureSet::get(clean_name).ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_LOCATE_STRUCTURE_INVALID,
                    [TextComponent::text(name.to_string())],
                ))
            })?;

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let seed = world.level.seed.0 as i64;

            let (center_x, center_z) = match sender {
                CommandSender::Player(player) => {
                    let pos = player.living_entity.entity.pos.load();
                    ((pos.x as i32) >> 4, (pos.z as i32) >> 4)
                }
                _ => (0, 0),
            };

            let result = find_nearest_structure(structure_set, seed, center_x, center_z);

            match result {
                Some(chunk_pos) => {
                    let block_x = chunk_pos.x * 16 + 8;
                    let block_z = chunk_pos.y * 16 + 8;
                    let dx = block_x - (center_x * 16 + 8);
                    let dz = block_z - (center_z * 16 + 8);
                    let distance = (dx as f64).hypot(dz as f64) as i32;

                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_LOCATE_STRUCTURE_SUCCESS,
                            [
                                TextComponent::text(clean_name.to_string()),
                                TextComponent::text(format!("[{block_x}, ~, {block_z}]")),
                                TextComponent::text(distance.to_string()),
                            ],
                        ))
                        .await;
                    Ok(1)
                }
                None => Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
                    [TextComponent::text(clean_name.to_string())],
                ))),
            }
        })
    }
}

struct LocateBiomeExecutor;

impl CommandExecutor for LocateBiomeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_BIOME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_BIOME.into())));
            };

            let clean_name = name.strip_prefix("minecraft:").unwrap_or(name);

            let target_biome =
                pumpkin_data::biome::Biome::from_name(clean_name).ok_or_else(|| {
                    CommandError::CommandFailed(TextComponent::text(format!(
                        "Unknown biome: {name}"
                    )))
                })?;

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

            let (center_x, center_z) = match sender {
                CommandSender::Player(player) => {
                    let pos = player.living_entity.entity.pos.load();
                    (pos.x as i32, pos.z as i32)
                }
                _ => (0, 0),
            };

            // Search in expanding square pattern, checking biome every 32 blocks
            let step = 32;
            let max_radius = 6400 * 16; // Same as structure search but in blocks

            let result = search_biome(
                &world,
                target_biome.id,
                center_x,
                center_z,
                step,
                max_radius,
            )
            .await;

            match result {
                Some(pos) => {
                    let dx = pos.x - center_x;
                    let dz = pos.y - center_z;
                    let distance = (dx as f64).hypot(dz as f64) as i32;

                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_LOCATE_BIOME_SUCCESS,
                            [
                                TextComponent::text(clean_name.to_string()),
                                TextComponent::text(format!("[{}, ~, {}]", pos.x, pos.y)),
                                TextComponent::text(distance.to_string()),
                            ],
                        ))
                        .await;
                    Ok(1)
                }
                None => Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_LOCATE_BIOME_NOT_FOUND,
                    [TextComponent::text(clean_name.to_string())],
                ))),
            }
        })
    }
}

async fn search_biome(
    world: &crate::world::World,
    target_id: u8,
    center_x: i32,
    center_z: i32,
    step: i32,
    max_radius: i32,
) -> Option<Vector2<i32>> {
    let mut radius = 0;
    while radius <= max_radius {
        if radius == 0 {
            let pos = BlockPos(Vector3::new(center_x, 64, center_z));
            let biome = world.level.get_rough_biome(&pos).await;
            if biome.id == target_id {
                return Some(Vector2::new(center_x, center_z));
            }
            radius += step;
            continue;
        }

        // Check edges of square at current radius
        let min = -radius;
        let max = radius;

        // Top and bottom edges
        let mut x = min;
        while x <= max {
            for &z in &[min, max] {
                let bx = center_x + x;
                let bz = center_z + z;
                let pos = BlockPos(Vector3::new(bx, 64, bz));
                let biome = world.level.get_rough_biome(&pos).await;
                if biome.id == target_id {
                    return Some(Vector2::new(bx, bz));
                }
            }
            x += step;
        }

        // Left and right edges (excluding corners)
        let mut z = min + step;
        while z < max {
            for &x in &[min, max] {
                let bx = center_x + x;
                let bz = center_z + z;
                let pos = BlockPos(Vector3::new(bx, 64, bz));
                let biome = world.level.get_rough_biome(&pos).await;
                if biome.id == target_id {
                    return Some(Vector2::new(bx, bz));
                }
            }
            z += step;
        }

        radius += step;
    }
    None
}

struct LocatePoiExecutor;

impl CommandExecutor for LocatePoiExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_POI)?;

            // TODO: Implement POI location when POI system is available
            Err(CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_LOCATE_POI_NOT_FOUND,
                [TextComponent::text(name.to_string())],
            )))
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("structure")
                .then(argument(ARG_STRUCTURE, SimpleArgConsumer).execute(LocateStructureExecutor)),
        )
        .then(
            literal("biome")
                .then(argument(ARG_BIOME, SimpleArgConsumer).execute(LocateBiomeExecutor)),
        )
        .then(literal("poi").then(argument(ARG_POI, SimpleArgConsumer).execute(LocatePoiExecutor)))
}

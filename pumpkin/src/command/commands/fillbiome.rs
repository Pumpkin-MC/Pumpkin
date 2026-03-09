use pumpkin_data::biome::Biome;
use pumpkin_data::translation;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::world::World;

const NAMES: [&str; 1] = ["fillbiome"];

const DESCRIPTION: &str = "Fills a region with a specific biome.";

const ARG_FROM: &str = "from";
const ARG_TO: &str = "to";
const ARG_BIOME: &str = "biome";
const ARG_REPLACE: &str = "replace_biome";

fn parse_biome(name: &str) -> Result<&'static Biome, CommandError> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    Biome::from_name(name).ok_or(CommandError::CommandFailed(TextComponent::translate(
        "argument.resource.invalid_type",
        [
            TextComponent::text(format!("minecraft:{name}")),
            TextComponent::text("worldgen/biome".to_string()),
        ],
    )))
}

async fn fill_biome_region(
    world: &World,
    min: Vector3<i32>,
    max: Vector3<i32>,
    biome: &'static Biome,
    replace_biome: Option<&'static Biome>,
) -> i32 {
    let biome_min_x = min.x >> 2;
    let biome_min_y = min.y >> 2;
    let biome_min_z = min.z >> 2;
    let biome_max_x = max.x >> 2;
    let biome_max_y = max.y >> 2;
    let biome_max_z = max.z >> 2;

    let mut changed = 0i32;

    for bx in biome_min_x..=biome_max_x {
        for bz in biome_min_z..=biome_max_z {
            let chunk_x = bx >> 2;
            let chunk_z = bz >> 2;
            let chunk_pos = Vector2::new(chunk_x, chunk_z);
            let chunk = world.level.get_chunk(chunk_pos).await;

            let rel_x = (bx & 3) as usize;
            let rel_z = (bz & 3) as usize;

            for by in biome_min_y..=biome_max_y {
                if let Some(replace) = replace_biome {
                    let section_idx = ((by - (chunk.section.min_y >> 2)) / 4).max(0) as usize;
                    let rel_y_in_section = ((by - (chunk.section.min_y >> 2)) & 3) as usize;
                    if let Some(current_biome_id) =
                        chunk
                            .section
                            .get_noise_biome(section_idx, rel_x, rel_y_in_section, rel_z)
                        && current_biome_id != replace.id
                    {
                        continue;
                    }
                }

                let rel_y = (by - (chunk.section.min_y >> 2)) as usize;
                chunk
                    .section
                    .set_relative_biome(rel_x, rel_y, rel_z, biome.id);
                changed += 1;
            }

            chunk
                .dirty
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    changed
}

struct FillBiomeExecutor {
    has_replace: bool,
}

impl CommandExecutor for FillBiomeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        let has_replace = self.has_replace;
        Box::pin(async move {
            let from = BlockPosArgumentConsumer::find_arg(args, ARG_FROM)?;
            let to = BlockPosArgumentConsumer::find_arg(args, ARG_TO)?;

            let Some(Arg::Simple(biome_name)) = args.get(ARG_BIOME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_BIOME.into())));
            };
            let biome = parse_biome(biome_name)?;

            let replace_biome = if has_replace {
                let Some(Arg::Simple(replace_name)) = args.get(ARG_REPLACE) else {
                    return Err(CommandError::InvalidConsumption(Some(ARG_REPLACE.into())));
                };
                Some(parse_biome(replace_name)?)
            } else {
                None
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

            if !world.is_in_build_limit(from) || !world.is_in_build_limit(to) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }

            let min = Vector3::new(
                from.0.x.min(to.0.x),
                from.0.y.min(to.0.y),
                from.0.z.min(to.0.z),
            );
            let max = Vector3::new(
                from.0.x.max(to.0.x),
                from.0.y.max(to.0.y),
                from.0.z.max(to.0.z),
            );

            let total_biomes = (((max.x >> 2) - (min.x >> 2) + 1) as i64)
                * (((max.y >> 2) - (min.y >> 2) + 1) as i64)
                * (((max.z >> 2) - (min.z >> 2) + 1) as i64);

            let max_block_modifications = {
                let level_info = server.level_info.load();
                level_info.game_rules.max_block_modifications
            };

            if total_biomes > max_block_modifications {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_FILLBIOME_TOOBIG,
                    [
                        TextComponent::text(max_block_modifications.to_string()),
                        TextComponent::text(total_biomes.to_string()),
                    ],
                )));
            }

            let changed = fill_biome_region(&world, min, max, biome, replace_biome).await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_FILLBIOME_SUCCESS,
                    [TextComponent::text(changed.to_string())],
                ))
                .await;

            Ok(changed)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_FROM, BlockPosArgumentConsumer).then(
            argument(ARG_TO, BlockPosArgumentConsumer).then(
                argument(ARG_BIOME, SimpleArgConsumer)
                    .then(
                        literal("replace").then(
                            argument(ARG_REPLACE, SimpleArgConsumer)
                                .execute(FillBiomeExecutor { has_replace: true }),
                        ),
                    )
                    .execute(FillBiomeExecutor { has_replace: false }),
            ),
        ),
    )
}

use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::position_2d::Position2DArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["spreadplayers"];

const DESCRIPTION: &str = "Teleports entities to random surface locations in an area.";

const ARG_CENTER: &str = "center";
const ARG_SPREAD_DISTANCE: &str = "spreadDistance";
const ARG_MAX_RANGE: &str = "maxRange";
const ARG_RESPECT_TEAMS: &str = "respectTeams";
const ARG_TARGETS: &str = "targets";

struct SpreadExecutor;

async fn find_surface_y(world: &crate::world::World, x: i32, z: i32) -> Option<i32> {
    let top_y = world.get_top_y();
    let bottom_y = world.get_bottom_y();
    let mut y = top_y;
    while y >= bottom_y {
        let pos = BlockPos(Vector3::new(x, y, z));
        let state = world.get_block_state(&pos).await;
        if state.is_solid() {
            return Some(y + 1);
        }
        y -= 1;
    }
    None
}

/// Vector2 uses x=worldX, y=worldZ
fn spread_positions(
    center: Vector2<f64>,
    spread_distance: f64,
    max_range: f64,
    count: usize,
) -> Vec<Vector2<f64>> {
    let mut positions = Vec::with_capacity(count);
    let min_x = center.x - max_range;
    let min_z = center.y - max_range;
    let range = max_range * 2.0;

    for _ in 0..count {
        let x = min_x + rand::random::<f64>() * range;
        let z = min_z + rand::random::<f64>() * range;
        positions.push(Vector2::new(x, z));
    }

    // Iterative relaxation to ensure minimum distance
    for _ in 0..10000 {
        let mut any_moved = false;
        for i in 0..count {
            let mut dx = 0.0;
            let mut dz = 0.0;
            let mut too_close = false;

            for j in 0..count {
                if i == j {
                    continue;
                }
                let diff_x = positions[i].x - positions[j].x;
                let diff_z = positions[i].y - positions[j].y;
                let dist_sq = diff_x * diff_x + diff_z * diff_z;
                let min_dist_sq = spread_distance * spread_distance;

                if dist_sq < min_dist_sq {
                    too_close = true;
                    if diff_x.abs() < 1e-10 && diff_z.abs() < 1e-10 {
                        dx += rand::random::<f64>() - 0.5;
                        dz += rand::random::<f64>() - 0.5;
                    } else {
                        let dist = dist_sq.sqrt();
                        dx += diff_x / dist;
                        dz += diff_z / dist;
                    }
                }
            }

            if too_close {
                let len = dx.hypot(dz);
                if len > 0.0 {
                    dx /= len;
                    dz /= len;
                }
                let new_x = (positions[i].x + dx).clamp(min_x, min_x + range);
                let new_z = (positions[i].y + dz).clamp(min_z, min_z + range);
                positions[i] = Vector2::new(new_x, new_z);
                any_moved = true;
            }
        }
        if !any_moved {
            break;
        }
    }

    positions
}

impl CommandExecutor for SpreadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let center = Position2DArgumentConsumer::find_arg(args, ARG_CENTER)?;

            let Some(Arg::Simple(spread_str)) = args.get(ARG_SPREAD_DISTANCE) else {
                return Err(CommandError::InvalidConsumption(Some(
                    ARG_SPREAD_DISTANCE.into(),
                )));
            };
            let spread_distance: f64 = spread_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_SPREAD_DISTANCE.into())))?;

            let Some(Arg::Simple(range_str)) = args.get(ARG_MAX_RANGE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_MAX_RANGE.into())));
            };
            let max_range: f64 = range_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_MAX_RANGE.into())))?;

            let _respect_teams = BoolArgConsumer::find_arg(args, ARG_RESPECT_TEAMS)?;
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_SPREADPLAYERS_FAILED_ENTITIES,
                    [],
                )));
            }

            if spread_distance < 0.0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_SPREADPLAYERS_FAILED_INVALID_HEIGHT,
                    [],
                )));
            }

            if max_range < spread_distance + 1.0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_SPREADPLAYERS_FAILED_ENTITIES,
                    [],
                )));
            }

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

            let positions = spread_positions(center, spread_distance, max_range, targets.len());

            let mut spread_count = 0i32;
            for (i, target) in targets.iter().enumerate() {
                let pos = positions[i];
                let x = pos.x.floor() as i32;
                let z = pos.y.floor() as i32; // Vector2.y = world Z

                let y = match find_surface_y(&world, x, z).await {
                    Some(y) => y as f64,
                    None => continue,
                };

                let dest = Vector3::new(pos.x, y, pos.y); // Vector2.y = world Z

                // Check if entity is a player for request_teleport
                let entity = target.get_entity();
                if let Some(player) = entity.world.load().get_player_by_id(entity.entity_id) {
                    player.request_teleport(dest, 0.0, 0.0).await;
                } else {
                    let target_clone: Arc<dyn EntityBase> = target.clone();
                    target_clone
                        .teleport(dest, Some(0.0), Some(0.0), world.clone())
                        .await;
                }
                spread_count += 1;
            }

            if spread_count == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_SPREADPLAYERS_FAILED_ENTITIES,
                    [],
                )));
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SPREADPLAYERS_SUCCESS_ENTITIES,
                    [
                        TextComponent::text(spread_count.to_string()),
                        TextComponent::text(format!("{:.1}", max_range * 2.0)),
                    ],
                ))
                .await;
            Ok(spread_count)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_CENTER, Position2DArgumentConsumer).then(
            argument(ARG_SPREAD_DISTANCE, SimpleArgConsumer).then(
                argument(ARG_MAX_RANGE, SimpleArgConsumer).then(
                    argument(ARG_RESPECT_TEAMS, BoolArgConsumer).then(
                        argument(ARG_TARGETS, EntitiesArgumentConsumer).execute(SpreadExecutor),
                    ),
                ),
            ),
        ),
    )
}

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_world::chunk::ChunkHeightmapType;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::coordinates::column_pos::ColumnPosArgumentType;
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::World;

const DESCRIPTION: &str = "Teleports entities to random surface locations within an area.";

const PERMISSION: &str = "minecraft:command.spreadplayers";

const ARG_CENTER: &str = "center";
const ARG_SPREAD_DISTANCE: &str = "spreadDistance";
const ARG_MAX_RANGE: &str = "maxRange";
const ARG_RESPECT_TEAMS: &str = "respectTeams";
const ARG_TARGETS: &str = "targets";

/// The maximum number of relaxation/retry iterations, matching Vanilla.
const MAX_ITERATIONS: usize = 10000;

const FAILED_ENTITIES_ERROR_TYPE: CommandErrorType<4> = CommandErrorType::new(
    translation::java::COMMANDS_SPREADPLAYERS_FAILED_ENTITIES,
    translation::bedrock::COMMANDS_SPREADPLAYERS_FAILURE_PLAYERS,
);

const FAILED_TEAMS_ERROR_TYPE: CommandErrorType<4> = CommandErrorType::new(
    translation::java::COMMANDS_SPREADPLAYERS_FAILED_TEAMS,
    translation::bedrock::COMMANDS_SPREADPLAYERS_FAILURE_TEAMS,
);

/// A candidate spread location, equivalent to Vanilla's
/// `SpreadPlayersCommand.Position`.
#[derive(Clone, Copy, Default)]
struct Pile {
    x: f64,
    z: f64,
}

impl Pile {
    fn distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        dx.hypot(dz)
    }

    fn randomize(&mut self, min_x: f64, min_z: f64, max_x: f64, max_z: f64) {
        self.x = rand::random_range(min_x..=max_x);
        self.z = rand::random_range(min_z..=max_z);
    }

    /// Clamps this pile into the spread area. Returns whether it moved.
    fn clamp(&mut self, min_x: f64, min_z: f64, max_x: f64, max_z: f64) -> bool {
        let mut moved = false;
        if self.x < min_x {
            self.x = min_x;
            moved = true;
        } else if self.x > max_x {
            self.x = max_x;
            moved = true;
        }
        if self.z < min_z {
            self.z = min_z;
            moved = true;
        } else if self.z > max_z {
            self.z = max_z;
            moved = true;
        }
        moved
    }

    /// Returns the y coordinate an entity should stand at for this pile, or
    /// `None` if the location is unsafe (on top of a liquid).
    async fn surface_y(&self, world: &World) -> Option<i32> {
        let block_x = self.x.floor() as i32;
        let block_z = self.z.floor() as i32;
        let top = world
            .get_heightmap_height_async(ChunkHeightmapType::WorldSurface, block_x, block_z)
            .await;

        let ground = pumpkin_util::math::position::BlockPos(Vector3::new(block_x, top, block_z));
        let state = world.get_block_state_async(&ground).await;
        if state.is_liquid() {
            return None;
        }

        Some(top + 1)
    }
}

/// Spreads `piles` apart until every pair is at least `spread_distance` apart
/// and all piles are inside the spread area, mirroring Vanilla's algorithm.
/// Returns whether the piles converged within [`MAX_ITERATIONS`].
fn spread_piles(
    piles: &mut [Pile],
    spread_distance: f64,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
) -> bool {
    for _ in 0..MAX_ITERATIONS {
        let mut moved = false;
        for i in 0..piles.len() {
            let mut push = Pile::default();
            let mut collisions = 0;

            for j in 0..piles.len() {
                if i == j {
                    continue;
                }
                let distance = piles[i].distance(&piles[j]);
                if distance < spread_distance {
                    collisions += 1;
                    push.x += piles[j].x - piles[i].x;
                    push.z += piles[j].z - piles[i].z;
                }
            }

            if collisions > 0 {
                push.x /= f64::from(collisions);
                push.z /= f64::from(collisions);
                let length = push.x.hypot(push.z);
                if length > 0.0 {
                    piles[i].x -= push.x / length;
                    piles[i].z -= push.z / length;
                } else {
                    piles[i].randomize(min_x, min_z, max_x, max_z);
                }
                moved = true;
            }

            if piles[i].clamp(min_x, min_z, max_x, max_z) {
                moved = true;
            }
        }
        if !moved {
            return true;
        }
    }
    false
}

struct SpreadArea {
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
}

/// Randomizes, relaxes, and grounds `pile_count` piles inside `area`.
/// Returns the piles and their standing y coordinates, or `None` if no safe
/// arrangement was found.
async fn find_spread_positions(
    world: &World,
    pile_count: usize,
    spread_distance: f64,
    area: &SpreadArea,
) -> Option<(Vec<Pile>, Vec<i32>)> {
    let mut piles = vec![Pile::default(); pile_count];
    for pile in &mut piles {
        pile.randomize(area.min_x, area.min_z, area.max_x, area.max_z);
    }

    let mut surface_ys = vec![0; pile_count];
    // A handful of surface retries is plenty; each retry re-runs the
    // full relaxation, so this loop must stay small.
    for _ in 0..16 {
        if !spread_piles(
            &mut piles,
            spread_distance,
            area.min_x,
            area.min_z,
            area.max_x,
            area.max_z,
        ) {
            return None;
        }

        let mut all_safe = true;
        for (pile, y) in piles.iter_mut().zip(&mut surface_ys) {
            if let Some(surface) = pile.surface_y(world).await {
                *y = surface;
            } else {
                pile.randomize(area.min_x, area.min_z, area.max_x, area.max_z);
                all_safe = false;
            }
        }
        if all_safe {
            return Some((piles, surface_ys));
        }
    }
    None
}

/// Returns the average distance from each pile to its closest neighbor.
fn average_min_distance(piles: &[Pile]) -> f64 {
    if piles.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for pile in piles {
        let mut closest = f64::MAX;
        for other in piles {
            if !std::ptr::eq(pile, other) {
                closest = closest.min(pile.distance(other));
            }
        }
        total += closest;
    }
    #[expect(clippy::cast_precision_loss)]
    let count = piles.len() as f64;
    total / count
}

struct SpreadPlayersExecutor;

impl CommandExecutor for SpreadPlayersExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let center = ColumnPosArgumentType::get_column_pos(context, ARG_CENTER)?;
            let spread_distance = f64::from(FloatArgumentType::get(context, ARG_SPREAD_DISTANCE)?);
            let max_range = f64::from(FloatArgumentType::get(context, ARG_MAX_RANGE)?);
            let respect_teams = BoolArgumentType::get(context, ARG_RESPECT_TEAMS)?;
            let targets = EntityArgumentType::get_entities(context, ARG_TARGETS).await?;

            // Vanilla center-corrects the vec2 argument to the middle of the block.
            let center_x = f64::from(center.0.x) + 0.5;
            let center_z = f64::from(center.0.y) + 0.5;
            let area = SpreadArea {
                min_x: center_x - max_range,
                min_z: center_z - max_range,
                max_x: center_x + max_range,
                max_z: center_z + max_range,
            };

            // Teams are not implemented yet, so every target is teamless. With
            // respectTeams=true that matches Vanilla's behavior of gathering all
            // teamless entities onto a single position.
            let pile_count = if respect_teams { 1 } else { targets.len() };

            let world = context.source.world().clone();

            let Some((piles, surface_ys)) =
                find_spread_positions(&world, pile_count, spread_distance, &area).await
            else {
                let error_type = if respect_teams {
                    &FAILED_TEAMS_ERROR_TYPE
                } else {
                    &FAILED_ENTITIES_ERROR_TYPE
                };
                #[expect(clippy::cast_precision_loss)]
                let suggested = max_range / (pile_count as f64).sqrt();
                return Err(error_type.create_without_context(
                    TextComponent::text(pile_count.to_string()),
                    TextComponent::text(format!("{center_x:.2}")),
                    TextComponent::text(format!("{center_z:.2}")),
                    TextComponent::text(format!("{suggested:.2}")),
                ));
            };

            for (index, target) in targets.iter().enumerate() {
                let pile = piles[index % pile_count];
                let y = surface_ys[index % pile_count];
                target
                    .clone()
                    .teleport(
                        Vector3::new(pile.x.floor() + 0.5, f64::from(y), pile.z.floor() + 0.5),
                        None,
                        None,
                        world.clone(),
                    )
                    .await;
            }

            let average_distance = average_min_distance(&piles);

            let success_key = if respect_teams {
                (
                    translation::java::COMMANDS_SPREADPLAYERS_SUCCESS_TEAMS,
                    translation::java::COMMANDS_SPREADPLAYERS_SUCCESS_TEAMS,
                )
            } else {
                (
                    translation::java::COMMANDS_SPREADPLAYERS_SUCCESS_ENTITIES,
                    translation::java::COMMANDS_SPREADPLAYERS_SUCCESS_ENTITIES,
                )
            };

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        success_key.0,
                        success_key.1,
                        [
                            TextComponent::text(pile_count.to_string()),
                            TextComponent::text(format!("{center_x:.2}")),
                            TextComponent::text(format!("{center_z:.2}")),
                            TextComponent::text(format!("{average_distance:.2}")),
                        ],
                    ),
                    true,
                )
                .await;

            #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            Ok(pile_count as i32)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("spreadplayers", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                argument(ARG_CENTER, ColumnPosArgumentType).then(
                    argument(ARG_SPREAD_DISTANCE, FloatArgumentType::with_min(0.0)).then(
                        argument(ARG_MAX_RANGE, FloatArgumentType::with_min(1.0)).then(
                            argument(ARG_RESPECT_TEAMS, BoolArgumentType).then(
                                argument(ARG_TARGETS, EntityArgumentType::Entities)
                                    .executes(SpreadPlayersExecutor),
                            ),
                        ),
                    ),
                ),
            ),
    );
}

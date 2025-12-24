use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::command::CommandError;
use crate::command::CommandResult;
use crate::command::args::ConsumedArgs;
use crate::command::args::FindArg;
use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::args::position_3d::Position3DArgumentConsumer;
use crate::command::args::rotation::RotationArgumentConsumer;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandSender};
use crate::entity::EntityBase;
use crate::world::World;

const NAMES: [&str; 2] = ["teleport", "tp"];
const DESCRIPTION: &str = "Teleports entities, including players."; // todo

/// position
const ARG_LOCATION: &str = "location";

/// single entity
const ARG_DESTINATION: &str = "destination";

/// multiple entities
const ARG_TARGETS: &str = "targets";

/// rotation: yaw/pitch
const ARG_ROTATION: &str = "rotation";

/// single entity
const ARG_FACING_ENTITY: &str = "facingEntity";

/// position
const ARG_FACING_LOCATION: &str = "facingLocation";

fn yaw_pitch_facing_position(
    looking_from: &Vector3<f64>,
    looking_towards: &Vector3<f64>,
) -> (f32, f32) {
    let direction_vector = (looking_towards.sub(looking_from)).normalize();

    let yaw_radians = -direction_vector.x.atan2(direction_vector.z);
    let pitch_radians = (-direction_vector.y).asin();

    let yaw_degrees = yaw_radians.to_degrees();
    let pitch_degrees = pitch_radians.to_degrees();

    (yaw_degrees as f32, pitch_degrees as f32)
}

struct EntitiesToEntityExecutor;

impl CommandExecutor for EntitiesToEntityExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let destination = EntityArgumentConsumer::find_arg(args, ARG_DESTINATION)?;
            let pos = destination.get_entity().pos.load();
            if !World::is_valid(pos) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }
            for target in targets {
                let base_entity = target.get_entity();
                let yaw = base_entity.yaw.load();
                let pitch = base_entity.pitch.load();
                let world = base_entity.world.clone();
                target
                    .clone()
                    .teleport(pos, yaw.into(), pitch.into(), world)
                    .await;
            }

            Ok(())
        })
    }
}

struct EntitiesToPosFacingPosExecutor;

impl CommandExecutor for EntitiesToPosFacingPosExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;
            if !World::is_valid(pos) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }
            let facing_pos = Position3DArgumentConsumer::find_arg(args, ARG_FACING_LOCATION)?;
            let (yaw, pitch) = yaw_pitch_facing_position(&pos, &facing_pos);
            //todo
            let world = match sender {
                CommandSender::Rcon(_) | CommandSender::Console => {
                    server.worlds.read().await.first().unwrap().clone()
                }
                CommandSender::Player(player) => player.world().clone(),
            };

            for target in targets {
                target
                    .clone()
                    .teleport(pos, Some(yaw), Some(pitch), world.clone())
                    .await;
            }

            Ok(())
        })
    }
}

struct EntitiesToPosFacingEntityExecutor;

impl CommandExecutor for EntitiesToPosFacingEntityExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;
            if !World::is_valid(pos) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }
            let facing_entity = EntityArgumentConsumer::find_arg(args, ARG_FACING_ENTITY)?;
            let (yaw, pitch) =
                yaw_pitch_facing_position(&pos, &facing_entity.get_entity().pos.load());

            for target in targets {
                target
                    .clone()
                    .teleport(
                        pos,
                        Some(yaw),
                        Some(pitch),
                        facing_entity.get_entity().world.clone(),
                    )
                    .await;
            }

            Ok(())
        })
    }
}

struct EntitiesToPosWithRotationExecutor;

impl CommandExecutor for EntitiesToPosWithRotationExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;
            if !World::is_valid(pos) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }
            let (yaw, pitch) = RotationArgumentConsumer::find_arg(args, ARG_ROTATION)?;

            // todo command context
            let world = server.worlds.read().await.first().unwrap().clone();
            for target in targets {
                target
                    .clone()
                    .teleport(pos, Some(yaw), Some(pitch), world.clone())
                    .await;
            }

            Ok(())
        })
    }
}

struct EntitiesToPosExecutor;

impl CommandExecutor for EntitiesToPosExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;
            if !World::is_valid(pos) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }
            // todo command context
            let world = match sender {
                CommandSender::Rcon(_) | CommandSender::Console => {
                    server.worlds.read().await.first().unwrap().clone()
                }
                CommandSender::Player(player) => player.world().clone(),
            };
            for target in targets {
                let yaw = target.get_entity().yaw.load();
                let pitch = target.get_entity().pitch.load();
                target
                    .clone()
                    .teleport(pos, Some(yaw), Some(pitch), world.clone())
                    .await;
            }

            Ok(())
        })
    }
}

struct SelfToEntityExecutor;

impl CommandExecutor for SelfToEntityExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let destination = EntityArgumentConsumer::find_arg(args, ARG_DESTINATION)?;
            let pos = destination.get_entity().pos.load();
            let world = destination.get_entity().world.clone();

            match sender {
                CommandSender::Player(player) => {
                    let yaw = player.living_entity.entity.yaw.load();
                    let pitch = player.living_entity.entity.pitch.load();
                    if !World::is_valid(pos) {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "argument.pos.outofbounds",
                            [],
                        )));
                    }
                    player
                        .clone()
                        .teleport(pos, Some(yaw), Some(pitch), world)
                        .await;
                }
                _ => {
                    sender
                        .send_message(TextComponent::translate("permissions.requires.player", []))
                        .await;
                }
            }

            Ok(())
        })
    }
}
struct SelfToPosExecutor;

impl CommandExecutor for SelfToPosExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            match sender {
                CommandSender::Player(player) => {
                    let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;
                    let yaw = player.living_entity.entity.yaw.load();
                    let pitch = player.living_entity.entity.pitch.load();
                    if !World::is_valid(pos) {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "argument.pos.outofbounds",
                            [],
                        )));
                    }
                    player
                        .clone()
                        .teleport(pos, Some(yaw), Some(pitch), player.world().clone())
                        .await;
                }
                _ => {
                    sender
                        .send_message(TextComponent::translate("permissions.requires.player", []))
                        .await;
                }
            }

            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_LOCATION, Position3DArgumentConsumer).execute(SelfToPosExecutor))
        .then(argument(ARG_DESTINATION, EntityArgumentConsumer).execute(SelfToEntityExecutor))
        .then(
            argument(ARG_TARGETS, EntitiesArgumentConsumer)
                .then(
                    argument(ARG_LOCATION, Position3DArgumentConsumer)
                        .execute(EntitiesToPosExecutor)
                        .then(
                            argument(ARG_ROTATION, RotationArgumentConsumer)
                                .execute(EntitiesToPosWithRotationExecutor),
                        )
                        .then(
                            literal("facing")
                                .then(
                                    literal("entity").then(
                                        argument(ARG_FACING_ENTITY, EntityArgumentConsumer)
                                            .execute(EntitiesToPosFacingEntityExecutor),
                                    ),
                                )
                                .then(
                                    argument(ARG_FACING_LOCATION, Position3DArgumentConsumer)
                                        .execute(EntitiesToPosFacingPosExecutor),
                                ),
                        ),
                )
                .then(
                    argument(ARG_DESTINATION, EntityArgumentConsumer)
                        .execute(EntitiesToEntityExecutor),
                ),
        )
}

use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::rotation::RotationArgumentType;
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::world::World;

const DESCRIPTION: &str = "Teleports entities, including players.";
const PERMISSION: &str = "minecraft:command.teleport";

const ERROR_NOT_PLAYER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
);

const ERROR_INVALID_POSITION: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TELEPORT_INVALIDPOSITION,
    translation::java::COMMANDS_TELEPORT_INVALIDPOSITION,
);

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

fn success_key_and_arg(
    targets: &[Arc<dyn EntityBase>],
    single_key: &'static str,
    multiple_key: &'static str,
) -> (&'static str, TextComponent) {
    if targets.len() == 1 {
        (single_key, targets[0].get_display_name())
    } else {
        (multiple_key, TextComponent::text(targets.len().to_string()))
    }
}

struct SelfToPosExecutor;

impl CommandExecutor for SelfToPosExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context
            .source
            .output
            .as_player()
            .ok_or_else(|| ERROR_NOT_PLAYER.create_without_context())?;

        let pos = Vec3ArgumentType::get_coordinates(context, "location")?.resolve(&context.source);
        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        let yaw = player.get_entity().yaw.load();
        let pitch = player.get_entity().pitch.load();
        let player_world = player.world();
        player.teleport(pos, Some(yaw), Some(pitch), player_world);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_SINGLE,
                translation::bedrock::COMMANDS_TP_SUCCESS_COORDINATES,
                [
                    player.get_display_name(),
                    TextComponent::text(pos.x.to_string()),
                    TextComponent::text(pos.y.to_string()),
                    TextComponent::text(pos.z.to_string()),
                ],
            ),
            true,
        );

        Ok(1)
    }
}

struct SelfToEntityExecutor;

impl CommandExecutor for SelfToEntityExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context
            .source
            .output
            .as_player()
            .ok_or_else(|| ERROR_NOT_PLAYER.create_without_context())?;

        let destination = EntityArgumentType::get_entity(context, "destination")?;
        let destination_entity = destination.get_entity();
        let pos = destination_entity.pos.load();
        let yaw = destination_entity.yaw.load();
        let pitch = destination_entity.pitch.load();
        let world = destination_entity.world.load_full();

        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        player.teleport(pos, Some(yaw), Some(pitch), world);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_TELEPORT_SUCCESS_ENTITY_SINGLE,
                translation::bedrock::COMMANDS_TP_SUCCESSVICTIM,
                [
                    player.get_display_name(),
                    destination_entity.get_display_name(),
                ],
            ),
            true,
        );

        Ok(1)
    }
}

struct EntitiesToEntityExecutor;

impl CommandExecutor for EntitiesToEntityExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let destination = EntityArgumentType::get_entity(context, "destination")?;
        let destination_entity = destination.get_entity();
        let pos = destination_entity.pos.load();
        let yaw = destination_entity.yaw.load();
        let pitch = destination_entity.pitch.load();
        let world = destination_entity.world.load_full();

        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        for target in &targets {
            target.teleport(pos, Some(yaw), Some(pitch), world.clone());
        }

        let (key, target_arg) = success_key_and_arg(
            &targets,
            translation::java::COMMANDS_TELEPORT_SUCCESS_ENTITY_SINGLE,
            translation::java::COMMANDS_TELEPORT_SUCCESS_ENTITY_MULTIPLE,
        );
        context.source.send_feedback(
            TextComponent::translate_cross(
                key,
                translation::bedrock::COMMANDS_TP_SUCCESSVICTIM,
                [target_arg, destination_entity.get_display_name()],
            ),
            true,
        );

        Ok(targets.len() as i32)
    }
}

struct EntitiesToPosExecutor;

impl CommandExecutor for EntitiesToPosExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let pos = Vec3ArgumentType::get_coordinates(context, "location")?.resolve(&context.source);

        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        let world = context.source.world();
        for target in &targets {
            let yaw = target.get_entity().yaw.load();
            let pitch = target.get_entity().pitch.load();
            target.teleport(pos, Some(yaw), Some(pitch), world.clone());
        }

        let (key, target_arg) = success_key_and_arg(
            &targets,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_SINGLE,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_MULTIPLE,
        );
        context.source.send_feedback(
            TextComponent::translate_cross(
                key,
                translation::bedrock::COMMANDS_TP_SUCCESS_COORDINATES,
                [
                    target_arg,
                    TextComponent::text(pos.x.to_string()),
                    TextComponent::text(pos.y.to_string()),
                    TextComponent::text(pos.z.to_string()),
                ],
            ),
            true,
        );

        Ok(targets.len() as i32)
    }
}

struct EntitiesToPosWithRotationExecutor;

impl CommandExecutor for EntitiesToPosWithRotationExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let pos = Vec3ArgumentType::get_coordinates(context, "location")?.resolve(&context.source);

        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        let rot = RotationArgumentType::get(context, "rotation")?.rotation(&context.source);
        let yaw = rot.y;
        let pitch = rot.x;
        let world = context.source.world();

        for target in &targets {
            target.teleport(pos, Some(yaw), Some(pitch), world.clone());
        }

        let (key, target_arg) = success_key_and_arg(
            &targets,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_SINGLE,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_MULTIPLE,
        );
        context.source.send_feedback(
            TextComponent::translate_cross(
                key,
                translation::bedrock::COMMANDS_TP_SUCCESS_COORDINATES,
                [
                    target_arg,
                    TextComponent::text(pos.x.to_string()),
                    TextComponent::text(pos.y.to_string()),
                    TextComponent::text(pos.z.to_string()),
                ],
            ),
            true,
        );

        Ok(targets.len() as i32)
    }
}

struct EntitiesToPosFacingPosExecutor;

impl CommandExecutor for EntitiesToPosFacingPosExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let pos = Vec3ArgumentType::get_coordinates(context, "location")?.resolve(&context.source);

        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        let facing_pos =
            Vec3ArgumentType::get_coordinates(context, "facingLocation")?.resolve(&context.source);
        let (yaw, pitch) = yaw_pitch_facing_position(&pos, &facing_pos);
        let world = context.source.world();

        for target in &targets {
            target.teleport(pos, Some(yaw), Some(pitch), world.clone());
        }

        let (key, target_arg) = success_key_and_arg(
            &targets,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_SINGLE,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_MULTIPLE,
        );
        context.source.send_feedback(
            TextComponent::translate_cross(
                key,
                translation::bedrock::COMMANDS_TP_SUCCESS_COORDINATES,
                [
                    target_arg,
                    TextComponent::text(pos.x.to_string()),
                    TextComponent::text(pos.y.to_string()),
                    TextComponent::text(pos.z.to_string()),
                ],
            ),
            true,
        );

        Ok(targets.len() as i32)
    }
}

struct EntitiesToPosFacingEntityExecutor;

impl CommandExecutor for EntitiesToPosFacingEntityExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let pos = Vec3ArgumentType::get_coordinates(context, "location")?.resolve(&context.source);

        if !World::is_valid(BlockPos(pos.floor_to_i32())) {
            return Err(ERROR_INVALID_POSITION.create_without_context());
        }

        let facing_entity = EntityArgumentType::get_entity(context, "facingEntity")?;
        let (yaw, pitch) = yaw_pitch_facing_position(&pos, &facing_entity.get_entity().pos.load());
        let world = context.source.world();

        for target in &targets {
            target.teleport(pos, Some(yaw), Some(pitch), world.clone());
        }

        let (key, target_arg) = success_key_and_arg(
            &targets,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_SINGLE,
            translation::java::COMMANDS_TELEPORT_SUCCESS_LOCATION_MULTIPLE,
        );
        context.source.send_feedback(
            TextComponent::translate_cross(
                key,
                translation::bedrock::COMMANDS_TP_SUCCESS_COORDINATES,
                [
                    target_arg,
                    TextComponent::text(pos.x.to_string()),
                    TextComponent::text(pos.y.to_string()),
                    TextComponent::text(pos.z.to_string()),
                ],
            ),
            true,
        );

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let cmd = command("teleport", DESCRIPTION)
        .requires(PERMISSION)
        .then(argument("location", Vec3ArgumentType::Default).executes(SelfToPosExecutor))
        .then(argument("destination", EntityArgumentType::Entity).executes(SelfToEntityExecutor))
        .then(
            argument("targets", EntityArgumentType::Entities)
                .then(
                    argument("location", Vec3ArgumentType::Default)
                        .executes(EntitiesToPosExecutor)
                        .then(
                            argument("rotation", RotationArgumentType)
                                .executes(EntitiesToPosWithRotationExecutor),
                        )
                        .then(
                            literal("facing")
                                .then(
                                    literal("entity").then(
                                        argument("facingEntity", EntityArgumentType::Entity)
                                            .executes(EntitiesToPosFacingEntityExecutor),
                                    ),
                                )
                                .then(
                                    argument("facingLocation", Vec3ArgumentType::Default)
                                        .executes(EntitiesToPosFacingPosExecutor),
                                ),
                        ),
                )
                .then(
                    argument("destination", EntityArgumentType::Entity)
                        .executes(EntitiesToEntityExecutor),
                ),
        );

    dispatcher.register_with_aliases(cmd, &["tp"]);
}

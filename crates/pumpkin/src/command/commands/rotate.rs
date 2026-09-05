use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::rotation::RotationArgumentType;
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::entity_anchor::{EntityAnchor, EntityAnchorArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Changes the rotation of an entity.";
const PERMISSION: &str = "minecraft:command.rotate";

fn yaw_pitch_facing_position(
    looking_from: &Vector3<f64>,
    looking_towards: &Vector3<f64>,
) -> (f32, f32) {
    let direction_vector = looking_towards.sub(looking_from).normalize();

    let yaw_radians = -direction_vector.x.atan2(direction_vector.z);
    let pitch_radians = (-direction_vector.y).asin();

    let yaw_degrees = yaw_radians.to_degrees();
    let pitch_degrees = pitch_radians.to_degrees();

    (yaw_degrees as f32, pitch_degrees as f32)
}

fn rotate_entity(target: &Arc<dyn EntityBase>, yaw: f32, pitch: f32) {
    let entity = target.get_entity();
    let pos = entity.pos.load();
    let world = entity.world.load_full();
    let clamped_pitch = pitch.clamp(-90.0, 90.0);
    target.teleport(pos, Some(yaw), Some(clamped_pitch), world);
}

fn send_success_message(context: &CommandContext, target: &dyn EntityBase) {
    let target_name = target.get_display_name();
    context.source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_ROTATE_SUCCESS,
            translation::java::COMMANDS_ROTATE_SUCCESS,
            [target_name],
        ),
        true,
    );
}

struct RotateToRotationExecutor;

impl CommandExecutor for RotateToRotationExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, "target")?;
        let rot = RotationArgumentType::get(context, "rotation")?.rotation(&context.source);
        let pitch = rot.x;
        let yaw = rot.y;

        rotate_entity(&target, yaw, pitch);
        send_success_message(context, target.as_ref());

        Ok(1)
    }
}

struct RotateFacingLocationExecutor;

impl CommandExecutor for RotateFacingLocationExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, "target")?;
        let facing_pos =
            Vec3ArgumentType::get_coordinates(context, "facingLocation")?.resolve(&context.source);

        let looking_from = context
            .source
            .entity_anchor
            .position_at_entity(target.get_entity());

        let (yaw, pitch) = yaw_pitch_facing_position(&looking_from, &facing_pos);

        rotate_entity(&target, yaw, pitch);
        send_success_message(context, target.as_ref());

        Ok(1)
    }
}

struct RotateFacingEntityExecutor {
    has_anchor: bool,
}

impl CommandExecutor for RotateFacingEntityExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, "target")?;
        let facing_entity = EntityArgumentType::get_entity(context, "facingEntity")?;
        let anchor = if self.has_anchor {
            EntityAnchorArgumentType::get(context, "facingAnchor")?
        } else {
            EntityAnchor::Feet
        };

        let looking_from = context
            .source
            .entity_anchor
            .position_at_entity(target.get_entity());

        let looking_towards = anchor.position_at_entity(facing_entity.get_entity());

        let (yaw, pitch) = yaw_pitch_facing_position(&looking_from, &looking_towards);

        rotate_entity(&target, yaw, pitch);
        send_success_message(context, target.as_ref());

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("rotate", DESCRIPTION).requires(PERMISSION).then(
            argument("target", EntityArgumentType::Entity)
                .then(argument("rotation", RotationArgumentType).executes(RotateToRotationExecutor))
                .then(
                    literal("facing")
                        .then(
                            literal("entity").then(
                                argument("facingEntity", EntityArgumentType::Entity)
                                    .executes(RotateFacingEntityExecutor { has_anchor: false })
                                    .then(
                                        argument("facingAnchor", EntityAnchorArgumentType)
                                            .executes(RotateFacingEntityExecutor {
                                                has_anchor: true,
                                            }),
                                    ),
                            ),
                        )
                        .then(
                            argument("facingLocation", Vec3ArgumentType::Default)
                                .executes(RotateFacingLocationExecutor),
                        ),
                ),
        ),
    );
}

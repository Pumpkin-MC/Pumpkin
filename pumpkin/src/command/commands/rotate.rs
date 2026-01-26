use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::args::entity_anchor::{EntityAnchor, EntityAnchorArgumentConsumer};
use crate::command::args::position_3d::Position3DArgumentConsumer;
use crate::command::args::rotation::RotationArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::builder::{argument, literal};
use crate::command::tree::CommandTree;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["rotate"];
const DESCRIPTION: &str = "Changes the rotation of an entity.";

const ARG_TARGET: &str = "target";
const ARG_ROTATION: &str = "rotation";
const ARG_FACING_LOCATION: &str = "facingLocation";
const ARG_FACING_ENTITY: &str = "facingEntity";
const ARG_FACING_ANCHOR: &str = "facingAnchor";

/// Calculates yaw and pitch to look from one position towards another.
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

/// Gets the position to look at for an entity, considering the anchor point.
/// Matches vanilla's `EntityAnchorArgumentType.EntityAnchor.positionAt()`.
fn get_anchor_position(entity: &crate::entity::Entity, anchor: EntityAnchor) -> Vector3<f64> {
    let pos = entity.pos.load();
    match anchor {
        EntityAnchor::Eyes => {
            let eye_height = f64::from(entity.entity_type.eye_height);
            Vector3::new(pos.x, pos.y + eye_height, pos.z)
        }
        EntityAnchor::Feet => pos,
    }
}

/// Rotates an entity using vanilla-style rotation logic.
/// If relative flags are true, the values are added to current rotation.
/// If relative flags are false, the values are absolute.
async fn rotate_entity(
    target: std::sync::Arc<dyn crate::entity::EntityBase>,
    yaw: f32,
    is_yaw_relative: bool,
    pitch: f32,
    is_pitch_relative: bool,
) {
    let entity = target.get_entity();

    // Calculate final rotation values like vanilla's Entity.rotate()
    // If relative, add to current rotation; if absolute, use directly
    let final_yaw = if is_yaw_relative {
        entity.yaw.load() + yaw
    } else {
        yaw
    };

    let final_pitch = if is_pitch_relative {
        // Clamp pitch to valid range [-90, 90]
        (entity.pitch.load() + pitch).clamp(-90.0, 90.0)
    } else {
        pitch.clamp(-90.0, 90.0)
    };

    // Use teleport with same position to update rotation
    // This properly handles both players (sends CPlayerPosition) and other entities
    let pos = entity.pos.load();
    let world = entity.world.clone();
    target
        .teleport(pos, Some(final_yaw), Some(final_pitch), world)
        .await;
}

/// Sends success message for the rotate command.
async fn send_success_message(sender: &CommandSender, target: &dyn crate::entity::EntityBase) {
    let target_name = target.get_display_name().await;
    sender
        .send_message(TextComponent::translate(
            "commands.rotate.success",
            [target_name],
        ))
        .await;
}

// /rotate <target> <rotation>
struct RotateToRotationExecutor;

impl CommandExecutor for RotateToRotationExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
            let (yaw, yaw_rel, pitch, pitch_rel) =
                RotationArgumentConsumer::find_arg(args, ARG_ROTATION)?;

            rotate_entity(target.clone(), yaw, yaw_rel, pitch, pitch_rel).await;
            send_success_message(sender, target.as_ref()).await;

            Ok(())
        })
    }
}

// /rotate <target> facing <x> <y> <z>
struct RotateFacingLocationExecutor;

impl CommandExecutor for RotateFacingLocationExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
            let facing_pos = Position3DArgumentConsumer::find_arg(args, ARG_FACING_LOCATION)?;

            let entity = target.get_entity();
            let eye_height = f64::from(entity.entity_type.eye_height);
            let pos = entity.pos.load();
            let looking_from = Vector3::new(pos.x, pos.y + eye_height, pos.z);

            let (yaw, pitch) = yaw_pitch_facing_position(&looking_from, &facing_pos);

            // Facing uses absolute rotation
            rotate_entity(target.clone(), yaw, false, pitch, false).await;
            send_success_message(sender, target.as_ref()).await;

            Ok(())
        })
    }
}

// /rotate <target> facing entity <entity> [eyes|feet]
struct RotateFacingEntityExecutor;

impl CommandExecutor for RotateFacingEntityExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
            let facing_entity = EntityArgumentConsumer::find_arg(args, ARG_FACING_ENTITY)?;
            let anchor = EntityAnchorArgumentConsumer::find_arg(args, ARG_FACING_ANCHOR)?;

            let target_entity = target.get_entity();
            let eye_height = f64::from(target_entity.entity_type.eye_height);
            let pos = target_entity.pos.load();
            let looking_from = Vector3::new(pos.x, pos.y + eye_height, pos.z);

            let looking_towards = get_anchor_position(facing_entity.get_entity(), anchor);

            let (yaw, pitch) = yaw_pitch_facing_position(&looking_from, &looking_towards);

            // Facing uses absolute rotation
            rotate_entity(target.clone(), yaw, false, pitch, false).await;
            send_success_message(sender, target.as_ref()).await;

            Ok(())
        })
    }
}

// /rotate <target> facing entity <entity> (no anchor - defaults to feet)
struct RotateFacingEntityNoAnchorExecutor;

impl CommandExecutor for RotateFacingEntityNoAnchorExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
            let facing_entity = EntityArgumentConsumer::find_arg(args, ARG_FACING_ENTITY)?;

            let target_entity = target.get_entity();
            let eye_height = f64::from(target_entity.entity_type.eye_height);
            let pos = target_entity.pos.load();
            let looking_from = Vector3::new(pos.x, pos.y + eye_height, pos.z);

            // Default to feet (vanilla behavior)
            let looking_towards =
                get_anchor_position(facing_entity.get_entity(), EntityAnchor::Feet);

            let (yaw, pitch) = yaw_pitch_facing_position(&looking_from, &looking_towards);

            // Facing uses absolute rotation
            rotate_entity(target.clone(), yaw, false, pitch, false).await;
            send_success_message(sender, target.as_ref()).await;

            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, EntityArgumentConsumer)
            .then(argument(ARG_ROTATION, RotationArgumentConsumer).execute(RotateToRotationExecutor))
            .then(
                literal("facing")
                    .then(
                        literal("entity").then(
                            argument(ARG_FACING_ENTITY, EntityArgumentConsumer)
                                .execute(RotateFacingEntityNoAnchorExecutor)
                                .then(
                                    argument(ARG_FACING_ANCHOR, EntityAnchorArgumentConsumer)
                                        .execute(RotateFacingEntityExecutor),
                                ),
                        ),
                    )
                    .then(
                        argument(ARG_FACING_LOCATION, Position3DArgumentConsumer)
                            .execute(RotateFacingLocationExecutor),
                    ),
            ),
    )
}

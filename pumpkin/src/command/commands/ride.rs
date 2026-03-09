use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::SimpleWorld;

use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["ride"];

const DESCRIPTION: &str = "Used to mount or dismount entities.";

const ARG_VEHICLE: &str = "vehicle";

struct MountExecutor;

impl CommandExecutor for MountExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target: Arc<dyn EntityBase> = EntityArgumentConsumer::find_arg(args, "target")?;
            let vehicle: Arc<dyn EntityBase> = EntityArgumentConsumer::find_arg(args, ARG_VEHICLE)?;

            let target_entity = target.get_entity();
            let vehicle_entity = vehicle.get_entity();

            // Can't ride players
            if vehicle.get_player().is_some() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RIDE_MOUNT_FAILURE_CANT_RIDE_PLAYERS,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            // Already riding something
            if target_entity.vehicle.lock().await.is_some() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RIDE_ALREADY_RIDING,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            // Check for circular loop: vehicle can't already be (transitively) riding target
            {
                let mut current = Some(vehicle.clone());
                while let Some(ref entity) = current {
                    if entity.get_entity().entity_id == target_entity.entity_id {
                        sender
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_RIDE_MOUNT_FAILURE_LOOP,
                                [],
                            ))
                            .await;
                        return Ok(0);
                    }
                    let next = entity.get_entity().vehicle.lock().await.clone();
                    current = next;
                }
            }

            // Different dimensions
            if target_entity.world.load().get_dimension().await
                != vehicle_entity.world.load().get_dimension().await
            {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RIDE_MOUNT_FAILURE_WRONG_DIMENSION,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            vehicle_entity
                .add_passenger(vehicle.clone(), target.clone())
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_RIDE_MOUNT_SUCCESS,
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

struct DismountExecutor;

impl CommandExecutor for DismountExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target: Arc<dyn EntityBase> = EntityArgumentConsumer::find_arg(args, "target")?;

            let target_entity = target.get_entity();
            let vehicle = target_entity.vehicle.lock().await.clone();

            if let Some(vehicle) = vehicle {
                vehicle
                    .get_entity()
                    .remove_passenger(target_entity.entity_id)
                    .await;
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RIDE_DISMOUNT_SUCCESS,
                        [],
                    ))
                    .await;
                Ok(1)
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RIDE_NOT_RIDING,
                        [],
                    ))
                    .await;
                Ok(0)
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(EntityArgumentConsumer)
            .then(
                literal("mount")
                    .then(argument(ARG_VEHICLE, EntityArgumentConsumer).execute(MountExecutor)),
            )
            .then(literal("dismount").execute(DismountExecutor)),
    )
}

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Mounts or dismounts target entities.";
const PERMISSION: &str = "minecraft:command.ride";

static ERROR_CANT_RIDE_PLAYERS: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RIDE_MOUNT_FAILURE_CANT_RIDE_PLAYERS,
    "Cannot ride players",
);
static ERROR_WRONG_DIMENSION: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RIDE_MOUNT_FAILURE_WRONG_DIMENSION,
    "Cannot ride across dimensions",
);
static ERROR_LOOP: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RIDE_MOUNT_FAILURE_LOOP,
    "Cannot ride recursively",
);
static ERROR_ALREADY_RIDING: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_RIDE_ALREADY_RIDING,
    "Already riding",
);
static ERROR_NOT_RIDING: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_RIDE_NOT_RIDING,
    "Not riding anything",
);
static ERROR_GENERIC: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_RIDE_MOUNT_FAILURE_GENERIC,
    "Failed to mount",
);

#[allow(clippy::assigning_clones)]
async fn is_riding_recursive(entity: &dyn EntityBase, possible_vehicle: &dyn EntityBase) -> bool {
    let mut current = possible_vehicle.get_entity().vehicle.lock().await.clone();
    while let Some(vehicle) = current {
        if vehicle.get_entity().entity_id == entity.get_entity().entity_id {
            return true;
        }
        current = vehicle.get_entity().vehicle.lock().await.clone();
    }
    false
}

struct RideMountExecutor;

impl CommandExecutor for RideMountExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, "target").await?;
            let vehicle = EntityArgumentType::get_entity(context, "vehicle").await?;

            if vehicle.get_player().is_some() {
                return Err(ERROR_CANT_RIDE_PLAYERS.create_without_context());
            }

            let vehicle_world = vehicle.get_entity().world.load();

            let mut success_count = 0;
            let mut last_error = None;

            for target in &targets {
                let target_world = target.get_entity().world.load();
                if target_world.dimension.minecraft_name != vehicle_world.dimension.minecraft_name {
                    last_error = Some(ERROR_WRONG_DIMENSION.create_without_context());
                    continue;
                }

                if target.get_entity().entity_id == vehicle.get_entity().entity_id {
                    last_error = Some(ERROR_LOOP.create_without_context());
                    continue;
                }

                if is_riding_recursive(target.as_ref(), vehicle.as_ref()).await {
                    last_error = Some(ERROR_LOOP.create_without_context());
                    continue;
                }

                let current_vehicle = target.get_entity().vehicle.lock().await.clone();
                if let Some(ref curr_veh) = current_vehicle {
                    if curr_veh.get_entity().entity_id == vehicle.get_entity().entity_id {
                        last_error = Some(ERROR_ALREADY_RIDING.create_without_context(
                            target.get_display_name().await,
                            vehicle.get_display_name().await,
                        ));
                        continue;
                    }
                    // Dismount first
                    curr_veh
                        .get_entity()
                        .remove_passenger(target.get_entity().entity_id)
                        .await;
                }

                vehicle
                    .get_entity()
                    .add_passenger(vehicle.clone(), target.clone())
                    .await;
                success_count += 1;

                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_RIDE_MOUNT_SUCCESS,
                    translation::java::COMMANDS_RIDE_MOUNT_SUCCESS,
                    [
                        target.get_display_name().await,
                        vehicle.get_display_name().await,
                    ],
                );
                context.source.send_feedback(msg, true).await;
            }

            if success_count == 0 {
                if let Some(err) = last_error {
                    return Err(err);
                }
                return Err(ERROR_GENERIC.create_without_context(
                    targets[0].get_display_name().await,
                    vehicle.get_display_name().await,
                ));
            }

            Ok(success_count)
        })
    }
}

struct RideDismountExecutor;

impl CommandExecutor for RideDismountExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, "target").await?;

            let mut success_count = 0;
            let mut last_error = None;

            for target in &targets {
                let current_vehicle = target.get_entity().vehicle.lock().await.clone();
                if let Some(vehicle) = current_vehicle {
                    vehicle
                        .get_entity()
                        .remove_passenger(target.get_entity().entity_id)
                        .await;
                    success_count += 1;

                    let msg = TextComponent::translate_cross(
                        translation::java::COMMANDS_RIDE_DISMOUNT_SUCCESS,
                        translation::java::COMMANDS_RIDE_DISMOUNT_SUCCESS,
                        [target.get_display_name().await],
                    );
                    context.source.send_feedback(msg, true).await;
                } else {
                    last_error = Some(
                        ERROR_NOT_RIDING.create_without_context(target.get_display_name().await),
                    );
                }
            }

            if success_count == 0 {
                if let Some(err) = last_error {
                    return Err(err);
                }
                return Err(
                    ERROR_NOT_RIDING.create_without_context(targets[0].get_display_name().await)
                );
            }

            Ok(success_count)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder =
        command("ride", DESCRIPTION).requires(PERMISSION).then(
            argument("target", EntityArgumentType::Entities)
                .then(literal("mount").then(
                    argument("vehicle", EntityArgumentType::Entity).executes(RideMountExecutor),
                ))
                .then(literal("dismount").executes(RideDismountExecutor)),
        );

    dispatcher.register(builder);
}

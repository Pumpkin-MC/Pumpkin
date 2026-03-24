use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["attribute"];

const DESCRIPTION: &str = "Queries, adds, removes, or sets an entity attribute.";

const ARG_ATTRIBUTE: &str = "attribute";
const ARG_SCALE: &str = "scale";
const ARG_VALUE: &str = "value";

fn lookup_attribute(name: &str) -> Option<Attributes> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    match name {
        "armor" => Some(Attributes::ARMOR),
        "armor_toughness" => Some(Attributes::ARMOR_TOUGHNESS),
        "attack_damage" => Some(Attributes::ATTACK_DAMAGE),
        "attack_knockback" => Some(Attributes::ATTACK_KNOCKBACK),
        "attack_speed" => Some(Attributes::ATTACK_SPEED),
        "block_break_speed" => Some(Attributes::BLOCK_BREAK_SPEED),
        "block_interaction_range" => Some(Attributes::BLOCK_INTERACTION_RANGE),
        "burning_time" => Some(Attributes::BURNING_TIME),
        "camera_distance" => Some(Attributes::CAMERA_DISTANCE),
        "explosion_knockback_resistance" => Some(Attributes::EXPLOSION_KNOCKBACK_RESISTANCE),
        "entity_interaction_range" => Some(Attributes::ENTITY_INTERACTION_RANGE),
        "fall_damage_multiplier" => Some(Attributes::FALL_DAMAGE_MULTIPLIER),
        "flying_speed" => Some(Attributes::FLYING_SPEED),
        "follow_range" => Some(Attributes::FOLLOW_RANGE),
        "gravity" => Some(Attributes::GRAVITY),
        "jump_strength" => Some(Attributes::JUMP_STRENGTH),
        "knockback_resistance" => Some(Attributes::KNOCKBACK_RESISTANCE),
        "luck" => Some(Attributes::LUCK),
        "max_absorption" => Some(Attributes::MAX_ABSORPTION),
        "max_health" => Some(Attributes::MAX_HEALTH),
        "mining_efficiency" => Some(Attributes::MINING_EFFICIENCY),
        "movement_efficiency" => Some(Attributes::MOVEMENT_EFFICIENCY),
        "movement_speed" => Some(Attributes::MOVEMENT_SPEED),
        "oxygen_bonus" => Some(Attributes::OXYGEN_BONUS),
        "safe_fall_distance" => Some(Attributes::SAFE_FALL_DISTANCE),
        "scale" => Some(Attributes::SCALE),
        "sneaking_speed" => Some(Attributes::SNEAKING_SPEED),
        "spawn_reinforcements" => Some(Attributes::SPAWN_REINFORCEMENTS),
        "step_height" => Some(Attributes::STEP_HEIGHT),
        "submerged_mining_speed" => Some(Attributes::SUBMERGED_MINING_SPEED),
        "sweeping_damage_ratio" => Some(Attributes::SWEEPING_DAMAGE_RATIO),
        "tempt_range" => Some(Attributes::TEMPT_RANGE),
        "water_movement_efficiency" => Some(Attributes::WATER_MOVEMENT_EFFICIENCY),
        _ => None,
    }
}

fn get_attribute_arg<'a>(
    args: &'a ConsumedArgs<'a>,
    entity_name: &str,
) -> Result<Attributes, CommandError> {
    let Some(Arg::Simple(attr_name)) = args.get(ARG_ATTRIBUTE) else {
        return Err(CommandError::InvalidConsumption(Some(ARG_ATTRIBUTE.into())));
    };
    lookup_attribute(attr_name).ok_or(CommandError::CommandFailed(TextComponent::translate(
        translation::COMMANDS_ATTRIBUTE_FAILED_NO_ATTRIBUTE,
        [
            TextComponent::text(entity_name.to_string()),
            TextComponent::text(attr_name.to_string()),
        ],
    )))
}

fn get_living_entity(
    target: &Arc<dyn EntityBase>,
) -> Result<&crate::entity::living::LivingEntity, CommandError> {
    let entity_name = target.get_entity().entity_type.resource_name;
    target
        .get_living_entity()
        .ok_or(CommandError::CommandFailed(TextComponent::translate(
            translation::COMMANDS_ATTRIBUTE_FAILED_ENTITY,
            [TextComponent::text(entity_name.to_string())],
        )))
}

struct GetExecutor;

impl CommandExecutor for GetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, "target")?;
            let entity_name = target.get_entity().entity_type.resource_name;
            let attribute = get_attribute_arg(args, entity_name)?;
            let living = get_living_entity(&target)?;

            let Some(Arg::Simple(attr_name)) = args.get(ARG_ATTRIBUTE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_ATTRIBUTE.into())));
            };

            let scale: f64 = args
                .get(ARG_SCALE)
                .and_then(|a| {
                    if let Arg::Simple(s) = a {
                        s.parse::<f64>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(1.0);

            let value = living.get_attribute_value(&attribute) * scale;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_ATTRIBUTE_VALUE_GET_SUCCESS,
                    [
                        TextComponent::text(attr_name.to_string()),
                        TextComponent::text(entity_name.to_string()),
                        TextComponent::text(format!("{value}")),
                    ],
                ))
                .await;
            Ok(value as i32)
        })
    }
}

struct BaseGetExecutor;

impl CommandExecutor for BaseGetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, "target")?;
            let entity_name = target.get_entity().entity_type.resource_name;
            let attribute = get_attribute_arg(args, entity_name)?;
            let living = get_living_entity(&target)?;

            let Some(Arg::Simple(attr_name)) = args.get(ARG_ATTRIBUTE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_ATTRIBUTE.into())));
            };

            let scale: f64 = args
                .get(ARG_SCALE)
                .and_then(|a| {
                    if let Arg::Simple(s) = a {
                        s.parse::<f64>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(1.0);

            let value = living.get_attribute_base(&attribute) * scale;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_ATTRIBUTE_BASE_VALUE_GET_SUCCESS,
                    [
                        TextComponent::text(attr_name.to_string()),
                        TextComponent::text(entity_name.to_string()),
                        TextComponent::text(format!("{value}")),
                    ],
                ))
                .await;
            Ok(value as i32)
        })
    }
}

struct BaseSetExecutor;

impl CommandExecutor for BaseSetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, "target")?;
            let entity_name = target.get_entity().entity_type.resource_name;
            let attribute = get_attribute_arg(args, entity_name)?;
            let living = get_living_entity(&target)?;

            let Some(Arg::Simple(attr_name)) = args.get(ARG_ATTRIBUTE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_ATTRIBUTE.into())));
            };
            let Some(Arg::Simple(val_str)) = args.get(ARG_VALUE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_VALUE.into())));
            };
            let value: f64 = val_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_VALUE.into())))?;

            living.set_attribute_base(&attribute, value);

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_ATTRIBUTE_BASE_VALUE_SET_SUCCESS,
                    [
                        TextComponent::text(attr_name.to_string()),
                        TextComponent::text(entity_name.to_string()),
                        TextComponent::text(format!("{value}")),
                    ],
                ))
                .await;
            Ok(1)
        })
    }
}

struct BaseResetExecutor;

impl CommandExecutor for BaseResetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentConsumer::find_arg(args, "target")?;
            let entity_name = target.get_entity().entity_type.resource_name;
            let attribute = get_attribute_arg(args, entity_name)?;
            let living = get_living_entity(&target)?;

            let Some(Arg::Simple(attr_name)) = args.get(ARG_ATTRIBUTE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_ATTRIBUTE.into())));
            };

            living.set_attribute_base(&attribute, attribute.default_value);

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_ATTRIBUTE_BASE_VALUE_RESET_SUCCESS,
                    [
                        TextComponent::text(attr_name.to_string()),
                        TextComponent::text(entity_name.to_string()),
                        TextComponent::text(format!("{}", attribute.default_value)),
                    ],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(EntityArgumentConsumer).then(
            argument(ARG_ATTRIBUTE, SimpleArgConsumer)
                .then(
                    literal("get")
                        .execute(GetExecutor)
                        .then(argument(ARG_SCALE, SimpleArgConsumer).execute(GetExecutor)),
                )
                .then(
                    literal("base")
                        .then(
                            literal("get").execute(BaseGetExecutor).then(
                                argument(ARG_SCALE, SimpleArgConsumer).execute(BaseGetExecutor),
                            ),
                        )
                        .then(
                            literal("set").then(
                                argument(ARG_VALUE, SimpleArgConsumer).execute(BaseSetExecutor),
                            ),
                        )
                        .then(literal("reset").execute(BaseResetExecutor)),
                ),
        ),
    )
}

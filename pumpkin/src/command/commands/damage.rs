use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_util::text::{
    color::{Color, NamedColor},
    TextComponent,
};

use crate::command::{
    args::{
        bounded_num::BoundedNumArgumentConsumer,
        damage_type::DamageTypeArgumentConsumer, // Add this import
        entity::EntityArgumentConsumer,
        position_3d::Position3DArgumentConsumer,
        Arg,
        ConsumedArgs,
        FindArg,
    },
    tree::CommandTree,
    tree_builder::{argument, literal},
    CommandError, CommandExecutor, CommandSender,
};

const NAMES: [&str; 1] = ["damage"];
const DESCRIPTION: &str = "Deals damage to entities";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";
const ARG_DAMAGE_TYPE: &str = "damageType";
const ARG_LOCATION: &str = "location";
const ARG_ENTITY: &str = "entity";
const ARG_CAUSE: &str = "cause";

fn amount_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new().name(ARG_AMOUNT).min(0.0)
}

struct DamageLocationExecutor;
struct DamageEntityExecutor(bool); // true if has cause

#[async_trait]
impl CommandExecutor for DamageLocationExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
        let amount = match BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_AMOUNT) {
            Ok(Ok(v)) => v,
            _ => {
                sender
                    .send_message(
                        TextComponent::text("Invalid damage amount")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        let damage_type = args
            .get(ARG_DAMAGE_TYPE)
            .map(|arg| match arg {
                Arg::DamageType(dt) => **dt,
                _ => DamageType::Generic,
            })
            .unwrap_or(DamageType::Generic);

        let location = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        // Apply damage from location with position
        target
            .living_entity
            .damage_with_context(amount, damage_type, Some(location.clone()), None, None)
            .await;

        // Send success message
        sender
            .send_message(TextComponent::translate(
                "commands.damage.success",
                [
                    TextComponent::text(amount.to_string()),
                    TextComponent::text(target.gameprofile.name.clone()),
                ]
                .into(),
            ))
            .await;

        Ok(())
    }
}

#[async_trait]
impl CommandExecutor for DamageEntityExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
        let amount = match BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_AMOUNT) {
            Ok(Ok(v)) => v,
            _ => {
                sender
                    .send_message(
                        TextComponent::text("Invalid damage amount")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        // Default to generic damage if no type specified
        let damage_type = args
            .get(ARG_DAMAGE_TYPE)
            .map(|arg| match arg {
                Arg::DamageType(dt) => **dt,
                _ => DamageType::Generic,
            })
            .unwrap_or(DamageType::Generic);

        // Get source and cause entities if specified
        let source = EntityArgumentConsumer::find_arg(args, ARG_ENTITY).ok();
        let cause = if self.0 {
            EntityArgumentConsumer::find_arg(args, ARG_CAUSE).ok()
        } else {
            None
        };

        // Apply damage with entity context
        target
            .living_entity
            .damage_with_context(amount, damage_type, None, source, cause)
            .await;

        // Send success message
        sender
            .send_message(TextComponent::translate(
                "commands.damage.success",
                [
                    TextComponent::text(amount.to_string()),
                    TextComponent::text(target.gameprofile.name.clone()),
                ]
                .into(),
            ))
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, EntityArgumentConsumer).then(
            argument(ARG_AMOUNT, amount_consumer())
                // Basic damage
                .execute(DamageEntityExecutor(false))
                // With damage type
                .then(
                    argument(ARG_DAMAGE_TYPE, DamageTypeArgumentConsumer)
                        .execute(DamageEntityExecutor(false))
                        // At location
                        .then(
                            literal("at").then(
                                argument(ARG_LOCATION, Position3DArgumentConsumer)
                                    .execute(DamageLocationExecutor),
                            ),
                        )
                        // By entity
                        .then(
                            literal("by").then(
                                argument(ARG_ENTITY, EntityArgumentConsumer)
                                    .execute(DamageEntityExecutor(false))
                                    // From cause
                                    .then(
                                        literal("from").then(
                                            argument(ARG_CAUSE, EntityArgumentConsumer)
                                                .execute(DamageEntityExecutor(true)),
                                        ),
                                    ),
                            ),
                        ),
                ),
        ),
    )
}

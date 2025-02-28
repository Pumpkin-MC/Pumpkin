use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};

use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{
        Arg, ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer,
        entity::EntityArgumentConsumer, position_3d::Position3DArgumentConsumer,
        resource::damage_type::DamageTypeArgumentConsumer,
    },
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
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

struct LocationExecutor;
struct EntityExecutor(bool);

async fn send_damage_result(
    sender: &mut CommandSender<'_>,
    success: bool,
    amount: f32,
    target_name: String,
) {
    if !success {
        sender
            .send_message(
                TextComponent::translate("commands.damage.invulnerable", [])
                    .color(Color::Named(NamedColor::Red)),
            )
            .await;
        return;
    }

    sender
        .send_message(TextComponent::translate(
            "commands.damage.success",
            [
                TextComponent::text(amount.to_string()),
                TextComponent::text(target_name),
            ],
        ))
        .await;
}

#[async_trait]
impl CommandExecutor for LocationExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;

        let Ok(Ok(amount)) = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_AMOUNT) else {
            sender
                .send_message(
                    TextComponent::text("Invalid damage amount")
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        let damage_type = args
            .get(ARG_DAMAGE_TYPE)
            .map_or(DamageType::GENERIC, |arg| match arg {
                Arg::DamageType(dt) => *dt,
                _ => DamageType::GENERIC,
            });

        let location = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        let success = target
            .living_entity
            .damage_with_context(amount, damage_type, Some(location), None, None)
            .await;

        send_damage_result(sender, success, amount, target.gameprofile.name.clone()).await;

        Ok(())
    }
}

#[async_trait]
impl CommandExecutor for EntityExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;

        let Ok(Ok(amount)) = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_AMOUNT) else {
            sender
                .send_message(
                    TextComponent::text("Invalid damage amount")
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        let damage_type = args
            .get(ARG_DAMAGE_TYPE)
            .map_or(DamageType::GENERIC, |arg| match arg {
                Arg::DamageType(dt) => *dt,
                _ => DamageType::GENERIC,
            });

        let source = EntityArgumentConsumer::find_arg(args, ARG_ENTITY).ok();
        let cause = if self.0 {
            EntityArgumentConsumer::find_arg(args, ARG_CAUSE).ok()
        } else {
            None
        };

        let success = target
            .living_entity
            .damage_with_context(
                amount,
                damage_type,
                None,
                source.as_ref().map(|e| &e.living_entity.entity),
                cause.as_ref().map(|e| &e.living_entity.entity),
            )
            .await;

        send_damage_result(sender, success, amount, target.gameprofile.name.clone()).await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, EntityArgumentConsumer).then(
            argument(ARG_AMOUNT, amount_consumer())
                // Basic damage
                .execute(EntityExecutor(false))
                // With damage type
                .then(
                    argument(ARG_DAMAGE_TYPE, DamageTypeArgumentConsumer)
                        .execute(EntityExecutor(false))
                        // At location
                        .then(
                            literal("at").then(
                                argument(ARG_LOCATION, Position3DArgumentConsumer)
                                    .execute(LocationExecutor),
                            ),
                        )
                        // By entity
                        .then(
                            literal("by").then(
                                argument(ARG_ENTITY, EntityArgumentConsumer)
                                    .execute(EntityExecutor(false))
                                    // From cause
                                    .then(
                                        literal("from").then(
                                            argument(ARG_CAUSE, EntityArgumentConsumer)
                                                .execute(EntityExecutor(true)),
                                        ),
                                    ),
                            ),
                        ),
                ),
        ),
    )
}

use pumpkin_data::{damage::DamageType, translation};
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{
        Arg, ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer,
        entity::EntityArgumentConsumer, position_3d::Position3DArgumentConsumer,
        resource::damage_type::DamageTypeArgumentConsumer,
    },
    dispatcher::CommandError,
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["damage"];
const DESCRIPTION: &str = "Deals damage to entities";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";
const ARG_DAMAGE_TYPE: &str = "damageType";
const ARG_LOCATION: &str = "location";
const ARG_ENTITY: &str = "entity";
const ARG_CAUSE: &str = "cause";

const fn amount_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new().name(ARG_AMOUNT).min(0.0)
}

struct LocationExecutor;
struct EntityExecutor(bool);

fn send_damage_result(
    sender: &CommandSender,
    success: bool,
    amount: f32,
    target_name: TextComponent,
) -> Result<i32, CommandError> {
    if !success {
        return Err(CommandError::CommandFailed(TextComponent::translate_cross(
            translation::java::COMMANDS_DAMAGE_INVULNERABLE,
            translation::java::COMMANDS_DAMAGE_INVULNERABLE,
            [],
        )));
    }

    sender.send_message(TextComponent::translate_cross(
        translation::java::COMMANDS_DAMAGE_SUCCESS,
        translation::bedrock::COMMANDS_DAMAGE_SUCCESS,
        [TextComponent::text(amount.to_string()), target_name],
    ));

    Ok(1) // not arbitrary, this is what vanilla does
}

impl CommandExecutor for LocationExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;

        let amount = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_AMOUNT)??;

        let damage_type = args
            .get(ARG_DAMAGE_TYPE)
            .map_or(DamageType::GENERIC, |arg| match arg {
                Arg::DamageType(dt) => *dt,
                _ => DamageType::GENERIC,
            });

        let location = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        let success =
            target.damage_with_context(&*target, amount, damage_type, Some(location), None, None);

        send_damage_result(sender, success, amount, target.get_display_name())
    }
}

impl CommandExecutor for EntityExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;

        let amount = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_AMOUNT)??;

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

        let success = target.damage_with_context(
            &*target,
            amount,
            damage_type,
            None,
            source.as_ref().map(|e| e.as_ref() as &dyn EntityBase),
            cause.as_ref().map(|e| e.as_ref() as &dyn EntityBase),
        );

        send_damage_result(sender, success, amount, target.get_display_name())
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

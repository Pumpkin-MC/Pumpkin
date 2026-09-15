use pumpkin_data::damage::DamageType;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::resource::{DAMAGE_TYPE_ARGUMENT, ResourceArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Deals damage to entities";
const PERMISSION: &str = "minecraft:command.damage";

const ERROR_INVULNERABLE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DAMAGE_INVULNERABLE,
    translation::java::COMMANDS_DAMAGE_INVULNERABLE,
);

fn send_damage_result(
    context: &CommandContext,
    success: bool,
    amount: f32,
    target_name: TextComponent,
) -> CommandExecutorResult {
    if !success {
        return Err(ERROR_INVULNERABLE.create_without_context());
    }

    context.source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_DAMAGE_SUCCESS,
            translation::bedrock::COMMANDS_DAMAGE_SUCCESS,
            [TextComponent::text(amount.to_string()), target_name],
        ),
        true,
    );

    Ok(1)
}

struct DamageLocationExecutor;

impl CommandExecutor for DamageLocationExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, "target")?;
        let amount = FloatArgumentType::get(context, "amount")?;
        let damage_type = ResourceArgument::get_damage_type(context, "damageType")?;
        let location =
            Vec3ArgumentType::get_coordinates(context, "location")?.resolve(&context.source);

        let success =
            target.damage_with_context(&*target, amount, *damage_type, Some(location), None, None);

        send_damage_result(context, success, amount, target.get_display_name())
    }
}

enum EntityMode {
    Basic,
    WithType,
    WithSource,
    WithSourceAndCause,
}

struct DamageEntityExecutor(EntityMode);

impl CommandExecutor for DamageEntityExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, "target")?;
        let amount = FloatArgumentType::get(context, "amount")?;

        let damage_type = match self.0 {
            EntityMode::Basic => DamageType::GENERIC,
            EntityMode::WithType | EntityMode::WithSource | EntityMode::WithSourceAndCause => {
                *ResourceArgument::get_damage_type(context, "damageType")?
            }
        };

        let source = match self.0 {
            EntityMode::WithSource | EntityMode::WithSourceAndCause => {
                Some(EntityArgumentType::get_entity(context, "entity")?)
            }
            EntityMode::Basic | EntityMode::WithType => None,
        };

        let cause = match self.0 {
            EntityMode::WithSourceAndCause => {
                Some(EntityArgumentType::get_entity(context, "cause")?)
            }
            EntityMode::Basic | EntityMode::WithType | EntityMode::WithSource => None,
        };

        let success = target.damage_with_context(
            &*target,
            amount,
            damage_type,
            None,
            source.as_ref().map(|e| e.as_ref() as &dyn EntityBase),
            cause.as_ref().map(|e| e.as_ref() as &dyn EntityBase),
        );

        send_damage_result(context, success, amount, target.get_display_name())
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("damage", DESCRIPTION).requires(PERMISSION).then(
            argument("target", EntityArgumentType::Entity).then(
                argument("amount", FloatArgumentType::with_min(0.0))
                    .executes(DamageEntityExecutor(EntityMode::Basic))
                    .then(
                        argument("damageType", DAMAGE_TYPE_ARGUMENT.clone())
                            .executes(DamageEntityExecutor(EntityMode::WithType))
                            .then(
                                literal("at").then(
                                    argument("location", Vec3ArgumentType::Default)
                                        .executes(DamageLocationExecutor),
                                ),
                            )
                            .then(
                                literal("by").then(
                                    argument("entity", EntityArgumentType::Entity)
                                        .executes(DamageEntityExecutor(EntityMode::WithSource))
                                        .then(literal("from").then(
                                            argument("cause", EntityArgumentType::Entity).executes(
                                                DamageEntityExecutor(
                                                    EntityMode::WithSourceAndCause,
                                                ),
                                            ),
                                        )),
                                ),
                            ),
                    ),
            ),
        ),
    );
}

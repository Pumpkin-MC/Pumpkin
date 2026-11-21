use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::attribute::AttributeArgumentType;
use crate::command::argument_types::client_parsed::IdStringArgumentType;
use crate::command::argument_types::core::double::DoubleArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::attributes::ModifierOperation;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Queries or modifies attributes of an entity.";
const PERMISSION: &str = "minecraft:command.attribute";

const ARG_TARGET: &str = "target";
const ARG_ATTRIBUTE: &str = "attribute";
const ARG_SCALE: &str = "scale";
const ARG_VALUE: &str = "value";
/// A modifier's `ResourceLocation` id, as vanilla since 1.21.
const ARG_ID: &str = "id";

const FAILED_ENTITY_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ATTRIBUTE_FAILED_ENTITY,
    translation::java::COMMANDS_ATTRIBUTE_FAILED_ENTITY,
);

const NO_ATTRIBUTE_ERROR: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ATTRIBUTE_FAILED_NO_ATTRIBUTE,
    translation::java::COMMANDS_ATTRIBUTE_FAILED_NO_ATTRIBUTE,
);

const MODIFIER_ALREADY_PRESENT_ERROR: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ATTRIBUTE_FAILED_MODIFIER_ALREADY_PRESENT,
    translation::java::COMMANDS_ATTRIBUTE_FAILED_MODIFIER_ALREADY_PRESENT,
);

const NO_MODIFIER_ERROR: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ATTRIBUTE_FAILED_NO_MODIFIER,
    translation::java::COMMANDS_ATTRIBUTE_FAILED_NO_MODIFIER,
);

fn attribute_translation_key(attribute: &Attributes) -> String {
    let path = attribute
        .name
        .strip_prefix("minecraft:")
        .unwrap_or(attribute.name);
    format!("attribute.name.{path}")
}

struct GetExecutor {
    is_base: bool,
    has_scale: bool,
}

impl CommandExecutor for GetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, ARG_TARGET)?;
        let living = target
            .get_living_entity()
            .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

        let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
        let scale = if self.has_scale {
            DoubleArgumentType::get(context, ARG_SCALE)?
        } else {
            1.0
        };

        let has_attr = living
            .attributes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&attribute.id);
        if !has_attr {
            return Err(NO_ATTRIBUTE_ERROR.create_without_context(
                target.get_name(),
                TextComponent::translate(attribute_translation_key(&attribute), []),
            ));
        }

        if self.is_base {
            let base_value = living.get_attribute_base(&attribute);
            let scaled_value = base_value * scale;

            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_ATTRIBUTE_BASE_VALUE_GET_SUCCESS,
                    translation::java::COMMANDS_ATTRIBUTE_BASE_VALUE_GET_SUCCESS,
                    [
                        TextComponent::translate(attribute_translation_key(&attribute), []),
                        target.get_name(),
                        TextComponent::text(base_value.to_string()),
                    ],
                ),
                false,
            );

            Ok(scaled_value as i32)
        } else {
            let value = living.get_attribute_value(&attribute);
            let scaled_value = value * scale;

            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_ATTRIBUTE_VALUE_GET_SUCCESS,
                    translation::java::COMMANDS_ATTRIBUTE_VALUE_GET_SUCCESS,
                    [
                        TextComponent::translate(attribute_translation_key(&attribute), []),
                        target.get_name(),
                        TextComponent::text(value.to_string()),
                    ],
                ),
                false,
            );

            Ok(scaled_value as i32)
        }
    }
}

struct BaseSetExecutor;

impl CommandExecutor for BaseSetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, ARG_TARGET)?;
        let living = target
            .get_living_entity()
            .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

        let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
        let value = DoubleArgumentType::get(context, ARG_VALUE)?;

        let has_attr = living
            .attributes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&attribute.id);
        if !has_attr {
            return Err(NO_ATTRIBUTE_ERROR.create_without_context(
                target.get_name(),
                TextComponent::translate(attribute_translation_key(&attribute), []),
            ));
        }

        living.set_attribute_base(&attribute, value);
        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![attribute.clone()],
        );

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_ATTRIBUTE_BASE_VALUE_SET_SUCCESS,
                translation::java::COMMANDS_ATTRIBUTE_BASE_VALUE_SET_SUCCESS,
                [
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                    target.get_name(),
                    TextComponent::text(value.to_string()),
                ],
            ),
            true,
        );

        Ok(value as i32)
    }
}

struct BaseResetExecutor;

impl CommandExecutor for BaseResetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, ARG_TARGET)?;
        let living = target
            .get_living_entity()
            .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

        let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;

        let has_attr = living
            .attributes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&attribute.id);
        if !has_attr {
            return Err(NO_ATTRIBUTE_ERROR.create_without_context(
                target.get_name(),
                TextComponent::translate(attribute_translation_key(&attribute), []),
            ));
        }

        let default_val = attribute.default_value;
        living.set_attribute_base(&attribute, default_val);
        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![attribute.clone()],
        );

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_ATTRIBUTE_BASE_VALUE_RESET_SUCCESS,
                translation::java::COMMANDS_ATTRIBUTE_BASE_VALUE_RESET_SUCCESS,
                [
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                    target.get_name(),
                    TextComponent::text(default_val.to_string()),
                ],
            ),
            true,
        );

        Ok(default_val as i32)
    }
}

struct ModifierAddExecutor {
    operation: ModifierOperation,
}

impl CommandExecutor for ModifierAddExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, ARG_TARGET)?;
        let living = target
            .get_living_entity()
            .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

        let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
        let modifier_id = IdStringArgumentType::get(context, ARG_ID)?.to_owned();
        let value = DoubleArgumentType::get(context, ARG_VALUE)?;

        let res = {
            let mut map = living
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let inst = map.get_mut(&attribute.id).ok_or_else(|| {
                NO_ATTRIBUTE_ERROR.create_without_context(
                    target.get_name(),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                )
            })?;

            if inst.modifiers.iter().any(|m| m.id == modifier_id) {
                Err(MODIFIER_ALREADY_PRESENT_ERROR.create_without_context(
                    TextComponent::text(modifier_id.clone()),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                    target.get_name(),
                ))
            } else {
                inst.add_or_replace_permanent_modifier(crate::entity::attributes::Modifier {
                    id: modifier_id.clone(),
                    amount: value,
                    operation: self.operation,
                });
                Ok(())
            }
        };
        res?;

        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![attribute.clone()],
        );

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_ATTRIBUTE_MODIFIER_ADD_SUCCESS,
                translation::java::COMMANDS_ATTRIBUTE_MODIFIER_ADD_SUCCESS,
                [
                    TextComponent::text(modifier_id),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                    target.get_name(),
                ],
            ),
            true,
        );

        Ok(1)
    }
}

struct ModifierRemoveExecutor;

impl CommandExecutor for ModifierRemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, ARG_TARGET)?;
        let living = target
            .get_living_entity()
            .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

        let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
        let modifier_id = IdStringArgumentType::get(context, ARG_ID)?.to_owned();

        let res = {
            let mut map = living
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let inst = map.get_mut(&attribute.id).ok_or_else(|| {
                NO_ATTRIBUTE_ERROR.create_without_context(
                    target.get_name(),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                )
            })?;

            inst.modifiers
                .iter()
                .find(|m| m.id == modifier_id)
                .ok_or_else(|| {
                    NO_MODIFIER_ERROR.create_without_context(
                        TextComponent::translate(attribute_translation_key(&attribute), []),
                        target.get_name(),
                        TextComponent::text(modifier_id.clone()),
                    )
                })?;

            inst.remove_modifier(&modifier_id);
            Ok(())
        };
        res?;

        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![attribute.clone()],
        );

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_ATTRIBUTE_MODIFIER_REMOVE_SUCCESS,
                translation::java::COMMANDS_ATTRIBUTE_MODIFIER_REMOVE_SUCCESS,
                [
                    TextComponent::text(modifier_id),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                    target.get_name(),
                ],
            ),
            true,
        );

        Ok(1)
    }
}

struct ModifierGetExecutor;

impl CommandExecutor for ModifierGetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, ARG_TARGET)?;
        let living = target
            .get_living_entity()
            .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

        let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
        let modifier_id = IdStringArgumentType::get(context, ARG_ID)?.to_owned();

        let val = {
            let map = living
                .attributes
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let inst = map.get(&attribute.id).ok_or_else(|| {
                NO_ATTRIBUTE_ERROR.create_without_context(
                    target.get_name(),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                )
            })?;

            let modifier = inst
                .modifiers
                .iter()
                .find(|m| m.id == modifier_id)
                .ok_or_else(|| {
                    NO_MODIFIER_ERROR.create_without_context(
                        TextComponent::translate(attribute_translation_key(&attribute), []),
                        target.get_name(),
                        TextComponent::text(modifier_id.clone()),
                    )
                })?;

            modifier.amount
        };
        // `getAttributeModifier`: the feedback shows the value, the result
        // is it times the optional scale.
        let scale = DoubleArgumentType::get(context, ARG_SCALE).unwrap_or(1.0);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_ATTRIBUTE_MODIFIER_VALUE_GET_SUCCESS,
                translation::java::COMMANDS_ATTRIBUTE_MODIFIER_VALUE_GET_SUCCESS,
                [
                    TextComponent::text(modifier_id),
                    TextComponent::translate(attribute_translation_key(&attribute), []),
                    target.get_name(),
                    TextComponent::text(val.to_string()),
                ],
            ),
            false,
        );

        Ok((val * scale) as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("attribute", DESCRIPTION).requires(PERMISSION).then(
            argument(ARG_TARGET, EntityArgumentType::Entity).then(
                argument(ARG_ATTRIBUTE, AttributeArgumentType)
                    .then(
                        literal("get")
                            .executes(GetExecutor {
                                is_base: false,
                                has_scale: false,
                            })
                            .then(argument(ARG_SCALE, DoubleArgumentType::any()).executes(
                                GetExecutor {
                                    is_base: false,
                                    has_scale: true,
                                },
                            )),
                    )
                    .then(
                        literal("base")
                            .then(
                                literal("get")
                                    .executes(GetExecutor {
                                        is_base: true,
                                        has_scale: false,
                                    })
                                    .then(argument(ARG_SCALE, DoubleArgumentType::any()).executes(
                                        GetExecutor {
                                            is_base: true,
                                            has_scale: true,
                                        },
                                    )),
                            )
                            .then(
                                literal("set").then(
                                    argument(ARG_VALUE, DoubleArgumentType::any())
                                        .executes(BaseSetExecutor),
                                ),
                            )
                            .then(literal("reset").executes(BaseResetExecutor)),
                    )
                    .then(
                        literal("modifier")
                            .then(
                                literal("add").then(
                                    argument(ARG_ID, IdStringArgumentType).then(
                                        argument(ARG_VALUE, DoubleArgumentType::any())
                                            .then(literal("add_value").executes(
                                                ModifierAddExecutor {
                                                    operation: ModifierOperation::Add,
                                                },
                                            ))
                                            .then(literal("add_multiplied_base").executes(
                                                ModifierAddExecutor {
                                                    operation: ModifierOperation::MultiplyBase,
                                                },
                                            ))
                                            .then(literal("add_multiplied_total").executes(
                                                ModifierAddExecutor {
                                                    operation: ModifierOperation::MultiplyTotal,
                                                },
                                            )),
                                    ),
                                ),
                            )
                            .then(
                                literal("remove").then(
                                    argument(ARG_ID, IdStringArgumentType)
                                        .executes(ModifierRemoveExecutor),
                                ),
                            )
                            .then(
                                literal("value").then(
                                    literal("get").then(
                                        argument(ARG_ID, IdStringArgumentType)
                                            .executes(ModifierGetExecutor)
                                            .then(
                                                argument(ARG_SCALE, DoubleArgumentType::any())
                                                    .executes(ModifierGetExecutor),
                                            ),
                                    ),
                                ),
                            ),
                    ),
            ),
        ),
    );
}

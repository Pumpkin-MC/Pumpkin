use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::attribute::AttributeArgumentType;
use crate::command::argument_types::core::double::DoubleArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::uuid::UuidArgumentType;
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
const ARG_UUID: &str = "uuid";
const ARG_NAME: &str = "name";

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
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentType::get_entity(context, ARG_TARGET).await?;
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
                .unwrap()
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

                context
                    .source
                    .send_feedback(
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
                    )
                    .await;

                Ok(scaled_value as i32)
            } else {
                let value = living.get_attribute_value(&attribute);
                let scaled_value = value * scale;

                context
                    .source
                    .send_feedback(
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
                    )
                    .await;

                Ok(scaled_value as i32)
            }
        })
    }
}

struct BaseSetExecutor;

impl CommandExecutor for BaseSetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentType::get_entity(context, ARG_TARGET).await?;
            let living = target
                .get_living_entity()
                .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

            let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
            let value = DoubleArgumentType::get(context, ARG_VALUE)?;

            let has_attr = living
                .attributes
                .read()
                .unwrap()
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
            )
            .await;

            context
                .source
                .send_feedback(
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
                )
                .await;

            Ok(value as i32)
        })
    }
}

struct BaseResetExecutor;

impl CommandExecutor for BaseResetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentType::get_entity(context, ARG_TARGET).await?;
            let living = target
                .get_living_entity()
                .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

            let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;

            let has_attr = living
                .attributes
                .read()
                .unwrap()
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
            )
            .await;

            context
                .source
                .send_feedback(
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
                )
                .await;

            Ok(default_val as i32)
        })
    }
}

struct ModifierAddExecutor {
    operation: ModifierOperation,
}

impl CommandExecutor for ModifierAddExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentType::get_entity(context, ARG_TARGET).await?;
            let living = target
                .get_living_entity()
                .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

            let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
            let uuid = UuidArgumentType::get(context, ARG_UUID)?;
            let _name = StringArgumentType::get(context, ARG_NAME)?;
            let value = DoubleArgumentType::get(context, ARG_VALUE)?;

            let modifier_id = uuid.to_string();

            let res = {
                let mut map = living.attributes.write().unwrap();
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
                    inst.modifiers.push(crate::entity::attributes::Modifier {
                        id: modifier_id.clone(),
                        amount: value,
                        operation: self.operation,
                    });
                    inst.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
            };
            res?;

            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            )
            .await;

            context
                .source
                .send_feedback(
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
                )
                .await;

            Ok(1)
        })
    }
}

struct ModifierRemoveExecutor;

impl CommandExecutor for ModifierRemoveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentType::get_entity(context, ARG_TARGET).await?;
            let living = target
                .get_living_entity()
                .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

            let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
            let uuid = UuidArgumentType::get(context, ARG_UUID)?;

            let modifier_id = uuid.to_string();

            let res = {
                let mut map = living.attributes.write().unwrap();
                let inst = map.get_mut(&attribute.id).ok_or_else(|| {
                    NO_ATTRIBUTE_ERROR.create_without_context(
                        target.get_name(),
                        TextComponent::translate(attribute_translation_key(&attribute), []),
                    )
                })?;

                let index = inst
                    .modifiers
                    .iter()
                    .position(|m| m.id == modifier_id)
                    .ok_or_else(|| {
                        NO_MODIFIER_ERROR.create_without_context(
                            TextComponent::translate(attribute_translation_key(&attribute), []),
                            target.get_name(),
                            TextComponent::text(modifier_id.clone()),
                        )
                    })?;

                inst.modifiers.remove(index);
                inst.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            };
            res?;

            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            )
            .await;

            context
                .source
                .send_feedback(
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
                )
                .await;

            Ok(1)
        })
    }
}

struct ModifierGetExecutor;

impl CommandExecutor for ModifierGetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = EntityArgumentType::get_entity(context, ARG_TARGET).await?;
            let living = target
                .get_living_entity()
                .ok_or_else(|| FAILED_ENTITY_ERROR.create_without_context(target.get_name()))?;

            let attribute = AttributeArgumentType::get(context, ARG_ATTRIBUTE)?;
            let uuid = UuidArgumentType::get(context, ARG_UUID)?;

            let modifier_id = uuid.to_string();

            let val = {
                let map = living.attributes.read().unwrap();
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

            context
                .source
                .send_feedback(
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
                )
                .await;

            Ok(val as i32)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
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
                                    argument(ARG_UUID, UuidArgumentType).then(
                                        argument(ARG_NAME, StringArgumentType::SingleWord).then(
                                            argument(ARG_VALUE, DoubleArgumentType::any())
                                                .then(literal("add").executes(
                                                    ModifierAddExecutor {
                                                        operation: ModifierOperation::Add,
                                                    },
                                                ))
                                                .then(literal("multiply").executes(
                                                    ModifierAddExecutor {
                                                        operation: ModifierOperation::MultiplyTotal,
                                                    },
                                                ))
                                                .then(literal("multiply_base").executes(
                                                    ModifierAddExecutor {
                                                        operation: ModifierOperation::MultiplyBase,
                                                    },
                                                )),
                                        ),
                                    ),
                                ),
                            )
                            .then(
                                literal("remove").then(
                                    argument(ARG_UUID, UuidArgumentType)
                                        .executes(ModifierRemoveExecutor),
                                ),
                            )
                            .then(literal("value").then(literal("get").then(
                                argument(ARG_UUID, UuidArgumentType).executes(ModifierGetExecutor),
                            ))),
                    ),
            ),
        ),
    );
}

use std::collections::BTreeSet;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Manages the scoreboard tags of entities.";

const PERMISSION: &str = "minecraft:command.tag";

const ARG_TARGETS: &str = "targets";
const ARG_NAME: &str = "name";

const ADD_FAILED_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TAG_ADD_FAILED,
    translation::bedrock::COMMANDS_TAG_ADD_FAILED,
);

const REMOVE_FAILED_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TAG_REMOVE_FAILED,
    translation::bedrock::COMMANDS_TAG_REMOVE_FAILED,
);

#[derive(Clone, Copy)]
enum Action {
    Add,
    Remove,
}

struct ChangeExecutor(Action);

impl CommandExecutor for ChangeExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, ARG_TARGETS).await?;
            let tag = StringArgumentType::get(context, ARG_NAME)?.to_owned();

            let mut changed = 0;
            for target in &targets {
                let entity = target.get_entity();
                let success = match self.0 {
                    Action::Add => entity.add_scoreboard_tag(&tag).await,
                    Action::Remove => entity.remove_scoreboard_tag(&tag).await,
                };
                if success {
                    changed += 1;
                }
            }

            if changed == 0 {
                return Err(match self.0 {
                    Action::Add => ADD_FAILED_ERROR_TYPE.create_without_context(),
                    Action::Remove => REMOVE_FAILED_ERROR_TYPE.create_without_context(),
                });
            }

            let (single_key, multiple_key) = match self.0 {
                Action::Add => (
                    (
                        translation::java::COMMANDS_TAG_ADD_SUCCESS_SINGLE,
                        translation::bedrock::COMMANDS_TAG_ADD_SUCCESS_SINGLE,
                    ),
                    (
                        translation::java::COMMANDS_TAG_ADD_SUCCESS_MULTIPLE,
                        translation::bedrock::COMMANDS_TAG_ADD_SUCCESS_MULTIPLE,
                    ),
                ),
                Action::Remove => (
                    (
                        translation::java::COMMANDS_TAG_REMOVE_SUCCESS_SINGLE,
                        translation::bedrock::COMMANDS_TAG_REMOVE_SUCCESS_SINGLE,
                    ),
                    (
                        translation::java::COMMANDS_TAG_REMOVE_SUCCESS_MULTIPLE,
                        translation::bedrock::COMMANDS_TAG_REMOVE_SUCCESS_MULTIPLE,
                    ),
                ),
            };

            let msg = if targets.len() == 1 {
                TextComponent::translate_cross(
                    single_key.0,
                    single_key.1,
                    [
                        TextComponent::text(tag),
                        targets[0].get_display_name().await,
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    multiple_key.0,
                    multiple_key.1,
                    [
                        TextComponent::text(tag),
                        TextComponent::text(targets.len().to_string()),
                    ],
                )
            };

            context.source.send_feedback(msg, true).await;

            Ok(changed)
        })
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, ARG_TARGETS).await?;

            // BTreeSet keeps the output deterministic.
            let mut all_tags = BTreeSet::new();
            for target in &targets {
                let tags = target.get_entity().scoreboard_tags.lock().await;
                all_tags.extend(tags.iter().cloned());
            }

            let tag_list =
                TextComponent::text(all_tags.iter().cloned().collect::<Vec<String>>().join(", "));

            let msg = if targets.len() == 1 {
                let name = targets[0].get_display_name().await;
                if all_tags.is_empty() {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TAG_LIST_SINGLE_EMPTY,
                        translation::bedrock::COMMANDS_TAG_LIST_SINGLE_EMPTY,
                        [name],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TAG_LIST_SINGLE_SUCCESS,
                        translation::bedrock::COMMANDS_TAG_LIST_SINGLE_SUCCESS,
                        [
                            name,
                            TextComponent::text(all_tags.len().to_string()),
                            tag_list,
                        ],
                    )
                }
            } else {
                let count = TextComponent::text(targets.len().to_string());
                if all_tags.is_empty() {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TAG_LIST_MULTIPLE_EMPTY,
                        translation::bedrock::COMMANDS_TAG_LIST_MULTIPLE_EMPTY,
                        [count],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TAG_LIST_MULTIPLE_SUCCESS,
                        translation::bedrock::COMMANDS_TAG_LIST_MULTIPLE_SUCCESS,
                        [
                            count,
                            TextComponent::text(all_tags.len().to_string()),
                            tag_list,
                        ],
                    )
                }
            };

            context.source.send_feedback(msg, false).await;

            Ok(all_tags.len() as i32)
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
        command("tag", DESCRIPTION).requires(PERMISSION).then(
            argument(ARG_TARGETS, EntityArgumentType::Entities)
                .then(
                    literal("add").then(
                        argument(ARG_NAME, StringArgumentType::SingleWord)
                            .executes(ChangeExecutor(Action::Add)),
                    ),
                )
                .then(literal("list").executes(ListExecutor))
                .then(
                    literal("remove").then(
                        argument(ARG_NAME, StringArgumentType::SingleWord)
                            .executes(ChangeExecutor(Action::Remove)),
                    ),
                ),
        ),
    );
}

use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["tag"];

const DESCRIPTION: &str = "Controls entity tags.";

const ARG_NAME: &str = "name";

const MAX_TAGS: usize = 1024;

struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, "targets")?;
            let Some(Arg::Simple(tag_name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            let tag_name = (*tag_name).to_string();

            let mut count = 0;
            for target in targets {
                let entity = target.get_entity();
                let mut tags = entity.tags.lock().await;
                if tags.len() >= MAX_TAGS {
                    continue;
                }
                if !tags.contains(&tag_name) {
                    tags.push(tag_name.clone());
                    count += 1;
                }
            }

            if count == 0 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TAG_ADD_FAILED,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            if count == 1 {
                let entity_name = targets[0]
                    .get_entity()
                    .entity_type
                    .resource_name
                    .to_string();
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TAG_ADD_SUCCESS_SINGLE,
                        [
                            TextComponent::text(tag_name.clone()),
                            TextComponent::text(entity_name),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TAG_ADD_SUCCESS_MULTIPLE,
                        [
                            TextComponent::text(tag_name),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }

            Ok(count)
        })
    }
}

struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, "targets")?;
            let Some(Arg::Simple(tag_name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            let tag_name = (*tag_name).to_string();

            let mut count = 0;
            for target in targets {
                let entity = target.get_entity();
                let mut tags = entity.tags.lock().await;
                if let Some(pos) = tags.iter().position(|t| *t == tag_name) {
                    tags.remove(pos);
                    count += 1;
                }
            }

            if count == 0 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TAG_REMOVE_FAILED,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            if count == 1 {
                let entity_name = targets[0]
                    .get_entity()
                    .entity_type
                    .resource_name
                    .to_string();
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TAG_REMOVE_SUCCESS_SINGLE,
                        [
                            TextComponent::text(tag_name.clone()),
                            TextComponent::text(entity_name),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TAG_REMOVE_SUCCESS_MULTIPLE,
                        [
                            TextComponent::text(tag_name),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }

            Ok(count)
        })
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, "targets")?;

            let is_single = targets.len() == 1;
            let mut total_tags = 0;
            let mut all_tags: Vec<String> = Vec::new();

            for target in targets {
                let entity = target.get_entity();
                let tags = entity.tags.lock().await;
                total_tags += tags.len();

                if is_single {
                    let entity_name = entity.entity_type.resource_name;
                    if tags.is_empty() {
                        sender
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TAG_LIST_SINGLE_EMPTY,
                                [TextComponent::text(entity_name.to_string())],
                            ))
                            .await;
                    } else {
                        let tag_list = tags.join(", ");
                        sender
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TAG_LIST_SINGLE_SUCCESS,
                                [
                                    TextComponent::text(entity_name.to_string()),
                                    TextComponent::text(tags.len().to_string()),
                                    TextComponent::text(tag_list),
                                ],
                            ))
                            .await;
                    }
                } else {
                    for tag in tags.iter() {
                        if !all_tags.contains(tag) {
                            all_tags.push(tag.clone());
                        }
                    }
                }
            }

            if !is_single {
                if total_tags == 0 {
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_TAG_LIST_MULTIPLE_EMPTY,
                            [TextComponent::text(targets.len().to_string())],
                        ))
                        .await;
                } else {
                    let tag_list = all_tags.join(", ");
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_TAG_LIST_MULTIPLE_SUCCESS,
                            [
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(total_tags.to_string()),
                                TextComponent::text(tag_list),
                            ],
                        ))
                        .await;
                }
            }

            Ok(total_tags as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(EntitiesArgumentConsumer)
            .then(literal("add").then(argument(ARG_NAME, SimpleArgConsumer).execute(AddExecutor)))
            .then(
                literal("remove")
                    .then(argument(ARG_NAME, SimpleArgConsumer).execute(RemoveExecutor)),
            )
            .then(literal("list").execute(ListExecutor)),
    )
}

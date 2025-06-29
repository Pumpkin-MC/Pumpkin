use async_trait::async_trait;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, block_properties::get_block};
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::text::TextComponent;

use crate::{command::dispatcher::CommandError, server::Server};

use super::{
    super::{
        CommandSender,
        args::{ArgumentConsumer, RawArgs},
    },
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

pub struct BlockArgumentConsumer;

impl GetClientSideArgParser for BlockArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::BlockState
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for BlockArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;
        Some(Arg::Block(s))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for BlockArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "block"
    }
}

impl<'a> FindArg<'a> for BlockArgumentConsumer {
    type Data = Block;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Block(name)) => get_block(name).map_or_else(
                || {
                    Err(CommandError::CommandFailed(Box::new(
                        TextComponent::translate(
                            "argument.block.id.invalid",
                            [TextComponent::text((*name).to_string())],
                        ),
                    )))
                },
                Result::Ok,
            ),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

pub struct BlockPredicateArgumentConsumer;
#[derive(Debug)]
pub enum BlockPredicate {
    Tag(Vec<u16>),
    Block(u16),
}

impl GetClientSideArgParser for BlockPredicateArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::BlockPredicate
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for BlockPredicateArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;
        Some(Arg::BlockPredicate(s))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for BlockPredicateArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "filter"
    }
}

impl<'a> FindArg<'a> for BlockPredicateArgumentConsumer {
    type Data = Option<BlockPredicate>;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::BlockPredicate(name)) => {
                name.strip_prefix("#").map_or_else(
                    || {
                        get_block(name).map_or_else(
                            || {
                                Err(CommandError::GeneralCommandIssue(format!(
                                    // TODO translate error message
                                    "Block {name} does not exist."
                                )))
                            },
                            |block| Ok(Some(BlockPredicate::Block(block.id))),
                        )
                    },
                    |tag| {
                        get_tag_values(RegistryKey::Block, tag).map_or_else(
                            || {
                                Err(CommandError::GeneralCommandIssue(format!(
                                    // TODO translate error message
                                    "Tag {tag} does not exist."
                                )))
                            },
                            |blocks| {
                                let mut block_ids = Vec::with_capacity(blocks.len());
                                // TODO it will be slow to check name str, we should make a tag list of ids
                                for block_name in blocks {
                                    block_ids.push(get_block(block_name).unwrap().id);
                                }
                                Ok(Some(BlockPredicate::Tag(block_ids)))
                            },
                        )
                    },
                )
            }
            _ => Ok(None),
        }
    }
}

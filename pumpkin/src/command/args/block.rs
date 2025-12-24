use pumpkin_data::Block;
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::text::TextComponent;

use crate::command::args::ConsumeResult;
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
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::BlockState
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for BlockArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let block = args.pop();
        match block {
            Some(s) => Box::pin(async move { Some(Arg::Block(s)) }),
            None => Box::pin(async move { None }),
        }
    }
}

impl DefaultNameArgConsumer for BlockArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "block"
    }
}

impl<'a> FindArg<'a> for BlockArgumentConsumer {
    type Data = &'static Block;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Block(name)) => Block::from_name(name).map_or_else(
                || {
                    if name.starts_with("minecraft:") {
                        Err(CommandError::CommandFailed(TextComponent::translate(
                            "argument.block.id.invalid",
                            [TextComponent::text((*name).to_string())],
                        )))
                    } else {
                        Err(CommandError::CommandFailed(TextComponent::translate(
                            "argument.block.id.invalid",
                            [TextComponent::text("minecraft:".to_string() + *name)],
                        )))
                    }
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
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::BlockPredicate
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for BlockPredicateArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let block = args.pop();
        match block {
            Some(s) => Box::pin(async move { Some(Arg::BlockPredicate(s)) }),
            None => Box::pin(async move { None }),
        }
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
            Some(Arg::BlockPredicate(name)) => name.strip_prefix("#").map_or_else(
                || {
                    Block::from_name(name).map_or_else(
                        || {
                            if name.starts_with("minecraft:") {
                                Err(CommandError::CommandFailed(TextComponent::translate(
                                    "argument.block.id.invalid",
                                    [TextComponent::text((*name).to_string())],
                                )))
                            } else {
                                Err(CommandError::CommandFailed(TextComponent::translate(
                                    "argument.block.id.invalid",
                                    [TextComponent::text("minecraft:".to_string() + *name)],
                                )))
                            }
                        },
                        |block| Ok(Some(BlockPredicate::Block(block.id))),
                    )
                },
                |tag| {
                    get_tag_ids(RegistryKey::Block, tag).map_or_else(
                        || {
                            Err(CommandError::CommandFailed(TextComponent::translate(
                                "arguments.block.tag.unknown",
                                [TextComponent::text((*tag).to_string())],
                            )))
                        },
                        |blocks| Ok(Some(BlockPredicate::Tag(blocks.to_vec()))),
                    )
                },
            ),
            _ => Ok(None),
        }
    }
}

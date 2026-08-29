use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource::item::ItemArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["give"];

const DESCRIPTION: &str = "Give items to player(s).";

const ARG_ITEM: &str = "item";

const fn item_count_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("count")
        .min(1)
        .max(i32::MAX)
}

struct Executor;

impl CommandExecutor for Executor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;

        let (item_name, parsed_stack) = ItemArgumentConsumer::find_arg(args, ARG_ITEM)?;
        let item = parsed_stack.item;

        let item_count = match item_count_consumer().find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(count)) => count,
            Ok(Err(err)) => return Err(err.into()),
        };

        for target in targets {
            let max_stack = i32::from(parsed_stack.get_max_stack_size());
            let mut remaining = item_count;

            while remaining > 0 {
                let take = remaining.min(max_stack);
                let mut stack = parsed_stack.clone();
                stack.item_count = take as u8;
                target.inventory().insert_stack_anywhere(&mut stack);
                if !stack.is_empty() {
                    target.drop_item(stack);
                }
                remaining -= take;
            }
        }

        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_GIVE_SUCCESS_SINGLE,
                pumpkin_data::translation::bedrock::COMMANDS_GIVE_SUCCESS,
                [
                    TextComponent::text(item_count.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(item_count.min(99)),
                        }),
                    targets[0].get_display_name(),
                ],
            )
        } else {
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_GIVE_SUCCESS_MULTIPLE,
                pumpkin_data::translation::bedrock::COMMANDS_GIVE_SUCCESS,
                [
                    TextComponent::text(item_count.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(item_count.min(99)),
                        }),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        };
        sender.send_message(msg);

        Ok(targets.len() as i32)
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(PlayersArgumentConsumer).then(
            argument(ARG_ITEM, ItemArgumentConsumer)
                .execute(Executor)
                .then(argument_default_name(item_count_consumer()).execute(Executor)),
        ),
    )
}

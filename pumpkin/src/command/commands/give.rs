use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::hover::HoverEvent;

use crate::command::args::bounded_num::{BoundedNumArgumentConsumer, NotInBounds};
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource::item::ItemArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name};
use crate::command::{CommandError, CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["give"];

const DESCRIPTION: &str = "Give items to player(s).";

const ARG_ITEM: &str = "item";

fn item_count_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("count")
        .min(1)
        .max(i32::MAX)
}

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;

        let (item_name, item) = ItemArgumentConsumer::find_arg(args, ARG_ITEM)?;

        let item_count = match item_count_consumer().find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(count)) => count,
            Ok(Err(err)) => {
                let err_msg = match err {
                    NotInBounds::LowerBound(_, min) => {
                        format!("Can't give less than {min} of {item_name}")
                    }
                    NotInBounds::UpperBound(_, max) => {
                        format!("Can't give more than {max} of {item_name}")
                    }
                };

                sender
                    .send_message(TextComponent::text(err_msg).color(Color::Named(NamedColor::Red)))
                    .await;
                return Ok(());
            }
        };

        for target in targets {
            target.give_items(item.clone(), item_count as u32).await;
        }
        let msg = if targets.len() == 1 {
            TextComponent::translate(
                "commands.give.success.single",
                [
                    TextComponent::text(item_count.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(item_count),
                        }),
                    TextComponent::text(targets[0].gameprofile.name.to_string())
                        .hover_event(HoverEvent::show_entity(
                            targets[0].living_entity.entity.entity_uuid.to_string(),
                            targets[0]
                                .living_entity
                                .entity
                                .entity_type
                                .resource_name
                                .into(),
                            Some(TextComponent::text(targets[0].gameprofile.name.clone())),
                        ))
                        .click_event(ClickEvent::SuggestCommand {
                            command: format!("/tell {} ", targets[0].gameprofile.name.clone())
                                .into(),
                        }),
                ],
            )
        } else {
            TextComponent::translate(
                "commands.give.success.multiple",
                [
                    TextComponent::text(item_count.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(item_count),
                        }),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        };
        sender.send_message(msg).await;

        Ok(())
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

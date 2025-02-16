use async_trait::async_trait;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::TextComponent;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::item::ItemArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::tree::builder::{argument, argument_default_name};
use crate::command::tree::CommandTree;
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

struct GiveExecutor;

#[async_trait]
impl CommandExecutor for GiveExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;

        let (item_name, item) = ItemArgumentConsumer::find_arg(args, ARG_ITEM)?;

        let item_count = match item_count_consumer().find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(count)) => count,
            Ok(Err(())) => {
                sender
                    .send_message(
                        TextComponent::text("Item count is too large or too small.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        for target in targets {
            target.give_items(item, item_count as u32).await;
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
                            tag: None,
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
                        .click_event(ClickEvent::SuggestCommand(
                            format!("/tell {} ", targets[0].gameprofile.name.clone()).into(),
                        )),
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
                            tag: None,
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
                .execute(GiveExecutor)
                .then(argument_default_name(item_count_consumer()).execute(GiveExecutor)),
        ),
    )
}

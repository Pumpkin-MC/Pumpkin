use pumpkin_protocol::java::client::play::CClearTitle;
use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::entity::EntityBase;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{
            Arg, ConsumedArgs, FindArg, players::PlayersArgumentConsumer,
            textcomponent::TextComponentArgConsumer,
        },
        tree::CommandTree,
        tree::builder::{argument, literal},
    },
    entity::player::TitleMode,
};

const NAMES: [&str; 1] = ["title"];

const DESCRIPTION: &str = "Displays a title.";

const ARG_TARGETS: &str = "targets";

const ARG_TITLE: &str = "title";

/// bool: Whether to reset or not
struct ClearOrResetExecutor(bool);

impl CommandExecutor for ClearOrResetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };
            let reset = self.0;

            for target in targets {
                target.client.enqueue_packet(&CClearTitle::new(reset)).await;
            }
            sender
                .send_message(if targets.len() == 1 {
                    let text = if reset {
                        "commands.title.reset.single"
                    } else {
                        "commands.title.cleared.single"
                    };
                    TextComponent::translate(text, [targets[0].get_display_name().await])
                } else {
                    let text = if reset {
                        "commands.title.reset.multiple"
                    } else {
                        "commands.title.cleared.multiple"
                    };
                    TextComponent::translate(text, [TextComponent::text(targets.len().to_string())])
                })
                .await;

            Ok(())
        })
    }
}

struct TitleExecutor(TitleMode);

impl CommandExecutor for TitleExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let text = TextComponentArgConsumer::find_arg(args, ARG_TITLE)?;

            let mode = &self.0;

            for target in targets {
                target.show_title(&text, mode).await;
            }

            let mode_name = format!("{mode:?}").to_lowercase();
            sender
                .send_message(if targets.len() == 1 {
                    TextComponent::translate(
                        format!("commands.title.show.{mode_name}.single"),
                        [targets[0].get_display_name().await],
                    )
                } else {
                    TextComponent::translate(
                        format!("commands.title.show.{mode_name}.multiple"),
                        [TextComponent::text(targets.len().to_string())],
                    )
                })
                .await;

            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .then(literal("clear").execute(ClearOrResetExecutor(false)))
            .then(literal("reset").execute(ClearOrResetExecutor(true)))
            .then(
                literal("title").then(
                    argument(ARG_TITLE, TextComponentArgConsumer)
                        .execute(TitleExecutor(TitleMode::Title)),
                ),
            )
            .then(
                literal("subtitle").then(
                    argument(ARG_TITLE, TextComponentArgConsumer)
                        .execute(TitleExecutor(TitleMode::SubTitle)),
                ),
            )
            .then(
                literal("actionbar").then(
                    argument(ARG_TITLE, TextComponentArgConsumer)
                        .execute(TitleExecutor(TitleMode::ActionBar)),
                ),
            ),
        // TODO: times
    )
}

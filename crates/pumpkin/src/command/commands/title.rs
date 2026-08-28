use pumpkin_protocol::java::client::play::CClearTitle;
use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::command::args::time::TimeArgumentConsumer;
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
const ARG_FADE_IN: &str = "fadeIn";
const ARG_STAY: &str = "stay";
const ARG_FADE_OUT: &str = "fadeOut";
/// bool: Whether to reset or not
struct ClearOrResetExecutor(bool);

impl CommandExecutor for ClearOrResetExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Players(targets)) = args.get(ARG_TARGETS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
        };
        let reset = self.0;

        for target in targets {
            target.try_send_client_packet(&CClearTitle::new(reset));
        }
        sender.send_message(if targets.len() == 1 {
            let text = if reset {
                "commands.title.reset.single"
            } else {
                "commands.title.cleared.single"
            };
            TextComponent::translate_cross(text, text, [targets[0].get_display_name()])
        } else {
            let text = if reset {
                "commands.title.reset.multiple"
            } else {
                "commands.title.cleared.multiple"
            };
            TextComponent::translate_cross(
                text,
                text,
                [TextComponent::text(targets.len().to_string())],
            )
        });

        Ok(targets.len() as i32)
    }
}

struct TitleExecutor(TitleMode);

impl CommandExecutor for TitleExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Players(targets)) = args.get(ARG_TARGETS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let text = TextComponentArgConsumer::find_arg(args, ARG_TITLE)?;

        let mode = self.0;

        for target in targets {
            target.show_title(&text, &mode);
        }

        let mode_name = format!("{mode:?}").to_lowercase();
        sender.send_message(if targets.len() == 1 {
            TextComponent::translate_cross(
                format!("commands.title.show.{mode_name}.single"),
                format!("commands.title.show.{mode_name}.single"),
                [targets[0].get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                format!("commands.title.show.{mode_name}.multiple"),
                format!("commands.title.show.{mode_name}.multiple"),
                [TextComponent::text(targets.len().to_string())],
            )
        });

        Ok(targets.len() as i32)
    }
}

struct TimesTitleExecutor;

impl CommandExecutor for TimesTitleExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Players(targets)) = args.get(ARG_TARGETS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let fade_in = TimeArgumentConsumer::find_arg(args, ARG_FADE_IN)?;
        let stay = TimeArgumentConsumer::find_arg(args, ARG_STAY)?;
        let fade_out = TimeArgumentConsumer::find_arg(args, ARG_FADE_OUT)?;

        for target in targets {
            target.send_title_animation(fade_in, stay, fade_out);
        }

        sender.send_message(if targets.len() == 1 {
            TextComponent::translate_cross(
                "commands.title.times.single",
                "commands.title.times.single",
                [targets[0].get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                "commands.title.times.multiple",
                "commands.title.times.multiple",
                [TextComponent::text(targets.len().to_string())],
            )
        });

        Ok(targets.len() as i32)
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
            )
            .then(
                literal("times").then(
                    argument(ARG_FADE_IN, TimeArgumentConsumer::new()).then(
                        argument(ARG_STAY, TimeArgumentConsumer::new()).then(
                            argument(ARG_FADE_OUT, TimeArgumentConsumer::new())
                                .execute(TimesTitleExecutor),
                        ),
                    ),
                ),
            ),
    )
}

use async_trait::async_trait;
use futures::StreamExt;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::command::CommandTreeArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArgDefaultName};
use crate::command::dispatcher::CommandError;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::tree::builder::{argument, argument_default_name};
use crate::command::tree::{Command, CommandTree};
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 3] = ["help", "h", "?"];

const DESCRIPTION: &str = "Print a help message.";

const ARG_COMMAND: &str = "command";

const COMMANDS_PER_PAGE: i32 = 7;

fn page_number_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().name("page").min(1)
}

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::CommandTree(tree)) = args.get(&ARG_COMMAND) else {
            return Err(InvalidConsumption(Some(ARG_COMMAND.into())));
        };

        let command_names = tree.names.join(", /");
        let usage = format!("{tree}");
        let description = &tree.description;

        let header_text = format!(" Help - /{} ", tree.names[0]);

        let mut message = TextComponent::text("")
            .add_child(
                TextComponent::text("-".repeat((52 - header_text.len()) / 2) + " ")
                    .color_named(NamedColor::Yellow),
            )
            .add_child(TextComponent::text(header_text.clone()))
            .add_child(
                TextComponent::text(
                    " ".to_owned() + &"-".repeat((52 - header_text.len()) / 2) + "\n",
                )
                .color_named(NamedColor::Yellow),
            )
            .add_child(
                TextComponent::text("Command: ")
                    .color_named(NamedColor::Aqua)
                    .add_child(
                        TextComponent::text(format!("/{command_names}"))
                            .color_named(NamedColor::Gold)
                            .bold(),
                    )
                    .add_child(TextComponent::text("\n").color_named(NamedColor::White))
                    .click_event(ClickEvent::SuggestCommand {
                        command: format!("/{}", tree.names[0]).into(),
                    }),
            )
            .add_child(
                TextComponent::text("Description: ")
                    .color_named(NamedColor::Aqua)
                    .add_child(
                        TextComponent::text(format!("{description}\n"))
                            .color_named(NamedColor::White),
                    ),
            )
            .add_child(
                TextComponent::text("Usage: ")
                    .color_named(NamedColor::Aqua)
                    .add_child(
                        TextComponent::text(format!("{usage}\n")).color_named(NamedColor::White),
                    )
                    .click_event(ClickEvent::SuggestCommand {
                        command: format!("{tree}").into(),
                    }),
            );

        message =
            message.add_child(TextComponent::text("-".repeat(52)).color_named(NamedColor::Yellow));

        sender.send_message(message).await;

        Ok(())
    }
}

struct BaseHelpExecutor;

#[async_trait]
impl CommandExecutor for BaseHelpExecutor {
    #[expect(clippy::too_many_lines)]
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let page_number = match page_number_consumer().find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(number)) => number,
            Ok(Err(_)) => {
                sender
                    .send_message(
                        TextComponent::text("Invalid page number.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        let dispatcher = server.command_dispatcher.read().await;
        let commands: Vec<&CommandTree> = dispatcher
            .commands
            .values()
            .filter_map(|cmd| match cmd {
                Command::Tree(tree) => Some(tree),
                Command::Alias(_) => None,
            })
            .collect();

        let mut commands: Vec<&CommandTree> = futures::stream::iter(commands.iter())
            .filter(|tree| async {
                if let Some(perm) = dispatcher.permissions.get(&tree.names[0]) {
                    return sender.has_permission(perm.as_str()).await;
                }
                false
            })
            .collect()
            .await;

        commands.sort_by(|a, b| a.names[0].cmp(&b.names[0]));

        let total_pages = (commands.len() as i32 + COMMANDS_PER_PAGE - 1) / COMMANDS_PER_PAGE;
        let page = page_number.min(total_pages);

        let start = (page - 1) * COMMANDS_PER_PAGE;
        let end = start + COMMANDS_PER_PAGE;
        let page_commands = &commands[start as usize..end.min(commands.len() as i32) as usize];

        let arrow_left = if page > 1 {
            let cmd = format!("/help {}", page - 1);
            TextComponent::text("<<<")
                .color(Color::Named(NamedColor::Aqua))
                .click_event(ClickEvent::RunCommand {
                    command: cmd.into(),
                })
        } else {
            TextComponent::text("<<<").color(Color::Named(NamedColor::Gray))
        };

        let arrow_right = if page < total_pages {
            let cmd = format!("/help {}", page + 1);
            TextComponent::text(">>>")
                .color(Color::Named(NamedColor::Aqua))
                .click_event(ClickEvent::RunCommand {
                    command: cmd.into(),
                })
        } else {
            TextComponent::text(">>>").color(Color::Named(NamedColor::Gray))
        };

        let header_text = format!(" Help - Page {page}/{total_pages} ");

        let mut message = TextComponent::text("")
            .add_child(
                TextComponent::text("-".repeat((52 - header_text.len() - 3) / 2) + " ")
                    .color_named(NamedColor::Yellow),
            )
            .add_child(arrow_left.clone())
            .add_child(TextComponent::text(header_text.clone()))
            .add_child(arrow_right.clone())
            .add_child(
                TextComponent::text(
                    " ".to_owned() + &"-".repeat((52 - header_text.len() - 3) / 2) + "\n",
                )
                .color_named(NamedColor::Yellow),
            );

        for tree in page_commands {
            message = message.add_child(
                TextComponent::text("/".to_owned() + &tree.names.join(", /"))
                    .color_named(NamedColor::Gold)
                    .add_child(TextComponent::text(" - ").color_named(NamedColor::Yellow))
                    .add_child(
                        TextComponent::text(tree.description.clone() + "\n")
                            .color_named(NamedColor::White),
                    )
                    .add_child(TextComponent::text("    Usage: ").color_named(NamedColor::Yellow))
                    .add_child(
                        TextComponent::text(format!("{tree}")).color_named(NamedColor::White),
                    )
                    .add_child(TextComponent::text("\n").color_named(NamedColor::White))
                    .click_event(ClickEvent::SuggestCommand {
                        command: format!("/{}", tree.names[0]).into(),
                    }),
            );
        }

        let footer_text = format!(" Page {page}/{total_pages} ");
        message = message
            .add_child(
                TextComponent::text("-".repeat((52 - footer_text.len() - 3) / 2) + " ")
                    .color_named(NamedColor::Yellow),
            )
            .add_child(arrow_left)
            .add_child(TextComponent::text(footer_text.clone()))
            .add_child(arrow_right)
            .add_child(
                TextComponent::text(" ".to_owned() + &"-".repeat((52 - footer_text.len() - 3) / 2))
                    .color_named(NamedColor::Yellow),
            );

        sender.send_message(message).await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_COMMAND, CommandTreeArgumentConsumer).execute(Executor))
        .then(argument_default_name(page_number_consumer()).execute(BaseHelpExecutor))
        .execute(BaseHelpExecutor)
}

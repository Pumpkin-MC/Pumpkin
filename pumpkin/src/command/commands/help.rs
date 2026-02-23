use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, INTEGER_TOO_HIGH, INTEGER_TOO_LOW};
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult, Redirection};
use crate::command::string_reader::StringReader;
use pumpkin_protocol::java::client::play::StringProtoArgBehavior;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};

const FAILED_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new("commands.help.failed");

const DESCRIPTION: &str = "Print a help message.";
const PERMISSION: &str = "pumpkin:command.help";

const ARG: &str = "commandOrPage";

const COMMANDS_PER_PAGE: usize = 7;

enum HelpArgument {
    Command(String),
    Page(usize),
}

struct HelpArgumentType;
impl ArgumentType for HelpArgumentType {
    type Item = HelpArgument;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let reader_start = reader.cursor();

        match reader.read_int() {
            Ok(integer) => {
                if integer < 1 {
                    reader.set_cursor(reader_start);
                    Err(INTEGER_TOO_LOW.create(
                        reader,
                        TextComponent::text(integer.to_string()),
                        TextComponent::text("1"),
                    ))
                } else if let Ok(a) = integer.try_into() {
                    Ok(HelpArgument::Page(a))
                } else {
                    reader.set_cursor(reader_start);
                    Err(INTEGER_TOO_HIGH.create(
                        reader,
                        TextComponent::text(integer.to_string()),
                        TextComponent::text(usize::MAX.to_string()),
                    ))
                }
            }
            Err(error) => {
                // Hacky way to greedily parse the remaining text.
                // This can never fail.
                //
                // We use greedy phrases for now
                // as the `?` command as the argument
                // doesn't work for the unquoted word one.
                let text = StringArgumentType::GreedyPhrase.parse(reader)?;

                {
                    let mut integer_text = text.as_str();
                    if let Some(magnitude) = integer_text.strip_prefix("-") {
                        integer_text = magnitude;
                    }
                    if !integer_text.is_empty() && integer_text.chars().all(|c| c.is_ascii_digit())
                    {
                        // The number was too large/small to be parsed into an i32.
                        // Instead of parsing it as a command,
                        // we act like we parsed it like an integer.
                        reader.set_cursor(reader_start);
                        return Err(error);
                    }
                }

                Ok(HelpArgument::Command(text))
            }
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::String(StringProtoArgBehavior::GreedyPhrase)
    }
}

impl HelpCommandExecutor {
    fn create_help_command_with_given_page_number(
        page_number: usize,
        arrow: &'static str,
    ) -> TextComponent {
        let cmd = format!("/help {page_number}");
        TextComponent::text(arrow)
            .color(Color::Named(NamedColor::Aqua))
            .click_event(ClickEvent::RunCommand {
                command: cmd.into(),
            })
    }

    fn page<'a>(context: &'a CommandContext, page_number: usize) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.source.server();

            let dispatcher = server.command_dispatcher.read().await;
            let commands = dispatcher
                .get_all_permitted_commands_usage(&context.source)
                .await;

            let commands_available = commands.len();

            let total_pages = commands.len().div_ceil(COMMANDS_PER_PAGE);
            let page = page_number.min(total_pages);

            let start = (page - 1) * COMMANDS_PER_PAGE;
            let end = (start + COMMANDS_PER_PAGE).min(commands.len());

            let page_commands = commands.into_iter().skip(start).take(end - start);

            let arrow_left = if page > 1 {
                Self::create_help_command_with_given_page_number(page - 1, "<<<")
            } else {
                TextComponent::text("<<<").color(Color::Named(NamedColor::Gray))
            };

            let arrow_right = if page < total_pages {
                Self::create_help_command_with_given_page_number(page + 1, ">>>")
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

            for (command, (description, usage)) in page_commands {
                let command_declaration = format!("/{command}");
                message = message.add_child(
                    TextComponent::text(command_declaration.clone())
                        .color_named(NamedColor::Gold)
                        .add_child(TextComponent::text(" - ").color_named(NamedColor::Yellow))
                        .add_child(
                            TextComponent::text(description.to_owned() + "\n")
                                .color_named(NamedColor::White),
                        )
                        .add_child(
                            TextComponent::text("    Usage: ").color_named(NamedColor::Yellow),
                        )
                        .add_child(
                            TextComponent::text(usage.into_string()).color_named(NamedColor::White),
                        )
                        .add_child(TextComponent::text("\n").color_named(NamedColor::White))
                        .click_event(ClickEvent::SuggestCommand {
                            command: command_declaration.into(),
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
                    TextComponent::text(
                        " ".to_owned() + &"-".repeat((52 - footer_text.len() - 3) / 2),
                    )
                    .color_named(NamedColor::Yellow),
                );

            context.source.send_message(message).await;

            Ok(commands_available as i32)
        })
    }

    fn command<'a>(context: &'a CommandContext, command: &'a str) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let dispatcher = context.source.server().command_dispatcher.read().await;

            let Some((usage, description)) = dispatcher
                .get_permitted_command_usage(&context.source, command)
                .await
            else {
                return Err(FAILED_ERROR_TYPE.create_without_context());
            };

            let header_text = format!(" Help - /{command} ");

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
                            TextComponent::text(format!("/{command}"))
                                .color_named(NamedColor::Gold)
                                .bold(),
                        )
                        .add_child(TextComponent::text("\n").color_named(NamedColor::White))
                        .click_event(ClickEvent::SuggestCommand {
                            command: format!("/{command}").into(),
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
                            TextComponent::text(format!("{usage}\n"))
                                .color_named(NamedColor::White),
                        )
                        .click_event(ClickEvent::SuggestCommand {
                            command: command.to_string().into(),
                        }),
                );

            message = message
                .add_child(TextComponent::text("-".repeat(52)).color_named(NamedColor::Yellow));

            context.source.send_message(message).await;

            Ok(1)
        })
    }
}

struct HelpCommandExecutor;
impl CommandExecutor for HelpCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        let arg = context.get_argument(ARG).unwrap_or(&HelpArgument::Page(1));

        match arg {
            HelpArgument::Command(command) => Self::command(context, command),
            HelpArgument::Page(page_number) => Self::page(context, *page_number),
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry
        .register_permission(Permission::new(
            PERMISSION,
            DESCRIPTION,
            PermissionDefault::Allow,
        ))
        .expect("Permission should have registered successfully");

    let node = dispatcher.register(
        command("help", DESCRIPTION)
            .requires_permission(PERMISSION)
            .then(argument(ARG, HelpArgumentType).executes(HelpCommandExecutor))
            .executes(HelpCommandExecutor),
    );
    dispatcher.register(
        command("h", DESCRIPTION)
            .requires_permission(PERMISSION)
            // This redirects to the .then() calls in the above node.
            .redirect(Redirection::Local(node.into()))
            // This is for the no-argument execution.
            .executes(HelpCommandExecutor),
    );
    dispatcher.register(
        command("?", DESCRIPTION)
            .requires_permission(PERMISSION)
            // This redirects to the .then() calls in the above node.
            .redirect(Redirection::Local(node.into()))
            // This is for the no-argument execution.
            .executes(HelpCommandExecutor),
    );
}

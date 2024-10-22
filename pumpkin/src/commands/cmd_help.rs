use crate::commands::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::commands::dispatcher::{CommandDispatcher, InvalidTreeError};
use crate::commands::tree::{Command, CommandTree, ConsumedArgs, RawArgs};
use crate::commands::tree_builder::argument;
use crate::commands::CommandSender;
use crate::server::Server;

const NAMES: [&str; 3] = ["help", "h", "?"];

const DESCRIPTION: &str = "Print a help message.";

const ARG_COMMAND: &str = "command";

fn consume_arg_command(
    _src: &CommandSender,
    _server: &Server,
    _args: &mut RawArgs,
) -> Result<String, Option<String>> {
    //   let s = args.pop()?;

    // dispatcher.get_tree(s).ok().map(|tree| tree.names[0].into())
    // TODO
    Err(None)
}

fn parse_arg_command<'a>(
    consumed_args: &'a ConsumedArgs,
    dispatcher: &'a CommandDispatcher,
) -> Result<&'a CommandTree<'a>, InvalidTreeError> {
    let command_name = consumed_args
        .get(ARG_COMMAND)
        .ok_or(InvalidConsumptionError(None))?;

    dispatcher
        .get_tree(command_name)
        .map_err(|_| InvalidConsumptionError(Some(command_name.into())))
}

#[allow(unused_variables)]

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            argument(ARG_COMMAND, consume_arg_command).execute(&|sender, server, args| {
                let tree = parse_arg_command(args, &server.command_dispatcher)?;

                // sender.send_message(TextComponent::text(&format!(
                //     "{} - {} Usage: {}",
                //     tree.names.join("/"),
                //     tree.description,
                //     tree
                // )));

                Ok(())
            }),
        )
        .execute(&|sender, server, _args| {
            let mut keys: Vec<&str> = server.command_dispatcher.commands.keys().copied().collect();
            keys.sort_unstable();

            for key in keys {
                let Command::Tree(tree) = &server.command_dispatcher.commands[key] else {
                    continue;
                };

                // sender.send_message(TextComponent::text(&format!(
                //     "{} - {} Usage: {}",
                //     tree.names.join("/"),
                //     tree.description,
                //     tree
                // )));
            }

            Ok(())
        })
}

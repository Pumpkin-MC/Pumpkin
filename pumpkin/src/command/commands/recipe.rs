use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["recipe"];

const DESCRIPTION: &str = "Gives or takes player recipes.";

const ARG_TARGETS: &str = "targets";
const ARG_RECIPE: &str = "recipe";

struct GiveExecutor;

impl CommandExecutor for GiveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let recipe = SimpleArgConsumer::find_arg(args, ARG_RECIPE)?;

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RECIPE_GIVE_FAILED,
                    [],
                )));
            }

            // TODO: Send CRecipeBookAdd packets to unlock recipes for players
            // Handle recipe == "*" to give all recipes

            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                        [
                            TextComponent::text(recipe.to_string()),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                        [
                            TextComponent::text(recipe.to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
        })
    }
}

struct TakeExecutor;

impl CommandExecutor for TakeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let recipe = SimpleArgConsumer::find_arg(args, ARG_RECIPE)?;

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RECIPE_TAKE_FAILED,
                    [],
                )));
            }

            // TODO: Send CRecipeBookRemove packets to lock recipes for players
            // Handle recipe == "*" to take all recipes

            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                        [
                            TextComponent::text(recipe.to_string()),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                        [
                            TextComponent::text(recipe.to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("give").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(argument(ARG_RECIPE, SimpleArgConsumer).execute(GiveExecutor)),
            ),
        )
        .then(
            literal("take").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(argument(ARG_RECIPE, SimpleArgConsumer).execute(TakeExecutor)),
            ),
        )
}

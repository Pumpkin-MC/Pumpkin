use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
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
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let Some(Arg::Simple(recipe)) = args.get(ARG_RECIPE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_RECIPE.into())));
            };

            // TODO: Use `recipe == "*"` to give/take all recipes when protocol support is added

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RECIPE_GIVE_FAILED,
                    [],
                )));
            }

            // TODO: Send CRecipeBookAdd packets to unlock recipes for players
            // Currently, the recipe book protocol packets are not yet implemented.

            let count = targets.len() as i32;
            if count == 1 {
                let name = targets[0].get_name();
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                        [TextComponent::text(recipe.to_string()), name],
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
            Ok(count)
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
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let Some(Arg::Simple(recipe)) = args.get(ARG_RECIPE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_RECIPE.into())));
            };

            // TODO: Use `recipe == "*"` to give/take all recipes when protocol support is added

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RECIPE_TAKE_FAILED,
                    [],
                )));
            }

            // TODO: Send CRecipeBookRemove packets to lock recipes for players

            let count = targets.len() as i32;
            if count == 1 {
                let name = targets[0].get_name();
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                        [TextComponent::text(recipe.to_string()), name],
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
            Ok(count)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("give").then(
                argument(ARG_TARGETS, EntitiesArgumentConsumer)
                    .then(argument(ARG_RECIPE, SimpleArgConsumer).execute(GiveExecutor)),
            ),
        )
        .then(
            literal("take").then(
                argument(ARG_TARGETS, EntitiesArgumentConsumer)
                    .then(argument(ARG_RECIPE, SimpleArgConsumer).execute(TakeExecutor)),
            ),
        )
}

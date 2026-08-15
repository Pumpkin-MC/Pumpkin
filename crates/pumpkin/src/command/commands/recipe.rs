use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::SuggestionProvider;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_protocol::java::client::play::CRecipeBookAdd;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::future::Future;
use std::pin::Pin;

const DESCRIPTION: &str = "Gives or takes player recipes.";
const PERMISSION: &str = "minecraft:command.recipe";

static ERROR_RECIPE_NOT_FOUND: CommandErrorType<1> =
    CommandErrorType::new(translation::java::RECIPE_NOTFOUND, "Unknown recipe: %s");

fn get_recipe_id(recipe: &DynamicRecipe) -> String {
    match recipe {
        DynamicRecipe::Crafting(crafting) => match crafting {
            pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shaped { result, .. }
            | pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shapeless { result, .. } => {
                result.item_id.clone()
            }
        },
        DynamicRecipe::Cooking(cooking) => match cooking {
            pumpkin_protocol::codec::recipe::OwnedCookingRecipeType::Smelting(r)
            | pumpkin_protocol::codec::recipe::OwnedCookingRecipeType::Blasting(r)
            | pumpkin_protocol::codec::recipe::OwnedCookingRecipeType::Smoking(r)
            | pumpkin_protocol::codec::recipe::OwnedCookingRecipeType::CampfireCooking(r) => {
                r.recipe_id.clone()
            }
        },
    }
}

struct RecipeSuggestionProvider;

impl SuggestionProvider for RecipeSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        let server = context.source.server.clone();

        Box::pin(async move {
            builder = builder.suggest("*");
            if let Some(server) = server {
                let recipes = server.recipe_manager.get_dynamic_recipes_internal().await;
                for recipe in recipes {
                    let id = get_recipe_id(&recipe);
                    builder = builder.suggest(id);
                }
            }
            builder.build()
        })
    }
}

struct RecipeGiveExecutor;

impl CommandExecutor for RecipeGiveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, "targets").await?;
            let recipe_str = StringArgumentType::get(context, "recipe")?;

            let server = context.source.server.as_ref().ok_or_else(|| {
                ERROR_RECIPE_NOT_FOUND
                    .create_without_context(TextComponent::text(recipe_str.to_string()))
            })?;

            let all_recipes = server.recipe_manager.get_dynamic_recipes_internal().await;

            let is_all = recipe_str == "*";
            let matching_recipes = if is_all {
                all_recipes.clone()
            } else {
                all_recipes
                    .iter()
                    .filter(|r| {
                        let id = get_recipe_id(r);
                        id == recipe_str
                            || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            };

            if !is_all && matching_recipes.is_empty() {
                return Err(ERROR_RECIPE_NOT_FOUND
                    .create_without_context(TextComponent::text(recipe_str.to_string())));
            }

            let recipe_count = matching_recipes.len();

            for player in &targets {
                if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref() {
                    java_client
                        .send_packet_now(&CRecipeBookAdd::new(false, &matching_recipes))
                        .await;
                }
            }

            let recipe_count_str = recipe_count.to_string();
            if targets.len() == 1 {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                    translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                    [
                        TextComponent::text(recipe_count_str),
                        targets[0].get_display_name().await,
                    ],
                );
                context.source.send_feedback(msg, true).await;
            } else {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                    [
                        TextComponent::text(recipe_count_str),
                        TextComponent::text(targets.len().to_string()),
                    ],
                );
                context.source.send_feedback(msg, true).await;
            }

            Ok((targets.len() * recipe_count) as i32)
        })
    }
}

struct RecipeTakeExecutor;

impl CommandExecutor for RecipeTakeExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, "targets").await?;
            let recipe_str = StringArgumentType::get(context, "recipe")?;

            let server = context.source.server.as_ref().ok_or_else(|| {
                ERROR_RECIPE_NOT_FOUND
                    .create_without_context(TextComponent::text(recipe_str.to_string()))
            })?;

            let all_recipes = server.recipe_manager.get_dynamic_recipes_internal().await;

            let is_all = recipe_str == "*";

            let mut matched = false;
            let remaining_recipes = if is_all {
                matched = true;
                Vec::new()
            } else {
                all_recipes
                    .iter()
                    .filter(|r| {
                        let id = get_recipe_id(r);
                        let is_match = id == recipe_str
                            || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str;
                        if is_match {
                            matched = true;
                        }
                        !is_match
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            };

            if !matched {
                return Err(ERROR_RECIPE_NOT_FOUND
                    .create_without_context(TextComponent::text(recipe_str.to_string())));
            }

            let taken_count = if is_all {
                all_recipes.len()
            } else {
                all_recipes.len() - remaining_recipes.len()
            };

            for player in &targets {
                if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref() {
                    java_client
                        .send_packet_now(&CRecipeBookAdd::new(true, &remaining_recipes))
                        .await;
                }
            }

            let taken_count_str = taken_count.to_string();
            if targets.len() == 1 {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                    translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                    [
                        TextComponent::text(taken_count_str),
                        targets[0].get_display_name().await,
                    ],
                );
                context.source.send_feedback(msg, true).await;
            } else {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                    [
                        TextComponent::text(taken_count_str),
                        TextComponent::text(targets.len().to_string()),
                    ],
                );
                context.source.send_feedback(msg, true).await;
            }

            Ok((targets.len() * taken_count) as i32)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("recipe", DESCRIPTION)
        .requires(PERMISSION)
        .then(
            literal("give").then(
                argument("targets", EntityArgumentType::Players).then(
                    argument("recipe", StringArgumentType::SingleWord)
                        .suggests(RecipeSuggestionProvider)
                        .executes(RecipeGiveExecutor),
                ),
            ),
        )
        .then(
            literal("take").then(
                argument("targets", EntityArgumentType::Players).then(
                    argument("recipe", StringArgumentType::SingleWord)
                        .suggests(RecipeSuggestionProvider)
                        .executes(RecipeTakeExecutor),
                ),
            ),
        );

    dispatcher.register(builder);
}

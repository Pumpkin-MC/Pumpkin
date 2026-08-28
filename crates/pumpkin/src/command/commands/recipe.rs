use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::entity::EntityBase;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_COOKING, RECIPES_CRAFTING};
use pumpkin_data::translation;
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CRecipeBookAdd, CRecipeBookRemove};
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Gives or takes player recipes.";
const PERMISSION: &str = "minecraft:command.recipe";

static ERROR_RECIPE_NOT_FOUND: CommandErrorType<1> =
    CommandErrorType::new(translation::java::RECIPE_NOTFOUND, "Unknown recipe: %s");

fn get_recipe_id(recipe: &DynamicRecipe) -> String {
    match recipe {
        DynamicRecipe::Crafting(crafting) => match crafting {
            pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shaped {
                recipe_id,
                result,
                ..
            }
            | pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shapeless {
                recipe_id,
                result,
                ..
            } => recipe_id.clone().unwrap_or_else(|| result.item_id.clone()),
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
    ) -> SuggestionProviderResult {
        if let Some(server) = &context.source.server {
            let recipes = server.recipe_manager.get_dynamic_recipes_internal();
            for recipe in recipes {
                let id = get_recipe_id(&recipe);
                builder = builder.suggest(id);
            }
        }
        builder.build()
    }
}

struct RecipeGiveExecutor {
    all: bool,
}

impl CommandExecutor for RecipeGiveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let recipe_str = if self.all {
            "*".to_string()
        } else {
            context.get_argument::<Identifier>("recipe")?.to_string()
        };

        let server = context.source.server.as_ref().ok_or_else(|| {
            ERROR_RECIPE_NOT_FOUND.create_without_context(TextComponent::text(recipe_str.clone()))
        })?;

        let all_recipes = server.recipe_manager.get_dynamic_recipes_internal();

        let is_all = recipe_str == "*";
        let matching_recipes = if is_all {
            all_recipes
        } else {
            all_recipes
                .iter()
                .filter(|r| {
                    let id = get_recipe_id(r);
                    id == recipe_str || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        if !is_all && matching_recipes.is_empty() {
            return Err(ERROR_RECIPE_NOT_FOUND
                .create_without_context(TextComponent::text(recipe_str.clone())));
        }

        let recipe_count = matching_recipes.len();
        let packet = CRecipeBookAdd::new(false, &matching_recipes);
        for player in &targets {
            player.try_send_client_packet(&packet);
        }

        let recipe_count_str = recipe_count.to_string();
        if targets.len() == 1 {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                [
                    TextComponent::text(recipe_count_str),
                    targets[0].get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
        } else {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(recipe_count_str),
                    TextComponent::text(targets.len().to_string()),
                ],
            );
            context.source.send_feedback(msg, true);
        }

        Ok((targets.len() * recipe_count) as i32)
    }
}

struct RecipeTakeExecutor {
    all: bool,
}

impl CommandExecutor for RecipeTakeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let recipe_str = if self.all {
            "*".to_string()
        } else {
            context.get_argument::<Identifier>("recipe")?.to_string()
        };

        let server = context.source.server.as_ref().ok_or_else(|| {
            ERROR_RECIPE_NOT_FOUND.create_without_context(TextComponent::text(recipe_str.clone()))
        })?;

        let all_recipes = server.recipe_manager.get_dynamic_recipes_internal();

        let crafting_display_count = RECIPES_CRAFTING
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                )
            })
            .count();

        let dynamic_recipe_offset = crafting_display_count + RECIPES_COOKING.len();

        let is_all = recipe_str == "*";

        let recipe_ids_to_remove: Vec<VarInt> = if is_all {
            (0..(dynamic_recipe_offset + all_recipes.len()))
                .map(|id| VarInt(id as i32))
                .collect()
        } else {
            all_recipes
                .iter()
                .enumerate()
                .filter_map(|(idx, r)| {
                    let id = get_recipe_id(r);
                    let is_match = id == recipe_str
                        || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str;

                    is_match.then_some(VarInt((dynamic_recipe_offset + idx) as i32))
                })
                .collect()
        };

        if recipe_ids_to_remove.is_empty() {
            return Err(ERROR_RECIPE_NOT_FOUND
                .create_without_context(TextComponent::text(recipe_str.clone())));
        }

        let taken_count = recipe_ids_to_remove.len();

        let packet = CRecipeBookRemove::new(&recipe_ids_to_remove);
        for player in &targets {
            player.try_send_client_packet(&packet);
        }

        let taken_count_str = taken_count.to_string();
        if targets.len() == 1 {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                [
                    TextComponent::text(taken_count_str),
                    targets[0].get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
        } else {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(taken_count_str),
                    TextComponent::text(targets.len().to_string()),
                ],
            );
            context.source.send_feedback(msg, true);
        }

        Ok((targets.len() * taken_count) as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("recipe", DESCRIPTION)
        .requires(PERMISSION)
        .then(
            literal("give").then(
                argument("targets", EntityArgumentType::Players)
                    .then(literal("*").executes(RecipeGiveExecutor { all: true }))
                    .then(
                        argument("recipe", IdentifierArgumentType)
                            .suggests(RecipeSuggestionProvider)
                            .executes(RecipeGiveExecutor { all: false }),
                    ),
            ),
        )
        .then(
            literal("take").then(
                argument("targets", EntityArgumentType::Players)
                    .then(literal("*").executes(RecipeTakeExecutor { all: true }))
                    .then(
                        argument("recipe", IdentifierArgumentType)
                            .suggests(RecipeSuggestionProvider)
                            .executes(RecipeTakeExecutor { all: false }),
                    ),
            ),
        );

    dispatcher.register(builder);
}

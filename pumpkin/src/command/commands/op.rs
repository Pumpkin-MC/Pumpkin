use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::game_profile::GameProfileArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::data::SaveJSONConfiguration;
use pumpkin_config::op::Op;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

pub const ALREADY_OP_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_OP_FAILED,
    translation::bedrock::COMMANDS_OP_FAILED,
);

const DESCRIPTION: &str = "Grants operator status to a player.";
const PERMISSION: &str = "minecraft:command.op";
const ARG_TARGETS: &str = "targets";

struct OpCommandExecutor;

impl CommandExecutor for OpCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let profiles = GameProfileArgumentType::get(context, ARG_TARGETS).await?;

            let mut config = server.data.operator_config.write().await;
            let mut successes: i32 = 0;
            let new_level = server.basic_config.op_permission_level;

            for profile in profiles {
                let maybe_existing_entry = config.ops.iter_mut().find(|o| o.uuid == profile.id);

                if let Some(op) = maybe_existing_entry {
                    if op.level == new_level {
                        continue;
                    }

                    op.level = new_level;
                    op.name.clone_from(&profile.name);
                } else {
                    let op_entry = Op::new(profile.id, profile.name.clone(), new_level, false);
                    config.ops.push(op_entry);
                }

                if let Some(player) = server.get_player_by_uuid(profile.id) {
                    let command_dispatcher = server.command_dispatcher.read().await;
                    player
                        .set_permission_lvl(server, new_level, &command_dispatcher)
                        .await;
                }

                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_OP_SUCCESS,
                            translation::bedrock::COMMANDS_OP_SUCCESS,
                            [TextComponent::text(profile.name.clone())],
                        ),
                        true,
                    )
                    .await;

                successes += 1;
            }

            if successes > 0 {
                config.save();
            }

            if successes == 0 {
                Err(ALREADY_OP_ERROR_TYPE.create_without_context())
            } else {
                Ok(successes)
            }
        })
    }
}

struct OpSuggestionProvider;

impl SuggestionProvider for OpSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        Box::pin(async move {
            // Suggest every non-opped player.
            let ops = context.server().data.operator_config.read().await;
            for player in context.source.server().get_all_players() {
                if ops.ops.iter().all(|op| op.uuid != player.gameprofile.id) {
                    builder = builder.suggest(player.gameprofile.name.clone());
                }
            }
            builder.build()
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("op", DESCRIPTION).requires(PERMISSION).then(
            argument(ARG_TARGETS, GameProfileArgumentType)
                .suggests(OpSuggestionProvider)
                .executes(OpCommandExecutor),
        ),
    );
}

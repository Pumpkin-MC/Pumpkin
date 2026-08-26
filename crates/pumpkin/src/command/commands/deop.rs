use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::game_profile::{GameProfileArgumentType, GameProfileResult};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::data::SaveJSONConfiguration;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

pub const ALREADY_NOT_OP_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DEOP_FAILED,
    translation::bedrock::COMMANDS_DEOP_FAILED,
);

const NAME: &str = "deop";
const DESCRIPTION: &str = "Revokes operator status from a player.";
const PERMISSION: &str = "minecraft:command.deop";
const ARG_TARGETS: &str = "targets";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let result = context.get_argument::<GameProfileResult>(ARG_TARGETS)?;
            let targets = result.resolve(context.source.as_ref()).await?;

            let server = context.server();
            let mut config = server.data.operator_config.write().await;
            let mut succeeded_deops: i32 = 0;

            for profile in targets {
                if let Some(op_index) = config.ops.iter().position(|o| o.uuid == profile.id) {
                    config.ops.remove(op_index);
                    succeeded_deops += 1;

                if let Some(player) = server.get_player_by_uuid(profile.id)
                    && let Some(server_arc) = player.world().server.upgrade()
                {
                    let command_dispatcher = server_arc.command_dispatcher.load();
                        player
                            .set_permission_lvl(&server_arc, PermissionLvl::Zero, &command_dispatcher);
                }

                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_DEOP_SUCCESS,
                    translation::bedrock::COMMANDS_DEOP_SUCCESS,
                    [TextComponent::text(profile.name.clone())],
                );
                context.source.send_feedback(msg, true);
            }
        }

            if succeeded_deops <= 0 {
            Err(ALREADY_NOT_OP_ERROR_TYPE.create_without_context())
            } else {
                config.save();
            Ok(succeeded_deops)
        }
    }
}

struct OpSuggestionProvider;

impl SuggestionProvider for OpSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        Box::pin(async move {
            // Suggest every oped player.
            let ops = context.server().data.operator_config.read().await;
            let suggestions: Vec<&str> = ops.ops.iter().map(|op| op.name.as_str()).collect();
            builder.filter_and_suggest(&suggestions).build()
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command(NAME, DESCRIPTION).then(
            argument(ARG_TARGETS, GameProfileArgumentType)
                .suggests(OpSuggestionProvider)
                .executes(Executor),
        ),
    );
}

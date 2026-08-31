use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::game_profile::GameProfileArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::data::SaveJSONConfiguration;

const DESCRIPTION: &str = "Revokes operator status from a player.";
const PERMISSION: &str = "minecraft:command.deop";

const ERROR_DEOP_FAILED: CommandErrorType<0> = CommandErrorType::new(
    pumpkin_data::translation::java::COMMANDS_DEOP_FAILED,
    pumpkin_data::translation::bedrock::COMMANDS_DEOP_FAILED,
);

struct DeopExecutor;

impl CommandExecutor for DeopExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = GameProfileArgumentType::get(context, "targets")?;
        let server = context.source.server();
        let mut config = server.data.operator_config.write().unwrap();

        let mut succeeded_deops: i32 = 0;
        for profile in &targets {
            if let Some(op_index) = config.ops.iter().position(|o| o.uuid == profile.id) {
                config.ops.remove(op_index);
                succeeded_deops += 1;

                if let Some(player) = server.get_player_by_uuid(profile.id)
                    && let Some(server_arc) = player.world().server.upgrade()
                {
                    let command_dispatcher = server_arc.command_dispatcher.load();
                    player.set_permission_lvl(
                        &server_arc,
                        PermissionLvl::Zero,
                        &command_dispatcher,
                    );
                }

                let msg = TextComponent::translate_cross(
                    pumpkin_data::translation::java::COMMANDS_DEOP_SUCCESS,
                    pumpkin_data::translation::bedrock::COMMANDS_DEOP_SUCCESS,
                    [TextComponent::text(profile.name.clone())],
                );
                context.source.send_feedback(msg, true);
            }
        }

        if succeeded_deops > 0 {
            config.save();
        }

        if succeeded_deops == 0 {
            Err(ERROR_DEOP_FAILED.create_without_context())
        } else {
            Ok(succeeded_deops)
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("deop", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument("targets", GameProfileArgumentType).executes(DeopExecutor)),
    );
}

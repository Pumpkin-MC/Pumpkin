use std::net::IpAddr;
use std::str::FromStr;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::data::SaveJSONConfiguration;

const DESCRIPTION: &str = "unbans a ip";
const PERMISSION: &str = "minecraft:command.pardonip";

const ERROR_PARDONIP_INVALID: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PARDONIP_INVALID,
    translation::bedrock::COMMANDS_UNBANIP_INVALID,
);

const ERROR_PARDONIP_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PARDONIP_FAILED,
    translation::java::COMMANDS_PARDONIP_FAILED,
);

struct PardonIpExecutor;

impl CommandExecutor for PardonIpExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = StringArgumentType::get(context, "ip")?;
        let ip = IpAddr::from_str(target)
            .map_err(|_| ERROR_PARDONIP_INVALID.create_without_context())?;

        let server = context.source.server();
        let mut lock = server
            .data
            .banned_ip_list
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let result = lock
            .banned_ips
            .iter()
            .position(|entry| entry.ip == ip)
            .map_or_else(
                || Err(ERROR_PARDONIP_FAILED.create_without_context()),
                |idx| {
                    lock.banned_ips.remove(idx);
                    context.source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_PARDONIP_SUCCESS,
                            translation::bedrock::COMMANDS_UNBANIP_SUCCESS,
                            [TextComponent::text(ip.to_string())],
                        ),
                        true,
                    );
                    Ok(1)
                },
            );

        lock.save();

        result
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    let cmd = command("pardon-ip", DESCRIPTION)
        .requires(PERMISSION)
        .then(argument("ip", StringArgumentType::SingleWord).executes(PardonIpExecutor));

    dispatcher.register_with_aliases(cmd, &["pardonip"]);
}

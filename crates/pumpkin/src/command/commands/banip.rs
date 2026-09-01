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
use crate::data::banlist_serializer::BannedIpEntry;
use crate::net::DisconnectReason;
use crate::server::Server;

const DESCRIPTION: &str = "bans a player-ip";
const PERMISSION: &str = "minecraft:command.banip";

const ERROR_BANIP_INVALID: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_BANIP_INVALID,
    translation::bedrock::COMMANDS_BANIP_INVALID,
);

const ERROR_BANIP_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_BANIP_FAILED,
    translation::java::COMMANDS_BANIP_FAILED,
);

fn parse_ip(target: &str, server: &Server) -> Option<IpAddr> {
    IpAddr::from_str(target).ok().or_else(|| {
        server
            .get_player_by_name(target)
            .map(|p| p.client.address().ip())
    })
}

fn ban_ip(context: &CommandContext, target: &str, reason: Option<String>) -> CommandExecutorResult {
    let server = context.source.server();
    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());

    let target_ip =
        parse_ip(target, server).ok_or_else(|| ERROR_BANIP_INVALID.create_without_context())?;

    let mut banned_ips = server.data.banned_ip_list.write().unwrap();

    if banned_ips.get_entry(&target_ip).is_some() {
        return Err(ERROR_BANIP_FAILED.create_without_context());
    }

    banned_ips.banned_ips.push(BannedIpEntry::new(
        target_ip,
        context.source.name.clone(),
        None,
        reason.clone(),
    ));

    banned_ips.save();
    drop(banned_ips);

    context.source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_BANIP_SUCCESS,
            translation::bedrock::COMMANDS_BANIP_SUCCESS,
            [
                TextComponent::text(target_ip.to_string()),
                TextComponent::text(reason),
            ],
        ),
        true,
    );

    let players_to_kick: Vec<_> = server
        .get_all_players()
        .iter()
        .filter(|player| player.client.address().ip() == target_ip)
        .cloned()
        .collect();

    for player in players_to_kick {
        let kick_msg = TextComponent::translate_cross(
            translation::java::MULTIPLAYER_DISCONNECT_IP_BANNED,
            translation::bedrock::DISCONNECTIONSCREEN_TITLE_BANNEDBYHOST,
            [],
        );
        player.kick(DisconnectReason::Kicked, &kick_msg);
    }

    Ok(1)
}

struct BanIpExecutor {
    has_reason: bool,
}

impl CommandExecutor for BanIpExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = StringArgumentType::get(context, "ip")?;
        let reason = if self.has_reason {
            Some(StringArgumentType::get(context, "reason")?.to_string())
        } else {
            None
        };

        ban_ip(context, target, reason)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    let cmd = command("ban-ip", DESCRIPTION).requires(PERMISSION).then(
        argument("ip", StringArgumentType::SingleWord)
            .executes(BanIpExecutor { has_reason: false })
            .then(
                argument("reason", StringArgumentType::GreedyPhrase)
                    .executes(BanIpExecutor { has_reason: true }),
            ),
    );

    dispatcher.register_with_aliases(cmd, &["banip"]);
}

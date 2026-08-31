use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::game_profile::GameProfileArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::data::SaveJSONConfiguration;
use crate::data::banlist_serializer::BannedPlayerEntry;
use crate::net::{DisconnectReason, GameProfile};

const DESCRIPTION: &str = "bans a player";
const PERMISSION: &str = "minecraft:command.ban";

const ERROR_BAN_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_BAN_FAILED,
    translation::bedrock::COMMANDS_BAN_FAILED,
);

fn ban_profile(context: &CommandContext, profile: &GameProfile, reason: Option<String>) -> bool {
    let server = context.source.server();
    let mut banned_players = server.data.banned_player_list.write().unwrap();

    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());

    if let Some(entry) = banned_players
        .banned_players
        .iter_mut()
        .find(|entry| entry.uuid == profile.id)
    {
        if entry.name != profile.name {
            entry.name.clone_from(&profile.name);
            banned_players.save();
        }
        return false;
    }

    banned_players.banned_players.push(BannedPlayerEntry::new(
        profile,
        context.source.name.clone(),
        None,
        reason.clone(),
    ));

    banned_players.save();
    drop(banned_players);

    context.source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_BAN_SUCCESS,
            translation::bedrock::COMMANDS_BAN_SUCCESS,
            [
                TextComponent::text(profile.name.clone()),
                TextComponent::text(reason),
            ],
        ),
        true,
    );

    if let Some(player) = server.get_player_by_uuid(profile.id) {
        let kick_msg = TextComponent::translate_cross(
            translation::java::MULTIPLAYER_DISCONNECT_BANNED,
            translation::bedrock::DISCONNECTIONSCREEN_TITLE_BANNEDBYHOST,
            [],
        );
        player.kick(DisconnectReason::Kicked, &kick_msg);
    }

    true
}

struct BanExecutor {
    has_reason: bool,
}

impl CommandExecutor for BanExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = GameProfileArgumentType::get(context, "player")?;
        let reason = if self.has_reason {
            Some(StringArgumentType::get(context, "reason")?.to_string())
        } else {
            None
        };

        let mut count: usize = 0;
        for target in &targets {
            if ban_profile(context, target, reason.clone()) {
                count += 1;
            }
        }

        if count == 0 {
            Err(ERROR_BAN_FAILED.create_without_context())
        } else {
            Ok(count as i32)
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
        command("ban", DESCRIPTION).requires(PERMISSION).then(
            argument("player", GameProfileArgumentType)
                .executes(BanExecutor { has_reason: false })
                .then(
                    argument("reason", StringArgumentType::GreedyPhrase)
                        .executes(BanExecutor { has_reason: true }),
                ),
        ),
    );
}

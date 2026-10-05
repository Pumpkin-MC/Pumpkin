use std::sync::Arc;

use pumpkin_data::translation::java::{COMMANDS_LIST_NAMEANDID, COMMANDS_LIST_PLAYERS};
use pumpkin_util::{
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::TextComponent,
};

use crate::{
    command::{
        argument_builder::{ArgumentBuilder, command, literal},
        context::command_context::CommandContext,
        node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
    },
    entity::{EntityBase, player::Player},
};

const DESCRIPTION: &str = "Print the list of online players.";

const PERMISSION: &str = "minecraft:command.list";

enum ListMode {
    Names,
    Uuids,
}

struct ListCommandExecutor(ListMode);

impl CommandExecutor for ListCommandExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let players: Vec<Arc<Player>> = context.server().get_all_players();
        let players_len = players.len();

        let list = match self.0 {
            ListMode::Names => get_player_names(&players),
            ListMode::Uuids => get_player_names_and_ids(&players),
        };

        let max_players = context.source.output.as_player().map_or_else(
            || context.server().advanced_config.networking.java.max_players,
            |player| match player.client.as_ref() {
                crate::net::ClientPlatform::Java(_) => {
                    context.server().advanced_config.networking.java.max_players
                }
                crate::net::ClientPlatform::Bedrock(_) => {
                    context
                        .server()
                        .advanced_config
                        .networking
                        .bedrock
                        .max_players
                }
            },
        );

        context.source.send_feedback(
            TextComponent::translate_cross(
                COMMANDS_LIST_PLAYERS,
                COMMANDS_LIST_PLAYERS,
                [
                    TextComponent::text(players_len.to_string()),
                    TextComponent::text(max_players.to_string()),
                    list,
                ],
            ),
            false,
        );

        Ok(players_len as i32)
    }
}

fn get_player_names(players: &[Arc<Player>]) -> TextComponent {
    let display_names = players.iter().map(|p| p.get_display_name()).collect();
    TextComponent::join_with_comma(display_names)
}

fn get_player_names_and_ids(players: &[Arc<Player>]) -> TextComponent {
    let names_and_ids = players
        .iter()
        .map(|p| {
            TextComponent::translate_cross(
                COMMANDS_LIST_NAMEANDID,
                COMMANDS_LIST_NAMEANDID,
                &[
                    p.get_name(),
                    TextComponent::text(p.gameprofile.id.to_string()),
                ],
            )
        })
        .collect();
    TextComponent::join_with_comma(names_and_ids)
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Allow,
    ));

    dispatcher.register(
        command("list", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("uuids").executes(ListCommandExecutor(ListMode::Uuids)))
            .executes(ListCommandExecutor(ListMode::Names)),
    );
}

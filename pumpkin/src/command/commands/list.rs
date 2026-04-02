use std::sync::Arc;

use pumpkin_data::translation::{COMMANDS_LIST_NAMEANDID, COMMANDS_LIST_PLAYERS};
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
    entity::{EntityBase, EntityBaseFuture, player::Player},
};

const DESCRIPTION: &str = "Print the list of online players.";

const PERMISSION: &str = "minecraft:command.list";

enum ListMode {
    Names,
    Uuids,
}

struct ListCommandExecutor(ListMode);

impl CommandExecutor for ListCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let players: Vec<Arc<Player>> = context.server().get_all_players();
            let players_len = players.len();

            let list = match self.0 {
                ListMode::Names => get_player_names(&players).await,
                ListMode::Uuids => get_player_names_and_ids(&players),
            };

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        COMMANDS_LIST_PLAYERS,
                        [
                            TextComponent::text(players_len.to_string()),
                            TextComponent::text(
                                context.server().basic_config.max_players.to_string(),
                            ),
                            list,
                        ],
                    ),
                    false,
                )
                .await;

            Ok(players_len as i32)
        })
    }
}

async fn get_player_names(players: &[Arc<Player>]) -> TextComponent {
    let display_name_futures: Vec<EntityBaseFuture<'_, TextComponent>> =
        players.iter().map(|p| p.get_display_name()).collect();
    let display_names = futures::future::join_all(display_name_futures).await;
    TextComponent::join_with_comma(display_names)
}

fn get_player_names_and_ids(players: &[Arc<Player>]) -> TextComponent {
    let names_and_ids = players
        .iter()
        .map(|p| {
            TextComponent::translate(
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

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry
        .register_permission(Permission::new(
            PERMISSION,
            DESCRIPTION,
            PermissionDefault::Allow,
        ))
        .expect("Permission should have registered successfully");

    dispatcher.register(
        command("list", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("uuids").executes(ListCommandExecutor(ListMode::Uuids)))
            .executes(ListCommandExecutor(ListMode::Names)),
    );
}

use crate::command::argument_builder::{ArgumentBuilder, command, literal};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Prints the banlist of players, IPs, or both at once.";

const PERMISSION: &str = "minecraft:command.banlist";

struct BanListEntry {
    name: String,
    source: String,
    reason: String,
}

struct BanListCommandExecutor {
    players: bool,
    ips: bool,
}

impl CommandExecutor for BanListCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let mut entries = Vec::new();

            let data = &context.server().data;

            if self.players {
                let lock = data.banned_player_list.read().await;
                for entry in &lock.banned_players {
                    entries.push(BanListEntry {
                        name: entry.name.clone(),
                        source: entry.source.clone(),
                        reason: entry.reason.clone(),
                    });
                }
            }

            if self.ips {
                let lock = data.banned_ip_list.read().await;
                for entry in &lock.banned_ips {
                    entries.push(BanListEntry {
                        name: entry.ip.to_string(),
                        source: entry.source.clone(),
                        reason: entry.reason.clone(),
                    });
                }
            }

            let entries_len = entries.len() as i32;
            let source = &context.source;

            if entries.is_empty() {
                source
                    .send_feedback(
                        TextComponent::translate(translation::COMMANDS_BANLIST_NONE, []),
                        false,
                    )
                    .await;
            } else {
                source
                    .send_feedback(
                        TextComponent::translate(
                            translation::COMMANDS_BANLIST_LIST,
                            [TextComponent::text(entries.len().to_string())],
                        ),
                        false,
                    )
                    .await;

                for entry in entries {
                    source
                        .send_feedback(
                            TextComponent::translate(
                                translation::COMMANDS_BANLIST_ENTRY,
                                [
                                    TextComponent::text(entry.name),
                                    TextComponent::text(entry.source),
                                    TextComponent::text(entry.reason),
                                ],
                            ),
                            false,
                        )
                        .await;
                }
            }

            Ok(entries_len)
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
        command("banlist", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("ips").executes(BanListCommandExecutor {
                players: false,
                ips: true,
            }))
            .then(literal("players").executes(BanListCommandExecutor {
                players: true,
                ips: false,
            }))
            .executes(BanListCommandExecutor {
                players: true,
                ips: true,
            }),
    );
}

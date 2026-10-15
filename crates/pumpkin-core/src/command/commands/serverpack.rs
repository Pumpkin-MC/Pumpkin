use pumpkin_protocol::java::client::play::{CAddResourcePack, CRemoveResourcePack};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use uuid::Uuid;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::uuid::UuidArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Pushes or pops a resource pack to players.";
const PERMISSION: &str = "minecraft:command.serverpack";

enum ServerPackMode {
    PushUrl,
    PushUrlUuid,
    PushUrlUuidHash,
    Pop,
}

struct ServerPackExecutor(ServerPackMode);

impl CommandExecutor for ServerPackExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        match self.0 {
            ServerPackMode::PushUrl => {
                let url = StringArgumentType::get(context, "url")?;
                let uuid = Uuid::new_v3(&Uuid::nil(), url.as_bytes());
                let packet = CAddResourcePack::new(&uuid, url, "", false, None);
                context.server().broadcast_packet_all(&packet);
                context.source.send_feedback(
                    TextComponent::text(format!("Pushed resource pack: {url}")),
                    true,
                );
                Ok(1)
            }
            ServerPackMode::PushUrlUuid => {
                let url = StringArgumentType::get(context, "url")?;
                let uuid = UuidArgumentType::get(context, "uuid")?;
                let packet = CAddResourcePack::new(&uuid, url, "", false, None);
                context.server().broadcast_packet_all(&packet);
                context.source.send_feedback(
                    TextComponent::text(format!("Pushed resource pack: {url}")),
                    true,
                );
                Ok(1)
            }
            ServerPackMode::PushUrlUuidHash => {
                let url = StringArgumentType::get(context, "url")?;
                let uuid = UuidArgumentType::get(context, "uuid")?;
                let hash = StringArgumentType::get(context, "hash")?;
                let packet = CAddResourcePack::new(&uuid, url, hash, false, None);
                context.server().broadcast_packet_all(&packet);
                context.source.send_feedback(
                    TextComponent::text(format!("Pushed resource pack: {url}")),
                    true,
                );
                Ok(1)
            }
            ServerPackMode::Pop => {
                let uuid = UuidArgumentType::get(context, "uuid")?;
                let packet = CRemoveResourcePack::new(Some(&uuid));
                context.server().broadcast_packet_all(&packet);
                context.source.send_feedback(
                    TextComponent::text(format!("Popped resource pack: {uuid}")),
                    true,
                );
                Ok(1)
            }
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("serverpack", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("push").then(
                    argument("url", StringArgumentType::QuotablePhrase)
                        .then(
                            argument("uuid", UuidArgumentType)
                                .then(
                                    argument("hash", StringArgumentType::SingleWord)
                                        .executes(ServerPackExecutor(ServerPackMode::PushUrlUuidHash)),
                                )
                                .executes(ServerPackExecutor(ServerPackMode::PushUrlUuid)),
                        )
                        .executes(ServerPackExecutor(ServerPackMode::PushUrl)),
                ),
            )
            .then(
                literal("pop").then(
                    argument("uuid", UuidArgumentType)
                        .executes(ServerPackExecutor(ServerPackMode::Pop)),
                ),
            ),
    );
}

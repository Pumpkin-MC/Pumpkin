use pumpkin_protocol::{
    bedrock::client::CommandPermissionLevel,
    codec::var_int::VarInt,
    java::client::play::{
        ArgumentType, CCommands, ProtoNode, ProtoNodeType, StringProtoArgBehavior,
    },
};
use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::command::node::{
    attached::{AttachedNode, NodeId},
    dispatcher::CommandDispatcher,
    tree::ROOT_NODE_ID,
};
use crate::entity::player::Player;
use crate::plugin::player::player_command_send::PlayerCommandSendEvent;
use crate::server::Server;
use pumpkin_protocol::bedrock::client::available_commands::{
    CAvailableCommands, CommandData, EnumData, OverloadData, ParamData, arg_flags, arg_types,
};
use pumpkin_protocol::java::client::play::SuggestionProviders;

fn filter_sent_commands<'a>(
    player: &Arc<Player>,
    server: &Arc<Server>,
    names: impl Iterator<Item = &'a str>,
) -> Option<FxHashSet<String>> {
    if !server
        .plugin_manager
        .has_handlers::<PlayerCommandSendEvent>()
    {
        return None;
    }
    let mut event = PlayerCommandSendEvent::new(player.clone(), names.map(str::to_owned).collect());
    event = server.plugin_manager.fire_blocking(server, event);
    Some(event.commands.into_iter().collect())
}

fn wire_literal_name(literal: &str) -> &str {
    if literal.starts_with("//") {
        &literal[1..]
    } else {
        literal
    }
}

fn is_enabled_unshadowed_root_literal(
    dispatcher: &CommandDispatcher,
    lowercase: &str,
    literal: &str,
) -> bool {
    !dispatcher.is_disabled(lowercase)
        && (!literal.starts_with("//") || dispatcher.tree.get(&literal[1..]).is_none())
}

#[allow(clippy::too_many_lines)]
pub fn send_c_commands_packet(
    player: &Arc<Player>,
    server: &Arc<Server>,
    dispatcher: &CommandDispatcher,
) {
    let mut proto_nodes: Vec<ProtoNode> = Vec::with_capacity(dispatcher.tree.len());
    let source = &super::CommandSender::Player(player.clone()).into_source(server);
    for node in &dispatcher.tree {
        let children: Box<[VarInt]> = match node {
            AttachedNode::Root(_) => {
                // Drop disabled, unpermitted, and plugin-filtered commands from the
                // root's child list so they disappear from the client's command graph
                // (and tab-completion) entirely.
                let usable: Vec<(NodeId, Option<&str>)> = node
                    .children_ref()
                    .values()
                    .copied()
                    .filter_map(|id| {
                        let (lowercase, requirement, literal) = match &dispatcher.tree[id] {
                            AttachedNode::Literal(child) => (
                                child.meta.literal_lowercase.as_str(),
                                child.owned.requirements.evaluate(source),
                                child.meta.literal.as_ref(),
                            ),
                            AttachedNode::Command(child) => (
                                child.meta.literal_lowercase.as_str(),
                                child.owned.requirements.evaluate(source),
                                child.meta.literal.as_ref(),
                            ),
                            _ => return Some((id, None)),
                        };
                        (requirement
                            && is_enabled_unshadowed_root_literal(dispatcher, lowercase, literal))
                        .then(|| (id, Some(wire_literal_name(literal))))
                    })
                    .collect();

                let allowed = filter_sent_commands(
                    player,
                    server,
                    usable.iter().filter_map(|(_, name)| *name),
                );

                usable
                    .into_iter()
                    .filter(|(_, name)| match (&allowed, name) {
                        (Some(allowed), Some(name)) => allowed.contains(*name),
                        _ => true,
                    })
                    .map(|(id, _)| VarInt((id.0.get() - 1) as i32))
                    .collect()
            }
            _ => node
                .children_ref()
                .values()
                .copied()
                .map(|id| VarInt((id.0.get() - 1) as i32))
                .collect(),
        };

        let redirect_target = node
            .redirect()
            .and_then(|redirection| dispatcher.tree.resolve(redirection))
            .map(|id| (id.0.get() - 1) as i32);

        let satisfies_requirements = node.requirements().evaluate(source);

        match node {
            AttachedNode::Root(_) => {
                proto_nodes.push(ProtoNode {
                    children,
                    node_type: ProtoNodeType::Root,
                });
            }
            AttachedNode::Literal(literal_attached_node) => {
                let name = wire_literal_name(&literal_attached_node.meta.literal);
                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Literal {
                        name,
                        is_executable: literal_attached_node.owned.command.is_some(),
                        redirect_target,
                        restricted: !satisfies_requirements,
                    },
                };
                proto_nodes.push(node);
            }
            AttachedNode::Command(command_attached_node) => {
                let name = wire_literal_name(&command_attached_node.meta.literal);
                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Literal {
                        name,
                        is_executable: command_attached_node.owned.command.is_some(),
                        redirect_target,
                        restricted: !satisfies_requirements,
                    },
                };
                proto_nodes.push(node);
            }
            AttachedNode::Argument(argument_attached_node) => {
                let arg_type = &argument_attached_node.meta.argument_type;

                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Argument {
                        name: &argument_attached_node.meta.name,
                        is_executable: argument_attached_node.owned.command.is_some(),
                        parser: arg_type.client_side_parser(),
                        override_suggestion_type: if argument_attached_node
                            .meta
                            .suggestion_provider
                            .is_some()
                        {
                            Some(SuggestionProviders::AskServer)
                        } else {
                            arg_type.override_suggestion_providers()
                        },
                        redirect_target,
                        restricted: !satisfies_requirements,
                    },
                };
                proto_nodes.push(node);
            }
        }
    }

    let root_node_index = ROOT_NODE_ID.0.get() - 1;
    let packet = CCommands::new(proto_nodes.into(), VarInt(root_node_index as i32));
    player.try_send_client_packet(&packet);
}

struct BuilderContext<'a> {
    enum_values: &'a mut Vec<String>,
    enums: &'a mut Vec<EnumData>,
}

pub fn send_bedrock_commands_packet(
    player: &Arc<Player>,
    server: &Arc<Server>,
    dispatcher: &CommandDispatcher,
) {
    let mut enum_values: Vec<String> = Vec::new();
    let mut enums: Vec<EnumData> = Vec::new();
    let mut commands: Vec<CommandData> = Vec::new();

    let tree_nodes: Vec<&AttachedNode> = dispatcher.tree.iter().collect();

    let root_child_ids: Vec<NodeId> = tree_nodes
        .first()
        .and_then(|n| {
            if let AttachedNode::Root(_) = n {
                Some(n.children_ref().values().copied().collect())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let mut usable_commands = Vec::with_capacity(root_child_ids.len());
    for child_id in root_child_ids {
        let idx = child_id.0.get() - 1;
        let Some(node) = tree_nodes.get(idx) else {
            continue;
        };

        let (name, lowercase, is_executable, child_ids) = match node {
            AttachedNode::Literal(lit) => (
                lit.meta.literal.as_ref(),
                lit.meta.literal_lowercase.as_str(),
                lit.owned.command.is_some(),
                node.children_ref().values().copied().collect::<Vec<_>>(),
            ),
            AttachedNode::Command(cmd) => (
                cmd.meta.literal.as_ref(),
                cmd.meta.literal_lowercase.as_str(),
                cmd.owned.command.is_some(),
                node.children_ref().values().copied().collect::<Vec<_>>(),
            ),
            _ => continue,
        };

        if !is_enabled_unshadowed_root_literal(dispatcher, lowercase, name) {
            continue;
        }

        usable_commands.push((name, is_executable, child_ids));
    }

    let allowed = filter_sent_commands(
        player,
        server,
        usable_commands.iter().map(|(name, _, _)| *name),
    );

    for (name, is_executable, child_ids) in usable_commands {
        if allowed.as_ref().is_some_and(|set| !set.contains(name)) {
            continue;
        }

        let mut ctx = BuilderContext {
            enum_values: &mut enum_values,
            enums: &mut enums,
        };

        let overloads =
            build_overloads_from_attached_nodes(&tree_nodes, &child_ids, is_executable, &mut ctx);

        commands.push(CommandData {
            name: name.to_string(),
            description: String::new(),
            flags: 0,
            permission_level: CommandPermissionLevel::Any.into(),
            alias_enum: -1,
            command_data_chained_subcommand_indexes: Vec::new(),
            overloads,
        });
    }

    let packet = CAvailableCommands {
        enum_values,
        chained_subcommand_values: Vec::new(),
        post_fixes: Vec::new(),
        chained_subcommand_data: Vec::new(),
        enum_data: enums,
        commands,
        soft_enums: Vec::new(),
        constraints: Vec::new(),
    };

    if let crate::net::ClientPlatform::Bedrock(bedrock_client) = player.client.as_ref()
        && let Ok(data) = bedrock_client.serialize_packet(&packet)
    {
        bedrock_client.try_enqueue_packet(data);
    }
}

fn build_overloads_from_attached_nodes(
    tree: &[&AttachedNode],
    child_ids: &[NodeId],
    is_root_executable: bool,
    ctx: &mut BuilderContext,
) -> Vec<OverloadData> {
    let mut overloads = Vec::new();
    if is_root_executable {
        overloads.push(OverloadData {
            is_chaining: false,
            parameter_data: Vec::new(),
        });
    }
    collect_overloads_from_attached(tree, child_ids, &Vec::new(), &mut overloads, ctx);
    if overloads.is_empty() {
        overloads.push(OverloadData {
            is_chaining: false,
            parameter_data: Vec::new(),
        });
    }
    overloads
}

fn collect_overloads_from_attached(
    tree: &[&AttachedNode],
    child_ids: &[NodeId],
    current_params: &[ParamData],
    overloads: &mut Vec<OverloadData>,
    ctx: &mut BuilderContext,
) {
    for &child_id in child_ids {
        let idx = child_id.0.get() - 1;
        let Some(node) = tree.get(idx) else { continue };

        match node {
            AttachedNode::Literal(lit) => {
                let name = lit.meta.literal.as_ref();
                let enum_idx = ensure_command_enum(
                    ctx.enums,
                    ctx.enum_values,
                    &format!("SubCommand_{name}"),
                    &[name.to_string()],
                );
                let mut params = current_params.to_vec();
                params.push(ParamData {
                    name: name.to_string(),
                    parse_symbol: arg_flags::ARG_FLAG_VALID
                        | arg_flags::ARG_FLAG_ENUM
                        | enum_idx as u32,
                    is_optional: false,
                    options: 0,
                });
                let grandchild_ids: Vec<NodeId> = node.children_ref().values().copied().collect();
                if lit.owned.command.is_some() {
                    overloads.push(OverloadData {
                        is_chaining: false,
                        parameter_data: params.clone(),
                    });
                }
                collect_overloads_from_attached(tree, &grandchild_ids, &params, overloads, ctx);
            }
            AttachedNode::Command(cmd) => {
                let name = cmd.meta.literal.as_ref();
                let enum_idx = ensure_command_enum(
                    ctx.enums,
                    ctx.enum_values,
                    &format!("SubCommand_{name}"),
                    &[name.to_string()],
                );
                let mut params = current_params.to_vec();
                params.push(ParamData {
                    name: name.to_string(),
                    parse_symbol: arg_flags::ARG_FLAG_VALID
                        | arg_flags::ARG_FLAG_ENUM
                        | enum_idx as u32,
                    is_optional: false,
                    options: 0,
                });
                let grandchild_ids: Vec<NodeId> = node.children_ref().values().copied().collect();
                if cmd.owned.command.is_some() {
                    overloads.push(OverloadData {
                        is_chaining: false,
                        parameter_data: params.clone(),
                    });
                }
                collect_overloads_from_attached(tree, &grandchild_ids, &params, overloads, ctx);
            }
            AttachedNode::Argument(arg) => {
                let parser = arg.meta.argument_type.client_side_parser();
                let mut params = current_params.to_vec();
                params.push(ParamData {
                    name: arg.meta.name.to_string(),
                    parse_symbol: bedrock_param_type(&parser),
                    is_optional: false,
                    options: 0,
                });
                let grandchild_ids: Vec<NodeId> = node.children_ref().values().copied().collect();
                if arg.owned.command.is_some() {
                    overloads.push(OverloadData {
                        is_chaining: false,
                        parameter_data: params.clone(),
                    });
                }
                collect_overloads_from_attached(tree, &grandchild_ids, &params, overloads, ctx);
            }
            AttachedNode::Root(_) => {}
        }
    }
}

fn ensure_enum_value(enum_values: &mut Vec<String>, value: &str) -> u32 {
    let index = enum_values
        .iter()
        .position(|v| v == value)
        .unwrap_or_else(|| {
            enum_values.push(value.to_string());
            enum_values.len() - 1
        });
    index as u32
}

fn ensure_command_enum(
    enums: &mut Vec<EnumData>,
    enum_values: &mut Vec<String>,
    name: &str,
    values: &[String],
) -> usize {
    if let Some(pos) = enums.iter().position(|e| e.name == name) {
        return pos;
    }

    enums.push(EnumData {
        name: name.to_string(),
        values: values
            .iter()
            .map(|val| ensure_enum_value(enum_values, val))
            .collect(),
    });

    enums.len() - 1
}

const fn bedrock_param_type(arg: &ArgumentType) -> u32 {
    let base = match arg {
        ArgumentType::Integer { .. } | ArgumentType::Long { .. } | ArgumentType::Time { .. } => {
            arg_types::ARG_TYPE_INT
        }
        ArgumentType::Float { .. } | ArgumentType::Double { .. } => arg_types::ARG_TYPE_FLOAT,
        ArgumentType::Bool => arg_types::ARG_TYPE_INT,
        ArgumentType::Entity { .. }
        | ArgumentType::GameProfile
        | ArgumentType::ScoreHolder { .. } => arg_types::ARG_TYPE_TARGET,
        ArgumentType::BlockPos | ArgumentType::ColumnPos => arg_types::ARG_TYPE_BLOCK_POS,
        ArgumentType::Vec3 | ArgumentType::Vec2 | ArgumentType::Rotation | ArgumentType::Angle => {
            arg_types::ARG_TYPE_ENTITY_POS
        }
        ArgumentType::String(StringProtoArgBehavior::GreedyPhrase) => arg_types::ARG_TYPE_RAW_TEXT,
        ArgumentType::Message => arg_types::ARG_TYPE_MESSAGE,
        ArgumentType::IntRange | ArgumentType::FloatRange => arg_types::ARG_TYPE_INT_RANGE,
        ArgumentType::ItemSlot | ArgumentType::ItemSlots => arg_types::ARG_TYPE_EQUIPMENT_SLOT,
        ArgumentType::Component
        | ArgumentType::Style
        | ArgumentType::NbtCompound
        | ArgumentType::NbtTag
        | ArgumentType::NbtPath => arg_types::ARG_TYPE_JSON,
        ArgumentType::Operation => arg_types::ARG_TYPE_OPERATOR,
        // Default to STRING for non-converted types as it's the most compatible fallback.
        _ => arg_types::ARG_TYPE_STRING,
    };
    base | arg_flags::ARG_FLAG_VALID
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use arc_swap::ArcSwap;
    use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ConnectionState,
        java::{
            client::play::CCommands, packet_decoder::TCPNetworkDecoder, server::play::SChatCommand,
        },
        packet::MultiVersionJavaPacket,
        ser::NetworkReadExt,
    };
    use pumpkin_util::GameMode;
    use uuid::Uuid;

    use super::send_c_commands_packet;
    use crate::{
        command::{
            argument_builder::{ArgumentBuilder, command},
            context::command_context::CommandContext,
            node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
        },
        data::VanillaData,
        entity::player::Player,
        net::{
            ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
            java::{JavaClient, pending::PendingConnection},
        },
        plugin::{
            BoxFuture, EventPriority, EventRwHandler,
            player::{
                player_command_preprocess::PlayerCommandPreprocessEvent,
                player_command_send::PlayerCommandSendEvent,
            },
        },
        server::Server,
    };

    struct NoopExecutor;

    impl CommandExecutor for NoopExecutor {
        fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
            Ok(1)
        }
    }

    struct CountingExecutor(Arc<AtomicUsize>);

    impl CommandExecutor for CountingExecutor {
        fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(1)
        }
    }

    struct CommandFilterHandler {
        seen: Arc<Mutex<Vec<String>>>,
    }

    impl EventRwHandler<PlayerCommandSendEvent> for CommandFilterHandler {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerCommandSendEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                *self
                    .seen
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = event.commands.clone();
                event.commands.retain(|cmd| cmd != "hidden_cmd");
                event.commands.push("op_only_cmd".to_string());
                event.commands.push("nonexistent_cmd".to_string());
            })
        }
    }

    struct CommandPreprocessHandler;

    impl EventRwHandler<PlayerCommandPreprocessEvent> for CommandPreprocessHandler {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerCommandPreprocessEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.command == "blocked_cmd" {
                    event.cancelled = true;
                } else if event.command == "alias_cmd" {
                    event.command = "counted_cmd".to_string();
                }
            })
        }
    }

    async fn test_server_and_player() -> Result<
        (
            tempfile::TempDir,
            Arc<Server>,
            Arc<Player>,
            tokio::io::DuplexStream,
        ),
        Box<dyn std::error::Error>,
    > {
        let directory = tempfile::tempdir()?;
        let basic = BasicConfiguration {
            default_level_name: directory
                .path()
                .join("world")
                .to_string_lossy()
                .into_owned(),
            allow_nether: false,
            allow_end: false,
            ..Default::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.bedrock.online_mode = false;
        let server = Server::new(
            basic,
            advanced,
            TelemetryConfig::default(),
            VanillaData::default(),
            vec![],
        )
        .await?;
        let world = server.worlds.load().first().cloned().ok_or("No world")?;
        let (transport, peer) = tokio::io::duplex(65536);
        let (read, write) = tokio::io::split(transport);
        let pending = PendingConnection::from_transport(
            Box::new(read),
            Box::new(write),
            "127.0.0.1:25565".parse()?,
            0,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Arc::downgrade(&server),
        );
        let profile = GameProfile {
            id: Uuid::new_v4(),
            name: "CmdEventTest".into(),
            properties: ArcSwap::from_pointee(vec![]),
            profile_actions: None,
        };
        let mut client =
            JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        client.connection_state.store(ConnectionState::Play);
        client.start_outgoing_packet_task();
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        Ok((directory, server, player, peer))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn command_send_event_filters_root_commands_without_adding_unpermitted_ones()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, server, player, peer) = test_server_and_player().await?;

        let mut dispatcher = CommandDispatcher::default();
        dispatcher.register(command("allowed_cmd", "allowed").executes(NoopExecutor));
        dispatcher.register(command("hidden_cmd", "hidden").executes(NoopExecutor));
        dispatcher.register(
            command("op_only_cmd", "op")
                .requires("minecraft:command.op")
                .executes(NoopExecutor),
        );

        let seen = Arc::new(Mutex::new(Vec::new()));
        server.plugin_manager.register_blocking(
            Arc::new(CommandFilterHandler { seen: seen.clone() }),
            EventPriority::Normal,
            None,
        );

        send_c_commands_packet(&player, &server, &dispatcher);

        let seen_cmds = seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(seen_cmds, vec!["allowed_cmd", "hidden_cmd"]);

        let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
        let packet =
            tokio::time::timeout(Duration::from_secs(10), decoder.get_raw_packet()).await??;
        assert_eq!(packet.id, CCommands::to_id(CURRENT_MC_VERSION));

        let mut cursor = &packet.payload[..];
        let node_count = cursor.get_var_int()?.0;
        assert_eq!(node_count, 4);
        let root_flags = cursor.get_u8()?;
        assert_eq!(root_flags, 0);
        let root_children_len = cursor.get_var_int()?.0;
        assert_eq!(root_children_len, 1);
        let only_child_idx = cursor.get_var_int()?.0;
        assert_eq!(only_child_idx, 1);

        if let Some(client) = player.client.java() {
            client.close();
            client.await_tasks().await;
        }
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn command_preprocess_event_can_cancel_or_rewrite_commands()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, server, player, _peer) = test_server_and_player().await?;

        let executions = Arc::new(AtomicUsize::new(0));
        server.command_dispatcher.rcu(|dispatcher| {
            let mut next = (**dispatcher).clone();
            next.register(
                command("counted_cmd", "counted").executes(CountingExecutor(executions.clone())),
            );
            next.register(
                command("blocked_cmd", "blocked").executes(CountingExecutor(executions.clone())),
            );
            Arc::new(next)
        });

        server.plugin_manager.register_blocking(
            Arc::new(CommandPreprocessHandler),
            EventPriority::Normal,
            None,
        );

        let client = player.client.java().ok_or("No Java client")?;
        client
            .handle_chat_command(
                &player,
                &server,
                &SChatCommand {
                    command: "blocked_cmd",
                },
            )
            .await;
        assert_eq!(executions.load(Ordering::SeqCst), 0);

        client
            .handle_chat_command(
                &player,
                &server,
                &SChatCommand {
                    command: "alias_cmd",
                },
            )
            .await;
        assert_eq!(executions.load(Ordering::SeqCst), 1);

        client.close();
        client.await_tasks().await;
        Ok(())
    }
}

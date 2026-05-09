use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::client::play::{
        ArgumentType, CCommands, ProtoNode, ProtoNodeType, StringProtoArgBehavior,
    },
};
use std::sync::Arc;

use super::tree::{Node, NodeType};
use crate::command::node::{
    attached::{AttachedNode, NodeId},
    dispatcher::CommandDispatcher,
    tree::ROOT_NODE_ID,
};
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_protocol::bedrock::client::available_commands::{
    CAvailableCommands, Command, CommandEnum, CommandOverload, CommandParameter, arg_flags,
    arg_types, command_permissions,
};
use pumpkin_protocol::java::client::play::SuggestionProviders;

#[expect(clippy::too_many_lines)]
pub async fn send_c_commands_packet(
    player: &Arc<Player>,
    server: &Server,
    dispatcher: &CommandDispatcher,
) {
    let cmd_src = super::CommandSender::Player(player.clone());

    let mut first_level = Vec::new();

    let fallback_dispatcher = &dispatcher.fallback_dispatcher;
    for key in fallback_dispatcher.commands.keys() {
        let Ok(tree) = fallback_dispatcher.get_tree(key) else {
            continue;
        };

        let Some(permission) = fallback_dispatcher.permissions.get(key) else {
            continue;
        };

        if !cmd_src.has_permission(server, permission.as_str()).await {
            continue;
        }

        let (is_executable, child_nodes) =
            nodes_to_proto_node_builders(&cmd_src, &tree.nodes, &tree.children);

        let proto_node = ProtoNodeBuilder {
            child_nodes,
            node_type: ProtoNodeType::Literal {
                name: key,
                is_executable,
                redirect_target: None,
                restricted: false,
            },
        };

        first_level.push(proto_node);
    }

    let root = ProtoNodeBuilder {
        child_nodes: first_level,
        node_type: ProtoNodeType::Root,
    };

    let mut proto_nodes = Vec::new();
    let root_node_index = root.build(&mut proto_nodes);

    let node_id_offset = proto_nodes.len();

    let mut root_node_children_second: Box<[VarInt]> = Box::new([]);

    for node in &dispatcher.tree {
        let children: Box<[VarInt]> = node
            .children_ref()
            .values()
            .copied()
            .map(|id| resolve_node_id(id, node_id_offset, root_node_index))
            .map(|i| i.try_into().expect("i32 limit reached for ids"))
            .collect();

        let redirect_target = node
            .redirect()
            .and_then(|redirection| dispatcher.tree.resolve(redirection))
            .map(|id| resolve_node_id(id, node_id_offset, root_node_index))
            .map(|i| i.try_into().expect("i32 limit reached for ids"));

        let satisfies_requirements = true;

        match node {
            AttachedNode::Root(_) => {
                root_node_children_second = children;
            }
            AttachedNode::Literal(literal_attached_node) => {
                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Literal {
                        name: &literal_attached_node.meta.literal,
                        is_executable: literal_attached_node.owned.command.is_some(),
                        redirect_target,
                        restricted: !satisfies_requirements,
                    },
                };
                proto_nodes.push(node);
            }
            AttachedNode::Command(command_attached_node) => {
                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Literal {
                        name: &command_attached_node.meta.literal,
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

    if !root_node_children_second.is_empty() {
        let root_node = &mut proto_nodes[root_node_index];
        let mut first = std::mem::take(&mut root_node.children).into_vec();
        first.append(&mut root_node_children_second.into_vec());
        root_node.children = first.into_boxed_slice();
    }

    let packet = CCommands::new(proto_nodes.into(), VarInt(root_node_index as i32));
    player.client.enqueue_packet(&packet).await;
}

fn resolve_node_id(node_id: NodeId, node_id_offset: usize, root_node_index: usize) -> usize {
    if node_id == ROOT_NODE_ID {
        root_node_index
    } else {
        const FIRST_NONROOT_ID: usize = 2;
        debug_assert!(
            node_id.0.get() >= FIRST_NONROOT_ID,
            "Root node should have been handled in the if body"
        );
        node_id_offset + node_id.0.get() - FIRST_NONROOT_ID
    }
}

struct ProtoNodeBuilder<'a> {
    child_nodes: Vec<Self>,
    node_type: ProtoNodeType<'a>,
}

impl<'a> ProtoNodeBuilder<'a> {
    fn build(self, buffer: &mut Vec<ProtoNode<'a>>) -> usize {
        let children: Box<[VarInt]> = self
            .child_nodes
            .into_iter()
            .map(|node| {
                node.build(buffer)
                    .try_into()
                    .expect("Buffer index exceeded i32 bounds")
            })
            .collect();

        let i = buffer.len();
        buffer.push(ProtoNode {
            children,
            node_type: self.node_type,
        });
        i
    }
}

fn nodes_to_proto_node_builders<'a>(
    cmd_src: &super::CommandSender,
    nodes: &'a [Node],
    children: &[usize],
) -> (bool, Vec<ProtoNodeBuilder<'a>>) {
    let mut child_nodes = Vec::new();
    let mut is_executable = false;

    for i in children {
        let node = &nodes[*i];
        match &node.node_type {
            NodeType::Argument { name, consumer } => {
                let (node_is_executable, node_children) =
                    nodes_to_proto_node_builders(cmd_src, nodes, &node.children);
                child_nodes.push(ProtoNodeBuilder {
                    child_nodes: node_children,
                    node_type: ProtoNodeType::Argument {
                        name,
                        is_executable: node_is_executable,
                        redirect_target: None,
                        parser: consumer.get_client_side_parser(),
                        override_suggestion_type: consumer
                            .get_client_side_suggestion_type_override(),
                        restricted: false,
                    },
                });
            }

            NodeType::Literal { string, .. } => {
                let (node_is_executable, node_children) =
                    nodes_to_proto_node_builders(cmd_src, nodes, &node.children);
                child_nodes.push(ProtoNodeBuilder {
                    child_nodes: node_children,
                    node_type: ProtoNodeType::Literal {
                        name: string,
                        is_executable: node_is_executable,
                        redirect_target: None,
                        restricted: false,
                    },
                });
            }

            NodeType::ExecuteLeaf { .. } => is_executable = true,

            NodeType::Require { predicate } => {
                if predicate(cmd_src) {
                    let (node_is_executable, node_children) =
                        nodes_to_proto_node_builders(cmd_src, nodes, &node.children);
                    if node_is_executable {
                        is_executable = true;
                    }
                    child_nodes.extend(node_children);
                }
            }
        }
    }

    (is_executable, child_nodes)
}

struct BuilderContext<'a> {
    enum_values: &'a mut Vec<String>,
    enums: &'a mut Vec<CommandEnum>,
}

pub async fn send_bedrock_commands_packet(
    player: &Arc<Player>,
    server: &Server,
    dispatcher: &CommandDispatcher,
) {
    let cmd_src = super::CommandSender::Player(player.clone());

    let mut enum_values: Vec<String> = Vec::new();
    let mut enums: Vec<CommandEnum> = Vec::new();
    let mut commands: Vec<Command> = Vec::new();

    let fallback_dispatcher = &dispatcher.fallback_dispatcher;
    for key in fallback_dispatcher.commands.keys() {
        let Ok(tree) = fallback_dispatcher.get_tree(key) else {
            continue;
        };

        let Some(permission) = fallback_dispatcher.permissions.get(key) else {
            continue;
        };

        if !cmd_src.has_permission(server, permission.as_str()).await {
            continue;
        }

        let mut ctx = BuilderContext {
            enum_values: &mut enum_values,
            enums: &mut enums,
        };

        let overloads = build_overloads_from_nodes(&tree.nodes, &tree.children, &mut ctx);

        commands.push(Command {
            name: key.clone(),
            description: String::new(),
            flags: 0,
            permission: command_permissions::ANY.to_string(),
            aliases_enum_index: -1,
            chained_subcommand_offsets: Vec::new(),
            overloads,
        });
    }

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

    for child_id in root_child_ids {
        let idx = child_id.0.get() - 1;
        let Some(node) = tree_nodes.get(idx) else {
            continue;
        };

        let (name, is_executable, child_ids) = match node {
            AttachedNode::Literal(lit) => (
                lit.meta.literal.to_string(),
                lit.owned.command.is_some(),
                node.children_ref().values().copied().collect::<Vec<_>>(),
            ),
            AttachedNode::Command(cmd) => (
                cmd.meta.literal.to_string(),
                cmd.owned.command.is_some(),
                node.children_ref().values().copied().collect::<Vec<_>>(),
            ),
            _ => continue,
        };

        let mut ctx = BuilderContext {
            enum_values: &mut enum_values,
            enums: &mut enums,
        };

        let overloads =
            build_overloads_from_attached_nodes(&tree_nodes, &child_ids, is_executable, &mut ctx);

        commands.push(Command {
            name,
            description: String::new(),
            flags: 0,
            permission: command_permissions::ANY.to_string(),
            aliases_enum_index: -1,
            chained_subcommand_offsets: Vec::new(),
            overloads,
        });
    }

    let packet = CAvailableCommands {
        enum_values,
        chained_subcommand_values: Vec::new(),
        suffixes: Vec::new(),
        chained_subcommands: Vec::new(),
        enums,
        commands,
        soft_enums: Vec::new(),
        constraints: Vec::new(),
    };

    if let crate::net::ClientPlatform::Bedrock(bedrock_client) = &player.client {
        bedrock_client.send_game_packet(&packet).await;
    }
}

fn build_overloads_from_nodes(
    nodes: &[Node],
    children: &[usize],
    ctx: &mut BuilderContext,
) -> Vec<CommandOverload> {
    let mut overloads = Vec::new();
    collect_overloads_from_nodes(nodes, children, &mut Vec::new(), &mut overloads, ctx);
    if overloads.is_empty() {
        overloads.push(CommandOverload {
            chaining: false,
            parameters: Vec::new(),
        });
    }
    overloads
}

fn collect_overloads_from_nodes(
    nodes: &[Node],
    children: &[usize],
    current_params: &mut Vec<CommandParameter>,
    overloads: &mut Vec<CommandOverload>,
    ctx: &mut BuilderContext,
) {
    let mut has_executable = false;

    for &i in children {
        let node = &nodes[i];
        match &node.node_type {
            NodeType::ExecuteLeaf { .. } => {
                has_executable = true;
            }
            NodeType::Literal { string, .. } => {
                let enum_idx = ensure_command_enum(
                    ctx.enums,
                    ctx.enum_values,
                    &format!("SubCommand_{string}"),
                    std::slice::from_ref(string),
                );
                let mut params = current_params.clone();
                params.push(CommandParameter {
                    name: string.clone(),
                    type_info: arg_flags::ARG_FLAG_VALID
                        | arg_flags::ARG_FLAG_ENUM
                        | enum_idx as u32,
                    optional: false,
                    options: 0,
                });
                collect_overloads_from_nodes(nodes, &node.children, &mut params, overloads, ctx);
            }
            NodeType::Argument { name, consumer } => {
                let mut params = current_params.clone();
                params.push(CommandParameter {
                    name: name.clone(),
                    type_info: bedrock_param_type(&consumer.get_client_side_parser()),
                    optional: false,
                    options: 0,
                });
                collect_overloads_from_nodes(nodes, &node.children, &mut params, overloads, ctx);
            }
            NodeType::Require { .. } => {
                collect_overloads_from_nodes(nodes, &node.children, current_params, overloads, ctx);
            }
        }
    }

    if has_executable {
        overloads.push(CommandOverload {
            chaining: false,
            parameters: current_params.clone(),
        });
    }
}

fn build_overloads_from_attached_nodes(
    tree: &[&AttachedNode],
    child_ids: &[NodeId],
    is_root_executable: bool,
    ctx: &mut BuilderContext,
) -> Vec<CommandOverload> {
    let mut overloads = Vec::new();
    if is_root_executable {
        overloads.push(CommandOverload {
            chaining: false,
            parameters: Vec::new(),
        });
    }
    collect_overloads_from_attached(tree, child_ids, &Vec::new(), &mut overloads, ctx);
    if overloads.is_empty() {
        overloads.push(CommandOverload {
            chaining: false,
            parameters: Vec::new(),
        });
    }
    overloads
}

fn collect_overloads_from_attached(
    tree: &[&AttachedNode],
    child_ids: &[NodeId],
    current_params: &[CommandParameter],
    overloads: &mut Vec<CommandOverload>,
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
                params.push(CommandParameter {
                    name: name.to_string(),
                    type_info: arg_flags::ARG_FLAG_VALID
                        | arg_flags::ARG_FLAG_ENUM
                        | enum_idx as u32,
                    optional: false,
                    options: 0,
                });
                let grandchild_ids: Vec<NodeId> = node.children_ref().values().copied().collect();
                if lit.owned.command.is_some() {
                    overloads.push(CommandOverload {
                        chaining: false,
                        parameters: params.clone(),
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
                params.push(CommandParameter {
                    name: name.to_string(),
                    type_info: arg_flags::ARG_FLAG_VALID
                        | arg_flags::ARG_FLAG_ENUM
                        | enum_idx as u32,
                    optional: false,
                    options: 0,
                });
                let grandchild_ids: Vec<NodeId> = node.children_ref().values().copied().collect();
                if cmd.owned.command.is_some() {
                    overloads.push(CommandOverload {
                        chaining: false,
                        parameters: params.clone(),
                    });
                }
                collect_overloads_from_attached(tree, &grandchild_ids, &params, overloads, ctx);
            }
            AttachedNode::Argument(arg) => {
                let parser = arg.meta.argument_type.client_side_parser();
                let mut params = current_params.to_vec();
                params.push(CommandParameter {
                    name: arg.meta.name.to_string(),
                    type_info: bedrock_param_type(&parser),
                    optional: false,
                    options: 0,
                });
                let grandchild_ids: Vec<NodeId> = node.children_ref().values().copied().collect();
                if arg.owned.command.is_some() {
                    overloads.push(CommandOverload {
                        chaining: false,
                        parameters: params.clone(),
                    });
                }
                collect_overloads_from_attached(tree, &grandchild_ids, &params, overloads, ctx);
            }
            AttachedNode::Root(_) => {}
        }
    }
}

fn ensure_enum_value(enum_values: &mut Vec<String>, value: &str) -> usize {
    enum_values
        .iter()
        .position(|v| v == value)
        .unwrap_or_else(|| {
            enum_values.push(value.to_string());
            enum_values.len() - 1
        })
}

fn ensure_command_enum(
    enums: &mut Vec<CommandEnum>,
    enum_values: &mut Vec<String>,
    name: &str,
    values: &[String],
) -> usize {
    if let Some(pos) = enums.iter().position(|e| e.name == name) {
        return pos;
    }

    let value_indices: Vec<usize> = values
        .iter()
        .map(|val| ensure_enum_value(enum_values, val))
        .collect();

    enums.push(CommandEnum {
        name: name.to_string(),
        value_indices,
    });

    enums.len() - 1
}

const fn bedrock_param_type(arg: &ArgumentType<'_>) -> u32 {
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
        ArgumentType::IntRange => arg_types::ARG_TYPE_INT_RANGE,
        ArgumentType::ItemSlot | ArgumentType::ItemSlots => arg_types::ARG_TYPE_EQUIPMENT_SLOT,
        ArgumentType::Component
        | ArgumentType::Style
        | ArgumentType::Nbt
        | ArgumentType::NbtTag
        | ArgumentType::NbtPath => arg_types::ARG_TYPE_JSON,
        ArgumentType::Operation => arg_types::ARG_TYPE_OPERATOR,
        // Default to STRING for non-converted types as it's the most compatible fallback.
        _ => arg_types::ARG_TYPE_STRING,
    };
    base | arg_flags::ARG_FLAG_VALID
}

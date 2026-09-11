use pumpkin_protocol::{
    bedrock::client::CommandPermissionLevel,
    codec::var_int::VarInt,
    java::client::play::{
        ArgumentType, CCommands, ProtoNode, ProtoNodeType, StringProtoArgBehavior,
    },
};
use std::sync::Arc;

use crate::command::node::{
    attached::{AttachedNode, NodeId},
    dispatcher::CommandDispatcher,
    tree::ROOT_NODE_ID,
};
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_protocol::bedrock::client::available_commands::{
    CAvailableCommands, CommandData, EnumData, OverloadData, ParamData, arg_flags, arg_types,
};
use pumpkin_protocol::java::client::play::SuggestionProviders;

#[allow(clippy::too_many_lines)]
pub fn send_c_commands_packet(
    player: &Arc<Player>,
    _server: &Server,
    dispatcher: &CommandDispatcher,
) {
    let mut proto_nodes: Vec<ProtoNode> = Vec::with_capacity(dispatcher.tree.len());

    for node in &dispatcher.tree {
        let children: Box<[VarInt]> = match node {
            AttachedNode::Root(_) => {
                // Drop disabled commands from the root's child list so they
                // disappear from the client's command graph (and tab-completion)
                // entirely.
                node.children_ref()
                    .values()
                    .copied()
                    .filter(|id| {
                        let (disabled, name) = match &dispatcher.tree[*id] {
                            AttachedNode::Literal(child) => (
                                dispatcher.is_disabled(&child.meta.literal_lowercase),
                                child.meta.literal.as_ref(),
                            ),
                            AttachedNode::Command(child) => (
                                dispatcher.is_disabled(&child.meta.literal_lowercase),
                                child.meta.literal.as_ref(),
                            ),
                            _ => (false, ""),
                        };
                        if disabled {
                            return false;
                        }
                        if name.starts_with("//") && dispatcher.tree.get(&name[1..]).is_some() {
                            return false;
                        }
                        true
                    })
                    .map(|id| VarInt((id.0.get() - 1) as i32))
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

        let satisfies_requirements = true;

        match node {
            AttachedNode::Root(_) => {
                proto_nodes.push(ProtoNode {
                    children,
                    node_type: ProtoNodeType::Root,
                });
            }
            AttachedNode::Literal(literal_attached_node) => {
                let name = if literal_attached_node.meta.literal.starts_with("//") {
                    &literal_attached_node.meta.literal[1..]
                } else {
                    &literal_attached_node.meta.literal
                };
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
                let name = if command_attached_node.meta.literal.starts_with("//") {
                    &command_attached_node.meta.literal[1..]
                } else {
                    &command_attached_node.meta.literal
                };
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
    _server: &Server,
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

        if dispatcher.is_disabled(&name.to_ascii_lowercase()) {
            continue;
        }

        if name.starts_with("//") && dispatcher.tree.get(&name[1..]).is_some() {
            continue;
        }

        let mut ctx = BuilderContext {
            enum_values: &mut enum_values,
            enums: &mut enums,
        };

        let overloads =
            build_overloads_from_attached_nodes(&tree_nodes, &child_ids, is_executable, &mut ctx);

        commands.push(CommandData {
            name,
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

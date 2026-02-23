use std::sync::Arc;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::{CCommands, ProtoNode, ProtoNodeType}};

use crate::{command::node::{attached::{AttachedNode, NodeId}, dispatcher::CommandDispatcher, tree::ROOT_NODE_ID}, entity::player::Player};

use super::{
    tree::{Node, NodeType},
};

pub async fn send_c_commands_packet(
    player: &Arc<Player>,
    server: &crate::server::Server,
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

    // We can finally assign indices from our tree:
    // ID = 2: node_id_offset
    // ID = 3: node_id_offset + 1
    // ID = 4: node_id_offset + 2 ...
    let mut root_node_children_second: Box<[VarInt]> = Box::new([]);
    for node in &dispatcher.tree {
        // We map IDs to the indexes:
        let children: Box<[VarInt]> =
            node.children_ref()
                .values()
                .cloned()
                .map(|id| resolve_node_id(id, node_id_offset, root_node_index))
                .map(|i| i.try_into().expect("i32 limit reached for ids"))
                .collect();

        let redirect_target =
            node.redirect()
                .and_then(|redirection| dispatcher.tree.resolve(redirection))
                .map(|id| resolve_node_id(id, node_id_offset, root_node_index))
                .map(|i| i.try_into().expect("i32 limit reached for ids"));

        match node {
            AttachedNode::Root(_) => {
                // We skip the root node because we already have a root node.
                // We do need to capture its children though, for later.
                root_node_children_second = children;
            },
            AttachedNode::Literal(literal_attached_node) => {
                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Literal {
                        name: &literal_attached_node.meta.literal,
                        is_executable: literal_attached_node.owned.command.is_some(),
                        redirect_target
                    },
                };
                proto_nodes.push(node);
            },
            AttachedNode::Command(command_attached_node) => {
                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Literal {
                        name: &command_attached_node.meta.literal,
                        is_executable: command_attached_node.owned.command.is_some(),
                        redirect_target,
                    },
                };
                proto_nodes.push(node);
            },
            AttachedNode::Argument(argument_attached_node) => {
                let arg_type = &argument_attached_node.meta.argument_type;

                let node = ProtoNode {
                    children,
                    node_type: ProtoNodeType::Argument {
                        name: &argument_attached_node.meta.name,
                        is_executable: argument_attached_node.owned.command.is_some(),
                        parser: arg_type.client_side_parser(),
                        override_suggestion_type: arg_type.override_suggestion_providers(),
                        redirect_target
                    }
                };
                proto_nodes.push(node);
            },
        }
    }
    
    if !root_node_children_second.is_empty() {
        let root_node = &mut proto_nodes[root_node_index];

        // Take the first children buffer, leaving it empty.
        let mut first = std::mem::take(&mut root_node.children).into_vec();

        // Add elements of the second buffer to the first.
        first.append(&mut root_node_children_second.into_vec());

        // Convert it back to a boxed slice.
        root_node.children = first.into_boxed_slice();
    }

    let packet = CCommands::new(proto_nodes.into(), root_node_index.try_into().unwrap());
    player.client.enqueue_packet(&packet).await;
}

fn resolve_node_id(node_id: NodeId, node_id_offset: usize, root_node_index: usize) -> usize {
    if node_id == ROOT_NODE_ID {
        root_node_index
    } else {
        node_id_offset + node_id.0.get() - 2
    }
}

struct ProtoNodeBuilder<'a> {
    child_nodes: Vec<Self>,
    node_type: ProtoNodeType<'a>,
}

impl<'a> ProtoNodeBuilder<'a> {
    fn build(self, buffer: &mut Vec<ProtoNode<'a>>) -> usize {
        let mut children = Vec::with_capacity(self.child_nodes.len());
        for node in self.child_nodes {
            let i = node.build(buffer);
            children.push(i.try_into().unwrap());
        }

        let i = buffer.len();
        buffer.push(ProtoNode {
            children: children.into(),
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

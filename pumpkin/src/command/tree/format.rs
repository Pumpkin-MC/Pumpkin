use crate::command::tree::{CommandTree, Node, NodeType};
use std::fmt::{Display, Formatter, Write};

trait IsVisible {
    /// whether node should be printed in help command/usage hint
    fn is_visible(&self) -> bool;
}

impl IsVisible for Node {
    fn is_visible(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Literal { .. } | NodeType::Argument { .. }
        )
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.node_type {
            NodeType::Literal { string } => {
                f.write_str(string)?;
            }
            NodeType::Argument { name, .. } => {
                f.write_char('<')?;
                f.write_str(name)?;
                f.write_char('>')?;
            }
            _ => {}
        }

        Ok(())
    }
}

fn flatten_require_nodes(nodes: &[Node], children: &[usize]) -> Vec<usize> {
    let mut new_children = Vec::with_capacity(children.len());

    for &i in children {
        let node = &nodes[i];
        match &node.node_type {
            NodeType::Require { .. } => {
                new_children.extend(flatten_require_nodes(nodes, &node.children));
            }
            _ => new_children.push(i),
        }
    }

    new_children
}

fn write_tree(f: &mut Formatter<'_>, nodes: &[Node], children: &[usize]) -> std::fmt::Result {
    // Map node indices to actual nodes
    // NOTE: We assume that Require nodes have already been "flattened" out
    let nodes_iter = children.iter().map(|&idx| &nodes[idx]);

    // Check if there is a sibling of type ExecuteLeaf
    // If there is, arguments on the current level are optional and will be printed surrounded by square brackets
    let argument_is_optional = nodes_iter
        .clone()
        .any(|node| matches!(node.node_type, NodeType::ExecuteLeaf { .. }));

    let visible_nodes: Vec<&Node> = nodes_iter
        .clone()
        .filter(|node| node.is_visible())
        .collect();

    if visible_nodes.is_empty() {
        return Ok(());
    }

    let multiple_visible_nodes = visible_nodes.len() > 1;

    write!(f, " ")?;

    if argument_is_optional {
        write!(f, "[")?;
    }

    if multiple_visible_nodes {
        write!(f, "(")?;
    }

    for (idx, &node) in visible_nodes.iter().enumerate() {
        // Print usage of this node
        write!(f, "{node}")?;

        // Recursively print usage of it's children
        write_tree(f, nodes, &node.children)?;

        if multiple_visible_nodes && idx != visible_nodes.len() - 1 {
            write!(f, " | ")?;
        }
    }

    if multiple_visible_nodes {
        write!(f, ")")?;
    }

    if argument_is_optional {
        write!(f, "]")?;
    }

    Ok(())
}

impl Display for CommandTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.names[0])?;
        // TODO: Help of commands like /bossbar becomes really long
        //       A possible solution would be to add a check if the first level consists of literals
        //       only and if so to run the printing as if it were separate commands

        // TODO: Consider adding a max depth to print command usage only up to a certain depth.
        //       Vanilla seems to do this too (as can be seen with /help effect)

        // Clean up graph by 'flattening' require nodes to their children
        let flattened = flatten_require_nodes(&self.nodes, &self.children);
        // Recursively iterate over tree to print help usage of a command
        write_tree(f, &self.nodes, &flattened)
    }
}

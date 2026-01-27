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
                new_children.extend(flatten_require_nodes(nodes, node.children.as_slice()));
            }
            _ => new_children.push(i),
        }
    }

    new_children
}

impl Display for CommandTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.names[0])?;

        let current_children = &self.children[..];

        while !current_children.is_empty() {
            let flattened = flatten_require_nodes(&self.nodes, current_children);

            let mut visible_iter = flattened
                .iter()
                .copied()
                .filter(|&i| self.nodes[i].is_visible());

            let Some(first_idx) = visible_iter.next() else {
                break;
            };

            let second_idx = visible_iter.next();

            f.write_char(' ')?;

            let is_optional = flattened
                .iter()
                .any(|&i| matches!(self.nodes[i].node_type, NodeType::ExecuteLeaf { .. }));

            if is_optional {
                f.write_char('[')?;
            }

            if let Some(second_idx) = second_idx {
                f.write_char('(')?;
                self.nodes[first_idx].fmt(f)?;

                write!(f, " | ")?;
                self.nodes[second_idx].fmt(f)?;

                for idx in visible_iter {
                    write!(f, " | ")?;
                    self.nodes[idx].fmt(f)?;
                }
                f.write_char(')')?;

                break;
            }

            if is_optional {
                f.write_char(']')?;
            }
        }

        Ok(())
    }
}

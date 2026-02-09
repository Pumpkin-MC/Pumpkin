use crate::command::context::string_range::StringRange;
use crate::command::node::NodeId;

/// Represents a parsed node.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ParsedNode {
    pub node: NodeId,
    pub range: StringRange
}
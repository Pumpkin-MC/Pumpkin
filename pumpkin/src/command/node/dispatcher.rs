use crate::command::node::detached::CommandDetachedNode;
use crate::command::node::tree::Tree;

pub const ARG_SEPARATOR: &str = " ";
pub const ARG_SEPARATOR_CHAR: char = ' ';

pub const USAGE_OPTIONAL_OPEN: &str = "[";
pub const USAGE_OPTIONAL_CLOSE: &str = "]";
pub const USAGE_REQUIRED_OPEN: &str = "(";
pub const USAGE_REQUIRED_CLOSE: &str = ")";
pub const USAGE_OR: &str = "|";

/// The core command dispatcher, used to register, parse and execute commands.
struct CommandDispatcher {
    tree: Tree
}

impl CommandDispatcher {
    /// Creates a new [`CommandDispatcher`] with a new [`Tree`].
    pub fn new() -> CommandDispatcher {
        CommandDispatcher { tree: Tree::new() }
    }

    /// Creates this [`CommandDispatcher`] from a pre-existing tree.
    pub fn from_existing_tree(tree: Tree) -> CommandDispatcher {
        CommandDispatcher { tree }
    }

    /// Registers a command which can then be dispatched.
    pub fn register(command_node: CommandDetachedNode) {

    }
}
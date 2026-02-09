use std::sync::Arc;
use rustc_hash::FxHashMap;
use crate::command::context::command_source::CommandSource;
use crate::command::context::parsed_argument::ParsedArgument;
use crate::command::context::parsed_node::ParsedNode;
use crate::command::context::string_range::StringRange;
use crate::command::node::{Command, RedirectModifier};

/// Represents the context used when commands are run.
#[derive(Clone)]
pub struct CommandContext {
    /// The source running the commands.
    pub source: Arc<CommandSource>,

    /// The input string ran as the command.
    pub input: String,

    /// Arguments which have been parsed and
    /// can be fetched for command execution.
    pub argument: FxHashMap<String, Arc<dyn ParsedArgument>>,

    /// All the parsed nodes of the command.
    pub nodes: Vec<ParsedNode>,

    /// The string range of input.
    pub range: StringRange,

    /// The child context of this context.
    pub child: Option<Box<CommandContext>>,

    /// The redirect modifier of this context.
    pub modifier: Option<RedirectModifier>,

    /// Whether this context forks or not.
    pub forks: bool,

    /// The command stored in this context which
    /// is run to get a command result.
    pub command: Option<Command>
}

impl CommandContext {
    /// Copies this context with the source provided.
    pub fn with_source(&self, source: Arc<CommandSource>) -> CommandContext {
        CommandContext {
            source,
            input: self.input.clone(),
            argument: self.argument.clone(),
            nodes: self.nodes.clone(),
            range: self.range.clone(),
            child: self.child.clone(),
            modifier: self.modifier.clone(),
            forks: self.forks,
            command: self.command.clone(),
        }
    }

    /// Returns the child immediately below this node.
    pub fn get_child(&self) -> Option<&CommandContext> {
        self.child.as_deref()
    }

    /// Returns the child which does not have a child which originated from this node.
    /// This may return itself.
    pub fn get_last_child(&self) -> &CommandContext {
        let mut current_child = self;
        while let Some(child) = &current_child.child {
            current_child = &*child;
        }
        current_child
    }
}

/// Represents a linked chain of [`CommandContext`]s, where the previous links to the next as a child.
#[derive(Clone)]
pub struct ContextChain {
    /// The modifiers of this context chain.
    modifiers: Vec<CommandContext>,

    /// That specific [`CommandContext`] to execute.
    execute: Box<CommandContext>
}

impl ContextChain {
    /// Creates a new chain of contexts from a vector of them and one to execute.
    ///
    /// # Panics
    ///
    /// Panics if the `execute` given is non-executable.
    pub fn new(modifiers: Vec<CommandContext>, execute: Box<CommandContext>) -> ContextChain {
        assert!(execute.command.is_some(), "Expected last command in chain to be executable");
        ContextChain {
            modifiers,
            execute
        }
    }

    /// Tries to flatten a [`CommandContext`] in a [`Box`]. If no command
    /// is available at the end of chain, [`None`] is returned.
    pub fn try_flatten(root: &CommandContext) -> Option<ContextChain> {
        let mut modifiers = Vec::new();
        let mut current = root;

        loop {
            if let Some(child) = current.get_child() {
                modifiers.push(child.clone());
                current = child;
            } else {
                return if current.command.is_some() {
                    Some(ContextChain::new(modifiers, Box::new(current.clone())))
                } else {
                    None
                }
            }
        }
    }
}

/// A builder that helps to create context
pub struct ContextBuilder {

}
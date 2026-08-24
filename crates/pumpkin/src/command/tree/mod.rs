use super::{CommandExecutor, args::ArgumentConsumer};
use crate::command::CommandSender;
use crate::command::suggestion::suggestions::Suggestions;
use crate::server::Server;
use std::pin::Pin;
use std::{borrow::Cow, collections::VecDeque, fmt::Debug, sync::Arc};

pub mod builder;
pub mod format;

#[derive(Clone, Copy)]
pub struct RawArg<'a> {
    pub value: &'a str,
    pub start: usize,
    pub end: usize,
    pub input: &'a str,
}

impl Debug for RawArg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawArg")
            .field("value", &self.value)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// see [`crate::command::tree::builder::argument`]
pub type RawArgs<'a> = Vec<RawArg<'a>>;

pub type CommandSuggestionResult<'a> = Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>>;

pub trait CommandSuggestionProvider: Send + Sync {
    fn suggest<'a>(
        &'a self,
        src: &'a CommandSender,
        server: &'a Server,
        input: &'a str,
        start: usize,
        end: usize,
    ) -> CommandSuggestionResult<'a>;
}

#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<usize>,
    pub node_type: NodeType,
}

#[derive(Clone)]
pub enum NodeType {
    ExecuteLeaf {
        executor: Arc<dyn CommandExecutor + Send>,
    },
    Literal {
        string: String,
    },
    Argument {
        name: String,
        consumer: Arc<dyn ArgumentConsumer + Send>,
        suggestion_provider: Option<Arc<dyn CommandSuggestionProvider>>,
    },
    Require {
        predicate: Arc<dyn Fn(&CommandSender) -> bool + Send + Sync>,
    },
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecuteLeaf { .. } => f
                .debug_struct("ExecuteLeaf")
                .field("executor", &"..")
                .finish(),
            Self::Literal { string } => f.debug_struct("Literal").field("string", string).finish(),
            Self::Argument { name, .. } => f
                .debug_struct("Argument")
                .field("name", name)
                .field("consumer", &"..")
                .finish(),
            Self::Require { .. } => f.debug_struct("Require").field("predicate", &"..").finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Tree(CommandTree),
    Alias(String),
}

#[derive(Debug, Clone)]
pub struct CommandTree {
    pub nodes: Vec<Node>,
    pub children: Vec<usize>,
    pub names: Vec<String>,
    pub description: Cow<'static, str>,
    pub source: Option<String>,
}

impl CommandTree {
    /// iterate over all possible paths that end in a [`NodeType::ExecuteLeaf`]
    pub(crate) fn iter_paths(&self) -> impl Iterator<Item = Vec<usize>> + use<'_> {
        let mut todo = VecDeque::<(usize, usize)>::new();

        // add root's children
        todo.extend(self.children.iter().map(|&i| (0, i)));

        TraverseAllPathsIter {
            tree: self,
            path: Vec::<usize>::new(),
            todo,
        }
    }
}

struct TraverseAllPathsIter<'a> {
    tree: &'a CommandTree,
    path: Vec<usize>,
    /// (depth, i)
    todo: VecDeque<(usize, usize)>,
}

impl Iterator for TraverseAllPathsIter<'_> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (depth, i) = self.todo.pop_front()?;
            let node = &self.tree.nodes[i];

            // add new children to front
            self.todo.reserve(node.children.len());
            node.children
                .iter()
                .rev()
                .for_each(|&c| self.todo.push_front((depth + 1, c)));

            // update path
            while self.path.len() > depth {
                self.path.pop();
            }
            self.path.push(i);

            if let NodeType::ExecuteLeaf { .. } = node.node_type {
                return Some(self.path.clone());
            }
        }
    }
}

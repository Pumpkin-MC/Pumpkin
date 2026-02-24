use crate::command::argument_types::argument_type::AnyArgumentType;
use crate::command::node::{
    ArgumentNodeMetadata, Command, CommandNodeMetadata, LiteralNodeMetadata, NodeMetadata,
    OwnedNodeData, RedirectModifier, Redirection, Requirements,
};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DETACHED_NODE_ID: AtomicU64 = AtomicU64::new(1);

/// Represents a **global** integral number of
/// any type of node which is unique at runtime.
///
/// This is important for nodes not bound to a
/// tree.
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct GlobalNodeId(pub NonZero<u64>);

impl GlobalNodeId {
    /// Generates an ID that is guaranteed
    /// to be unique at runtime, by using atomics.
    pub fn new() -> Self {
        Self(
            NonZero::new(NEXT_DETACHED_NODE_ID.fetch_add(1, Ordering::Relaxed))
                .expect("expected a non-zero id"),
        )
    }
}

impl Default for GlobalNodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a literal, non-command node that has not been attached
/// to a tree yet.
///
/// If you want to start a command with this node, use [`CommandDetachedNode`] instead.
///
/// To be of any utility, this must be attached to a tree later.
pub struct LiteralDetachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, DetachedNode>,
    pub redirect: Option<Redirection>,
    pub meta: LiteralNodeMetadata,
}

impl LiteralDetachedNode {
    /// Creates a detached literal node from its properties,
    /// without any children.
    ///
    /// # Note
    /// Prefer using the [`LiteralArgumentBuilder`] over this function.=
    pub fn new(
        global_id: GlobalNodeId,
        literal: impl Into<Cow<'static, str>>,
        command: Option<Command>,
        requirements: Requirements,
        redirect: Option<Redirection>,
        modifier: RedirectModifier,
        forks: bool,
    ) -> Self {
        Self {
            owned: OwnedNodeData {
                global_id,
                requirements,
                modifier,
                forks,
                command,
            },
            children: FxHashMap::default(),
            redirect,
            meta: LiteralNodeMetadata::new(literal),
        }
    }
}

/// Represents a literal, command node that has not been attached
/// to a tree yet.
///
/// If you don't want to start a command with this node, use [`LiteralDetachedNode`] instead.
///
/// To be of any utility, this must be attached to a tree later.
pub struct CommandDetachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, DetachedNode>,
    pub redirect: Option<Redirection>,
    pub meta: CommandNodeMetadata,
}

impl CommandDetachedNode {
    /// Creates a detached literal node from its properties,
    /// without any children.
    ///
    /// # Note
    /// Prefer using the [`LiteralArgumentBuilder`] over this function.
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        global_id: GlobalNodeId,
        literal: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
        command: Option<Command>,
        requirements: Requirements,
        redirect: Option<Redirection>,
        modifier: RedirectModifier,
        forks: bool,
    ) -> Self {
        Self {
            owned: OwnedNodeData {
                global_id,
                requirements,
                modifier,
                forks,
                command,
            },
            children: FxHashMap::default(),
            redirect,
            meta: CommandNodeMetadata::new(literal, description),
        }
    }
}

/// Represents a node that accepts a specific type of argument.
///
/// To be of any utility, this must be attached to a tree later.
pub struct ArgumentDetachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, DetachedNode>,
    pub redirect: Option<Redirection>,
    pub meta: ArgumentNodeMetadata,
}

impl ArgumentDetachedNode {
    /// Creates a detached argument node from its properties,
    /// without any children.
    ///
    /// # Note
    /// Prefer using the [`RequiredArgumentBuilder`] over this function.
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        global_id: GlobalNodeId,
        name: impl Into<Cow<'static, str>>,
        argument_type: Arc<dyn AnyArgumentType>,
        command: Option<Command>,
        requirements: Requirements,
        redirect: Option<Redirection>,
        modifier: RedirectModifier,
        forks: bool,
    ) -> Self {
        Self {
            owned: OwnedNodeData {
                global_id,
                requirements,
                modifier,
                forks,
                command,
            },
            children: FxHashMap::default(),
            redirect,
            meta: ArgumentNodeMetadata::new(name, argument_type),
        }
    }
}

/// Represents a node not attached to a [`Tree`] yet.
pub enum DetachedNode {
    Literal(LiteralDetachedNode),
    Command(CommandDetachedNode),
    Argument(ArgumentDetachedNode),
}

/// Represents a [`DetachedNode`] that has been irreversibly
/// decomposed into its elements so that it can be recast
/// into a new [`AttachedNode`].
pub struct DecomposedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, DetachedNode>,
    pub redirect: Option<Redirection>,
    pub meta: NodeMetadata,
}

impl From<LiteralDetachedNode> for DetachedNode {
    fn from(node: LiteralDetachedNode) -> Self {
        Self::Literal(node)
    }
}

impl From<CommandDetachedNode> for DetachedNode {
    fn from(node: CommandDetachedNode) -> Self {
        Self::Command(node)
    }
}

impl From<ArgumentDetachedNode> for DetachedNode {
    fn from(node: ArgumentDetachedNode) -> Self {
        Self::Argument(node)
    }
}

impl DetachedNode {
    /// Irreversibly decomposes this [`DetachedNode`] into its constituent elements.
    /// This allows it to then be recast into a new [`AttachedNode`].
    #[must_use]
    pub fn decompose(self) -> DecomposedNode {
        match self {
            Self::Literal(node) => DecomposedNode {
                owned: node.owned,
                children: node.children,
                redirect: node.redirect,
                meta: NodeMetadata::Literal(node.meta),
            },
            Self::Command(node) => DecomposedNode {
                owned: node.owned,
                children: node.children,
                redirect: node.redirect,
                meta: NodeMetadata::Command(node.meta),
            },
            Self::Argument(node) => DecomposedNode {
                owned: node.owned,
                children: node.children,
                redirect: node.redirect,
                meta: NodeMetadata::Argument(node.meta),
            },
        }
    }

    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Literal(node) => node.meta.literal.to_string(),
            Self::Command(node) => node.meta.literal.to_string(),
            Self::Argument(node) => node.meta.name.to_string(),
        }
    }
}

use crate::command::argument_builder::private::Sealed;
use crate::command::argument_types::argument_type::AnyArgumentType;
use crate::command::node::detached::{
    ArgumentDetachedNode, CommandDetachedNode, DetachedNode, LiteralDetachedNode,
};
use crate::command::node::{Command, RedirectModifier, Redirection, Requirement};
use std::borrow::Cow;
use std::sync::Arc;

/// Represents an intermediate struct for
/// building arguments for commands.
///
/// Note: This is an implementation detail.
pub struct CommonArgumentBuilder {
    pub arguments: Vec<DetachedNode>,
    pub command: Option<Command>,
    pub requirement: Requirement,
    pub target: Option<Redirection>,
    pub modifier: RedirectModifier,
    pub forks: bool,
}

/// A builder that builds a literal, non-command [`DetachedNode`].
pub struct LiteralArgumentBuilder {
    pub common: CommonArgumentBuilder,
    pub literal: Cow<'static, str>,
}

/// A builder that builds a command [`DetachedNode`].
pub struct CommandArgumentBuilder {
    pub common: CommonArgumentBuilder,
    pub literal: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

/// A builder that builds an argument [`DetachedNode`].
pub struct RequiredArgumentBuilder {
    pub common: CommonArgumentBuilder,
    pub name: String,
    pub argument_type: Arc<dyn AnyArgumentType>,
}

mod private {
    // We want to make this trait private so that
    // we can implement it for only our
    // argument builders defined here.
    pub trait Sealed {}
}

pub trait ArgumentBuilder<N: Into<DetachedNode>>: Sized + Sealed {
    /// Puts an argument to be specified, right after this one is specified.
    ///
    /// # Panics
    ///
    /// Panics if this node is redirected to another node.
    #[must_use]
    fn then(self, argument: impl Into<DetachedNode>) -> Self;

    /// Gets the command to execute for the node being built.
    #[must_use]
    fn command(&self) -> Option<Command>;

    /// Sets the command to execute for the node being built.
    #[must_use]
    fn executes(self, command: Command) -> Self;

    /// Gets a reference to the arguments of the node to be built.
    #[must_use]
    fn arguments(&self) -> &[DetachedNode];

    /// Gets the node to which the node being built by this [`ArgumentBuilder`] redirects.
    #[must_use]
    fn target(&self) -> Option<Redirection>;

    /// Gets the redirect modifier of the node this [`ArgumentBuilder`] is building.
    #[must_use]
    fn redirect_modifier(&self) -> RedirectModifier;

    /// Whether this builder forks.
    #[must_use]
    fn forks(&self) -> bool;

    /// Builds the node represented by this builder, consuming itself in the process.
    #[must_use]
    fn build(self) -> N;
}

// Implement the private trait for our builders!
impl Sealed for LiteralArgumentBuilder {}
impl Sealed for CommandArgumentBuilder {}
impl Sealed for RequiredArgumentBuilder {}

/// Helper macro to implement repeated code of `ArgumentBuilder` for our types.
macro_rules! impl_boilerplate_argument_builder {
    () => {
        fn then(mut self, argument: impl Into<DetachedNode>) -> Self {
            assert!(
                self.target().is_none(),
                "Cannot add children to a redirected node"
            );
            let node = argument.into();
            self.common.arguments.push(node);
            self
        }

        fn command(&self) -> Option<Command> {
            self.common.command.clone()
        }

        fn executes(mut self, command: Command) -> Self {
            self.common.command = Some(command);
            self
        }

        fn arguments(&self) -> &[DetachedNode] {
            &self.common.arguments
        }

        fn target(&self) -> Option<Redirection> {
            self.common.target.clone()
        }

        fn redirect_modifier(&self) -> RedirectModifier {
            self.common.modifier.clone()
        }

        fn forks(&self) -> bool {
            self.common.forks
        }
    };
}

impl ArgumentBuilder<LiteralDetachedNode> for LiteralArgumentBuilder {
    impl_boilerplate_argument_builder!();

    fn build(self) -> LiteralDetachedNode {
        LiteralDetachedNode::new(
            self.literal,
            self.common.command,
            self.common.requirement,
            self.common.target,
            self.common.modifier,
            self.common.forks,
        )
    }
}

impl ArgumentBuilder<CommandDetachedNode> for CommandArgumentBuilder {
    impl_boilerplate_argument_builder!();

    fn build(self) -> CommandDetachedNode {
        CommandDetachedNode::new(
            self.literal,
            self.description,
            self.common.command,
            self.common.requirement,
            self.common.target,
            self.common.modifier,
            self.common.forks,
        )
    }
}

impl ArgumentBuilder<ArgumentDetachedNode> for RequiredArgumentBuilder {
    impl_boilerplate_argument_builder!();

    fn build(self) -> ArgumentDetachedNode {
        ArgumentDetachedNode::new(
            self.name,
            self.argument_type,
            self.common.command,
            self.common.requirement,
            self.common.target,
            self.common.modifier,
            self.common.forks,
        )
    }
}

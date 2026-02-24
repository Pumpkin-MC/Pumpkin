use crate::command::argument_builder::private::Sealed;
use crate::command::argument_types::argument_type::AnyArgumentType;
use crate::command::node::detached::{
    ArgumentDetachedNode, CommandDetachedNode, DetachedNode, GlobalNodeId, LiteralDetachedNode,
};
use crate::command::node::{
    Command, CommandExecutor, RedirectModifier, Redirection, Requirement, Requirements,
};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::sync::Arc;

/// Represents an intermediate struct for
/// building arguments for commands.
///
/// # Note
///
/// This is an implementation detail.
struct CommonArgumentBuilder {
    pub global_id: GlobalNodeId,
    pub arguments: FxHashMap<String, DetachedNode>,
    pub command: Option<Command>,
    pub requirements: Requirements,
    pub target: Option<Redirection>,
    pub modifier: RedirectModifier,
    pub permission: Option<Cow<'static, str>>,
    pub forks: bool,
}

impl CommonArgumentBuilder {
    fn new() -> Self {
        Self {
            global_id: GlobalNodeId::new(),
            arguments: FxHashMap::default(),
            command: None,
            requirements: Requirements::new(),
            target: None,
            modifier: RedirectModifier::KeepSource,
            permission: None,
            forks: false,
        }
    }
}

impl Default for CommonArgumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A short-form way to create a new [`CommandArgumentBuilder`]
/// from a literal and a command description.
///
/// This can be imported directly without a prefix, or imported with the `argument_builder::` prefix. Here's an example of usage:
/// ```
/// use pumpkin::command::argument_builder::command;
///
/// let builder = command("foo", "A test command");
/// ```
///
/// The builder returned will eventually construct a [`CommandDetachedNode`].
/// This node can then be registered into a dispatcher.
pub fn command(
    literal: impl Into<Cow<'static, str>>,
    description: impl Into<Cow<'static, str>>,
) -> CommandArgumentBuilder {
    CommandArgumentBuilder::new(literal, description)
}

/// A short-form way to create a new [`LiteralArgumentBuilder`]
/// from a literal.
///
/// This can be imported directly without a prefix, or imported with the `argument_builder::` prefix. Here's an example of usage:
/// ```
/// use pumpkin::command::argument_builder::literal;
///
/// let builder = literal("bar");
/// ```
///
/// The builder returned will eventually construct a [`LiteralDetachedNode`].
pub fn literal(literal: impl Into<Cow<'static, str>>) -> LiteralArgumentBuilder {
    LiteralArgumentBuilder::new(literal)
}

/// A short-form way to create a new [`RequiredArgumentBuilder`]
/// from an argument type and name.
///
/// This can be imported directly without a prefix, or imported with the `argument_builder::` prefix. Here's an example of usage:
/// ```
/// use pumpkin::command::{
///     argument_builder::argument,
///     argument_types::core::integer::IntegerArgumentType
/// };
///
/// let argument_builder = argument("bar", IntegerArgumentType::new(1, 10));
/// ```
///
/// The builder returned will eventually construct a [`ArgumentDetachedNode`].
pub fn argument(
    name: impl Into<Cow<'static, str>>,
    arg_type: impl AnyArgumentType + 'static,
) -> RequiredArgumentBuilder {
    RequiredArgumentBuilder::new(name, arg_type)
}

/// A builder that builds a literal, non-command [`DetachedNode`].
pub struct LiteralArgumentBuilder {
    common: CommonArgumentBuilder,
    literal: Cow<'static, str>,
}

/// A builder that builds a command [`DetachedNode`].
pub struct CommandArgumentBuilder {
    common: CommonArgumentBuilder,
    literal: Cow<'static, str>,
    description: Cow<'static, str>,
}

/// A builder that builds an argument [`DetachedNode`].
pub struct RequiredArgumentBuilder {
    common: CommonArgumentBuilder,
    name: Cow<'static, str>,
    argument_type: Arc<dyn AnyArgumentType>,
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
    /// Panics if this node is redirected to another node, or the child
    /// provided is of the type [`CommandDetachedNode`].
    #[must_use]
    fn then(self, child: impl Into<DetachedNode>) -> Self;

    /// Gets the command to execute for the node being built.
    #[must_use]
    fn command(&self) -> Option<Command>;

    /// Sets the command to execute for the node being built.
    #[must_use]
    fn executes(self, command: impl CommandExecutor + 'static) -> Self;

    /// Sets the redirect target of the node being built to another, without a modifier.
    #[must_use]
    fn redirect(self, redirection: impl Into<Redirection>) -> Self;

    /// Sets the redirect target of the node being built to another, with a given modifier.
    #[must_use]
    fn redirect_with_modifier(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self;

    /// Forks the given context, using multiple for later.
    #[must_use]
    fn fork(self, redirection: impl Into<Redirection>, redirect_modifier: RedirectModifier)
    -> Self;

    /// Forwards the given context, with the given `fork` flag.
    #[must_use]
    fn forward(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
        fork: bool,
    ) -> Self;

    /// Gets a reference to the arguments of the node to be built.
    #[must_use]
    fn arguments(&self) -> &FxHashMap<String, DetachedNode>;

    /// Gets the node to which the node being built by this [`ArgumentBuilder`] redirects.
    #[must_use]
    fn target(&self) -> Option<Redirection>;

    /// Adds a given predicate to this list of requirements of the node being built.
    ///
    /// This means that it is possible to chain multiple predicates together, which must all
    /// be satisfied.
    ///
    /// Permissions can also be inserted directly into this method as a `requirement`.
    #[must_use]
    fn requires(self, requirement: impl Into<Requirement>) -> Self;

    /// Gets the redirect modifier of the node this [`ArgumentBuilder`] is building.
    #[must_use]
    fn redirect_modifier(&self) -> RedirectModifier;

    /// Whether this builder forks.
    #[must_use]
    fn forks(&self) -> bool;

    /// Returns the 'future [`GlobalId`]' of the node that will be produced by this Builder.
    /// Very useful for redirects.
    #[must_use]
    fn id(&self) -> GlobalNodeId;

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
            assert!(
                !matches!(node, DetachedNode::Command(_)),
                "Cannot add a CommandDetachedNode as a child of a builder"
            );

            self.common.arguments.insert(node.name(), node);
            self
        }

        fn command(&self) -> Option<Command> {
            self.common.command.clone()
        }

        fn executes(mut self, command: impl CommandExecutor + 'static) -> Self {
            self.common.command = Some(Arc::new(command));
            self
        }

        fn requires(mut self, requirement: impl Into<Requirement>) -> Self {
            self.common.requirements.0.push(requirement.into());
            self
        }

        fn redirect(self, redirection: impl Into<Redirection>) -> Self {
            self.forward(redirection.into(), RedirectModifier::KeepSource, false)
        }

        fn redirect_with_modifier(self, redirection: impl Into<Redirection>, redirect_modifier: RedirectModifier) -> Self {
            self.forward(redirection.into(), redirect_modifier, false)
        }

        fn fork(self, redirection: impl Into<Redirection>, redirect_modifier: RedirectModifier) -> Self {
            self.forward(redirection.into(), redirect_modifier, true)
        }

        fn forward(mut self, redirection: impl Into<Redirection>, redirect_modifier: RedirectModifier, fork: bool) -> Self {
            assert!(self.common.arguments.is_empty(), "Cannot forward a node with children. The node must have no children to redirect somewhere else");
            self.common.target = Some(redirection.into());
            self.common.modifier = redirect_modifier;
            self.common.forks = fork;
            self
        }

        fn arguments(&self) -> &FxHashMap<String, DetachedNode> {
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

        fn id(&self) -> GlobalNodeId {
            self.common.global_id
        }
    };
}

/// Helper macro to generate `From` impl blocks for each builder.
macro_rules! impl_builder_from_impls {
    ($builder: ty => $detached_node: ty) => {
        impl From<$builder> for $detached_node {
            fn from(value: $builder) -> Self {
                value.build()
            }
        }

        impl From<$builder> for DetachedNode {
            fn from(value: $builder) -> Self {
                value.build().into()
            }
        }
    };
}

impl_builder_from_impls!(LiteralArgumentBuilder => LiteralDetachedNode);
impl_builder_from_impls!(CommandArgumentBuilder => CommandDetachedNode);
impl_builder_from_impls!(RequiredArgumentBuilder => ArgumentDetachedNode);

impl LiteralArgumentBuilder {
    /// Creates a new [`LiteralArgumentBuilder`] from a literal.
    pub fn new(literal: impl Into<Cow<'static, str>>) -> Self {
        Self {
            common: CommonArgumentBuilder::new(),
            literal: literal.into(),
        }
    }
}

impl CommandArgumentBuilder {
    /// Creates a new [`CommandArgumentBuilder`] from a literal and a command description.
    pub fn new(
        literal: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            common: CommonArgumentBuilder::new(),
            literal: literal.into(),
            description: description.into(),
        }
    }
}

impl RequiredArgumentBuilder {
    /// Creates a new [`RequiredArgumentBuilder`] from a name and an argument type.
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        arg_type: impl AnyArgumentType + 'static,
    ) -> Self {
        Self {
            common: CommonArgumentBuilder::new(),
            name: name.into(),
            argument_type: Arc::new(arg_type),
        }
    }
}

impl ArgumentBuilder<LiteralDetachedNode> for LiteralArgumentBuilder {
    impl_boilerplate_argument_builder!();

    fn build(self) -> LiteralDetachedNode {
        let mut node = LiteralDetachedNode::new(
            self.common.global_id,
            self.literal,
            self.common.command,
            self.common.requirements,
            self.common.target,
            self.common.modifier,
            self.common.permission,
            self.common.forks,
        );
        node.children = self.common.arguments;
        node
    }
}

impl ArgumentBuilder<CommandDetachedNode> for CommandArgumentBuilder {
    impl_boilerplate_argument_builder!();

    fn build(self) -> CommandDetachedNode {
        let mut node = CommandDetachedNode::new(
            self.common.global_id,
            self.literal,
            self.description,
            self.common.command,
            self.common.requirements,
            self.common.target,
            self.common.modifier,
            self.common.permission,
            self.common.forks,
        );
        node.children = self.common.arguments;
        node
    }
}

impl ArgumentBuilder<ArgumentDetachedNode> for RequiredArgumentBuilder {
    impl_boilerplate_argument_builder!();

    fn build(self) -> ArgumentDetachedNode {
        let mut node = ArgumentDetachedNode::new(
            self.common.global_id,
            self.name,
            self.argument_type,
            self.common.command,
            self.common.requirements,
            self.common.target,
            self.common.modifier,
            self.common.permission,
            self.common.forks,
        );
        node.children = self.common.arguments;
        node
    }
}

#[cfg(test)]
mod test {
    use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
    use crate::command::argument_types::core::double::DoubleArgumentType;
    use crate::command::argument_types::core::integer::IntegerArgumentType;
    use crate::command::argument_types::core::string::StringArgumentType;
    use crate::command::errors::error_types;
    use crate::command::node::Redirection;
    use crate::command::node::attached::AttachedNode;
    use crate::command::node::tree::Tree;
    use crate::command::string_reader::StringReader;

    #[test]
    fn literal_one() {
        let builder = literal("test");
        let node = builder.build();

        assert_eq!(node.meta.literal, "test");
    }

    #[test]
    fn required_one() {
        let builder = argument("test", IntegerArgumentType::new(1, 10));
        let node = builder.build();

        assert_eq!(node.meta.name, "test");

        let mut reader1 = StringReader::new("5");
        let mut reader2 = StringReader::new("11");

        let boxed_result = node
            .meta
            .argument_type
            .parse(&mut reader1)
            .expect("The parsing should not have errored");
        let result = boxed_result
            .downcast::<i32>()
            .expect("Downcasting shouldn't have failed");
        assert_eq!(result, Box::new(5));

        let error = node
            .meta
            .argument_type
            .parse(&mut reader2)
            .expect_err("The parsing should have errored as 11 is outside the range");
        assert!(error.is(&error_types::INTEGER_TOO_HIGH));
    }

    #[test]
    fn literal_multiple() {
        let mut builder = command("letter", "A test command");
        for letter in 'a'..='z' {
            // Add a node per letter for the argument.
            let letter_string = letter.to_string();
            builder = builder.then(literal(letter_string));
        }

        let node = builder.build();
        assert_eq!(node.children.len(), 26);
    }

    #[test]
    fn required_multiple() {
        let builder = command("test", "A test command")
            .then(argument("number", DoubleArgumentType::any()))
            .then(argument("word", StringArgumentType::SingleWord));

        let node = builder.build();
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn redirect() {
        let builder = command("test", "A test command").redirect(Redirection::Root);

        let mut tree = Tree::new();
        let node_id = tree.add_child_to_root(builder);

        let node = &tree[node_id];
        let redirect = node
            .redirect
            .expect("Redirection should exist as it was added before");

        let target_id = tree
            .resolve(redirect)
            .expect("Target should have been resolved properly");
        let target = &tree[target_id];

        assert!(matches!(target, AttachedNode::Root(_)));
    }

    #[test]
    #[should_panic = "Cannot forward a node with children. The node must have no children to redirect somewhere else"]
    fn redirect_after_child() {
        let _ = command("test", "A test command")
            .then(literal("child"))
            .redirect(Redirection::Root);
    }

    #[test]
    #[should_panic = "Cannot add children to a redirected node"]
    fn redirect_before_child() {
        let _ = command("test", "A test command")
            .redirect(Redirection::Root)
            .then(literal("child"));
    }

    #[test]
    #[should_panic = "Cannot add a CommandDetachedNode as a child of a builder"]
    fn add_command_as_child() {
        let _ = command("foo", "A test command").then(command("bar", "Another test command"));
    }
}

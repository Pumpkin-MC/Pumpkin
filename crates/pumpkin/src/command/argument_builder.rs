use std::borrow::Cow;
use std::sync::Arc;

use crate::command::context::command_source::CommandSource;
use crate::command::node::detached::DetachedNode;
use crate::command::node::{
    ArcCommandExecutorAdapter, Command, CommandExecutor, CommandExecutorAdapter, RedirectModifier,
    Requirement,
};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderAdapter};

use pumpkin_command::argument_builder::ArgumentBuilder as CoreArgumentBuilder;
pub use pumpkin_command::node::Redirection;

pub struct CommandArgumentBuilder(
    pub pumpkin_command::argument_builder::CommandArgumentBuilder<CommandSource>,
);
pub struct LiteralArgumentBuilder(
    pub pumpkin_command::argument_builder::LiteralArgumentBuilder<CommandSource>,
);
pub struct RequiredArgumentBuilder(
    pub pumpkin_command::argument_builder::RequiredArgumentBuilder<CommandSource>,
);

pub fn command(
    literal: impl Into<Cow<'static, str>>,
    description: impl Into<Cow<'static, str>>,
) -> CommandArgumentBuilder {
    CommandArgumentBuilder(pumpkin_command::argument_builder::command(
        literal,
        description,
    ))
}

pub fn literal(literal: impl Into<Cow<'static, str>>) -> LiteralArgumentBuilder {
    LiteralArgumentBuilder(pumpkin_command::argument_builder::literal(literal))
}

pub fn argument(
    name: impl Into<Cow<'static, str>>,
    arg_type: impl pumpkin_command::argument_types::argument_type::AnyArgumentType<CommandSource>
    + 'static,
) -> RequiredArgumentBuilder {
    RequiredArgumentBuilder(pumpkin_command::argument_builder::argument(name, arg_type))
}

pub trait ArgumentBuilder: Sized {
    type Node;

    #[must_use]
    fn then(self, child: impl Into<DetachedNode>) -> Self;

    #[must_use]
    fn command(&self) -> Option<Command>;

    #[must_use]
    fn executes(self, command: impl CommandExecutor + 'static) -> Self {
        self.executes_arc(Arc::new(command))
    }

    #[must_use]
    fn executes_arc(self, command: Arc<dyn CommandExecutor + 'static>) -> Self;

    #[must_use]
    fn redirect(self, redirection: impl Into<Redirection>) -> Self;

    #[must_use]
    fn redirect_with_modifier(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self;

    #[must_use]
    fn fork(self, redirection: impl Into<Redirection>, redirect_modifier: RedirectModifier)
    -> Self;

    #[must_use]
    fn requires(self, requirement: impl Into<Requirement>) -> Self;

    #[must_use]
    fn build(self) -> Self::Node;
}

impl RequiredArgumentBuilder {
    #[must_use]
    pub fn suggests(self, provider: impl SuggestionProvider + 'static) -> Self {
        Self(self.0.suggests(SuggestionProviderAdapter(provider)))
    }
}

impl ArgumentBuilder for CommandArgumentBuilder {
    type Node = pumpkin_command::node::detached::CommandDetachedNode<CommandSource>;

    fn then(self, child: impl Into<DetachedNode>) -> Self {
        Self(self.0.then(child.into()))
    }

    fn command(&self) -> Option<Command> {
        self.0.command()
    }

    fn executes(self, command: impl CommandExecutor + 'static) -> Self {
        Self(self.0.executes(CommandExecutorAdapter(command)))
    }

    fn executes_arc(self, command: Arc<dyn CommandExecutor + 'static>) -> Self {
        Self(
            self.0
                .executes_arc(Arc::new(ArcCommandExecutorAdapter(command))),
        )
    }

    fn redirect(self, redirection: impl Into<Redirection>) -> Self {
        Self(self.0.redirect(redirection))
    }

    fn redirect_with_modifier(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self {
        Self(
            self.0
                .redirect_with_modifier(redirection, redirect_modifier),
        )
    }

    fn fork(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self {
        Self(self.0.fork(redirection, redirect_modifier))
    }

    fn requires(self, requirement: impl Into<Requirement>) -> Self {
        Self(self.0.requires(requirement))
    }

    fn build(self) -> Self::Node {
        self.0.build()
    }
}

impl ArgumentBuilder for LiteralArgumentBuilder {
    type Node = pumpkin_command::node::detached::LiteralDetachedNode<CommandSource>;

    fn then(self, child: impl Into<DetachedNode>) -> Self {
        Self(self.0.then(child.into()))
    }

    fn command(&self) -> Option<Command> {
        self.0.command()
    }

    fn executes(self, command: impl CommandExecutor + 'static) -> Self {
        Self(self.0.executes(CommandExecutorAdapter(command)))
    }

    fn executes_arc(self, command: Arc<dyn CommandExecutor + 'static>) -> Self {
        Self(
            self.0
                .executes_arc(Arc::new(ArcCommandExecutorAdapter(command))),
        )
    }

    fn redirect(self, redirection: impl Into<Redirection>) -> Self {
        Self(self.0.redirect(redirection))
    }

    fn redirect_with_modifier(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self {
        Self(
            self.0
                .redirect_with_modifier(redirection, redirect_modifier),
        )
    }

    fn fork(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self {
        Self(self.0.fork(redirection, redirect_modifier))
    }

    fn requires(self, requirement: impl Into<Requirement>) -> Self {
        Self(self.0.requires(requirement))
    }

    fn build(self) -> Self::Node {
        self.0.build()
    }
}

impl ArgumentBuilder for RequiredArgumentBuilder {
    type Node = pumpkin_command::node::detached::ArgumentDetachedNode<CommandSource>;

    fn then(self, child: impl Into<DetachedNode>) -> Self {
        Self(self.0.then(child.into()))
    }

    fn command(&self) -> Option<Command> {
        self.0.command()
    }

    fn executes(self, command: impl CommandExecutor + 'static) -> Self {
        Self(self.0.executes(CommandExecutorAdapter(command)))
    }

    fn executes_arc(self, command: Arc<dyn CommandExecutor + 'static>) -> Self {
        Self(
            self.0
                .executes_arc(Arc::new(ArcCommandExecutorAdapter(command))),
        )
    }

    fn redirect(self, redirection: impl Into<Redirection>) -> Self {
        Self(self.0.redirect(redirection))
    }

    fn redirect_with_modifier(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self {
        Self(
            self.0
                .redirect_with_modifier(redirection, redirect_modifier),
        )
    }

    fn fork(
        self,
        redirection: impl Into<Redirection>,
        redirect_modifier: RedirectModifier,
    ) -> Self {
        Self(self.0.fork(redirection, redirect_modifier))
    }

    fn requires(self, requirement: impl Into<Requirement>) -> Self {
        Self(self.0.requires(requirement))
    }

    fn build(self) -> Self::Node {
        self.0.build()
    }
}

impl From<CommandArgumentBuilder>
    for pumpkin_command::node::detached::CommandDetachedNode<CommandSource>
{
    fn from(b: CommandArgumentBuilder) -> Self {
        b.0.build()
    }
}

impl From<LiteralArgumentBuilder> for DetachedNode {
    fn from(b: LiteralArgumentBuilder) -> Self {
        Self::Literal(b.0.build())
    }
}

impl From<RequiredArgumentBuilder> for DetachedNode {
    fn from(b: RequiredArgumentBuilder) -> Self {
        Self::Argument(b.0.build())
    }
}

impl From<LiteralArgumentBuilder>
    for pumpkin_command::node::detached::LiteralDetachedNode<CommandSource>
{
    fn from(b: LiteralArgumentBuilder) -> Self {
        b.0.build()
    }
}

impl From<RequiredArgumentBuilder>
    for pumpkin_command::node::detached::ArgumentDetachedNode<CommandSource>
{
    fn from(b: RequiredArgumentBuilder) -> Self {
        b.0.build()
    }
}

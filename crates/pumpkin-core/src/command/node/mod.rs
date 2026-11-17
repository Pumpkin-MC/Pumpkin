use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;

pub use pumpkin_command::node::*;

pub mod attached {
    pub use pumpkin_command::node::attached::*;
    pub type AttachedNode =
        pumpkin_command::node::attached::AttachedNode<crate::command::CommandSource>;
    pub type RootAttachedNode =
        pumpkin_command::node::attached::RootAttachedNode<crate::command::CommandSource>;
    pub type LiteralAttachedNode =
        pumpkin_command::node::attached::LiteralAttachedNode<crate::command::CommandSource>;
    pub type CommandAttachedNode =
        pumpkin_command::node::attached::CommandAttachedNode<crate::command::CommandSource>;
    pub type ArgumentAttachedNode =
        pumpkin_command::node::attached::ArgumentAttachedNode<crate::command::CommandSource>;
}

pub mod detached {
    pub use pumpkin_command::node::detached::*;
    pub type DetachedNode =
        pumpkin_command::node::detached::DetachedNode<crate::command::CommandSource>;
    pub type LiteralDetachedNode =
        pumpkin_command::node::detached::LiteralDetachedNode<crate::command::CommandSource>;
    pub type CommandDetachedNode =
        pumpkin_command::node::detached::CommandDetachedNode<crate::command::CommandSource>;
    pub type ArgumentDetachedNode =
        pumpkin_command::node::detached::ArgumentDetachedNode<crate::command::CommandSource>;
}

pub mod tree {
    pub use pumpkin_command::node::tree::*;
    pub type Tree = pumpkin_command::node::tree::Tree<crate::command::CommandSource>;
}

pub mod dispatcher {
    pub use pumpkin_command::dispatcher::*;
    pub type CommandDispatcher =
        pumpkin_command::dispatcher::CommandDispatcher<crate::command::CommandSource>;
}

pub type Command = pumpkin_command::node::Command<CommandSource>;
pub type Requirement = pumpkin_command::node::Requirement<CommandSource>;
pub type Requirements = pumpkin_command::node::Requirements<CommandSource>;
pub type RedirectModifier = pumpkin_command::node::RedirectModifier<CommandSource>;
pub type RedirectModifierResult = pumpkin_command::node::RedirectModifierResult<CommandSource>;
pub type RedirectModifierExecutor = pumpkin_command::node::RedirectModifierExecutor<CommandSource>;

/// A struct implementing this trait is able to run with a given context.
pub trait CommandExecutor: Sync + Send {
    /// Executes this executor for a command.
    fn execute(&self, context: &CommandContext) -> pumpkin_command::node::CommandExecutorResult;
}

pub struct CommandExecutorAdapter<T>(pub T);

impl<T: CommandExecutor> pumpkin_command::node::CommandExecutor<CommandSource>
    for CommandExecutorAdapter<T>
{
    fn execute(
        &self,
        context: &pumpkin_command::context::command_context::CommandContext<'_, CommandSource>,
    ) -> pumpkin_command::node::CommandExecutorResult {
        self.0.execute(context)
    }
}

pub struct ArcCommandExecutorAdapter(pub std::sync::Arc<dyn CommandExecutor>);

impl pumpkin_command::node::CommandExecutor<CommandSource> for ArcCommandExecutorAdapter {
    fn execute(
        &self,
        context: &pumpkin_command::context::command_context::CommandContext<'_, CommandSource>,
    ) -> pumpkin_command::node::CommandExecutorResult {
        self.0.execute(context)
    }
}

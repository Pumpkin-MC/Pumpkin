//! Command and suggestion handlers for the async v0.2 world
//!
//! # Building versus running
//!
//! Building a command is synchronous, since `Command::new`, `then` and `execute` are plain host
//! calls, and only the callbacks the host runs later are async: one when the command executes and
//! one when a client asks for suggestions
//!
//! `execute` and `suggest` store your handler in a table here, get an id and hand only that id to
//! the host, the same way events and tasks work
//!
//! # Where to look
//!
//! * [`CommandHandler`] runs when the command executes
//! * [`CommandSuggestionHandler`] answers a client's request for completions
//! * [`Command::execute`] and [`CommandNode::execute`] attach a handler

use std::sync::Arc;

use crate::{
    Server,
    host::command::{
        Command, CommandError, CommandNode, CommandSender, CommandSuggestions, ConsumedArgs,
        SuggestionRequest,
    },
    registry::{LocalBoxFuture, Registry},
    text::TextComponent,
};

static COMMAND_HANDLERS: Registry<dyn ErasedCommandHandler> = Registry::new();
static COMMAND_SUGGESTION_HANDLERS: Registry<dyn ErasedSuggestionHandler> = Registry::new();

/// Handles the execution of a registered command
///
/// Attach an implementation with [`Command::execute`] or [`CommandNode::execute`]
///
/// # Examples
///
/// A handler can ask the server a question before it answers, because permission checks are
/// async host imports:
///
/// ```rust,ignore
/// struct Ping;
///
/// impl CommandHandler for Ping {
///     async fn handle(
///         &self,
///         sender: CommandSender,
///         _server: Server,
///         _args: ConsumedArgs,
///     ) -> Result<i32, CommandError> {
///         // has_permission takes only the node now, the host resolves the server itself
///         Ok(i32::from(sender.has_permission("example.ping".to_string()).await))
///     }
/// }
/// ```
#[allow(async_fn_in_trait, reason = "guest futures are intentionally not Send")]
pub trait CommandHandler: Send + Sync + 'static {
    /// Executes the command and returns the result code the server reports
    ///
    /// The sender, server and parsed arguments are owned, so they stay valid across every await
    ///
    /// # Errors
    ///
    /// Return a [`CommandError`] to report a failure to the sender, for example
    /// `CommandError::CommandFailed` with the message to show
    async fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError>;
}

/// Handles server-side suggestions for a command argument
///
/// Attach an implementation with [`CommandNode::suggest`]
#[allow(async_fn_in_trait, reason = "guest futures are intentionally not Send")]
pub trait CommandSuggestionHandler: Send + Sync + 'static {
    /// Computes the suggestions for the current command input
    ///
    /// The returned `start` and `length` say which part of the input the suggestions replace, so a
    /// handler can narrow the range when only part of an argument should change
    async fn suggest(
        &self,
        sender: CommandSender,
        server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions;
}

trait ErasedCommandHandler: Send + Sync {
    fn handle_erased(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> LocalBoxFuture<'_, Result<i32, CommandError>>;
}

impl<H: CommandHandler> ErasedCommandHandler for H {
    fn handle_erased(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> LocalBoxFuture<'_, Result<i32, CommandError>> {
        Box::pin(self.handle(sender, server, args))
    }
}

trait ErasedSuggestionHandler: Send + Sync {
    fn suggest_erased(
        &self,
        sender: CommandSender,
        server: Server,
        request: SuggestionRequest,
    ) -> LocalBoxFuture<'_, CommandSuggestions>;
}

impl<H: CommandSuggestionHandler> ErasedSuggestionHandler for H {
    fn suggest_erased(
        &self,
        sender: CommandSender,
        server: Server,
        request: SuggestionRequest,
    ) -> LocalBoxFuture<'_, CommandSuggestions> {
        Box::pin(self.suggest(sender, server, request))
    }
}

// A command id with no handler is a bug worth reporting, not a reason to trap the plugin, so the
// sender gets a failure message instead
pub(crate) async fn dispatch_command(
    command_id: u32,
    sender: CommandSender,
    server: Server,
    args: ConsumedArgs,
) -> Result<i32, CommandError> {
    // The registry lock is released before the handler runs
    let Some(handler) = COMMAND_HANDLERS.get(command_id) else {
        return Err(CommandError::CommandFailed(TextComponent::text(&format!(
            "no handler registered for command id {command_id}"
        ))));
    };
    handler.handle_erased(sender, server, args).await
}

// An unknown suggestion id answers with an empty list for the same reason
pub(crate) async fn dispatch_suggestion(
    handler_id: u32,
    sender: CommandSender,
    server: Server,
    request: SuggestionRequest,
) -> CommandSuggestions {
    let Some(handler) = COMMAND_SUGGESTION_HANDLERS.get(handler_id) else {
        return CommandSuggestions {
            start: request.start,
            length: 0,
            values: Vec::new(),
        };
    };
    handler.suggest_erased(sender, server, request).await
}

impl Command {
    /// Attaches an execution handler to this command and returns the command for chaining
    ///
    /// The handler is stored here and the host is given only its id, through
    /// `execute-with-handler-id`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ping = Command::new(&["ping".to_string()], "replies when allowed").execute(Ping);
    /// context.register_command(ping, "example.ping");
    /// ```
    #[must_use]
    pub fn execute<H: CommandHandler>(self, handler: H) -> Self {
        let id = COMMAND_HANDLERS.register(Arc::new(handler));
        self.execute_with_handler_id(id)
    }
}

impl CommandNode {
    /// Attaches an execution handler to this node and returns the node for chaining
    ///
    /// The handler runs when this node is the last one matched during dispatch, which makes it
    /// the way to give a subcommand or an argument branch its own behaviour
    #[must_use]
    pub fn execute<H: CommandHandler>(self, handler: H) -> Self {
        let id = COMMAND_HANDLERS.register(Arc::new(handler));
        self.execute_with_handler_id(id)
    }

    /// Attaches a server-side suggestion handler to this argument node and returns the node
    ///
    /// Java clients are told to ask the server for completions with `minecraft:ask_server`, and
    /// the handler runs each time a client does
    #[must_use]
    pub fn suggest<H: CommandSuggestionHandler>(self, handler: H) -> Self {
        let id = COMMAND_SUGGESTION_HANDLERS.register(Arc::new(handler));
        self.suggest_with_handler_id(id)
    }
}

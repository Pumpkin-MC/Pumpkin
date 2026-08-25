use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::command::Command;
use crate::{
    Result, Server,
    command::CommandNode,
    wit::pumpkin::plugin::command::{CommandError, CommandSender, ConsumedArgs},
};

pub(crate) static NEXT_COMMAND_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static COMMAND_HANDLERS: Mutex<BTreeMap<u32, Box<dyn CommandHandler>>> =
    Mutex::new(BTreeMap::new());
pub(crate) static COMMAND_SUGGESTION_HANDLERS: Mutex<
    BTreeMap<u32, Box<dyn CommandSuggestionHandler>>,
> = Mutex::new(BTreeMap::new());

/// Handles the execution of a registered command.
///
/// Implement this trait to define the logic that runs when a command is invoked.
/// The return value is the exit code passed back to the server; return `Ok(0)` for
/// success or an [`Err`] variant to report a failure message to the sender.
pub trait CommandHandler: Send + Sync {
    /// Executes the command.
    ///
    /// # Arguments
    /// - `sender` — who invoked the command (player or console).
    /// - `server` — handle to the server.
    /// - `args` — the parsed argument map for this command invocation.
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError>;
}

/// Handles server-side suggestions for a registered command argument.
///
/// ```rust,ignore
/// use pumpkin_plugin_api::command::{CommandSuggestion, CommandSuggestions, SuggestionRequest};
/// use pumpkin_plugin_api::commands::CommandSuggestionHandler;
/// use pumpkin_plugin_api::Server;
///
/// struct PatternSuggestions;
///
/// impl CommandSuggestionHandler for PatternSuggestions {
///     fn suggest(
///         &self,
///         _sender: pumpkin_plugin_api::command::CommandSender,
///         _server: Server,
///         request: SuggestionRequest,
///     ) -> CommandSuggestions {
///         let token_start = request
///             .input
///             .rfind([',', ' '])
///             .map_or(request.start as usize, |index| index + 1);
///         let block_start = request.input[token_start..]
///             .rfind('%')
///             .map_or(token_start, |index| token_start + index + 1);
///         let prefix = &request.input[block_start..];
///         let values = ["stone", "stripped_oak_log", "dirt", "diamond_block"]
///             .into_iter()
///             .filter(|block| block.starts_with(prefix))
///             .map(|block| CommandSuggestion {
///                 value: block.to_string(),
///                 tooltip: None,
///             })
///             .collect();
///
///         CommandSuggestions {
///             start: block_start as u32,
///             length: (request.input.len() - block_start) as u32,
///             values,
///         }
///     }
/// }
/// ```
pub trait CommandSuggestionHandler: Send + Sync {
    /// Computes suggestions for the current command input.
    ///
    /// `request.remaining` contains the replacement text currently covered by
    /// the suggestion range. Handlers may return a narrower range when only part
    /// of an argument should be replaced, for example after the last comma in a
    /// weighted block pattern.
    fn suggest(
        &self,
        sender: CommandSender,
        server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions;
}

impl Command {
    /// Attaches an execution handler to this command.
    ///
    /// Registers `handler` so that it is called whenever this command is invoked.
    /// Returns `self` to allow builder-style chaining.
    pub fn execute<H: CommandHandler + Send + Sync + 'static>(self, handler: H) -> Self {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        COMMAND_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Box::new(handler));

        self.execute_with_handler_id(id)
    }
}

impl CommandNode {
    /// Attaches an execution handler to this command node.
    ///
    /// Registers `handler` so that it is called when this specific node (subcommand
    /// or argument branch) is the final node matched during command dispatch.
    /// Returns `self` to allow builder-style chaining.
    pub fn execute<H: CommandHandler + Send + Sync + 'static>(self, handler: H) -> Self {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        COMMAND_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Box::new(handler));

        self.execute_with_handler_id(id)
    }

    /// Attaches a server-side suggestion handler to this argument node.
    ///
    /// The node is advertised to Java clients with `minecraft:ask_server`, and
    /// the handler is called whenever the client requests completions for this
    /// argument.
    pub fn suggest<H: CommandSuggestionHandler + Send + Sync + 'static>(self, handler: H) -> Self {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        COMMAND_SUGGESTION_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Box::new(handler));

        self.suggest_with_handler_id(id)
    }
}

pub use crate::wit::pumpkin::plugin::command::{
    CommandSuggestion, CommandSuggestions, SuggestionRequest,
};

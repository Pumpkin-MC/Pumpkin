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

impl Command {
    /// Attaches an execution handler to this command.
    ///
    /// Registers `handler` so that it is called whenever this command is invoked.
    /// Returns `self` to allow builder-style chaining.
    pub fn execute<H: CommandHandler + Send + Sync + 'static>(self, handler: H) -> Command {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        COMMAND_HANDLERS
            .lock()
            .unwrap()
            .insert(id, Box::new(handler));

        self.execute_with_handler_id(id);

        self
    }
}

impl CommandNode {
    /// Attaches an execution handler to this command node.
    ///
    /// Registers `handler` so that it is called when this specific node (subcommand
    /// or argument branch) is the final node matched during command dispatch.
    /// Returns `self` to allow builder-style chaining.
    pub fn execute<H: CommandHandler + Send + Sync + 'static>(self, handler: H) -> CommandNode {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        COMMAND_HANDLERS
            .lock()
            .unwrap()
            .insert(id, Box::new(handler));

        self.execute_with_handler_id(id);

        self
    }
}

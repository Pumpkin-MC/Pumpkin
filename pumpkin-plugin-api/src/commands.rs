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

pub trait CommandHandler: Send + Sync {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError>;
}

impl Command {
    /// Registers a command handler with the plugin.
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
    /// Registers a command handler with the plugin.
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

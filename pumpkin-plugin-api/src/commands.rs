use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::command::Command;
use crate::{
    Context, Result, Server,
    text::TextComponent,
    wit::pumpkin::plugin::command::{CommandError, CommandSender, ConsumedArgs},
};

pub(crate) static NEXT_COMMAND_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static COMMAND_HANDLERS: Mutex<BTreeMap<u32, Box<dyn ErasedCommandHandler>>> =
    Mutex::new(BTreeMap::new());

pub trait FromConsumedArgs: Sized {
    fn from_consumed_args(args: ConsumedArgs) -> Result<Self, CommandError>;
}

pub trait CommandHandler<C>: Send + Sync {
    fn handle(&self, sender: CommandSender, server: Server, args: C) -> Result<i32>;
}

pub(crate) trait ErasedCommandHandler: Send + Sync {
    fn handle_erased(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError>;
}

struct CommandWrapper<C, H> {
    handler: H,
    _phantom: PhantomData<C>,
}

impl<C, H> ErasedCommandHandler for CommandWrapper<C, H>
where
    C: FromConsumedArgs + Send + Sync,
    H: CommandHandler<C> + Send + Sync,
{
    fn handle_erased(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let typed = C::from_consumed_args(args)?;
        self.handler
            .handle(sender, server, typed)
            .map_err(|e| CommandError::CommandFailed(TextComponent::text(&e.to_string())))
    }
}

impl Context {
    /// Registers a command handler with the plugin.
    ///
    /// `build_command` receives the freshly-created [`Command`] so you can
    /// attach argument nodes before it is sent to the server.
    ///
    /// Returns the command id that will be passed to `handle_command`.
    pub async fn register_command_handler<
        C: FromConsumedArgs + Send + Sync + 'static,
        H: CommandHandler<C> + Send + Sync + 'static,
    >(
        &mut self,
        command: Command,
        handler: H,
    ) -> Result<u32, CommandError> {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        command.execute_with_handler_id(id);

        // TODO
        self.register_command(command, "");

        let wrapped = CommandWrapper {
            handler,
            _phantom: PhantomData::<C>,
        };

        COMMAND_HANDLERS
            .lock()
            .map_err(|e| CommandError::CommandFailed(TextComponent::text(&e.to_string())))?
            .insert(id, Box::new(wrapped));

        Ok(id)
    }
}

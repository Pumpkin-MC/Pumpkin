use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::command::Command;
use crate::{Context, Result, Server};

pub(crate) static NEXT_COMMAND_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static COMMAND_HANDLERS: Mutex<BTreeMap<u32, Box<dyn ErasedCommandHandler>>> =
    Mutex::new(BTreeMap::new());

pub trait FromIntoCommand: Sized {
    fn from_command(command: Command) -> Self;
    fn into_command(self) -> Command;
}

pub trait CommandHandler<C> {
    fn handle(&self, server: Server, command_data: C) -> Result<()>;
}

pub(crate) trait ErasedCommandHandler: Send + Sync {
    fn handle_erased(&self, server: Server, command: Command) -> Result<()>;
}

struct CommandWrapper<C, H> {
    handler: H,
    _phantom: PhantomData<C>,
}

impl<C: FromIntoCommand + Send + Sync, H: CommandHandler<C> + Send + Sync> ErasedCommandHandler
    for CommandWrapper<C, H>
{
    fn handle_erased(&self, server: Server, command: Command) -> Result<()> {
        let specific_command = C::from_command(command);
        self.handler.handle(server, specific_command)
    }
}

impl Context {
    /// Registers a command handler with the plugin.
    ///
    /// The handler must implement the [`CommandHandler`] trait.
    pub async fn register_command_handler<
        C: FromIntoCommand + Send + Sync + 'static,
        H: CommandHandler<C> + Send + Sync + 'static,
    >(
        &mut self,
        handler: H,
        permission: String,
    ) -> Result<u32> {
        let id = NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed);

        let wrapped = CommandWrapper {
            handler,
            _phantom: PhantomData::<C>,
        };

        COMMAND_HANDLERS
            .lock()
            .map_err(|e| e.to_string())?
            .insert(id, Box::new(wrapped));

        // TODO
        // self.register_command(
        //     context_res,
        //     &permission
        // );

        Ok(id)
    }
}

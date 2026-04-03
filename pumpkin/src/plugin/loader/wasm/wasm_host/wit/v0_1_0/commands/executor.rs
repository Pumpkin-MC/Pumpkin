use std::sync::Arc;

use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};

use crate::{
    command::{CommandExecutor, dispatcher::CommandError},
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt, PluginInstance, WasmPlugin,
        wit::v0_1_0::pumpkin::plugin::command::CommandError as CommandErrorWit,
    },
    server::Server,
};

pub struct WasmCommandExecutor {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
    pub server: Arc<Server>,
}

impl CommandExecutor for WasmCommandExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a crate::command::CommandSender,
        _server: &'a crate::server::Server,
        args: &'a crate::command::args::ConsumedArgs<'a>,
    ) -> crate::command::CommandResult<'a> {
        Box::pin(async move {
            let mut store = self.plugin.store.lock().await;

            let sender_resource = store.data_mut().add_command_sender(sender.clone()).unwrap();
            let server_resource = store.data_mut().add_server(self.server.clone()).unwrap();
            let args_resource = store.data_mut().add_consumed_args(args).unwrap();

            match self.plugin.plugin_instance {
                PluginInstance::V0_1_0(ref plugin) => {
                    let result = plugin
                        .call_handle_command(
                            &mut *store,
                            self.handler_id,
                            sender_resource,
                            server_resource,
                            args_resource,
                        )
                        .await
                        .map_err(|e| {
                            CommandError::CommandFailed(
                                TextComponent::text(format!(
                                    "Wasm command failed with following error: {e}"
                                ))
                                .color(Color::Named(NamedColor::Red)),
                            )
                        })?;

                    match result {
                        Ok(value) => Ok(value),
                        Err(err) => match err {
                            CommandErrorWit::InvalidConsumption(value) => {
                                Err(CommandError::InvalidConsumption(value))
                            }
                            CommandErrorWit::InvalidRequirement => {
                                Err(CommandError::InvalidRequirement)
                            }
                            CommandErrorWit::PermissionDenied => {
                                Err(CommandError::PermissionDenied)
                            }
                            CommandErrorWit::CommandFailed(resource) => {
                                Err(CommandError::CommandFailed(
                                    resource.consume(store.data_mut()).provider,
                                ))
                            }
                        },
                    }
                }
            }
        })
    }
}

use std::sync::Arc;

use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};

use crate::{
    command::{
        CommandExecutor,
        context::string_range::StringRange,
        dispatcher::CommandError,
        suggestion::{Suggestion, suggestions::Suggestions},
        tree::{CommandSuggestionProvider, CommandSuggestionResult},
    },
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt, PluginInstance, WasmPlugin,
        wit::v0_1::pumpkin::plugin::command::{CommandError as CommandErrorWit, SuggestionRequest},
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

            let sender_resource = store
                .data_mut()
                .add_command_sender(sender.clone())
                .expect("valid command sender");
            let server_resource = store
                .data_mut()
                .add_server(self.server.clone())
                .expect("valid server");
            let args_resource = store
                .data_mut()
                .add_consumed_args(args)
                .expect("valid consumed args");

            let sender_rep = sender_resource.rep();
            let server_rep = server_resource.rep();
            let args_rep = args_resource.rep();

            match self.plugin.plugin_instance {
                PluginInstance::V0_1(ref plugin) => {
                    let result = plugin
                        .call_handle_command(
                            &mut *store,
                            self.handler_id,
                            sender_resource,
                            server_resource,
                            args_resource,
                        )
                        .await;

                    let _ = store
                        .data_mut()
                        .resource_table
                        .delete::<crate::plugin::loader::wasm::wasm_host::state::CommandSenderResource>(
                            wasmtime::component::Resource::new_own(sender_rep),
                        );
                    let _ = store
                        .data_mut()
                        .resource_table
                        .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                        wasmtime::component::Resource::new_own(server_rep),
                    );
                    let _ = store
                        .data_mut()
                        .resource_table
                        .delete::<crate::plugin::loader::wasm::wasm_host::state::ConsumedArgsResource>(
                            wasmtime::component::Resource::new_own(args_rep),
                        );

                    let result = result.map_err(|e| {
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

pub struct WasmCommandSuggestionProvider {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
    pub server: Arc<Server>,
}

impl CommandSuggestionProvider for WasmCommandSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        src: &'a crate::command::CommandSender,
        _server: &'a Server,
        input: &'a str,
        start: usize,
        end: usize,
    ) -> CommandSuggestionResult<'a> {
        Box::pin(async move {
            let mut store = self.plugin.store.lock().await;

            let sender_resource = match store.data_mut().add_command_sender(src.clone()) {
                Ok(resource) => resource,
                Err(error) => {
                    tracing::error!(
                        "Failed to create command sender resource for suggestions: {error}"
                    );
                    return Suggestions::empty();
                }
            };
            let server_resource = match store.data_mut().add_server(self.server.clone()) {
                Ok(resource) => resource,
                Err(error) => {
                    tracing::error!("Failed to create server resource for suggestions: {error}");
                    return Suggestions::empty();
                }
            };

            let request = SuggestionRequest {
                input: input.to_string(),
                cursor: input.len().try_into().unwrap_or(u32::MAX),
                start: start.try_into().unwrap_or(u32::MAX),
                remaining: input[start.min(input.len())..end.min(input.len())].to_string(),
            };

            let response = match self.plugin.plugin_instance {
                PluginInstance::V0_1(ref plugin) => {
                    plugin
                        .call_handle_command_suggestion(
                            &mut *store,
                            self.handler_id,
                            sender_resource,
                            server_resource,
                            &request,
                        )
                        .await
                }
            };

            let response = match response {
                Ok(response) => response,
                Err(error) => {
                    tracing::error!("Wasm command suggestion failed: {error}");
                    return Suggestions::empty();
                }
            };

            let start = response.start as usize;
            let end = start.saturating_add(response.length as usize);
            let range = StringRange::between(start, end.min(input.len()));
            let suggestions = response
                .values
                .into_iter()
                .map(|suggestion| {
                    if let Some(tooltip) = suggestion.tooltip {
                        Suggestion::with_tooltip(
                            range,
                            suggestion.value,
                            tooltip.consume(store.data_mut()).provider,
                        )
                    } else {
                        Suggestion::without_tooltip(range, suggestion.value)
                    }
                })
                .collect();

            Suggestions::new(range, suggestions)
        })
    }
}

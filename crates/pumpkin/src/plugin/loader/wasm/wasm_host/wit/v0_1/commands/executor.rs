use std::sync::Arc;

use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};

use crate::{
    command::{
        context::command_context::CommandContext,
        errors::error_types::DISPATCHER_PARSE_EXCEPTION,
        node::{CommandExecutor, CommandExecutorResult},
        suggestion::{
            provider::SuggestionProvider,
            suggestions::{Suggestions, SuggestionsBuilder},
        },
    },
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt, PluginInstance, WasmPlugin,
        args::build_consumed_args_from_context,
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
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut store = self.plugin.store.lock().await;

                let sender_resource = store
                    .data_mut()
                    .add_command_sender(context.source.output.clone())
                    .map_err(|e| {
                        DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                            format!("Failed to create sender: {e}"),
                        ))
                    })?;
                let server_resource = store
                    .data_mut()
                    .add_server(self.server.clone())
                    .map_err(|e| {
                        DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                            format!("Failed to create server: {e}"),
                        ))
                    })?;
                let consumed_args = build_consumed_args_from_context(context);
                let args_resource = store
                    .data_mut()
                    .add_consumed_args(consumed_args)
                    .map_err(|e| {
                        DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                            format!("Failed to create args: {e}"),
                        ))
                    })?;

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
                            DISPATCHER_PARSE_EXCEPTION.create_without_context(
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
                                    Err(DISPATCHER_PARSE_EXCEPTION.create_without_context(
                                        TextComponent::text(format!(
                                            "Invalid consumption: {value:?}"
                                        )),
                                    ))
                                }
                                CommandErrorWit::InvalidRequirement => {
                                    Err(DISPATCHER_PARSE_EXCEPTION
                                        .create_without_context(TextComponent::text("Invalid requirement")))
                                }
                                CommandErrorWit::PermissionDenied => {
                                    Err(DISPATCHER_PARSE_EXCEPTION
                                        .create_without_context(TextComponent::text("Permission denied")))
                                }
                                CommandErrorWit::CommandFailed(resource) => {
                                    Err(DISPATCHER_PARSE_EXCEPTION.create_without_context(
                                        resource.consume(store.data_mut()).provider,
                                    ))
                                }
                            },
                        }
                    }
                }
            })
        })
    }
}

pub struct WasmCommandSuggestionProvider {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
    pub server: Arc<Server>,
}

impl SuggestionProvider for WasmCommandSuggestionProvider {
    fn suggest(&self, context: &CommandContext, builder: SuggestionsBuilder) -> Suggestions {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut store = self.plugin.store.lock().await;

                let sender_resource = match store
                    .data_mut()
                    .add_command_sender(context.source.output.clone())
                {
                    Ok(resource) => resource,
                    Err(error) => {
                        tracing::error!(
                            "Failed to create command sender resource for suggestions: {error}"
                        );
                        return builder.build();
                    }
                };
                let server_resource = match store.data_mut().add_server(self.server.clone()) {
                    Ok(resource) => resource,
                    Err(error) => {
                        tracing::error!(
                            "Failed to create server resource for suggestions: {error}"
                        );
                        return builder.build();
                    }
                };

                let input = &context.input;
                let request = SuggestionRequest {
                    input: input.clone(),
                    cursor: input.len().try_into().unwrap_or(u32::MAX),
                    start: builder.start.try_into().unwrap_or(u32::MAX),
                    remaining: builder.remaining().to_string(),
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
                        return builder.build();
                    }
                };

                let mut result_builder = builder;
                for suggestion in response.values {
                    if let Some(tooltip) = suggestion.tooltip {
                        let text = tooltip.consume(store.data_mut()).provider;
                        result_builder =
                            result_builder.suggest_with_tooltip(suggestion.value, text);
                    } else {
                        result_builder = result_builder.suggest(suggestion.value);
                    }
                }

                result_builder.build()
            })
        })
    }
}

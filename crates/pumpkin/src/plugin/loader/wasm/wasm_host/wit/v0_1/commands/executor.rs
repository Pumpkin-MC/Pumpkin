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
        PluginInstance, WasmPlugin,
        args::build_consumed_args_from_context,
        state::PluginHostState,
        wit::v0_1::pumpkin::plugin::command::{CommandError as CommandErrorWit, SuggestionRequest},
    },
    server::Server,
};

fn map_command_result(
    state: &mut PluginHostState,
    result: Result<i32, CommandErrorWit>,
) -> CommandExecutorResult {
    match result {
        Ok(value) => Ok(value),
        Err(CommandErrorWit::InvalidConsumption(value)) => Err(DISPATCHER_PARSE_EXCEPTION
            .create_without_context(TextComponent::text(format!(
                "Invalid consumption: {value:?}"
            )))),
        Err(CommandErrorWit::InvalidRequirement) => Err(DISPATCHER_PARSE_EXCEPTION
            .create_without_context(TextComponent::text("Invalid requirement"))),
        Err(CommandErrorWit::PermissionDenied) => Err(DISPATCHER_PARSE_EXCEPTION
            .create_without_context(TextComponent::text("Permission denied"))),
        Err(CommandErrorWit::CommandFailed(resource)) => Err(DISPATCHER_PARSE_EXCEPTION
            .create_without_context(
                state
                    .take(resource)
                    .expect("todo: make this method return a result"),
            )),
    }
}

pub struct WasmCommandExecutor {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
    pub server: Arc<Server>,
}

impl CommandExecutor for WasmCommandExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let sender = context.source.output.clone();
        let server = self.server.clone();
        let consumed_args = build_consumed_args_from_context(context);
        let handler_id = self.handler_id;
        let function = match self.plugin.plugin_instance.as_ref() {
            PluginInstance::V0_1(plugin) => plugin.func_handle_command(),
        };

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.plugin
                    .store
                    .call_guest(move |mut guest| {
                        Box::pin(async move {
                            let (sender_resource, server_resource, args_resource) =
                                guest.with(|mut store| {
                                    let sender_resource = store.data_mut().add(sender)?;
                                    let server_resource = match store.data_mut().add(server) {
                                        Ok(resource) => resource,
                                        Err(error) => {
                                            store.data_mut().discard(sender_resource);
                                            return Err(error);
                                        }
                                    };
                                    let args_resource = match store.data_mut().add(consumed_args) {
                                        Ok(resource) => resource,
                                        Err(error) => {
                                            store.data_mut().discard(sender_resource);
                                            store.data_mut().discard(server_resource);
                                            return Err(error);
                                        }
                                    };
                                    Ok::<_, wasmtime::Error>((
                                        sender_resource,
                                        server_resource,
                                        args_resource,
                                    ))
                                })?;

                            let result = guest
                                .call(
                                    function,
                                    (handler_id, sender_resource, server_resource, args_resource),
                                )
                                .await;

                            guest.with(|mut store| {
                                result.map(|(result,)| map_command_result(store.data_mut(), result))
                            })
                        })
                    })
                    .await
                    .map_err(|error| {
                        DISPATCHER_PARSE_EXCEPTION.create_without_context(
                            TextComponent::text(format!(
                                "Wasm command failed with following error: {error}"
                            ))
                            .color(Color::Named(NamedColor::Red)),
                        )
                    })?
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
        let sender = context.source.output.clone();
        let server = self.server.clone();
        let input = context.input.clone();
        let request = SuggestionRequest {
            input: input.clone(),
            cursor: input.len().try_into().unwrap_or(u32::MAX),
            start: builder.start.try_into().unwrap_or(u32::MAX),
            remaining: builder.remaining().to_string(),
        };
        let handler_id = self.handler_id;
        let function = match self.plugin.plugin_instance.as_ref() {
            PluginInstance::V0_1(plugin) => plugin.func_handle_command_suggestion(),
        };

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self
                    .plugin
                    .store
                    .call_guest(move |mut guest| {
                        Box::pin(async move {
                            let (sender_resource, server_resource) = guest.with(|mut store| {
                                let sender_resource = store.data_mut().add(sender)?;
                                let server_resource = match store.data_mut().add(server) {
                                    Ok(resource) => resource,
                                    Err(error) => {
                                        store.data_mut().discard(sender_resource);
                                        return Err(error);
                                    }
                                };
                                Ok::<_, wasmtime::Error>((sender_resource, server_resource))
                            })?;
                            let response = guest
                                .call(
                                    function,
                                    (handler_id, sender_resource, server_resource, request),
                                )
                                .await
                                .map(|(response,)| response);
                            guest.with(|mut store| {
                                response.map(|response| {
                                    let mut builder = builder;
                                    for suggestion in response.values {
                                        if let Some(tooltip) = suggestion.tooltip {
                                            let text = store
                                                .data_mut()
                                                .take(tooltip)
                                                .expect("Invalid text component");
                                            builder = builder
                                                .suggest_with_tooltip(suggestion.value, text);
                                        } else {
                                            builder = builder.suggest(suggestion.value);
                                        }
                                    }
                                    builder.build()
                                })
                            })
                        })
                    })
                    .await
                {
                    Ok(suggestions) => suggestions,
                    Err(error) => {
                        tracing::error!("Wasm command suggestion failed: {error}");
                        Suggestions::empty()
                    }
                }
            })
        })
    }
}

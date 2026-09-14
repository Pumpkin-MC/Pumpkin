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
        state::{CommandSenderResource, ConsumedArgsResource, PluginHostState, ServerResource},
        wit::v0_1::pumpkin::plugin::command::{CommandError as CommandErrorWit, SuggestionRequest},
    },
    server::Server,
};

fn remove_resource<T: 'static>(state: &mut PluginHostState, rep: u32) {
    let _ = state
        .resource_table
        .delete::<T>(wasmtime::component::Resource::new_own(rep));
}

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
        Err(CommandErrorWit::CommandFailed(resource)) => {
            Err(DISPATCHER_PARSE_EXCEPTION.create_without_context(resource.consume(state).provider))
        }
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
                            let (sender_resource, server_resource, args_resource, reps) = guest
                                .with(|mut store| {
                                    let sender_resource =
                                        store.data_mut().add_command_sender(sender)?;
                                    let sender_rep = sender_resource.rep();
                                    let server_resource = match store.data_mut().add_server(server)
                                    {
                                        Ok(resource) => resource,
                                        Err(error) => {
                                            remove_resource::<CommandSenderResource>(
                                                store.data_mut(),
                                                sender_rep,
                                            );
                                            return Err(error);
                                        }
                                    };
                                    let server_rep = server_resource.rep();
                                    let args_resource =
                                        match store.data_mut().add_consumed_args(consumed_args) {
                                            Ok(resource) => resource,
                                            Err(error) => {
                                                remove_resource::<ServerResource>(
                                                    store.data_mut(),
                                                    server_rep,
                                                );
                                                remove_resource::<CommandSenderResource>(
                                                    store.data_mut(),
                                                    sender_rep,
                                                );
                                                return Err(error);
                                            }
                                        };
                                    let reps = (sender_rep, server_rep, args_resource.rep());
                                    Ok::<_, wasmtime::Error>((
                                        sender_resource,
                                        server_resource,
                                        args_resource,
                                        reps,
                                    ))
                                })?;

                            let result = guest
                                .call(
                                    function,
                                    (handler_id, sender_resource, server_resource, args_resource),
                                )
                                .await;

                            guest.with(|mut store| {
                                let result = result
                                    .map(|(result,)| map_command_result(store.data_mut(), result));
                                remove_resource::<CommandSenderResource>(store.data_mut(), reps.0);
                                remove_resource::<ServerResource>(store.data_mut(), reps.1);
                                remove_resource::<ConsumedArgsResource>(store.data_mut(), reps.2);
                                result
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
                            let (sender_resource, server_resource, reps) =
                                guest.with(|mut store| {
                                    let sender_resource =
                                        store.data_mut().add_command_sender(sender)?;
                                    let sender_rep = sender_resource.rep();
                                    let server_resource = match store.data_mut().add_server(server)
                                    {
                                        Ok(resource) => resource,
                                        Err(error) => {
                                            remove_resource::<CommandSenderResource>(
                                                store.data_mut(),
                                                sender_rep,
                                            );
                                            return Err(error);
                                        }
                                    };
                                    let reps = (sender_rep, server_resource.rep());
                                    Ok::<_, wasmtime::Error>((
                                        sender_resource,
                                        server_resource,
                                        reps,
                                    ))
                                })?;
                            let response = guest
                                .call(
                                    function,
                                    (handler_id, sender_resource, server_resource, request),
                                )
                                .await
                                .map(|(response,)| response);
                            guest.with(|mut store| {
                                let suggestions = response.map(|response| {
                                    let mut builder = builder;
                                    for suggestion in response.values {
                                        if let Some(tooltip) = suggestion.tooltip {
                                            let text = tooltip.consume(store.data_mut()).provider;
                                            builder = builder
                                                .suggest_with_tooltip(suggestion.value, text);
                                        } else {
                                            builder = builder.suggest(suggestion.value);
                                        }
                                    }
                                    builder.build()
                                });
                                remove_resource::<CommandSenderResource>(store.data_mut(), reps.0);
                                remove_resource::<ServerResource>(store.data_mut(), reps.1);
                                suggestions
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

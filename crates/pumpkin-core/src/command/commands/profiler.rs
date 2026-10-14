use std::path::Path;

use pumpkin_profiler::{Operation, PROFILER};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::TextComponent,
};

use crate::command::{
    argument_builder::{ArgumentBuilder, command, literal},
    context::command_context::CommandContext,
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
};

const DESCRIPTION: &str = "Reports heap memory and tick CPU timings.";

struct ProfilerExecutor(Operation);

impl CommandExecutor for ProfilerExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let request = match PROFILER.try_reserve() {
            Ok(request) => request,
            Err(error) => {
                context
                    .source
                    .send_error(TextComponent::text(format!("Profiler: {error}")));
                return Ok(0);
            }
        };
        let server = context.server().clone();
        let worker_server = server.clone();
        let source = context.source.clone();
        let operation = self.0;
        server.spawn_task(async move {
            let result = tokio::task::spawn_blocking(move || {
                let counters = crate::server::profiler::counters(&worker_server);
                request.execute(
                    operation,
                    Path::new("debug/profiler"),
                    concat!(env!("CARGO_PKG_VERSION"), " ", env!("GIT_HASH_FULL")),
                    &counters,
                )
            })
            .await;
            pumpkin_profiler::without_tracking(|| match result {
                Ok(Ok(message)) => source.send_feedback(TextComponent::text(message), false),
                Ok(Err(error)) => {
                    source.send_error(TextComponent::text(format!("Profiler: {error}")))
                }
                Err(error) => source.send_error(TextComponent::text(format!(
                    "Profiler task failed: {error}"
                ))),
            });
        });
        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    for (name, permission) in [
        ("profiler", "pumpkin:command.profiler"),
        ("memory-profiler", "pumpkin:command.memory-profiler"),
    ] {
        registry.register_permission_or_panic(Permission::new(
            permission,
            DESCRIPTION,
            PermissionDefault::Op(PermissionLvl::Four),
        ));
        dispatcher.register(
            command(name, DESCRIPTION)
                .requires(permission)
                .then(literal("start").executes(ProfilerExecutor(Operation::Start)))
                .then(literal("snapshot").executes(ProfilerExecutor(Operation::Snapshot)))
                .then(literal("stop").executes(ProfilerExecutor(Operation::Stop)))
                .then(literal("status").executes(ProfilerExecutor(Operation::Status)))
                .then(literal("cancel").executes(ProfilerExecutor(Operation::Cancel))),
        );
    }
}

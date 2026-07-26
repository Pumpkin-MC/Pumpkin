use crate::command::argument_builder::{
    ArgumentBuilder, CommandArgumentBuilder, LiteralArgumentBuilder, RequiredArgumentBuilder,
};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::error_types::DISPATCHER_UNKNOWN_COMMAND;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

#[tokio::test]
async fn unknown_command() {
    let mut dispatcher = CommandDispatcher::new();
    dispatcher
        .register(CommandArgumentBuilder::new("unknown", "A command without an executor").build());
    let source = CommandSource::dummy();
    let result = dispatcher.execute_input("unknown", &source).await;
    assert!(result.is_err_and(|error| error.error_type == &DISPATCHER_UNKNOWN_COMMAND));
}

#[tokio::test]
async fn simple_command() {
    let mut dispatcher = CommandDispatcher::new();
    let executor: for<'c> fn(&'c CommandContext) -> CommandExecutorResult<'c> =
        |_| Box::pin(async move { Ok(1) });
    dispatcher
        .register(CommandArgumentBuilder::new("simple", "A simple command").executes(executor));
    let source = CommandSource::dummy();
    let result = dispatcher.execute_input("simple", &source).await;
    assert_eq!(result, Ok(1));
}

#[tokio::test]
async fn arithmetic_command() {
    enum Operation {
        Add,
        Subtract,
        Multiply,
        Divide,
    }

    struct Executor(Operation);
    impl CommandExecutor for Executor {
        fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
            Box::pin(async move {
                let operand1: i32 = *context.get_argument("operand1")?;
                let operand2: i32 = *context.get_argument("operand2")?;
                Ok(match self.0 {
                    Operation::Add => operand1 + operand2,
                    Operation::Subtract => operand1 - operand2,
                    Operation::Multiply => operand1 * operand2,
                    Operation::Divide => operand1 / operand2,
                })
            })
        }
    }

    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(
        CommandArgumentBuilder::new(
            "arithmetic",
            "A command which adds two integers, returning the result",
        )
        .then(
            RequiredArgumentBuilder::new("operand1", IntegerArgumentType::any())
                .then(
                    LiteralArgumentBuilder::new("+").then(
                        RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                            .executes(Executor(Operation::Add)),
                    ),
                )
                .then(
                    LiteralArgumentBuilder::new("-").then(
                        RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                            .executes(Executor(Operation::Subtract)),
                    ),
                )
                .then(
                    LiteralArgumentBuilder::new("*").then(
                        RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                            .executes(Executor(Operation::Multiply)),
                    ),
                )
                .then(
                    LiteralArgumentBuilder::new("/").then(
                        RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                            .executes(Executor(Operation::Divide)),
                    ),
                ),
        ),
    );
    let source = CommandSource::dummy();
    assert_eq!(
        dispatcher.execute_input("arithmetic 3 + -7", &source).await,
        Ok(-4)
    );
    assert_eq!(
        dispatcher.execute_input("arithmetic 4 - -8", &source).await,
        Ok(12)
    );
    assert_eq!(
        dispatcher.execute_input("arithmetic 2 * 9", &source).await,
        Ok(18)
    );
    assert_eq!(
        dispatcher.execute_input("arithmetic 9 / 2", &source).await,
        Ok(4)
    );
}

#[tokio::test]
async fn alias_simple() {
    let mut dispatcher = CommandDispatcher::new();
    let executor: for<'c> fn(&'c CommandContext) -> CommandExecutorResult<'c> =
        |_| Box::pin(async move { Ok(1) });
    dispatcher.register(CommandArgumentBuilder::new("a", "A command").executes(executor));
    // Note that we CANNOT use redirect here as node itself needs to execute the command,
    // not its 'children'.
    dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").executes(executor));
    let source = CommandSource::dummy();
    assert_eq!(dispatcher.execute_input("a", &source).await, Ok(1));
    assert_eq!(dispatcher.execute_input("b", &source).await, Ok(1));
}

#[tokio::test]
async fn alias_complex() {
    struct Executor;
    impl CommandExecutor for Executor {
        fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
            Box::pin(async move { Ok(*context.get_argument("result")?) })
        }
    }

    let mut dispatcher = CommandDispatcher::new();

    let a = dispatcher.register(CommandArgumentBuilder::new("a", "A command").then(
        RequiredArgumentBuilder::new("result", IntegerArgumentType::any()).executes(Executor),
    ));
    // Note that this time, we SHOULD use redirect - it is leading to another node having `command`.
    dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").redirect(a));
    let source = CommandSource::dummy();
    assert_eq!(dispatcher.execute_input("a 5", &source).await, Ok(5));
    assert_eq!(dispatcher.execute_input("b 7", &source).await, Ok(7));
}

#[tokio::test]
async fn recurse() {
    struct Executor;
    impl CommandExecutor for Executor {
        fn execute<'a>(&'a self, _context: &'a CommandContext) -> CommandExecutorResult<'a> {
            Box::pin(async move { Ok(1) })
        }
    }

    let mut dispatcher = CommandDispatcher::new();

    let mut builder = CommandArgumentBuilder::new(
        "recurse",
        "Recurses itself, doing nothing with the numbers provided",
    )
    .executes(Executor);

    let id = builder.id();
    builder = builder.then(
        RequiredArgumentBuilder::new("value", IntegerArgumentType::any())
            .executes(Executor)
            .redirect(id),
    );

    dispatcher.register(builder);

    let source = CommandSource::dummy();
    assert_eq!(dispatcher.execute_input("recurse", &source).await, Ok(1));
    assert_eq!(dispatcher.execute_input("recurse 4", &source).await, Ok(1));
    assert_eq!(
        dispatcher.execute_input("recurse 9 -1", &source).await,
        Ok(1)
    );
    assert_eq!(
        dispatcher
            .execute_input("recurse 9 7 -6 5 -4", &source)
            .await,
        Ok(1)
    );
    assert_eq!(
        dispatcher
            .execute_input("recurse 1 2 4 8 16 32 64 128 256 512", &source)
            .await,
        Ok(1)
    );
}

#[tokio::test]
async fn completion_suggestions_at_end() {
    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(CommandArgumentBuilder::new("alpha", "First command").build());
    dispatcher.register(CommandArgumentBuilder::new("beta", "Second command").build());
    let source = CommandSource::dummy();
    let parsed = dispatcher.parse_input("al", &source).await;
    let suggestions = dispatcher.get_completion_suggestions_at_end(parsed).await;
    assert_eq!(suggestions.suggestions.len(), 1);
    assert_eq!(
        suggestions.suggestions[0].text.cached_text().as_str(),
        "alpha"
    );
}

#[test]
fn all_commands_sorted_listing() {
    let mut dispatcher = CommandDispatcher::new();
    dispatcher.register(CommandArgumentBuilder::new("zulu", "Last").build());
    dispatcher.register(CommandArgumentBuilder::new("alfa", "First").build());
    let commands = dispatcher.get_all_commands();
    let names: Vec<&str> = commands.keys().copied().collect();
    assert_eq!(names, vec!["alfa", "zulu"]);
    assert_eq!(commands.get("alfa").copied(), Some("First"));
}

#[tokio::test]
async fn usage_of_command_with_argument() {
    let mut dispatcher = CommandDispatcher::new();
    let executor: for<'c> fn(&'c CommandContext) -> CommandExecutorResult<'c> =
        |_| Box::pin(async move { Ok(1) });
    let id = dispatcher.register(
        CommandArgumentBuilder::new("greet", "Greets with a value").then(
            RequiredArgumentBuilder::new("value", IntegerArgumentType::any()).executes(executor),
        ),
    );
    let source = CommandSource::dummy();
    let usage = dispatcher.get_usage_of_command(id, &source).await;
    assert_eq!(usage.as_deref(), Some("/greet <value>"));
}

#[tokio::test]
async fn register_with_aliases_redirects() {
    let mut dispatcher = CommandDispatcher::new();
    let executor: for<'c> fn(&'c CommandContext) -> CommandExecutorResult<'c> =
        |_| Box::pin(async move { Ok(3) });
    dispatcher.register_with_aliases(
        CommandArgumentBuilder::new("origin", "A command").executes(executor),
        &["alias"],
    );
    let source = CommandSource::dummy();
    assert_eq!(dispatcher.execute_input("origin", &source).await, Ok(3));
    assert_eq!(dispatcher.execute_input("alias", &source).await, Ok(3));
}

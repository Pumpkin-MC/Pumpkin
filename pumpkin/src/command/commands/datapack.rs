use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_datapack::command::{DatapackCommand, EnablePosition, ListMode};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Manage datapacks.";
const ARG_NAME: &str = "name";

struct ListExecutor;
impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let lines = DatapackCommand::list(&server.datapack_manager, ListMode::Available)
                .await
                .unwrap_or_default();
            for line in lines {
                sender.send_message(TextComponent::text(line)).await;
            }
            Ok(0)
        })
    }
}

struct ListEnabledExecutor;
impl CommandExecutor for ListEnabledExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let lines = DatapackCommand::list(&server.datapack_manager, ListMode::Enabled)
                .await
                .unwrap_or_default();
            for line in lines {
                sender.send_message(TextComponent::text(line)).await;
            }
            Ok(0)
        })
    }
}

struct EnableExecutor;
impl CommandExecutor for EnableExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            match DatapackCommand::enable(&server.datapack_manager, name).await {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    server.save_datapack_config().await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

struct DisableExecutor;
impl CommandExecutor for DisableExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            match DatapackCommand::disable(&server.datapack_manager, name).await {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    server.save_datapack_config().await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

struct EnableFirstExecutor;
impl CommandExecutor for EnableFirstExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            match DatapackCommand::enable_at_position(
                &server.datapack_manager,
                name,
                EnablePosition::First,
            )
            .await
            {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    server.save_datapack_config().await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

struct EnableLastExecutor;
impl CommandExecutor for EnableLastExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            match DatapackCommand::enable_at_position(
                &server.datapack_manager,
                name,
                EnablePosition::Last,
            )
            .await
            {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    server.save_datapack_config().await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

struct EnableAfterExecutor;
impl CommandExecutor for EnableAfterExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            let Some(Arg::Simple(existing)) = args.get("existing") else {
                return Err(CommandError::InvalidConsumption(Some("existing".into())));
            };
            match DatapackCommand::enable_at_position(
                &server.datapack_manager,
                name,
                EnablePosition::After(existing.to_string()),
            )
            .await
            {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    server.save_datapack_config().await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

struct EnableBeforeExecutor;
impl CommandExecutor for EnableBeforeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            let Some(Arg::Simple(existing)) = args.get("existing") else {
                return Err(CommandError::InvalidConsumption(Some("existing".into())));
            };
            match DatapackCommand::enable_at_position(
                &server.datapack_manager,
                name,
                EnablePosition::Before(existing.to_string()),
            )
            .await
            {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    server.save_datapack_config().await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

struct CreateExecutor;
impl CommandExecutor for CreateExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };
            let Some(Arg::Simple(description)) = args.get("description") else {
                return Err(CommandError::InvalidConsumption(Some("description".into())));
            };
            match DatapackCommand::create(&server.datapack_manager, name, description) {
                Ok(msg) => {
                    sender.send_message(TextComponent::text(msg)).await;
                    Ok(1)
                }
                Err(e) => {
                    sender
                        .send_message(TextComponent::text(format!("§c{e}")))
                        .await;
                    Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "§c{e}"
                    ))))
                }
            }
        })
    }
}

const NAMES: [&str; 1] = ["datapack"];

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("list")
                .then(literal("available").execute(ListExecutor))
                .then(literal("enabled").execute(ListEnabledExecutor))
                .execute(ListExecutor),
        )
        .then(
            literal("enable").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(
                        literal("after").then(
                            argument("existing", SimpleArgConsumer).execute(EnableAfterExecutor),
                        ),
                    )
                    .then(literal("before").then(
                        argument("existing", SimpleArgConsumer).execute(EnableBeforeExecutor),
                    ))
                    .then(literal("first").execute(EnableFirstExecutor))
                    .then(literal("last").execute(EnableLastExecutor))
                    .execute(EnableExecutor),
            ),
        )
        .then(
            literal("disable").then(argument(ARG_NAME, SimpleArgConsumer).execute(DisableExecutor)),
        )
        .then(
            literal("create").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(argument("description", SimpleArgConsumer).execute(CreateExecutor)),
            ),
        )
}

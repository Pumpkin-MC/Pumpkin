use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandExecutor, CommandSender,
    args::{
        ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer, time::TimeArgumentConsumer,
    },
    dispatcher::CommandError,
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["tick"];
const DESCRIPTION: &str = "Controls or queries the game's ticking state.";

fn rate_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .name("rate")
        .min(1.0)
        .max(10000.0)
}

fn time_consumer() -> TimeArgumentConsumer {
    TimeArgumentConsumer
}

enum SubCommand {
    Query,
    Rate,
    Freeze(bool),
    StepDefault,
    StepTimed,
    StepStop,
    SprintTimed,
    SprintStop,
}

struct TickExecutor(SubCommand);

#[async_trait]
impl CommandExecutor for TickExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let manager = &server.tick_rate_manager;

        match self.0 {
            SubCommand::Query => {
                let tickrate = manager.tickrate();
                // In a real implementation you would get real tick time stats
                let avg_mspt = 0.0; // Placeholder for server.getAverageTickTimeNanos()

                sender
                    .send_message(TextComponent::translate(
                        "commands.tick.query.rate.running",
                        [
                            TextComponent::text(format!("{tickrate:.1}")),
                            TextComponent::text(format!("{avg_mspt:.2}")),
                            TextComponent::text(format!("{:.1}", 1000.0 / tickrate)),
                        ],
                    ))
                    .await;
            }
            SubCommand::Rate => {
                let rate = BoundedNumArgumentConsumer::<f32>::find_arg(args, "rate")??;
                manager.set_tick_rate(server, rate).await;
                sender
                    .send_message(TextComponent::translate(
                        "commands.tick.rate.success",
                        [TextComponent::text(format!("{rate:.1}"))],
                    ))
                    .await;
            }
            SubCommand::Freeze(freeze) => {
                manager.set_frozen(server, freeze).await;
                let message_key = if freeze {
                    "commands.tick.status.frozen"
                } else {
                    "commands.tick.status.running"
                };
                sender
                    .send_message(TextComponent::translate(message_key, []))
                    .await;
            }
            SubCommand::StepDefault => {
                if manager.step_game_if_paused(server, 1).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.step.success",
                            [TextComponent::text("1")],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(TextComponent::translate("commands.tick.step.fail", []))
                        .await;
                }
            }
            SubCommand::StepTimed => {
                let ticks = TimeArgumentConsumer::find_arg(args, "time")?;
                if manager.step_game_if_paused(server, ticks).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.step.success",
                            [TextComponent::text(ticks.to_string())],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(TextComponent::translate("commands.tick.step.fail", []))
                        .await;
                }
            }
            SubCommand::StepStop => {
                if manager.stop_stepping(server).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.step.stop.success",
                            [],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(TextComponent::translate("commands.tick.step.stop.fail", []))
                        .await;
                }
            }
            SubCommand::SprintTimed => {
                let ticks = TimeArgumentConsumer::find_arg(args, "time")?;
                if manager.request_game_to_sprint(server, ticks as i64).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.sprint.stop.success",
                            [],
                        ))
                        .await;
                }
                sender
                    .send_message(TextComponent::translate(
                        "commands.tick.status.sprinting",
                        [],
                    ))
                    .await;
            }
            SubCommand::SprintStop => {
                if manager.stop_sprinting(server).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.sprint.stop.success",
                            [],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.sprint.stop.fail",
                            [],
                        ))
                        .await;
                }
            }
        }
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("query").execute(TickExecutor(SubCommand::Query)))
        .then(
            literal("rate")
                .then(argument("rate", rate_consumer()).execute(TickExecutor(SubCommand::Rate))),
        )
        .then(literal("freeze").execute(TickExecutor(SubCommand::Freeze(true))))
        .then(literal("unfreeze").execute(TickExecutor(SubCommand::Freeze(false))))
        .then(
            literal("step")
                .then(literal("stop").execute(TickExecutor(SubCommand::StepStop)))
                .then(
                    argument("time", time_consumer()).execute(TickExecutor(SubCommand::StepTimed)),
                )
                .execute(TickExecutor(SubCommand::StepDefault)),
        )
        .then(
            literal("sprint")
                .then(literal("stop").execute(TickExecutor(SubCommand::SprintStop)))
                .then(
                    argument("time", time_consumer())
                        .execute(TickExecutor(SubCommand::SprintTimed)),
                ),
        )
}

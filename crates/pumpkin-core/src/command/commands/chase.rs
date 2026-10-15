use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Connects multiple server instances for chase movement synchronization.";
const PERMISSION: &str = "minecraft:command.chase";

const ARG_HOST: &str = "host";
const ARG_BIND_ADDRESS: &str = "bind_address";
const ARG_PORT: &str = "port";

static CHASE_SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static CHASE_CLIENT_RUNNING: AtomicBool = AtomicBool::new(false);

struct ChaseStopExecutor;

impl CommandExecutor for ChaseStopExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mut stopped_anything = false;

        if CHASE_CLIENT_RUNNING.swap(false, Ordering::SeqCst) {
            context
                .source
                .send_feedback(TextComponent::text("You have now stopped chasing"), false);
            stopped_anything = true;
        }

        if CHASE_SERVER_RUNNING.swap(false, Ordering::SeqCst) {
            context
                .source
                .send_feedback(TextComponent::text("You are no longer being chased"), false);
            stopped_anything = true;
        }

        if !stopped_anything {
            context
                .source
                .send_feedback(TextComponent::text("Neither chasing nor being chased"), false);
        }

        Ok(0)
    }
}

struct ChaseLeadExecutor {
    has_bind_address: bool,
    has_port: bool,
}

impl CommandExecutor for ChaseLeadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        if CHASE_SERVER_RUNNING.load(Ordering::SeqCst) {
            context.source.send_feedback(
                TextComponent::text(
                    "Chase server is already running. Stop it using /chase stop",
                ),
                false,
            );
            return Ok(0);
        }
        if CHASE_CLIENT_RUNNING.load(Ordering::SeqCst) {
            context.source.send_feedback(
                TextComponent::text(
                    "You are already chasing someone. Stop it using /chase stop",
                ),
                false,
            );
            return Ok(0);
        }

        let bind_address = if self.has_bind_address {
            StringArgumentType::get(context, ARG_BIND_ADDRESS).unwrap_or("0.0.0.0")
        } else {
            "0.0.0.0"
        };

        let port: i32 = if self.has_port {
            IntegerArgumentType::get(context, ARG_PORT).unwrap_or(10000)
        } else {
            10000
        };

        CHASE_SERVER_RUNNING.store(true, Ordering::SeqCst);
        context.source.send_feedback(
            TextComponent::text(format!(
                "Chase server is now running on {bind_address}:{port}. Clients can follow you using /chase follow <ip> <port>"
            )),
            false,
        );

        Ok(port)
    }
}

struct ChaseFollowExecutor {
    has_host: bool,
    has_port: bool,
}

impl CommandExecutor for ChaseFollowExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        if CHASE_SERVER_RUNNING.load(Ordering::SeqCst) {
            context.source.send_feedback(
                TextComponent::text(
                    "Chase server is already running. Stop it using /chase stop",
                ),
                false,
            );
            return Ok(0);
        }
        if CHASE_CLIENT_RUNNING.load(Ordering::SeqCst) {
            context.source.send_feedback(
                TextComponent::text(
                    "You are already chasing someone. Stop it using /chase stop",
                ),
                false,
            );
            return Ok(0);
        }

        let host = if self.has_host {
            StringArgumentType::get(context, ARG_HOST).unwrap_or("localhost")
        } else {
            "localhost"
        };

        let port: i32 = if self.has_port {
            IntegerArgumentType::get(context, ARG_PORT).unwrap_or(10000)
        } else {
            10000
        };

        CHASE_CLIENT_RUNNING.store(true, Ordering::SeqCst);
        context.source.send_feedback(
            TextComponent::text(format!(
                "You are now chasing {host}:{port}. If that server does '/chase lead' then you will automatically go to the same position. Use '/chase stop' to stop chasing."
            )),
            false,
        );

        Ok(port)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Four),
    ));

    dispatcher.register(
        command("chase", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("follow")
                    .executes(ChaseFollowExecutor {
                        has_host: false,
                        has_port: false,
                    })
                    .then(
                        argument(ARG_HOST, StringArgumentType::SingleWord)
                            .executes(ChaseFollowExecutor {
                                has_host: true,
                                has_port: false,
                            })
                            .then(
                                argument(ARG_PORT, IntegerArgumentType::new(1, 65535))
                                    .executes(ChaseFollowExecutor {
                                        has_host: true,
                                        has_port: true,
                                    }),
                            ),
                    ),
            )
            .then(
                literal("lead")
                    .executes(ChaseLeadExecutor {
                        has_bind_address: false,
                        has_port: false,
                    })
                    .then(
                        argument(ARG_BIND_ADDRESS, StringArgumentType::SingleWord)
                            .executes(ChaseLeadExecutor {
                                has_bind_address: true,
                                has_port: false,
                            })
                            .then(
                                argument(ARG_PORT, IntegerArgumentType::new(1024, 65535))
                                    .executes(ChaseLeadExecutor {
                                        has_bind_address: true,
                                        has_port: true,
                                    }),
                            ),
                    ),
            )
            .then(literal("stop").executes(ChaseStopExecutor)),
    );
}

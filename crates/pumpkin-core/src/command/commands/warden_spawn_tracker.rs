use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::TextComponent,
};

use crate::command::{
    argument_builder::{ArgumentBuilder, argument, command, literal},
    argument_types::{core::integer::IntegerArgumentType, entity::EntityArgumentType},
    context::command_context::CommandContext,
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Modifies or resets the warden spawn tracker for players.";
const PERMISSION: &str = "minecraft:command.warden_spawn_tracker";

const ARG_WARNING_LEVEL: &str = "warning_level";
const ARG_TARGETS: &str = "targets";

struct ClearSelfExecutor;

impl CommandExecutor for ClearSelfExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.player_or_err()?;
        player.reset_warden_spawn_tracker();

        context.source.send_feedback(
            TextComponent::translate(
                "commands.warden_spawn_tracker.clear.success.single",
                [player.get_display_name()],
            ),
            true,
        );

        Ok(1)
    }
}

struct ClearTargetsExecutor;

impl CommandExecutor for ClearTargetsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, ARG_TARGETS)?;
        let count = targets.len();

        for target in &targets {
            target.reset_warden_spawn_tracker();
        }

        let msg = if count == 1 {
            TextComponent::translate(
                "commands.warden_spawn_tracker.clear.success.single",
                [targets[0].get_display_name()],
            )
        } else {
            TextComponent::translate(
                "commands.warden_spawn_tracker.clear.success.multiple",
                [TextComponent::text(count.to_string())],
            )
        };

        context.source.send_feedback(msg, true);
        Ok(count as i32)
    }
}

struct SetSelfExecutor;

impl CommandExecutor for SetSelfExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let warning_level = IntegerArgumentType::get(context, ARG_WARNING_LEVEL)?;
        let player = context.source.player_or_err()?;
        player.set_warden_warning_level(warning_level);

        context.source.send_feedback(
            TextComponent::translate(
                "commands.warden_spawn_tracker.set.success.single",
                [player.get_display_name()],
            ),
            true,
        );

        Ok(1)
    }
}

struct SetTargetsExecutor;

impl CommandExecutor for SetTargetsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let warning_level = IntegerArgumentType::get(context, ARG_WARNING_LEVEL)?;
        let targets = EntityArgumentType::get_players(context, ARG_TARGETS)?;
        let count = targets.len();

        for target in &targets {
            target.set_warden_warning_level(warning_level);
        }

        let msg = if count == 1 {
            TextComponent::translate(
                "commands.warden_spawn_tracker.set.success.single",
                [targets[0].get_display_name()],
            )
        } else {
            TextComponent::translate(
                "commands.warden_spawn_tracker.set.success.multiple",
                [TextComponent::text(count.to_string())],
            )
        };

        context.source.send_feedback(msg, true);
        Ok(count as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("warden_spawn_tracker", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("clear")
                    .then(
                        argument(ARG_TARGETS, EntityArgumentType::Players)
                            .executes(ClearTargetsExecutor),
                    )
                    .executes(ClearSelfExecutor),
            )
            .then(
                literal("set").then(
                    argument(ARG_WARNING_LEVEL, IntegerArgumentType::new(0, 4))
                        .then(
                            argument(ARG_TARGETS, EntityArgumentType::Players)
                                .executes(SetTargetsExecutor),
                        )
                        .executes(SetSelfExecutor),
                ),
            ),
    );
}

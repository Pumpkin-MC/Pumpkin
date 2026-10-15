use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Tests mob spawning for a specific category at a position.";
const PERMISSION: &str = "minecraft:command.debugmobspawning";

const CATEGORIES: &[&str] = &[
    "monster",
    "creature",
    "ambient",
    "axolotls",
    "underground_water_creature",
    "water_creature",
    "water_ambient",
    "misc",
];

struct DebugMobSpawningExecutor(&'static str);

impl CommandExecutor for DebugMobSpawningExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let pos = BlockPosArgumentType::get_block_pos(context, "at")?;
        context.source.send_feedback(
            TextComponent::text(format!(
                "Triggered mob spawning test for category '{}' at [{}, {}, {}]",
                self.0, pos.0.x, pos.0.y, pos.0.z
            )),
            true,
        );
        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let mut cmd = command("debugmobspawning", DESCRIPTION).requires(PERMISSION);

    for &cat in CATEGORIES {
        let cat_branch = literal(cat).then(
            argument("at", BlockPosArgumentType).executes(DebugMobSpawningExecutor(cat)),
        );
        cmd = cmd.then(cat_branch);
    }

    dispatcher.register(cmd);
}

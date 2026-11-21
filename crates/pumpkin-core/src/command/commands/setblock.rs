use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::block::BlockArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Place a block.";
const PERMISSION: &str = "minecraft:command.setblock";

const ERROR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SETBLOCK_FAILED,
    translation::java::COMMANDS_SETBLOCK_FAILED,
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Destroy,
    Keep,
    Replace,
    Strict,
}

const STRICT_SKIP_FLAGS: BlockFlags = BlockFlags::from_bits_truncate(
    BlockFlags::UPDATE_KNOWN_SHAPE.bits()
        | BlockFlags::SKIP_DROPS.bits()
        | BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK.bits()
        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK.bits(),
);

struct SetBlockExecutor(Mode);

impl CommandExecutor for SetBlockExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "pos")?;
        let block_input = BlockArgumentType::get(context, "block")?;
        let mode = self.0;
        let strict = mode == Mode::Strict;
        let world = context.source.world();

        if mode == Mode::Keep && !world.get_block_state(&pos).is_air() {
            return Err(ERROR_FAILED.create_without_context());
        }

        let place_needed = if mode == Mode::Destroy {
            world.break_block(&pos, None, BlockFlags::NOTIFY_ALL);
            !block_input.block.is_air() || !world.get_block_state(&pos).is_air()
        } else {
            true
        };

        let old_state_id = world.get_block_state_id(&pos);
        let place_flags = BlockFlags::NOTIFY_LISTENERS
            | if strict {
                STRICT_SKIP_FLAGS
            } else {
                BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK
            };

        if place_needed && !world.place_block_input(&pos, &block_input, place_flags) {
            return Err(ERROR_FAILED.create_without_context());
        }

        if !strict {
            world.update_neighbours_on_block_set(&pos, old_state_id);
        }

        world.flush_block_updates();
        context.source.send_feedback(
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_SETBLOCK_SUCCESS,
                pumpkin_data::translation::bedrock::COMMANDS_SETBLOCK_SUCCESS,
                [
                    TextComponent::text(pos.0.x.to_string()),
                    TextComponent::text(pos.0.y.to_string()),
                    TextComponent::text(pos.0.z.to_string()),
                ],
            ),
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

    dispatcher.register(
        command("setblock", DESCRIPTION).requires(PERMISSION).then(
            argument("pos", BlockPosArgumentType).then(
                argument("block", BlockArgumentType)
                    .executes(SetBlockExecutor(Mode::Replace))
                    .then(literal("destroy").executes(SetBlockExecutor(Mode::Destroy)))
                    .then(literal("keep").executes(SetBlockExecutor(Mode::Keep)))
                    .then(literal("replace").executes(SetBlockExecutor(Mode::Replace)))
                    .then(literal("strict").executes(SetBlockExecutor(Mode::Strict))),
            ),
        ),
    );
}

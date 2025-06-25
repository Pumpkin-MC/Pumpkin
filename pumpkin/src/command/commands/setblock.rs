use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::command::args::block::BlockArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["setblock"];

const DESCRIPTION: &str = "Place a block.";

const ARG_BLOCK: &str = "block";
const ARG_BLOCK_POS: &str = "pos";

#[derive(Clone, Copy)]
enum Mode {
    /// with particles + item drops
    Destroy,

    /// only replaces air
    Keep,

    /// default; without particles
    Replace,
}

struct Executor(Mode);

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
        let block_state_id = block.default_state.id;
        let pos = BlockPosArgumentConsumer::find_arg(args, ARG_BLOCK_POS)?;
        let mode = self.0;
        let world = match sender {
            CommandSender::Console | CommandSender::Rcon(_) => {
                let guard = server.worlds.read().await;

                guard
                    .first()
                    .cloned()
                    .ok_or(CommandError::InvalidRequirement)?
            }
            CommandSender::Player(player) => player.world().await,
        };
        let success = match mode {
            Mode::Destroy => {
                world
                    .clone()
                    .break_block(&pos, None, BlockFlags::SKIP_DROPS | BlockFlags::FORCE_STATE)
                    .await;
                world
                    .set_block_state(
                        &pos,
                        block_state_id,
                        BlockFlags::FORCE_STATE | BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                true
            }
            Mode::Replace => {
                world
                    .set_block_state(
                        &pos,
                        block_state_id,
                        BlockFlags::FORCE_STATE | BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                true
            }
            Mode::Keep => {
                let old_state = world.get_block_state(&pos).await;
                if old_state.is_air() {
                    world
                        .set_block_state(
                            &pos,
                            block_state_id,
                            BlockFlags::FORCE_STATE | BlockFlags::NOTIFY_NEIGHBORS,
                        )
                        .await;
                    true
                } else {
                    false
                }
            }
        };

        sender
            .send_message(if success {
                TextComponent::translate(
                    "commands.setblock.success",
                    [
                        TextComponent::text(pos.0.x.to_string()),
                        TextComponent::text(pos.0.y.to_string()),
                        TextComponent::text(pos.0.z.to_string()),
                    ],
                )
            } else {
                TextComponent::translate("commands.setblock.failed", [])
            })
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_BLOCK_POS, BlockPosArgumentConsumer).then(
            argument(ARG_BLOCK, BlockArgumentConsumer)
                .then(literal("replace").execute(Executor(Mode::Replace)))
                .then(literal("destroy").execute(Executor(Mode::Destroy)))
                .then(literal("keep").execute(Executor(Mode::Keep)))
                .execute(Executor(Mode::Replace)),
        ),
    )
}

use async_trait::async_trait;
use pumpkin_util::{math::vector3::Vector3, text::TextComponent};

use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{
        ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer,
        position_3d::Position3DArgumentConsumer, resource::particle::ParticleArgumentConsumer,
    },
    tree::{CommandTree, builder::argument},
};
const NAMES: [&str; 1] = ["particle"];

const DESCRIPTION: &str = "Spawns a Particle at position.";

const ARG_NAME: &str = "name";

const ARG_POS: &str = "pos";
const ARG_DELTA: &str = "delta";
const ARG_SPEED: &str = "speed";
const ARG_COUNT: &str = "count";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let particle = ParticleArgumentConsumer::find_arg(args, ARG_NAME)?;
        let pos = Position3DArgumentConsumer::find_arg(args, ARG_POS);
        let delta = Position3DArgumentConsumer::find_arg(args, ARG_DELTA);
        let speed = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_SPEED);
        let count = BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_COUNT);

        // TODO: Make this work in console
        if let Some(player) = sender.as_player() {
            let pos = pos.unwrap_or(player.living_entity.entity.pos.load());
            let delta = delta.unwrap_or(Vector3::new(0.0, 0.0, 0.0));
            let delta: Vector3<f32> = Vector3::new(delta.x as f32, delta.y as f32, delta.z as f32);
            let speed = speed.unwrap_or(Ok(0.0)).unwrap_or(0.0);
            let count = count.unwrap_or(Ok(0)).unwrap_or(0);

            player
                .world()
                .await
                .spawn_particle(pos, delta, speed, count, *particle)
                .await;
            sender
                .send_message(TextComponent::translate(
                    "commands.particle.success",
                    [TextComponent::text(format!("{particle:?}"))],
                ))
                .await;
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_NAME, ParticleArgumentConsumer)
            .execute(Executor)
            .then(
                argument(ARG_POS, Position3DArgumentConsumer)
                    .execute(Executor)
                    .then(
                        argument(ARG_DELTA, Position3DArgumentConsumer)
                            .execute(Executor)
                            .then(
                                argument(
                                    ARG_SPEED,
                                    BoundedNumArgumentConsumer::<f32>::new().min(0.0),
                                )
                                .execute(Executor)
                                .then(
                                    argument(
                                        ARG_COUNT,
                                        BoundedNumArgumentConsumer::<i32>::new().min(0),
                                    )
                                    .execute(Executor),
                                ),
                            ),
                    ),
            ),
        // TODO: Add NBT
    )
}

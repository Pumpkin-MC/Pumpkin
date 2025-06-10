use async_trait::async_trait;
use pumpkin_util::{math::vector3::Vector3, text::TextComponent};

use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{
            ConsumedArgs, FindArg, position_3d::Position3DArgumentConsumer,
            summonable_entities::SummonableEntitiesArgumentConsumer,
        },
        tree::{CommandTree, builder::argument},
    },
    entity::mob,
};
const NAMES: [&str; 1] = ["summon"];

const DESCRIPTION: &str = "Spawns a Entity at position.";

const ARG_ENTITY: &str = "entity";

const ARG_POS: &str = "pos";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let entity = SummonableEntitiesArgumentConsumer::find_arg(args, ARG_ENTITY)?;
        let pos = Position3DArgumentConsumer::find_arg(args, ARG_POS);
        let (world, pos) = match sender {
            CommandSender::Console | CommandSender::Rcon(_) => {
                let guard = server.worlds.read().await;
                let world = guard
                    .first()
                    .cloned()
                    .ok_or(CommandError::InvalidRequirement)?;
                let pos = {
                    let info = &world.level_info.read().await;
                    // default position for spawning a player, in this case for mob
                    pos.unwrap_or(Vector3::new(
                        f64::from(info.spawn_x),
                        f64::from(info.spawn_y) + 1.0,
                        f64::from(info.spawn_z),
                    ))
                };

                (world, pos)
            }
            CommandSender::Player(player) => {
                let pos = pos.unwrap_or(player.living_entity.entity.pos.load());

                (player.world().await, pos)
            }
        };
        let mob = mob::from_type(entity, pos, &world).await;
        world.spawn_entity(mob).await;

        sender
            .send_message(TextComponent::translate(
                "commands.summon.success",
                [TextComponent::text(format!("{entity:?}"))],
            ))
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_ENTITY, SummonableEntitiesArgumentConsumer)
            .execute(Executor)
            .then(argument(ARG_POS, Position3DArgumentConsumer).execute(Executor)),
        // TODO: Add NBT
    )
}

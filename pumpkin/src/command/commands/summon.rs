use pumpkin_util::{math::vector3::Vector3, text::TextComponent};
use uuid::Uuid;

use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{
            ConsumedArgs, FindArg, position_3d::Position3DArgumentConsumer,
            summonable_entities::SummonableEntitiesArgumentConsumer,
        },
        tree::{CommandTree, builder::argument},
    },
    entity::r#type::from_type,
};
const NAMES: [&str; 1] = ["summon"];

const DESCRIPTION: &str = "Spawns a Entity at position.";

const ARG_ENTITY: &str = "entity";

const ARG_POS: &str = "pos";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let entity_type = SummonableEntitiesArgumentConsumer::find_arg(args, ARG_ENTITY)?;
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
                            f64::from(info.spawn_x) + 0.5,
                            f64::from(info.spawn_y) + 1.0,
                            f64::from(info.spawn_z) + 0.5,
                        ))
                    };

                    (world, pos)
                }
                CommandSender::Player(player) => {
                    let pos = pos.unwrap_or(player.living_entity.entity.pos.load());

                    (player.world().clone(), pos)
                }
                CommandSender::CommandBlock(c, w) => {
                    (w.clone(), c.get_position().to_centered_f64())
                }
            };
            let entity = from_type(entity_type, pos, &world, Uuid::new_v4()).await;
            let name = entity.get_display_name().await;
            world.spawn_entity(entity).await;
            sender
                .send_message(TextComponent::translate("commands.summon.success", [name]))
                .await;

            Ok(())
        })
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

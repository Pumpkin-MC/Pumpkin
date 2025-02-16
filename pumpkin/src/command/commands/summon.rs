use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::{
    command::{
        args::{
            position_3d::Position3DArgumentConsumer,
            summonable_entities::SummonableEntitiesArgumentConsumer, ConsumedArgs, FindArg,
        },
        tree::builder::argument,
        tree::CommandTree,
        CommandError, CommandExecutor, CommandSender,
    },
    entity::mob,
};
const NAMES: [&str; 1] = ["summon"];

const DESCRIPTION: &str = "Spawns a Entity at position.";

const ARG_ENTITY: &str = "entity";

const ARG_POS: &str = "pos";

struct SummonExecutor;

#[async_trait]
impl CommandExecutor for SummonExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let entity = SummonableEntitiesArgumentConsumer::find_arg(args, ARG_ENTITY)?;
        let pos = Position3DArgumentConsumer::find_arg(args, ARG_POS);

        // TODO: Make this work in console
        if let Some(player) = sender.as_player() {
            let pos = pos.unwrap_or(player.living_entity.entity.pos.load());
            let mob = mob::from_type(entity, server, pos, &player.world().await).await;
            player.world().await.spawn_entity(mob).await;
            sender
                .send_message(TextComponent::translate(
                    "commands.summon.success",
                    [TextComponent::text(format!("{entity:?}"))],
                ))
                .await;
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_ENTITY, SummonableEntitiesArgumentConsumer)
            .execute(SummonExecutor)
            .then(argument(ARG_POS, Position3DArgumentConsumer).execute(SummonExecutor)),
        // TODO: Add NBT
    )
}

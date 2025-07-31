use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::tree::builder::literal;
use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{Arg, ConsumedArgs},
    tree::{CommandTree, builder::argument},
};
use crate::entity::NBTStorage;
use CommandError::InvalidConsumption;
use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["data"];
const DESCRIPTION: &str = "Query and modify data of entities and blocks";

const ARG_ENTITY: &str = "entity";

struct GetEntityDataExecutor;

#[async_trait]
impl CommandExecutor for GetEntityDataExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Entity(entity)) = args.get(&ARG_ENTITY) else {
            return Err(InvalidConsumption(Some(ARG_ENTITY.into())));
        };

        let data_storage = &**entity as &dyn NBTStorage;

        sender.send_message(display_data(data_storage).await?).await;
        Ok(())
    }
}

async fn display_data(storage: &dyn NBTStorage) -> Result<TextComponent, CommandError> {
    let mut nbt = NbtCompound::new();
    storage.write_nbt(&mut nbt).await;
    let display = format!("{nbt}");
    Ok(TextComponent::text(display))
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        literal("get").then(
            literal("entity")
                .then(argument(ARG_ENTITY, EntityArgumentConsumer).execute(GetEntityDataExecutor)),
        ),
    )
}

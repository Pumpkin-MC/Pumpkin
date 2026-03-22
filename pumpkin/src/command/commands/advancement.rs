use crate::command::CommandResult;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, message::MsgArgConsumer, players::PlayersArgumentConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::{SaveJSONConfiguration, banlist_serializer::BannedPlayerEntry},
    entity::player::Player,
    net::DisconnectReason,
};
use CommandError::InvalidConsumption;
use crate::command::args::resource::advancement::AdvancementArgumentConsumer;
use crate::command::tree::builder::literal;

const NAMES: [&str; 1] = ["advancement"];
const DESCRIPTION: &str = "manage advancement of the player";

const ARG_TARGET: &str = "player";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            (sender, server, targets.as_slice(), None).await
        })
    }
}

async fn grant_advancement(){

}

async fn revoke_advancement(){

}

fn get_parameter()->Self{
    argument(ARG_TARGET,PlayersArgumentConsumer)
        .then(literal("only").then(
            argument("advancement",AdvancementArgumentConsumer)
                .then(argument("criterion",GreedyStringArgument)
                )
        ))
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        literal("grant").execute(Executor),
    )
}
use crate::command::CommandResult;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
    }
};
use CommandError::InvalidConsumption;
use pumpkin::entity::player::Player;
use pumpkin_data::Advancement;
use crate::command::argument_types::core::string::StringArgumentType;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::PermissionLvl;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::args::Arg::Players;
use crate::command::args::resource::advancement::AdvancementArgumentConsumer;
use crate::command::argument_builder::{command,argument,literal, ArgumentBuilder};
use crate::command::node::dispatcher::CommandDispatcher;

const NAME: &str = "advancement";
const DESCRIPTION: &str = "manage advancement of the player";
const PERMISSION: &str = "minecraft:command.help";


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
            Ok(1)
        })
    }
}

async fn grant_advancement(advancement: &Advancement, sender: &Player) {
    sender
}

async fn revoke_advancement(){

}


pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry
        .register_permission(Permission::new(
            PERMISSION,
            DESCRIPTION,
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .expect("Permission should have registered successfully");

    dispatcher.register(
        command(NAME, DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("grant"))
            .then(literal("revoke"))
    );
}
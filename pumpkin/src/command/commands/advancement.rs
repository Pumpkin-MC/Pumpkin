use crate::command::args::{Arg, ConsumedArgs};
use crate::command::argument_builder::{argument, command, literal, ArgumentBuilder};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::CommandResult;
use crate::command::{
    CommandError, CommandExecutor, CommandSender,
};
use pumpkin::entity::player::Player;
use pumpkin_data::Advancement;
use pumpkin_protocol::java::client::play::ArgumentType;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::PermissionLvl;
use CommandError::InvalidConsumption;

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
            .then(literal("grant")
                .then(argument("targets",EntityArgumentType::Players)
                    .then(literal("only")
                        .then(argument("advancement",ArgumentType::ResourceKey {"advancement"})))
                    .then(literal("from"))
                    .then(literal("until"))
                    .then(literal("through"))
                    .then(literal("everything")))
            .then(literal("revoke")
                .then(argument("targets",EntityArgumentType::Players))
    );
}
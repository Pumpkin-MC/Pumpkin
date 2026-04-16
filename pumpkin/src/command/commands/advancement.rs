use std::sync::Arc;
use crate::command::argument_builder::{argument, command, literal, ArgumentBuilder};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::resource_key::{ResourceKeyArgument, ADVANCEMENT_REGISTRY};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::CommandExecutorResult;
use pumpkin_data::Advancement;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::PermissionLvl;
use pumpkin_util::text::TextComponent;
use crate::command::context::command_source::CommandSource;
use crate::entity::EntityBase;
use crate::entity::player::Player;

const NAME: &str = "advancement";
const DESCRIPTION: &str = "manage advancement of the player";
const PERMISSION: &str = "minecraft:command.help";


const ARG_TARGET: &str = "player";

enum Action {
    Grant,
    Revoke
}

impl Action {
    fn perform(&self, player: &Arc<Player>, advancements: &[&Advancement],grant_everything: bool) -> u32 {
        let mut count = 0;

        if !grant_everything {
            player.advancements.flush_dirty(true);
        }

        for advancement in advancements {
            if self.perform_single(player, &advancement) {
                count += 1;
            }
        }

        if !grant_everything {
            player.advancements.flush_dirty(false);
        }
        count
    }

    fn perform_single(&self, player: &Arc<Player>, advancement: &Advancement) -> bool {
        match self {
            Action::Grant => {
                let progress = player.advancements.get_or_start_progress(advancement);
                if progress.is_done() {
                    false
                } else {
                    player.advancements.award(advancement);
                    true
                }
            }

            Action::Revoke => {
                let progress = player.advancements.get_or_start_progress(advancement);
                if !progress.has_progress() {
                    false
                } else {
                    player.advancements.revoke(advancement);
                    true
                }
            }
        }

    }
    fn get_key(&self) -> &str{
        match self {
            Action::Grant => "grant",
            Action::Revoke => "revoke"
        }
    }

}

enum Mode {
    Only,
    Through,
    From,
    Until,
    Everything,
}

impl Mode {
    fn parents(&self) -> bool {
        match self {
            Mode::Only => false,
            Mode::Through => true,
            Mode::From => false,
            Mode::Until => true,
            Mode::Everything => true,
        }
    }

    fn children(&self) -> bool {
        match self {
            Mode::Only => false,
            Mode::Through => true,
            Mode::From => true,
            Mode::Until => false,
            Mode::Everything => true,
        }
    }
}

fn get_advancement<'a>(_context : &CommandContext, advancement :&'a Advancement, mode:Mode) -> Vec<&'a Advancement>{
    //TODO Advancement Tree
    let mut result = Vec::new();
    if mode.parents() {
        todo!()
    }
    result.push(advancement);
    if mode.children() {
        todo!()
    }
    result
}

fn perform_everything(context: Arc<CommandSource>, players: Vec<Arc<Player>>, action: Action, advancements : &[&Advancement])->CommandExecutorResult{
    perform(context,players,action,advancements,true)
}

fn perform(context: Arc<CommandSource>, targets: Vec<Arc<Player>>, action: Action, advancements : &[&Advancement], grant_everything:bool) -> CommandExecutorResult {
    Box::pin(async move {
        let mut i = 0;
        for player in &targets {
            i += action.perform(player, advancements, grant_everything);
        }
        if (i == 0) {
            if (advancements.size() == 1) {
                if (targets.len() == 1) {
                    ERROR_NO_ACTION_PERFORMED.create(Component.translatable(action.getKey() + ".one.to.one.failure", new Object[]{Advancement.name((AdvancementHolder)advancements.iterator().next()), ((ServerPlayer)targets.iterator().next()).getDisplayName()}));
                } else {
                    ERROR_NO_ACTION_PERFORMED.create(Component.translatable(action.getKey() + ".one.to.many.failure", new Object[]{Advancement.name((AdvancementHolder)advancements.iterator().next()), targets.size()}));
                }
            } else if (targets.size() == 1) {
                ERROR_NO_ACTION_PERFORMED.create(Component.translatable(action.getKey() + ".many.to.one.failure", new Object[]{advancements.size(), ((ServerPlayer)targets.iterator().next()).getDisplayName()}));
            } else {
                ERROR_NO_ACTION_PERFORMED.create(Component.translatable(action.getKey() + ".many.to.many.failure", new Object[]{advancements.size(), targets.size()}));
            }
        } else {
            if let [first_advancement] = advancements {
                if let [first_player] = targets.as_slice(){
                    context.send_feedback(TextComponent::translate(action.get_key() + ".one.to.one.success", [first_advancement.name(), first_player.get_display_name()]), true);
                } else {
                    context.send_feedback(TextComponent::translate(action.get_key() +  ".one.to.many.success", new Object[]{Advancement.name((AdvancementHolder)advancements.iterator().next()), targets.size()}), true);
                }
            } else if let [first] = targets.as_slice() {
                context.send_feedback(TextComponent::translate(action.get_key() + ".many.to.many.success", [advancements.size(), first.get_display_name()]), true).await;
            } else {
                context.send_feedback(TextComponent::translate(action.get_key() + ".many.to.many.success", [advancements.size(), targets.len()]), true).await;
            }
        }
        Ok(i)
    })
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
                        .then(argument("advancement",ResourceKeyArgument(ADVANCEMENT_REGISTRY))
                            .executes(|context| perform_everything(
                                context.source,
                                EntityArgumentType::get_players(context,"targets"),
                                Action::Grant,
                                get_advancement(context,ResourceKeyArgument::get_advancement(context,"advancement"),Mode::Only)
                            ))))))
                    .then(literal("from"))
                    .then(literal("until"))
                    .then(literal("through"))
                    .then(literal("everything")))
            .then(literal("revoke")
                .then(argument("targets",EntityArgumentType::Players))
    );
}
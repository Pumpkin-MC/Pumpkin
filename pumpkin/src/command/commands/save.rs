use std::sync::atomic::Ordering;

use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::literal;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES_SAVE_ALL: [&str; 1] = ["save-all"];
const NAMES_SAVE_OFF: [&str; 1] = ["save-off"];
const NAMES_SAVE_ON: [&str; 1] = ["save-on"];

const DESCRIPTION_SAVE_ALL: &str = "Saves the server to disk.";
const DESCRIPTION_SAVE_OFF: &str = "Disables automatic saving.";
const DESCRIPTION_SAVE_ON: &str = "Enables automatic saving.";

// /save-all
struct SaveAllExecutor {
    flush: bool,
}

impl CommandExecutor for SaveAllExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        let flush = self.flush;
        Box::pin(async move {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SAVE_SAVING,
                    [],
                ))
                .await;

            for world in server.worlds.load().iter() {
                world.level.should_save.store(true, Ordering::Relaxed);
                world.level.level_channel.notify();
            }

            if flush {
                // Flush: save entities and wait for chunk saver
                for world in server.worlds.load().iter() {
                    for entity in world.entities.load().iter() {
                        world.save_entity(entity).await;
                    }
                }
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SAVE_SUCCESS,
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

// /save-off
struct SaveOffExecutor;

impl CommandExecutor for SaveOffExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let mut any_changed = false;
            for world in server.worlds.load().iter() {
                if !world.level.saving_disabled.swap(true, Ordering::Relaxed) {
                    any_changed = true;
                }
            }

            if any_changed {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SAVE_DISABLED,
                        [],
                    ))
                    .await;
                Ok(1)
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SAVE_ALREADYOFF,
                        [],
                    ))
                    .await;
                Ok(0)
            }
        })
    }
}

// /save-on
struct SaveOnExecutor;

impl CommandExecutor for SaveOnExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let mut any_changed = false;
            for world in server.worlds.load().iter() {
                if world.level.saving_disabled.swap(false, Ordering::Relaxed) {
                    any_changed = true;
                }
            }

            if any_changed {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SAVE_ENABLED,
                        [],
                    ))
                    .await;
                Ok(1)
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SAVE_ALREADYON,
                        [],
                    ))
                    .await;
                Ok(0)
            }
        })
    }
}

pub fn init_command_tree_save_all() -> CommandTree {
    CommandTree::new(NAMES_SAVE_ALL, DESCRIPTION_SAVE_ALL)
        .execute(SaveAllExecutor { flush: false })
        .then(literal("flush").execute(SaveAllExecutor { flush: true }))
}

pub fn init_command_tree_save_off() -> CommandTree {
    CommandTree::new(NAMES_SAVE_OFF, DESCRIPTION_SAVE_OFF).execute(SaveOffExecutor)
}

pub fn init_command_tree_save_on() -> CommandTree {
    CommandTree::new(NAMES_SAVE_ON, DESCRIPTION_SAVE_ON).execute(SaveOnExecutor)
}

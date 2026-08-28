use pumpkin_data::{Enchantment, translation};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::command::args::bounded_num::{BoundedNumArgumentConsumer, NotInBounds};
use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::resource::enchantment::EnchantmentArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument_default_name;
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use pumpkin_data::data_component_impl::EnchantmentsImpl;

const NAMES: [&str; 1] = ["enchant"];
const DESCRIPTION: &str = "Adds an enchantment to a player's selected item, subject to the same restrictions as an anvil. Also works on any mob or entity holding a weapon/tool/armor in its main hand.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let targets = EntitiesArgumentConsumer.find_arg_default_name(args)?;
        let enchantment = EnchantmentArgumentConsumer.find_arg_default_name(args)?;
        let level = match enchantment_level_consumer().find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(level)) => level,
            Ok(Err(err)) => {
                let err_msg = match err {
                    NotInBounds::LowerBound(val, min) => TextComponent::translate_cross(
                        "argument.integer.low",
                        "argument.integer.low",
                        &[
                            TextComponent::text(min.to_string()),
                            TextComponent::text(val.to_string()),
                        ],
                    ),
                    NotInBounds::UpperBound(val, max) => TextComponent::translate_cross(
                        "argument.integer.big",
                        "argument.integer.big",
                        &[
                            TextComponent::text(max.to_string()),
                            TextComponent::text(val.to_string()),
                        ],
                    ),
                };

                return Err(CommandError::CommandFailed(err_msg));
            }
        };

        if level > enchantment.max_level {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_ENCHANT_FAILED_LEVEL,
                translation::bedrock::COMMANDS_ENCHANT_INVALIDLEVEL,
                [
                    TextComponent::text(level.to_string()),
                    TextComponent::text(enchantment.max_level.to_string()),
                ],
            );
            return Err(CommandError::CommandFailed(msg));
        }

        let mut successful_targets = 0;

        if targets.len() == 1 {
            return match enchant_target(&targets[0], enchantment, level) {
                Ok(()) => {
                    let msg = TextComponent::translate_cross(
                        translation::java::COMMANDS_ENCHANT_SUCCESS_SINGLE,
                        translation::bedrock::COMMANDS_ENCHANT_SUCCESS,
                        [
                            enchantment.get_fullname(level),
                            targets[0].get_display_name(),
                        ],
                    );
                    sender.send_message(msg);
                    Ok(1)
                }
                Err(e) => Err(e),
            };
        }

        for target in targets {
            if enchant_target(target, enchantment, level).is_ok() {
                successful_targets += 1;
            }
        }

        if successful_targets == 0 {
            return Err(commands_enchant_failed());
        }

        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ENCHANT_SUCCESS_MULTIPLE,
            translation::bedrock::COMMANDS_ENCHANT_SUCCESS,
            [
                enchantment.get_fullname(level),
                TextComponent::text(targets.len().to_string()),
            ],
        );
        sender.send_message(msg);
        Ok(successful_targets)
    }
}

const fn enchantment_level_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("level")
        .min(0)
        .max(i32::MAX)
}

fn commands_enchant_failed() -> CommandError {
    let msg = TextComponent::translate_cross(
        translation::java::COMMANDS_ENCHANT_FAILED,
        translation::bedrock::COMMANDS_ENCHANT_CANTENCHANT,
        [TextComponent::text("")],
    );
    CommandError::CommandFailed(msg)
}

fn enchant_target(
    target: &Arc<dyn EntityBase>,
    enchantment: &'static Enchantment,
    level: i32,
) -> Result<(), CommandError> {
    let Some(player) = target.get_player() else {
        return Err(commands_enchant_failed());
    };

    let mut item = player.inventory().held_item();

    if item.is_empty() {
        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ENCHANT_FAILED_ITEMLESS,
            translation::bedrock::COMMANDS_ENCHANT_NOITEM,
            [target.get_display_name()],
        );
        return Err(CommandError::CommandFailed(msg));
    }

    if !enchantment.can_enchant(item.item) {
        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ENCHANT_FAILED_INCOMPATIBLE,
            translation::bedrock::COMMANDS_ENCHANT_CANTENCHANT,
            [item.item.translated_name()],
        );
        return Err(CommandError::CommandFailed(msg));
    }

    if let Some(data) = item.get_data_component::<EnchantmentsImpl>()
        && !enchantment.is_enchantment_compatible(data)
    {
        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ENCHANT_FAILED_INCOMPATIBLE,
            translation::bedrock::COMMANDS_ENCHANT_CANTENCHANT,
            [item.item.translated_name()],
        );
        return Err(CommandError::CommandFailed(msg));
    }

    item.enchant(enchantment, level);
    let inventory = player.inventory();
    inventory.set_held_item(item.clone());

    player.sync_hand_slot(inventory.get_selected_slot() as usize, item);

    Ok(())
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(EntitiesArgumentConsumer).then(
            argument_default_name(EnchantmentArgumentConsumer)
                .then(argument_default_name(enchantment_level_consumer()).execute(Executor))
                .execute(Executor),
        ),
    )
}

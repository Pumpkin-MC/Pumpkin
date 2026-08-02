use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::bounded_num::{BoundedNumArgumentConsumer, NotInBounds};
use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::resource::enchantment::EnchantmentArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument_default_name;
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use pumpkin_data::data_component_impl::EnchantmentsImpl;

const NAMES: [&str; 1] = ["enchant"];
const DESCRIPTION: &str = "Adds an enchantment to a player's selected item, subject to the same restrictions as an anvil. Also works on any mob or entity holding a weapon/tool/armor in its main hand.";

struct Executor;

impl CommandExecutor for Executor {
    #[expect(clippy::too_many_lines)]
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
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

            let only_one = targets.len() == 1;
            let mut success = 0;

            for target in targets {
                let Some(living) = target.get_living_entity() else {
                    continue;
                };
                let player = target.get_player();
                let lock = if let Some(player) = player {
                    player.inventory.held_item()
                } else {
                    living
                        .entity_equipment
                        .lock()
                        .await
                        .get(&EquipmentSlot::MAIN_HAND)
                };
                let mut item = lock.lock().await;
                let mut updated_item = None;
                if item.is_empty() {
                    if only_one {
                        let msg = TextComponent::translate_cross(
                            translation::java::COMMANDS_ENCHANT_FAILED_ITEMLESS,
                            translation::bedrock::COMMANDS_ENCHANT_NOITEM,
                            [targets[0].get_display_name().await],
                        );
                        return Err(CommandError::CommandFailed(msg));
                    }
                    continue;
                }
                if !enchantment.can_enchant(item.item) {
                    if only_one {
                        let msg = TextComponent::translate_cross(
                            translation::java::COMMANDS_ENCHANT_FAILED_INCOMPATIBLE,
                            translation::bedrock::COMMANDS_ENCHANT_CANTENCHANT,
                            [item.item.translated_name()],
                        );
                        return Err(CommandError::CommandFailed(msg));
                    }
                    continue;
                }
                if let Some(data) = item.get_data_component::<EnchantmentsImpl>() {
                    if enchantment.is_enchantment_compatible(data) {
                        item.enchant(enchantment, level);
                        success += 1;
                        updated_item = Some(item.clone());
                    } else if only_one {
                        let msg = TextComponent::translate_cross(
                            translation::java::COMMANDS_ENCHANT_FAILED_INCOMPATIBLE,
                            translation::bedrock::COMMANDS_ENCHANT_CANTENCHANT,
                            [item.item.translated_name()],
                        );
                        return Err(CommandError::CommandFailed(msg));
                    }
                } else {
                    item.enchant(enchantment, level);
                    success += 1;
                    updated_item = Some(item.clone());
                }
                drop(item);
                if let Some(updated_item) = updated_item {
                    if let Some(player) = player {
                        player
                            .sync_hand_slot(
                                player.inventory.get_selected_slot() as usize,
                                updated_item,
                            )
                            .await;
                    } else {
                        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, updated_item)]);
                    }
                }
            }
            if success == 0 {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_ENCHANT_FAILED,
                    translation::bedrock::COMMANDS_ENCHANT_CANTENCHANT,
                    [TextComponent::text("")],
                );
                return Err(CommandError::CommandFailed(msg));
            }
            if only_one {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_ENCHANT_SUCCESS_SINGLE,
                    translation::bedrock::COMMANDS_ENCHANT_SUCCESS,
                    [
                        enchantment.get_fullname(level),
                        targets[0].get_display_name().await,
                    ],
                );
                sender.send_message(msg).await;
            } else {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_ENCHANT_SUCCESS_MULTIPLE,
                    translation::bedrock::COMMANDS_ENCHANT_SUCCESS,
                    [
                        enchantment.get_fullname(level),
                        TextComponent::text(targets.len().to_string()),
                    ],
                );
                sender.send_message(msg).await;
            }
            Ok(success)
        })
    }
}

const fn enchantment_level_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("level")
        .min(0)
        .max(i32::MAX)
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

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
    #[allow(clippy::too_many_lines)]
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
                        NotInBounds::LowerBound(val, min) => TextComponent::translate(
                            "argument.integer.low",
                            &[
                                TextComponent::text(min.to_string()),
                                TextComponent::text(val.to_string()),
                            ],
                        ),
                        NotInBounds::UpperBound(val, max) => TextComponent::translate(
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
                let msg = TextComponent::translate(
                    "commands.enchant.failed.level",
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
                // let Some(target) = target.get_living_entity() else {
                //     if only_one {
                //         let msg = TextComponent::translate(
                //             "commands.enchant.failed.entity",
                //             [targets[0].get_display_name().await],
                //         );
                //         sender.send_message(msg).await;
                //         return Ok(());
                //     }
                //     continue;
                // };
                // let lock = target.entity_equipment.lock().await.get(&EquipmentSlot::MAIN_HAND); TODO this dont work
                let Some(player) = target.get_player() else {
                    todo!()
                };
                let lock = player.inventory.held_item();
                let mut item = lock.lock().await;
                if item.is_empty() {
                    if only_one {
                        let msg = TextComponent::translate(
                            "commands.enchant.failed.itemless",
                            [targets[0].get_display_name().await],
                        );
                        return Err(CommandError::CommandFailed(msg));
                    }
                    continue;
                }
                if !enchantment.can_enchant(item.item) {
                    if only_one {
                        let msg = TextComponent::translate(
                            "commands.enchant.failed.incompatible",
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
                    } else if only_one {
                        let msg = TextComponent::translate(
                            "commands.enchant.failed.incompatible",
                            [item.item.translated_name()],
                        );
                        return Err(CommandError::CommandFailed(msg));
                    }
                } else {
                    item.enchant(enchantment, level);
                    success += 1;
                }
            }
            if success == 0 {
                let msg = TextComponent::translate("commands.enchant.failed", []);
                return Err(CommandError::CommandFailed(msg));
            }
            if only_one {
                let msg = TextComponent::translate(
                    "commands.enchant.success.single",
                    [
                        enchantment.get_fullname(level),
                        targets[0].get_display_name().await,
                    ],
                );
                sender.send_message(msg).await;
            } else {
                let msg = TextComponent::translate(
                    "commands.enchant.success.multiple",
                    [
                        enchantment.get_fullname(level),
                        TextComponent::text(targets.len().to_string()),
                    ],
                );
                sender.send_message(msg).await;
            }
            Ok(())
        })
    }
}

fn enchantment_level_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("level")
        .min(0)
        .max(i32::MAX)
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(EntitiesArgumentConsumer).then(
            argument_default_name(EnchantmentArgumentConsumer)
                .then(argument_default_name(enchantment_level_consumer()).execute(Executor))
                .execute(Executor),
        ),
    )
}

use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;

use crate::block::BrokenArgs;
use crate::block::{BlockBehaviour, BlockFuture};
use crate::entity::Entity;

#[pumpkin_block_from_tag("c:cobblestones/infested")]
pub struct InfestedBlock;

impl BlockBehaviour for InfestedBlock {
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.player.gamemode.load() == GameMode::Creative {
                return;
            }
            if !args.world.level_info.load().game_rules.block_drops {
                return;
            }

            let tool = args.player.inventory.held_item();
            let prevents_spawn = {
                let tool = tool.lock().await;
                tool.get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
                    .is_some_and(|enchantments| {
                        enchantments.enchantment.iter().any(|(enchantment, _)| {
                            enchantment
                                .has_tag(&tag::Enchantment::MINECRAFT_PREVENTS_INFESTED_SPAWNS)
                        })
                    })
            };
            if prevents_spawn {
                return;
            }

            let entity = Entity::new(
                args.world.clone(),
                args.position.0.to_f64(),
                &EntityType::SILVERFISH,
            );

            args.world.spawn_entity(Arc::new(entity)).await;
        })
    }
}

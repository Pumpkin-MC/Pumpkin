use super::MobEntity;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use pumpkin_data::item_stack::ItemStack;
use std::sync::Arc;

impl MobEntity {
    pub async fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let entity = &self.living_entity.entity;

        // If already leashed to player, right-clicking unleashes the mob
        let currently_leashed = {
            let guard = entity.leashed_to.lock().await;
            guard.is_some()
        };

        if currently_leashed {
            entity.unleash().await;
            let lead_item =
                pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::LEAD);
            entity
                .world
                .load()
                .drop_stack(&entity.block_pos.load(), lead_item)
                .await;
            return true;
        }

        // If holding a lead, leash the mob to the player
        if item_stack.item.registry_key == "lead"
            || item_stack.item.registry_key == "minecraft:lead"
        {
            let diff = entity.pos.load() - player.get_entity().pos.load();
            let dist_sq = diff.length_squared();
            if dist_sq <= Entity::LEASH_SNAP_DISTANCE * Entity::LEASH_SNAP_DISTANCE {
                entity.leash_to(player.clone() as Arc<dyn EntityBase>).await;
                if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                    item_stack.decrement(1);
                }
                return true;
            }
        }

        false
    }
}

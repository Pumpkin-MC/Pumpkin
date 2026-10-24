use crate::block::registry::BlockActionResult;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// Vanilla offsets a block-launched rocket's spawn point by this many blocks along the
/// clicked face, so it doesn't spawn embedded in the block.
const ROCKET_PLACEMENT_OFFSET: f64 = 0.15;

pub struct FireworkRocketItem;

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

impl ItemBehaviour for FireworkRocketItem {
    // A firework rocket always launches, whether or not the player is gliding - it's a
    // functional item in its own right (decorative in the open, a boost while gliding), not
    // an elytra-only action. The boost itself is gated separately, inside the spawned
    // entity's own tick logic (`FireworkRocketEntity::tick`), based on the owner's
    // fall-flying state at the time - so spawning unconditionally here is what lets both
    // cases (plain launch, and launch-while-gliding boost) share one code path.
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if player.get_entity().is_fall_flying() {
            return BlockActionResult::Pass;
        }
        let world = player.world();
        // The exact click point, nudged just outside the clicked face so the rocket doesn't
        // spawn embedded in the block. Ownerless (no shooter attached), matching a dispensed
        // rocket - this isn't the gliding-boost case, so it shouldn't follow the player around.
        let step = face.to_offset();
        let entity = Entity::new(
            world.clone(),
            Vector3::new(
                f64::from(location.0.x)
                    + f64::from(cursor_pos.x)
                    + f64::from(step.x) * ROCKET_PLACEMENT_OFFSET,
                f64::from(location.0.y)
                    + f64::from(cursor_pos.y)
                    + f64::from(step.y) * ROCKET_PLACEMENT_OFFSET,
                f64::from(location.0.z)
                    + f64::from(cursor_pos.z)
                    + f64::from(step.z) * ROCKET_PLACEMENT_OFFSET,
            ),
            &EntityType::FIREWORK_ROCKET,
        );
        let entity = FireworkRocketEntity::new_with_stack(entity, item.copy_with_count(1));
        world.spawn_entity(Arc::new(entity));
        item.decrement_unless_creative(player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    // Same reasoning as `use_on_block`: always launches, boost is gated inside the entity's tick.
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let mut held = match hand {
            pumpkin_util::Hand::Right => player.inventory().held_item(),
            pumpkin_util::Hand::Left => player.inventory().off_hand_item(),
        };
        if held.is_empty() || held.item.id != Item::FIREWORK_ROCKET.id {
            return;
        }

        let world = player.world();
        let entity = Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::FIREWORK_ROCKET,
        );
        let entity = FireworkRocketEntity::new_shot_with_stack(
            entity,
            player.get_entity(),
            held.copy_with_count(1),
        );
        world.spawn_entity(Arc::new(entity));

        held.decrement_unless_creative(player.gamemode.load(), 1);
        match hand {
            pumpkin_util::Hand::Right => player.inventory().set_held_item(held),
            pumpkin_util::Hand::Left => player
                .inventory()
                .set_stack_in_hand(pumpkin_util::Hand::Left, held),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

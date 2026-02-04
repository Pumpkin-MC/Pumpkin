use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::world::World;

pub struct BoatItem;

impl BoatItem {
    /// Maps boat item to corresponding entity type
    fn item_to_entity(item: &Item) -> &'static EntityType {
        match item.id {
            val if val == Item::OAK_BOAT.id => &EntityType::OAK_BOAT,
            val if val == Item::OAK_CHEST_BOAT.id => &EntityType::OAK_CHEST_BOAT,
            val if val == Item::SPRUCE_BOAT.id => &EntityType::SPRUCE_BOAT,
            val if val == Item::SPRUCE_CHEST_BOAT.id => &EntityType::SPRUCE_CHEST_BOAT,
            val if val == Item::BIRCH_BOAT.id => &EntityType::BIRCH_BOAT,
            val if val == Item::BIRCH_CHEST_BOAT.id => &EntityType::BIRCH_CHEST_BOAT,
            val if val == Item::JUNGLE_BOAT.id => &EntityType::JUNGLE_BOAT,
            val if val == Item::JUNGLE_CHEST_BOAT.id => &EntityType::JUNGLE_CHEST_BOAT,
            val if val == Item::ACACIA_BOAT.id => &EntityType::ACACIA_BOAT,
            val if val == Item::ACACIA_CHEST_BOAT.id => &EntityType::ACACIA_CHEST_BOAT,
            val if val == Item::DARK_OAK_BOAT.id => &EntityType::DARK_OAK_BOAT,
            val if val == Item::DARK_OAK_CHEST_BOAT.id => &EntityType::DARK_OAK_CHEST_BOAT,
            val if val == Item::MANGROVE_BOAT.id => &EntityType::MANGROVE_BOAT,
            val if val == Item::MANGROVE_CHEST_BOAT.id => &EntityType::MANGROVE_CHEST_BOAT,
            val if val == Item::CHERRY_BOAT.id => &EntityType::CHERRY_BOAT,
            val if val == Item::CHERRY_CHEST_BOAT.id => &EntityType::CHERRY_CHEST_BOAT,
            val if val == Item::PALE_OAK_BOAT.id => &EntityType::PALE_OAK_BOAT,
            val if val == Item::PALE_OAK_CHEST_BOAT.id => &EntityType::PALE_OAK_CHEST_BOAT,
            val if val == Item::BAMBOO_RAFT.id => &EntityType::BAMBOO_RAFT,
            val if val == Item::BAMBOO_CHEST_RAFT.id => &EntityType::BAMBOO_CHEST_RAFT,
            _ => unreachable!(),
        }
    }

    /// Gets entity dimensions for the boat type
    fn get_entity_dimensions(entity_type: &EntityType) -> EntityDimensions {
        EntityDimensions::new(
            entity_type.dimension[0],
            entity_type.dimension[1],
            entity_type.eye_height,
        )
    }
}

impl ItemMetadata for BoatItem {
    fn ids() -> Box<[u16]> {
        [
            Item::OAK_BOAT.id,
            Item::OAK_CHEST_BOAT.id,
            Item::SPRUCE_BOAT.id,
            Item::SPRUCE_CHEST_BOAT.id,
            Item::BIRCH_BOAT.id,
            Item::BIRCH_CHEST_BOAT.id,
            Item::JUNGLE_BOAT.id,
            Item::JUNGLE_CHEST_BOAT.id,
            Item::ACACIA_BOAT.id,
            Item::ACACIA_CHEST_BOAT.id,
            Item::DARK_OAK_BOAT.id,
            Item::DARK_OAK_CHEST_BOAT.id,
            Item::MANGROVE_BOAT.id,
            Item::MANGROVE_CHEST_BOAT.id,
            Item::CHERRY_BOAT.id,
            Item::CHERRY_CHEST_BOAT.id,
            Item::PALE_OAK_BOAT.id,
            Item::PALE_OAK_CHEST_BOAT.id,
            Item::BAMBOO_RAFT.id,
            Item::BAMBOO_CHEST_RAFT.id,
        ]
        .into()
    }
}

impl ItemBehaviour for BoatItem {
    /// Vanilla: `BoatItem.use()` - raycasts to find placement position
    fn normal_use<'a>(
        &'a self,
        item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let (start_pos, end_pos) = self.get_start_and_end_pos(player);

            // Vanilla: raycast with FluidHandling.ANY - stops on water/lava surface or solid blocks
            let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
                let state_id = world_inner.get_block_state_id(pos).await;

                // Air doesn't stop the raycast
                if state_id == Block::AIR.id {
                    return false;
                }

                // Check if it's a fluid - stop on any fluid (FluidHandling.ANY in vanilla)
                if Fluid::from_state_id(state_id).is_some() {
                    return true;
                }

                // Stop on solid blocks
                true
            };

            let Some((hit_pos, _direction)) = world.raycast(start_pos, end_pos, checker).await
            else {
                return;
            };

            // Calculate hit position - center of the block top or water surface
            // TODO: Vanilla uses exact raycast intersection point (hitResult.getPos()),
            // Pumpkin's raycast only returns block positions.
            let hit_state_id = world.get_block_state_id(&hit_pos).await;
            let hit_vec = if Fluid::from_state_id(hit_state_id).is_some() {
                // Place on water surface (top of the fluid block)
                Vector3::new(
                    f64::from(hit_pos.0.x) + 0.5,
                    f64::from(hit_pos.0.y) + 1.0,
                    f64::from(hit_pos.0.z) + 0.5,
                )
            } else {
                // Place on top of solid block
                Vector3::new(
                    f64::from(hit_pos.0.x) + 0.5,
                    f64::from(hit_pos.0.y) + 1.0,
                    f64::from(hit_pos.0.z) + 0.5,
                )
            };

            // Vanilla: Check for entities in the path that would block placement
            // Get player's rotation vector stretched by 5.0 and expanded by 1.0
            let (yaw, pitch) = player.rotation();
            let rotation_vec =
                Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)).multiply(5.0, 5.0, 5.0);
            let player_entity = player.living_entity.entity.bounding_box.load();
            let search_box = player_entity.stretch(rotation_vec).expand_all(1.0);

            let entities = world.get_entities_at_box(&search_box);
            let player_eye_pos = player.eye_position();

            for entity in &entities {
                // Vanilla: EntityPredicates.CAN_HIT = !isSpectator() && canHit()
                if entity.is_spectator() || !entity.can_hit() {
                    continue;
                }

                // Vanilla: expand by getTargetingMargin() (0.0 for most entities)
                let entity_box = entity.get_entity().bounding_box.load();
                // Check if entity's bounding box contains the player's eye position
                if entity_box.intersects(&BoundingBox::new(player_eye_pos, player_eye_pos)) {
                    // Entity is blocking the line of sight at eye position
                    return;
                }
            }

            // Create the boat entity
            let entity_type = Self::item_to_entity(item);
            let dimensions = Self::get_entity_dimensions(entity_type);
            let boat_box = BoundingBox::new_from_pos(hit_vec.x, hit_vec.y, hit_vec.z, &dimensions);

            // Vanilla: if (!world.isSpaceEmpty(lv7, lv7.getBoundingBox())) return FAIL
            if !world.is_space_empty(boat_box).await {
                return;
            }

            // Check no entities occupy the space
            if !world.get_entities_at_box(&boat_box).is_empty() {
                return;
            }

            // Create and spawn the boat
            let entity = Entity::new(world.clone(), hit_vec, entity_type);

            // Set yaw to player's yaw
            let (player_yaw, _) = player.rotation();
            entity.set_rotation(player_yaw, 0.0);

            let entity = Arc::new(entity);
            world.spawn_entity(entity).await;

            // Decrement item unless in creative mode
            let held_item = player.inventory.held_item();
            let mut stack = held_item.lock().await;
            stack.decrement_unless_creative(player.gamemode.load(), 1);

            // TODO: world.emitGameEvent(user, GameEvent.ENTITY_PLACE, hitResult.getPos())
            // TODO: user.incrementStat(Stats.USED.getOrCreateStat(this))
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockId};
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{position::BlockPos, vector2::to_chunk_pos};
use pumpkin_world::world::BlockFlags;
use tokio::sync::RwLock;

const GRAVITY: f64 = 0.05;

pub struct SplashPotionEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
}

impl SplashPotionEntity {
    pub fn new(entity: Entity) -> Self {
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &pumpkin_data::item::Item::SPLASH_POTION)),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &pumpkin_data::item::Item::SPLASH_POTION)),
        }
    }

    pub async fn set_item_stack(&self, item_stack: ItemStack) {
        let mut write = self.item_stack.write().await;
        *write = item_stack;
    }
}

impl NBTStorage for SplashPotionEntity {}

fn is_water_potion(stack: &ItemStack) -> bool {
    stack
        .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
        .and_then(|pc| pc.potion_id)
        == Some(pumpkin_data::potion::Potion::WATER.id as i32)
}

/// Extinguishes fire (including soul fire) at the hit position and its four horizontal neighbors.
async fn extinguish_fire(world: &Arc<crate::world::World>, hit_pos: Vector3<f64>) {
    let air_state_id = Block::AIR.default_state.id;

    let neighbors = [
        hit_pos,
        Vector3::new(hit_pos.x + 1.0, hit_pos.y, hit_pos.z),
        Vector3::new(hit_pos.x - 1.0, hit_pos.y, hit_pos.z),
        Vector3::new(hit_pos.x, hit_pos.y, hit_pos.z + 1.0),
        Vector3::new(hit_pos.x, hit_pos.y, hit_pos.z - 1.0),
    ];

    for p in neighbors {
        let pos = BlockPos(Vector3::new(
            p.x.floor() as i32,
            p.y.floor() as i32,
            p.z.floor() as i32,
        ));
        let state_id = world.get_block_state_id(&pos);
        let raw_block_id = state_id.to_block_id();
        if raw_block_id == BlockId::FIRE || raw_block_id == BlockId::SOUL_FIRE {
            world
                .set_block_state(&pos, air_state_id, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }
}

pub(crate) async fn extinguish_fire_if_water_potion(
    world: &Arc<crate::world::World>,
    hit_pos: Vector3<f64>,
    stack: &ItemStack,
) {
    if is_water_potion(stack) {
        extinguish_fire(world, hit_pos).await;
    }
}

fn average_effect_color(
    effects: &[(
        &'static pumpkin_data::effect::StatusEffect,
        i32,
        u8,
        bool,
        bool,
        bool,
    )],
) -> i32 {
    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;
    let count = effects.len() as f32;
    for (eff, _, _, _, _, _) in effects {
        let c = eff.color;
        r_sum += ((c >> 16) & 0xFF) as f32;
        g_sum += ((c >> 8) & 0xFF) as f32;
        b_sum += (c & 0xFF) as f32;
    }
    let r = (r_sum / count) as i32;
    let g = (g_sum / count) as i32;
    let b = (b_sum / count) as i32;
    (r << 16) | (g << 8) | b
}

impl EntityBase for SplashPotionEntity {
    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let stack = self.item_stack.read().await;

            // Sync the item stack
            entity.send_meta_data(
                &[pumpkin_protocol::java::client::play::Metadata::new(
                    pumpkin_data::tracked_data::TrackedData::ITEM_STACK,
                    pumpkin_data::meta_data_type::MetaDataType::ITEM_STACK,
                    &pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer::from(
                        stack.clone(),
                    ),
                )],
                None,
            );
        })
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.thrown.process_tick(caller, server).await })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: crate::entity::projectile::ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();
            let hit_pos = hit.hit_pos();

            // Only extinguish fire for plain water potions
            let stack = self.item_stack.read().await.clone();
            extinguish_fire_if_water_potion(&world, hit_pos, &stack).await;

            let effects = crate::item::potion::PotionContents::read_potion_effects(&stack);

            let mut color = 0x385dc6; // Default to water color if no effects/color found
            if let Some(pc) =
                stack.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
            {
                if let Some(c) = pc.custom_color {
                    color = c;
                } else if !effects.is_empty() {
                    color = average_effect_color(&effects);
                }
            } else if !effects.is_empty() {
                // Try to guess from effects directly if potion contents missing but effects present
                color = average_effect_color(&effects);
            }

            // Play splash particles
            let has_instant = effects.iter().any(|(e, _, _, _, _, _)| {
                e.id == pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE.id
                    || e.id == pumpkin_data::effect::StatusEffect::INSTANT_HEALTH.id
            });
            let event_id = if has_instant { 2007 } else { 2002 };

            // Convert hit_pos to BlockPos
            let block_pos = BlockPos(Vector3::new(
                hit_pos.x.floor() as i32,
                hit_pos.y.floor() as i32,
                hit_pos.z.floor() as i32,
            ));
            world.broadcast_to_chunk(
                to_chunk_pos(&Vector2::new(block_pos.0.x, block_pos.0.z)),
                &CWorldEvent::new(event_id, block_pos, color, false),
            );

            // If no effects, just splash (like water bottles)
            if effects.is_empty() {
                return;
            }

            let this_entity = self.get_entity();
            let potion_aabb = this_entity
                .bounding_box
                .load()
                .shift(hit_pos - this_entity.pos.load());
            let effect_aabb = potion_aabb.expand(4.0, 2.0, 4.0);

            // Gather entity and player candidates
            let mut candidates = world.get_entities_at_box(&effect_aabb);
            let players = world.get_players_at_box(&effect_aabb);
            for p in players {
                candidates.push(p.clone() as Arc<dyn EntityBase>);
            }

            let margin = ((this_entity.age.load(std::sync::atomic::Ordering::Relaxed) as f32 - 2.0)
                / 20.0)
                .clamp(0.0, 0.3) as f64;

            for cand in candidates {
                let cand_clone = cand.clone();
                let effs_clone: Vec<_> = effects.clone();
                tokio::spawn(async move {
                    if let Some(living) = cand_clone.get_living_entity() {
                        let target_aabb = cand_clone
                            .get_entity()
                            .bounding_box
                            .load()
                            .expand_all(margin);
                        let dist_sq = potion_aabb.squared_distance_to_box(&target_aabb);
                        if dist_sq >= 16.0 {
                            return;
                        }

                        // Distance scaling
                        let scale = (1.0f32 - (dist_sq.sqrt() as f32 / 4.0)).max(0.0);

                        crate::item::potion::PotionContents::apply_effects_to(
                            living,
                            effs_clone,
                            scale,
                            crate::item::potion::PotionApplicationSource::Normal,
                        )
                        .await;
                    }
                });
            }
        })
    }
}

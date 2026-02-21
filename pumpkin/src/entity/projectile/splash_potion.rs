use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
};
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use tokio::sync::RwLock;

pub struct SplashPotionEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
}

impl SplashPotionEntity {
    pub async fn new(entity: Entity) -> Self {
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0)).await;
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
        };

        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &pumpkin_data::item::Item::SPLASH_POTION)),
        }
    }

    pub async fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter);
        thrown
            .entity
            .set_velocity(Vector3::new(0.0, 0.1, 0.0))
            .await;
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

impl EntityBase for SplashPotionEntity {
    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let stack = self.item_stack.read().await;

            // Sync the item stack
            entity
                .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                    pumpkin_data::tracked_data::TrackedData::DATA_ITEM,
                    pumpkin_data::meta_data_type::MetaDataType::ItemStack,
                    &pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer::from(
                        stack.clone(),
                    ),
                )])
                .await;
        })
    }

    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
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

    fn on_hit(&self, hit: crate::entity::projectile::ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();
            let hit_pos = hit.hit_pos();

            // Read stored item stack and compute potion effects
            let stack = self.item_stack.read().await.clone();
            let effects = crate::item::potion::PotionContents::read_potion_effects(&stack);

            let mut color = 0x385dc6; // Default to water color if no effects/color found
            if let Some(pc) =
                stack.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
            {
                if let Some(c) = pc.custom_color {
                    color = c;
                } else if !effects.is_empty() {
                    let mut r_sum = 0.0;
                    let mut g_sum = 0.0;
                    let mut b_sum = 0.0;
                    let count = effects.len() as f32;
                    for (eff, _, _, _, _, _) in &effects {
                        let c = eff.color;
                        r_sum += ((c >> 16) & 0xFF) as f32;
                        g_sum += ((c >> 8) & 0xFF) as f32;
                        b_sum += (c & 0xFF) as f32;
                    }
                    let r = (r_sum / count) as i32;
                    let g = (g_sum / count) as i32;
                    let b = (b_sum / count) as i32;
                    color = (r << 16) | (g << 8) | b;
                }
            } else {
                // Try to guess from effects directly if potion contents missing but effects present
                if !effects.is_empty() {
                    let mut r_sum = 0.0;
                    let mut g_sum = 0.0;
                    let mut b_sum = 0.0;
                    let count = effects.len() as f32;
                    for (eff, _, _, _, _, _) in &effects {
                        let c = eff.color;
                        r_sum += ((c >> 16) & 0xFF) as f32;
                        g_sum += ((c >> 8) & 0xFF) as f32;
                        b_sum += (c & 0xFF) as f32;
                    }
                    let r = (r_sum / count) as i32;
                    let g = (g_sum / count) as i32;
                    let b = (b_sum / count) as i32;
                    color = (r << 16) | (g << 8) | b;
                }
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
            world
                .broadcast_packet_all(&CWorldEvent::new(event_id, block_pos, color, false))
                .await;

            // If no effects, just splash (like water bottles)
            if effects.is_empty() {
                return;
            }

            let radius = 4.0f64;
            let min = Vector3::new(hit_pos.x - radius, hit_pos.y - radius, hit_pos.z - radius);
            let max = Vector3::new(hit_pos.x + radius, hit_pos.y + radius, hit_pos.z + radius);
            let aabb = BoundingBox::new(min, max);

            // Gather entity and player candidates
            let mut candidates = world.get_entities_at_box(&aabb);
            let players = world.get_players_at_box(&aabb);
            for p in players {
                candidates.push(p.clone() as Arc<dyn EntityBase>);
            }

            for cand in candidates {
                let cand_clone = cand.clone();
                let effs_clone: Vec<_> = effects.clone();
                let hit_pos_clone = hit_pos;
                tokio::spawn(async move {
                    if let Some(living) = cand_clone.get_living_entity() {
                        let pos = cand_clone.get_entity().pos.load();
                        let dx = pos.x - hit_pos_clone.x;
                        let dy = pos.y - hit_pos_clone.y;
                        let dz = pos.z - hit_pos_clone.z;
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                        if dist > radius {
                            return;
                        }

                        // Distance scaling
                        let mut scale = 1.0f32 - (dist as f32 / radius as f32);
                        if scale < 0.0 {
                            scale = 0.0;
                        }

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

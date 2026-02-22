use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage},
    server::Server,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

type EffectEntry = (&'static StatusEffect, i32, u8, bool, bool, bool);
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

/// The effect cloud entity that is spawned where a lingering potion lands.
pub struct AreaEffectCloudEntity {
    pub entity: Entity,
    /// Stored potion item stack (may be empty) to read effects from.
    pub item_stack: Mutex<ItemStack>,
    /// Active potion effects as tuples: (`StatusEffect`, `duration_ticks`, `amplifier`, `ambient`, `show_particles`, `show_icon`)
    pub effects: Mutex<Vec<EffectEntry>>,
    pub radius: Mutex<f32>,
    pub duration: Mutex<i32>,
    pub age: Mutex<i32>,
    /// ticks between reapplications to the same entity
    pub reapplication_delay: Mutex<i32>,
    /// map of `entity_id` -> ticks remaining until that entity can be affected again
    pub reapplication_map: Mutex<HashMap<i32, i32>>,
    /// linear radius change per tick
    pub radius_on_tick: Mutex<f32>,
    /// radius change when an entity is affected
    pub radius_on_use: Mutex<f32>,
    /// duration change (ticks) when an entity is affected
    pub duration_on_use: Mutex<i32>,
    /// ticks to wait before the cloud becomes active and applies effects (grace period)
    pub wait_time: Mutex<i32>,
}

impl AreaEffectCloudEntity {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(entity: Entity) -> Arc<dyn EntityBase> {
        let cloud = Self {
            entity,
            item_stack: Mutex::new(ItemStack::new(0, &pumpkin_data::item::Item::GLASS_BOTTLE)),
            effects: Mutex::new(Vec::new()),
            radius: Mutex::new(3.0),
            duration: Mutex::new(600), // default for lingering potions
            age: Mutex::new(0),
            reapplication_delay: Mutex::new(20),
            reapplication_map: Mutex::new(HashMap::new()),
            radius_on_tick: Mutex::new(-3.0 / 600.0), // default for lingering potions
            radius_on_use: Mutex::new(0.0),
            duration_on_use: Mutex::new(0),
            wait_time: Mutex::new(20),
        };

        Arc::new(cloud)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        entity: Entity,
        item_stack: ItemStack,
        effects_in: Vec<EffectEntry>,
        duration_in: i32,
        radius_in: f32,
        reapplication_delay_in: i32,
        wait_time_in: i32,
        radius_on_use_in: f32,
        duration_on_use_in: i32,
    ) -> Arc<dyn EntityBase> {
        let cloud = Self {
            entity,
            item_stack: Mutex::new(item_stack),
            effects: Mutex::new(effects_in),
            radius: Mutex::new(radius_in),
            duration: Mutex::new(duration_in),
            age: Mutex::new(0),
            reapplication_delay: Mutex::new(reapplication_delay_in),
            reapplication_map: Mutex::new(HashMap::new()),
            radius_on_tick: Mutex::new(-radius_in / (duration_in as f32).max(1.0)),
            radius_on_use: Mutex::new(radius_on_use_in),
            duration_on_use: Mutex::new(duration_on_use_in),
            wait_time: Mutex::new(wait_time_in),
        };

        Arc::new(cloud)
    }
}

impl NBTStorage for AreaEffectCloudEntity {}

impl EntityBase for AreaEffectCloudEntity {
    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            #[derive(serde::Serialize)]
            struct ParticleMeta<'a> {
                particle_id: pumpkin_protocol::codec::var_int::VarInt,
                #[serde(serialize_with = "pumpkin_protocol::ser::network_serialize_no_prefix")]
                data: &'a [u8],
            }

            // Send initial radius and particle (color) so clients render correctly
            let radius = *self.radius.lock().await;

            // Compute particle color
            let stack = self.item_stack.lock().await.clone();
            let effects = self.effects.lock().await.clone();

            // Use ARGB format
            let mut color: i32 = (0xFFi32 << 24) | 0x385dc6; // default water-like color

            if let Some(pc) =
                stack.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
            {
                if let Some(c) = pc.custom_color {
                    color = c | (0xFFi32 << 24);
                } else if !effects.is_empty() {
                    let mut r_sum = 0.0f32;
                    let mut g_sum = 0.0f32;
                    let mut b_sum = 0.0f32;
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
                    color = (0xFFi32 << 24) | (r << 16) | (g << 8) | b;
                }
            } else if !effects.is_empty() {
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;
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
                color = (0xFFi32 << 24) | (r << 16) | (g << 8) | b;
            }

            // Build raw particle option bytes for ENTITY_EFFECT
            let data_bytes = color.to_be_bytes();

            let meta = ParticleMeta {
                particle_id: pumpkin_protocol::codec::var_int::VarInt(
                    pumpkin_data::particle::Particle::EntityEffect as i32,
                ),
                data: &data_bytes,
            };

            // Send initial particle and radius
            self.entity
                .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                    pumpkin_data::tracked_data::TrackedData::DATA_PARTICLE,
                    pumpkin_data::meta_data_type::MetaDataType::Particle,
                    &meta,
                )])
                .await;

            self.entity
                .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                    pumpkin_data::tracked_data::TrackedData::DATA_RADIUS,
                    pumpkin_data::meta_data_type::MetaDataType::Float,
                    radius,
                )])
                .await;

            // Initial waiting flag
            let wait_time = *self.wait_time.lock().await;
            let is_waiting = 0 < wait_time;
            self.entity
                .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                    pumpkin_data::tracked_data::TrackedData::DATA_WAITING,
                    pumpkin_data::meta_data_type::MetaDataType::Boolean,
                    is_waiting,
                )])
                .await;
        })
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::semicolon_outside_block)]
    fn tick<'a>(
        &'a self,
        _caller: Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Age & duration handling
            {
                let mut age = self.age.lock().await;
                *age += 1;
                let duration = *self.duration.lock().await;
                if *age > duration {
                    // Remove old entities
                    self.entity.remove().await;
                    return;
                }
            }

            // Get current age and waiting period
            let age = *self.age.lock().await;
            let wait_time = *self.wait_time.lock().await;

            // When the waiting period ends, notify clients so they render full particles
            if age == wait_time && wait_time > 0 {
                self.entity
                    .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                        pumpkin_data::tracked_data::TrackedData::DATA_WAITING,
                        pumpkin_data::meta_data_type::MetaDataType::Boolean,
                        false,
                    )])
                    .await;
            }

            if age < wait_time {
                // Respect waiting/grace period
                return;
            }

            // Update radius
            {
                let mut radius = self.radius.lock().await;
                let delta = *self.radius_on_tick.lock().await;
                *radius += delta;
                let current_radius = *radius;
                if current_radius <= 0.0 {
                    self.entity.remove().await;
                    return;
                }

                // Send new radius
                drop(radius);
                self.entity
                    .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                        pumpkin_data::tracked_data::TrackedData::DATA_RADIUS,
                        pumpkin_data::meta_data_type::MetaDataType::Float,
                        current_radius,
                    )])
                    .await;
            }

            // Tick down reapplication map
            {
                let map = self.reapplication_map.lock().await;
                let keys: Vec<i32> = map.keys().copied().collect();
                drop(map);
                for k in keys {
                    let mut map = self.reapplication_map.lock().await;
                    if let Some(v) = map.get_mut(&k) {
                        *v -= 1;
                        if *v <= 0 {
                            map.remove(&k);
                        }
                    }
                }
            }

            // Apply effects to nearby entities if eligible
            let pos = self.entity.pos.load();
            let r = *self.radius.lock().await as f64;
            let min = Vector3::new(pos.x - r, pos.y - r, pos.z - r);
            let max = Vector3::new(pos.x + r, pos.y + r, pos.z + r);
            let aabb = BoundingBox::new(min, max);
            let world = self.entity.world.load();

            let mut candidates = world.get_entities_at_box(&aabb);
            let players = world.get_players_at_box(&aabb);
            for p in players {
                candidates.push(p.clone() as Arc<dyn EntityBase>);
            }

            for cand in candidates {
                let cand_clone = cand.clone();

                // Skip self and other `AreaEffectCloud` entities
                if cand_clone.get_entity().entity_id == self.get_entity().entity_id {
                    continue;
                }
                if *cand_clone.get_entity().entity_type
                    == pumpkin_data::entity::EntityType::AREA_EFFECT_CLOUD
                {
                    continue;
                }

                // Determine candidate id early
                let ent_id = cand_clone.get_entity().entity_id;

                let radius_f = *self.radius.lock().await as f64;
                let pos_e = cand_clone.get_entity().pos.load();
                let dx = pos_e.x - pos.x;
                let dy = pos_e.y - pos.y;
                let dz = pos_e.z - pos.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist > radius_f {
                    continue;
                }

                let scale = 1.0f32 - (dist as f32 / radius_f as f32);

                // Decide whether this contact will actually apply an effect
                let effs_clone = self.effects.lock().await.clone();
                let mut will_apply = false;

                // Only living entities can receive effects
                if let Some(living_ref) = cand_clone.get_living_entity() {
                    for (eff, _, _, _, _, _) in &effs_clone {
                        // Instant effects always apply
                        let is_instant = eff.id
                            == pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE.id
                            || eff.id == pumpkin_data::effect::StatusEffect::INSTANT_HEALTH.id;
                        if is_instant {
                            will_apply = true;
                            break;
                        }

                        // Only apply if entity does not already have that effect
                        if !living_ref.has_effect(eff).await {
                            will_apply = true;
                            break;
                        }
                    }
                }

                // If nothing would be applied, skip
                if !will_apply {
                    continue;
                }

                // Apply scaled effects inside a spawned task
                let cand_for_spawn = cand_clone.clone();
                let effs_for_spawn = effs_clone.clone();
                tokio::spawn(async move {
                    if let Some(living) = cand_for_spawn.get_living_entity() {
                        crate::item::potion::PotionContents::apply_effects_to(
                            living,
                            effs_for_spawn,
                            scale,
                            crate::item::potion::PotionApplicationSource::AreaEffectCloud,
                        )
                        .await;
                    }
                });

                // Set reapplication delay for this entity
                let mut map = self.reapplication_map.lock().await;
                let delay = *self.reapplication_delay.lock().await;
                map.insert(ent_id, delay);

                // Apply radius-on-use (shrink)
                let radius_on_use = *self.radius_on_use.lock().await;
                if radius_on_use != 0.0 {
                    let mut radius_lock = self.radius.lock().await;
                    *radius_lock += radius_on_use;
                    let current_radius = *radius_lock;
                    if current_radius < 0.5 {
                        drop(radius_lock);
                        self.entity.remove().await;
                        return;
                    }
                    drop(radius_lock);

                    // Send updated radius to clients
                    self.entity
                        .send_meta_data(&[pumpkin_protocol::java::client::play::Metadata::new(
                            pumpkin_data::tracked_data::TrackedData::DATA_RADIUS,
                            pumpkin_data::meta_data_type::MetaDataType::Float,
                            current_radius,
                        )])
                        .await;
                }

                // Apply duration-on-use (shorten lifespan)
                let duration_on_use = *self.duration_on_use.lock().await;
                if duration_on_use != 0 {
                    let mut duration_lock = self.duration.lock().await;
                    if *duration_lock != -1 {
                        *duration_lock += duration_on_use;
                        if *duration_lock <= 0 {
                            drop(duration_lock);
                            self.entity.remove().await;
                            return;
                        }
                    }
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
}

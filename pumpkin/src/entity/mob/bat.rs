use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use crate::entity::attributes::AttributeBuilder;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::Sound;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use tokio::sync::Mutex;

use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture};

const ROOSTING_FLAG: u8 = 1;
const CLOSE_PLAYER_DISTANCE: f64 = 4.0;
/// Vanilla: `getMinAmbientSoundDelay()` returns 80 for most mobs
const MIN_AMBIENT_SOUND_DELAY: i32 = 80;

pub struct BatEntity {
    pub mob_entity: MobEntity,
    hanging_position: Mutex<Option<BlockPos>>,
    roosting: AtomicBool,
    ambient_sound_chance: AtomicI32,
}

impl BatEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bat = Self {
            mob_entity,
            hanging_position: Mutex::new(None),
            roosting: AtomicBool::new(true),
            ambient_sound_chance: AtomicI32::new(MIN_AMBIENT_SOUND_DELAY),
        };
        let mob_arc = Arc::new(bat);

        mob_arc.set_roosting_metadata(true).await;

        mob_arc
    }

    fn is_roosting(&self) -> bool {
        self.roosting.load(Relaxed)
    }

    async fn set_roosting(&self, roosting: bool) {
        self.roosting.store(roosting, Relaxed);
        self.set_roosting_metadata(roosting).await;
    }

    async fn set_roosting_metadata(&self, roosting: bool) {
        let flags: u8 = if roosting { ROOSTING_FLAG } else { 0 };
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_BAT_FLAGS,
                MetaDataType::BYTE,
                flags,
            )])
            .await;
    }

    #[must_use]
    pub fn create_attributes() -> AttributeBuilder {
        AttributeBuilder::new().add(Attributes::MAX_HEALTH, 6.0)
    }
}

impl NBTStorage for BatEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            let flags: u8 = if self.is_roosting() { ROOSTING_FLAG } else { 0 };
            nbt.put_byte("BatFlags", flags as i8);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;
            let flags = nbt.get_byte("BatFlags").unwrap_or(0) as u8;
            let roosting = (flags & ROOSTING_FLAG) != 0;
            self.set_roosting(roosting).await;
        })
    }
}

impl Mob for BatEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let block_pos = entity.block_pos.load();
            let above_pos = BlockPos::new(block_pos.0.x, block_pos.0.y + 1, block_pos.0.z);
            let world = entity.world.load();

            // Ambient idle sound (vanilla: MobEntity.mobTick → playAmbientSound)
            let chance = self.ambient_sound_chance.fetch_sub(1, Relaxed);
            if chance <= 0 {
                self.ambient_sound_chance
                    .store(MIN_AMBIENT_SOUND_DELAY, Relaxed);
                entity.play_sound(Sound::EntityBatAmbient).await;
            }

            if self.is_roosting() {
                let above_state = world.get_block_state(&above_pos).await;
                if above_state.is_solid_block() {
                    let rotate_head = {
                        let mut rng = rand::rng();
                        (rng.random_range(0u32..200) == 0)
                            .then(|| rng.random_range(0i32..360) as f32)
                    };
                    if let Some(head_yaw) = rotate_head {
                        entity.head_yaw.store(head_yaw);
                    }

                    let pos = entity.pos.load();
                    if world
                        .get_closest_player(pos, CLOSE_PLAYER_DISTANCE)
                        .is_some()
                    {
                        self.set_roosting(false).await;
                    }
                } else {
                    self.set_roosting(false).await;
                }
            } else {
                let mut hanging_pos = self.hanging_position.lock().await;

                if let Some(hp) = *hanging_pos {
                    let hp_state = world.get_block_state(&hp).await;
                    if !hp_state.is_air() || hp.0.y <= world.dimension.min_y {
                        *hanging_pos = None;
                    }
                }

                let (should_pick_new, new_target, try_roost) = {
                    let mut rng = rand::rng();
                    let should_pick = hanging_pos.is_none()
                        || rng.random_range(0u32..30) == 0
                        || hanging_pos.is_some_and(|hp| {
                            let pos = entity.pos.load();
                            let dx = f64::from(hp.0.x) + 0.5 - pos.x;
                            let dy = f64::from(hp.0.y) + 0.1 - pos.y;
                            let dz = f64::from(hp.0.z) + 0.5 - pos.z;
                            dx * dx + dy * dy + dz * dz < 4.0
                        });
                    let new_target = should_pick.then(|| {
                        let pos = entity.pos.load();
                        BlockPos::new(
                            pos.x as i32 + rng.random_range(0i32..7) - rng.random_range(0i32..7),
                            (pos.y + f64::from(rng.random_range(0i32..6)) - 2.0) as i32,
                            pos.z as i32 + rng.random_range(0i32..7) - rng.random_range(0i32..7),
                        )
                    });
                    let try_roost = rng.random_range(0u32..100) == 0;
                    (should_pick, new_target, try_roost)
                };

                if should_pick_new {
                    // Pre-validate: only accept targets in air (avoids water, lava, hazards)
                    if let Some(target) = new_target {
                        let target_state = world.get_block_state(&target).await;
                        if target_state.is_air() && target.0.y > world.dimension.min_y {
                            *hanging_pos = Some(target);
                        } else {
                            *hanging_pos = None;
                        }
                    } else {
                        *hanging_pos = None;
                    }
                }

                if let Some(target) = *hanging_pos {
                    let pos = entity.pos.load();
                    let d = f64::from(target.0.x) + 0.5 - pos.x;
                    let e = f64::from(target.0.y) + 0.1 - pos.y;
                    let f = f64::from(target.0.z) + 0.5 - pos.z;

                    let velo = entity.velocity.load();
                    let new_velo = Vector3::new(
                        velo.x + (d.signum() * 0.5 - velo.x) * 0.1,
                        velo.y + (e.signum() * 0.7 - velo.y) * 0.1,
                        velo.z + (f.signum() * 0.5 - velo.z) * 0.1,
                    );
                    entity.velocity.store(new_velo);

                    let yaw = (new_velo.z.atan2(new_velo.x) as f32).to_degrees() - 90.0;
                    let yaw_diff = pumpkin_util::math::wrap_degrees(yaw - entity.yaw.load());
                    entity.yaw.store(entity.yaw.load() + yaw_diff);
                }
                drop(hanging_pos);

                if try_roost {
                    let above_state = world.get_block_state(&above_pos).await;
                    if above_state.is_solid_block() {
                        self.set_roosting(true).await;
                    }
                }
            }
        })
    }

    fn post_tick(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if self.is_roosting() {
                let entity = &self.mob_entity.living_entity.entity;
                entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                let pos = entity.pos.load();
                let snapped_y = (pos.y.floor()) + 1.0 - f64::from(entity.height());
                entity.set_pos(Vector3::new(pos.x, snapped_y, pos.z));
            }
        })
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.6)
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.is_roosting() {
                self.set_roosting(false).await;
            }
        })
    }
}

use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    entity::EntityType,
    item::Item,
    meta_data_type::MetaDataType,
    particle::Particle,
    sound::{Sound, SoundCategory},
    tag,
    tag::Taggable,
    tracked_data::TrackedData,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::client::play::{CEntityPositionSync, Metadata},
};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, NBTStorage, NbtFuture,
    ai::{
        goal::{
            GoalFuture, active_target::ActiveTargetGoal, chase_player::ChasePlayerGoal,
            look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal,
            melee_attack::MeleeAttackGoal, pick_up_block::PickUpBlockGoal,
            place_block::PlaceBlockGoal, revenge::RevengeGoal, swim::SwimGoal,
            teleport_towards_player::TeleportTowardsPlayerGoal, wander_around::WanderAroundGoal,
        },
        pathfinder::node::PathType,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

const SPEED_BOOST: f64 = 0.15;

pub const ENDERMAN_EYE_HEIGHT: f64 = 2.55;
pub const ENDERMAN_BODY_Y_OFFSET: f64 = 1.45;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;

fn is_projectile_damage(dt: DamageType) -> bool {
    let (names, _) = pumpkin_data::tag::DamageType::MINECRAFT_IS_PROJECTILE;
    names.contains(&dt.message_id)
}

pub struct EndermanEntity {
    pub mob_entity: MobEntity,
    carried_block: AtomicCell<Option<u16>>,
    angry: AtomicBool,
    provoked: AtomicBool,
    speed_boosted: AtomicBool,
}

impl EndermanEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        mob_entity.attack_damage = 7.0;
        mob_entity.follow_range = 64.0;
        let entity = Self {
            mob_entity,
            carried_block: AtomicCell::new(None),
            angry: AtomicBool::new(false),
            provoked: AtomicBool::new(false),
            speed_boosted: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(entity);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        mob_arc.mob_entity.living_entity.movement_speed.store(0.3);

        let mut navigator = mob_arc.mob_entity.navigator.lock().await;
        navigator.set_mob_dimensions(0.6, 2.9);
        navigator.set_follow_range(64.0);
        navigator.set_pathfinding_malus(PathType::Water, -1.0);
        drop(navigator);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ChasePlayerGoal::new(mob_arc.clone())));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
            goal_selector.add_goal(10, Box::new(PlaceBlockGoal::new(mob_arc.clone())));
            goal_selector.add_goal(11, Box::new(PickUpBlockGoal::new(mob_arc.clone())));

            target_selector.add_goal(1, Box::new(TeleportTowardsPlayerGoal::new(mob_arc.clone())));
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::ENDERMITE, true),
            );
        };

        mob_arc
    }

    pub async fn teleport_randomly(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let (x, y, z) = {
            let mut rng = self.get_random();
            (
                pos.x + (rng.random_range(0.0..1.0) - 0.5) * 64.0,
                pos.y + (rng.random_range(0i32..64) - 32) as f64,
                pos.z + (rng.random_range(0.0..1.0) - 0.5) * 64.0,
            )
        };

        self.teleport_to(x, y, z).await
    }

    pub async fn teleport_towards(&self, target: &dyn EntityBase) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();

        let dx = pos.x - target_pos.x;
        let dy = (pos.y + ENDERMAN_BODY_Y_OFFSET) - (target_pos.y + PLAYER_EYE_HEIGHT);
        let dz = pos.z - target_pos.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < 1e-6 {
            return false;
        }

        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;
        let (x, y, z) = {
            let mut rng = self.get_random();
            (
                pos.x + (rng.random_range(0.0..1.0) - 0.5) * 8.0 - nx * 16.0,
                pos.y + (rng.random_range(0i32..16) - 8) as f64 - ny * 16.0,
                pos.z + (rng.random_range(0.0..1.0) - 0.5) * 8.0 - nz * 16.0,
            )
        };

        self.teleport_to(x, y, z).await
    }

    pub async fn teleport_to(&self, x: f64, y: f64, z: f64) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let origin = entity.pos.load();
        let world = entity.world.load();

        let min_y = f64::from(world.dimension.min_y);
        let max_y = f64::from(world.dimension.min_y + world.dimension.height - 1);
        let mut target_y = y.clamp(min_y, max_y);

        let block_x = x.floor() as i32;
        let mut block_y = target_y.floor() as i32;
        let block_z = z.floor() as i32;
        let mut found_ground = false;
        loop {
            let below_pos = BlockPos::new(block_x, block_y - 1, block_z);
            let below_state = world.get_block_state(&below_pos).await;
            if below_state.is_solid() {
                found_ground = true;
                break;
            }
            if block_y <= world.dimension.min_y {
                break;
            }
            block_y -= 1;
            target_y = block_y as f64;
        }

        if !found_ground {
            return false;
        }

        let dest_pos = BlockPos::new(block_x, block_y, block_z);
        let dest_fluid = world.get_fluid(&dest_pos).await;
        if dest_fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            return false;
        }

        let half_width = 0.3;
        let height = 2.9;
        let bb = BoundingBox::new(
            Vector3::new(x - half_width, target_y, z - half_width),
            Vector3::new(x + half_width, target_y + height, z + half_width),
        );
        if !world.is_space_empty(bb).await {
            return false;
        }

        let new_pos = Vector3::new(x, target_y, z);

        for pos in &[origin, new_pos] {
            world
                .spawn_particle(
                    *pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    128,
                    Particle::Portal,
                )
                .await;
            world
                .play_sound(Sound::EntityEndermanTeleport, SoundCategory::Hostile, pos)
                .await;
        }

        entity.set_pos(new_pos);

        world
            .broadcast_packet_all(&CEntityPositionSync::new(
                entity.entity_id.into(),
                new_pos,
                Vector3::new(0.0, 0.0, 0.0),
                entity.yaw.load(),
                entity.pitch.load(),
                entity.on_ground.load(Ordering::Relaxed),
            ))
            .await;

        self.mob_entity.navigator.lock().await.stop();

        true
    }

    pub async fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let mut mob_target = self.mob_entity.target.lock().await;
        (*mob_target).clone_from(&target);
        drop(mob_target);

        if target.is_some() {
            self.set_angry(true).await;
            // TODO: use attribute modifiers instead of direct speed arithmetic
            if !self.speed_boosted.swap(true, Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;
                let current = living.movement_speed.load();
                living.movement_speed.store(current + SPEED_BOOST);
            }
        } else {
            self.set_angry(false).await;
            self.set_provoked(false).await;
            if self.speed_boosted.swap(false, Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;
                let current = living.movement_speed.load();
                living.movement_speed.store(current - SPEED_BOOST);
            }
        }
    }

    pub async fn set_angry(&self, angry: bool) {
        self.angry.store(angry, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_ANGRY,
                MetaDataType::Boolean,
                angry,
            )])
            .await;
    }

    pub fn is_angry(&self) -> bool {
        self.angry.load(Ordering::Relaxed)
    }

    pub async fn set_provoked(&self, provoked: bool) {
        self.provoked.store(provoked, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_PROVOKED,
                MetaDataType::Boolean,
                provoked,
            )])
            .await;
    }

    pub async fn set_carried_block(&self, block_state: Option<u16>) {
        self.carried_block.store(block_state);
        let value = block_state.map_or(VarInt(0), |id| VarInt(id as i32));
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_CARRIED_BLOCK,
                MetaDataType::OptionalBlockState,
                value,
            )])
            .await;
    }

    pub fn get_carried_block(&self) -> Option<u16> {
        self.carried_block.load()
    }

    pub async fn is_player_staring(&self, player: &Player) -> bool {
        let equipment = player.living_entity.entity_equipment.lock().await;
        let head_item = equipment.get(&EquipmentSlot::HEAD);
        let head_stack = head_item.lock().await;
        if !head_stack.is_empty() && head_stack.item == &Item::CARVED_PUMPKIN {
            return false;
        }
        drop(head_stack);
        drop(equipment);

        let entity = &self.mob_entity.living_entity.entity;
        let enderman_pos = entity.pos.load();
        let enderman_eye_y = enderman_pos.y + ENDERMAN_EYE_HEIGHT;

        let player_entity = player.get_entity();
        let player_pos = player_entity.pos.load();
        let player_eye_y = player_pos.y + PLAYER_EYE_HEIGHT;

        let pitch = player_entity.pitch.load().to_radians();
        let yaw = -player_entity.yaw.load().to_radians();

        let cos_pitch = pitch.cos();
        let look_dir = Vector3::new(
            (yaw.sin() * cos_pitch) as f64,
            (-pitch.sin()) as f64,
            (yaw.cos() * cos_pitch) as f64,
        );

        let dx = enderman_pos.x - player_pos.x;
        let dy = enderman_eye_y - player_eye_y;
        let dz = enderman_pos.z - player_pos.z;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        if distance < 0.1 {
            return false;
        }

        let dir_x = dx / distance;
        let dir_y = dy / distance;
        let dir_z = dz / distance;

        let dot = look_dir.x * dir_x + look_dir.y * dir_y + look_dir.z * dir_z;

        if dot <= 1.0 - 0.025 / distance {
            return false;
        }

        let enderman_eye_pos = Vector3::new(enderman_pos.x, enderman_eye_y, enderman_pos.z);
        let player_eye_pos = Vector3::new(player_pos.x, player_eye_y, player_pos.z);
        let world = entity.world.load();
        world
            .raycast(enderman_eye_pos, player_eye_pos, async |block_pos, w| {
                let state = w.get_block_state(block_pos).await;
                state.is_solid()
            })
            .await
            .is_none()
    }
}

impl NBTStorage for EndermanEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            if let Some(block_state) = self.carried_block.load() {
                nbt.put_int("carriedBlockState", block_state as i32);
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            if let Some(block_state) = nbt.get_int("carriedBlockState") {
                self.set_carried_block(Some(block_state as u16)).await;
            }
        })
    }
}

impl Mob for EndermanEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn set_mob_target(&self, target: Option<Arc<dyn EntityBase>>) -> GoalFuture<'_, ()> {
        Box::pin(async move {
            self.set_target(target).await;
        })
    }

    // TODO: sunlight avoidance, carried block drop on death, angerable system, ambient sound override
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            if !entity.is_alive() {
                return;
            }

            // TODO: also check rain
            if entity.touching_water.load(Ordering::SeqCst) {
                self.mob_entity
                    .living_entity
                    .damage_with_context(self, 1.0, DamageType::DROWN, None, None, None)
                    .await;
            }

            let pos = entity.pos.load();
            let world = entity.world.load();
            let particles = {
                let mut rng = self.get_random();
                std::array::from_fn::<_, 2, _>(|_| {
                    (
                        Vector3::new(
                            pos.x + rng.random_range(-0.5..0.5),
                            pos.y + rng.random_range(0.0..2.9),
                            pos.z + rng.random_range(-0.5..0.5),
                        ),
                        Vector3::new(
                            (rng.random_range(0.0f32..1.0) - 0.5) * 2.0,
                            -rng.random_range(0.0f32..1.0),
                            (rng.random_range(0.0f32..1.0) - 0.5) * 2.0,
                        ),
                    )
                })
            };
            for (particle_pos, offset) in particles {
                world
                    .spawn_particle(particle_pos, offset, 0.0, 1, Particle::Portal)
                    .await;
            }
        })
    }

    fn pre_damage<'a>(
        &'a self,
        damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> GoalFuture<'a, bool> {
        let is_projectile = is_projectile_damage(damage_type);
        Box::pin(async move {
            if is_projectile {
                for _ in 0..64 {
                    if self.teleport_randomly().await {
                        return false;
                    }
                }
            }
            true
        })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if source.is_some_and(|s| s.get_living_entity().is_some()) {
                return;
            }
            let should_teleport = self.get_random().random_range(0..10) != 0;
            if should_teleport {
                self.teleport_randomly().await;
            }
        })
    }
}

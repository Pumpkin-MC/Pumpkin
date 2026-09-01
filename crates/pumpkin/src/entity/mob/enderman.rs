use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use crate::entity::attributes::Modifier;
use crate::entity::attributes::ModifierOperation;
use pumpkin_data::{BlockStateId, attributes::Attributes};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    entity::EntityType,
    item::Item,
    particle::Particle,
    sound::{Sound, SoundCategory},
    tag,
    tag::Taggable,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::{
        goal::{
            active_target::ActiveTargetGoal, chase_player::ChasePlayerGoal,
            look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
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
const ENDERMAN_SPEED_BOOST_ID: &str = "minecraft:attacking";

pub const ENDERMAN_EYE_HEIGHT: f64 = 2.55;
pub const ENDERMAN_BODY_Y_OFFSET: f64 = 1.45;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;

fn is_projectile_damage(dt: DamageType) -> bool {
    let (names, _) = pumpkin_data::tag::DamageType::MINECRAFT_IS_PROJECTILE;
    names.contains(&dt.message_id)
}

pub struct EndermanEntity {
    pub mob_entity: MobEntity,
    carried_block: AtomicCell<Option<BlockStateId>>,
    angry: AtomicBool,
    provoked: AtomicBool,
    speed_boosted: AtomicBool,
}

impl EndermanEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
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

        let mut navigator = mob_arc
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_mob_dimensions(0.6, 2.9);
        navigator.set_pathfinding_malus(PathType::Water, -1.0);
        drop(navigator);

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ChasePlayerGoal::new(mob_arc.clone())));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
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

    pub fn teleport_randomly(&self) -> bool {
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

        self.teleport_to(x, y, z)
    }

    pub fn teleport_towards(&self, target: &dyn EntityBase) -> bool {
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

        self.teleport_to(x, y, z)
    }

    pub fn teleport_to(&self, x: f64, y: f64, z: f64) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let origin = entity.pos.load();
        let world = entity.world.load();

        let min_y = f64::from(world.dimension.min_y);
        let max_y = f64::from(world.dimension.min_y + world.dimension.height - 1);
        let mut target_y = y.clamp(min_y, max_y);

        let block_x = x.floor() as i32;
        let mut block_y = target_y.floor() as i32;
        let block_z = z.floor() as i32;
        let ground_pos = loop {
            let below_pos = BlockPos::new(block_x, block_y - 1, block_z);
            let below_state = world.get_block_state(&below_pos);
            if below_state.is_solid() {
                break Some(below_pos);
            }
            if block_y <= world.dimension.min_y {
                break None;
            }
            block_y -= 1;
            target_y = block_y as f64;
        };
        let Some(ground_pos) = ground_pos else {
            return false;
        };

        if world
            .get_fluid(&ground_pos)
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
        {
            return false;
        }

        let bb = BoundingBox::new_from_pos(x, target_y, z, &entity.entity_dimension.load());
        if !world.is_space_empty(bb) || world.contains_any_liquid(bb) {
            return false;
        }

        let new_pos = Vector3::new(x, target_y, z);

        for pos in &[origin, new_pos] {
            world.spawn_particle(
                *pos,
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                128,
                Particle::Portal,
            );
            world.play_sound(Sound::EntityEndermanTeleport, SoundCategory::Hostile, pos);
        }

        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_teleport::EntityTeleportEvent::new(
                    entity.entity_id,
                    origin,
                    new_pos,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return false;
            }
        }

        entity.teleport(new_pos, None, None, &world);

        self.mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();

        true
    }

    pub fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let is_some = target.is_some();
        let mut mob_target = self
            .mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *mob_target = target;
        drop(mob_target);

        if is_some {
            self.set_angry(true);
            // Use attribute modifier instead of direct speed arithmetic
            if !self.speed_boosted.swap(true, Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;
                let modifier = Modifier {
                    id: ENDERMAN_SPEED_BOOST_ID.to_string(),
                    amount: SPEED_BOOST,
                    operation: ModifierOperation::Add,
                };

                living.update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                    inst.add_or_replace_modifier(modifier);
                });

                crate::entity::attributes::send_attribute_updates_for_living(
                    living,
                    vec![Attributes::MOVEMENT_SPEED],
                );
            }
        } else {
            self.set_angry(false);
            self.set_provoked(false);
            if self.speed_boosted.swap(false, Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;

                living.update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                    inst.remove_modifier(ENDERMAN_SPEED_BOOST_ID);
                });

                crate::entity::attributes::send_attribute_updates_for_living(
                    living,
                    vec![Attributes::MOVEMENT_SPEED],
                );
            }
        }
    }

    pub fn set_angry(&self, angry: bool) {
        self.angry.store(angry, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::enderman::CREEPY,
                angry,
            )],
            None,
        );
    }

    pub fn is_angry(&self) -> bool {
        self.angry.load(Ordering::Relaxed)
    }

    pub fn set_provoked(&self, provoked: bool) {
        self.provoked.store(provoked, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::enderman::STARED_AT,
                provoked,
            )],
            None,
        );
    }

    pub fn set_carried_block(&self, block_state: Option<BlockStateId>) {
        self.carried_block.store(block_state);
        let value = block_state.map_or(VarInt(0), |id| VarInt(id.as_u16() as i32));
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::enderman::CARRY_STATE,
                value,
            )],
            None,
        );
    }

    pub fn get_carried_block(&self) -> Option<BlockStateId> {
        self.carried_block.load()
    }

    pub fn is_player_staring(&self, player: &Player) -> bool {
        let equipment = player.living_entity.entity_equipment.try_lock();
        if let Ok(equipment) = equipment
            && let Some(head_stack) = equipment.equipment.get(&EquipmentSlot::HEAD)
            && !head_stack.is_empty()
            && head_stack.item == &Item::CARVED_PUMPKIN
        {
            return false;
        }

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
            .raycast(enderman_eye_pos, player_eye_pos, |block_pos, w| {
                let state = w.get_block_state(block_pos);
                state.is_solid()
            })
            .is_none()
    }
}

impl Mob for EndermanEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(block_state) = self.carried_block.load() {
            nbt.put_int("carriedBlockState", block_state.as_u16() as i32);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(block_state) = nbt.get_int("carriedBlockState") {
            self.set_carried_block(BlockStateId::new(block_state as u16));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn set_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        self.set_target(target);
    }

    // TODO: sunlight avoidance, carried block drop on death, angerable system, ambient sound override
    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        let raining_at_feet = world.is_raining_at(&entity.block_pos.load());
        let raining_at_head = world.is_raining_at(&entity.bounding_box.load().max_block_pos());
        if entity.touching_water.load(Ordering::SeqCst) || raining_at_feet || raining_at_head {
            caller.damage(caller, 1.0, DamageType::DROWN);
        }
    }

    fn pre_damage(&self, damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        let is_projectile = is_projectile_damage(damage_type);
        if is_projectile {
            for _ in 0..64 {
                if self.teleport_randomly() {
                    return false;
                }
            }
        }
        true
    }

    fn on_damage(&self, _damage_type: DamageType, source: Option<&dyn EntityBase>) {
        if source.is_some_and(|s| s.get_living_entity().is_some()) {
            return;
        }
        let should_teleport = self.get_random().random_range(0..10) != 0;
        if should_teleport {
            self.teleport_randomly();
        }
    }
}

use crossbeam::atomic::AtomicCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI8, AtomicI32, Ordering::Relaxed};

use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{Entity, EntityBase};
use crate::world::World;

const ROOSTING_FLAG: i8 = 1;
const TICKS_PER_FLAP: i32 = 10;
const MOVEMENT_DISTANCE_SCALE: f32 = 0.6;
const CLOSE_PLAYER_DISTANCE: f64 = 4.0;
const AMBIENT_SOUND_INTERVAL: i32 = 80;

pub struct BatEntity {
    pub mob_entity: MobEntity,
    hanging_position: Mutex<Option<BlockPos>>,
    flags: AtomicI8,
    move_dist: AtomicCell<f32>,
    next_step: AtomicCell<f32>,
    ambient_sound_time: AtomicI32,
}

impl BatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bat = Self {
            mob_entity,
            hanging_position: Mutex::new(None),
            flags: AtomicI8::new(ROOSTING_FLAG),
            move_dist: AtomicCell::new(0.0),
            next_step: AtomicCell::new(1.0),
            ambient_sound_time: AtomicI32::new(0),
        };
        Arc::new(bat)
    }

    #[must_use]
    pub fn check_bat_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if pos.0.y >= world.get_heightmap_height(ChunkHeightmapType::WorldSurface, pos.0.x, pos.0.z)
        {
            return false;
        }
        if rand::random_bool(0.5) {
            return false;
        }
        if world.get_max_local_raw_brightness(pos) > rand::random_range(0..4) {
            return false;
        }
        let below_pos = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
        if !world
            .get_block(&below_pos)
            .has_tag(&tag::Block::MINECRAFT_BATS_SPAWNABLE_ON)
        {
            return false;
        }
        MobEntity::check_mob_spawn_rules(world, pos)
    }

    #[must_use]
    pub fn is_roosting(&self) -> bool {
        self.flags.load(Relaxed) & ROOSTING_FLAG != 0
    }

    pub fn set_roosting(&self, roosting: bool) {
        let flags = if roosting {
            self.flags.fetch_or(ROOSTING_FLAG, Relaxed) | ROOSTING_FLAG
        } else {
            self.flags.fetch_and(!ROOSTING_FLAG, Relaxed) & !ROOSTING_FLAG
        };
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::bat::DATA_ID_FLAGS, flags);
    }

    fn tick_ambient_sound(&self, world: &World, pos: &Vector3<f64>) {
        let entity = self.get_entity();
        if !entity.is_alive() || self.mob_entity.living_entity.is_dead_or_dying() {
            return;
        }
        let roll = rand::random_range(0..1000);
        let time = self.ambient_sound_time.fetch_add(1, Relaxed);
        if roll < time {
            self.ambient_sound_time
                .store(-AMBIENT_SOUND_INTERVAL, Relaxed);
            if (!self.is_roosting() || rand::random_range(0..4) == 0) && !entity.is_silent() {
                world.play_sound_fine(
                    Sound::EntityBatAmbient,
                    SoundCategory::Ambient,
                    pos,
                    self.sound_volume(),
                    self.voice_pitch(),
                );
            }
        }
    }

    fn tick_roosting(&self, world: &World, above_pos: &BlockPos, pos: &Vector3<f64>) {
        let entity = &self.mob_entity.living_entity.entity;
        let is_silent = entity.is_silent();
        let above_state = world.get_block_state(above_pos);
        if above_state.is_solid_block() {
            let rotate_head = {
                let mut rng = rand::rng();
                (rng.random_range(0u32..200) == 0).then(|| rng.random_range(0i32..360) as f32)
            };
            if let Some(head_yaw) = rotate_head {
                entity.head_yaw.store(head_yaw);
            }

            let resting_targeting = TargetPredicate::create_non_attackable()
                .set_base_max_distance(CLOSE_PLAYER_DISTANCE);
            if world
                .get_nearest_player(*pos, CLOSE_PLAYER_DISTANCE, |player| {
                    resting_targeting.test(world, Some(self), player.as_ref())
                })
                .is_some()
            {
                self.set_roosting(false);
                if !is_silent {
                    world.sync_world_event(WorldEvent::SoundBatLiftoff, entity.block_pos.load(), 0);
                }
            }
        } else {
            self.set_roosting(false);
            if !is_silent {
                world.sync_world_event(WorldEvent::SoundBatLiftoff, entity.block_pos.load(), 0);
            }
        }
    }

    fn tick_flying(&self, world: &World, above_pos: &BlockPos, pos: &Vector3<f64>) {
        let entity = &self.mob_entity.living_entity.entity;
        let mut hanging_pos = self
            .hanging_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Vanilla check previous target before randomly generating a new one
        if let Some(target) = *hanging_pos {
            if !world.get_block_state(&target).is_air() || target.0.y <= world.dimension.min_y {
                *hanging_pos = None;
            }
        }

        let mut rng = rand::rng();

        let should_pick_new = hanging_pos.is_none()
            || rng.random_range(0u32..30) == 0
            || hanging_pos.is_some_and(|target| {
                let dx = f64::from(target.0.x) + 0.5 - pos.x;
                let dy = f64::from(target.0.y) + 0.5 - pos.y;
                let dz = f64::from(target.0.z) + 0.5 - pos.z;
                dx * dx + dy * dy + dz * dz < 4.0
            });

        if should_pick_new {
            *hanging_pos = Some(BlockPos::new(
                (pos.x + f64::from(rng.random_range(0i32..7))
                    - f64::from(rng.random_range(0i32..7)))
                .floor() as i32,
                (pos.y + f64::from(rng.random_range(0i32..6)) - 2.0).floor() as i32,
                (pos.z + f64::from(rng.random_range(0i32..7))
                    - f64::from(rng.random_range(0i32..7)))
                .floor() as i32,
            ));
        }

        let Some(target) = *hanging_pos else {
            return;
        };
        drop(hanging_pos);

        let dx = f64::from(target.0.x) + 0.5 - pos.x;
        let dy = f64::from(target.0.y) + 0.1 - pos.y;
        let dz = f64::from(target.0.z) + 0.5 - pos.z;

        // Java Math.signum(0.0) returns 0.0.
        let signum = |value: f64| {
            if value == 0.0 { value } else { value.signum() }
        };

        let velocity = entity.velocity.load();
        let acceleration = f64::from(0.1_f32);
        let vertical_speed = f64::from(0.7_f32);

        let new_velocity = Vector3::new(
            velocity.x + (signum(dx) * 0.5 - velocity.x) * acceleration,
            velocity.y + (signum(dy) * vertical_speed - velocity.y) * acceleration,
            velocity.z + (signum(dz) * 0.5 - velocity.z) * acceleration,
        );
        entity.velocity.store(new_velocity);

        let yaw = (pumpkin_util::math::atan2(new_velocity.z, new_velocity.x) * 57.2957763671875)
            as f32
            - 90.0;
        let old_yaw = entity.yaw.load();
        let yaw_diff = pumpkin_util::math::wrap_degrees(yaw - old_yaw);

        let input = self.mob_entity.living_entity.movement_input.load();
        self.mob_entity
            .living_entity
            .movement_input
            .store(Vector3::new(input.x, input.y, 0.5));

        entity.yaw.store(old_yaw + yaw_diff);

        if rng.random_range(0u32..100) == 0 && world.get_block_state(above_pos).is_solid_block() {
            self.set_roosting(true);
        }
    }
}

impl Mob for BatEntity {
    fn should_check_fall_damage(&self) -> bool {
        false
    }

    fn ignores_block_triggers(&self) -> bool {
        true
    }

    fn movement_emission(&self, movement: Vector3<f64>) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let supporting_pos = entity.get_on_pos();
        let (supporting_block, supporting_state) = world.get_block_and_state(&supporting_pos);
        let is_climbable = |block: &pumpkin_data::Block| {
            block.has_tag(&tag::Block::MINECRAFT_CLIMBABLE)
                || block == &pumpkin_data::Block::POWDER_SNOW
        };
        let climbing = is_climbable(supporting_block);
        let distance = if climbing {
            movement.length()
        } else {
            (movement.x * movement.x + movement.z * movement.z).sqrt()
        };
        let move_dist =
            self.move_dist.load() + (distance * f64::from(MOVEMENT_DISTANCE_SCALE)) as f32;
        self.move_dist.store(move_dist);
        let context = crate::world::game_event::GameEventContext {
            source_entity: Some(self),
        };
        if move_dist > self.next_step.load() && !supporting_state.is_air() {
            let effect_pos = entity.get_block_pos_below_that_affects_my_movement();
            let vibrate_from_block = |pos: &BlockPos, emit: bool| {
                let (block, state) = world.get_block_and_state(pos);
                let climbable = is_climbable(block);
                if !state.is_air()
                    && (entity.on_ground.load(Relaxed)
                        || climbable
                        || entity.pose.load() == pumpkin_data::entity::EntityPose::Crouching
                            && movement.y == 0.0)
                    && !entity.is_swimming()
                {
                    if emit {
                        world.emit_game_event_with_context("step", entity.pos.load(), &context);
                    }
                    true
                } else {
                    false
                }
            };
            let same_position = supporting_pos == effect_pos;
            let mut produced_effects = vibrate_from_block(&effect_pos, same_position);
            if !same_position {
                produced_effects |= vibrate_from_block(&supporting_pos, true);
            }
            if produced_effects || entity.is_in_water() {
                self.next_step.store((move_dist as i32 + 1) as f32);
                if !produced_effects {
                    world.emit_game_event_with_context("swim", entity.pos.load(), &context);
                }
            }
        } else if supporting_state.is_air()
            && !self.is_roosting()
            && entity.tick_count.load(Relaxed) % TICKS_PER_FLAP == 0
        {
            world.emit_game_event_with_context("flap", entity.pos.load(), &context);
        }
    }

    fn can_be_leashed(&self) -> bool {
        false
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_is_pushable(&self) -> bool {
        false
    }

    fn pushes_entities(&self) -> bool {
        false
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let flags = self.flags.load(Relaxed);
        entity.set_synced_data(tracked_data::bat::DATA_ID_FLAGS, flags);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("BatFlags", self.flags.load(Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        let flags = nbt.get_byte("BatFlags").unwrap_or(0);
        self.flags.store(flags, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::bat::DATA_ID_FLAGS, flags);
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        Some(crate::entity::mob::MobMetadata::new().bat_flags(self.flags.load(Relaxed)))
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn base_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        self.tick_ambient_sound(&entity.world.load(), &entity.pos.load());
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        let block_pos = entity.block_pos.load();
        let above_pos = BlockPos::new(block_pos.0.x, block_pos.0.y + 1, block_pos.0.z);
        let world = entity.world.load();
        let pos = entity.pos.load();

        if self.is_roosting() {
            self.tick_roosting(&world, &above_pos, &pos);
        } else {
            self.tick_flying(&world, &above_pos, &pos);
        }
    }

    fn post_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if self.is_roosting() {
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
            let pos = entity.pos.load();
            let snapped_y = (pos.y.floor()) + 1.0 - f64::from(entity.height());
            entity.set_pos(Vector3::new(pos.x, snapped_y, pos.z));
        } else {
            let velocity = entity.velocity.load();
            entity
                .velocity
                .store(Vector3::new(velocity.x, velocity.y * 0.6, velocity.z));
        }
    }

    fn pre_damage_with_context(
        &self,
        damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.get_entity().is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        if self.is_roosting() {
            self.set_roosting(false);
        }
        true
    }
}

impl CustomSound for BatEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBatDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBatHurt)
    }

    fn sound_volume(&self) -> f32 {
        0.1
    }

    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Ambient
    }

    fn voice_pitch(&self) -> f32 {
        self.mob_entity.living_entity.get_pitch() * 0.95
    }

    fn on_hurt_sound(&self) {
        self.ambient_sound_time
            .store(-AMBIENT_SOUND_INTERVAL, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::entity::EntityType;

    struct EventListener {
        mob: MobEntity,
        events: Mutex<Vec<pumpkin_data::game_event::GameEvent>>,
    }

    impl Mob for EventListener {
        fn get_mob_entity(&self) -> &MobEntity {
            &self.mob
        }

        fn for_each_game_event_listener(
            &self,
            visit: &mut dyn FnMut(&dyn crate::world::game_event::GameEventListener),
        ) {
            visit(self);
        }
    }

    impl crate::world::game_event::GameEventListener for EventListener {
        fn listener_position(&self) -> Vector3<f64> {
            self.get_entity().pos.load()
        }

        fn listener_radius(&self) -> i32 {
            16
        }

        fn handle_game_event(
            &self,
            _world: &World,
            event: pumpkin_data::game_event::GameEvent,
            _context: &crate::world::game_event::GameEventContext<'_>,
            _source_position: Vector3<f64>,
        ) -> bool {
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(event);
            true
        }
    }

    #[tokio::test]
    async fn movement_emits_flap_only_on_flying_flap_ticks()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world.level.loaded_chunks.insert(
            pumpkin_util::math::vector2::Vector2::new(0, 0),
            pumpkin_world::chunk::ChunkData::empty_sync(0, 0),
        );
        let pos = Vector3::new(0.5, 64.0, 0.5);
        let listener = Arc::new(EventListener {
            mob: MobEntity::new(Entity::new(world.clone(), pos, &EntityType::COW)),
            events: Mutex::new(Vec::new()),
        });
        world.add_entity_silent(listener.clone());
        let bat = BatEntity::new(Entity::new(world.clone(), pos, &EntityType::BAT));
        bat.set_roosting(false);
        let entity = bat.get_entity();
        let motion = Vector3::new(0.1, 0.0, 0.0);
        entity.tick_count.store(9, Relaxed);
        entity.move_entity(bat.as_ref(), motion);
        entity.tick_count.store(10, Relaxed);
        entity.move_entity(bat.as_ref(), motion);
        bat.set_roosting(true);
        entity.tick_count.store(20, Relaxed);
        entity.move_entity(bat.as_ref(), motion);
        let events = listener
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        world.level.shutdown().await;
        assert_eq!(events, vec![pumpkin_data::game_event::GameEvent::Flap]);
        Ok(())
    }

    #[tokio::test]
    async fn bat_flags_preserve_other_bits_when_resting_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let bat = BatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BAT,
        ));
        let mut saved = NbtCompound::new();
        saved.put_byte("BatFlags", -127);
        bat.mob_read_nbt(&saved);
        assert!(bat.is_roosting());
        bat.mob_write_nbt(&mut saved);
        assert_eq!(saved.get_byte("BatFlags"), Some(-127));
        bat.set_roosting(false);
        bat.mob_write_nbt(&mut saved);
        assert_eq!(saved.get_byte("BatFlags"), Some(-128));
        bat.set_roosting(true);
        bat.mob_write_nbt(&mut saved);
        assert_eq!(saved.get_byte("BatFlags"), Some(-127));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn bat_does_not_power_pressure_plates_or_accumulate_fall_distance()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::block::blocks::redstone::pressure_plate::{
            PressurePlate, plate::PressurePlateBlock, weighted::WeightedPressurePlateBlock,
        };
        use pumpkin_data::Block;

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = BlockPos::new(0, 64, 0);
        let bat = BatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::BAT,
        ));
        world.add_entity_silent(bat.clone());
        assert_eq!(
            world
                .get_entities_at_box(&pumpkin_util::math::boundingbox::BoundingBox::from_block(
                    &pos
                ))
                .len(),
            1
        );
        let normal =
            PressurePlateBlock.calculate_redstone_output(&world, &Block::OAK_PRESSURE_PLATE, &pos);
        let weighted = WeightedPressurePlateBlock.calculate_redstone_output(
            &world,
            &Block::LIGHT_WEIGHTED_PRESSURE_PLATE,
            &pos,
        );
        bat.mob_entity
            .living_entity
            .fall(bat.as_ref(), -10.0, false, false);
        let fall_distance = bat.mob_entity.living_entity.fall_distance.load();
        world.level.shutdown().await;
        assert_eq!(normal, 0);
        assert_eq!(weighted, 0);
        assert_eq!(fall_distance, 0.0);
        Ok(())
    }

    #[tokio::test]
    async fn ambient_sound_timer_resets_and_waits_after_hurt()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = Vector3::new(0.0, 64.0, 0.0);
        let bat = BatEntity::new(Entity::new(world.clone(), pos, &EntityType::BAT));
        bat.get_entity().set_silent(true);
        bat.ambient_sound_time.store(1000, Relaxed);
        bat.tick_ambient_sound(&world, &pos);
        let reset_time = bat.ambient_sound_time.load(Relaxed);

        bat.ambient_sound_time.store(10, Relaxed);
        bat.on_hurt_sound();
        for _ in 0..80 {
            bat.tick_ambient_sound(&world, &pos);
        }
        let after_delay = bat.ambient_sound_time.load(Relaxed);
        bat.tick_ambient_sound(&world, &pos);
        let after_zero = bat.ambient_sound_time.load(Relaxed);
        bat.mob_entity.living_entity.set_health(0.0);
        bat.tick_ambient_sound(&world, &pos);
        let after_death = bat.ambient_sound_time.load(Relaxed);
        world.level.shutdown().await;

        assert_eq!(reset_time, -80);
        assert_eq!(after_delay, 0);
        assert_eq!(after_zero, 1);
        assert_eq!(after_death, 1);
        Ok(())
    }

    #[tokio::test]
    async fn damage_wakes_bat_before_hurt_cooldown_but_not_when_invulnerable()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let bat = BatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BAT,
        ));
        let entity = bat.get_entity();
        entity.set_invulnerable(true);
        let invulnerable_damage = bat.damage(bat.as_ref(), 1.0, DamageType::GENERIC);
        let stayed_roosting = bat.is_roosting();

        entity.set_invulnerable(false);
        bat.mob_entity
            .living_entity
            .hurt_cooldown
            .store(20, Relaxed);
        bat.mob_entity.living_entity.last_damage_taken.store(2.0);
        let cooldown_damage = bat.damage(bat.as_ref(), 1.0, DamageType::GENERIC);
        let woke_up = !bat.is_roosting();
        world.level.shutdown().await;

        assert!(!invulnerable_damage);
        assert!(stayed_roosting);
        assert!(!cooldown_damage);
        assert!(woke_up);
        Ok(())
    }
}

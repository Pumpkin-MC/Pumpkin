use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Weak};

use crate::entity::ai::control::{Control, LookControlTrait};
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::{World, game_event::GameEventContext};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, ShulkerColorImpl};
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use rand::RngExt;

use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::projectile::shulker_bullet::ShulkerBulletEntity;
use crate::entity::{Entity, EntityBase};

const DEFAULT_ATTACH_FACE: BlockDirection = BlockDirection::Down;
const NO_COLOR: u8 = 16;
const PEEK_PER_TICK: f32 = 0.05;
const MAX_TELEPORT_DISTANCE: i32 = 8;
const OTHER_SHULKER_SCAN_RADIUS: f64 = 8.0;
const OTHER_SHULKER_LIMIT: f32 = 5.0;
const COVERED_ARMOR_MODIFIER_ID: &str = "minecraft:covered";
pub(crate) const MAX_SCALE: f32 = 3.0;

pub struct ShulkerEntity {
    pub mob_entity: MobEntity,
    attach_face: AtomicU8,
    peek_amount: AtomicU8,
    color: AtomicU8,
    current_peek_amount: AtomicCell<f32>,
    prev_peek_amount: AtomicCell<f32>,
}

impl ShulkerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.set_pos(Self::snap_position(entity.pos.load()));
        let mob_entity = MobEntity::new(entity);
        let shulker = Self {
            mob_entity,
            attach_face: AtomicU8::new(DEFAULT_ATTACH_FACE as u8),
            peek_amount: AtomicU8::new(0),
            color: AtomicU8::new(NO_COLOR),
            current_peek_amount: AtomicCell::new(0.0),
            prev_peek_amount: AtomicCell::new(0.0),
        };
        let mob_arc = Arc::new(shulker);
        let _ = mob_arc.get_entity().shulker.set(Arc::downgrade(&mob_arc));
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(
                1,
                Box::new(LookAtEntityGoal::new(
                    mob_weak,
                    &EntityType::PLAYER,
                    8.0,
                    0.02,
                    true,
                )),
            );
            goal_selector.add_goal(
                4,
                Box::new(ShulkerAttackGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(7, Box::new(ShulkerPeekGoal::new(Arc::downgrade(&mob_arc))));
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true)
                        .ignoring(|entity_type| entity_type == &EntityType::SHULKER)
                        .alerting_others(),
                ),
            );
            for (priority, defense) in [(2, false), (3, true)] {
                target_selector.add_goal(
                    priority,
                    Box::new(ShulkerNearestAttackGoal::new(&mob_arc.mob_entity, defense)),
                );
            }
            *mob_arc
                .mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Box::new(ShulkerLookControl {
                    shulker: Arc::downgrade(&mob_arc),
                    position: Vector3::default(),
                    timer: 0,
                    max_yaw_change: 0.0,
                });
        };

        mob_arc
    }

    pub fn get_attach_face(&self) -> BlockDirection {
        BlockDirection::try_from(i32::from(self.attach_face.load(Ordering::Relaxed)))
            .unwrap_or(DEFAULT_ATTACH_FACE)
    }

    pub fn set_attach_face(&self, face: BlockDirection) {
        self.attach_face.store(face as u8, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(
            pumpkin_data::tracked_data::shulker::ATTACH_FACE_ID,
            VarInt(face as i32),
        );
        entity.refresh_dimensions_without_relocation();
    }

    pub fn get_color(&self) -> Option<u8> {
        let color = self.color.load(Ordering::Relaxed) as i8;
        (color < NO_COLOR as i8).then_some(color.max(0) as u8)
    }

    pub fn set_color(&self, color: Option<u8>) {
        let val = color.unwrap_or(NO_COLOR);
        self.color.store(val, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::shulker::COLOR, val as i8);
    }

    pub fn get_raw_peek(&self) -> u8 {
        self.peek_amount.load(Ordering::Relaxed)
    }

    pub fn set_raw_peek(&self, amount: u8) {
        let living = &self.mob_entity.living_entity;
        living.update_attribute(&Attributes::ARMOR, |attribute| {
            attribute.remove_modifier(COVERED_ARMOR_MODIFIER_ID);
            if amount == 0 {
                attribute.add_or_replace_permanent_modifier(Modifier {
                    id: COVERED_ARMOR_MODIFIER_ID.into(),
                    amount: 20.0,
                    operation: ModifierOperation::Add,
                });
            }
        });
        self.play_entity_sound(
            if amount == 0 {
                Sound::EntityShulkerClose
            } else {
                Sound::EntityShulkerOpen
            },
            1.0,
            1.0,
        );
        let entity = &living.entity;
        entity.world.load().emit_game_event_with_context(
            if amount == 0 {
                GameEvent::ContainerClose
            } else {
                GameEvent::ContainerOpen
            }
            .name(),
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(self),
            },
        );
        self.store_peek(amount);
    }

    // Vanilla's direct DATA_PEEK_ID writes leave the covered modifier unchanged.
    fn store_peek(&self, amount: u8) {
        self.peek_amount.store(amount, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::shulker::PEEK_ID, amount);
    }

    pub fn is_closed(&self) -> bool {
        self.peek_amount.load(Ordering::Relaxed) == 0
    }

    fn update_peek_amount(&self) -> bool {
        let current = self.current_peek_amount.load();
        self.prev_peek_amount.store(current);
        let target = f32::from(self.get_raw_peek() as i8) * 0.01;

        if current == target {
            return false;
        }

        let (value, min, max) = if current > target {
            (current - PEEK_PER_TICK, target, 1.0)
        } else {
            (current + PEEK_PER_TICK, 0.0, target)
        };
        // Saved bytes can put the bounds in reverse order; match Mth.clamp there too.
        let new = if value < min { min } else { value.min(max) };

        self.current_peek_amount.store(new);
        true
    }

    pub(crate) fn snap_position(pos: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            pos.x.floor() + 0.5,
            (pos.y + 0.5).floor(),
            pos.z.floor() + 0.5,
        )
    }

    pub(crate) fn on_position_changed(&self, old: BlockPos) {
        let entity = self.get_entity();
        if entity.tick_count.load(Ordering::Relaxed) != 0 && old != entity.block_pos.load() {
            self.store_peek(0);
            entity.velocity_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn physical_peek(amount: f32) -> f32 {
        0.5 - pumpkin_util::math::sin((0.5 + amount) * std::f32::consts::PI) * 0.5
    }

    fn progress_aabb(
        size: f32,
        direction: BlockDirection,
        from: f32,
        to: f32,
        pos: Vector3<f64>,
    ) -> BoundingBox {
        let size = f64::from(size);
        let offset = direction.to_offset();
        let movement = Vector3::new(
            f64::from(offset.x),
            f64::from(offset.y),
            f64::from(offset.z),
        );
        let mut bounds = BoundingBox {
            min: Vector3::new(-size * 0.5, 0.0, -size * 0.5),
            max: Vector3::new(size * 0.5, size, size * 0.5),
        }
        .expand_towards(
            movement.x * f64::from(from.max(to)) * size,
            movement.y * f64::from(from.max(to)) * size,
            movement.z * f64::from(from.max(to)) * size,
        );
        let inset = (1.0 + f64::from(from.min(to))) * size;
        match direction {
            BlockDirection::Down => bounds.max.y -= inset,
            BlockDirection::Up => bounds.min.y += inset,
            BlockDirection::North => bounds.max.z -= inset,
            BlockDirection::South => bounds.min.z += inset,
            BlockDirection::West => bounds.max.x -= inset,
            BlockDirection::East => bounds.min.x += inset,
        }
        bounds.shift(pos)
    }

    pub(crate) fn height_scale(&self) -> f32 {
        let peek = self.current_peek_amount.load();
        if self.get_attach_face() == BlockDirection::Down && peek > 0.0 {
            1.0 + peek
        } else {
            1.0
        }
    }

    pub(crate) fn make_bounding_box(&self, pos: Vector3<f64>) -> BoundingBox {
        Self::progress_aabb(
            self.get_entity().scale.load(),
            self.get_attach_face().opposite(),
            -1.0,
            Self::physical_peek(self.current_peek_amount.load()),
            pos,
        )
    }

    fn on_peek_amount_change(&self) {
        let entity = self.get_entity();
        entity.refresh_dimensions_without_relocation();
        let old = Self::physical_peek(self.prev_peek_amount.load());
        let current = Self::physical_peek(self.current_peek_amount.load());
        let push = (current - old) * entity.scale.load();
        if push <= 0.0 {
            return;
        }
        let direction = self.get_attach_face().opposite();
        let area = Self::progress_aabb(
            entity.scale.load(),
            direction,
            old,
            current,
            entity.pos.load(),
        );
        let offset = direction.to_offset();
        let motion = Vector3::new(
            f64::from(offset.x) * f64::from(push),
            f64::from(offset.y) * f64::from(push),
            f64::from(offset.z) * f64::from(push),
        );
        for other in entity.world.load().get_all_at_box(&area) {
            if !other.is_spectator()
                && other.get_entity().entity_type != &EntityType::SHULKER
                && !other.get_entity().no_physics.load(Ordering::Relaxed)
                && !self.is_passenger_of_same_vehicle(other.as_ref())
            {
                other.move_by_shulker(other.as_ref(), motion);
            }
        }
    }

    fn find_attachable_face(&self, pos: &BlockPos) -> Option<BlockDirection> {
        BlockDirection::all()
            .into_iter()
            .find(|face| self.can_stay_at(pos, *face))
    }

    fn find_teleport_attachment(&self, pos: &BlockPos) -> Option<BlockDirection> {
        let entity = self.get_entity();
        let world = entity.world.load();
        if pos.0.y <= world.dimension.min_y
            || !world
                .get_block_state_if_loaded(pos)
                .is_some_and(pumpkin_data::BlockState::is_air)
            || !world
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains_block(pos.0.x, pos.0.z)
            || !self.no_collision(BoundingBox::from_block(pos).contract_all(1.0E-6))
        {
            return None;
        }
        BlockDirection::all().into_iter().find(|face| {
            let support = pos.offset(face.to_offset());
            if world
                .get_block_state(&support)
                .id
                .to_block()
                .has_tag(&tag::Block::MINECRAFT_SHULKER_DOES_NOT_TELEPORT_TO)
            {
                return false;
            }
            let bounds = Self::progress_aabb(
                entity.scale.load(),
                face.opposite(),
                -1.0,
                1.0,
                pos.to_f64().add_raw(0.5, 0.0, 0.5),
            )
            .contract_all(1.0E-6);
            let min = bounds.min_block_pos();
            let max = bounds.max_block_pos();
            world.has_chunks_at(min.0.x, min.0.z, max.0.x, max.0.z) && self.can_stay_at(pos, *face)
        })
    }

    fn no_collision(&self, bounds: BoundingBox) -> bool {
        let world = self.get_entity().world.load();
        world.get_block_collisions(bounds, self).0.is_empty()
            && world.no_entity_collision(self, bounds)
            && world.no_border_collision(self, bounds)
    }

    fn can_stay_at(&self, pos: &BlockPos, face: BlockDirection) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let own_state = world.get_block_state(pos);
        let moving_piston =
            own_state.id.to_block() == &Block::MOVING_PISTON && *pos == entity.block_pos.load();
        if !(own_state.is_air() || moving_piston) {
            return false;
        }
        let neighbour = pos.offset(face.to_offset());
        if !world.loaded_and_entity_can_stand_on_face(&neighbour, self, face.opposite()) {
            return false;
        }
        let position = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y),
            f64::from(pos.0.z) + 0.5,
        );
        self.no_collision(
            Self::progress_aabb(entity.scale.load(), face.opposite(), -1.0, 1.0, position)
                .contract_all(1.0E-6),
        )
    }

    fn find_new_attachment(&self) {
        let pos = self.mob_entity.living_entity.entity.block_pos.load();
        if let Some(dir) = self.find_attachable_face(&pos) {
            self.set_attach_face(dir);
        } else {
            self.teleport_somewhere();
        }
    }

    pub fn teleport_somewhere(&self) -> bool {
        let entity = self.get_entity();
        if self.mob_entity.is_no_ai() || !self.mob_entity.living_entity.is_alive() {
            return false;
        }
        let old_pos = entity.block_pos.load();
        let world = entity.world.load();
        let mut rng = rand::rng();
        for _ in 0..5 {
            let candidate = old_pos.offset(Vector3::new(
                rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
            ));
            if self.find_teleport_attachment(&candidate).is_none() {
                continue;
            }
            let new_pos = Vector3::new(
                f64::from(candidate.0.x) + 0.5,
                f64::from(candidate.0.y),
                f64::from(candidate.0.z) + 0.5,
            );
            if let Some(server) = world.server.upgrade() {
                let mut event =
                    crate::plugin::api::events::entity::entity_teleport::EntityTeleportEvent::new(
                        entity.entity_id,
                        entity.pos.load(),
                        new_pos,
                    );
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return false;
                }
            }
            if let Some(vehicle) = entity.get_vehicle() {
                if self.find_teleport_attachment(&candidate).is_none() {
                    continue;
                }
                vehicle
                    .get_entity()
                    .remove_passenger_before_teleport_if(entity.entity_id, &|| {
                        self.mob_entity.living_entity.is_alive()
                            && self.find_teleport_attachment(&candidate).is_some()
                    });
                if entity.has_vehicle() {
                    return false;
                }
            }
            if !self.mob_entity.living_entity.is_alive() {
                return false;
            }
            let Some(face) = self.find_teleport_attachment(&candidate) else {
                continue;
            };
            self.set_attach_face(face);
            self.play_entity_sound(Sound::EntityShulkerTeleport, 1.0, 1.0);
            entity.set_pos(new_pos);
            entity.velocity_dirty.store(true, Ordering::Relaxed);
            world.emit_game_event_with_context(
                GameEvent::Teleport.name(),
                Vector3::new(
                    f64::from(old_pos.0.x),
                    f64::from(old_pos.0.y),
                    f64::from(old_pos.0.z),
                ),
                &GameEventContext {
                    source_entity: Some(self),
                },
            );
            let packed = ((candidate.0.x - old_pos.0.x + MAX_TELEPORT_DISTANCE) << 16)
                | ((candidate.0.y - old_pos.0.y + MAX_TELEPORT_DISTANCE) << 8)
                | (candidate.0.z - old_pos.0.z + MAX_TELEPORT_DISTANCE);
            world.sync_world_event(WorldEvent::ParticlesShulkerTeleport, old_pos, packed);
            self.store_peek(0);
            self.set_mob_target(None, TargetReason::Unknown);
            return true;
        }
        false
    }

    fn hit_by_shulker_bullet(&self) {
        let entity = self.get_entity();
        let old_pos = entity.pos.load();
        let old_box = entity.bounding_box.load();
        if self.is_closed() || !self.teleport_somewhere() {
            return;
        }
        let world = entity.world.load();
        let count = world
            .get_entities_of_type_at_box(&old_box.expand_all(OTHER_SHULKER_SCAN_RADIUS), |kind| {
                kind == &EntityType::SHULKER
            })
            .iter()
            .filter(|other| {
                other
                    .get_living_entity()
                    .is_some_and(crate::entity::living::LivingEntity::is_alive)
            })
            .count();
        if rand::rng().random::<f32>() < (count as f32 - 1.0) / OTHER_SHULKER_LIMIT {
            return;
        }
        let clone = Self::new(Entity::from_uuid(
            uuid::Uuid::new_v4(),
            (*world).clone(),
            old_pos,
            &EntityType::SHULKER,
        ));
        clone.set_color(self.get_color());
        world.spawn_entity(clone);
    }
}

impl Mob for ShulkerEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("AttachFace", self.attach_face.load(Ordering::Relaxed) as i8);
        nbt.put_byte("Peek", self.peek_amount.load(Ordering::Relaxed) as i8);
        nbt.put_byte("Color", self.color.load(Ordering::Relaxed) as i8);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        let face = nbt
            .get_byte("AttachFace")
            .and_then(|value| BlockDirection::by_index(i32::from(value).unsigned_abs() as usize))
            .unwrap_or(DEFAULT_ATTACH_FACE);
        self.set_attach_face(face);
        self.store_peek(
            nbt.get_byte("Peek")
                .or_else(|| nbt.get_byte("PeekAmount"))
                .unwrap_or(0) as u8,
        );
        let color = nbt.get_byte("Color").unwrap_or(NO_COLOR as i8) as u8;
        self.color.store(color, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::shulker::COLOR, color as i8);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn post_tick(&self) {
        let entity = self.get_entity();
        if !entity.has_vehicle()
            && !self.can_stay_at(&entity.block_pos.load(), self.get_attach_face())
        {
            self.find_new_attachment();
        }
        if self.update_peek_amount() {
            self.on_peek_amount_change();
        }
    }

    fn on_damage(&self, damage_type: DamageType, source: Option<&dyn EntityBase>) {
        let living = &self.mob_entity.living_entity;
        if living.health.load() < living.get_max_health() * 0.5
            && self.get_random().random_range(0..4) == 0
        {
            self.teleport_somewhere();
        } else if damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)
            && source.is_some_and(|source| {
                source.get_entity().entity_type == &EntityType::SHULKER_BULLET
            })
        {
            self.hit_by_shulker_bullet();
        }
    }

    fn pre_damage(&self, _damage_type: DamageType, source: Option<&dyn EntityBase>) -> bool {
        !(self.is_closed()
            && source.is_some_and(|source| {
                crate::entity::projectile::abstract_arrow::is_abstract_arrow(
                    source.get_entity().entity_type,
                )
            }))
    }

    fn additional_tick_movement(&self) -> f64 {
        f64::from(MAX_TELEPORT_DISTANCE) * 3.0f64.sqrt()
    }

    fn can_be_leashed(&self) -> bool {
        false
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        false
    }
    fn ticks_body_rotation(&self) -> bool {
        false
    }
    fn get_max_look_pitch_change(&self) -> f32 {
        180.0
    }
    fn get_max_head_rotation(&self) -> f32 {
        180.0
    }
    fn get_ambient_sound_interval(&self) -> i32 {
        120
    }
    fn get_ambient_sound(&self) -> Option<Sound> {
        (!self.is_closed()).then_some(Sound::EntityShulkerAmbient)
    }
    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let entity = self.get_entity();
        entity.set_rotation(0.0, entity.pitch.load());
        entity.head_yaw.store(0.0);
        entity.body_yaw.store(0.0);
        self.default_finalize_spawn(world, group_data)
    }

    fn mob_init_data_tracker(&self) {
        self.set_attach_face(self.get_attach_face());
        self.store_peek(self.get_raw_peek());
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::shulker::COLOR,
            self.color.load(Ordering::Relaxed) as i8,
        );
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        if component != DataComponent::ShulkerColor {
            return None;
        }
        self.get_color().map(|color| {
            Box::new(ShulkerColorImpl {
                value: DyeColor::from(color).name().into(),
            }) as Box<dyn DataComponentImpl>
        })
    }

    fn mob_apply_implicit_component(&self, component: DataComponent, value: &str) {
        if component == DataComponent::ShulkerColor
            && let Some(color) = DyeColor::ALL.iter().find(|color| color.name() == value)
        {
            self.set_color(Some(color.id()));
        }
    }
}

impl CustomSound for ShulkerEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityShulkerDeath)
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_closed() {
            Sound::EntityShulkerHurtClosed
        } else {
            Sound::EntityShulkerHurt
        })
    }
}

struct ShulkerNearestAttackGoal {
    inner: Box<ActiveTargetGoal>,
    defense: bool,
}

impl ShulkerNearestAttackGoal {
    fn new(mob: &MobEntity, defense: bool) -> Self {
        let mut inner = ActiveTargetGoal::with_default(
            mob,
            if defense {
                TargetMatch::Enemy
            } else {
                TargetMatch::Type(&EntityType::PLAYER)
            },
            true,
        );
        if defense {
            inner = inner.selecting(|mob, target, _world| {
                let Some(shulker) = mob.cast_any().downcast_ref::<ShulkerEntity>() else {
                    return false;
                };
                let range = mob
                    .get_mob_entity()
                    .living_entity
                    .get_attribute_value(&Attributes::FOLLOW_RANGE);
                let (x, y, z) = match shulker.get_attach_face().to_axis() {
                    Axis::X => (4.0, range, range),
                    Axis::Y => (range, 4.0, range),
                    Axis::Z => (range, range, 4.0),
                };
                mob.get_entity()
                    .bounding_box
                    .load()
                    .expand(x, y, z)
                    .intersects(&target.entity.bounding_box.load())
            });
        }
        Self { inner, defense }
    }
}

impl Goal for ShulkerNearestAttackGoal {
    fn controls(&self) -> Controls {
        self.inner.controls()
    }
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.defense {
            if mob.get_team().is_none() {
                return false;
            }
        } else if mob.get_entity().world.load().level_info.load().difficulty == Difficulty::Peaceful
        {
            return false;
        }
        self.inner.can_start(mob)
    }
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }
    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }
    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }
}

struct ShulkerAttackGoal {
    shulker: Weak<ShulkerEntity>,
    attack_cooldown: i32,
}

impl ShulkerAttackGoal {
    const fn new(shulker: Weak<ShulkerEntity>) -> Self {
        Self {
            shulker,
            attack_cooldown: 20,
        }
    }
}

impl Goal for ShulkerAttackGoal {
    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_entity().world.load().level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }
        let target = mob.get_mob_entity().get_target();
        target.as_ref().is_some_and(|t| {
            t.get_living_entity()
                .is_some_and(crate::entity::living::LivingEntity::is_alive)
        })
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.attack_cooldown = 20;
        if let Some(shulker) = self.shulker.upgrade() {
            shulker.set_raw_peek(100);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(shulker) = self.shulker.upgrade() {
            shulker.set_raw_peek(0);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(shulker) = self.shulker.upgrade() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let target_arc = mob_entity.get_target();

        let Some(target) = target_arc else {
            return;
        };

        if mob.get_entity().world.load().level_info.load().difficulty == Difficulty::Peaceful {
            return;
        }
        if !target.get_entity().is_alive() {
            return;
        }

        let entity = &mob_entity.living_entity.entity;
        let shulker_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = shulker_pos.squared_distance_to_vec(&target_pos);

        mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 180.0, 180.0);
        self.attack_cooldown -= 1;
        if dist_sq >= 400.0 {
            mob.set_mob_target(None, TargetReason::Unknown);
            return;
        }

        if self.attack_cooldown <= 0 {
            let new_cd = 20 + mob.get_random().random_range(0..10) * 10;
            self.attack_cooldown = new_cd;

            let world = entity.world.load();
            let bullet = ShulkerBulletEntity::new(
                entity,
                target.get_entity(),
                shulker.get_attach_face().to_axis(),
            );
            let mut launch_event =
                crate::plugin::api::events::entity::projectile_launch::ProjectileLaunchEvent::new(
                    bullet.entity.entity_id,
                    Some(entity.entity_id),
                );
            if let Some(server) = world.server.upgrade() {
                server
                    .plugin_manager
                    .fire_blocking(&server, &mut launch_event);
            }
            if !launch_event.cancelled {
                world.spawn_entity_non_save(Arc::new(bullet));
            }

            let pitch =
                1.0 + (mob.get_random().random::<f32>() - mob.get_random().random::<f32>()) * 0.2;
            shulker.play_entity_sound(Sound::EntityShulkerShoot, 2.0, pitch);
        }
    }
}

struct ShulkerPeekGoal {
    shulker: Weak<ShulkerEntity>,
    peek_time: i32,
}

impl ShulkerPeekGoal {
    const fn new(shulker: Weak<ShulkerEntity>) -> Self {
        Self {
            shulker,
            peek_time: 0,
        }
    }
}

impl Goal for ShulkerPeekGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(shulker) = self.shulker.upgrade() else {
            return false;
        };
        let has_target = mob.get_mob_entity().get_target().is_some();
        if has_target {
            return false;
        }
        if mob.get_random().random_range(0..to_goal_ticks(40)) != 0 {
            return false;
        }
        let pos = mob.get_mob_entity().living_entity.entity.block_pos.load();
        let face = shulker.get_attach_face();
        shulker.can_stay_at(&pos, face)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let has_target = mob.get_mob_entity().get_target().is_some();
        !has_target && self.peek_time > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        let duration = self.get_tick_count(20 * (1 + mob.get_random().random_range(0..3)));
        self.peek_time = duration;
        if let Some(shulker) = self.shulker.upgrade() {
            shulker.set_raw_peek(30);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        let Some(shulker) = self.shulker.upgrade() else {
            return;
        };
        let has_target = shulker.mob_entity.get_target().is_some();
        if !has_target {
            shulker.set_raw_peek(0);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.peek_time -= 1;
    }
}

struct ShulkerLookControl {
    shulker: Weak<ShulkerEntity>,
    position: Vector3<f64>,
    timer: i32,
    max_yaw_change: f32,
}

impl Control for ShulkerLookControl {}

impl LookControlTrait for ShulkerLookControl {
    fn wanted_position(&self) -> Option<Vector3<f64>> {
        Some(self.position)
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        _max_pitch_change: f32,
    ) {
        self.position = Vector3::new(x, y, z);
        self.max_yaw_change = max_yaw_change;
        self.timer = 2;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        entity.set_pitch(0.0);
        if self.timer <= 0 {
            entity.head_yaw.store(self.change_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                10.0,
            ));
            return;
        }
        self.timer -= 1;
        let Some(shulker) = self.shulker.upgrade() else {
            return;
        };
        let delta = (self.position - entity.get_eye_pos()).to_f32_lossy();
        // Direction.getRotation transforms SOUTH into this forward vector on each face.
        let (right, forward) = match shulker.get_attach_face().opposite() {
            BlockDirection::Up => (delta.x, delta.z),
            BlockDirection::Down => (delta.x, -delta.z),
            BlockDirection::North => (-delta.x, -delta.y),
            BlockDirection::South => (delta.x, -delta.y),
            BlockDirection::West => (delta.z, -delta.y),
            BlockDirection::East => (-delta.z, -delta.y),
        };
        if right.abs() > 1.0E-5 || forward.abs() > 1.0E-5 {
            let yaw = (pumpkin_util::math::atan2(f64::from(-right), f64::from(forward)) * 180.0
                / f64::from(std::f32::consts::PI)) as f32;
            entity.head_yaw.store(self.change_angle(
                entity.head_yaw.load(),
                yaw,
                self.max_yaw_change,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::world::BlockFlags;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_world::chunk::ChunkData;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn spawn(world: &Arc<World>) -> Arc<ShulkerEntity> {
        let shulker = ShulkerEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SHULKER,
        ));
        world.spawn_entity(shulker.clone());
        shulker
    }

    #[tokio::test]
    async fn vanilla_nbt_restores_peek_face_and_color_without_toggling_armor() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let shulker = spawn(&world);
        let mut nbt = NbtCompound::new();
        nbt.put_byte("AttachFace", 5);
        nbt.put_byte("PeekAmount", 30);
        shulker.mob_read_nbt(&nbt);
        assert_eq!(shulker.get_raw_peek(), 30);
        nbt.put_byte("Peek", 100);
        nbt.put_byte("Color", 11);
        shulker.mob_read_nbt(&nbt);
        assert_eq!(shulker.get_attach_face(), BlockDirection::East);
        assert_eq!(shulker.get_color(), Some(11));
        assert_eq!(
            shulker
                .mob_entity
                .living_entity
                .get_attribute_value(&Attributes::ARMOR),
            0.0
        );
        for _ in 0..10 {
            assert!(shulker.update_peek_amount());
        }
        shulker.on_peek_amount_change();
        let bounds = shulker.get_entity().bounding_box.load();
        assert!((bounds.min.x + 0.5).abs() < 1.0E-4);
        assert_eq!(bounds.max.x, 1.0);
        let mut saved = NbtCompound::new();
        shulker.mob_write_nbt(&mut saved);
        assert_eq!(saved.get_byte("Peek"), Some(100));
        assert!(!saved.has("PeekAmount"));
        assert_eq!(saved.get_byte("AttachFace"), Some(5));
        assert_eq!(saved.get_byte("Color"), Some(11));
        shulker.get_entity().tick_count.store(1, Ordering::Relaxed);
        shulker
            .get_entity()
            .teleport(Vector3::new(-0.1, 65.6, -0.1), None, None, &world);
        assert_eq!(
            shulker.get_entity().pos.load(),
            Vector3::new(-0.5, 66.0, -0.5)
        );
        assert!(shulker.is_closed());
        assert_eq!(
            shulker.get_entity().last_sent_pos.load(),
            shulker.get_entity().pos.load(),
        );
        assert!((shulker.get_entity().bounding_box.load().min.x + 1.5).abs() < 1.0E-4);
        let mut placement = NbtCompound::new();
        placement.put_list(
            "Pos",
            [10.2, 64.4, 3.8]
                .into_iter()
                .map(pumpkin_nbt::tag::NbtTag::Double)
                .collect(),
        );
        shulker.get_entity().read_placement_nbt(&placement);
        assert_eq!(
            shulker.get_entity().pos.load(),
            Vector3::new(10.5, 64.0, 3.5),
        );
        assert_eq!(
            shulker.get_entity().last_sent_pos.load(),
            shulker.get_entity().pos.load(),
        );
        shulker.mob_read_nbt(&NbtCompound::new());
        assert_eq!(shulker.get_attach_face(), BlockDirection::Down);
        assert!(shulker.is_closed());
        assert_eq!(shulker.get_color(), None);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn closing_uses_the_armor_attribute_and_rejects_direct_arrows() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let shulker = spawn(&world);
        let living = &shulker.mob_entity.living_entity;
        living.entity.velocity.store(Vector3::new(1.0, 2.0, 3.0));
        living.entity.push_by(Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(living.entity.velocity.load(), Vector3::default());
        assert!(shulker.is_collidable(None));
        living.update_attribute(&Attributes::ARMOR, |attribute| {
            attribute.base_value = 4.0;
            attribute.mark_dirty();
        });
        shulker.set_raw_peek(0);
        shulker.set_raw_peek(0);
        assert_eq!(living.get_attribute_value(&Attributes::ARMOR), 24.0);
        let arrow = Entity::new(
            world.clone(),
            Vector3::default(),
            &EntityType::SPECTRAL_ARROW,
        );
        assert!(!shulker.pre_damage(DamageType::MOB_PROJECTILE, Some(&arrow)));
        assert!(shulker.pre_damage(DamageType::ARROW, None));
        shulker.set_raw_peek(100);
        assert_eq!(living.get_attribute_value(&Attributes::ARMOR), 4.0);
        assert!(shulker.pre_damage(DamageType::ARROW, Some(&arrow)));
        shulker.mob_entity.set_no_ai(true);
        assert!(!shulker.teleport_somewhere());
        shulker.mob_entity.set_no_ai(false);
        living.set_health(0.0);
        assert!(!shulker.is_collidable(None));
        assert!(!shulker.teleport_somewhere());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn attachment_requires_support_and_room_for_the_whole_open_shell() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let shulker = spawn(&world);
        let pos = shulker.get_entity().block_pos.load();
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        assert!(!shulker.can_stay_at(&pos, BlockDirection::Down));
        world.set_block_state(&pos.down(), Block::STONE.default_state.id, flags);
        assert!(shulker.can_stay_at(&pos, BlockDirection::Down));
        world.set_block_state(&pos.up(), Block::STONE.default_state.id, flags);
        assert!(!shulker.can_stay_at(&pos, BlockDirection::Down));
        world.set_block_state(&pos.up(), Block::AIR.default_state.id, flags);
        assert!(shulker.can_stay_at(&pos, BlockDirection::Down));
        world.set_block_state(&pos.down(), Block::AIR.default_state.id, flags);
        assert_eq!(shulker.find_attachable_face(&pos), None);
        world.set_block_state(&pos.down(), Block::POWDER_SNOW.default_state.id, flags);
        assert!(!shulker.can_stay_at(&pos, BlockDirection::Down));
        shulker
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(
                &pumpkin_data::data_component_impl::EquipmentSlot::FEET,
                pumpkin_data::item_stack::ItemStack::new(
                    1,
                    &pumpkin_data::item::Item::LEATHER_BOOTS,
                ),
            );
        assert!(shulker.can_stay_at(&pos, BlockDirection::Down));
        world.set_block_state(&pos.down(), Block::AIR.default_state.id, flags);
        world.set_block_state(
            &pos.offset(BlockDirection::East.to_offset()),
            Block::STONE.default_state.id,
            flags,
        );
        shulker.post_tick();
        assert_eq!(shulker.get_attach_face(), BlockDirection::East);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn opening_shell_displaces_entities_without_adding_momentum() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let shulker = spawn(&world);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 65.0, 0.5),
            &EntityType::COW,
        ));
        world.spawn_entity(cow.clone());
        let velocity = Vector3::new(0.2, 0.0, 0.0);
        cow.get_entity().velocity.store(velocity);
        shulker.set_raw_peek(100);
        assert!(shulker.update_peek_amount());
        shulker.on_peek_amount_change();
        assert!(cow.get_entity().pos.load().y > 65.0);
        assert_eq!(cow.get_entity().velocity.load(), velocity);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn mounted_peek_and_attachment_changes_keep_the_seat_position() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let shulker = spawn(&world);
        let entity = shulker.get_entity();
        let vehicle = Arc::new(Entity::new(
            world.clone(),
            entity.pos.load(),
            &EntityType::COW,
        ));
        world.spawn_entity(vehicle.clone());
        vehicle.add_passenger_silently(vehicle.clone(), shulker.clone());
        entity.first_tick.store(false, Ordering::Relaxed);
        let seat = entity.pos.load();
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            seat.add_raw(0.0, 1.001, 0.0),
            &EntityType::COW,
        ));
        world.spawn_entity(cow.clone());
        shulker.set_raw_peek(100);
        assert!(shulker.update_peek_amount());
        shulker.on_peek_amount_change();
        assert_eq!(entity.pos.load(), seat);
        assert!(cow.get_entity().pos.load().y > seat.y + 1.001);
        for _ in 0..20 {
            shulker.update_peek_amount();
        }
        shulker.on_peek_amount_change();
        shulker.set_attach_face(BlockDirection::East);
        assert_eq!(entity.entity_dimension.load().height, 1.0);
        shulker.set_attach_face(BlockDirection::Down);
        assert_eq!(entity.entity_dimension.load().height, 2.0);
        assert_eq!(entity.get_eye_height(), 1.0);
        assert_eq!(entity.pos.load(), seat);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn teleport_faces_require_legal_support_and_loaded_clearance() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let shulker = spawn(&world);
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        let candidate = BlockPos::new(2, 64, 2);
        world.set_block_state(&candidate.down(), Block::BEDROCK.default_state.id, flags);
        world.set_block_state(
            &candidate.offset(BlockDirection::East.to_offset()),
            Block::STONE.default_state.id,
            flags,
        );
        assert_eq!(
            shulker.find_attachable_face(&candidate),
            Some(BlockDirection::Down)
        );
        assert_eq!(
            shulker.find_teleport_attachment(&candidate),
            Some(BlockDirection::East)
        );
        world.set_block_state(&candidate, Block::STONE.default_state.id, flags);
        assert_eq!(shulker.find_teleport_attachment(&candidate), None);

        let unloaded = BlockPos::new(16, 64, 2);
        world.set_block_state(
            &BlockPos::new(15, 64, 2),
            Block::STONE.default_state.id,
            flags,
        );
        assert!(shulker.can_stay_at(&unloaded, BlockDirection::West));
        assert_eq!(shulker.find_teleport_attachment(&unloaded), None);
        let edge = BlockPos::new(15, 64, 2);
        world.set_block_state(&edge, Block::AIR.default_state.id, flags);
        world.set_block_state(
            &BlockPos::new(14, 64, 2),
            Block::STONE.default_state.id,
            flags,
        );
        assert!(shulker.can_stay_at(&edge, BlockDirection::West));
        assert_eq!(shulker.find_teleport_attachment(&edge), None);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(1, 0), ChunkData::empty_sync(0, 0));
        assert_eq!(
            shulker.find_teleport_attachment(&edge),
            Some(BlockDirection::West)
        );
        world.set_block_state(
            &BlockPos::new(16, 64, 2),
            Block::STONE.default_state.id,
            flags,
        );
        assert_eq!(shulker.find_teleport_attachment(&edge), None);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn dismount_clearance_uses_the_snapped_shell() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let shulker = spawn(&world);
        world.set_block_state(
            &BlockPos::new(2, 64, 2),
            Block::DAYLIGHT_DETECTOR.default_state.id,
            BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE,
        );
        let (position, bounds) = shulker.get_entity().dismount_candidate(
            Vector3::new(2.5, 64.375, 2.5),
            pumpkin_data::entity::EntityPose::Standing,
        );
        assert_eq!(position.y, 64.0);
        assert!(!world.is_space_empty(bounds));
        let (position, bounds) = shulker.get_entity().dismount_candidate(
            Vector3::new(2.5, 65.0, 2.5),
            pumpkin_data::entity::EntityPose::Standing,
        );
        assert_eq!(position.y, 65.0);
        assert!(world.is_space_empty(bounds));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn invalidated_teleport_keeps_the_rider_linked_before_unlinking() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let shulker = spawn(&world);
        let vehicle = Arc::new(Entity::new(
            world.clone(),
            shulker.get_entity().pos.load(),
            &EntityType::COW,
        ));
        world.spawn_entity(vehicle.clone());
        vehicle.add_passenger_silently(vehicle.clone(), shulker.clone());
        let candidate = BlockPos::new(2, 64, 2);
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        world.set_block_state(&candidate.down(), Block::STONE.default_state.id, flags);
        assert!(shulker.find_teleport_attachment(&candidate).is_some());
        vehicle.remove_passenger_before_teleport_if(shulker.get_entity().entity_id, &|| {
            world.set_block_state(&candidate, Block::STONE.default_state.id, flags);
            shulker.find_teleport_attachment(&candidate).is_some()
        });
        assert!(shulker.get_entity().has_vehicle());
        assert_eq!(
            vehicle
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len(),
            1
        );
        world.set_block_state(&candidate, Block::AIR.default_state.id, flags);
        vehicle.remove_passenger_before_teleport_if(shulker.get_entity().entity_id, &|| {
            shulker.find_teleport_attachment(&candidate).is_some()
        });
        assert!(!shulker.get_entity().has_vehicle());
        assert!(
            vehicle
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn look_control_turns_in_each_attachment_plane() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let shulker = spawn(&world);
        for (face, delta, expected_yaw) in [
            (BlockDirection::Down, Vector3::new(1.0, 0.0, 0.0), -90.0),
            (BlockDirection::Up, Vector3::new(1.0, 0.0, 0.0), -90.0),
            (BlockDirection::North, Vector3::new(1.0, 0.0, 0.0), -90.0),
            (BlockDirection::South, Vector3::new(1.0, 0.0, 0.0), 90.0),
            (BlockDirection::West, Vector3::new(0.0, 0.0, 1.0), 90.0),
            (BlockDirection::East, Vector3::new(0.0, 0.0, 1.0), -90.0),
        ] {
            shulker.set_attach_face(face);
            shulker.get_entity().head_yaw.store(0.0);
            let position = shulker.get_entity().get_eye_pos() + delta;
            let mut look = shulker
                .mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            look.look_at_with_range(position.x, position.y, position.z, 180.0, 180.0);
            look.tick(shulker.as_ref());
            assert_eq!(shulker.get_entity().head_yaw.load(), expected_yaw);
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn attack_waits_twenty_ticks_and_drops_a_target_at_twenty_blocks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let shulker = spawn(&world);
        shulker.set_attach_face(BlockDirection::East);
        let target = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(10.5, 64.0, 0.5),
            &EntityType::COW,
        ));
        world.spawn_entity(target.clone());
        shulker.mob_entity.set_target(Some(target.clone()));
        let mut goal = ShulkerAttackGoal::new(Arc::downgrade(&shulker));
        assert!(goal.can_start(shulker.as_ref()));
        goal.start(shulker.as_ref());
        for _ in 0..19 {
            goal.tick(shulker.as_ref());
        }
        assert_eq!(world.entities.load().len(), 2);
        goal.tick(shulker.as_ref());
        assert_eq!(world.entities.load().len(), 3);
        let entities = world.entities.load();
        let bullet = entities
            .iter()
            .find_map(|entity| entity.cast_any().downcast_ref::<ShulkerBulletEntity>())
            .ok_or("attack did not spawn a shulker bullet")?;
        assert_eq!(
            bullet.target().map(|entity| entity.get_entity().entity_id),
            Some(target.get_entity().entity_id),
        );
        assert_eq!(shulker.get_raw_peek(), 100);
        target.get_entity().set_pos(Vector3::new(20.5, 64.0, 0.5));
        goal.tick(shulker.as_ref());
        assert!(shulker.mob_entity.get_target().is_none());
        goal.stop(shulker.as_ref());
        assert!(shulker.is_closed());
        world.level.shutdown().await;
        Ok(())
    }
}

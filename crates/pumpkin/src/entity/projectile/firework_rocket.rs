use crate::{
    entity::{
        Entity, EntityBase, decoration::armor_stand::ArmorStandEntity, projectile::ProjectileHit,
    },
    server::Server,
    world::World,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::FireworksImpl;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::optional_int::OptionalInt;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, RwLock};

const EXPLOSION_RADIUS: f64 = 5.0;

pub struct FireworkRocketEntity {
    entity: Entity,
    owner: AtomicCell<Option<uuid::Uuid>>,
    item: RwLock<ItemStack>,
    can_break: RwLock<Option<pumpkin_nbt::tag::NbtTag>>,
    life: AtomicI32,
    life_time: AtomicI32,
    attached_to: Option<i32>,
    shot_at_angle: AtomicBool,
    has_been_shot: AtomicBool,
    left_owner: AtomicBool,
    last_deflected_by: AtomicCell<Option<i32>>,
}

impl FireworkRocketEntity {
    pub fn new(entity: Entity, owner: Option<&Entity>, item: ItemStack) -> Self {
        let (thrown, life_time) = Self::launch(entity, &item);
        Self {
            entity: thrown,
            owner: AtomicCell::new(owner.map(|owner| owner.entity_uuid)),
            item: RwLock::new(item),
            can_break: RwLock::new(None),
            life: 0.into(),
            life_time: life_time.into(),
            attached_to: None,
            shot_at_angle: AtomicBool::new(false),
            has_been_shot: AtomicBool::new(false),
            left_owner: AtomicBool::new(false),
            last_deflected_by: AtomicCell::new(None),
        }
    }

    pub fn new_shot_at_angle(entity: Entity, item: ItemStack) -> Self {
        let rocket = Self::new(entity, None, item);
        rocket.shot_at_angle.store(true, Ordering::Relaxed);
        rocket
    }

    pub fn new_crossbow(entity: Entity, shooter: &Entity, item: ItemStack) -> Self {
        let rocket = Self::new(entity, Some(shooter), item);
        rocket.shot_at_angle.store(true, Ordering::Relaxed);
        rocket
    }

    pub fn shoot(&self, pitch: f32, yaw: f32, angle: f32, power: f32, uncertainty: f32) {
        let view = Entity::calculate_view_vector(pitch, yaw);
        let up = Entity::calculate_view_vector(pitch - 90.0, yaw);
        let angle = f64::from(angle.to_radians());
        let direction = view * angle.cos()
            + up.cross(&view) * angle.sin()
            + up * (up.dot(&view) * (1.0 - angle.cos()));
        let direction = direction.normalize();
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let spread = 0.0172275 * f64::from(uncertainty);
        let velocity = (direction
            + Vector3::new(
                random.next_triangular(0.0, spread),
                random.next_triangular(0.0, spread),
                random.next_triangular(0.0, spread),
            ))
            * f64::from(power);
        self.entity.velocity.store(velocity);
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z).to_degrees() as f32,
            velocity.y.atan2(velocity.horizontal_length()).to_degrees() as f32,
        );
    }

    pub fn new_attached(entity: Entity, item: ItemStack, stuck_to: &Entity) -> Self {
        entity.set_pos(stuck_to.pos.load());
        let mut rocket = Self::new(entity, Some(stuck_to), item);
        rocket.attached_to = Some(stuck_to.entity_id);
        rocket
    }

    pub fn get_item(&self) -> ItemStack {
        self.item
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn owner_entity(&self) -> Option<Arc<dyn EntityBase>> {
        self.owner.load().and_then(|id| {
            let world = self.entity.world.load();
            world
                .get_player_by_uuid(id)
                .map(|player| player as Arc<dyn EntityBase>)
                .or_else(|| world.get_entity_by_uuid(id))
        })
    }

    pub fn spawn(self, world: &Arc<World>) {
        let rocket = Arc::new(self);
        world.spawn_entity(rocket.clone());
        super::apply_on_projectile_spawned(rocket.get_entity(), &rocket.get_item(), None, None);
    }

    fn launch(entity: Entity, item: &ItemStack) -> (Entity, i32) {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        entity.velocity.store(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));

        let flight_count = 1 + item
            .get_data_component::<FireworksImpl>()
            .map_or(0, |fireworks| fireworks.flight_duration);
        let life_time = 10 * flight_count + random.next_bounded_i32(6) + random.next_bounded_i32(7);

        (entity, life_time)
    }

    fn explosion_count(&self) -> usize {
        self.get_item()
            .get_data_component::<FireworksImpl>()
            .map_or(0, |fireworks| fireworks.explosions.len())
    }

    fn root_vehicle_id(entity: &Entity) -> i32 {
        let mut id = entity.entity_id;
        let mut vehicle = entity
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        while let Some(current) = vehicle.take() {
            let entity = current.get_entity();
            id = entity.entity_id;
            vehicle.clone_from(
                &entity
                    .vehicle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }
        id
    }

    fn can_be_hit(entity: &dyn EntityBase) -> bool {
        let base = entity.get_entity();
        base.is_alive()
            && !entity.is_spectator()
            && entity
                .get_living_entity()
                .is_none_or(|living| living.health.load() > 0.0)
            && base.entity_type != &EntityType::INTERACTION
            && Self::is_pickable(entity)
    }

    fn is_pickable(entity: &dyn EntityBase) -> bool {
        let base = entity.get_entity();
        if !base.is_alive() {
            return false;
        }
        if let Some(stand) = entity.cast_any().downcast_ref::<ArmorStandEntity>() {
            return !stand.is_marker();
        }
        if base.entity_type == &EntityType::ENDER_DRAGON {
            return false;
        }
        base.entity_type == &EntityType::INTERACTION
            || entity.get_living_entity().is_some()
            || entity.can_hit()
            || base
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_REDIRECTABLE_PROJECTILE)
            || [
                &EntityType::TNT,
                &EntityType::FALLING_BLOCK,
                &EntityType::END_CRYSTAL,
                &EntityType::SHULKER_BULLET,
                &EntityType::ITEM_FRAME,
                &EntityType::GLOW_ITEM_FRAME,
                &EntityType::PAINTING,
                &EntityType::LEASH_KNOT,
            ]
            .contains(&base.entity_type)
    }

    fn check_left_owner(&self, world: &World) {
        if self.left_owner.load(Ordering::Relaxed) {
            return;
        }
        let owner = self.owner_entity();
        let Some(owner) = owner else {
            self.left_owner.store(true, Ordering::Relaxed);
            return;
        };
        let root_id = Self::root_vehicle_id(owner.get_entity());
        let entity = self.get_entity();
        let bounds = entity
            .bounding_box
            .load()
            .stretch(entity.velocity.load())
            .expand(1.0, 1.0, 1.0);
        let overlaps_owner = world.get_all_at_box(&bounds).iter().any(|target| {
            Self::is_pickable(target.as_ref())
                && Self::root_vehicle_id(target.get_entity()) == root_id
        });
        if !overlaps_owner {
            self.left_owner.store(true, Ordering::Relaxed);
        }
    }

    #[expect(clippy::too_many_lines)]
    fn get_hit_result(&self, world: &World) -> Option<(ProjectileHit, bool)> {
        let entity = self.get_entity();
        let start = entity.pos.load();
        let movement = entity.velocity.load();
        let end = start + movement;
        let normal = movement.normalize() * -1.0;
        let mut block_hit = world.ray_trace_collision(start, end, self);
        let mut border_hit = false;
        {
            let border = world
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let to = block_hit.map_or(end, |(_, _, pos)| pos);
            if border.contains(start.x, start.z) && !border.contains(to.x, to.z) {
                let half = border.new_diameter / 2.0;
                let limit = f64::from(border.portal_teleport_boundary);
                let hit_pos = Vector3::new(
                    to.x.clamp(
                        (border.center_x - half).max(-limit),
                        (border.center_x + half).min(limit) - f64::from(1.0e-5f32),
                    ),
                    to.y,
                    to.z.clamp(
                        (border.center_z - half).max(-limit),
                        (border.center_z + half).min(limit) - f64::from(1.0e-5f32),
                    ),
                );
                let delta = to - start;
                let face = if delta.y.abs() > delta.x.abs() && delta.y.abs() > delta.z.abs() {
                    if delta.y > 0.0 {
                        pumpkin_data::BlockDirection::Up
                    } else {
                        pumpkin_data::BlockDirection::Down
                    }
                } else if delta.x.abs() > delta.z.abs() {
                    if delta.x > 0.0 {
                        pumpkin_data::BlockDirection::East
                    } else {
                        pumpkin_data::BlockDirection::West
                    }
                } else if delta.z > 0.0 {
                    pumpkin_data::BlockDirection::South
                } else {
                    pumpkin_data::BlockDirection::North
                };
                block_hit = Some((
                    pumpkin_util::math::position::BlockPos::floored_v(hit_pos),
                    face,
                    hit_pos,
                ));
                border_hit = true;
            }
        }
        let to = block_hit.map_or(end, |(_, _, pos)| pos);
        let mut hit = block_hit.map(|(pos, face, hit_pos)| ProjectileHit::Block {
            pos,
            face,
            hit_pos,
            normal,
        });
        let mut nearest = f64::INFINITY;
        let bounds = entity
            .bounding_box
            .load()
            .stretch(movement)
            .expand(1.0, 1.0, 1.0);
        let margin = ((entity.age.load(Ordering::Relaxed) as f32 - 2.0) / 20.0).clamp(0.0, 0.3);
        let owner_root = if self.left_owner.load(Ordering::Relaxed) {
            None
        } else {
            self.owner_entity()
                .map(|owner| Self::root_vehicle_id(owner.get_entity()))
        };
        for target in world.get_all_at_box(&bounds) {
            let target_entity = target.get_entity();
            if target_entity.entity_id == entity.entity_id
                || Some(target_entity.entity_id) == self.attached_to
                || !Self::can_be_hit(target.as_ref())
                || owner_root.is_some_and(|id| Self::root_vehicle_id(target_entity) == id)
            {
                continue;
            }
            let target_box = target_entity
                .bounding_box
                .load()
                .expand_all(f64::from(margin));
            if let Some((distance, _, hit_pos)) =
                World::intersects_aabb_with_hit(start, to, target_box.min, target_box.max)
                && distance > 0.0
                && distance < 1.0
                && distance < nearest
            {
                nearest = distance;
                border_hit = false;
                hit = Some(ProjectileHit::Entity {
                    entity: target,
                    hit_pos,
                    normal,
                });
            }
        }
        hit.map(|hit| (hit, border_hit))
    }

    fn hit_target_or_deflect(&self, hit: ProjectileHit, border_hit: bool) {
        if !border_hit
            && let ProjectileHit::Block { pos, .. } = &hit
            && self.get_entity().world.load().get_block_state(pos).is_air()
        {
            return;
        }
        if let ProjectileHit::Entity { entity: target, .. } = &hit {
            let target_entity = target.get_entity();
            if target_entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_DEFLECTS_PROJECTILES)
            {
                if self.last_deflected_by.load() != Some(target_entity.entity_id) {
                    let entity = self.get_entity();
                    let rotation = rand::random::<f32>().mul_add(20.0, 170.0);
                    entity.velocity.store(entity.velocity.load() * -0.5);
                    entity.velocity_dirty.store(true, Ordering::Relaxed);
                    entity.set_rotation(entity.yaw.load() + rotation, entity.pitch.load());
                    self.last_deflected_by.store(Some(target_entity.entity_id));
                    if target_entity.entity_type == &EntityType::BREEZE {
                        target_entity.world.load().play_sound_fine(
                            Sound::EntityBreezeDeflect,
                            SoundCategory::Hostile,
                            &target_entity.pos.load(),
                            1.0,
                            1.0,
                        );
                    }
                }
                return;
            }
        }
        self.on_hit(hit);
    }

    fn update_rotation(&self) {
        let entity = self.get_entity();
        let movement = entity.velocity.load();
        let target_yaw = movement.x.atan2(movement.z).to_degrees() as f32;
        let target_pitch = movement.y.atan2(movement.horizontal_length()).to_degrees() as f32;
        let yaw = entity.yaw.load();
        let pitch = entity.pitch.load();
        entity.set_rotation(
            yaw + ((target_yaw - yaw + 180.0).rem_euclid(360.0) - 180.0) * 0.2,
            pitch + ((target_pitch - pitch + 180.0).rem_euclid(360.0) - 180.0) * 0.2,
        );
    }

    pub fn explode_and_remove(&self, caller: &dyn EntityBase, world: &Arc<World>) {
        let entity = self.get_entity();
        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::firework_explode::FireworkExplodeEvent {
                    entity_id: entity.entity_id,
                    cancelled: false,
                };
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        world.send_entity_status(
            entity,
            EntityStatus::FireworksExplode,
            Some(ActorEventID::FireworksExplode),
        );
        world.emit_game_event(GameEvent::Explode.name(), entity.pos.load());
        self.deal_explosion_damage(caller, world);

        entity.remove();
    }

    fn deal_explosion_damage(&self, caller: &dyn EntityBase, world: &Arc<World>) {
        let explosions = self.explosion_count();
        if explosions == 0 {
            return;
        }
        let damage_amount = 5.0 + (explosions * 2) as f32;

        let entity = self.get_entity();
        let rocket_pos = entity.pos.load();
        let owner = self.owner_entity();
        let owner_source = owner
            .as_ref()
            .map(|owner| owner.as_ref() as &dyn EntityBase);

        if let Some(attached_id) = self.attached_to
            && let Some(attached) = world.get_entity_by_id(attached_id)
            && attached.get_living_entity().is_some()
        {
            attached.damage_with_context(
                attached.as_ref(),
                damage_amount,
                DamageType::FIREWORKS,
                Some(rocket_pos),
                Some(caller),
                owner_source,
            );
        }

        let search_box =
            entity
                .bounding_box
                .load()
                .expand(EXPLOSION_RADIUS, EXPLOSION_RADIUS, EXPLOSION_RADIUS);
        for target in world.get_all_at_box(&search_box) {
            if target.get_living_entity().is_none() {
                continue;
            }
            let target_entity = target.get_entity();
            if Some(target_entity.entity_id) == self.attached_to {
                continue;
            }
            let target_pos = target_entity.pos.load();
            let squared_distance = rocket_pos.squared_distance_to_vec(&target_pos);
            if squared_distance > EXPLOSION_RADIUS * EXPLOSION_RADIUS {
                continue;
            }

            let target_box = target_entity.bounding_box.load();
            let height = target_box.max.y - target_box.min.y;
            let can_see = (0..2).any(|step| {
                let to = Vector3::new(
                    target_pos.x,
                    height.mul_add(0.5 * f64::from(step), target_pos.y),
                    target_pos.z,
                );
                world.ray_trace_collision(rocket_pos, to, caller).is_none()
            });
            if !can_see {
                continue;
            }

            let delta = rocket_pos - target_pos;
            let (x, y, z) = (delta.x as f32, delta.y as f32, delta.z as f32);
            let distance = (x * x + y * y + z * z).sqrt();
            let damage = damage_amount
                * ((EXPLOSION_RADIUS - f64::from(distance)) / EXPLOSION_RADIUS).sqrt() as f32;
            target.damage_with_context(
                target.as_ref(),
                damage,
                DamageType::FIREWORKS,
                Some(rocket_pos),
                Some(caller),
                owner_source,
            );
        }
    }

    fn boost_attached(&self, world: &Arc<World>) {
        let Some(attached_id) = self.attached_to else {
            return;
        };
        let Some(attached) = world.get_entity_by_id(attached_id) else {
            return;
        };
        if attached.get_living_entity().is_none() {
            return;
        }
        let attached_entity = attached.get_entity();
        let entity = self.get_entity();
        let attached_player = world.get_player_by_id(attached_id);

        let hand_angle = if attached_entity.is_fall_flying() {
            let look = Entity::calculate_view_vector(
                attached_entity.pitch.load(),
                attached_entity.yaw.load(),
            );
            let movement = attached_entity.velocity.load();
            attached_entity.velocity.store(Vector3::new(
                movement.x + look.x.mul_add(0.1, (look.x * 1.5 - movement.x) * 0.5),
                movement.y + look.y.mul_add(0.1, (look.y * 1.5 - movement.y) * 0.5),
                movement.z + look.z.mul_add(0.1, (look.z * 1.5 - movement.z) * 0.5),
            ));

            attached_player.as_ref().map_or_else(
                || Vector3::new(0.0, 0.0, 0.0),
                |player| player.get_hand_holding_item_angle(&Item::FIREWORK_ROCKET),
            )
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };

        entity.set_pos(attached_entity.pos.load().add(&hand_angle));
        entity.velocity.store(attached_entity.velocity.load());
    }
}

impl EntityBase for FireworkRocketEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.owner_entity()
            .map(|owner| owner.get_entity().entity_id)
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Life", self.life.load(Ordering::Relaxed));
        nbt.put_int("LifeTime", self.life_time.load(Ordering::Relaxed));
        nbt.put_bool("ShotAtAngle", self.shot_at_angle.load(Ordering::Relaxed));
        nbt.put_bool("LeftOwner", self.left_owner.load(Ordering::Relaxed));
        nbt.put_bool("HasBeenShot", self.has_been_shot.load(Ordering::Relaxed));
        if let Some(owner) = self.owner.load() {
            nbt.put_uuid("Owner", owner);
        }
        if let Some(predicate) = self
            .can_break
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            nbt.put("can_break", predicate.clone());
        }
        let mut item = NbtCompound::new();
        self.get_item().write_item_stack(&mut item);
        nbt.put_compound("FireworksItem", item);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.life
            .store(nbt.get_int("Life").unwrap_or(0), Ordering::Relaxed);
        self.life_time
            .store(nbt.get_int("LifeTime").unwrap_or(0), Ordering::Relaxed);
        self.shot_at_angle.store(
            nbt.get_bool("ShotAtAngle").unwrap_or(false),
            Ordering::Relaxed,
        );
        self.left_owner.store(
            nbt.get_bool("LeftOwner").unwrap_or(false),
            Ordering::Relaxed,
        );
        self.has_been_shot.store(
            nbt.get_bool("HasBeenShot").unwrap_or(false),
            Ordering::Relaxed,
        );
        self.owner.store(nbt.get_uuid("Owner"));
        *self
            .can_break
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = nbt.get("can_break").cloned();
        let item = nbt
            .get_compound("FireworksItem")
            .and_then(ItemStack::read_item_stack)
            .unwrap_or_else(|| ItemStack::new(1, &Item::FIREWORK_ROCKET));
        *self
            .item
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = item;
        self.init_data_tracker();
    }

    fn init_data_tracker(&self) {
        self.entity
            .data
            .store(self.get_owner_id().unwrap_or(0), Ordering::Relaxed);
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::ID_FIREWORKS_ITEM,
            ItemStackSerializer::from(self.get_item()),
        );
        if let Some(attached_to) = self.attached_to {
            self.entity.set_synced_data(
                pumpkin_data::tracked_data::firework_rocket::ATTACHED_TO_TARGET,
                OptionalInt(Some(attached_to)),
            );
        }
        if self.shot_at_angle.load(Ordering::Relaxed) {
            self.entity.set_synced_data(
                pumpkin_data::tracked_data::firework_rocket::SHOT_AT_ANGLE,
                true,
            );
        }
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let entity = self.get_entity();
        if !self.has_been_shot.swap(true, Ordering::Relaxed) {
            entity
                .world
                .load()
                .emit_game_event(GameEvent::ProjectileShoot.name(), entity.pos.load());
        }
        self.check_left_owner(&entity.world.load());
        entity.tick(caller, server);
        let world = entity.world.load();

        let hit = if self.attached_to.is_some() {
            self.boost_attached(&world);
            self.get_hit_result(&world)
        } else {
            if !self.shot_at_angle.load(Ordering::Relaxed) {
                let mut velocity = entity.velocity.load();
                let acceleration = if entity.horizontal_collision.load(Ordering::Relaxed) {
                    1.0
                } else {
                    1.15
                };
                velocity.x *= acceleration;
                velocity.z *= acceleration;
                velocity.y += 0.04;
                entity.velocity.store(velocity);
            }
            let movement = entity.velocity.load();
            let hit = self.get_hit_result(&world);
            entity.move_entity(caller, movement);
            entity.tick_block_collisions(caller);
            entity.velocity.store(movement);
            hit
        };
        if !entity.no_physics.load(Ordering::Relaxed)
            && entity.is_alive()
            && let Some((hit, border_hit)) = hit
        {
            self.hit_target_or_deflect(hit, border_hit);
            entity.velocity_dirty.store(true, Ordering::Relaxed);
        }
        self.update_rotation();

        let life = self.life.fetch_add(1, Ordering::Relaxed);
        if life == 0 && !entity.is_silent() {
            world.play_sound_fine(
                Sound::EntityFireworkRocketLaunch,
                SoundCategory::Ambient,
                &entity.pos.load(),
                3.0,
                1.0,
            );
        }

        if life + 1 > self.life_time.load(Ordering::Relaxed) {
            self.explode_and_remove(caller, &world);
        }
    }

    fn get_entity(&self) -> &crate::entity::Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = self.get_entity();
        let world = entity.world.load();
        match hit {
            ProjectileHit::Entity {
                entity: target,
                hit_pos,
                ..
            } => {
                let owner = self.owner_entity();
                if let Some(fireball) = target
                    .cast_any()
                    .downcast_ref::<super::fireball::FireballEntity>()
                {
                    fireball.redirect(owner.as_deref());
                } else if let Some(wind_charge) = target
                    .cast_any()
                    .downcast_ref::<super::wind_charge::WindChargeEntity>(
                ) {
                    wind_charge.redirect(owner.as_deref());
                }
                self.explode_and_remove(self, &world);
                world.emit_game_event(GameEvent::ProjectileLand.name(), hit_pos);
            }
            ProjectileHit::Block { pos, hit_pos, .. } => {
                let (block, state) = world.get_block_and_state(&pos);
                if let Some(server) = world.server.upgrade() {
                    world
                        .block_registry
                        .on_entity_collision(block, &world, self, &pos, state, &server);
                }
                if self.explosion_count() > 0 {
                    self.explode_and_remove(self, &world);
                }
                if let Some(server) = world.server.upgrade() {
                    world
                        .block_registry
                        .on_projectile_hit(block, &world, self, &pos, state, &hit_pos, &server);
                }
                world.emit_game_event(GameEvent::ProjectileLand.name(), pos.to_centered_f64());
            }
        }
    }
}

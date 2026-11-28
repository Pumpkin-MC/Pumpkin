mod dismount;
mod physics;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::atomic::AtomicCell;

use crate::entity::player::Player;
use crate::entity::vehicle::minecart::container::{self, MinecartInventory};
use crate::entity::{
    Entity, EntityBase,
    living::{LivingEntity, is_allowed_by_team_rules},
};
use crate::net::ClientPlatform;
use crate::plugin::api::events::vehicle::vehicle_move::VehicleMoveEvent;
use crate::server::Server;
use crate::world::game_event::GameEventContext;

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;

use crate::entity::vehicle::vehicle::VehicleEntity;
use physics::BoatStatus;

pub(crate) const PICKUP_RANGE: f64 = 0.2f32 as f64;
const PICKUP_Y_INSET: f64 = 0.01f32 as f64;

const FRONT_SEAT_OFFSET: f32 = 0.2;
const BACK_SEAT_OFFSET: f32 = -0.6;
const ANIMAL_SEAT_SHIFT: f32 = 0.2;
const CHEST_BOAT_CONTAINER_SIZE: usize = 27;
const TIME_TO_EJECT: f32 = 60.0;
const BUBBLE_TIME: u8 = 60;
const BUBBLE_COLUMN_LAUNCH_SPEED: f64 = 0.6;
const BUBBLE_COLUMN_SINK_SPEED: f64 = 0.7;

#[derive(Clone, Copy)]
enum BubbleColumnDirection {
    Up,
    Down,
}

#[must_use]
pub const fn is_chest_boat(entity_type: &EntityType) -> bool {
    entity_type.id == EntityType::ACACIA_CHEST_BOAT.id
        || entity_type.id == EntityType::BAMBOO_CHEST_RAFT.id
        || entity_type.id == EntityType::BIRCH_CHEST_BOAT.id
        || entity_type.id == EntityType::CHERRY_CHEST_BOAT.id
        || entity_type.id == EntityType::DARK_OAK_CHEST_BOAT.id
        || entity_type.id == EntityType::JUNGLE_CHEST_BOAT.id
        || entity_type.id == EntityType::MANGROVE_CHEST_BOAT.id
        || entity_type.id == EntityType::OAK_CHEST_BOAT.id
        || entity_type.id == EntityType::PALE_OAK_CHEST_BOAT.id
        || entity_type.id == EntityType::POPLAR_CHEST_BOAT.id
        || entity_type.id == EntityType::SPRUCE_CHEST_BOAT.id
}

pub struct BoatEntity {
    pub vehicle: VehicleEntity,
    ticks_underwater: AtomicCell<f32>,
    status: AtomicCell<Option<BoatStatus>>,
    water_level: AtomicCell<f64>,
    land_friction: AtomicCell<f32>,
    last_yd: AtomicCell<f64>,
    surface_bubble_contact: AtomicCell<Option<BubbleColumnDirection>>,
    bubble_time: AtomicCell<u8>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
    inventory: Option<Arc<MinecartInventory>>,
}

fn uses_animal_boat_seat(passenger: &dyn EntityBase) -> bool {
    // Java's Hoglin extends Animal. Pumpkin exposes only its ageable capability.
    passenger.get_entity().entity_type == &EntityType::HOGLIN
        || passenger
            .get_mob()
            .and_then(|mob| mob.as_animal())
            .is_some()
}

impl BoatEntity {
    #[must_use]
    pub fn new(mut entity: Entity) -> Self {
        entity.blocks_building = true;
        let inventory = is_chest_boat(entity.entity_type)
            .then(|| Arc::new(MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE)));
        Self {
            vehicle: VehicleEntity::new(entity),
            ticks_underwater: AtomicCell::new(0.0),
            status: AtomicCell::new(None),
            water_level: AtomicCell::new(0.0),
            land_friction: AtomicCell::new(0.0),
            last_yd: AtomicCell::new(0.0),
            surface_bubble_contact: AtomicCell::new(None),
            bubble_time: AtomicCell::new(0),
            left_paddle_moving: AtomicBool::new(false),
            right_paddle_moving: AtomicBool::new(false),
            inventory,
        }
    }

    const fn max_passengers(&self) -> usize {
        if self.inventory.is_some() { 1 } else { 2 }
    }

    pub(crate) fn has_enough_space_for(&self, passenger: &Entity) -> bool {
        passenger.width() < self.vehicle.entity.width()
    }

    fn is_client_controlled(&self) -> bool {
        self.get_controlling_passenger()
            .and_then(|passenger| {
                passenger
                    .get_player()
                    .map(|player| matches!(&*player.client, ClientPlatform::Java(_)))
            })
            .unwrap_or(false)
    }

    fn tick_underwater(&self, status: BoatStatus) {
        if !status.is_underwater() {
            self.ticks_underwater.store(0.0);
            return;
        }
        let ticks = (self.ticks_underwater.load() + 1.0).min(TIME_TO_EJECT);
        self.ticks_underwater.store(ticks);
        if ticks >= TIME_TO_EJECT {
            self.eject_passengers();
        }
    }

    fn eject_passengers(&self) {
        self.get_entity().eject_vehicle_passengers();
    }

    fn tick_bubble_column(&self) {
        let entity = self.get_entity();
        let previous = self.bubble_time.load();
        let contact = self.surface_bubble_contact.swap(None);
        let remaining = match contact {
            None => 0,
            Some(_) if previous == 0 => BUBBLE_TIME - 1,
            Some(_) => previous - 1,
        };
        if remaining == previous {
            return;
        }
        self.bubble_time.store(remaining);
        entity.set_synced_data(
            pumpkin_data::tracked_data::abstract_boat::ID_BUBBLE_TIME,
            VarInt(i32::from(remaining)),
        );
        if let Some(direction) = contact.filter(|_| remaining == 0) {
            let world = entity.world.load_full();
            let position = entity.pos.load();
            let epoch = entity.vehicle_motion_epoch();
            let still_current = || {
                !entity.is_removed()
                    && entity.pos.load() == position
                    && entity.vehicle_motion_epoch() == epoch
                    && Arc::ptr_eq(&world, &entity.world.load())
            };
            let event = match direction {
                BubbleColumnDirection::Up => EntityStatus::BoatLaunch,
                BubbleColumnDirection::Down => {
                    self.eject_passengers();
                    EntityStatus::BoatSink
                }
            };
            if !still_current() {
                return;
            }
            world.broadcast_entity_event(entity, event, ActorEventID::None);
            if still_current() && !self.is_client_controlled() {
                let mut velocity = entity.velocity.load();
                match direction {
                    BubbleColumnDirection::Up => velocity.y = BUBBLE_COLUMN_LAUNCH_SPEED,
                    BubbleColumnDirection::Down => velocity.y -= BUBBLE_COLUMN_SINK_SPEED,
                }
                entity.velocity.store(velocity);
            }
        }
    }

    fn move_from_physics(
        &self,
        caller: &dyn EntityBase,
        surface_snap: Option<Vector3<f64>>,
    ) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let from = entity.pos.load();
        let epoch = entity.vehicle_motion_epoch();
        let start = from + surface_snap.unwrap_or_default();
        let mut motion = entity.velocity.load();
        if surface_snap.is_some() {
            motion.y = 0.0;
        }
        let requested = start + motion;
        if requested != from
            && let Some(server) = world.server.upgrade()
            && server.plugin_manager.has_handlers::<VehicleMoveEvent>()
        {
            let event = VehicleMoveEvent::new(entity.entity_id, from, requested);
            let event = server.plugin_manager.fire_blocking(&server, event);
            if entity.is_removed()
                || entity.pos.load() != from
                || entity.vehicle_motion_epoch() != epoch
                || !Arc::ptr_eq(&world, &entity.world.load())
                || self.is_client_controlled()
            {
                return false;
            }
            if event.cancelled
                || !event.to.x.is_finite()
                || !event.to.y.is_finite()
                || !event.to.z.is_finite()
            {
                entity.velocity.store(Vector3::default());
                return true;
            }
            if event.to != requested {
                motion = event.to - start;
            }
            if let Some(offset) = surface_snap
                && !self.retains_water_entry_offset(offset)
            {
                // Retry the uncommitted water entry using the live surface on the next tick.
                self.status.store(Some(BoatStatus::InAir));
                return true;
            }
        }
        if surface_snap.is_some() {
            entity
                .velocity
                .store(entity.velocity.load().multiply(1.0, 0.0, 1.0));
            self.last_yd.store(0.0);
            entity.set_pos(start);
        }
        entity.move_entity(caller, motion)
    }

    pub(crate) fn additional_vertical_tick_movement(&self) -> f64 {
        // A fall can leave the first air-to-water surface search above the current hull.
        self.last_yd.load().abs() + 2.0
    }

    pub fn open_custom_inventory_screen(&self, player: &Arc<Player>) -> bool {
        let Some(inventory) = &self.inventory else {
            return false;
        };
        let entity_type = self.vehicle.entity.entity_type;
        let bedrock_key = if entity_type.id == EntityType::BAMBOO_CHEST_RAFT.id {
            translation::bedrock::ENTITY_CHEST_RAFT_NAME
        } else {
            translation::bedrock::ENTITY_CHEST_BOAT_NAME
        };
        let title = TextComponent::translate_cross(
            format!("entity.minecraft.{}", entity_type.resource_name),
            bedrock_key,
            [],
        );
        let opened = container::open(&self.vehicle.entity, player, inventory, title, false);
        if opened {
            let world = self.vehicle.entity.world.load();
            world.emit_game_event_with_context(
                GameEvent::ContainerOpen.name(),
                self.vehicle.entity.pos.load(),
                &GameEventContext {
                    source_entity: Some(player.as_ref()),
                },
            );
        }
        opened
    }

    pub fn set_paddles(&self, left: bool, right: bool) {
        self.left_paddle_moving.store(left, Ordering::Relaxed);
        self.right_paddle_moving.store(right, Ordering::Relaxed);

        self.vehicle.entity.send_meta_data(
            &[
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_LEFT, left),
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_RIGHT, right),
            ],
            None,
        );
    }

    fn tick_contacts(&self, world: &crate::world::World, still_current: impl Fn() -> bool) {
        let entity = self.get_entity();
        // Vanilla AbstractBoat.tick checks boarding even for a stationary, empty boat.
        let bounds = entity
            .bounding_box
            .load()
            .expand(PICKUP_RANGE, -PICKUP_Y_INSET, PICKUP_RANGE);
        let add_new_passengers = self.get_controlling_passenger().is_none();
        let own_team = self.get_team();
        for other in world.get_all_at_box(&bounds) {
            if !still_current() {
                return;
            }
            let passenger = other.get_entity();
            if passenger.entity_id == entity.entity_id
                || other.is_spectator()
                || passenger.is_removed()
                || !world.is_pushable_by_vehicle(other.as_ref())
                || !is_allowed_by_team_rules(own_team.as_ref(), other.get_team().as_ref())
                || other.has_passenger(self)
            {
                continue;
            }

            let has_room = entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len()
                < self.max_passengers();
            if add_new_passengers
                && has_room
                && !other.is_passenger()
                && self.has_enough_space_for(passenger)
                && other.get_living_entity().is_some()
                && !passenger
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_CANNOT_BE_PUSHED_ONTO_BOATS)
            {
                // The remaining Entity.startRiding checks may reject boarding without a push.
                if !other.can_ride(self)
                    || entity.is_submerged_in_water()
                    || self.is_passenger_of_same_vehicle(other.as_ref())
                {
                    // Shulker.startRiding resets its attachment even when vanilla checks fail.
                    if let Some(mob) = other.get_mob() {
                        mob.on_start_riding();
                    }
                    continue;
                }
                let Some(vehicle) = world.get_entity_by_id(entity.entity_id) else {
                    continue;
                };
                entity.board_vehicle_contact(
                    &vehicle,
                    &other,
                    self.max_passengers(),
                    &still_current,
                    || {
                        bounds.intersects(&passenger.bounding_box.load())
                            && self.has_enough_space_for(passenger)
                            && !entity.is_submerged_in_water()
                            && self.get_controlling_passenger().is_none()
                    },
                );
            } else {
                self.push_contact(other.as_ref(), Entity::push_by);
            }
        }
    }

    fn send_wobble_metadata(&self) {
        self.vehicle.send_wobble_metadata();
    }
}

impl EntityBase for BoatEntity {
    fn vehicle_contact_tick_reach(&self) -> Option<f64> {
        Some(PICKUP_RANGE)
    }

    fn vehicle_requires_serial_tick(&self) -> bool {
        // An automatic exit can put a rider beyond the hull's planned contact area.
        (self.bubble_time.load() == 1 || self.ticks_underwater.load() >= TIME_TO_EJECT - 1.0)
            && self.get_entity().has_passengers()
    }

    fn client_vehicle_float_threshold(&self) -> Option<f64> {
        Some(-0.03125)
    }

    fn vehicle_dismount_position(
        &self,
        passenger: &dyn EntityBase,
        still_current: &dyn Fn() -> bool,
    ) -> Option<Vector3<f64>> {
        Self::get_dismount_location_for_passenger(self.get_entity(), passenger, still_current)
    }

    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        index: usize,
    ) -> Option<Vector3<f64>> {
        let entity = self.get_entity();
        let seats = entity.entity_type.boat?;
        let mut offset = seats.single_passenger_x_offset;
        let passenger_count = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if passenger_count > 1 {
            offset = if index == 0 {
                FRONT_SEAT_OFFSET
            } else {
                BACK_SEAT_OFFSET
            };
            if uses_animal_boat_seat(passenger) {
                offset += ANIMAL_SEAT_SHIFT;
            }
        }
        Some(crate::entity::attachment_point(
            [0.0, seats.ride_height, f64::from(offset)],
            1.0,
            entity.yaw.load(),
        ))
    }

    fn get_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .filter(|passenger| passenger.get_player().is_some())
            .cloned()
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(inventory) = &self.inventory {
            inventory.write_nbt(nbt);
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(inventory) = &self.inventory {
            inventory.read_nbt(nbt);
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn has_container_inventory(&self) -> bool {
        self.inventory.is_some()
    }

    fn get_container_inventory(&self) -> Option<Arc<dyn Inventory>> {
        let container = self.inventory.as_ref()?;
        if container.has_loot_table() {
            let world = self.vehicle.entity.world.load();
            container.unpack_loot(&crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(self.vehicle.entity.pos.load()),
                ..Default::default()
            });
        }
        Some(container.clone() as Arc<dyn Inventory>)
    }

    fn on_above_bubble_column(&self, drag_down: bool, _pos: &BlockPos) {
        self.surface_bubble_contact.store(Some(if drag_down {
            BubbleColumnDirection::Down
        } else {
            BubbleColumnDirection::Up
        }));
        if !self.status.load().is_some_and(BoatStatus::is_underwater) && rand::random_ratio(1, 100)
        {
            let driver = self.get_controlling_passenger();
            let entity = self.get_entity();
            entity.world.load().emit_game_event_with_context(
                GameEvent::Splash.name(),
                entity.pos.load(),
                &GameEventContext {
                    source_entity: driver.as_deref(),
                },
            );
        }
    }

    fn on_inside_bubble_column(&self, drag_down: bool) {
        if !self.is_client_controlled() {
            crate::block::blocks::bubble_column::push_entity(self.get_entity(), drag_down, false);
        }
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = self.get_entity();
        let tick_start = entity.pos.load();
        let world = entity.world.load_full();
        let epoch = entity.vehicle_motion_epoch();
        let still_at_start = || {
            !entity.is_removed()
                && entity.pos.load() == tick_start
                && entity.vehicle_motion_epoch() == epoch
                && Arc::ptr_eq(&world, &entity.world.load())
        };
        self.surface_bubble_contact.store(None);
        let status = self.get_status();
        let old_status = self.status.swap(Some(status));
        self.tick_underwater(status);
        if !still_at_start() {
            return;
        }
        self.vehicle.tick();
        if !still_at_start() {
            return;
        }
        entity.update_last_pos();
        entity.base_tick(caller);
        if !still_at_start() {
            return;
        }

        if self.get_controlling_passenger().is_none()
            && (self.left_paddle_moving.load(Ordering::Relaxed)
                || self.right_paddle_moving.load(Ordering::Relaxed))
        {
            self.set_paddles(false, false);
            if !still_at_start() {
                return;
            }
        }
        // A paddle packet observer can mount a driver without moving the boat.
        if self.is_client_controlled() {
            entity.velocity.store(Vector3::default());
        } else {
            // A callback can change water or support without moving the boat.
            let status = self.get_status();
            self.status.store(Some(status));
            let offset = self.float_boat(status, old_status);
            if !self.move_from_physics(caller, offset) {
                return;
            }
        }

        // Each pass has its own visited set. The second starts from the old position after the history is drained.
        if !entity.apply_recorded_vehicle_block_effects(self, tick_start)
            || !entity.apply_vehicle_block_effects_from(self, tick_start)
        {
            return;
        }
        let contact_position = entity.pos.load();
        self.tick_bubble_column();
        let still_at_contact_position = || {
            !entity.is_removed()
                && entity.vehicle_motion_epoch() == epoch
                && entity.pos.load() == contact_position
                && Arc::ptr_eq(&world, &entity.world.load())
        };
        if still_at_contact_position() {
            self.tick_contacts(&world, still_at_contact_position);
        }
    }

    fn can_push_contact(&self, other: &dyn EntityBase) -> bool {
        let own_bounds = self.get_entity().bounding_box.load();
        let other = other.get_entity();
        let other_bottom = other.bounding_box.load().min.y;
        if other.entity_type.boat.is_some() {
            other_bottom < own_bounds.max.y
        } else {
            other_bottom <= own_bounds.min.y
        }
    }

    fn on_rider_positioned(&self, passenger: &dyn EntityBase) {
        let rider = passenger.get_entity();
        if passenger
            .get_player()
            .is_some_and(|player| player.client.bedrock().is_some())
            || rider
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_CAN_TURN_IN_BOATS)
        {
            return;
        }

        let hull = self.get_entity();
        let mob = passenger.get_mob();
        let animal_seat_yaw = if uses_animal_boat_seat(passenger)
            && hull
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len()
                == self.max_passengers()
        {
            if rider.entity_id % 2 == 0 {
                90.0
            } else {
                270.0
            }
        } else {
            0.0
        };
        let body_yaw = hull.yaw.load() + animal_seat_yaw;
        rider.body_yaw.store(body_yaw);
        if let Some(mob) = mob {
            rider.yaw.store(body_yaw);
            let head_yaw = rider.head_yaw.load();
            let delta = wrap_degrees(body_yaw - head_yaw);
            let limit = mob.get_max_head_rotation();
            rider
                .head_yaw
                .store(head_yaw + delta - delta.clamp(-limit, limit));
        } else {
            let yaw = rider.yaw.load();
            let delta = wrap_degrees(yaw - body_yaw);
            rider.yaw.store(yaw + delta.clamp(-105.0, 105.0) - delta);
        }
    }

    fn init_data_tracker(&self) {
        self.send_wobble_metadata();
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        let creative = source
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);
        let will_break = self.vehicle.entity.is_alive()
            && (creative || self.vehicle.get_damage() + amount * 10.0 > 40.0);

        let damaged = self.vehicle.damage_with_context(amount, source);

        if will_break && !creative && self.vehicle.entity.is_removed() {
            let world = self.vehicle.entity.world.load();
            if world.level_info.load().game_rules.entity_drops {
                let position = self.vehicle.entity.block_pos.load();
                if let Some(container) = &self.inventory
                    && container.claim_drops()
                {
                    container.unpack_loot(&crate::world::loot::LootContextParameters {
                        world: Some(&world),
                        position: Some(self.vehicle.entity.pos.load()),
                        ..Default::default()
                    });
                    let inventory: Arc<dyn Inventory> = container.clone();
                    world.scatter_inventory(&position, &inventory);
                }
                if let Some(item) =
                    Item::from_registry_key(self.vehicle.entity.entity_type.resource_name)
                {
                    let mut stack = ItemStack::new(1, item);
                    if let Some(custom_name) = self.vehicle.entity.custom_name.load().as_ref() {
                        stack.set_data_component(CustomNameImpl {
                            name: custom_name.clone(),
                        });
                    }
                    world.drop_stack(&position, stack);
                }
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        let hull = self.get_entity();
        let still_eligible = || {
            self.ticks_underwater.load() < TIME_TO_EJECT
                && !hull.is_submerged_in_water()
                && !player.get_entity().is_sneaking()
        };
        let can_add_passenger = still_eligible()
            && (player.client.java().is_some() || !player.get_entity().has_vehicle())
            && hull
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len()
                < self.max_passengers();

        if self.inventory.is_some() && !can_add_passenger {
            let _ = self.open_custom_inventory_screen(player);
            return true;
        }

        if !can_add_passenger {
            return false;
        }

        hull.mount_vehicle_by_interaction(player, self.max_passengers(), still_eligible)
    }

    fn set_paddle_state(&self, left: bool, right: bool) {
        self.set_paddles(left, right);
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        true
    }

    fn vehicle_bounce_air_drag(&self) -> Option<f64> {
        Some(1.0)
    }

    fn get_gravity(&self) -> f64 {
        if self.get_entity().has_no_gravity() {
            0.0
        } else {
            physics::DEFAULT_GRAVITY
        }
    }
}

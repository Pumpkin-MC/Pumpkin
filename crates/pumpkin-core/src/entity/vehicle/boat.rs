use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crossbeam::atomic::AtomicCell;

use crate::entity::living::{LivingEntity, is_allowed_by_team_rules};
use crate::entity::player::Player;
use crate::entity::vehicle::minecart::container::{self, MinecartInventory};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
use crate::world::World;
use crate::world::game_event::GameEventContext;

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_util::text::TextComponent;

use crate::entity::vehicle::vehicle::VehicleEntity;
use block_effects::BoatMovements;
use physics::BoatStatus;

mod block_effects;
mod dismount;
mod physics;
#[cfg(test)]
mod tests;

const FRONT_SEAT_OFFSET: f32 = 0.2;
const BACK_SEAT_OFFSET: f32 = -0.6;
const ANIMAL_SEAT_SHIFT: f32 = 0.2;
const CHEST_BOAT_CONTAINER_SIZE: usize = 27;
const TIME_TO_EJECT: f32 = 60.0;
const BUBBLE_TIME: u8 = 60;
const BUBBLE_COLUMN_LAUNCH_SPEED: f64 = 0.6;
const BUBBLE_COLUMN_SINK_SPEED: f64 = 0.7;
const CONTACT_VERTICAL_INSET: f64 = 0.01f32 as f64;

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
    movements: Mutex<BoatMovements>,
    surface_bubble_contact: AtomicCell<Option<BubbleColumnDirection>>,
    bubble_time: AtomicCell<u8>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
    inventory: Option<Arc<MinecartInventory>>,
}

struct BoatPosition {
    position: Vector3<f64>,
    world: Arc<World>,
    generation: u64,
}

impl BoatEntity {
    pub(crate) const CONTACT_RANGE: f64 = 0.2f32 as f64;

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
            movements: Mutex::default(),
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
        let entity = self.get_entity();
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers.iter().rev() {
            entity.remove_passenger(passenger.get_entity().entity_id);
        }
    }

    fn tick_bubble_column(&self) {
        let previous = self.bubble_time.load();
        let (contact, launch_position) = if previous == 1 {
            let movements = self.lock_movements();
            let contact = self.surface_bubble_contact.swap(None);
            (contact, contact.map(|_| self.movement_snapshot(&movements)))
        } else {
            (self.surface_bubble_contact.swap(None), None)
        };
        let remaining = contact.map_or(0, |_| {
            if previous == 0 {
                BUBBLE_TIME - 1
            } else {
                previous - 1
            }
        });
        if previous == remaining {
            return;
        }

        let entity = self.get_entity();
        self.bubble_time.store(remaining);
        entity.set_synced_data(
            pumpkin_data::tracked_data::abstract_boat::ID_BUBBLE_TIME,
            VarInt(i32::from(remaining)),
        );

        if let (Some(direction), Some(snapshot)) = (contact, launch_position) {
            if !self.movement_is_current(&snapshot, snapshot.position) {
                return;
            }
            let event = match direction {
                BubbleColumnDirection::Up => EntityStatus::BoatLaunch,
                BubbleColumnDirection::Down => {
                    self.eject_passengers();
                    EntityStatus::BoatSink
                }
            };
            let client_controlled = self.is_client_controlled();
            {
                let mut movements = self.lock_movements();
                if !self.movement_matches(&mut movements, &snapshot, snapshot.position) {
                    return;
                }
                // A controlling Java client applies the impulse when it receives the event.
                if !client_controlled {
                    let mut velocity = entity.velocity.load();
                    match direction {
                        BubbleColumnDirection::Up => velocity.y = BUBBLE_COLUMN_LAUNCH_SPEED,
                        BubbleColumnDirection::Down => velocity.y -= BUBBLE_COLUMN_SINK_SPEED,
                    }
                    entity.velocity.store(velocity);
                }
            }
            snapshot
                .world
                .broadcast_entity_event(entity, event, ActorEventID::None);
        }
    }

    pub(crate) fn additional_tick_movement(&self) -> f64 {
        // Air-to-water repositioning can search upward along the last fall step.
        self.last_yd.load().abs() + 2.0 + self.pending_block_movement_reach()
    }

    pub(crate) fn is_client_controlled(&self) -> bool {
        self.get_controlling_passenger()
            .is_some_and(|passenger| passenger.get_player().is_some())
    }

    fn lock_movements(&self) -> MutexGuard<'_, BoatMovements> {
        self.movements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn movement_snapshot(&self, movements: &BoatMovements) -> BoatPosition {
        let entity = self.get_entity();
        BoatPosition {
            position: entity.pos.load(),
            world: entity.world.load_full(),
            generation: movements.generation(),
        }
    }

    fn movement_matches(
        &self,
        movements: &mut BoatMovements,
        snapshot: &BoatPosition,
        expected_position: Vector3<f64>,
    ) -> bool {
        let entity = self.get_entity();
        if entity.is_removed() || movements.generation() != snapshot.generation {
            return false;
        }
        let position = entity.pos.load();
        if position != expected_position || !Arc::ptr_eq(&snapshot.world, &entity.world.load()) {
            // Native listeners can relocate the base entity directly.
            movements.invalidate();
            self.surface_bubble_contact.store(None);
            entity.last_pos.store(position);
            return false;
        }
        true
    }

    fn movement_is_current(
        &self,
        snapshot: &BoatPosition,
        expected_position: Vector3<f64>,
    ) -> bool {
        self.movement_matches(&mut self.lock_movements(), snapshot, expected_position)
    }

    fn push_nearby_boats(&self) {
        let entity = self.get_entity();
        let bounds = entity.bounding_box.load().expand(
            Self::CONTACT_RANGE,
            -CONTACT_VERTICAL_INSET,
            Self::CONTACT_RANGE,
        );
        let world = entity.world.load();
        let mut own_name = None;
        for other in world.get_boats_at_box(&bounds) {
            let other_entity = other.get_entity();
            if other_entity.entity_id == entity.entity_id
                || !other_entity.is_alive()
                || other.is_spectator()
                || !other.is_pushable()
            {
                continue;
            }

            let allowed = {
                let scoreboard = world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                scoreboard.get_teams().is_empty()
                    || is_allowed_by_team_rules(
                        scoreboard.get_entity_team(
                            own_name.get_or_insert_with(|| self.get_scoreboard_name()),
                        ),
                        scoreboard.get_entity_team(&other.get_scoreboard_name()),
                    )
            };
            if allowed {
                // Shared push logic skips riders in the same vehicle chain and applies the hull height check.
                self.push(other.as_ref());
            }
        }
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

    fn send_wobble_metadata(&self) {
        self.vehicle.send_wobble_metadata();
    }
}

impl EntityBase for BoatEntity {
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
            if passenger
                .get_mob()
                .and_then(|mob| mob.as_animal())
                .is_some()
            {
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
            .filter(|passenger| passenger.get_living_entity().is_some())
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

    fn teleport(
        &self,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) {
        let entity = self.get_entity();
        let (chunk, packet) = {
            let mut movements = self.lock_movements();
            movements.invalidate();
            self.surface_bubble_contact.store(None);
            entity.last_pos.store(position);
            entity.apply_teleport(position, yaw, pitch)
        };
        // Packet listeners may teleport the boat again.
        world.broadcast_to_chunk(chunk, &packet);
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
        let direction = if drag_down {
            BubbleColumnDirection::Down
        } else {
            BubbleColumnDirection::Up
        };
        self.surface_bubble_contact.store(Some(direction));

        // The Java client provides the surface splash sound and particles locally.
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
        let snapshot = self.movement_snapshot(&self.lock_movements());
        let status = self.get_status();
        let old_status = self.status.swap(Some(status));
        self.tick_underwater(status);
        self.vehicle.tick();
        let entity = &self.vehicle.entity;
        {
            let _movements = self.lock_movements();
            entity.update_last_pos()
        };
        entity.tick_base(caller);
        if entity.is_removed() {
            return;
        }

        // The controlling player's client simulates its boat.
        if self.is_client_controlled() {
            entity.velocity.store(Vector3::default());
        } else {
            if self.left_paddle_moving.load(Ordering::Relaxed)
                || self.right_paddle_moving.load(Ordering::Relaxed)
            {
                self.set_paddles(false, false);
            }
            // Surfacing only moves vertically, so the hull's current X/Z bounds suffice.
            let surface_border_clear = (old_status == Some(BoatStatus::InAir)
                && !matches!(status, BoatStatus::InAir | BoatStatus::OnLand))
            .then(|| {
                snapshot
                    .world
                    .no_border_collision(self, entity.bounding_box.load())
            });
            let surface_offset = {
                let mut movements = self.lock_movements();
                self.movement_matches(&mut movements, &snapshot, snapshot.position)
                    .then(|| self.float_boat(status, surface_border_clear))
            };
            if let Some(offset) = surface_offset {
                self.move_boat(caller, offset, &snapshot);
            }
        }
        self.apply_effects_from_blocks();
        entity.tick_block_collisions(caller);
        if !entity.is_removed() {
            self.tick_bubble_column();
            if !entity.is_removed() {
                self.push_nearby_boats();
            }
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
        let can_add_passenger = self.ticks_underwater.load() < TIME_TO_EJECT
            && !self.is_eye_in_water()
            && !player.get_entity().has_vehicle()
            && self
                .vehicle
                .entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len()
                < self.max_passengers();

        if self.inventory.is_some() && (!can_add_passenger || player.get_entity().is_sneaking()) {
            let _ = self.open_custom_inventory_screen(player);
            return true;
        }

        if player.get_entity().is_sneaking() || !can_add_passenger {
            return false;
        }

        let world = self.vehicle.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.vehicle.entity.entity_id) else {
            return false;
        };

        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        self.vehicle
            .entity
            .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);

        self.vehicle.entity.has_passenger(player.entity_id())
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

    fn get_gravity(&self) -> f64 {
        if self.vehicle.entity.has_no_gravity() {
            0.0
        } else {
            physics::DEFAULT_GRAVITY
        }
    }
}

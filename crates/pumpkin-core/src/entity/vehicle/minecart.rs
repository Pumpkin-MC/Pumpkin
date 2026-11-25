mod behavior;
mod chest;
pub mod command_block;
pub(crate) mod container;
mod furnace;
mod hopper;
mod new_behavior;
mod old_behavior;
mod rideable;
mod spawner;
mod tnt;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{BlockDirection, BlockStateId, HorizontalFacingExt, tracked_data};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::MinecartStep;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::block::BlockStateCodec;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::projectile::abstract_arrow::is_abstract_arrow;
use crate::entity::vehicle::vehicle::{VehicleEntity, creative_attacker};
use crate::entity::velocity_transfer::{self, CartClass, ExchangeRule, Motion};
use crate::entity::{Entity, EntityBase, living::LivingEntity, player::Player};
use crate::server::Server;
use crate::world::World;
use behavior::MinecartBehavior;
use chest::ChestMinecart;
use command_block::CommandBlockMinecart;
use container::MinecartInventory;
use furnace::FurnaceMinecart;
use hopper::HopperMinecart;
use new_behavior::Snap;
use rideable::RideableMinecart;
use spawner::SpawnerMinecart;
use tnt::TntMinecart;

/// Rail top above the block floor.
pub(crate) const RAIL_HEIGHT: f64 = 0.0625;
/// Vanilla `AbstractMinecart.WATER_SLOWDOWN_FACTOR`.
const WATER_SLOWDOWN_FACTOR: f64 = 0.95f32 as f64;
const LOWERED_PASSENGER_ATTACHMENT: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);
const DEFAULT_DISPLAY_OFFSET: i32 = 6;
const HOPPER_DISPLAY_OFFSET: i32 = 1;

/// Every `AbstractMinecart` subclass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MinecartType {
    Rideable,
    Chest,
    CommandBlock,
    Furnace,
    Hopper,
    Spawner,
    Tnt,
}

impl MinecartType {
    pub const ALL: [Self; 7] = [
        Self::Rideable,
        Self::Chest,
        Self::CommandBlock,
        Self::Furnace,
        Self::Hopper,
        Self::Spawner,
        Self::Tnt,
    ];

    #[must_use]
    pub const fn entity_type(self) -> &'static EntityType {
        match self {
            Self::Rideable => &EntityType::MINECART,
            Self::Chest => &EntityType::CHEST_MINECART,
            Self::CommandBlock => &EntityType::COMMAND_BLOCK_MINECART,
            Self::Furnace => &EntityType::FURNACE_MINECART,
            Self::Hopper => &EntityType::HOPPER_MINECART,
            Self::Spawner => &EntityType::SPAWNER_MINECART,
            Self::Tnt => &EntityType::TNT_MINECART,
        }
    }

    #[must_use]
    pub fn of(entity_type: &EntityType) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.entity_type().id == entity_type.id)
    }

    /// The item that places this cart, `None` for the spawner cart, which has no item
    /// (MC-163800).
    #[must_use]
    pub const fn item(self) -> Option<&'static Item> {
        match self {
            Self::Rideable => Some(&Item::MINECART),
            Self::Chest => Some(&Item::CHEST_MINECART),
            Self::CommandBlock => Some(&Item::COMMAND_BLOCK_MINECART),
            Self::Furnace => Some(&Item::FURNACE_MINECART),
            Self::Hopper => Some(&Item::HOPPER_MINECART),
            Self::Tnt => Some(&Item::TNT_MINECART),
            Self::Spawner => None,
        }
    }

    /// Vanilla `MinecartItem.type`: the cart an item places.
    #[must_use]
    pub fn from_item(item: &Item) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.item().is_some_and(|placed| placed.id == item.id))
    }

    /// Vanilla `getDropItem`.
    #[must_use]
    pub const fn drop_item(self) -> &'static Item {
        match self {
            Self::Chest => &Item::CHEST_MINECART,
            Self::Furnace => &Item::FURNACE_MINECART,
            Self::Hopper => &Item::HOPPER_MINECART,
            Self::Tnt => &Item::TNT_MINECART,
            Self::Rideable | Self::CommandBlock | Self::Spawner => &Item::MINECART,
        }
    }

    /// Vanilla `isFurnace`, as the class used by `pushOtherMinecart`.
    const fn cart_class(self) -> CartClass {
        match self {
            Self::Furnace => CartClass::Furnace,
            _ => CartClass::Plain,
        }
    }

    /// Vanilla `getDefaultDisplayOffset`.
    const fn default_display_offset(self) -> i32 {
        match self {
            Self::Hopper => HOPPER_DISPLAY_OFFSET,
            _ => DEFAULT_DISPLAY_OFFSET,
        }
    }
}

/// Vanilla `instanceof AbstractMinecart`.
#[must_use]
pub fn is_minecart(entity_type: &EntityType) -> bool {
    MinecartType::of(entity_type).is_some()
}

/// Subclass state.
enum MinecartKind {
    Rideable(RideableMinecart),
    Chest(ChestMinecart),
    CommandBlock(CommandBlockMinecart),
    Furnace(FurnaceMinecart),
    Hopper(HopperMinecart),
    Spawner(SpawnerMinecart),
    Tnt(TntMinecart),
}

impl MinecartKind {
    fn new(kind: MinecartType) -> Self {
        match kind {
            MinecartType::Rideable => Self::Rideable(RideableMinecart),
            MinecartType::Chest => Self::Chest(ChestMinecart::new()),
            MinecartType::CommandBlock => Self::CommandBlock(CommandBlockMinecart::new()),
            MinecartType::Furnace => Self::Furnace(FurnaceMinecart::new()),
            MinecartType::Hopper => Self::Hopper(HopperMinecart::new()),
            MinecartType::Spawner => Self::Spawner(SpawnerMinecart::new()),
            MinecartType::Tnt => Self::Tnt(TntMinecart::new()),
        }
    }

    const fn minecart_type(&self) -> MinecartType {
        match self {
            Self::Rideable(_) => MinecartType::Rideable,
            Self::Chest(_) => MinecartType::Chest,
            Self::CommandBlock(_) => MinecartType::CommandBlock,
            Self::Furnace(_) => MinecartType::Furnace,
            Self::Hopper(_) => MinecartType::Hopper,
            Self::Spawner(_) => MinecartType::Spawner,
            Self::Tnt(_) => MinecartType::Tnt,
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        match self {
            Self::Chest(minecart) => Some(minecart.inventory()),
            Self::Hopper(minecart) => Some(minecart.inventory()),
            _ => None,
        }
    }
}

/// Vanilla `AbstractMinecart`: state shared by the minecart family, the subclass lives in
/// `kind`, the movement in `behavior`.
pub struct AbstractMinecart {
    pub vehicle: VehicleEntity,
    kind: MinecartKind,
    behavior: MinecartBehavior,
    on_rails: AtomicBool,
    flipped: AtomicBool,
    custom_display_block: AtomicCell<Option<BlockStateId>>,
    display_offset: AtomicI32,
}

impl AbstractMinecart {
    #[must_use]
    pub fn new(world: Arc<World>, position: Vector3<f64>, kind: MinecartType) -> Self {
        Self::from_uuid(Uuid::new_v4(), world, position, kind)
    }

    #[must_use]
    pub fn from_uuid(
        uuid: Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        kind: MinecartType,
    ) -> Self {
        let behavior = MinecartBehavior::for_world(&world);
        let mut entity = Entity::from_uuid(uuid, world, position, kind.entity_type());
        entity.blocks_building = true;
        Self {
            vehicle: VehicleEntity::new(entity),
            kind: MinecartKind::new(kind),
            behavior,
            on_rails: AtomicBool::new(false),
            flipped: AtomicBool::new(false),
            custom_display_block: AtomicCell::new(None),
            display_offset: AtomicI32::new(kind.default_display_offset()),
        }
    }

    /// Vanilla `AbstractMinecart.createMinecart`: a placed or dispensed cart, which takes the
    /// stack's name and data first and is then snapped onto the rail by the experimental
    /// movement.
    #[must_use]
    pub fn create(
        world: Arc<World>,
        position: Vector3<f64>,
        kind: MinecartType,
        stack: &ItemStack,
        player: Option<&Player>,
    ) -> Self {
        let minecart = Self::new(world, position, kind);
        crate::item::items::spawn_egg::apply_entity_variant(stack, &minecart, player);
        if let Some(behavior) = minecart.behavior.new_behavior() {
            let (pos, rail) = minecart.current_rail(&minecart.vehicle.entity.world.load());
            minecart.adjust_to_rails(behavior, pos, rail, Snap::Instant);
        }
        minecart
    }

    /// Vanilla `useExperimentalMovement`.
    #[must_use]
    pub const fn uses_experimental_movement(&self) -> bool {
        self.behavior.new_behavior().is_some()
    }

    #[must_use]
    pub const fn minecart_type(&self) -> MinecartType {
        self.kind.minecart_type()
    }

    #[must_use]
    pub const fn command_block(&self) -> Option<&CommandBlockMinecart> {
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => Some(minecart),
            _ => None,
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        self.kind.container()
    }

    /// Vanilla `unpackChestVehicleLootTable`, which every item read runs first.
    fn unpack_container_loot(&self, container: &Arc<MinecartInventory>) {
        if container.has_loot_table() {
            let world = self.vehicle.entity.world.load();
            container.unpack_loot(&crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(self.vehicle.entity.pos.load()),
                ..Default::default()
            });
        }
    }

    /// Vanilla `getRedstoneSignalFromContainer` of a container cart, `None` for other carts.
    #[must_use]
    pub fn container_signal(&self) -> Option<u8> {
        let container = self.container()?;
        self.unpack_container_loot(container);
        Some(container.redstone_signal())
    }

    /// Vanilla `MinecartBehavior.getMaxSpeed`.
    fn behavior_max_speed(&self) -> f64 {
        let entity = &self.vehicle.entity;
        let in_water = entity.is_in_water();
        match self.behavior {
            MinecartBehavior::Old if in_water => old_behavior::MAX_SPEED_IN_WATER,
            MinecartBehavior::Old => old_behavior::MAX_SPEED,
            MinecartBehavior::New(_) => {
                let blocks_per_second = entity
                    .world
                    .load()
                    .level_info
                    .load()
                    .game_rules
                    .max_minecart_speed as f64;
                blocks_per_second * if in_water { 0.5 } else { 1.0 } / 20.0
            }
        }
    }

    /// Vanilla `AbstractMinecart.getMaxSpeed` and the furnace override.
    fn max_speed(&self) -> f64 {
        let speed = self.behavior_max_speed();
        match self.kind {
            MinecartKind::Furnace(_) if self.vehicle.entity.is_in_water() => speed * 0.75,
            MinecartKind::Furnace(_) => speed * 0.5,
            _ => speed,
        }
    }

    /// Vanilla `applyNaturalSlowdown` and its overrides.
    fn apply_natural_slowdown(&self, velocity: Vector3<f64>) -> Vector3<f64> {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Chest(minecart) => {
                container::slowdown(entity, minecart.inventory(), velocity)
            }
            MinecartKind::Hopper(minecart) => {
                container::slowdown(entity, minecart.inventory(), velocity)
            }
            MinecartKind::Furnace(minecart) => {
                self.base_slowdown(minecart.slowdown(entity, velocity))
            }
            _ => self.base_slowdown(velocity),
        }
    }

    /// Vanilla `AbstractMinecart.applyNaturalSlowdown`.
    fn base_slowdown(&self, velocity: Vector3<f64>) -> Vector3<f64> {
        let entity = &self.vehicle.entity;
        // `getSlowdownFactor` of the behaviour
        let factor = match (&self.behavior, entity.has_passengers()) {
            (_, true) => 0.997,
            (MinecartBehavior::Old, false) => 0.96,
            (MinecartBehavior::New(_), false) => 0.975,
        };
        let slowed = velocity.multiply(factor, 0.0, factor);
        if entity.is_in_water() {
            slowed * WATER_SLOWDOWN_FACTOR
        } else {
            slowed
        }
    }

    /// Vanilla `getMotionDirection`: old movement rides sideways, a flipped cart backwards.
    fn motion_direction(&self) -> BlockDirection {
        let facing = self
            .vehicle
            .entity
            .get_horizontal_facing()
            .to_block_direction();
        match self.behavior {
            MinecartBehavior::Old if self.flipped.load(Ordering::Relaxed) => {
                facing.opposite().rotate_clockwise()
            }
            MinecartBehavior::Old => facing.rotate_clockwise(),
            MinecartBehavior::New(_) => facing,
        }
    }

    /// Vanilla `activateMinecart`: the cart is on an activator rail.
    fn activate(&self, powered: bool) {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Rideable(_) if powered => {
                RideableMinecart::eject_passengers(entity);
                if self.vehicle.get_hurt_time() == 0 {
                    self.vehicle.wobble(50.0);
                }
            }
            MinecartKind::CommandBlock(minecart) if powered => minecart.activate(entity),
            MinecartKind::Tnt(minecart) if powered && !minecart.is_primed() => {
                minecart.prime_fuse(entity);
            }
            MinecartKind::Hopper(minecart) => minecart.set_enabled(!powered),
            _ => {}
        }
    }

    /// Subclass `tick` after `super.tick()`.
    fn tick_subclass(&self) {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Furnace(minecart) => minecart.tick(entity),
            MinecartKind::Hopper(minecart) => minecart.tick(entity),
            MinecartKind::Spawner(minecart) => minecart.tick(entity),
            MinecartKind::Tnt(minecart) => minecart.tick(self),
            MinecartKind::Rideable(_) | MinecartKind::Chest(_) | MinecartKind::CommandBlock(_) => {}
        }
    }

    fn set_custom_display_block(&self, state: Option<BlockStateId>) {
        self.custom_display_block.store(state);
        let value = state.map_or(VarInt(0), |id| VarInt(i32::from(id.as_u16())));
        self.vehicle.entity.set_synced_data(
            tracked_data::abstract_minecart::DATA_ID_CUSTOM_DISPLAY_BLOCK,
            value,
        );
    }

    fn set_display_offset(&self, offset: i32) {
        self.display_offset.store(offset, Ordering::Relaxed);
        self.vehicle.entity.set_synced_data(
            tracked_data::abstract_minecart::DATA_ID_DISPLAY_OFFSET,
            VarInt(offset),
        );
    }

    /// Vanilla `AbstractMinecartContainer.remove` with a destroying reason: a container cart
    /// spills its items, whatever `doEntityDrops` says (MC-312690).
    fn spill_contents(&self) {
        let Some(container) = self.container() else {
            return;
        };
        if !container.claim_drops() {
            return;
        }
        let entity = &self.vehicle.entity;
        let world = entity.world.load();
        container.unpack_loot(&crate::world::loot::LootContextParameters {
            world: Some(&world),
            position: Some(entity.pos.load()),
            ..Default::default()
        });
        let inventory: Arc<dyn Inventory> = container.clone();
        world.scatter_inventory(&entity.block_pos.load(), &inventory);
    }

    /// Vanilla `discard`.
    fn discard(&self) {
        self.spill_contents();
        self.vehicle.entity.remove();
    }

    /// Vanilla `VehicleEntity.destroy(level, source)` and the `MinecartTNT` override.
    fn destroy(&self, damage_type: &DamageType, source: Option<&dyn EntityBase>) {
        let entity = &self.vehicle.entity;
        if let MinecartKind::Tnt(minecart) = &self.kind
            && (tnt::damage_source_ignites_tnt(damage_type, source)
                || entity.velocity.load().horizontal_length_squared() >= tnt::IGNITE_SPEED_SQUARED)
        {
            if !minecart.is_primed() {
                minecart.prime_fuse(entity);
                let mut rng = rand::rng();
                minecart.set_fuse(rng.random_range(0..20) + rng.random_range(0..20));
            }
            return;
        }

        // `destroy(level, dropItem)`
        if !self.vehicle.destroy_event_allows(source) {
            return;
        }
        self.spill_contents();
        self.vehicle.kill();
        if !entity
            .world
            .load()
            .level_info
            .load()
            .game_rules
            .entity_drops
        {
            return;
        }
        let mut stack = ItemStack::new(1, self.minecart_type().drop_item());
        if let Some(custom_name) = entity.custom_name.load().as_ref() {
            stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                name: custom_name.clone(),
            });
        }
        // Vanilla `spawnAtLocation`; `spawn_item_at` also keeps firing `ItemSpawnEvent`.
        entity.world.load().spawn_item_at(entity.pos.load(), stack);
    }
}

impl EntityBase for AbstractMinecart {
    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        let passenger_type = passenger.get_entity().entity_type;
        (passenger_type == &EntityType::VILLAGER || passenger_type == &EntityType::WANDERING_TRADER)
            .then_some(LOWERED_PASSENGER_ATTACHMENT)
    }

    fn dismount_location_for_passenger(&self, passenger: &dyn EntityBase) -> Vector3<f64> {
        crate::entity::dismount_helper::minecart_dismount_location(
            &self.vehicle.entity,
            self.motion_direction(),
            passenger,
        )
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(state) = self.custom_display_block.load() {
            nbt.put_compound("DisplayState", BlockStateCodec::state_to_nbt(state));
        }
        let offset = self.display_offset.load(Ordering::Relaxed);
        if offset != self.minecart_type().default_display_offset() {
            nbt.put_int("DisplayOffset", offset);
        }
        nbt.put_bool("FlippedRotation", self.flipped.load(Ordering::Relaxed));
        // Vanilla stores `firstTick` under this name.
        nbt.put_bool(
            "HasTicked",
            self.vehicle.entity.first_tick.load(Ordering::Relaxed),
        );

        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.write_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Furnace(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Rideable(_) => {}
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let display = nbt
            .get_compound("DisplayState")
            .and_then(BlockStateCodec::state_from_nbt);
        if display.is_some() || self.custom_display_block.load().is_some() {
            self.set_custom_display_block(display);
        }
        let default_offset = self.minecart_type().default_display_offset();
        let offset = nbt.get_int("DisplayOffset").unwrap_or(default_offset);
        if offset != self.display_offset.load(Ordering::Relaxed) {
            self.set_display_offset(offset);
        }
        self.flipped.store(
            nbt.get_bool("FlippedRotation").unwrap_or(false),
            Ordering::Relaxed,
        );
        self.vehicle.entity.first_tick.store(
            nbt.get_bool("HasTicked").unwrap_or(false),
            Ordering::Relaxed,
        );

        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.read_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.read_nbt(entity, nbt),
            MinecartKind::Furnace(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Rideable(_) => {}
        }
    }

    /// Vanilla `AbstractMinecart.tick`, then the subclass tick.
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.vehicle.entity;
        if let MinecartKind::CommandBlock(minecart) = &self.kind {
            minecart.step_tick();
        }
        entity.update_last_pos();
        let last_yaw = entity.yaw.load();

        self.vehicle.tick();
        entity.check_out_of_world(caller);
        if entity.is_removed() {
            return;
        }
        entity.tick_portal(caller);
        match &self.behavior {
            MinecartBehavior::Old => self.tick_old(caller, last_yaw),
            MinecartBehavior::New(behavior) => self.tick_new(caller, behavior),
        }
        // Lava's ignite, hurt and fall distance halving run in `update_fluid_state`.
        entity.update_fluid_state(caller);
        entity.first_tick.store(false, Ordering::Relaxed);

        if entity.is_removed() {
            return;
        }
        self.tick_subclass();
    }

    /// Vanilla `MinecartTNT.causeFallDamage`: a long fall sets the cart off.
    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f64,
        damage_modifier: f32,
    ) -> bool {
        if let MinecartKind::Tnt(minecart) = &self.kind
            && fall_distance >= tnt::FALL_EXPLODE_DISTANCE
        {
            let power = fall_distance / 10.0;
            minecart.explode(caller, power * power, None);
        }
        // Passengers take the fall too, even when the cart lands off the rails (MC-275743).
        self.vehicle
            .entity
            .cause_fall_damage(fall_distance, damage_modifier)
    }

    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn has_container_inventory(&self) -> bool {
        self.container().is_some()
    }

    fn get_container_inventory(&self) -> Option<Arc<dyn Inventory>> {
        let container = self.container()?;
        self.unpack_container_loot(container);
        Some(container.clone() as Arc<dyn Inventory>)
    }

    fn is_pushable(&self) -> bool {
        true
    }

    fn is_on_rails(&self) -> bool {
        self.on_rails.load(Ordering::Relaxed)
    }

    /// Vanilla `AbstractMinecart.getBlockSpeedFactor`: rails never slow a cart.
    fn get_block_speed_factor(&self) -> f32 {
        let entity = &self.vehicle.entity;
        if entity
            .world
            .load()
            .get_block(&entity.block_pos.load())
            .has_tag(&tag::Block::MINECRAFT_RAILS)
        {
            1.0
        } else {
            entity.get_block_speed_factor()
        }
    }

    /// Vanilla `AbstractMinecart.push`.
    fn push(&self, other: &dyn EntityBase) {
        let this = &self.vehicle.entity;
        let other_entity = other.get_entity();
        if this.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
            || this.has_passenger(other_entity.entity_id)
        {
            return;
        }

        let this_motion = Motion::of(this);
        let other_motion = Motion::of(other_entity);
        let Some(push) = velocity_transfer::minecart_push(&this_motion, &other_motion) else {
            return;
        };

        if let Some(other_type) = MinecartType::of(other_entity.entity_type) {
            let rule = match self.behavior {
                MinecartBehavior::Old => ExchangeRule::HeadOn {
                    this_yaw: this.yaw.load(),
                },
                MinecartBehavior::New(_) => ExchangeRule::Always,
            };
            if let Some((mine, theirs)) = velocity_transfer::minecart_exchange(
                (&this_motion, self.minecart_type().cart_class()),
                (&other_motion, other_type.cart_class()),
                rule,
                push,
            ) {
                mine.apply(this);
                theirs.apply(other_entity);
            }
        } else {
            this.push_by(push * -1.0);
            other_entity.push_by(push * 0.25);
        }
    }

    fn init_data_tracker(&self) {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => minecart.init_data_tracker(entity),
            MinecartKind::Furnace(minecart) => minecart.init_data_tracker(entity),
            _ => {}
        }
    }

    /// Vanilla `isPickable`.
    fn can_hit(&self) -> bool {
        !self.vehicle.entity.is_removed()
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        let entity = &self.vehicle.entity;
        if !entity.is_alive() {
            return true;
        }
        // Plugins cancel the hit before a TNT cart can prime or explode.
        if !self.vehicle.damage_event_allows(amount, source) {
            return false;
        }
        // Vanilla `MinecartTNT.hurtServer`: a burning arrow sets the cart off even before the
        // invulnerability check. Only arrows do (MC-119000), whatever `mobGriefing` says
        // (MC-312790), and the arrow's speed powers the blast (MC-312963).
        if let MinecartKind::Tnt(minecart) = &self.kind
            && let Some(arrow) = source.filter(|source| {
                is_abstract_arrow(source.get_entity().entity_type)
                    && source.get_entity().is_on_fire()
            })
        {
            let world = entity.world.load();
            let credit =
                cause.and_then(|cause| world.get_entity_by_id(cause.get_entity().entity_id));
            minecart.explode(
                self,
                arrow.get_entity().velocity.load().length_squared(),
                credit,
            );
            if entity.is_removed() {
                return true;
            }
        }
        if entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }

        // `MinecartTNT.shouldSourceDestroy`
        let should_source_destroy = matches!(self.kind, MinecartKind::Tnt(_))
            && tnt::damage_source_ignites_tnt(&damage_type, source);
        if self.vehicle.hurt(amount, cause, should_source_destroy) {
            self.destroy(&damage_type, source);
        } else if creative_attacker(cause) && self.vehicle.destroy_event_allows(source) {
            // A creative player's projectile discards the cart too (MC-21098).
            self.discard();
        }
        true
    }

    /// Vanilla `Entity.kill`, which a container cart's `remove` spills.
    fn kill(&self, _caller: &dyn EntityBase) {
        self.spill_contents();
        self.vehicle.kill();
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Chest(minecart) => {
                minecart.interact(entity, player);
                true
            }
            MinecartKind::CommandBlock(_) => CommandBlockMinecart::interact(player),
            MinecartKind::Furnace(minecart) => minecart.interact(entity, player, item_stack),
            MinecartKind::Hopper(minecart) => {
                minecart.interact(entity, player);
                true
            }
            MinecartKind::Rideable(_) => RideableMinecart::interact(entity, player),
            // Flint and steel can't light a TNT cart (MC-108558).
            MinecartKind::Spawner(_) | MinecartKind::Tnt(_) => false,
        }
    }

    /// Vanilla `AbstractMinecart.move`.
    fn move_entity(&self, caller: &dyn EntityBase, motion: Vector3<f64>) {
        let entity = &self.vehicle.entity;
        match self.behavior {
            // Old movement: block effects follow every move.
            MinecartBehavior::Old => {
                entity.move_entity(caller, motion);
                entity.tick_block_collisions(caller);
            }
            // TODO: vanilla clears `onRails` after a `MoverType.PISTON` move; needs piston
            // entity moves to tell their mover type apart.
            MinecartBehavior::New(_) => self.move_new(caller, motion),
        }
    }

    /// Vanilla `AbstractMinecart.getKnownMovement`.
    fn known_movement(&self) -> Vector3<f64> {
        let velocity = self.vehicle.entity.velocity.load();
        match self.behavior {
            MinecartBehavior::Old => old_behavior::known_movement(velocity),
            MinecartBehavior::New(_) => velocity,
        }
    }

    fn ignored_block_collisions(&self) -> Option<(BlockPos, Option<BlockPos>)> {
        self.behavior.new_behavior()?;
        self.ignored_collisions()
    }

    fn minecart_lerp_steps(&self) -> Option<&std::sync::Mutex<Vec<MinecartStep>>> {
        self.behavior
            .new_behavior()
            .map(|behavior| &behavior.lerp_steps)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_util::text::TextComponent;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn command_block_minecart_nbt_and_activation_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = AbstractMinecart::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            MinecartType::CommandBlock,
        );

        let mut nbt = NbtCompound::new();
        nbt.put_string("Command", "Searge".to_string());
        nbt.put_bool("TrackOutput", true);
        nbt.put_long("LastExecution", 12);
        cart.read_nbt_non_mut(&nbt);

        let Some(cb) = cart.command_block() else {
            return Err("expected CommandBlockMinecart".into());
        };
        assert_eq!(cb.get_command(), "Searge");
        assert_eq!(cb.get_success_count(), 0);

        // Fewer than 4 ticks elapsed since `last_activated` (0): must not activate.
        for _ in 0..3 {
            cb.step_tick();
            cb.activate(&cart.vehicle.entity);
        }
        assert_eq!(cb.get_success_count(), 0);

        // Tick 4 reaches ACTIVATION_DELAY (4) and triggers the command.
        cb.step_tick();
        cb.activate(&cart.vehicle.entity);
        assert_eq!(cb.get_success_count(), 1);
        assert_eq!(
            cb.get_last_output_or_empty(),
            TextComponent::text("#itzlipofutzli")
        );

        let mut saved = NbtCompound::new();
        cart.write_nbt(&mut saved);
        assert_eq!(saved.get_string("Command"), Some("Searge"));
        assert_eq!(saved.get_int("SuccessCount"), Some(1));
        assert_eq!(saved.get_bool("TrackOutput"), Some(true));
        assert!(saved.get("LastOutput").is_some());

        // Setting TrackOutput=false clears LastOutput on load and omits it on save.
        let mut no_track = saved.clone();
        no_track.put_bool("TrackOutput", false);
        cart.read_nbt_non_mut(&no_track);
        let mut saved_no_track = NbtCompound::new();
        cart.write_nbt(&mut saved_no_track);
        assert!(saved_no_track.get("LastOutput").is_none());

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn spawner_minecart_nbt_roundtrip() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = AbstractMinecart::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            MinecartType::Spawner,
        );

        let mut spawn_entity = NbtCompound::new();
        spawn_entity.put_string("id", "minecraft:pig".to_string());
        let mut spawn_data = NbtCompound::new();
        spawn_data.put_compound("entity", spawn_entity);

        let mut nbt = NbtCompound::new();
        nbt.put_short("Delay", 45);
        nbt.put_short("MinSpawnDelay", 100);
        nbt.put_short("MaxSpawnDelay", 300);
        nbt.put_short("SpawnCount", 2);
        nbt.put_short("SpawnRange", 3);
        nbt.put_short("MaxNearbyEntities", 5);
        nbt.put_short("RequiredPlayerRange", 12);
        nbt.put_compound("SpawnData", spawn_data);

        cart.read_nbt_non_mut(&nbt);

        let mut saved = NbtCompound::new();
        cart.write_nbt(&mut saved);
        assert_eq!(saved.get_short("Delay"), Some(45));
        assert_eq!(saved.get_short("MinSpawnDelay"), Some(100));
        assert_eq!(saved.get_short("MaxSpawnDelay"), Some(300));
        assert_eq!(saved.get_short("SpawnCount"), Some(2));
        assert_eq!(saved.get_short("SpawnRange"), Some(3));
        assert_eq!(saved.get_short("MaxNearbyEntities"), Some(5));
        assert_eq!(saved.get_short("RequiredPlayerRange"), Some(12));
        assert_eq!(
            saved
                .get_compound("SpawnData")
                .and_then(|d| d.get_compound("entity"))
                .and_then(|e| e.get_string("id")),
            Some("minecraft:pig")
        );

        world.level.shutdown().await;
        Ok(())
    }
}

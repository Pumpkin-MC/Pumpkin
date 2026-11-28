mod chest;
mod classic;
pub mod command_block;
pub(crate) mod container;
mod dismount;
mod furnace;
mod hopper;
mod rideable;
mod spawner;
mod tnt;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rand::RngExt;

use crate::entity::projectile::abstract_arrow::is_abstract_arrow;
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;

use super::block_effects::hurt_vehicle_in_lava;
use crate::entity::vehicle::vehicle::VehicleEntity;
use chest::ChestMinecart;
use classic::Rail;
use command_block::CommandBlockMinecart;
use container::MinecartInventory;
use furnace::FurnaceMinecart;
use hopper::HopperMinecart;
use rideable::RideableMinecart;
use spawner::SpawnerMinecart;
use tnt::TntMinecart;

const DEFAULT_FLIPPED_ROTATION: bool = false;

pub struct MinecartEntity {
    pub vehicle: VehicleEntity,
    kind: MinecartKind,
    flipped_rotation: AtomicBool,
    on_rails: AtomicBool,
}

enum MinecartKind {
    Rideable(RideableMinecart),
    Chest(ChestMinecart),
    CommandBlock(CommandBlockMinecart),
    Furnace(FurnaceMinecart),
    Hopper(HopperMinecart),
    Spawner(SpawnerMinecart),
    Tnt(TntMinecart),
    Other,
}

impl MinecartEntity {
    pub fn new(mut entity: Entity) -> Self {
        entity.blocks_building = true;
        let kind = match entity.entity_type.id {
            id if id == EntityType::MINECART.id => MinecartKind::Rideable(RideableMinecart),
            id if id == EntityType::CHEST_MINECART.id => MinecartKind::Chest(ChestMinecart::new()),
            id if id == EntityType::COMMAND_BLOCK_MINECART.id => {
                MinecartKind::CommandBlock(CommandBlockMinecart::new())
            }
            id if id == EntityType::FURNACE_MINECART.id => {
                MinecartKind::Furnace(FurnaceMinecart::new())
            }
            id if id == EntityType::HOPPER_MINECART.id => {
                MinecartKind::Hopper(HopperMinecart::new())
            }
            id if id == EntityType::SPAWNER_MINECART.id => {
                MinecartKind::Spawner(SpawnerMinecart::new())
            }
            id if id == EntityType::TNT_MINECART.id => MinecartKind::Tnt(TntMinecart::new()),
            _ => MinecartKind::Other,
        };
        Self {
            vehicle: VehicleEntity::new(entity),
            kind,
            flipped_rotation: AtomicBool::new(DEFAULT_FLIPPED_ROTATION),
            on_rails: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub const fn command_block(&self) -> Option<&CommandBlockMinecart> {
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => Some(minecart),
            _ => None,
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        match &self.kind {
            MinecartKind::Chest(minecart) => Some(minecart.inventory()),
            MinecartKind::Hopper(minecart) => Some(minecart.inventory()),
            _ => None,
        }
    }

    const fn drop_item(&self) -> &'static Item {
        match &self.kind {
            MinecartKind::Chest(_) => &Item::CHEST_MINECART,
            MinecartKind::Furnace(_) => &Item::FURNACE_MINECART,
            MinecartKind::Hopper(_) => &Item::HOPPER_MINECART,
            MinecartKind::Tnt(_) => &Item::TNT_MINECART,
            MinecartKind::Rideable(_)
            | MinecartKind::CommandBlock(_)
            | MinecartKind::Spawner(_)
            | MinecartKind::Other => &Item::MINECART,
        }
    }
    fn activate_rideable_minecart(&self) {
        let cart = &self.vehicle.entity;
        if cart.eject_vehicle_passengers() && self.vehicle.get_hurt_time() == 0 {
            self.vehicle.set_hurt_dir(-self.vehicle.get_hurt_dir());
            self.vehicle.set_hurt_time(10);
            self.vehicle.set_damage(50.0);
            self.vehicle.send_wobble_metadata();
        }
    }

    /// The classic move applies its endpoint effects before rail activation.
    fn move_with_destination_effects(
        &self,
        caller: &dyn EntityBase,
        movement: Vector3<f64>,
    ) -> bool {
        let entity = &self.vehicle.entity;
        entity.move_entity(caller, movement)
            && entity.apply_vehicle_destination_block_effects(caller)
    }

    fn activate_rail(&self, powered: bool) {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Hopper(cart) => cart.set_enabled(!powered),
            MinecartKind::Rideable(_) if powered => self.activate_rideable_minecart(),
            MinecartKind::CommandBlock(cart) if powered => cart.activate(entity),
            MinecartKind::Tnt(cart) if powered => cart.prime(entity, 80),
            _ => {}
        }
    }

    fn tick_contacts(&self, moved: bool, still_current: impl Fn() -> bool) {
        let cart = &self.vehicle.entity;
        let world = cart.world.load();
        let bounds = cart
            .bounding_box
            .load()
            .expand(f64::from(0.2f32), 0.0, f64::from(0.2f32));
        let fast_rideable = matches!(self.kind, MinecartKind::Rideable(_))
            && cart.velocity.load().horizontal_length_squared() >= 0.01;
        // Preserve the slow pickup inherited from the consolidated riding dependency.
        if !fast_rideable && moved && matches!(self.kind, MinecartKind::Rideable(_)) {
            RideableMinecart::pick_up_nearby_mob(cart, &still_current);
            if !still_current() {
                return;
            }
        }
        let team = fast_rideable.then(|| self.get_team()).flatten();
        for other in world.get_all_at_box(&bounds) {
            let target = other.get_entity();
            if target.entity_id == cart.entity_id
                || target.is_removed()
                || !world.is_pushable_by_vehicle(other.as_ref())
            {
                continue;
            }
            let other_is_cart = target.entity_type.has_tag(&tag::EntityType::C_MINECARTS);
            if fast_rideable {
                if other.is_spectator()
                    || !crate::entity::living::is_allowed_by_team_rules(
                        team.as_ref(),
                        other.get_team().as_ref(),
                    )
                {
                    continue;
                }
                if other.get_player().is_some()
                    || target.entity_type == &EntityType::IRON_GOLEM
                    || other_is_cart
                    || cart.has_passengers()
                    || other.is_passenger()
                {
                    other.push(self);
                } else if other.can_ride(self)
                    && let Some(vehicle) = world.get_entity_by_id(cart.entity_id)
                {
                    cart.board_vehicle_contact(&vehicle, &other, 1, &still_current, || {
                        bounds.intersects(&target.bounding_box.load())
                    });
                }
            } else if other_is_cart && !cart.has_passenger(target.entity_id) {
                other.push(self);
            }
            if !still_current() {
                return;
            }
        }
    }
}

const LOWERED_PASSENGER_ATTACHMENT: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);

impl EntityBase for MinecartEntity {
    fn get_gravity(&self) -> f64 {
        self.classic_gravity()
    }

    fn vehicle_bounce_air_drag(&self) -> Option<f64> {
        Some(classic::AIR_DRAG)
    }

    fn is_on_rails(&self) -> bool {
        self.on_rails.load(Ordering::Relaxed)
    }

    fn get_block_speed_factor(&self) -> f32 {
        let entity = self.get_entity();
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

    fn vehicle_contact_tick_reach(&self) -> Option<f64> {
        // Track alignment and acceleration can extend movement before projection.
        const RAIL_REACH: f64 = 2.0;
        Some(RAIL_REACH)
    }

    fn vehicle_requires_serial_tick(&self) -> bool {
        matches!(self.kind, MinecartKind::CommandBlock(_) | MinecartKind::Tnt(_))
            || self.get_entity().portal_manager.try_lock().map_or(true, |portal| portal.is_some())
            || (matches!(self.kind, MinecartKind::Rideable(_))
                && self.get_entity().world.load().server.upgrade().is_some_and(|server| {
                    server.plugin_manager.has_handlers::<crate::plugin::api::events::entity::entity_mount::EntityMountEvent>()
                        || server.plugin_manager.has_handlers::<crate::plugin::api::events::vehicle::vehicle_enter::VehicleEnterEvent>()
                }))
    }

    fn vehicle_dismount_position(
        &self,
        passenger: &dyn EntityBase,
        still_current: &dyn Fn() -> bool,
    ) -> Option<Vector3<f64>> {
        self.get_dismount_location_for_passenger(passenger, still_current)
    }

    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        let passenger_type = passenger.get_entity().entity_type;
        (passenger_type == &EntityType::VILLAGER || passenger_type == &EntityType::WANDERING_TRADER)
            .then_some(LOWERED_PASSENGER_ATTACHMENT)
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool(
            "FlippedRotation",
            self.flipped_rotation.load(Ordering::Relaxed),
        );
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
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.flipped_rotation.store(
            nbt.get_bool("FlippedRotation")
                .unwrap_or(DEFAULT_FLIPPED_ROTATION),
            Ordering::Relaxed,
        );
        self.vehicle.entity.first_tick.store(
            nbt.get_bool("HasTicked").unwrap_or(false),
            Ordering::Relaxed,
        );
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.read_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.read_nbt(&self.vehicle.entity, nbt),
            MinecartKind::Furnace(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    fn move_entity(&self, caller: &dyn EntityBase, movement: Vector3<f64>) {
        self.move_with_destination_effects(caller, movement);
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.vehicle.entity;
        let world = entity.world.load_full();
        let previous_position = entity.pos.load();
        let previous_yaw = entity.yaw.load();
        let motion_epoch = entity.vehicle_motion_epoch();
        let still_current = |position| {
            !entity.is_removed()
                && entity.pos.load() == position
                && entity.vehicle_motion_epoch() == motion_epoch
                && Arc::ptr_eq(&world, &entity.world.load())
        };
        self.vehicle.tick();
        if !still_current(previous_position) {
            return;
        }
        entity.update_last_pos();
        if let MinecartKind::CommandBlock(minecart) = &self.kind {
            minecart.step_tick();
        }
        entity.check_out_of_world(caller);
        if !still_current(previous_position) {
            return;
        }
        self.apply_classic_gravity();
        let rail = Rail::at(&world, entity.pos.load());
        self.on_rails.store(rail.is_some(), Ordering::Relaxed);
        if let Some(rail) = rail {
            if !self.move_along_classic_rail(caller, &rail) {
                return;
            }
            let position = entity.pos.load();
            if let Some(powered) = rail.activator_power() {
                self.activate_rail(powered);
            }
            if !still_current(position) {
                return;
            }
        } else if !self.come_off_track(caller) {
            return;
        }

        // The first destination pass runs as part of move; the second follows activation.
        if !entity.apply_vehicle_destination_block_effects(caller) {
            return;
        }
        let position = entity.pos.load();
        self.update_classic_rotation(previous_position, previous_yaw);
        let moved = (position - previous_position).horizontal_length_squared() > 1.0e-6;
        self.tick_contacts(moved, || still_current(position));
        if !still_current(position) {
            return;
        }

        // Update fluid flags and currents after contacts. The destination passes above already
        // dispatch fluid effects, and AbstractMinecart.tick adds one final lava hit.
        entity.update_fluid_interaction(caller);
        if !still_current(position) {
            return;
        }
        if !entity.first_tick.load(Ordering::Relaxed)
            && entity.touching_lava.load(Ordering::Relaxed)
            && !entity.is_fire_immune()
        {
            caller.set_on_fire_for(15.0);
            if !still_current(position) {
                return;
            }
            hurt_vehicle_in_lava(caller);
            if !still_current(position) {
                return;
            }
        }
        entity.first_tick.store(false, Ordering::Relaxed);

        match &self.kind {
            MinecartKind::Furnace(minecart) => minecart.tick(entity),
            MinecartKind::Hopper(minecart) => minecart.tick(entity),
            MinecartKind::Spawner(minecart) => minecart.tick(entity),
            MinecartKind::Tnt(minecart)
                if !minecart.tick(self) && entity.horizontal_collision.load(Ordering::Relaxed) =>
            {
                let speed_squared = entity.velocity.load().horizontal_length_squared();
                if speed_squared >= f64::from(0.01f32) {
                    minecart.explode(self, speed_squared, None);
                }
            }
            _ => {}
        }
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

    fn is_pushable(&self) -> bool {
        true
    }

    #[allow(clippy::too_many_lines)]
    fn push(&self, entity: &dyn EntityBase) {
        let self_entity = self.get_entity();
        let other_entity = entity.get_entity();

        if self_entity.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
        {
            return;
        }

        {
            let passengers = self_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if passengers
                .iter()
                .any(|p| p.get_entity().entity_id == other_entity.entity_id)
            {
                return;
            }
        }
        {
            let passengers = other_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if passengers
                .iter()
                .any(|p| p.get_entity().entity_id == self_entity.entity_id)
            {
                return;
            }
        }

        let delta = other_entity.pos.load() - self_entity.pos.load();
        let horizontal = Vector3::new(delta.x, 0.0, delta.z);
        let distance_squared = horizontal.length_squared();
        if distance_squared >= f64::from(1.0e-4f32) {
            let distance = distance_squared.sqrt();
            let direction = horizontal / distance;
            let impulse = direction * (1.0 / distance).min(1.0) * f64::from(0.1f32) * 0.5;
            let (xa, za) = (impulse.x, impulse.z);

            if other_entity
                .entity_type
                .has_tag(&tag::EntityType::C_MINECARTS)
            {
                let angle = self_entity.yaw.load().to_radians();
                let facing = Vector3::new(
                    f64::from(pumpkin_util::math::cos(angle)),
                    0.0,
                    f64::from(pumpkin_util::math::sin(angle)),
                )
                .normalize();
                if direction.dot(&facing).abs() >= f64::from(0.8f32) {
                    let vel = self_entity.velocity.load();
                    let ovel = other_entity.velocity.load();

                    let is_self_furnace =
                        self_entity.entity_type.id == EntityType::FURNACE_MINECART.id;
                    let is_other_furnace =
                        other_entity.entity_type.id == EntityType::FURNACE_MINECART.id;

                    let receive_impulse = |velocity: Vector3<f64>, x, z| {
                        let mut velocity = velocity.multiply(0.2, 1.0, 0.2);
                        velocity.x += x;
                        velocity.z += z;
                        velocity
                    };
                    let (new_self_vel, new_other_vel) = match (is_self_furnace, is_other_furnace) {
                        (false, true) => (
                            receive_impulse(vel, ovel.x - xa, ovel.z - za),
                            ovel.multiply(0.95, 1.0, 0.95),
                        ),
                        (true, false) => (
                            vel.multiply(0.95, 1.0, 0.95),
                            receive_impulse(ovel, vel.x + xa, vel.z + za),
                        ),
                        _ => {
                            #[allow(clippy::manual_midpoint)]
                            let xdd = (ovel.x + vel.x) / 2.0;
                            #[allow(clippy::manual_midpoint)]
                            let zdd = (ovel.z + vel.z) / 2.0;
                            (
                                receive_impulse(vel, xdd - xa, zdd - za),
                                receive_impulse(ovel, xdd + xa, zdd + za),
                            )
                        }
                    };

                    // Commit both impulses before tracking exposes either cart to packet observers.
                    self_entity.velocity.store(new_self_vel);
                    other_entity.velocity.store(new_other_vel);
                    // Java's push requests motion sync. A mixed pair only pushes the ordinary cart.
                    if !is_self_furnace || is_other_furnace {
                        self_entity.velocity_dirty.store(true, Ordering::SeqCst);
                    }
                    if !is_other_furnace || is_self_furnace {
                        other_entity.velocity_dirty.store(true, Ordering::SeqCst);
                    }
                }
            } else {
                // Vanilla `AbstractMinecart.push`
                self_entity.push_by(Vector3::new(-xa, 0.0, -za));
                other_entity.push_by(Vector3::new(xa / 4.0, 0.0, za / 4.0));
            }
        }
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn init_data_tracker(&self) {
        self.vehicle.send_wobble_metadata();
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => {
                minecart.init_data_tracker(&self.vehicle.entity);
            }
            MinecartKind::Furnace(minecart) => {
                minecart.init_data_tracker(&self.vehicle.entity);
            }
            _ => {}
        }
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
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
        let creative = source
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);

        if let MinecartKind::Tnt(minecart) = &self.kind
            && let Some(projectile) = source
            && is_abstract_arrow(projectile.get_entity().entity_type)
            && projectile.get_entity().is_on_fire()
        {
            let world = self.vehicle.entity.world.load();
            let cause =
                cause.and_then(|cause| world.get_entity_by_id(cause.get_entity().entity_id));
            minecart.explode(
                self,
                projectile.get_entity().velocity.load().length_squared(),
                cause,
            );
            if self.vehicle.entity.is_removed() {
                return true;
            }
        }

        let will_break = self.vehicle.entity.is_alive()
            && (creative || self.vehicle.get_damage() + amount * 10.0 > 40.0);

        if let MinecartKind::Tnt(minecart) = &self.kind
            && will_break
            && !creative
        {
            let velocity = self.vehicle.entity.velocity.load();
            let speed_squared = velocity.x.mul_add(velocity.x, velocity.z * velocity.z);
            let ignites = damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE)
                || damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION)
                || self.vehicle.entity.fire_ticks.load(Ordering::Relaxed) > 0;
            if ignites || speed_squared >= 0.01 {
                self.vehicle.apply_damage_wobble(amount);
                let fuse = rand::rng().random_range(0..20) + rand::rng().random_range(0..20);
                if self
                    .vehicle
                    .entity
                    .world
                    .load()
                    .level_info
                    .load()
                    .game_rules
                    .tnt_explodes
                {
                    minecart.prime(&self.vehicle.entity, fuse);
                } else {
                    minecart.set_fuse(fuse);
                }
                return true;
            }
        }

        let damaged = self.vehicle.damage_with_context(amount, source);

        if will_break && !creative && self.vehicle.entity.is_removed() {
            let world = self.vehicle.entity.world.load();
            if world.level_info.load().game_rules.entity_drops {
                let position = self.vehicle.entity.block_pos.load();
                if let Some(container) = self.container()
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
                let mut stack = ItemStack::new(1, self.drop_item());
                if let Some(custom_name) = self.vehicle.entity.custom_name.load().as_ref() {
                    stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                        name: custom_name.clone(),
                    });
                }
                world.drop_stack(&position, stack);
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        match &self.kind {
            MinecartKind::Chest(minecart) => {
                minecart.interact(&self.vehicle.entity, player);
                true
            }
            MinecartKind::CommandBlock(_) => CommandBlockMinecart::interact(player),
            MinecartKind::Furnace(minecart) => {
                minecart.interact(&self.vehicle.entity, player, item_stack)
            }
            MinecartKind::Hopper(minecart) => {
                minecart.interact(&self.vehicle.entity, player);
                true
            }
            MinecartKind::Rideable(_) => RideableMinecart::interact(&self.vehicle.entity, player),
            MinecartKind::Spawner(_) | MinecartKind::Tnt(_) | MinecartKind::Other => false,
        }
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::{Block, block_properties::PoweredRailLikeProperties};
    use pumpkin_util::text::TextComponent;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn powered_activator_ejects_a_vehicle_passenger_without_holding_the_mount_lock()
    -> TestResult {
        use crate::{data::VanillaData, entity::passive::pig::PigEntity};
        use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
        use pumpkin_util::math::vector2::Vector2;
        use pumpkin_world::chunk::ChunkData;
        use std::time::Duration;

        let folder = tempfile::tempdir()?;
        let basic = BasicConfiguration {
            default_level_name: folder.path().join("world").to_string_lossy().into_owned(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            ..Default::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.bedrock.online_mode = false;
        let server = Server::new(
            basic,
            advanced,
            TelemetryConfig::default(),
            VanillaData::default(),
            vec![],
        )
        .await?;
        let world = server.worlds.load().first().cloned().ok_or("No world")?;
        let chunk = ChunkData::empty_sync(0, 0);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), chunk.clone());
        let mut rail = PoweredRailLikeProperties::default(&Block::ACTIVATOR_RAIL);
        rail.powered = true;
        chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::ACTIVATOR_RAIL));
        chunk.set_block_absolute_y(8, 63, 7, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
        let pos = Vector3::new(8.5, 64.0, 8.5);
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            pos,
            &EntityType::MINECART,
        )));
        let rider = PigEntity::new(Entity::new(world.clone(), pos, &EntityType::PIG));
        world.spawn_entity(cart.clone());
        world.spawn_entity(rider.clone());
        let cart_entity = cart.get_entity();
        let rider_entity = rider.get_entity();
        cart_entity.add_passenger(cart.clone(), rider.clone());
        assert!(cart_entity.has_passenger(rider_entity.entity_id));
        assert!(rider_entity.has_vehicle());

        let (finished, tick) = tokio::sync::oneshot::channel();
        let tick_cart = cart.clone();
        let thread = std::thread::spawn(move || {
            tick_cart.tick(tick_cart.as_ref(), &server);
            let _ = finished.send(());
        });
        tokio::time::timeout(Duration::from_secs(5), tick).await??;
        thread.join().map_err(|_| "Minecart tick panicked")?;

        assert!(!cart_entity.has_passenger(rider_entity.entity_id));
        assert!(!rider_entity.has_vehicle());
        assert_eq!(rider_entity.pos.load(), Vector3::new(8.5, 64.0, 7.5));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn command_block_minecart_nbt_and_activation_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COMMAND_BLOCK_MINECART,
        ));

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
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SPAWNER_MINECART,
        ));

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

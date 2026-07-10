use std::any::Any;
use std::array::from_fn;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use pumpkin_protocol::java::server::play::SPlayerInput;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
        player::Player,
    },
    server::Server,
    world::loot::fill_chest_inventory,
};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, PoweredRailLikeProperties};
use pumpkin_data::chest_loot_table::get_chest_loot_table;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, split_stack};
use tokio::sync::Mutex;

use crate::entity::vehicle::vehicle::VehicleEntity;

const fn get_exits(
    shape: pumpkin_data::block_properties::RailShape,
) -> (Vector3<f64>, Vector3<f64>) {
    use pumpkin_data::block_properties::RailShape;
    match shape {
        RailShape::NorthSouth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        RailShape::EastWest => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::AscendingEast => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 0.0)),
        RailShape::AscendingWest => (Vector3::new(1.0, 0.0, 0.0), Vector3::new(-1.0, 1.0, 0.0)),
        RailShape::AscendingNorth => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, -1.0)),
        RailShape::AscendingSouth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 1.0, 1.0)),
        RailShape::SouthEast => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::SouthWest => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthWest => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthEast => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(1.0, 0.0, 0.0)),
    }
}

pub struct MinecartEntity {
    pub vehicle: VehicleEntity,
    pub active: AtomicBool,
    pub tnt_fuse: AtomicI32,
    container: Option<Arc<MinecartInventory>>,
}

impl MinecartEntity {
    pub fn new(entity: Entity) -> Self {
        let container = (entity.entity_type.id == EntityType::CHEST_MINECART.id)
            .then(|| Arc::new(MinecartInventory::new()));
        Self {
            vehicle: VehicleEntity::new(entity),
            active: AtomicBool::new(false),
            tnt_fuse: AtomicI32::new(-1),
            container,
        }
    }
}

struct MinecartInventory {
    items: [Arc<Mutex<ItemStack>>; Self::SIZE],
    loot_table: StdMutex<Option<(String, i64)>>,
    drops_claimed: AtomicBool,
}

impl MinecartInventory {
    const SIZE: usize = 27;

    fn new() -> Self {
        Self {
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            loot_table: StdMutex::new(None),
            drops_claimed: AtomicBool::new(false),
        }
    }

    fn claim_drops(&self) -> bool {
        !self.drops_claimed.swap(true, Ordering::AcqRel)
    }

    fn read_nbt(&self, nbt: &NbtCompound) {
        let loot_table = nbt.get_string("LootTable").map(|loot_table| {
            (
                loot_table.to_owned(),
                nbt.get_long("LootTableSeed").unwrap_or(0),
            )
        });
        let has_loot_table = loot_table.is_some();
        *self.loot_table.lock().expect("Loot table mutex poisoned") = loot_table;

        if !has_loot_table {
            self.read_data(nbt, &self.items);
        }
    }

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        let loot_table = self
            .loot_table
            .lock()
            .expect("Loot table mutex poisoned")
            .clone();
        if let Some((loot_table, seed)) = loot_table {
            nbt.put_string("LootTable", loot_table);
            if seed != 0 {
                nbt.put_long("LootTableSeed", seed);
            }
        } else {
            self.write_inventory_nbt(nbt, true).await;
        }
    }

    fn has_loot_table(&self) -> bool {
        self.loot_table
            .lock()
            .expect("Loot table mutex poisoned")
            .is_some()
    }

    async fn unpack_loot(self: &Arc<Self>) {
        let loot_table = self
            .loot_table
            .lock()
            .expect("Loot table mutex poisoned")
            .take();
        let Some((loot_table, seed)) = loot_table else {
            return;
        };
        let Some(table) = get_chest_loot_table(&loot_table) else {
            *self.loot_table.lock().expect("Loot table mutex poisoned") = Some((loot_table, seed));
            return;
        };

        let inventory: Arc<dyn Inventory> = self.clone();
        fill_chest_inventory(&inventory, table, seed).await;
    }
}

impl Inventory for MinecartInventory {
    fn size(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.items {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            std::mem::swap(&mut removed, &mut *self.items[slot].lock().await);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for MinecartInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.items {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

struct MinecartScreenFactory {
    inventory: Arc<MinecartInventory>,
    title: TextComponent,
}

impl ScreenHandlerFactory for MinecartScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let inventory: Arc<dyn Inventory> = self.inventory.clone();
            let handler = create_generic_9x3(sync_id, player_inventory, inventory).await;
            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        self.title.clone()
    }
}

impl NBTStorage for MinecartEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.vehicle.entity.write_nbt(nbt).await;
            if let Some(container) = &self.container {
                container.write_nbt(nbt).await;
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.vehicle.entity.read_nbt_non_mut(nbt).await;
            if let Some(container) = &self.container {
                container.read_nbt(nbt);
            }
        })
    }
}

impl EntityBase for MinecartEntity {
    #[allow(clippy::too_many_lines)]
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.vehicle.tick();

            let world = self.vehicle.entity.world.load();
            let pos = self.vehicle.entity.pos.load();
            let mut block_pos = BlockPos(Vector3::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            ));

            let mut block = world.get_block(&block_pos);
            let mut state_id = world.get_block_state_id(&block_pos);

            let mut is_powered_rail = block.id == Block::POWERED_RAIL.id;
            let mut is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
            let mut is_on_rails = is_powered_rail
                || is_activator_rail
                || block.id == Block::RAIL.id
                || block.id == Block::DETECTOR_RAIL.id;

            // If not on rails at current Y level, check the block directly below
            if !is_on_rails {
                let below_block_pos = BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y - 1,
                    block_pos.0.z,
                ));
                let below_block = world.get_block(&below_block_pos);
                if below_block.id == Block::RAIL.id
                    || below_block.id == Block::POWERED_RAIL.id
                    || below_block.id == Block::DETECTOR_RAIL.id
                    || below_block.id == Block::ACTIVATOR_RAIL.id
                {
                    block_pos = below_block_pos;
                    block = below_block;
                    state_id = world.get_block_state_id(&block_pos);
                    is_powered_rail = block.id == Block::POWERED_RAIL.id;
                    is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
                    is_on_rails = true;
                }
            }

            if is_powered_rail || is_activator_rail {
                let props = PoweredRailLikeProperties::from_state_id(state_id, block);
                let powered = props.powered;

                self.active.store(powered, Ordering::Relaxed);

                if powered {
                    if is_powered_rail {
                        let mut velocity = self.vehicle.entity.velocity.load();
                        let speed = velocity.length();
                        if speed > 0.01 {
                            let new_speed = (speed + 0.06).min(0.4);
                            velocity = velocity
                                .normalize()
                                .multiply(new_speed, new_speed, new_speed);
                            self.vehicle.entity.velocity.store(velocity);
                        } else {
                            let yaw = self.vehicle.entity.yaw.load();
                            let push_dir = Vector3::new(
                                -f64::from((yaw.to_radians()).sin()),
                                0.0,
                                f64::from((yaw.to_radians()).cos()),
                            );
                            self.vehicle
                                .entity
                                .velocity
                                .store(push_dir.multiply(0.1, 0.1, 0.1));
                        }
                        self.vehicle.entity.send_velocity();
                    } else if is_activator_rail {
                        let passengers = self.vehicle.entity.passengers.lock().await.clone();
                        for passenger in passengers {
                            let passenger_id = passenger.get_entity().entity_id;
                            self.vehicle.entity.remove_passenger(passenger_id).await;
                        }

                        if self.vehicle.entity.entity_type.id == EntityType::TNT_MINECART.id {
                            let fuse = self.tnt_fuse.load(Ordering::Relaxed);
                            if fuse == -1 {
                                self.tnt_fuse.store(80, Ordering::Relaxed);
                                world.play_sound(
                                    pumpkin_data::sound::Sound::EntityTntPrimed,
                                    pumpkin_data::sound::SoundCategory::Blocks,
                                    &pos,
                                );
                            }
                        } else if self.vehicle.get_hurt_time() == 0 {
                            self.vehicle.set_hurt_dir(-self.vehicle.get_hurt_dir());
                            self.vehicle.set_hurt_time(10);
                            self.vehicle.set_damage(50.0);
                            // TODO: Send entity status
                        }
                    }
                } else if is_powered_rail {
                    let mut velocity = self.vehicle.entity.velocity.load();
                    velocity = velocity.multiply(0.5, 0.5, 0.5);
                    if velocity.length() < 0.01 {
                        velocity = Vector3::new(0.0, 0.0, 0.0);
                    }
                    self.vehicle.entity.velocity.store(velocity);
                    self.vehicle.entity.send_velocity();
                }
            }

            if self.vehicle.entity.entity_type.id == EntityType::TNT_MINECART.id {
                let fuse = self.tnt_fuse.load(Ordering::Relaxed);
                if fuse > 0 {
                    let new_fuse = fuse - 1;
                    self.tnt_fuse.store(new_fuse, Ordering::Relaxed);

                    if new_fuse % 2 == 0 {
                        world.spawn_particle(
                            pos,
                            Vector3::new(0.0, 0.0, 0.0),
                            0.0,
                            1,
                            pumpkin_data::particle::Particle::Smoke,
                        );
                    }

                    if new_fuse == 0 {
                        self.vehicle.entity.remove().await;
                        world.explode(pos, 4.0).await;
                    }
                }
            }

            let mut velocity = self.vehicle.entity.velocity.load();

            let mut has_driver = false;
            let mut driver_input = 0;
            let mut driver_yaw = 0.0f32;

            {
                let passengers = self.vehicle.entity.passengers.lock().await;
                if let Some(passenger) = passengers.first()
                    && let Some(player) = passenger.get_player()
                {
                    driver_input = player.last_input.load(Ordering::Relaxed);
                    driver_yaw = player.get_entity().yaw.load();
                    has_driver = true;
                }
            }

            if has_driver && is_on_rails {
                let forward = driver_input & SPlayerInput::FORWARD != 0;
                let backward = driver_input & SPlayerInput::BACKWARD != 0;

                let mut force_dir = Vector3::new(0.0, 0.0, 0.0);
                if forward {
                    let yaw_rad = f64::from(driver_yaw).to_radians();
                    force_dir.x = -yaw_rad.sin();
                    force_dir.z = yaw_rad.cos();
                } else if backward {
                    let yaw_rad = f64::from(driver_yaw).to_radians();
                    force_dir.x = yaw_rad.sin();
                    force_dir.z = -yaw_rad.cos();
                }

                if forward || backward {
                    velocity.x += force_dir.x * 0.02;
                    velocity.z += force_dir.z * 0.02;

                    let speed = velocity.x.hypot(velocity.z);
                    if speed > 0.15 {
                        #[allow(clippy::suboptimal_flops)]
                        let old_speed = self
                            .vehicle
                            .entity
                            .velocity
                            .load()
                            .x
                            .hypot(self.vehicle.entity.velocity.load().z);

                        let max_speed = old_speed.clamp(0.15, 0.4);
                        if speed > max_speed {
                            velocity.x = (velocity.x / speed) * max_speed;
                            velocity.z = (velocity.z / speed) * max_speed;
                        }
                    }
                    self.vehicle.entity.velocity.store(velocity);
                    self.vehicle.entity.send_velocity();
                }
            }

            let mut velocity = self.vehicle.entity.velocity.load();

            if is_on_rails {
                use pumpkin_data::block_properties::RailLikeProperties;
                use pumpkin_data::block_properties::{RailShape, RailShapeStraight};

                let shape = if block.id == Block::RAIL.id {
                    let props = RailLikeProperties::from_state_id(state_id, block);
                    props.shape
                } else {
                    let props = PoweredRailLikeProperties::from_state_id(state_id, block);
                    match props.shape {
                        RailShapeStraight::NorthSouth => RailShape::NorthSouth,
                        RailShapeStraight::EastWest => RailShape::EastWest,
                        RailShapeStraight::AscendingEast => RailShape::AscendingEast,
                        RailShapeStraight::AscendingWest => RailShape::AscendingWest,
                        RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
                        RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
                    }
                };

                let pos = self.vehicle.entity.pos.load();
                let block_center_bottom = Vector3::new(
                    f64::from(block_pos.0.x) + 0.5,
                    f64::from(block_pos.0.y),
                    f64::from(block_pos.0.z) + 0.5,
                );

                let (exit0, exit1) = get_exits(shape);
                let exit0 = exit0.multiply(0.5, 0.5, 0.5);
                let exit1 = exit1.multiply(0.5, 0.5, 0.5);

                let in_corner = exit0.x != exit1.x && exit0.z != exit1.z;
                let mut target_position = pos;

                if in_corner {
                    let from0to1 = exit1 - exit0;
                    let from0topos = pos - block_center_bottom - exit0;
                    let dot_num = from0to1.dot(&from0topos);
                    let dot_den = from0to1.dot(&from0to1);
                    if dot_den != 0.0 {
                        let travel_vector_from0 = from0to1.multiply(
                            dot_num / dot_den,
                            dot_num / dot_den,
                            dot_num / dot_den,
                        );
                        target_position = block_center_bottom.add(&exit0).add(&travel_vector_from0);
                    }
                } else {
                    let z_snap = (exit0.x - exit1.x).abs() > 1e-5;
                    let x_snap = (exit0.z - exit1.z).abs() > 1e-5;
                    if x_snap {
                        target_position.x = block_center_bottom.x;
                    }
                    if z_snap {
                        target_position.z = block_center_bottom.z;
                    }
                }

                target_position.y = pos.y;
                self.vehicle.entity.pos.store(target_position);

                let horizontal_in_direction = Vector3::new(exit1.x, 0.0, exit1.z);
                let mut horizontal_out_direction = Vector3::new(exit0.x, 0.0, exit0.z);

                if velocity.dot(&horizontal_out_direction) < velocity.dot(&horizontal_in_direction)
                {
                    horizontal_out_direction = horizontal_in_direction;
                }

                let out_position = block_center_bottom.add(&horizontal_out_direction).add(
                    &horizontal_out_direction
                        .normalize()
                        .multiply(1e-5, 1e-5, 1e-5),
                );

                let mut towards_out = out_position - target_position;
                towards_out.y = 0.0;
                let towards_length = towards_out.length();
                if towards_length > 1e-5 {
                    towards_out = towards_out.normalize();
                    let speed = velocity.length();
                    velocity = towards_out.multiply(speed, speed, speed);
                }

                self.vehicle.entity.velocity.store(velocity);
            }

            if velocity.length() > 0.001 {
                self.move_entity(caller, velocity).await;
                let new_pos = self.vehicle.entity.pos.load();

                let passengers = self.vehicle.entity.passengers.lock().await;
                for passenger in passengers.iter() {
                    passenger.get_entity().set_pos(new_pos);
                }
                drop(passengers);

                self.vehicle.entity.send_pos_rot();

                #[allow(clippy::useless_let_if_seq)]
                let mut friction = 0.98; // Default off-rail air drag

                if is_on_rails {
                    let passengers = self.vehicle.entity.passengers.lock().await;
                    let has_passengers = !passengers.is_empty();
                    drop(passengers);
                    friction = if has_passengers { 0.99 } else { 0.96 };
                } else {
                    let below_block_pos = BlockPos(Vector3::new(
                        block_pos.0.x,
                        block_pos.0.y - 1,
                        block_pos.0.z,
                    ));
                    let below_block = world.get_block(&below_block_pos);

                    let is_on_ground = self.vehicle.entity.on_ground.load(Ordering::Relaxed)
                        || (below_block.id != Block::AIR.id
                            && below_block.id != Block::WATER.id
                            && below_block.id != Block::LAVA.id);
                    let is_in_water = self.vehicle.entity.touching_water.load(Ordering::Relaxed)
                        || below_block.id == Block::WATER.id;

                    if is_on_ground {
                        friction = 0.5;
                    } else if is_in_water {
                        friction = 0.95;
                    }
                }

                let mut next_vel = velocity.multiply(friction, friction, friction);
                if next_vel.length() < 0.005 {
                    next_vel = Vector3::new(0.0, 0.0, 0.0);
                }
                self.vehicle.entity.velocity.store(next_vel);
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn is_pushable(&self) -> bool {
        true
    }

    #[allow(clippy::too_many_lines)]
    fn push<'a>(&'a self, entity: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let self_entity = self.get_entity();
            let other_entity = entity.get_entity();

            if self_entity.no_clip.load(Ordering::Relaxed)
                || other_entity.no_clip.load(Ordering::Relaxed)
            {
                return;
            }

            {
                let passengers = self_entity.passengers.lock().await;
                if passengers
                    .iter()
                    .any(|p| p.get_entity().entity_id == other_entity.entity_id)
                {
                    return;
                }
            }
            {
                let passengers = other_entity.passengers.lock().await;
                if passengers
                    .iter()
                    .any(|p| p.get_entity().entity_id == self_entity.entity_id)
                {
                    return;
                }
            }

            let mut xa = other_entity.pos.load().x - self_entity.pos.load().x;
            let mut za = other_entity.pos.load().z - self_entity.pos.load().z;
            let mut dd = xa * xa + za * za;
            if dd >= 1.0E-4 {
                dd = dd.sqrt();
                xa /= dd;
                za /= dd;
                let mut pow = 1.0 / dd;
                if pow > 1.0 {
                    pow = 1.0;
                }
                xa *= pow;
                za *= pow;
                xa *= 0.1;
                za *= 0.1;
                xa *= 0.5;
                za *= 0.5;

                let is_other_minecart = other_entity.entity_type.id == EntityType::MINECART.id
                    || other_entity.entity_type.id == EntityType::CHEST_MINECART.id
                    || other_entity.entity_type.id == EntityType::COMMAND_BLOCK_MINECART.id
                    || other_entity.entity_type.id == EntityType::FURNACE_MINECART.id
                    || other_entity.entity_type.id == EntityType::HOPPER_MINECART.id
                    || other_entity.entity_type.id == EntityType::SPAWNER_MINECART.id
                    || other_entity.entity_type.id == EntityType::TNT_MINECART.id;

                if is_other_minecart {
                    let xo = self_entity.velocity.load().x;
                    let zo = self_entity.velocity.load().z;

                    let dir = Vector3::new(xo, 0.0, zo).normalize();
                    let facing = Vector3::new(
                        f64::from(self_entity.yaw.load().to_radians().cos()),
                        0.0,
                        f64::from(self_entity.yaw.load().to_radians().sin()),
                    )
                    .normalize();

                    let dot = dir.dot(&facing).abs();
                    if dot >= 0.8 {
                        let vel = self_entity.velocity.load();
                        let ovel = other_entity.velocity.load();

                        let is_self_furnace =
                            self_entity.entity_type.id == EntityType::FURNACE_MINECART.id;
                        let is_other_furnace =
                            other_entity.entity_type.id == EntityType::FURNACE_MINECART.id;

                        if is_other_furnace && !is_self_furnace {
                            self_entity.velocity.store(vel.multiply(0.2, 1.0, 0.2));
                            let mut new_self_vel = self_entity.velocity.load();
                            new_self_vel.x += ovel.x - xa;
                            new_self_vel.z += ovel.z - za;
                            self_entity.velocity.store(new_self_vel);
                            self_entity.send_velocity();

                            other_entity.velocity.store(ovel.multiply(0.95, 1.0, 0.95));
                            other_entity.send_velocity();
                        } else if !is_other_furnace && is_self_furnace {
                            other_entity.velocity.store(ovel.multiply(0.2, 1.0, 0.2));
                            let mut new_other_vel = other_entity.velocity.load();
                            new_other_vel.x += vel.x + xa;
                            new_other_vel.z += vel.z + za;
                            other_entity.velocity.store(new_other_vel);
                            other_entity.send_velocity();

                            self_entity.velocity.store(vel.multiply(0.95, 1.0, 0.95));
                            self_entity.send_velocity();
                        } else {
                            #[allow(clippy::manual_midpoint)]
                            let xdd = (ovel.x + vel.x) / 2.0;
                            #[allow(clippy::manual_midpoint)]
                            let zdd = (ovel.z + vel.z) / 2.0;

                            self_entity.velocity.store(vel.multiply(0.2, 1.0, 0.2));
                            let mut new_self_vel = self_entity.velocity.load();
                            new_self_vel.x += xdd - xa;
                            new_self_vel.z += zdd - za;
                            self_entity.velocity.store(new_self_vel);
                            self_entity.send_velocity();

                            other_entity.velocity.store(ovel.multiply(0.2, 1.0, 0.2));
                            let mut new_other_vel = other_entity.velocity.load();
                            new_other_vel.x += xdd + xa;
                            new_other_vel.z += zdd + za;
                            other_entity.velocity.store(new_other_vel);
                            other_entity.send_velocity();
                        }
                    }
                } else {
                    if !self_entity.has_passengers().await && self.is_pushable() {
                        let mut vel = self_entity.velocity.load();
                        vel.x -= xa;
                        vel.z -= za;
                        self_entity.velocity.store(vel);
                        self_entity.send_velocity();
                    }

                    if !other_entity.has_passengers().await && entity.is_pushable() {
                        let mut vel = other_entity.velocity.load();
                        vel.x += xa / 4.0;
                        vel.z += za / 4.0;
                        other_entity.velocity.store(vel);
                        other_entity.send_velocity();
                    }
                }
            }
        })
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.vehicle.send_wobble_metadata();
        })
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        amount: f32,
        _damage_type: pumpkin_data::damage::DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let creative = source
                .and_then(EntityBase::get_player)
                .is_some_and(|player| player.gamemode.load() == GameMode::Creative);
            let will_break = self.vehicle.entity.is_alive()
                && (creative || self.vehicle.get_damage() + amount * 10.0 > 40.0);
            let damaged = self.vehicle.damage_with_context(amount, source).await;

            if will_break
                && !creative
                && self.vehicle.entity.is_removed()
                && let Some(container) = &self.container
            {
                let world = self.vehicle.entity.world.load();
                if world.level_info.load().game_rules.entity_drops && container.claim_drops() {
                    container.unpack_loot().await;
                    let inventory: Arc<dyn Inventory> = container.clone();
                    let position = self.vehicle.entity.block_pos.load();
                    world.scatter_inventory(&position, &inventory).await;
                    world
                        .drop_stack(&position, ItemStack::new(1, &Item::CHEST_MINECART))
                        .await;
                }
            }

            damaged
        })
    }

    fn interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        _item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if let Some(container) = &self.container {
                if player.is_spectator() && container.has_loot_table() {
                    return false;
                }
                if !player.is_spectator() {
                    container.unpack_loot().await;
                }

                let title = self
                    .vehicle
                    .entity
                    .custom_name
                    .load()
                    .as_ref()
                    .clone()
                    .unwrap_or_else(|| {
                        TextComponent::translate_cross(
                            translation::java::ENTITY_MINECRAFT_CHEST_MINECART,
                            translation::bedrock::ENTITY_CHEST_MINECART_NAME,
                            [],
                        )
                    });
                return player
                    .open_handled_screen(
                        &MinecartScreenFactory {
                            inventory: container.clone(),
                            title,
                        },
                        None,
                    )
                    .await
                    .is_some();
            }

            if player.get_entity().is_sneaking() {
                return false;
            }

            if !self.vehicle.entity.passengers.lock().await.is_empty() {
                return false;
            }

            if player.get_entity().has_vehicle().await {
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
                .add_passenger(vehicle, passenger as Arc<dyn EntityBase>)
                .await;

            true
        })
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self
                .vehicle
                .entity
                .passengers
                .lock()
                .await
                .iter()
                .any(|passenger| passenger.get_entity().entity_id == player.entity_id())
            {
                return;
            }

            if player.is_spectator() {
                return;
            }

            let player_pos = player.get_entity().pos.load();
            let minecart_pos = self.vehicle.entity.pos.load();

            let mut diff_x = minecart_pos.x - player_pos.x;
            let mut diff_z = minecart_pos.z - player_pos.z;

            let dist_sq = diff_x * diff_x + diff_z * diff_z;
            if dist_sq > 0.0001 {
                let dist = dist_sq.sqrt();
                diff_x /= dist;
                diff_z /= dist;

                let push_force = 0.1;
                let mut vel = self.vehicle.entity.velocity.load();
                vel.x += diff_x * push_force;
                vel.z += diff_z * push_force;

                let horizontal_speed = vel.x.hypot(vel.z);
                if horizontal_speed > 0.4 {
                    vel.x = (vel.x / horizontal_speed) * 0.4;
                    vel.z = (vel.z / horizontal_speed) * 0.4;
                }

                self.vehicle.entity.velocity.store(vel);
                self.vehicle.entity.send_velocity();
            }
        })
    }

    fn move_entity<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        motion: Vector3<f64>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let to_position = self.vehicle.entity.pos.load().add(&motion);
            self.vehicle.entity.move_entity(caller, motion).await;
            let should_continue = self.push_entities(caller).await;
            if should_continue {
                let current_pos = self.vehicle.entity.pos.load();
                let back_motion = to_position.sub(&current_pos);
                self.vehicle.entity.move_entity(caller, back_motion).await;
            }
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::MinecartInventory;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_world::inventory::Inventory;

    #[tokio::test]
    async fn deferred_mineshaft_loot_is_preserved_until_unpacked() {
        let inventory = std::sync::Arc::new(MinecartInventory::new());
        let mut source = NbtCompound::new();
        source.put_string(
            "LootTable",
            "minecraft:chests/abandoned_mineshaft".to_string(),
        );
        source.put_long("LootTableSeed", 1234);
        inventory.read_nbt(&source);

        let mut deferred = NbtCompound::new();
        inventory.write_nbt(&mut deferred).await;
        assert_eq!(
            deferred.get_string("LootTable"),
            Some("minecraft:chests/abandoned_mineshaft")
        );
        assert_eq!(deferred.get_long("LootTableSeed"), Some(1234));
        assert!(deferred.get_list("Items").is_none());

        inventory.unpack_loot().await;
        assert!(!inventory.is_empty().await);

        let mut unpacked = NbtCompound::new();
        inventory.write_nbt(&mut unpacked).await;
        assert!(unpacked.get_string("LootTable").is_none());
        assert!(unpacked.get_list("Items").is_some());
    }

    #[tokio::test]
    async fn chest_minecart_items_round_trip_through_nbt() {
        let inventory = MinecartInventory::new();
        inventory
            .set_stack(8, ItemStack::new(3, &Item::POWERED_RAIL))
            .await;

        let mut nbt = NbtCompound::new();
        inventory.write_nbt(&mut nbt).await;

        let restored = MinecartInventory::new();
        restored.read_nbt(&nbt);
        let stack = restored.get_stack(8).await;
        let stack = stack.lock().await;
        assert_eq!(stack.get_item().id, Item::POWERED_RAIL.id);
        assert_eq!(stack.item_count, 3);
    }
}

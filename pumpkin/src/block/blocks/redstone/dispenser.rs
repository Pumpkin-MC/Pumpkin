use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::entity::projectile::egg::EggEntity;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::tnt::TNTEntity;
use crate::entity::vehicle::boat::BoatEntity;

use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, FacingExt, translation};
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::dispenser::DispenserBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use tokio::sync::Mutex;

struct DispenserScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for DispenserScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_3x3(sync_id, player_inventory, self.0.clone()).await;
            let screen_handler_arc = Arc::new(Mutex::new(handler));

            Some(screen_handler_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate(translation::CONTAINER_DISPENSER, &[])
    }
}

#[pumpkin_block("minecraft:dispenser")]
pub struct DispenserBlock;

type DispenserLikeProperties = pumpkin_data::block_properties::DispenserLikeProperties;

// TNT constants
const TNT_POWER: f32 = 4.0;
const TNT_FUSE: u32 = 80;

// Projectile constants
const PROJECTILE_POWER: f32 = 1.1;
const PROJECTILE_UNCERTAINTY: f32 = 6.0;

fn triangle<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    (rng.random::<f64>() - rng.random::<f64>()).mul_add(max, min)
}

const fn to_normal(facing: Facing) -> Vector3<f64> {
    match facing {
        Facing::North => Vector3::new(0., 0., -1.),
        Facing::East => Vector3::new(1., 0., 0.),
        Facing::South => Vector3::new(0., 0., 1.),
        Facing::West => Vector3::new(-1., 0., 0.),
        Facing::Up => Vector3::new(0., 1., 0.),
        Facing::Down => Vector3::new(0., -1., 0.),
    }
}

const fn to_data3d(facing: Facing) -> i32 {
    match facing {
        Facing::North => 2,
        Facing::East => 5,
        Facing::South => 3,
        Facing::West => 4,
        Facing::Up => 1,
        Facing::Down => 0,
    }
}

impl BlockBehaviour for DispenserBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&DispenserScreenFactory(inventory), Some(*args.position))
                    .await;
            }
            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = DispenserLikeProperties::default(args.block);
            props.facing = args.player.living_entity.entity.get_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let dispenser_block_entity = DispenserBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(dispenser_block_entity))
                .await;
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let powered = block_receives_redstone_power(args.world, args.position).await
                || block_receives_redstone_power(args.world, &args.position.up()).await;
            let mut props = DispenserLikeProperties::from_state_id(
                args.world.get_block_state(args.position).await.id,
                args.block,
            );
            if powered && !props.triggered {
                args.world
                    .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal)
                    .await;
                props.triggered = true;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            } else if !powered && props.triggered {
                props.triggered = false;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await {
                let dispenser = block_entity
                    .as_any()
                    .downcast_ref::<DispenserBlockEntity>()
                    .unwrap();

                if let Some(mut item) = dispenser.get_random_slot().await {
                    let props = DispenserLikeProperties::from_state_id(
                        args.world.get_block_state(args.position).await.id,
                        args.block,
                    );

                    let success =
                        Self::dispense_item(args.world, args.position, &mut item, props.facing)
                            .await;

                    if success {
                        args.world
                            .sync_world_event(WorldEvent::DispenserDispenses, *args.position, 0)
                            .await;
                        args.world
                            .sync_world_event(
                                WorldEvent::DispenserActivated,
                                *args.position,
                                to_data3d(props.facing),
                            )
                            .await;
                    } else {
                        args.world
                            .sync_world_event(WorldEvent::DispenserFails, *args.position, 0)
                            .await;
                    }
                } else {
                    args.world
                        .sync_world_event(WorldEvent::DispenserFails, *args.position, 0)
                        .await;
                }
            }
        })
    }
}

impl DispenserBlock {
    async fn dispense_item(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let item_id = item.item.id;

        // Check for special dispenser behaviors
        match item_id {
            // TNT
            id if id == Item::TNT.id => Self::dispense_tnt(world, position, item, facing).await,
            // Fire Charge
            id if id == Item::FIRE_CHARGE.id => {
                Self::dispense_fire_charge(world, position, item, facing).await
            }
            // Flint and Steel
            id if id == Item::FLINT_AND_STEEL.id => {
                Self::dispense_flint_and_steel(world, position, item, facing).await
            }
            // Bone Meal
            id if id == Item::BONE_MEAL.id => {
                Self::dispense_bone_meal(world, position, item, facing)
            }
            // Water Bucket
            id if id == Item::WATER_BUCKET.id => {
                Self::dispense_bucket(world, position, item, facing, &Block::WATER, &Item::BUCKET)
                    .await
            }
            // Lava Bucket
            id if id == Item::LAVA_BUCKET.id => {
                Self::dispense_bucket(world, position, item, facing, &Block::LAVA, &Item::BUCKET)
                    .await
            }
            // Powder Snow Bucket
            id if id == Item::POWDER_SNOW_BUCKET.id => {
                Self::dispense_bucket(
                    world,
                    position,
                    item,
                    facing,
                    &Block::POWDER_SNOW,
                    &Item::BUCKET,
                )
                .await
            }
            // Snowball
            id if id == Item::SNOWBALL.id => {
                Self::dispense_snowball(world, position, item, facing).await
            }
            // Egg
            id if id == Item::EGG.id => Self::dispense_egg(world, position, item, facing).await,
            // Boats - all variants
            id if Self::is_boat_item(id) => {
                Self::dispense_boat(world, position, item, facing).await
            }
            // Arrows - TODO: implement arrow entity and shooting
            id if id == Item::ARROW.id
                || id == Item::SPECTRAL_ARROW.id
                || id == Item::TIPPED_ARROW.id =>
            {
                // For now, just drop like dropper until arrow entity is implemented
                Self::dispense_as_dropper(world, position, item, facing).await
            }
            // Ender Pearl - TODO: implement when ender pearl entity exists
            id if id == Item::ENDER_PEARL.id => {
                Self::dispense_as_dropper(world, position, item, facing).await
            }
            // Default: drop item like dropper
            _ => Self::dispense_as_dropper(world, position, item, facing).await,
        }
    }

    const fn is_boat_item(item_id: u16) -> bool {
        matches!(item_id,
            id if id == Item::OAK_BOAT.id ||
            id == Item::OAK_CHEST_BOAT.id ||
            id == Item::SPRUCE_BOAT.id ||
            id == Item::SPRUCE_CHEST_BOAT.id ||
            id == Item::BIRCH_BOAT.id ||
            id == Item::BIRCH_CHEST_BOAT.id ||
            id == Item::JUNGLE_BOAT.id ||
            id == Item::JUNGLE_CHEST_BOAT.id ||
            id == Item::ACACIA_BOAT.id ||
            id == Item::ACACIA_CHEST_BOAT.id ||
            id == Item::DARK_OAK_BOAT.id ||
            id == Item::DARK_OAK_CHEST_BOAT.id ||
            id == Item::MANGROVE_BOAT.id ||
            id == Item::MANGROVE_CHEST_BOAT.id ||
            id == Item::CHERRY_BOAT.id ||
            id == Item::CHERRY_CHEST_BOAT.id ||
            id == Item::PALE_OAK_BOAT.id ||
            id == Item::PALE_OAK_CHEST_BOAT.id ||
            id == Item::BAMBOO_RAFT.id ||
            id == Item::BAMBOO_CHEST_RAFT.id
        )
    }

    fn get_boat_entity_type(item_id: u16) -> &'static EntityType {
        match item_id {
            id if id == Item::OAK_BOAT.id => &EntityType::OAK_BOAT,
            id if id == Item::OAK_CHEST_BOAT.id => &EntityType::OAK_CHEST_BOAT,
            id if id == Item::SPRUCE_BOAT.id => &EntityType::SPRUCE_BOAT,
            id if id == Item::SPRUCE_CHEST_BOAT.id => &EntityType::SPRUCE_CHEST_BOAT,
            id if id == Item::BIRCH_BOAT.id => &EntityType::BIRCH_BOAT,
            id if id == Item::BIRCH_CHEST_BOAT.id => &EntityType::BIRCH_CHEST_BOAT,
            id if id == Item::JUNGLE_BOAT.id => &EntityType::JUNGLE_BOAT,
            id if id == Item::JUNGLE_CHEST_BOAT.id => &EntityType::JUNGLE_CHEST_BOAT,
            id if id == Item::ACACIA_BOAT.id => &EntityType::ACACIA_BOAT,
            id if id == Item::ACACIA_CHEST_BOAT.id => &EntityType::ACACIA_CHEST_BOAT,
            id if id == Item::DARK_OAK_BOAT.id => &EntityType::DARK_OAK_BOAT,
            id if id == Item::DARK_OAK_CHEST_BOAT.id => &EntityType::DARK_OAK_CHEST_BOAT,
            id if id == Item::MANGROVE_BOAT.id => &EntityType::MANGROVE_BOAT,
            id if id == Item::MANGROVE_CHEST_BOAT.id => &EntityType::MANGROVE_CHEST_BOAT,
            id if id == Item::CHERRY_BOAT.id => &EntityType::CHERRY_BOAT,
            id if id == Item::CHERRY_CHEST_BOAT.id => &EntityType::CHERRY_CHEST_BOAT,
            id if id == Item::PALE_OAK_BOAT.id => &EntityType::PALE_OAK_BOAT,
            id if id == Item::PALE_OAK_CHEST_BOAT.id => &EntityType::PALE_OAK_CHEST_BOAT,
            id if id == Item::BAMBOO_RAFT.id => &EntityType::BAMBOO_RAFT,
            id if id == Item::BAMBOO_CHEST_RAFT.id => &EntityType::BAMBOO_CHEST_RAFT,
            _ => unreachable!(),
        }
    }

    async fn dispense_tnt(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let facing_vec = to_normal(facing);
        let spawn_pos = position.to_f64() + facing_vec * 0.5 + Vector3::new(0.5, 0.0, 0.5);

        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::TNT);
        let tnt = Arc::new(TNTEntity::new(entity, TNT_POWER, TNT_FUSE));

        world.spawn_entity(tnt).await;
        item.decrement(1);
        true
    }

    async fn dispense_fire_charge(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        // TODO: Shoot small fireball projectile when implemented
        // For now, just place fire
        let target_pos = position.offset(facing.to_block_direction().to_offset());
        let target_state = world.get_block_state(&target_pos).await;

        if target_state.is_air() {
            world
                .set_block_state(
                    &target_pos,
                    Block::FIRE.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            item.decrement(1);
            return true;
        }

        false
    }

    async fn dispense_flint_and_steel(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let target_pos = position.offset(facing.to_block_direction().to_offset());
        let target_state = world.get_block_state(&target_pos).await;

        // Try to ignite fire
        if target_state.is_air() {
            world
                .set_block_state(
                    &target_pos,
                    Block::FIRE.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            // Apply durability damage
            if item.damage_item(1) {
                // Item broke
                item.clear();
            }
            return true;
        }

        // TODO: Light campfire, candles, TNT, nether portal, etc.

        false
    }

    fn dispense_bone_meal(
        _world: &Arc<crate::world::World>,
        _position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        _facing: Facing,
    ) -> bool {
        // TODO: Implement bone meal growth logic
        // For now, just consume and pretend it worked
        item.decrement(1);
        true
    }

    async fn dispense_bucket(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
        fluid_block: &'static Block,
        empty_bucket: &'static Item,
    ) -> bool {
        let target_pos = position.offset(facing.to_block_direction().to_offset());
        let target_state = world.get_block_state(&target_pos).await;

        // Try to place fluid
        if target_state.is_air() || target_state.replaceable() {
            world
                .set_block_state(
                    &target_pos,
                    fluid_block.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            // Replace with empty bucket
            item.item = empty_bucket;

            return true;
        }

        false
    }

    async fn dispense_snowball(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let facing_vec = to_normal(facing);
        let spawn_pos = position.to_f64() + facing_vec * 1.2 + Vector3::new(0.5, 0.5, 0.5);

        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::SNOWBALL);
        let snowball = Arc::new(SnowballEntity::new(entity).await);

        // Calculate pitch and yaw from facing
        let (pitch, yaw) = Self::facing_to_rotation(facing);

        // Set velocity
        snowball.thrown.set_velocity_from(
            &snowball.thrown.entity,
            pitch,
            yaw,
            0.0,
            PROJECTILE_POWER,
            PROJECTILE_UNCERTAINTY,
        );

        world.spawn_entity(snowball).await;
        item.decrement(1);
        true
    }

    async fn dispense_egg(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let facing_vec = to_normal(facing);
        let spawn_pos = position.to_f64() + facing_vec * 1.2 + Vector3::new(0.5, 0.5, 0.5);

        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::EGG);
        let egg = Arc::new(EggEntity::new(entity).await);

        // Calculate pitch and yaw from facing
        let (pitch, yaw) = Self::facing_to_rotation(facing);

        // Set velocity
        egg.thrown.set_velocity_from(
            &egg.thrown.entity,
            pitch,
            yaw,
            0.0,
            PROJECTILE_POWER,
            PROJECTILE_UNCERTAINTY,
        );

        world.spawn_entity(egg).await;
        item.decrement(1);
        true
    }

    const fn facing_to_rotation(facing: Facing) -> (f32, f32) {
        let (pitch, yaw) = match facing {
            Facing::North => (0.0, 180.0),
            Facing::South => (0.0, 0.0),
            Facing::West => (0.0, 90.0),
            Facing::East => (0.0, 270.0),
            Facing::Up => (-90.0, 0.0),
            Facing::Down => (90.0, 0.0),
        };
        (pitch, yaw)
    }

    async fn dispense_boat(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let facing_vec = to_normal(facing);
        let spawn_pos = position.to_f64() + facing_vec * 1.5 + Vector3::new(0.5, 0.0, 0.5);

        let entity_type = Self::get_boat_entity_type(item.item.id);
        let entity = Entity::new(world.clone(), spawn_pos, entity_type);
        let boat = Arc::new(BoatEntity::new(entity));

        world.spawn_entity(boat).await;
        item.decrement(1);
        true
    }

    async fn dispense_as_dropper(
        world: &Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        item: &mut tokio::sync::MutexGuard<'_, ItemStack>,
        facing: Facing,
    ) -> bool {
        let drop_item = item.split(1);
        let facing_vec = to_normal(facing);
        let mut spawn_pos = position.to_centered_f64().add(&(facing_vec * 0.7));
        spawn_pos.y -= match facing {
            Facing::Up | Facing::Down => 0.125,
            _ => 0.15625,
        };

        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::ITEM);
        let rd = rng().random::<f64>().mul_add(0.1, 0.2);
        let velocity = Vector3::new(
            triangle(&mut rng(), facing_vec.x * rd, 0.017_227_5 * 6.),
            triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
            triangle(&mut rng(), facing_vec.z * rd, 0.017_227_5 * 6.),
        );
        let item_entity =
            Arc::new(ItemEntity::new_with_velocity(entity, drop_item, velocity, 40).await);
        world.spawn_entity(item_entity).await;

        true
    }
}

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;

use pumpkin_data::FacingExt;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::world::WorldEvent;
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::dropper::DropperBlockEntity;
use pumpkin_world::block::entities::hopper::HopperBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use rand::{Rng, rng};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

struct DropperScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for DropperScreenFactory {
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
        TextComponent::translate("container.dropper", &[])
    }
}

#[pumpkin_block("minecraft:dropper")]
pub struct DropperBlock;

type DispenserLikeProperties = pumpkin_data::block_properties::DispenserLikeProperties;

fn triangle<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    min + (rng.random::<f64>() - rng.random::<f64>()) * max
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

impl BlockBehaviour for DropperBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&DropperScreenFactory(inventory))
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
            let dropper_block_entity = DropperBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(dropper_block_entity))
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
                let dropper = block_entity
                    .as_any()
                    .downcast_ref::<DropperBlockEntity>()
                    .unwrap();
                if let Some(mut item) = dropper.get_random_slot().await {
                    let props = DispenserLikeProperties::from_state_id(
                        args.world.get_block_state(args.position).await.id,
                        args.block,
                    );
                    if let Some(entity) = args
                        .world
                        .get_block_entity(
                            &args
                                .position
                                .offset(props.facing.to_block_direction().to_offset()),
                        )
                        .await
                        && let Some(container) = entity.get_inventory()
                    {
                        // TODO check WorldlyContainer
                        let mut is_full = true;
                        for i in 0..container.size() {
                            let bind = container.get_stack(i).await;
                            let item = bind.lock().await;
                            if item.item_count < item.get_max_stack_size() {
                                is_full = false;
                                break;
                            }
                        }
                        if is_full {
                            return;
                        }
                        //TODO WorldlyContainer
                        let backup = item.clone();
                        let one_item = item.split(1);
                        if HopperBlockEntity::add_one_item(dropper, container.as_ref(), one_item)
                            .await
                        {
                            return;
                        }
                        *item = backup;
                        return;
                    }
                    let drop_item = item.split(1);
                    let facing = to_normal(props.facing);
                    let mut position = args.position.to_centered_f64().add(&(facing * 0.7));
                    position.y -= match props.facing {
                        Facing::Up | Facing::Down => 0.125,
                        _ => 0.15625,
                    };
                    let entity = Entity::new(
                        Uuid::new_v4(),
                        args.world.clone(),
                        position,
                        &EntityType::ITEM,
                        false,
                    );
                    let rd = rng().random::<f64>() * 0.1 + 0.2;
                    let velocity = Vector3::new(
                        triangle(&mut rng(), facing.x * rd, 0.017_227_5 * 6.),
                        triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
                        triangle(&mut rng(), facing.z * rd, 0.017_227_5 * 6.),
                    );
                    let item_entity = Arc::new(
                        ItemEntity::new_with_velocity(entity, drop_item, velocity, 40).await,
                    );
                    args.world.spawn_entity(item_entity).await;
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
            }
        })
    }
}

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::{
    NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
    PlacedArgs, PumpkinBlock,
};
use crate::block::registry::BlockActionResult;
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::world::WorldEvent;
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::dropper::DropperBlockEntity;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;
use rand::{Rng, rng};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

struct DropperScreenFactory(Arc<dyn Inventory>);

#[async_trait]
impl ScreenHandlerFactory for DropperScreenFactory {
    async fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn ScreenHandler>>> {
        Some(Arc::new(Mutex::new(create_generic_3x3(
            sync_id,
            player_inventory,
            self.0.clone(),
        ))))
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

#[async_trait]
impl PumpkinBlock for DropperBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(inventory) = block_entity.get_inventory() {
                args.player
                    .open_handled_screen(&DropperScreenFactory(inventory))
                    .await;
            }
        }
        BlockActionResult::Success
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = DispenserLikeProperties::default(args.block);
        props.facing = args.player.living_entity.entity.get_facing().opposite();
        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let dropper_block_entity = DropperBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(dropper_block_entity))
            .await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.position).await;
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
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
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
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
                // TODO add item to container
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
                    EntityType::ITEM,
                    false,
                );
                let rd = rng().random::<f64>() * 0.1 + 0.2;
                let velocity = Vector3::new(
                    triangle(&mut rng(), facing.x * rd, 0.017_227_5 * 6.),
                    triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
                    triangle(&mut rng(), facing.z * rd, 0.017_227_5 * 6.),
                );
                let item_entity =
                    Arc::new(ItemEntity::new_with_velocity(entity, drop_item, velocity, 40).await);
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
    }
}

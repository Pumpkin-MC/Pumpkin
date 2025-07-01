use crate::block::BlockIsReplacing;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
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
    async fn normal_use(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        if let Some(block_entity) = world.get_block_entity(&location).await {
            if let Some(inventory) = block_entity.1.get_inventory() {
                player
                    .open_handled_screen(&DropperScreenFactory(inventory))
                    .await;
            }
        }
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        if let Some(block_entity) = world.get_block_entity(&location).await {
            if let Some(inventory) = block_entity.1.get_inventory() {
                player
                    .open_handled_screen(&DropperScreenFactory(inventory))
                    .await;
            }
        }
        BlockActionResult::Consume
    }

    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut props = DispenserLikeProperties::default(block);
        props.facing = player.living_entity.entity.get_facing().opposite();
        props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        let dropper_block_entity = DropperBlockEntity::new(*pos);
        world.add_block_entity(Arc::new(dropper_block_entity)).await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: BlockStateId,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let powered = block_receives_redstone_power(world, block_pos).await
            || block_receives_redstone_power(world, &block_pos.up()).await;
        let mut props = DispenserLikeProperties::from_state_id(
            world.get_block_state(block_pos).await.id,
            block,
        );
        if powered && !props.triggered {
            world
                .schedule_block_tick(block, *block_pos, 4, TickPriority::Normal)
                .await;
            props.triggered = true;
            world
                .set_block_state(
                    block_pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        } else if !powered && props.triggered {
            props.triggered = false;
            world
                .set_block_state(
                    block_pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
        if let Some(block_entity) = world.get_block_entity(block_pos).await {
            let dropper = block_entity
                .1
                .as_any()
                .downcast_ref::<DropperBlockEntity>()
                .unwrap();
            if let Some(mut item) = dropper.get_random_slot().await {
                let props = DispenserLikeProperties::from_state_id(
                    world.get_block_state(block_pos).await.id,
                    block,
                );
                // TODO add item to container
                let drop_item = item.split(1);
                let facing = to_normal(props.facing);
                let mut position = block_pos.to_centered_f64().add(&(facing * 0.7));
                position.y -= match props.facing {
                    Facing::Up | Facing::Down => 0.125,
                    _ => 0.15625,
                };
                let entity = Entity::new(
                    Uuid::new_v4(),
                    world.clone(),
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
                world.spawn_entity(item_entity).await;
                world
                    .sync_world_event(WorldEvent::DispenserDispenses, *block_pos, 0)
                    .await;
                world
                    .sync_world_event(
                        WorldEvent::DispenserActivated,
                        *block_pos,
                        to_data3d(props.facing),
                    )
                    .await;
            } else {
                world
                    .sync_world_event(WorldEvent::DispenserFails, *block_pos, 0)
                    .await;
            }
        }
    }
}

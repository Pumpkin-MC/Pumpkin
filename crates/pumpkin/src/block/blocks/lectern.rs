use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::entities::lectern::LecternBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, EmitsRedstonePowerArgs, GetComparatorOutputArgs,
    GetRedstonePowerArgs, NormalUseArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
    PlacedArgs, UseWithItemArgs,
};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, LecternLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockStateId, tag, translation};
use pumpkin_inventory::lectern_screen_handler::{LecternController, LecternScreenHandler};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, ScreenHandlerFuture, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

/// Bridges the screen handler back into the world: page changes emit the
/// vanilla redstone pulse and taking the book clears `has_book`.
struct LecternPageController {
    world: Arc<World>,
    position: BlockPos,
    inventory: Arc<dyn Inventory>,
}

impl LecternPageController {
    fn entity(&self) -> Option<&LecternBlockEntity> {
        self.inventory.as_any().downcast_ref::<LecternBlockEntity>()
    }
}

impl LecternController for LecternPageController {
    fn current_page(&self) -> i32 {
        self.entity()
            .map_or(0, |entity| entity.page.load(Ordering::Relaxed) as i32)
    }

    fn set_page(&self, page: i32) -> ScreenHandlerFuture<'_, ()> {
        Box::pin(async move {
            let Some(entity) = self.entity() else {
                return;
            };
            let page_count = entity.page_count().await;
            let page = page.clamp(0, (page_count - 1).max(0));
            if page == entity.page.load(Ordering::Relaxed) as i32 {
                return;
            }
            entity.page.store(page as usize, Ordering::Relaxed);
            entity.mark_dirty();
            LecternBlock::pulse(&self.world, &self.position).await;
        })
    }

    fn on_book_taken(&self) -> ScreenHandlerFuture<'_, ()> {
        Box::pin(async move {
            if let Some(entity) = self.entity() {
                entity.page.store(0, Ordering::Relaxed);
            }
            LecternBlock::set_has_book(&self.world, &self.position, false).await;
        })
    }
}

struct LecternScreenFactory {
    inventory: Arc<dyn Inventory>,
    controller: Arc<dyn LecternController>,
}

impl ScreenHandlerFactory for LecternScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        _player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler =
                LecternScreenHandler::new(sync_id, self.inventory.clone(), self.controller.clone());
            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_LECTERN,
            translation::bedrock::TILE_LECTERN_NAME
        )
    }
}

#[pumpkin_block("minecraft:lectern")]
pub struct LecternBlock;

impl LecternBlock {
    /// Vanilla pulse length of a page-turn signal, in game ticks.
    const PAGE_TURN_PULSE_TICKS: u8 = 2;

    /// The lectern strongly powers the block below it, so its neighbors need
    /// updating whenever the power or book state changes.
    async fn update_neighbors_below(world: &Arc<World>, position: &BlockPos) {
        world.update_neighbors(&position.down(), None).await;
    }

    /// Emits the vanilla page-turn redstone pulse: powered for two game ticks.
    pub(crate) async fn pulse(world: &Arc<World>, position: &BlockPos) {
        let (block, state_id) = world.get_block_and_state_id(position);
        if block != &Block::LECTERN {
            return;
        }
        let mut props = LecternLikeProperties::from_state_id(state_id, block);
        props.powered = true;
        world
            .set_block_state(position, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        Self::update_neighbors_below(world, position).await;
        world.schedule_block_tick(
            block,
            *position,
            Self::PAGE_TURN_PULSE_TICKS,
            TickPriority::Normal,
        );
        world.sync_world_event(WorldEvent::SoundPageTurn, *position, 0);
    }

    /// Sets `has_book`, dropping any pending pulse like vanilla `setHasBook`.
    pub(crate) async fn set_has_book(world: &Arc<World>, position: &BlockPos, has_book: bool) {
        let (block, state_id) = world.get_block_and_state_id(position);
        if block != &Block::LECTERN {
            return;
        }
        let mut props = LecternLikeProperties::from_state_id(state_id, block);
        props.powered = false;
        props.has_book = has_book;
        world
            .set_block_state(position, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        Self::update_neighbors_below(world, position).await;
    }
}

impl BlockBehaviour for LecternBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_entity = LecternBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(block_entity));
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = LecternLikeProperties::default(args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.to_state_id(args.block)
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let props = LecternLikeProperties::from_state_id(
                args.world.get_block_state(args.position).id,
                args.block,
            );
            if !props.has_book {
                return BlockActionResult::Pass;
            }

            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            let Some(inventory) = block_entity.get_inventory() else {
                return BlockActionResult::Pass;
            };

            args.player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::InteractWithLectern as i32,
                    1,
                )
                .await;

            let controller = Arc::new(LecternPageController {
                world: args.world.clone(),
                position: *args.position,
                inventory: inventory.clone(),
            });
            args.player
                .open_handled_screen(
                    &LecternScreenFactory {
                        inventory,
                        controller,
                    },
                    Some(*args.position),
                )
                .await;

            BlockActionResult::Success
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item_stack = &mut *args.item_stack;
            if !item_stack.item.has_tag(&tag::Item::MINECRAFT_LECTERN_BOOKS) {
                return BlockActionResult::PassToDefaultBlockAction;
            }

            let props = LecternLikeProperties::from_state_id(
                args.world.get_block_state(args.position).id,
                args.block,
            );
            if props.has_book {
                // Fall through so `normal_use` opens the reading screen.
                return BlockActionResult::PassToDefaultBlockAction;
            }

            let Some(lectern) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::PassToDefaultBlockAction;
            };
            let Some(lectern) = lectern.as_any().downcast_ref::<LecternBlockEntity>() else {
                return BlockActionResult::PassToDefaultBlockAction;
            };

            let book = item_stack.split_unless_creative(args.player.gamemode.load(), 1);
            let _ = item_stack;
            lectern.set_stack(0, book).await;

            Self::set_has_book(args.world, args.position, true).await;
            args.world
                .play_block_sound(Sound::ItemBookPut, SoundCategory::Blocks, *args.position);

            BlockActionResult::Success
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let mut props = LecternLikeProperties::from_state_id(
                args.world.get_block_state(args.position).id,
                args.block,
            );
            props.powered = false;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            Self::update_neighbors_below(args.world, args.position).await;
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let props = LecternLikeProperties::from_state_id(args.state.id, args.block);
            if props.powered { 15 } else { 0 }
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let props = LecternLikeProperties::from_state_id(args.state.id, args.block);
            if props.powered && args.direction == BlockDirection::Up {
                15
            } else {
                0
            }
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !args.moved {
                let props = LecternLikeProperties::from_state_id(args.old_state_id, args.block);
                if props.powered {
                    Self::update_neighbors_below(args.world, args.position).await;
                }
            }
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(lectern_entity) =
                    block_entity.as_any().downcast_ref::<LecternBlockEntity>()
            {
                let book = lectern_entity.remove_stack(0).await;
                if !book.is_empty() {
                    // Drop the book item
                    let entity = Entity::new(
                        args.world.clone(),
                        Vector3::new(
                            f64::from(args.position.0.x) + 0.5,
                            f64::from(args.position.0.y) + 0.5,
                            f64::from(args.position.0.z) + 0.5,
                        ),
                        &EntityType::ITEM,
                    );
                    let item_entity = ItemEntity::new(entity, book);
                    args.world.spawn_entity(Arc::new(item_entity)).await;
                }
            }
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(lectern_entity) =
                    block_entity.as_any().downcast_ref::<LecternBlockEntity>()
            {
                Some(lectern_entity.comparator_output().await)
            } else {
                Some(0)
            }
        })
    }
}

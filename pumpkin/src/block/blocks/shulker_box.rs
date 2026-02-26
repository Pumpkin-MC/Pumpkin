use std::sync::Arc;

use crate::block::{
    BlockFuture, BlockMetadata, OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs, PlayerPlacedArgs,
};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};
use crate::world::World;
use crate::world::loot::LootContextParameters;

use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{ContainerImpl, DataComponentImpl};
use pumpkin_data::item::Item;
use pumpkin_data::tag::{self};
use pumpkin_data::translation;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::shulker_box::ShulkerBoxBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

struct ShulkerBoxScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for ShulkerBoxScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone()).await;
            let screen_handler_arc = Arc::new(Mutex::new(handler));

            Some(screen_handler_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate(translation::CONTAINER_SHULKERBOX, &[])
    }
}

pub struct ShulkerBoxBlock;

impl BlockMetadata for ShulkerBoxBlock {
    fn ids() -> Box<[u16]> {
        tag::Block::MINECRAFT_SHULKER_BOXES.1.into()
    }
}

type EndRodLikeProperties = pumpkin_data::block_properties::EndRodLikeProperties;

impl BlockBehaviour for ShulkerBoxBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = EndRodLikeProperties::default(args.block);
            props.facing = args.direction.to_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn on_synced_block_event<'a>(
        &'a self,
        args: OnSyncedBlockEventArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            // On the server, we don't need the Animation steps for now, because the client is responsible for that.
            // TODO: Do not open the shulker box when it is currently closing
            args.r#type == Self::OPEN_ANIMATION_EVENT_TYPE
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let barrel_block_entity = ShulkerBoxBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(barrel_block_entity))
                .await;
        })
    }

    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                let items = if let Some(container) = args
                    .player_placed_item
                    .and_then(|stack| stack.get_data_component::<ContainerImpl>())
                {
                    let mut items = Vec::new();
                    for item_tag in &container.items {
                        if let Some(item_compound) = item_tag.extract_compound()
                            && let Some(slot) = item_compound.get_byte("Slot")
                            && let Some(item_data) = item_compound.get_compound("item")
                            && let Some(item_stack) = ItemStack::read_item_stack(item_data)
                        {
                            items.push((slot as usize, item_stack));
                        }
                    }
                    items
                } else {
                    Vec::new()
                };

                for (slot, item_stack) in items {
                    if slot < inventory.size() {
                        let slot_stack = inventory.get_stack(slot).await;
                        *slot_stack.lock().await = item_stack;
                    }
                }
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&ShulkerBoxScreenFactory(inventory), Some(*args.position))
                    .await;
            }

            BlockActionResult::Success
        })
    }
}

impl ShulkerBoxBlock {
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;

    pub fn build_container_items(items: &[(usize, ItemStack)]) -> Vec<NbtTag> {
        let mut tags = Vec::with_capacity(items.len());

        for (slot, stack) in items {
            let mut compound = NbtCompound::new();
            compound.put_byte("Slot", *slot as i8);
            let mut item_compound = NbtCompound::new();
            stack.write_item_stack(&mut item_compound);
            compound.put_component("item", item_compound);
            tags.push(NbtTag::Compound(compound));
        }

        tags
    }

    pub async fn drop_shulker_loot(
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        params: LootContextParameters,
    ) {
        let is_creative = params.broken_in_creative.unwrap_or(false);
        let inventory = params.shulker_box_inventory.unwrap_or_default();

        if is_creative && inventory.is_empty() {
            return;
        }

        let item = Item::from_id(block.item_id).unwrap_or(&Item::AIR);
        let stack = if inventory.is_empty() {
            ItemStack::new(1, item)
        } else {
            let container = ContainerImpl {
                items: Self::build_container_items(&inventory),
            };
            ItemStack::new_with_component(
                1,
                item,
                vec![(DataComponent::Container, Some(container.to_dyn()))],
            )
        };

        world.drop_stack(pos, stack).await;
    }
}

use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use std::sync::Mutex;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::crafter::CrafterBlockEntity;
use crate::block::entities::hopper::HopperBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, GetScreenHandlerFactoryArgs, NormalUseArgs,
    OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::World;
use pumpkin_data::block_properties::{CrafterLikeProperties, HorizontalFacing, Orientation};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipe_remainder::get_recipe_remainder_id;
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_inventory::crafting::crafting_screen_handler::match_crafting_recipe;
use pumpkin_inventory::generic_container_screen_handler::create_crafter_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct CrafterScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for CrafterScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler = create_crafter_3x3(sync_id, player_inventory, self.0.clone(), player);
        let screen_handler_arc = Arc::new(Mutex::new(handler));

        Some(screen_handler_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_CRAFTER,
            translation::bedrock::CONTAINER_CRAFTER
        )
    }
}

fn triangle<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    (rng.random::<f64>() - rng.random::<f64>()).mul_add(max, min)
}

const fn orientation_to_front(orientation: Orientation) -> BlockDirection {
    match orientation {
        Orientation::DownEast
        | Orientation::DownNorth
        | Orientation::DownSouth
        | Orientation::DownWest => BlockDirection::Down,
        Orientation::UpEast | Orientation::UpNorth | Orientation::UpSouth | Orientation::UpWest => {
            BlockDirection::Up
        }
        Orientation::NorthUp => BlockDirection::North,
        Orientation::SouthUp => BlockDirection::South,
        Orientation::WestUp => BlockDirection::West,
        Orientation::EastUp => BlockDirection::East,
    }
}

const fn to_normal_direction(direction: BlockDirection) -> Vector3<f64> {
    match direction {
        BlockDirection::North => Vector3::new(0., 0., -1.),
        BlockDirection::East => Vector3::new(1., 0., 0.),
        BlockDirection::South => Vector3::new(0., 0., 1.),
        BlockDirection::West => Vector3::new(-1., 0., 0.),
        BlockDirection::Up => Vector3::new(0., 1., 0.),
        BlockDirection::Down => Vector3::new(0., -1., 0.),
    }
}

const fn to_data3d(direction: BlockDirection) -> i32 {
    match direction {
        BlockDirection::Down => 0,
        BlockDirection::Up => 1,
        BlockDirection::North => 2,
        BlockDirection::South => 3,
        BlockDirection::West => 4,
        BlockDirection::East => 5,
    }
}

#[pumpkin_block("minecraft:crafter")]
pub struct CrafterBlock;

impl CrafterBlock {
    pub const CRAFTING_TICK_DELAY: u8 = 4;
    pub const MAX_CRAFTING_TICKS: i32 = 6;

    fn dispense_from(world: &Arc<World>, pos: &BlockPos, block: &Block) {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(crafter) = block_entity.as_any().downcast_ref::<CrafterBlockEntity>() else {
            return;
        };

        let recipe = match_crafting_recipe(crafter, None);
        if let Some(recipe_result) = recipe
            && let Some(item) = Item::from_registry_key(
                recipe_result
                    .item_id
                    .strip_prefix("minecraft:")
                    .unwrap_or(&recipe_result.item_id),
            )
            && recipe_result.count > 0
        {
            let state = world.get_block_state(pos);
            let mut props = CrafterLikeProperties::from_state_id(state.id);

            crafter.set_crafting_ticks_remaining(Self::MAX_CRAFTING_TICKS);
            props.crafting = true;
            world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS);

            let mut result_stack = ItemStack::new(recipe_result.count, item);
            Self::dispense_item(world, pos, crafter, &mut result_stack, props.orientation);

            for i in 0..CrafterBlockEntity::INVENTORY_SIZE {
                let stack = crafter.get_stack(i);
                if !stack.is_empty()
                    && let Some(remainder_id) = get_recipe_remainder_id(stack.item.id)
                    && let Some(remainder_item) = Item::from_id(remainder_id)
                {
                    let mut remainder_stack = ItemStack::new(1, remainder_item);
                    Self::dispense_item(
                        world,
                        pos,
                        crafter,
                        &mut remainder_stack,
                        props.orientation,
                    );
                }
            }

            for i in 0..CrafterBlockEntity::INVENTORY_SIZE {
                let mut stack = crafter.get_stack(i);
                if !stack.is_empty() {
                    stack.decrement(1);
                    crafter.set_stack(i, stack);
                }
            }
            crafter.mark_dirty();
        } else {
            world.sync_world_event(WorldEvent::SoundCrafterFail, *pos, 0);
        }
    }

    fn dispense_item(
        world: &Arc<World>,
        pos: &BlockPos,
        crafter: &CrafterBlockEntity,
        stack: &mut ItemStack,
        orientation: Orientation,
    ) {
        let direction = orientation_to_front(orientation);
        let target_pos = pos.offset(direction.to_offset());

        if let Some(target_entity) = world.get_block_entity(&target_pos)
            && let Some(target_inv) = target_entity.get_inventory()
        {
            while !stack.is_empty() {
                let one_item = stack.split(1);
                if !HopperBlockEntity::add_one_item(crafter, target_inv.as_ref(), &one_item) {
                    stack.item_count += one_item.item_count;
                    break;
                }
            }
        }

        if !stack.is_empty() {
            let drop_stack = stack.split(stack.item_count);
            let facing_normal = to_normal_direction(direction);
            let mut spawn_pos = pos.to_centered_f64().add(&(facing_normal * 0.7));
            spawn_pos.y -= match direction {
                BlockDirection::Up | BlockDirection::Down => 0.125,
                _ => 0.15625,
            };

            let entity = Entity::new(world.clone(), spawn_pos, &EntityType::ITEM);
            let rd = rng().random::<f64>().mul_add(0.1, 0.2);
            let velocity = Vector3::new(
                triangle(&mut rng(), facing_normal.x * rd, 0.017_227_5 * 6.),
                triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
                triangle(&mut rng(), facing_normal.z * rd, 0.017_227_5 * 6.),
            );

            let item_entity = Arc::new(ItemEntity::new_with_velocity(
                entity, drop_stack, velocity, 40,
            ));
            world.spawn_entity(item_entity);

            world.sync_world_event(WorldEvent::SoundCrafterCraft, *pos, 0);
            world.sync_world_event(
                WorldEvent::ParticlesShootWhiteSmoke,
                *pos,
                to_data3d(direction),
            );
        }
    }
}

impl BlockBehaviour for CrafterBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }
        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.get_inventory()?;
        Some(Box::new(CrafterScreenFactory(inventory)))
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CrafterLikeProperties::default(args.block);
        let facing = args.direction;
        let horizontal = args.player.living_entity.entity.get_horizontal_facing();
        props.orientation = match facing {
            BlockDirection::Down => match horizontal {
                HorizontalFacing::North => Orientation::DownSouth,
                HorizontalFacing::South => Orientation::DownNorth,
                HorizontalFacing::East => Orientation::DownWest,
                HorizontalFacing::West => Orientation::DownEast,
            },
            BlockDirection::Up => match horizontal {
                HorizontalFacing::North => Orientation::UpNorth,
                HorizontalFacing::South => Orientation::UpSouth,
                HorizontalFacing::East => Orientation::UpEast,
                HorizontalFacing::West => Orientation::UpWest,
            },
            BlockDirection::North => Orientation::NorthUp,
            BlockDirection::South => Orientation::SouthUp,
            BlockDirection::East => Orientation::EastUp,
            BlockDirection::West => Orientation::WestUp,
        };
        props.triggered = block_receives_redstone_power(args.world, args.position);
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let props = CrafterLikeProperties::from_state_id(state.id);
        let crafter_block_entity = CrafterBlockEntity::new(*args.position);
        crafter_block_entity.set_triggered(props.triggered);
        args.world.add_block_entity(Arc::new(crafter_block_entity));

        if props.triggered {
            args.world.schedule_block_tick(
                args.block,
                *args.position,
                Self::CRAFTING_TICK_DELAY,
                TickPriority::Normal,
            );
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let powered = block_receives_redstone_power(args.world, args.position);
        let mut props =
            CrafterLikeProperties::from_state_id(args.world.get_block_state(args.position).id);

        if powered && !props.triggered {
            props.triggered = true;
            args.world.schedule_block_tick(
                args.block,
                *args.position,
                Self::CRAFTING_TICK_DELAY,
                TickPriority::Normal,
            );
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
            if let Some(be) = args.world.get_block_entity(args.position)
                && let Some(crafter) = be.as_any().downcast_ref::<CrafterBlockEntity>()
            {
                crafter.set_triggered(true);
            }
        } else if !powered && props.triggered {
            props.triggered = false;
            props.crafting = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
            if let Some(be) = args.world.get_block_entity(args.position)
                && let Some(crafter) = be.as_any().downcast_ref::<CrafterBlockEntity>()
            {
                crafter.set_triggered(false);
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        Self::dispense_from(args.world, args.position, args.block);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position) {
            let crafter = block_entity.as_any().downcast_ref::<CrafterBlockEntity>()?;
            Some(crafter.get_redstone_signal())
        } else {
            None
        }
    }
}

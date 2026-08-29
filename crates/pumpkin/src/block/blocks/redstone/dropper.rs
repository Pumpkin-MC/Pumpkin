use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use std::sync::Mutex;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::item::ItemEntity;
use crate::entity::{Entity, EntityBase};

use crate::block::entities::dropper::DropperBlockEntity;
use crate::block::entities::hopper::HopperBlockEntity;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{FacingExt, translation};
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct DropperScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for DropperScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler = create_generic_3x3(sync_id, player_inventory, self.0.clone());
        let screen_handler_arc = Arc::new(Mutex::new(handler));

        Some(screen_handler_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_DROPPER,
            translation::bedrock::CONTAINER_DROPPER
        )
    }
}

#[pumpkin_block("minecraft:dropper")]
pub struct DropperBlock;

type DispenserLikeProperties = pumpkin_data::block_properties::DispenserLikeProperties;

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

impl BlockBehaviour for DropperBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(inventory) = block_entity.get_inventory()
        {
            args.player
                .open_handled_screen(&DropperScreenFactory(inventory), Some(*args.position));
        }
        BlockActionResult::Success
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = DispenserLikeProperties::default(args.block);
        props.facing = args.player.get_entity().get_facing().opposite();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let dropper_block_entity = DropperBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(dropper_block_entity));
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let powered = block_receives_redstone_power(args.world, args.position)
            || block_receives_redstone_power(args.world, &args.position.up());

        let mut props = DispenserLikeProperties::from_state_id(
            args.world.get_block_state(args.position).id,
            args.block,
        );

        if powered && !props.triggered {
            args.world
                .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal);
            props.triggered = true;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        } else if !powered && props.triggered {
            props.triggered = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(args.position);
        if let Some(block_entity) = args.world.get_block_entity(args.position) {
            let Some(dropper) = block_entity.as_any().downcast_ref::<DropperBlockEntity>() else {
                return;
            };

            if let Some((slot_index, mut item)) = dropper.get_random_slot() {
                let props = DispenserLikeProperties::from_state_id(state.id, block);

                let target_pos = args
                    .position
                    .offset(props.facing.to_block_direction().to_offset());

                if let Some(entity) = args.world.get_block_entity(&target_pos)
                    && let Some(container) = entity.get_inventory()
                {
                    let backup = item.clone();
                    let one_item = item.split(1);

                    if HopperBlockEntity::add_one_item(dropper, container.as_ref(), &one_item) {
                        dropper.set_stack(slot_index, item);
                        return;
                    }

                    dropper.set_stack(slot_index, backup);
                    return;
                }

                // No container found, dispense item into the world
                let drop_item = item.split(1);
                dropper.set_stack(slot_index, item);
                let facing = to_normal(props.facing);
                let mut pos = args.position.to_centered_f64().add(&(facing * 0.7));

                pos.y -= match props.facing {
                    Facing::Up | Facing::Down => 0.125,
                    _ => 0.15625,
                };

                let entity = Entity::new(args.world.clone(), pos, &EntityType::ITEM);
                let rd = rng().random::<f64>().mul_add(0.1, 0.2);

                let velocity = Vector3::new(
                    triangle(&mut rng(), facing.x * rd, 0.017_227_5 * 6.),
                    triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
                    triangle(&mut rng(), facing.z * rd, 0.017_227_5 * 6.),
                );

                let item_entity = Arc::new(ItemEntity::new_with_velocity(
                    entity, drop_item, velocity, 40,
                ));
                args.world.spawn_entity(item_entity);

                args.world
                    .sync_world_event(WorldEvent::SoundDispenserDispense, *args.position, 0);

                args.world.sync_world_event(
                    WorldEvent::ParticlesShootSmoke,
                    *args.position,
                    to_data3d(props.facing),
                );
            } else {
                args.world
                    .sync_world_event(WorldEvent::SoundDispenserDispense, *args.position, 0);
            }
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(inventory) = block_entity.get_inventory()
        {
            Some(crate::block::calculate_comparator_output(
                inventory.as_ref(),
            ))
        } else {
            None
        }
    }
}

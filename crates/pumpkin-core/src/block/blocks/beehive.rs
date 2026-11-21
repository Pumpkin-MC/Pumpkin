use std::sync::Arc;

use crate::block::blocks::campfire::CampfireBlock;
use crate::block::entities::beehive::BeehiveBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockMetadata, ExplodeArgs, GetComparatorOutputArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs, PlacedArgs, UseWithItemArgs,
};
use crate::entity::item::ItemEntity;
use crate::entity::passive::bee::BeeEntity;
use crate::entity::{Entity, EntityBase};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::plugin::api::events::player::player_harvest_block::PlayerHarvestBlockEvent;
use crate::world::World;
use pumpkin_data::block_properties::BeeNestLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

pub struct BeehiveBlock;

impl BlockMetadata for BeehiveBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BEEHIVE, BlockId::BEE_NEST].into()
    }
}

impl BeehiveBlock {
    const MAX_HONEY_LEVEL: u8 = 5;

    /// Vanilla `BeehiveBlock.angerNearbyBees`.
    pub(crate) fn anger_nearby_bees(world: &World, pos: &BlockPos) {
        let min = pos.to_f64();
        let search_box = BoundingBox::new(min, min.add_raw(1.0, 1.0, 1.0)).expand(8.0, 6.0, 8.0);
        let players = world.get_players_at_box(&search_box);
        if players.is_empty() {
            return;
        }

        let mut rng = rand::rng();
        for entity in world.get_entities_at_box(&search_box) {
            let Some(mob) = entity.get_mob() else {
                continue;
            };
            if mob.cast_any().downcast_ref::<BeeEntity>().is_none() {
                continue;
            }
            if mob.get_mob_entity().get_target().is_none() {
                let anger_target = players[rng.random_range(0..players.len())].clone();
                mob.set_mob_target(
                    Some(anger_target as Arc<dyn EntityBase>),
                    TargetReason::Angry,
                );
            }
        }
    }

    fn shear_honeycomb(args: &UseWithItemArgs<'_>, state_id: BlockStateId) -> bool {
        let drop_pos = args.position.to_centered_f64();
        let mut drops = args
            .world
            .get_loot_table("minecraft:harvest/beehive")
            .map_or_else(
                || vec![ItemStack::new(3, &Item::HONEYCOMB)],
                |handle| {
                    let params = crate::world::loot::LootContextParameters {
                        world: Some(args.world),
                        position: Some(drop_pos),
                        block_state: Some(BlockState::from_id(state_id)),
                        this_entity: Some(args.player.as_ref() as &dyn EntityBase),
                        tool: Some(args.item_stack.clone()),
                        ..Default::default()
                    };
                    crate::world::loot::generate_loot_from_handle(&handle, rand::random(), &params)
                },
            );

        let mut event = PlayerHarvestBlockEvent {
            player: args.player.clone(),
            block_pos: *args.position,
            harvested_items: drops,
            cancelled: false,
        };
        if let Some(server) = args.world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return false;
        }
        drops = event.harvested_items;

        for drop_item in drops {
            let item_entity = Arc::new(ItemEntity::new(
                Entity::new(args.world.clone(), drop_pos, &EntityType::ITEM),
                drop_item,
            ));
            args.world.spawn_entity(item_entity);
        }

        args.world.play_sound(
            Sound::BlockBeehiveShear,
            SoundCategory::Blocks,
            &args.player.position(),
        );
        args.player.damage_item_in_slot(args.equipment_slot, 1);
        args.world
            .emit_game_event(GameEvent::Shear.name(), args.position.to_centered_f64());
        true
    }

    fn collect_honey_bottle(args: &mut UseWithItemArgs<'_>) -> bool {
        let mut event = PlayerHarvestBlockEvent {
            player: args.player.clone(),
            block_pos: *args.position,
            harvested_items: vec![ItemStack::new(1, &Item::HONEY_BOTTLE)],
            cancelled: false,
        };
        if let Some(server) = args.world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return false;
        }

        args.item_stack
            .decrement_unless_creative(args.player.gamemode.load(), 1);
        args.world.play_sound(
            Sound::ItemBottleFill,
            SoundCategory::Blocks,
            &args.player.position(),
        );

        let mut inserted_into_inventory = false;
        for mut harvested in event.harvested_items {
            if args.item_stack.is_empty() {
                *args.item_stack = harvested;
            } else if args
                .player
                .get_inventory()
                .insert_stack_anywhere(&mut harvested)
            {
                inserted_into_inventory = true;
                if !harvested.is_empty() {
                    args.player.drop_item(harvested);
                }
            } else {
                args.player.drop_item(harvested);
            }
        }
        if inserted_into_inventory {
            args.player.sync_inventory_to_client();
        }

        args.world.emit_game_event(
            GameEvent::FluidPickup.name(),
            args.position.to_centered_f64(),
        );
        true
    }
}

impl BlockBehaviour for BeehiveBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(BeehiveBlockEntity::new(*args.position)));
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BeeNestLikeProperties::default(args.block);
        props.facing = args.player.get_entity().get_horizontal_facing().opposite();
        props.to_state_id(args.block)
    }

    fn use_with_item(&self, mut args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = BeeNestLikeProperties::from_state_id(state_id);
        if props.honey_level < Self::MAX_HONEY_LEVEL {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        let used_item_id = args.item_stack.item.id;
        let hive_emptied = if used_item_id == Item::SHEARS.id {
            Self::shear_honeycomb(&args, state_id)
        } else if used_item_id == Item::GLASS_BOTTLE.id {
            Self::collect_honey_bottle(&mut args)
        } else {
            false
        };

        if !hive_emptied {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        args.player
            .increment_stat(StatisticCategory::Used, i32::from(used_item_id), 1);

        if !CampfireBlock::is_smokey_pos(args.world, args.position)
            && let Some(entity) = args.world.get_block_entity(args.position)
            && let Some(hive) = entity.as_any().downcast_ref::<BeehiveBlockEntity>()
            && !hive.is_empty()
        {
            Self::anger_nearby_bees(args.world, args.position);
            hive.release_occupants(args.world, state_id, true, Some(args.player));
        }

        props.honey_level = 0;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );

        BlockActionResult::Success
    }

    fn explode(&self, args: ExplodeArgs<'_>) {
        Self::anger_nearby_bees(args.world, args.position);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if Block::from_state_id(args.neighbor_state_id) == &Block::FIRE
            && let Some(entity) = args.world.get_block_entity(args.position)
            && let Some(hive) = entity.as_any().downcast_ref::<BeehiveBlockEntity>()
        {
            hive.release_for_fire();
        }
        args.state_id
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let state_id = args.world.get_block_state_id(args.position);
        let props = BeeNestLikeProperties::from_state_id(state_id);
        Some(props.honey_level)
    }
}

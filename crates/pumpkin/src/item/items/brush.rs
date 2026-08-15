use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct BrushItem;

impl ItemMetadata for BrushItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::BRUSH.id])
    }
}

fn get_dusted_stage(block: &Block, state_id: BlockStateId) -> u8 {
    block
        .properties(state_id)
        .and_then(|props| {
            props
                .to_props()
                .into_iter()
                .find(|(key, _)| *key == "dusted")
                .and_then(|(_, val)| val.parse::<u8>().ok())
        })
        .unwrap_or(0)
}

fn set_dusted_stage(block: &Block, state_id: BlockStateId, stage: u8) -> BlockStateId {
    let Some(props) = block.properties(state_id) else {
        return state_id;
    };
    let stage_str = stage.to_string();
    let original_props = props.to_props();
    let updated_props: Vec<(&str, &str)> = original_props
        .iter()
        .map(|(key, value)| {
            if *key == "dusted" {
                ("dusted", stage_str.as_str())
            } else {
                (*key, *value)
            }
        })
        .collect();
    block.from_properties(&updated_props).to_state_id(block)
}

fn get_random_archaeology_loot() -> &'static Item {
    let loot_table = [
        &Item::ARCHER_POTTERY_SHERD,
        &Item::PRIZE_POTTERY_SHERD,
        &Item::SKULL_POTTERY_SHERD,
        &Item::ARMS_UP_POTTERY_SHERD,
        &Item::ANGLER_POTTERY_SHERD,
        &Item::EMERALD,
        &Item::DIAMOND,
        &Item::SNIFFER_EGG,
        &Item::BONE,
        &Item::GOLD_NUGGET,
        &Item::COAL,
    ];
    let idx = (rand::random::<u32>() as usize) % loot_table.len();
    loot_table[idx]
}

impl ItemBehaviour for BrushItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            player.world().play_sound(
                Sound::ItemBrushBrushingGeneric,
                SoundCategory::Players,
                &player.position(),
            );
            let stack = player.inventory().held_item().await;
            player
                .living_entity
                .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION)
                .await;
        })
    }

    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let is_sand = block == &Block::SUSPICIOUS_SAND;
            let is_gravel = block == &Block::SUSPICIOUS_GRAVEL;
            let block_center = Vector3::new(
                f64::from(location.0.x) + 0.5,
                f64::from(location.0.y) + 0.5,
                f64::from(location.0.z) + 0.5,
            );

            if is_sand || is_gravel {
                let current_state_id = world.get_block_state_id(&location);
                let current_stage = get_dusted_stage(block, current_state_id);

                if current_stage < 3 {
                    let next_stage_id =
                        set_dusted_stage(block, current_state_id, current_stage + 1);
                    world
                        .set_block_state(&location, next_stage_id, BlockFlags::NOTIFY_ALL)
                        .await;

                    world.play_sound(
                        if is_sand {
                            Sound::ItemBrushBrushingSand
                        } else {
                            Sound::ItemBrushBrushingGravel
                        },
                        SoundCategory::Blocks,
                        &block_center,
                    );
                } else {
                    let replacement_state_id = if is_sand {
                        Block::SAND.default_state.id
                    } else {
                        Block::GRAVEL.default_state.id
                    };

                    world
                        .set_block_state(&location, replacement_state_id, BlockFlags::NOTIFY_ALL)
                        .await;

                    world.play_sound(
                        if is_sand {
                            Sound::ItemBrushBrushingSandComplete
                        } else {
                            Sound::ItemBrushBrushingGravelComplete
                        },
                        SoundCategory::Blocks,
                        &block_center,
                    );

                    let loot_item = get_random_archaeology_loot();
                    let spawn_pos = Vector3::new(
                        f64::from(location.0.x) + 0.5,
                        f64::from(location.0.y) + 1.0,
                        f64::from(location.0.z) + 0.5,
                    );
                    let item_entity = Arc::new(ItemEntity::new(
                        Entity::new(world.clone(), spawn_pos, &EntityType::ITEM),
                        ItemStack::new(1, loot_item),
                    ));
                    world.spawn_entity(item_entity).await;
                }

                player.damage_held_item(1).await;
            } else {
                world.play_sound(
                    Sound::ItemBrushBrushingGeneric,
                    SoundCategory::Blocks,
                    &block_center,
                );
            }

            let stack = player.inventory().held_item().await;
            player
                .living_entity
                .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION)
                .await;
        })
    }

    fn use_on_entity<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        entity: Arc<dyn EntityBase>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let ent = entity.get_entity();
            if ent.entity_type == &EntityType::ARMADILLO {
                let world = player.world();
                world.play_sound(
                    Sound::EntityArmadilloBrush,
                    SoundCategory::Neutral,
                    &ent.pos.load(),
                );

                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), ent.pos.load(), &EntityType::ITEM),
                    ItemStack::new(1, &Item::ARMADILLO_SCUTE),
                ));
                world.spawn_entity(item_entity).await;

                player.damage_held_item(16).await;
            } else {
                let world = player.world();
                world.play_sound(
                    Sound::ItemBrushBrushingGeneric,
                    SoundCategory::Neutral,
                    &ent.pos.load(),
                );
            }

            let stack = player.inventory().held_item().await;
            player
                .living_entity
                .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION)
                .await;
        })
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BrushItem {
    pub const USE_DURATION: i32 = 200;
}

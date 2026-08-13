use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct BoneMealItem;

impl ItemMetadata for BoneMealItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::BONE_MEAL.id])
    }
}

impl ItemBehaviour for BoneMealItem {
    #[allow(clippy::too_many_lines)]
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let state_id = world.get_block_state_id(&location);

            // Compute crop age progression without holding Box<dyn BlockProperties> across await
            let crop_action = block.properties(state_id).and_then(|props| {
                let prop_map = props.to_props();
                prop_map
                    .iter()
                    .find(|(k, _)| *k == "age")
                    .and_then(|(_, age_val)| age_val.parse::<u8>().ok())
                    .and_then(|current_age| {
                        let max_age = match block.id {
                            id if id == Block::BEETROOTS.id || id == Block::SWEET_BERRY_BUSH.id => {
                                3
                            }
                            id if id == Block::TORCHFLOWER_CROP.id => 1,
                            id if id == Block::PITCHER_CROP.id => 4,
                            _ => 7,
                        };
                        (current_age < max_age).then(|| {
                            let bonus = (rand::random::<u32>() % 4 + 2) as u8;
                            let new_age = (current_age + bonus).min(max_age).to_string();
                            let new_props: Vec<(&str, &str)> = prop_map
                                .iter()
                                .map(|(k, v)| {
                                    if *k == "age" {
                                        (*k, new_age.as_str())
                                    } else {
                                        (*k, *v)
                                    }
                                })
                                .collect();
                            block.from_properties(&new_props).to_state_id(block)
                        })
                    })
            });

            if let Some(new_state_id) = crop_action {
                world
                    .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                world.play_sound(
                    Sound::ItemBoneMealUse,
                    SoundCategory::Blocks,
                    &location.to_f64(),
                );
                item.decrement_unless_creative(player.gamemode.load(), 1);
                return;
            }

            // Compute sapling stage progression without holding Box<dyn BlockProperties> across await
            let sapling_action = block.properties(state_id).and_then(|props| {
                let prop_map = props.to_props();
                prop_map
                    .iter()
                    .find(|(k, _)| *k == "stage")
                    .and_then(|(_, stage_val)| stage_val.parse::<u8>().ok())
                    .filter(|&stage| stage < 1)
                    .map(|_| {
                        let new_props: Vec<(&str, &str)> = prop_map
                            .iter()
                            .map(|(k, v)| if *k == "stage" { (*k, "1") } else { (*k, *v) })
                            .collect();
                        block.from_properties(&new_props).to_state_id(block)
                    })
            });

            if let Some(new_state_id) = sapling_action {
                world
                    .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                world.play_sound(
                    Sound::ItemBoneMealUse,
                    SoundCategory::Blocks,
                    &location.to_f64(),
                );
                item.decrement_unless_creative(player.gamemode.load(), 1);
                return;
            }

            // Handle Grass Block / Moss Block bone-mealing
            if block.id == Block::GRASS_BLOCK.id || block.id == Block::MOSS_BLOCK.id {
                let center_x = location.0.x;
                let center_y = location.0.y;
                let center_z = location.0.z;

                for dx in -2..=2 {
                    for dz in -2..=2 {
                        if dx == 0 && dz == 0 {
                            continue;
                        }
                        if rand::random::<u32>().is_multiple_of(2) {
                            let target_pos =
                                BlockPos::new(center_x + dx, center_y + 1, center_z + dz);
                            let target_below =
                                BlockPos::new(center_x + dx, center_y, center_z + dz);
                            let (below_block, _) = world.get_block_and_state_id(&target_below);
                            let (target_block, _) = world.get_block_and_state_id(&target_pos);

                            if (below_block.id == Block::GRASS_BLOCK.id
                                || below_block.id == Block::MOSS_BLOCK.id)
                                && target_block.id == Block::AIR.id
                            {
                                let plant_block = if rand::random::<u32>().is_multiple_of(8) {
                                    Block::DANDELION.default_state.id
                                } else if (rand::random::<u32>() + 1).is_multiple_of(8) {
                                    Block::POPPY.default_state.id
                                } else {
                                    Block::SHORT_GRASS.default_state.id
                                };
                                world
                                    .set_block_state(
                                        &target_pos,
                                        plant_block,
                                        BlockFlags::NOTIFY_ALL,
                                    )
                                    .await;
                            }
                        }
                    }
                }

                world.play_sound(
                    Sound::ItemBoneMealUse,
                    SoundCategory::Blocks,
                    &location.to_f64(),
                );
                item.decrement_unless_creative(player.gamemode.load(), 1);
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

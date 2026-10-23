use std::any::Any;

use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::WaterCauldronLikeProperties;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::PotionContentsImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

/// A potion holding water; the contents component is what names it and makes it
/// brewable.
#[must_use]
pub fn water_bottle() -> ItemStack {
    ItemStack::new_with_component(
        1,
        &Item::POTION,
        vec![(
            DataComponent::PotionContents,
            Some(Box::new(PotionContentsImpl {
                potion_id: Some(i32::from(Potion::WATER.id)),
                custom_color: None,
                custom_effects: Vec::new(),
                custom_name: None,
            }) as Box<_>),
        )],
    )
}

pub struct GlassBottleItem;

impl ItemMetadata for GlassBottleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::GLASS_BOTTLE.id])
    }
}

impl GlassBottleItem {
    fn turn_bottle_into_item(
        player: &Player,
        hand: pumpkin_util::Hand,
        mut held: ItemStack,
        mut stack_to_give: ItemStack,
    ) {
        let world = player.world();
        if held.item_count == 1 && player.gamemode.load() != pumpkin_util::GameMode::Creative {
            match hand {
                pumpkin_util::Hand::Right => player.inventory().set_held_item(stack_to_give),
                pumpkin_util::Hand::Left => player
                    .inventory()
                    .set_stack_in_hand(pumpkin_util::Hand::Left, stack_to_give),
            }
        } else {
            held.decrement_unless_creative(player.gamemode.load(), 1);
            match hand {
                pumpkin_util::Hand::Right => player.inventory().set_held_item(held),
                pumpkin_util::Hand::Left => player
                    .inventory()
                    .set_stack_in_hand(pumpkin_util::Hand::Left, held),
            }
            let was_added = player.inventory().insert_stack_anywhere(&mut stack_to_give);
            if !was_added && !stack_to_give.is_empty() {
                world.drop_stack(&player.position().to_block_pos(), stack_to_give);
            }
        }
    }
}

impl ItemBehaviour for GlassBottleItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let held = match hand {
            pumpkin_util::Hand::Right => player.inventory().held_item(),
            pumpkin_util::Hand::Left => player.inventory().off_hand_item(),
        };
        if held.is_empty() || held.item.id != Item::GLASS_BOTTLE.id {
            return;
        }

        let world = player.world();
        let search_box = player
            .get_entity()
            .bounding_box
            .load()
            .expand(2.0, 2.0, 2.0);
        for candidate in world.get_entities_at_box(&search_box) {
            if let Some(cloud) = candidate
                .cast_any()
                .downcast_ref::<crate::entity::area_effect_cloud::AreaEffectCloudEntity>(
            ) && cloud.entity.is_alive()
            {
                let mut radius = cloud
                    .radius
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *radius -= 0.5;
                let new_radius = *radius;
                drop(radius);
                if new_radius <= 0.0 {
                    cloud.entity.remove();
                } else {
                    cloud.entity.set_synced_data(
                        pumpkin_data::tracked_data::area_effect_cloud::RADIUS,
                        new_radius,
                    );
                }

                world.play_sound(
                    Sound::ItemBottleFillDragonbreath,
                    SoundCategory::Neutral,
                    &player.position(),
                );
                Self::turn_bottle_into_item(
                    player,
                    hand,
                    held,
                    ItemStack::new(1, &Item::DRAGON_BREATH),
                );
                return;
            }
        }

        let (start_pos, end_pos) = self.get_start_and_end_pos(player);
        let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
            start_pos,
            end_pos,
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::SourceOnly,
        ));

        if !hit.is_miss {
            let hit_pos = hit.block_pos;
            let (block, state_id) = world.get_block_and_state_id(&hit_pos);
            if block.id == Block::WATER.id || block.is_waterlogged(state_id) {
                world.play_sound(
                    Sound::ItemBottleFill,
                    SoundCategory::Neutral,
                    &hit_pos.to_f64(),
                );
                Self::turn_bottle_into_item(player, hand, held, water_bottle());
            }
        }
    }

    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        let world = player.world();
        let state_id = world.get_block_state_id(&location);

        let is_water_target = block.id == Block::WATER.id
            || block.id == Block::WATER_CAULDRON.id
            || block.is_waterlogged(state_id);

        let check_pos = if is_water_target {
            location
        } else {
            location.offset(face.to_offset())
        };

        let (check_block, check_state_id) = world.get_block_and_state_id(&check_pos);

        if !matches!(check_block.id, BlockId::WATER | BlockId::WATER_CAULDRON)
            && !check_block.is_waterlogged(check_state_id)
        {
            return BlockActionResult::Pass;
        }

        if check_block.id == BlockId::WATER_CAULDRON {
            let mut props = WaterCauldronLikeProperties::from_state_id(check_state_id);
            let new_cauldron = if props.level > 1 {
                props.level -= 1;
                props.to_state_id(check_block)
            } else {
                Block::CAULDRON.default_state.id
            };
            world.set_block_state(&check_pos, new_cauldron, BlockFlags::NOTIFY_ALL);
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::UseCauldron as i32,
                1,
            );
        }

        world.play_sound(
            Sound::ItemBottleFill,
            SoundCategory::Neutral,
            &check_pos.to_f64(),
        );

        let mut water_bottle = water_bottle();
        if item.item_count == 1 && player.gamemode.load() != pumpkin_util::GameMode::Creative {
            *item = water_bottle;
        } else {
            item.decrement_unless_creative(player.gamemode.load(), 1);
            let was_added = player.inventory().insert_stack_anywhere(&mut water_bottle);
            if !was_added && !water_bottle.is_empty() {
                world.drop_stack(&player.position().to_block_pos(), water_bottle);
            }
        }
        BlockActionResult::Success
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

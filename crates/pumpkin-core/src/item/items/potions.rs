use rand::{RngExt, rng};
use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::projectile::{
    lingering_potion::LingeringPotionEntity, splash_potion::SplashPotionEntity,
};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::data_component_impl::PotionContentsImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct PotionItem;
pub struct SplashPotionItem;
pub struct LingeringPotionItem;

impl ItemMetadata for PotionItem {
    fn ids() -> Box<[u16]> {
        [Item::POTION.id].into()
    }
}

impl ItemMetadata for SplashPotionItem {
    fn ids() -> Box<[u16]> {
        [Item::SPLASH_POTION.id].into()
    }
}

impl ItemMetadata for LingeringPotionItem {
    fn ids() -> Box<[u16]> {
        [Item::LINGERING_POTION.id].into()
    }
}

const ROLL: f32 = -20.0;
const POWER: f32 = 0.5;
const DIVERGENCE: f32 = 1.0;
const THROW_SOUND_VOLUME: f32 = 0.5;

fn try_convert_to_mud(
    item: &mut ItemStack,
    player: &Player,
    location: BlockPos,
    face: BlockDirection,
    block: &Block,
) -> BlockActionResult {
    if face == BlockDirection::Down || !block.has_tag(&tag::Block::MINECRAFT_CONVERTIBLE_TO_MUD) {
        return BlockActionResult::Pass;
    }

    let is_water = item
        .get_data_component::<PotionContentsImpl>()
        .is_some_and(|pc| {
            pc.potion_id == Some(i32::from(Potion::WATER.id)) && pc.custom_effects.is_empty()
        });
    if !is_water {
        return BlockActionResult::Pass;
    }

    let world = player.world();
    world.play_sound(
        Sound::EntityGenericSplash,
        SoundCategory::Blocks,
        &location.to_f64(),
    );
    world.play_sound(
        Sound::ItemBottleEmpty,
        SoundCategory::Blocks,
        &location.to_f64(),
    );
    world.emit_game_event(GameEvent::FluidPlace.name(), location.to_f64());
    world.set_block_state(
        &location,
        Block::MUD.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );

    player.increment_stat(StatisticCategory::Used, i32::from(item.item.id), 1);

    let mut glass_bottle = ItemStack::new(1, &Item::GLASS_BOTTLE);
    if item.item_count == 1 && player.gamemode.load() != GameMode::Creative {
        *item = glass_bottle;
    } else if player.gamemode.load() != GameMode::Creative {
        item.decrement(1);
        let was_added = player.inventory.insert_stack_anywhere(&mut glass_bottle);
        if !was_added && !glass_bottle.is_empty() {
            world.drop_stack(&player.position().to_block_pos(), glass_bottle);
        }
    }

    BlockActionResult::Success
}

impl ItemBehaviour for PotionItem {
    fn normal_use(&self, _item: &Item, _player: &Player) {
        // Drinking is handled by the consumable flow in the server (active hand + consumption tick).
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
        try_convert_to_mud(item, player, location, face, block)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for SplashPotionItem {
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
        try_convert_to_mud(item, player, location, face, block)
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let mut stack = match hand {
            pumpkin_util::Hand::Right => player.inventory.held_item(),
            pumpkin_util::Hand::Left => player.inventory.off_hand_item(),
        };
        if stack.is_empty() || stack.item.id != Item::SPLASH_POTION.id {
            return;
        }

        let position = player.position();
        let world = player.world();
        world.play_sound_fine(
            Sound::EntitySplashPotionThrow,
            SoundCategory::Players,
            &position,
            THROW_SOUND_VOLUME,
            0.4 / (rng().random::<f32>() * 0.4 + 0.8),
        );
        let entity = Entity::new(world.clone(), position, &EntityType::SPLASH_POTION);
        let splash = SplashPotionEntity::new_shot(entity, player.get_entity());
        splash.set_item_stack(stack.copy_with_count(1));

        splash
            .thrown
            .set_velocity_from(pitch, yaw, ROLL, POWER, DIVERGENCE);

        world.spawn_entity(Arc::new(splash));

        let item_id = stack.item.id;
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        match hand {
            pumpkin_util::Hand::Right => player.inventory.set_held_item(stack),
            pumpkin_util::Hand::Left => player
                .inventory
                .set_stack_in_hand(pumpkin_util::Hand::Left, stack),
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item_id), 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for LingeringPotionItem {
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
        try_convert_to_mud(item, player, location, face, block)
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let mut stack = match hand {
            pumpkin_util::Hand::Right => player.inventory.held_item(),
            pumpkin_util::Hand::Left => player.inventory.off_hand_item(),
        };
        if stack.is_empty() || stack.item.id != Item::LINGERING_POTION.id {
            return;
        }

        let position = player.position();
        let world = player.world();
        world.play_sound_fine(
            Sound::EntityLingeringPotionThrow,
            SoundCategory::Neutral,
            &position,
            THROW_SOUND_VOLUME,
            0.4 / (rng().random::<f32>() * 0.4 + 0.8),
        );
        let entity = Entity::new(world.clone(), position, &EntityType::LINGERING_POTION);
        let ling = LingeringPotionEntity::new_shot(entity, player.get_entity());
        ling.set_item_stack(stack.copy_with_count(1));

        ling.thrown
            .set_velocity_from(pitch, yaw, ROLL, POWER, DIVERGENCE);

        world.spawn_entity(Arc::new(ling));

        let item_id = stack.item.id;
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        match hand {
            pumpkin_util::Hand::Right => player.inventory.set_held_item(stack),
            pumpkin_util::Hand::Left => player
                .inventory
                .set_stack_in_hand(pumpkin_util::Hand::Left, stack),
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item_id), 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

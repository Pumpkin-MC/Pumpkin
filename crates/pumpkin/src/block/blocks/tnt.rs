use pumpkin_data::Block;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::TntLikeProperties;
use pumpkin_data::item::Item;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::translation;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use super::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, ExplodeArgs, OnNeighborUpdateArgs, OnProjectileHitArgs, PlacedArgs,
    UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::entity::tnt::TNTEntity;
use crate::world::World;

#[pumpkin_block("minecraft:tnt")]
pub struct TNTBlock;

impl TNTBlock {
    /// Vanilla `TntBlock.prime` plus the block removal its callers do.
    pub fn prime(world: &Arc<World>, location: &BlockPos) -> bool {
        let Some(tnt) = Self::prepare(world, location) else {
            return false;
        };

        // Swap in air to claim the block: `set_block_state` locks the chunk section, so a task
        // racing for the same spot gets a non-TNT old state back and drops out here. Nothing is
        // restored, the events above already ran and cannot re-enter through `placed`.
        if world
            .set_block_state(location, BlockStateId::AIR, BlockFlags::NOTIFY_ALL)
            .to_block()
            != &Block::TNT
        {
            return false;
        }

        Self::ignite(world, tnt);
        true
    }

    /// Primes a TNT block the caller already took out of the world.
    pub fn spawn_primed(world: &Arc<World>, location: &BlockPos) -> bool {
        let Some(tnt) = Self::prepare(world, location) else {
            return false;
        };
        Self::ignite(world, tnt);
        true
    }

    /// Gamerule and plugin events; `None` when priming is denied.
    fn prepare(world: &Arc<World>, location: &BlockPos) -> Option<Arc<TNTEntity>> {
        if !world.level_info.load().game_rules.tnt_explodes {
            return None;
        }

        let mut event = crate::plugin::api::events::block::tnt_prime::TNTPrimeEvent::new(
            *location,
            "REDSTONE".to_string(),
        );
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return None;
        }

        let tnt = TNTEntity::primed(world, location, TNTEntity::DEFAULT_FUSE);
        let mut prime_event =
            crate::plugin::api::events::entity::explosion_prime::ExplosionPrimeEvent::new(
                tnt.get_entity().entity_id,
                TNTEntity::DEFAULT_POWER,
                false,
            );
        if let Some(server) = world.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut prime_event);
        }
        if prime_event.cancelled {
            return None;
        }

        Some(tnt)
    }

    fn ignite(world: &Arc<World>, tnt: Arc<TNTEntity>) {
        let pos = tnt.get_entity().pos.load();
        world.spawn_entity(tnt);
        world.play_sound(
            pumpkin_data::sound::Sound::EntityTntPrimed,
            SoundCategory::Blocks,
            &pos,
        );
    }
}

impl BlockBehaviour for TNTBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let item_id = args.item_stack.item.id;
        if item_id != Item::FLINT_AND_STEEL.id && item_id != Item::FIRE_CHARGE.id {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        if Self::prime(args.world, args.position) {
            if args.player.gamemode.load() != GameMode::Creative {
                if item_id == Item::FLINT_AND_STEEL.id {
                    let _ = args.item_stack.damage_item(1);
                } else {
                    args.item_stack.decrement(1);
                }
            }
            BlockActionResult::Success
        } else if !args.world.level_info.load().game_rules.tnt_explodes {
            args.player.send_system_message_raw(
                &TextComponent::translate(translation::java::BLOCK_MINECRAFT_TNT_DISABLED, []),
                true,
            );
            BlockActionResult::Pass
        } else {
            BlockActionResult::Success
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if args.block != Block::from_state_id(args.old_state_id)
            && block_receives_redstone_power(args.world, args.position)
        {
            Self::prime(args.world, args.position);
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if block_receives_redstone_power(args.world, args.position) {
            Self::prime(args.world, args.position);
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        if args.player.gamemode.load() != GameMode::Creative {
            let props = TntLikeProperties::from_state_id(args.state.id);
            if props.r#unstable {
                // `break_block` already swapped the TNT away, so `prime` would find no TNT here.
                Self::spawn_primed(args.world, args.position);
            }
        }
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        if args.projectile.get_entity().is_on_fire() {
            Self::prime(args.world, args.position);
        }
    }

    fn explode(&self, args: ExplodeArgs<'_>) {
        // Vanilla `TntBlock.wasExploded`: gated on `GameRules.TNT_EXPLODES`, no sound.
        if !args.world.level_info.load().game_rules.tnt_explodes {
            return;
        }
        let fuse = TNTEntity::random_short_fuse(TNTEntity::DEFAULT_FUSE);
        let tnt = TNTEntity::primed(args.world, args.position, fuse);
        args.world.spawn_entity(tnt);
    }

    fn should_drop_items_on_explosion(&self) -> bool {
        false
    }
}

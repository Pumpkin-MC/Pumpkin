use std::sync::Arc;

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::{DamageResult, ItemStack};
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::GameMode;

use crate::entity::Entity;
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent;
use crate::world::loot::LootContextParameters;

/// Vanilla `Shearable`: a mob that players and dispensers can shear.
pub trait Shearable: Mob {
    /// Shears this mob. Returns `false` if it was already sheared or a plugin stopped it.
    ///
    /// Vanilla's `shear` returns nothing, but because Pumpkin handles players' interactions in parallel, so
    /// two of them can pass [`Self::ready_for_shearing`] at once.
    /// IMPORTANT: Implementations must claim their shear state
    /// with a single atomic swap so only one of them goes on to drop loot.
    /// See [`SnowGolemEntity::shear`]
    fn shear(&self, sound_category: SoundCategory, tool: &ItemStack) -> bool;

    fn ready_for_shearing(&self) -> bool;
}

/// The shears branch every shearable mob has in vanilla `mobInteract`.
///
/// Returns `false` if a plugin cancels the shearing or another shear got there first.
pub fn shear_by_player(
    shearable: &dyn Shearable,
    player: &Arc<Player>,
    tool: &mut ItemStack,
) -> bool {
    let entity = &shearable.get_mob_entity().living_entity.entity;
    let world = entity.world.load();
    if let Some(server) = world.server.upgrade() {
        let mut event = PlayerShearEntityEvent {
            player: player.clone(),
            entity_id: entity.entity_id,
            hand: 0,
            cancelled: false,
        };
        server.plugin_manager.fire_blocking(&server, &mut event);
        if event.cancelled {
            return false;
        }
    }

    let pos = entity.pos.load();
    if !shearable.shear(SoundCategory::Players, tool) {
        return false;
    }
    world.emit_game_event(GameEvent::Shear.name(), pos);
    if player.gamemode.load() != GameMode::Creative {
        let item: &pumpkin_data::item::Item = tool.item;
        let result = tool.damage_item(1);
        if result != DamageResult::Untouched {
            // Not `damage_item_in_slot`: the interact handler writes `tool` back to the hand and counts the break.
            player.fire_item_damage_events(item, 1, result == DamageResult::Broken);
        }
    }
    true
}

/// Vanilla `Entity.dropFromShearingLootTable`
#[must_use]
pub fn shearing_loot(entity: &Entity, loot_key: &str, tool: &ItemStack) -> Vec<ItemStack> {
    let Some(loot_table) = entity.world.load().get_loot_table(loot_key) else {
        return Vec::new();
    };
    let params = LootContextParameters {
        this_entity: Some(entity.entity_type),
        position: Some(entity.pos.load()),
        tool: Some(tool.clone()),
        ..Default::default()
    };
    loot_table.generate_loot_with_context(rand::random(), &params)
}

use std::sync::Arc;

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::Hand;

use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent;
use crate::world::game_event::GameEventContext;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

/// Vanilla `Shearable`: a mob that players and dispensers can shear.
pub trait Shearable: Mob {
    /// Shears this mob. Returns `false` if it was already sheared.
    ///
    /// Player interactions run in parallel, so two shears can pass [`Self::ready_for_shearing`] at once.
    /// Implementations claim their shear state with one atomic swap so only one of them drops loot.
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
    shear_with_player(shearable, player, tool, None)
}

pub fn shear_by_player_in_hand(
    shearable: &dyn Shearable,
    player: &Arc<Player>,
    tool: &mut ItemStack,
    hand: Hand,
) -> bool {
    shear_with_player(shearable, player, tool, Some(hand))
}

fn shear_with_player(
    shearable: &dyn Shearable,
    player: &Arc<Player>,
    tool: &mut ItemStack,
    hand: Option<Hand>,
) -> bool {
    let entity = &shearable.get_mob_entity().living_entity.entity;
    let world = entity.world.load();
    if let Some(server) = world.server.upgrade() {
        let mut event = PlayerShearEntityEvent {
            player: player.clone(),
            entity_id: entity.entity_id,
            hand: hand.map_or(0, Hand::packet_id),
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
    world.emit_game_event_with_context(
        GameEvent::Shear.name(),
        pos,
        &GameEventContext {
            source_entity: Some(player.as_ref()),
        },
    );
    // Without a hand, the interact handler writes `tool` back to the hand and counts the break.
    match hand {
        Some(hand) => {
            let _ = player.damage_held_stack(tool, 1, hand);
        }
        None => {
            let _ = player.damage_item_stack(tool, 1);
        }
    }
    true
}

/// Vanilla `Entity.dropFromShearingLootTable`
#[must_use]
pub fn shearing_loot(
    shearable: &dyn EntityBase,
    loot_key: &str,
    tool: &ItemStack,
) -> Vec<ItemStack> {
    let entity = shearable.get_entity();
    let world = entity.world.load();
    let Some(loot_table) = world.get_loot_table(loot_key) else {
        return Vec::new();
    };
    let params = LootContextParameters {
        world: Some(&world),
        this_entity: Some(shearable),
        position: Some(entity.pos.load()),
        tool: Some(tool.clone()),
        ..Default::default()
    };
    generate_loot_from_handle(&loot_table, rand::random(), &params)
}

use std::sync::Arc;

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::GameMode;

use crate::entity::Entity;
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent;
use crate::world::loot::LootContextParameters;

/// Vanilla `Shearable`: a mob that players and dispensers can shear.
pub trait Shearable: Mob {
    fn shear(&self, sound_category: SoundCategory, tool: &ItemStack);

    fn ready_for_shearing(&self) -> bool;
}

/// The shears branch every shearable mob has in vanilla `mobInteract`.
///
/// Returns `false` if a plugin cancels the shearing.
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
    shearable.shear(SoundCategory::Players, tool);
    world.emit_game_event(GameEvent::Shear.name(), pos);
    if player.gamemode.load() != GameMode::Creative {
        let _ = tool.damage_item(1);
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

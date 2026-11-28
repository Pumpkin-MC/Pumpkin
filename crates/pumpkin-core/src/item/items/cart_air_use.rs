use pumpkin_data::{
    data_component_impl::{
        BlocksAttacksImpl, ConsumableImpl, EquippableImpl, FoodImpl, KineticWeaponImpl,
    },
    entity::EntityType,
    item_stack::ItemStack,
    sound::SoundCategory,
    statistic::StatisticCategory,
};
use pumpkin_util::{GameMode, Hand};
use std::sync::Arc;

use crate::{
    entity::{
        hand_equipment_slot,
        player::{ItemCooldown, Player},
    },
    world::World,
};

use super::vehicle_placement::{
    block_use_cooldown, commit_item_use, complete_item_use, finish_consumed_inventory,
    placement_cooldown, publish_placement_cooldown,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CartAirUseResult {
    Pass,
    Use,
    Equip,
}

impl CartAirUseResult {
    pub const fn consumes_action(self) -> bool {
        !matches!(self, Self::Pass)
    }
}

/// The base air action inherited by classic minecart items. The packet caller
/// owns the interaction veto and revalidates the world captured before it.
pub fn use_cart_in_air(
    player: &Player,
    hand: Hand,
    slot: usize,
    stack: &ItemStack,
    world: &Arc<World>,
) -> CartAirUseResult {
    use CartAirUseResult::{Equip, Pass, Use};
    let gamemode = player.gamemode.load();
    let inventory = player.inventory();
    let (group, duration) = placement_cooldown(stack);
    if gamemode == GameMode::Spectator
        || !Arc::ptr_eq(&player.world(), world)
        || inventory.equipment_slot_index(&hand_equipment_slot(hand)) != Some(slot)
        || player.is_on_cooldown(group)
    {
        return Pass;
    }
    let live = inventory.get_slot(slot);
    if live.uid != stack.uid || !live.are_equal(stack) {
        return Pass;
    }

    if let Some(consumable) = stack.get_data_component::<ConsumableImpl>() {
        if stack
            .get_data_component::<FoodImpl>()
            .is_some_and(|food| !player.can_eat(food.can_always_eat))
        {
            return Pass;
        }
        let ticks = consumable.consume_ticks();
        if ticks > 0 {
            return if player
                .living_entity
                .start_cart_use(hand, slot, live, ticks, world)
            {
                Use
            } else {
                Pass
            };
        }
        let Some(committed) = commit_cart_consumption(player, slot, stack, false) else {
            return Pass;
        };
        committed.publish(player, slot, stack, false, world);
        return Use;
    }
    if let Some(equippable) = stack.get_data_component::<EquippableImpl>()
        && equippable.swappable
    {
        if inventory.equipment_slot_index(equippable.slot).is_none()
            || equippable
                .allowed_entities
                .as_ref()
                .is_some_and(|entities| !entities.contains(&EntityType::PLAYER))
        {
            return Pass;
        }
        let before = player.screen_equipment();
        let Some((extras, cooldown)) = commit_item_use(player, group, duration, || {
            if inventory.equipment_slot_index(&hand_equipment_slot(hand)) != Some(slot) {
                return None;
            }
            inventory.swap_equippable(hand, equippable.slot, stack, gamemode == GameMode::Creative)
        }) else {
            return Pass;
        };
        player.increment_stat(StatisticCategory::Used, i32::from(stack.get_item().id), 1);
        for extra in extras {
            inventory.offer(extra, false, player);
        }
        player.publish_screen_equipment(&before);
        player.send_player_screen_updates();
        if let Some(cooldown) = cooldown {
            publish_placement_cooldown(player, group, cooldown);
        }
        return Equip;
    }

    let blocks = stack.get_data_component::<BlocksAttacksImpl>().is_some();
    let kinetic = stack.get_data_component::<KineticWeaponImpl>();
    if !blocks && kinetic.is_none() {
        return Pass;
    }
    if !player
        .living_entity
        .start_cart_use(hand, slot, live, stack.get_max_use_time(), world)
    {
        return Pass;
    }
    if !blocks
        && Arc::ptr_eq(&player.world(), world)
        && let Some(sound) = kinetic.and_then(|weapon| weapon.sound.as_ref())
    {
        world.play_cart_use_sound(
            player,
            sound,
            SoundCategory::Players,
            &player.position(),
            1.0,
            1.0,
        );
    }
    Use
}

/// Commits accepted use without rechecking the cooldown that gated its start.
pub fn commit_timed_cart_use(
    player: &Player,
    slot: usize,
    stack: &ItemStack,
) -> Option<TimedCartUse> {
    if stack.get_data_component::<ConsumableImpl>().is_some() {
        return commit_cart_consumption(player, slot, stack, true).map(TimedCartUse::Consumption);
    }
    let (group, duration) = block_use_cooldown(stack);
    let ((), cooldown) = complete_item_use(player, group, duration, || Some(()))?;
    Some(TimedCartUse::Other(cooldown))
}

pub enum TimedCartUse {
    Consumption(CartConsumption),
    Other(Option<ItemCooldown>),
}

impl TimedCartUse {
    pub(crate) fn publish(
        self,
        player: &Player,
        slot: usize,
        stack: &ItemStack,
        origin: &Arc<World>,
    ) {
        match self {
            Self::Consumption(consumption) => {
                consumption.publish(player, slot, stack, true, origin);
            }
            Self::Other(cooldown) => {
                send_cart_use_complete(player);
                if let Some(cooldown) = cooldown {
                    publish_placement_cooldown(player, block_use_cooldown(stack).0, cooldown);
                }
            }
        }
    }
}

pub struct CartConsumption {
    remaining: ItemStack,
    gamemode: GameMode,
    cooldown: Option<ItemCooldown>,
}

impl CartConsumption {
    fn publish(
        self,
        player: &Player,
        slot: usize,
        stack: &ItemStack,
        timed: bool,
        origin: &Arc<World>,
    ) {
        if timed {
            send_cart_use_complete(player);
        }
        player
            .living_entity
            .apply_cart_consumption_effects(player, stack, origin);
        finish_consumed_inventory(player, slot, stack, &self.remaining, self.gamemode);
        if let Some(cooldown) = self.cooldown {
            publish_placement_cooldown(player, block_use_cooldown(stack).0, cooldown);
        }
    }
}

fn send_cart_use_complete(player: &Player) {
    player.try_send_client_packet(&pumpkin_protocol::java::client::play::CEntityStatus::new(
        player.living_entity.entity.entity_id,
        pumpkin_data::entity::EntityStatus::UseItemComplete as i8,
    ));
}

fn commit_cart_consumption(
    player: &Player,
    slot: usize,
    stack: &ItemStack,
    timed: bool,
) -> Option<CartConsumption> {
    let gamemode = player.gamemode.load();
    if gamemode == GameMode::Spectator {
        return None;
    }
    let (group, duration) = block_use_cooldown(stack);
    let debit = || {
        let mut remaining = stack.clone();
        remaining.decrement_unless_creative(gamemode, 1);
        player
            .inventory
            .replace_slot_if_same(slot, stack, remaining.clone())
            .then_some(remaining)
    };
    let (remaining, cooldown) = if timed {
        complete_item_use(player, group, duration, debit)
    } else {
        commit_item_use(player, group, duration, debit)
    }?;
    Some(CartConsumption {
        remaining,
        gamemode,
        cooldown,
    })
}

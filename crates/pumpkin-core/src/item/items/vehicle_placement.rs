use std::sync::atomic::Ordering;

use pumpkin_data::{data_component_impl::UseRemainderImpl, item_stack::ItemStack};
use pumpkin_protocol::{
    java::client::play::CItemCooldown,
    ser::{NetworkReadExt, NetworkWriteExt},
};
use pumpkin_util::GameMode;

use crate::entity::player::{ItemCooldown, Player};

pub(super) fn block_use_cooldown(stack: &ItemStack) -> (&str, Option<i32>) {
    let cooldown = stack.get_use_cooldown();
    let group = cooldown
        .and_then(|cooldown| cooldown.cooldown_group.as_deref())
        .unwrap_or(stack.get_item().registry_key);
    (
        group,
        cooldown.map(|cooldown| (cooldown.seconds * 20.0) as i32),
    )
}

pub(super) fn placement_cooldown(stack: &ItemStack) -> (&str, Option<i32>) {
    let (group, duration) = block_use_cooldown(stack);
    let duration = duration.filter(|_| stack.get_base_use_time() == 0);
    (group, duration)
}

pub(super) fn publish_placement_cooldown(player: &Player, group: &str, cooldown: ItemCooldown) {
    let Some(client) = player.client.java() else {
        return;
    };
    let group_key = Player::cooldown_group_key(group);
    let lock_current = || {
        let cooldowns = player
            .item_cooldowns
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cooldowns
            .get(group_key)
            .is_some_and(|current| {
                current.start_tick == cooldown.start_tick && current.duration == cooldown.duration
            })
            .then_some(cooldowns)
    };
    let remaining = || {
        let elapsed = player
            .tick_counter
            .load(Ordering::Relaxed)
            .saturating_sub(cooldown.start_tick);
        cooldown.duration.saturating_sub(elapsed).max(0)
    };
    let Some(duration) = lock_current().map(|_guard| remaining()) else {
        return;
    };
    let serialize =
        |ticks: i32| client.serialize_packet(&CItemCooldown::new(group.to_owned(), ticks.into()));
    let Ok(original) = serialize(duration) else {
        return;
    };
    let client_id = CItemCooldown::group_packet_id(client.version.load());
    client.try_enqueue_packet_data_projecting(
        original.clone(),
        lock_current,
        |data, primary, guard| {
            let mut payload = data.as_ref();
            let same_group = payload
                .get_var_int()
                .is_ok_and(|id| client_id == Some(id.0) || (primary && data == original))
                && payload
                    .get_var_int()
                    .ok()
                    .and_then(|length| usize::try_from(length.0).ok())
                    .is_some_and(|length| {
                        let Some((name, rest)) = payload.split_at_checked(length) else {
                            return false;
                        };
                        payload = rest;
                        std::str::from_utf8(name)
                            .is_ok_and(|name| Player::cooldown_group_key(name) == group_key)
                    });
            if guard.is_none() {
                return (!primary && !same_group).then_some(data);
            }
            if !same_group {
                return Some(data);
            }
            let ticks = remaining();
            let mut old_duration = payload;
            if old_duration.get_var_int().is_ok_and(|old| old.0 == ticks) && old_duration.is_empty()
            {
                return Some(data);
            }
            let mut projected = data[..data.len() - payload.len()].to_vec();
            projected.write_var_int(&ticks.into()).ok()?;
            Some(projected.into())
        },
        |_, _, _| {},
    );
}

pub(super) fn commit_placement_item(
    player: &Player,
    slot_index: usize,
    stack: &ItemStack,
    gamemode: GameMode,
    group: &str,
    duration: Option<i32>,
) -> Option<(ItemStack, Option<ItemCooldown>)> {
    commit_item_use(player, group, duration, || {
        let mut consumed = stack.clone();
        consumed.decrement_unless_creative(gamemode, 1);
        player
            .inventory
            .replace_slot_if_same(slot_index, stack, consumed.clone())
            .then_some(consumed)
    })
}

/// Runs an item debit while its cooldown group is eligible, before publication.
pub(super) fn commit_item_use<T>(
    player: &Player,
    group: &str,
    duration: Option<i32>,
    debit: impl FnOnce() -> Option<T>,
) -> Option<(T, Option<ItemCooldown>)> {
    record_item_use(player, group, duration, debit, true)
}

pub(super) fn complete_item_use<T>(
    player: &Player,
    group: &str,
    duration: Option<i32>,
    debit: impl FnOnce() -> Option<T>,
) -> Option<(T, Option<ItemCooldown>)> {
    record_item_use(player, group, duration, debit, false)
}

fn record_item_use<T>(
    player: &Player,
    group: &str,
    duration: Option<i32>,
    debit: impl FnOnce() -> Option<T>,
    require_eligible: bool,
) -> Option<(T, Option<ItemCooldown>)> {
    let mut cooldowns = player
        .item_cooldowns
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let group = Player::cooldown_group_key(group);
    let tick = player.tick_counter.load(Ordering::Relaxed);
    if require_eligible
        && cooldowns
            .get(group)
            .is_some_and(|cooldown| tick.saturating_sub(cooldown.start_tick) < cooldown.duration)
    {
        return None;
    }
    let committed = debit()?;
    let cooldown = duration.map(|duration| {
        let cooldown = ItemCooldown {
            start_tick: tick,
            duration,
        };
        cooldowns.insert(group.to_owned(), cooldown);
        cooldown
    });
    Some((committed, cooldown))
}

pub(super) fn finish_placement_inventory(
    player: &Player,
    slot_index: usize,
    stack: &ItemStack,
    consumed: &ItemStack,
    gamemode: GameMode,
) {
    finish_spent_inventory(
        player,
        slot_index,
        stack,
        consumed,
        gamemode,
        stack.get_base_use_time() == 0,
    );
}

pub(super) fn finish_consumed_inventory(
    player: &Player,
    slot_index: usize,
    stack: &ItemStack,
    consumed: &ItemStack,
    gamemode: GameMode,
) -> bool {
    finish_spent_inventory(player, slot_index, stack, consumed, gamemode, true)
}

pub(super) fn use_remainder(
    stack_before_use: &ItemStack,
    used_stack: ItemStack,
    gamemode: GameMode,
) -> (ItemStack, Option<ItemStack>) {
    let Some(remainder) = stack_before_use.get_data_component::<UseRemainderImpl>() else {
        return (used_stack, None);
    };
    let mut extra = None;
    let remaining = remainder.convert_into_remainder(
        used_stack,
        stack_before_use.item_count,
        gamemode == GameMode::Creative,
        |stack| extra = Some(stack),
    );
    (remaining, extra)
}

fn finish_spent_inventory(
    player: &Player,
    slot_index: usize,
    stack: &ItemStack,
    consumed: &ItemStack,
    gamemode: GameMode,
    apply_remainder: bool,
) -> bool {
    let (remaining, mut extra) = if apply_remainder {
        use_remainder(stack, consumed.clone(), gamemode)
    } else {
        (consumed.clone(), None)
    };
    let updated = player
        .inventory
        .replace_slot_if_same(slot_index, consumed, remaining.clone());
    if updated {
        player.sync_hand_slot(slot_index, remaining);
    } else if consumed.is_empty() && !remaining.is_empty() {
        // The placement earned this remainder before a callback refilled the hand.
        extra = Some(remaining);
    }
    if let Some(extra) = extra {
        player.inventory().offer(extra, false, player);
    }
    updated
}

//! Beacon screen handler, mirroring vanilla `BeaconMenu`.
//!
//! Layout: slot 0 is the payment slot (owned by the menu, returned to the player on
//! close), slots 1..28 are the player's main inventory, 28..37 the hotbar.
//! Three container properties are tracked: power level, primary effect, secondary
//! effect. Effects are encoded as `registry id + 1`, with `0` meaning "no effect".

use std::{
    any::Any,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use pumpkin_data::{
    data_component_impl::IDSetContent, effect::StatusEffect, item_stack::ItemStack,
    screen::WindowType, tag, tag::Taggable,
};

use crate::{
    inventory::{Inventory, SimpleInventory},
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
        ScreenProperty,
    },
    slot::Slot,
    window_property::PropertyDelegate,
};

pub const DATA_LEVELS: i32 = 0;
pub const DATA_PRIMARY: i32 = 1;
pub const DATA_SECONDARY: i32 = 2;
pub const NUM_DATA_VALUES: i32 = 3;

const PAYMENT_SLOT: i32 = 0;
const INV_SLOT_START: i32 = 1;
const INV_SLOT_END: i32 = 28;
const USE_ROW_SLOT_START: i32 = 28;
const USE_ROW_SLOT_END: i32 = 37;

/// Effects selectable per beacon tier, index = tier - 1. Mirrors `BeaconBlockEntity.BEACON_EFFECTS`.
pub const BEACON_EFFECTS: [&[&StatusEffect]; 4] = [
    &[&StatusEffect::SPEED, &StatusEffect::HASTE],
    &[&StatusEffect::RESISTANCE, &StatusEffect::JUMP_BOOST],
    &[&StatusEffect::STRENGTH],
    &[&StatusEffect::REGENERATION],
];

#[must_use]
pub fn encode_effect(effect: Option<&'static StatusEffect>) -> i32 {
    effect.map_or(0, |e| i32::from(e.id) + 1)
}

#[must_use]
pub fn decode_effect(value: i32) -> Option<&'static StatusEffect> {
    if value <= 0 {
        return None;
    }
    u16::try_from(value - 1)
        .ok()
        .and_then(<StatusEffect as IDSetContent>::from_id)
}

/// Drops effects a beacon can never grant. Mirrors `BeaconBlockEntity.filterEffect`.
#[must_use]
pub fn filter_effect(effect: Option<&'static StatusEffect>) -> Option<&'static StatusEffect> {
    effect.filter(|e| required_levels(Some(e)) <= BEACON_EFFECTS.len() as i32)
}

/// Beacon tier needed for an effect; `0` for none, `i32::MAX` for effects a beacon can't grant.
#[must_use]
pub fn required_levels(effect: Option<&StatusEffect>) -> i32 {
    let Some(effect) = effect else { return 0 };
    BEACON_EFFECTS
        .iter()
        .position(|tier| tier.iter().any(|e| e.id == effect.id))
        .map_or(i32::MAX, |i| i as i32 + 1)
}

/// Mirrors `BeaconBlockEntity.validateEffects`.
#[must_use]
pub fn validate_effects(
    primary: Option<&StatusEffect>,
    secondary: Option<&StatusEffect>,
    levels: i32,
) -> bool {
    if secondary.is_some() && levels < 4 {
        return false;
    }
    let primary_level = required_levels(primary);
    let secondary_level = required_levels(secondary);
    if primary_level > levels || secondary_level > levels || primary_level >= 4 {
        return false;
    }
    secondary_level == 0 || secondary_level >= 4 || primary.map(|e| e.id) == secondary.map(|e| e.id)
}

/// Accepts a single item tagged `minecraft:beacon_payment_items`.
struct PaymentSlot {
    inventory: Arc<dyn Inventory>,
    id: AtomicU8,
}

impl Slot for PaymentSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        0
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> bool {
        stack
            .item
            .has_tag(&tag::Item::MINECRAFT_BEACON_PAYMENT_ITEMS)
    }

    fn get_max_item_count(&self) -> u8 {
        1
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

pub fn create_beacon_handler(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    beacon_data: &Arc<dyn PropertyDelegate>,
) -> BeaconScreenHandler {
    BeaconScreenHandler::new(sync_id, player_inventory, beacon_data)
}

pub struct BeaconScreenHandler {
    /// Menu-owned payment inventory, dropped back to the player on close.
    payment: Arc<SimpleInventory>,
    /// The beacon block entity's levels / primary / secondary data.
    pub beacon_data: Arc<dyn PropertyDelegate>,
    behaviour: ScreenHandlerBehaviour,
}

impl BeaconScreenHandler {
    fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        beacon_data: &Arc<dyn PropertyDelegate>,
    ) -> Self {
        struct BeaconScreenListener;
        impl ScreenHandlerListener for BeaconScreenListener {
            fn on_property_update(
                &self,
                screen_handler: &ScreenHandlerBehaviour,
                property: u8,
                value: i32,
            ) {
                if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                    sync_handler.update_property(screen_handler, i32::from(property), value);
                }
            }
        }

        let payment = Arc::new(SimpleInventory::new(1));
        let mut handler = Self {
            payment: payment.clone(),
            beacon_data: beacon_data.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Beacon)),
        };

        handler.add_slot(Arc::new(PaymentSlot {
            inventory: payment,
            id: AtomicU8::new(0),
        }));
        for i in 0..NUM_DATA_VALUES {
            handler.add_property(ScreenProperty::new(beacon_data.clone(), i as u8));
        }
        handler.add_listener(Arc::new(BeaconScreenListener));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);
        handler
    }

    #[must_use]
    pub fn get_levels(&self) -> i32 {
        self.beacon_data.get_property(DATA_LEVELS)
    }

    #[must_use]
    pub fn has_payment(&self) -> bool {
        !self.payment.get_stack(0).is_empty()
    }

    /// Mirrors `BeaconMenu.updateEffects`: validates against the current tier, stores the
    /// effects, and consumes the payment item. Returns `false` when the request is invalid.
    pub fn update_effects(
        &mut self,
        primary: Option<&'static StatusEffect>,
        secondary: Option<&'static StatusEffect>,
    ) -> bool {
        if !self.has_payment() {
            return false;
        }
        if !validate_effects(primary, secondary, self.get_levels()) {
            return false;
        }
        self.beacon_data
            .set_property(DATA_PRIMARY, encode_effect(primary));
        self.beacon_data
            .set_property(DATA_SECONDARY, encode_effect(secondary));
        self.payment.remove_stack_specific(0, 1);
        self.send_content_updates();
        true
    }
}

impl ScreenHandler for BeaconScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        let stack = self.payment.remove_stack(0);
        if !stack.is_empty() {
            player.drop_item(stack, false);
        }
    }

    /// Mirrors `BeaconMenu.quickMoveStack`.
    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let slot = self.get_behaviour().slots[slot_index as usize].clone();
        if !slot.has_stack() {
            return ItemStack::EMPTY.clone();
        }
        let mut stack = slot.get_stack();
        let clicked = stack.clone();

        let payment_slot = self.get_behaviour().slots[PAYMENT_SLOT as usize].clone();
        let moved = if slot_index == PAYMENT_SLOT {
            self.insert_item(&mut stack, INV_SLOT_START, USE_ROW_SLOT_END, true)
        } else if !payment_slot.has_stack()
            && payment_slot.can_insert(&stack)
            && stack.item_count == 1
        {
            self.insert_item(&mut stack, PAYMENT_SLOT, PAYMENT_SLOT + 1, false)
        } else if (INV_SLOT_START..INV_SLOT_END).contains(&slot_index) {
            self.insert_item(&mut stack, USE_ROW_SLOT_START, USE_ROW_SLOT_END, false)
        } else if (USE_ROW_SLOT_START..USE_ROW_SLOT_END).contains(&slot_index) {
            self.insert_item(&mut stack, INV_SLOT_START, INV_SLOT_END, false)
        } else {
            self.insert_item(&mut stack, INV_SLOT_START, USE_ROW_SLOT_END, false)
        };
        if !moved {
            return ItemStack::EMPTY.clone();
        }

        if stack.is_empty() {
            slot.set_stack(ItemStack::EMPTY.clone());
        } else {
            slot.mark_dirty();
        }
        if stack.item_count == clicked.item_count {
            return ItemStack::EMPTY.clone();
        }
        slot.on_take_item(player, &stack);
        clicked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEED: Option<&StatusEffect> = Some(&StatusEffect::SPEED);
    const STRENGTH: Option<&StatusEffect> = Some(&StatusEffect::STRENGTH);
    const REGEN: Option<&StatusEffect> = Some(&StatusEffect::REGENERATION);
    const POISON: Option<&StatusEffect> = Some(&StatusEffect::POISON);

    #[test]
    fn validate_effects_matches_vanilla() {
        assert!(validate_effects(None, None, 0));
        assert!(validate_effects(SPEED, None, 1));
        assert!(!validate_effects(STRENGTH, None, 2));
        assert!(validate_effects(STRENGTH, None, 3));
        // Secondary needs a full pyramid.
        assert!(!validate_effects(SPEED, REGEN, 3));
        assert!(validate_effects(SPEED, REGEN, 4));
        // Secondary may only be regeneration or the primary itself.
        assert!(validate_effects(SPEED, SPEED, 4));
        assert!(!validate_effects(SPEED, STRENGTH, 4));
        // Regeneration is never a primary; non-beacon effects are rejected.
        assert!(!validate_effects(REGEN, None, 4));
        assert!(!validate_effects(POISON, None, 4));
    }

    #[test]
    fn effect_encoding_round_trips() {
        assert_eq!(encode_effect(None), 0);
        assert_eq!(decode_effect(0), None);
        for effect in BEACON_EFFECTS.iter().flat_map(|t| t.iter()) {
            assert_eq!(
                decode_effect(encode_effect(Some(effect))).map(|e| e.id),
                Some(effect.id)
            );
        }
        assert_eq!(filter_effect(POISON), None);
        assert!(filter_effect(SPEED).is_some());
    }
}

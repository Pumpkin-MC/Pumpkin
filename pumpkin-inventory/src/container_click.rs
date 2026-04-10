//! Container click handling.
//!
//! This module processes inventory click packets from the client and converts
//! them into structured click events. It handles all click types:
//! - Mouse clicks (left/right)
//! - Shift-clicks
//! - Hotbar key presses
//! - Drop actions (Q key or clicking outside)
//! - Drag operations (click and drag across slots)
//! - Double-clicks (pickup all)
//!
//! # Click Types
//!
//! The client sends a mode and button value that are decoded into
//! [`ClickType`] variants. See the Minecraft protocol documentation
//! for packet format details.

use crate::InventoryError;
use pumpkin_protocol::java::server::play::SlotActionType;

/// A parsed container click event.
///
/// Contains the slot being clicked and the type of click action.
#[derive(Debug)]
pub struct Click {
    /// The slot being clicked (or outside the inventory).
    pub slot: Slot,
    /// The type of click action (mouse click, shift-click, drag, etc.).
    pub click_type: ClickType,
}

/// Button values for normal mouse clicks.
const BUTTON_CLICK_LEFT: i8 = 0;
const BUTTON_CLICK_RIGHT: i8 = 1;

/// Key code for offhand swap (F key by default).
const KEY_CLICK_OFFHAND: i8 = 40;

/// Hotbar slot key range (1-9 keys).
const KEY_CLICK_HOTBAR_START: i8 = 0;
const KEY_CLICK_HOTBAR_END: i8 = 9;

/// Slot index indicating a click outside the inventory.
const SLOT_INDEX_OUTSIDE: i16 = -999;

impl Click {
    /// Parses a slot action into a click event.
    ///
    /// # Arguments
    /// - `mode` - The action type from the protocol
    /// - `button` - The button value from the protocol
    /// - `slot` - The slot index (-999 for outside)
    ///
    /// # Returns
    /// The parsed click or an error if invalid.
    ///
    /// # Errors
    /// Returns [`InventoryError::InvalidSlot`] or [`InventoryError::InvalidPacket`]
    /// for malformed input.
    pub fn new(mode: &SlotActionType, button: i8, slot: i16) -> Result<Self, InventoryError> {
        match mode {
            SlotActionType::Pickup => Self::new_normal_click(button, slot),
            // Both buttons do the same here, so we omit it
            SlotActionType::QuickMove => Self::new_shift_click(slot),
            SlotActionType::Swap => Self::new_key_click(button, slot),
            SlotActionType::Clone => Ok(Self {
                click_type: ClickType::CreativePickItem,
                slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
            }),
            SlotActionType::Throw => Self::new_drop_item(button, slot),
            SlotActionType::QuickCraft => Self::new_drag_item(button, slot),
            SlotActionType::PickupAll => Ok(Self {
                click_type: ClickType::DoubleClick,
                slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
            }),
        }
    }

    fn new_normal_click(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let slot = if slot == SLOT_INDEX_OUTSIDE {
            Slot::OutsideInventory
        } else {
            let slot = slot.try_into().unwrap_or(0);
            Slot::Normal(slot)
        };
        let button = match button {
            BUTTON_CLICK_LEFT => MouseClick::Left,
            BUTTON_CLICK_RIGHT => MouseClick::Right,
            _ => Err(InventoryError::InvalidPacket)?,
        };
        Ok(Self {
            click_type: ClickType::MouseClick(button),
            slot,
        })
    }

    fn new_shift_click(slot: i16) -> Result<Self, InventoryError> {
        Ok(Self {
            slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
            click_type: ClickType::ShiftClick,
        })
    }

    fn new_key_click(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let key = match button {
            KEY_CLICK_HOTBAR_START..KEY_CLICK_HOTBAR_END => {
                KeyClick::Slot(button.try_into().or(Err(InventoryError::InvalidSlot))?)
            }
            KEY_CLICK_OFFHAND => KeyClick::Offhand,
            _ => Err(InventoryError::InvalidSlot)?,
        };

        Ok(Self {
            click_type: ClickType::KeyClick(key),
            slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
        })
    }

    fn new_drop_item(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let drop_type = DropType::from_i8(button)?;
        let slot = if slot == SLOT_INDEX_OUTSIDE {
            Slot::OutsideInventory
        } else {
            let slot = slot.try_into().unwrap_or(0);
            Slot::Normal(slot)
        };
        Ok(Self {
            click_type: ClickType::DropType(drop_type),
            slot,
        })
    }

    fn new_drag_item(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let state = match button {
            0 => MouseDragState::Start(MouseDragType::Left),
            4 => MouseDragState::Start(MouseDragType::Right),
            8 => MouseDragState::Start(MouseDragType::Middle),
            1 | 5 | 9 => {
                MouseDragState::AddSlot(slot.try_into().or(Err(InventoryError::InvalidSlot))?)
            }
            2 | 6 | 10 => MouseDragState::End,
            _ => Err(InventoryError::InvalidPacket)?,
        };
        Ok(Self {
            slot: match &state {
                MouseDragState::AddSlot(slot) => Slot::Normal(*slot),
                _ => Slot::OutsideInventory,
            },
            click_type: ClickType::MouseDrag { drag_state: state },
        })
    }
}

/// The type of click action.
#[derive(Debug)]
pub enum ClickType {
    /// Normal mouse click (left or right).
    MouseClick(MouseClick),
    /// Shift-click to quick-move an item.
    ShiftClick,
    /// Hotbar key press (1-9 or offhand swap).
    KeyClick(KeyClick),
    /// Creative mode middle-click (pick block).
    CreativePickItem,
    /// Drop item (Q key or drop click).
    DropType(DropType),
    /// Drag items across multiple slots.
    MouseDrag { drag_state: MouseDragState },
    /// Double-click to gather items.
    DoubleClick,
}

/// Normal mouse button clicks.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MouseClick {
    Left,
    Right,
}

/// Hotbar key clicks.
#[derive(Debug)]
pub enum KeyClick {
    /// Hotbar slot index (0-8).
    Slot(u8),
    /// Swap with offhand.
    Offhand,
}

/// A slot reference - either a specific slot or outside the inventory.
#[derive(Debug, Copy, Clone)]
pub enum Slot {
    Normal(usize),
    OutsideInventory,
}

/// Drop action types.
#[derive(Debug)]
pub enum DropType {
    /// Drop a single item (Ctrl+Q).
    SingleItem,
    /// Drop the full stack (Q).
    FullStack,
}

impl DropType {
    const fn from_i8(value: i8) -> Result<Self, InventoryError> {
        Ok(match value {
            0 => Self::SingleItem,
            1 => Self::FullStack,
            _ => return Err(InventoryError::InvalidPacket),
        })
    }
}

/// Mouse drag button types.
#[derive(Debug, PartialEq, Eq)]
pub enum MouseDragType {
    /// Left button drag - even distribution.
    Left,
    /// Right button drag - one item per slot.
    Right,
    /// Middle button drag - create full stacks (creative only).
    Middle,
}

/// Drag operation state.
#[derive(PartialEq, Eq, Debug)]
pub enum MouseDragState {
    /// Start of drag - button determines drag type.
    Start(MouseDragType),
    /// Adding a slot to the drag.
    AddSlot(usize),
    /// End of drag - apply to all selected slots.
    End,
}

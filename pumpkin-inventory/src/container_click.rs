use crate::InventoryError;
use pumpkin_protocol::java::server::play::SlotActionType;

#[derive(Debug)]
pub struct Click {
    pub slot: Slot,
    pub click_type: ClickType,
}

const BUTTON_CLICK_LEFT: i8 = 0;
const BUTTON_CLICK_RIGHT: i8 = 1;

const KEY_CLICK_OFFHAND: i8 = 40;
const KEY_CLICK_HOTBAR_START: i8 = 0;
const KEY_CLICK_HOTBAR_END: i8 = 9;

const SLOT_INDEX_OUTSIDE: i16 = -999;

impl Click {
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

#[derive(Debug)]
pub enum ClickType {
    MouseClick(MouseClick),
    ShiftClick,
    KeyClick(KeyClick),
    CreativePickItem,
    DropType(DropType),
    MouseDrag { drag_state: MouseDragState },
    DoubleClick,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MouseClick {
    Left,
    Right,
}

#[derive(Debug)]
pub enum KeyClick {
    Slot(u8),
    Offhand,
}
#[derive(Debug, Copy, Clone)]
pub enum Slot {
    Normal(usize),
    OutsideInventory,
}

#[derive(Debug)]
pub enum DropType {
    SingleItem,
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

#[derive(Debug, PartialEq, Eq)]
pub enum MouseDragType {
    Left,
    Right,
    Middle,
}
#[derive(PartialEq, Eq, Debug)]
pub enum MouseDragState {
    Start(MouseDragType),
    AddSlot(usize),
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn click(mode: &SlotActionType, button: i8, slot: i16) -> Result<Click, InventoryError> {
        Click::new(mode, button, slot)
    }

    #[test]
    fn normal_left_click_on_slot() {
        let c = click(&SlotActionType::Pickup, 0, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseClick(MouseClick::Left)
        ));
        assert!(matches!(c.slot, Slot::Normal(5)));
    }

    #[test]
    fn normal_right_click_on_slot() {
        let c = click(&SlotActionType::Pickup, 1, 10).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseClick(MouseClick::Right)
        ));
        assert!(matches!(c.slot, Slot::Normal(10)));
    }

    #[test]
    fn normal_click_outside_inventory() {
        let c = click(&SlotActionType::Pickup, 0, -999).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseClick(MouseClick::Left)
        ));
        assert!(matches!(c.slot, Slot::OutsideInventory));
    }

    #[test]
    fn normal_click_invalid_button() {
        let result = click(&SlotActionType::Pickup, 2, 5);
        assert!(result.is_err());
    }

    #[test]
    fn shift_click_normal_slot() {
        let c = click(&SlotActionType::QuickMove, 0, 3).unwrap();
        assert!(matches!(c.click_type, ClickType::ShiftClick));
        assert!(matches!(c.slot, Slot::Normal(3)));
    }

    #[test]
    fn key_click_hotbar_slot_0() {
        let c = click(&SlotActionType::Swap, 0, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::KeyClick(KeyClick::Slot(0))
        ));
        assert!(matches!(c.slot, Slot::Normal(5)));
    }

    #[test]
    fn key_click_hotbar_slot_8() {
        let c = click(&SlotActionType::Swap, 8, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::KeyClick(KeyClick::Slot(8))
        ));
    }

    #[test]
    fn key_click_offhand() {
        let c = click(&SlotActionType::Swap, 40, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::KeyClick(KeyClick::Offhand)
        ));
    }

    #[test]
    fn key_click_invalid_button() {
        let result = click(&SlotActionType::Swap, 41, 5);
        assert!(result.is_err());
    }

    #[test]
    fn creative_pick_item() {
        let c = click(&SlotActionType::Clone, 2, 7).unwrap();
        assert!(matches!(c.click_type, ClickType::CreativePickItem));
        assert!(matches!(c.slot, Slot::Normal(7)));
    }

    #[test]
    fn drop_single_item() {
        let c = click(&SlotActionType::Throw, 0, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::DropType(DropType::SingleItem)
        ));
    }

    #[test]
    fn drop_full_stack() {
        let c = click(&SlotActionType::Throw, 1, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::DropType(DropType::FullStack)
        ));
    }

    #[test]
    fn drop_invalid_button() {
        let result = click(&SlotActionType::Throw, 2, 5);
        assert!(result.is_err());
    }

    #[test]
    fn drag_start_left() {
        let c = click(&SlotActionType::QuickCraft, 0, -999).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseDrag {
                drag_state: MouseDragState::Start(MouseDragType::Left)
            }
        ));
    }

    #[test]
    fn drag_start_right() {
        let c = click(&SlotActionType::QuickCraft, 4, -999).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseDrag {
                drag_state: MouseDragState::Start(MouseDragType::Right)
            }
        ));
    }

    #[test]
    fn drag_start_middle() {
        let c = click(&SlotActionType::QuickCraft, 8, -999).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseDrag {
                drag_state: MouseDragState::Start(MouseDragType::Middle)
            }
        ));
    }

    #[test]
    fn drag_add_slot() {
        let c = click(&SlotActionType::QuickCraft, 1, 5).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseDrag {
                drag_state: MouseDragState::AddSlot(5)
            }
        ));
        assert!(matches!(c.slot, Slot::Normal(5)));
    }

    #[test]
    fn drag_end() {
        let c = click(&SlotActionType::QuickCraft, 2, -999).unwrap();
        assert!(matches!(
            c.click_type,
            ClickType::MouseDrag {
                drag_state: MouseDragState::End
            }
        ));
    }

    #[test]
    fn drag_invalid_button() {
        let result = click(&SlotActionType::QuickCraft, 3, -999);
        assert!(result.is_err());
    }

    #[test]
    fn double_click() {
        let c = click(&SlotActionType::PickupAll, 0, 5).unwrap();
        assert!(matches!(c.click_type, ClickType::DoubleClick));
        assert!(matches!(c.slot, Slot::Normal(5)));
    }
}

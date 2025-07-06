//use pumpkin_world::inventory::Inventory;

pub mod inventory_click;

/* /// A trait representing events related to inventories.
///
/// This trait provides a method to retrieve the inventory associated with the event.
pub trait InventoryEvent: Send + Sync {
    /// Retrieves a reference to the inventory associated with the event.
    ///
    /// # Returns
    /// A reference to the `Arc<dyn Inventory>` involved in the event.
    fn get_inventory(&self) -> &Arc<dyn Inventory>;
} */

// TODO: implement all these inventory types
/// A collection of all possible types of inventories.
#[derive(Clone)]
pub enum InventoryType {
    Chest,
    Dispenser,
    Dropper,
    Furnace,
    Workbench,
    Crafting,
    Enchanting,
    Brewing,
    Player,
    Creative,
    Merchant,
    EnderChest,
    Anvil,
    Smithing,
    Beacon,
    Hopper,
    ShulkerBox,
    Barrel,
    BlastFurnace,
    Lectern,
    Smoker,
    Loom,
    Cartography,
    Grindstone,
    Stonecutter,
    Composter,
    ChiseledBookshelf,
    Jukebox,
    DecoratedPot,
    Crafter,
}

#[derive(Clone)]
pub enum SlotType {
    Result,
    Crafting,
    Armor,
    Container,
    Quickbar,
    Outside,
    Fuel,
}

#[derive(Clone)]
pub enum ClickType {
    Left,
    ShiftLeft,
    Right,
    ShiftRight,
    WindowBorderLeft,
    WindowBorderRight,
    Middle,
    NumberKey,
    DoubleClick,
    Drop,
    ControlDrop,
    Creative,
    SwapOffhand,
    Unknown,
}

impl ClickType {
    #[must_use]
    pub fn is_keyboard_click(&self) -> bool {
        matches!(
            self,
            Self::NumberKey | Self::Drop | Self::ControlDrop | Self::SwapOffhand
        )
    }

    #[must_use]
    pub fn is_mouse_click(&self) -> bool {
        matches!(
            self,
            Self::DoubleClick
                | Self::Left
                | Self::Right
                | Self::Middle
                | Self::WindowBorderLeft
                | Self::WindowBorderRight
                | Self::ShiftLeft
                | Self::ShiftRight
        )
    }

    #[must_use]
    pub fn is_creative_action(&self) -> bool {
        matches!(self, Self::Middle | Self::Creative)
    }

    #[must_use]
    pub fn is_right_click(&self) -> bool {
        matches!(self, Self::Right | Self::ShiftRight)
    }

    #[must_use]
    pub fn is_left_click(&self) -> bool {
        matches!(
            self,
            Self::Left | Self::ShiftLeft | Self::DoubleClick | Self::Creative
        )
    }

    #[must_use]
    pub fn is_shift_click(&self) -> bool {
        matches!(self, Self::ShiftLeft | Self::ShiftRight)
    }
}

#[derive(Clone)]
pub enum InventoryAction {
    Nothing,
    PickupAll,
    PickupSome,
    PickupHalf,
    PickupOne,
    PlaceAll,
    PlaceSome,
    PlaceOne,
    SwapWithCursor,
    DropAllCursor,
    DropOneCursor,
    DropAllSlot,
    DropOneSlot,
    MoveToOtherInventory,
    HotbarSwap,
    CloneStack,
    CollectToCursor,
    PickupFromBundle,
    PickupAllIntoBundle,
    PickupSomeIntoBundle,
    PlaceFromBundle,
    PlaceAllIntoBundle,
    PlaceSomeIntoBundle,
    Unknown,
}

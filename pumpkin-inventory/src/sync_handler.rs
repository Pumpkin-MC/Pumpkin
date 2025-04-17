use pumpkin_world::item::ItemStack;

use crate::screen_handler::ScreenHandler;

pub struct SyncHandler {}

impl SyncHandler {
    pub fn update_state(
        &self,
        screen_handler: &dyn ScreenHandler,
        stacks: &[ItemStack],
        cursor_stack: &ItemStack,
        properties: Vec<i32>,
    ) {
    }

    pub fn update_slot(&self, screen_handler: &dyn ScreenHandler, slot: usize, stack: &ItemStack) {}

    pub fn update_cursor_stack(&self, screen_handler: &dyn ScreenHandler, stack: &ItemStack) {}

    pub fn update_property(&self, screen_handler: &dyn ScreenHandler, property: i32, value: i32) {}

    pub fn send_property_update(
        &self,
        screen_handler: &dyn ScreenHandler,
        property: i32,
        value: i32,
    ) {
    }

    pub fn create_tracked_slot(&self) {}
}

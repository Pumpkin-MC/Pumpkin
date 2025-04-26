use pumpkin_world::item::ItemStack;

use crate::screen_handler::DefaultScreenHandlerBehaviour;

pub struct SyncHandler {}

impl SyncHandler {
    pub async fn update_state(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        stacks: &Vec<ItemStack>,
        cursor_stack: &ItemStack,
        properties: Vec<i32>,
    ) {
    }

    pub async fn update_slot(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        slot: usize,
        stack: &ItemStack,
    ) {
    }

    pub async fn update_cursor_stack(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        stack: &ItemStack,
    ) {
    }

    pub async fn update_property(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        property: i32,
        value: i32,
    ) {
    }
}

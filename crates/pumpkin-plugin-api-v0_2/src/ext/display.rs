#![allow(async_fn_in_trait)]

use crate::host::{
    display::{
        BlockDisplayEntity, DisplayEntity, InteractionEntity, ItemDisplayEntity, TextDisplayEntity,
        Vector3f,
    },
    item_stack::ItemStack,
    text::TextComponent,
    world::Entity,
};

/// Gets a typed display or interaction handle from a borrowed [`Entity`]
pub trait EntityDisplayExt {
    /// Returns a [`DisplayEntity`] handle for this entity, or `None` if it is not one
    fn as_display(&self) -> Option<DisplayEntity>;

    /// Returns a [`BlockDisplayEntity`] handle for this entity, or `None` if it is not one
    fn as_block_display(&self) -> Option<BlockDisplayEntity>;

    /// Returns a [`ItemDisplayEntity`] handle for this entity, or `None` if it is not one
    fn as_item_display(&self) -> Option<ItemDisplayEntity>;

    /// Returns a [`TextDisplayEntity`] handle for this entity, or `None` if it is not one
    fn as_text_display(&self) -> Option<TextDisplayEntity>;

    /// Returns a [`InteractionEntity`] handle for this entity, or `None` if it is not one
    fn as_interaction(&self) -> Option<InteractionEntity>;
}

impl EntityDisplayExt for Entity {
    fn as_display(&self) -> Option<DisplayEntity> {
        DisplayEntity::from_entity(self)
    }

    fn as_block_display(&self) -> Option<BlockDisplayEntity> {
        BlockDisplayEntity::from_entity(self)
    }

    fn as_item_display(&self) -> Option<ItemDisplayEntity> {
        ItemDisplayEntity::from_entity(self)
    }

    fn as_text_display(&self) -> Option<TextDisplayEntity> {
        TextDisplayEntity::from_entity(self)
    }

    fn as_interaction(&self) -> Option<InteractionEntity> {
        InteractionEntity::from_entity(self)
    }
}

/// Extension trait for [`DisplayEntity`] providing convenience mutation helpers.
pub trait DisplayEntityExt {
    /// Sets translation directly without manually constructing a `DisplayTransformation`.
    fn set_translation(&self, x: f32, y: f32, z: f32);
    /// Sets scale directly without manually constructing a `DisplayTransformation`.
    fn set_scale(&self, x: f32, y: f32, z: f32);
}

impl DisplayEntityExt for DisplayEntity {
    fn set_translation(&self, x: f32, y: f32, z: f32) {
        let mut transform = self.get_transformation();
        transform.translation = Vector3f { x, y, z };
        self.set_transformation(transform);
    }

    fn set_scale(&self, x: f32, y: f32, z: f32) {
        let mut transform = self.get_transformation();
        transform.scale = Vector3f { x, y, z };
        self.set_transformation(transform);
    }
}

/// Extension trait for [`ItemDisplayEntity`] providing convenience methods.
pub trait ItemDisplayEntityExt {
    /// Sets the displayed item stack.
    async fn set_item_stack(&self, item: Option<ItemStack>);
}

impl ItemDisplayEntityExt for ItemDisplayEntity {
    async fn set_item_stack(&self, item: Option<ItemStack>) {
        self.set_item(item).await;
    }
}

/// Extension trait for [`TextDisplayEntity`] providing convenience methods.
pub trait TextDisplayEntityExt {
    /// Sets text from a plain string.
    fn set_plain_text(&self, text: &str);
}

impl TextDisplayEntityExt for TextDisplayEntity {
    fn set_plain_text(&self, text: &str) {
        self.set_text(TextComponent::text(text));
    }
}

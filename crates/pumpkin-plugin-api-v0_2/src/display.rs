//! Borrowed views of an entity as a display or interaction entity
//!
//! A successful cast returns an owned typed handle to the same host entity
//! The original entity handle remains usable whether the cast succeeds or fails

use crate::host::{
    display::{
        BlockDisplayEntity, DisplayEntity, InteractionEntity, ItemDisplayEntity, TextDisplayEntity,
    },
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

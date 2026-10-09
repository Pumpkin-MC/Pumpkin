//! Ownership-preserving conversions from an entity to a display or interaction entity
//!
//! A conversion in the WIT consumes the entity handle and returns `result<converted, entity>`: on
//! success the entity belongs to the converted resource, and on failure the original handle comes
//! back in the error
//!
//! v0.1 returned an `Option` from a borrow here, which could not give the handle back, so this is
//! the difference to watch for when you migrate

use crate::host::{
    display::{
        BlockDisplayEntity, DisplayEntity, InteractionEntity, ItemDisplayEntity, TextDisplayEntity,
    },
    world::Entity,
};

/// Converts an owned [`Entity`] into a display or interaction entity
///
/// The methods take `self` because borrowing would let two handles to one entity exist at once,
/// and the error path gives the handle back instead of dropping it
///
/// # Examples
///
/// ```rust,ignore
/// match entity.into_text_display_entity() {
///     Ok(text) => text.set_text(message),
///     // not a text display, so the entity is still ours to use
///     Err(entity) => other_handling(entity),
/// }
/// ```
pub trait EntityDisplayExt: Sized {
    /// Converts into a [`DisplayEntity`]
    ///
    /// # Errors
    ///
    /// Returns the original entity when it is not a display entity
    fn into_display_entity(self) -> Result<DisplayEntity, Entity>;

    /// Converts into a [`BlockDisplayEntity`]
    ///
    /// # Errors
    ///
    /// Returns the original entity when it is not a block display
    fn into_block_display_entity(self) -> Result<BlockDisplayEntity, Entity>;

    /// Converts into an [`ItemDisplayEntity`]
    ///
    /// # Errors
    ///
    /// Returns the original entity when it is not an item display
    fn into_item_display_entity(self) -> Result<ItemDisplayEntity, Entity>;

    /// Converts into a [`TextDisplayEntity`]
    ///
    /// # Errors
    ///
    /// Returns the original entity when it is not a text display
    fn into_text_display_entity(self) -> Result<TextDisplayEntity, Entity>;

    /// Converts into an [`InteractionEntity`]
    ///
    /// # Errors
    ///
    /// Returns the original entity when it is not an interaction entity
    fn into_interaction_entity(self) -> Result<InteractionEntity, Entity>;
}

impl EntityDisplayExt for Entity {
    fn into_display_entity(self) -> Result<DisplayEntity, Entity> {
        DisplayEntity::from_entity(self)
    }

    fn into_block_display_entity(self) -> Result<BlockDisplayEntity, Entity> {
        BlockDisplayEntity::from_entity(self)
    }

    fn into_item_display_entity(self) -> Result<ItemDisplayEntity, Entity> {
        ItemDisplayEntity::from_entity(self)
    }

    fn into_text_display_entity(self) -> Result<TextDisplayEntity, Entity> {
        TextDisplayEntity::from_entity(self)
    }

    fn into_interaction_entity(self) -> Result<InteractionEntity, Entity> {
        InteractionEntity::from_entity(self)
    }
}

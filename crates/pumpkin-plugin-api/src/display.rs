pub use crate::wit::pumpkin::plugin::display::{
    BillboardMode, BlockDisplayEntity, DisplayEntity, DisplayTransformation, InteractionEntity,
    ItemDisplayEntity, ItemDisplayMode, Quaternionf, TextAlignment, TextDisplayEntity, Vector3f,
};
use crate::wit::pumpkin::plugin::item_stack::ItemStack;
use crate::wit::pumpkin::plugin::text::TextComponent;
use crate::wit::pumpkin::plugin::world::Entity;

/// Builder for constructing [`DisplayTransformation`].
#[derive(Clone, Copy, Debug)]
pub struct TransformationBuilder {
    translation: Vector3f,
    scale: Vector3f,
    left_rotation: Quaternionf,
    right_rotation: Quaternionf,
}

impl Default for TransformationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformationBuilder {
    /// Creates a new identity `TransformationBuilder`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            translation: Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            scale: Vector3f {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            left_rotation: Quaternionf {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            right_rotation: Quaternionf {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
        }
    }

    /// Sets the translation vector.
    #[must_use]
    pub const fn translation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.translation = Vector3f { x, y, z };
        self
    }

    /// Sets the scale vector.
    #[must_use]
    pub const fn scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = Vector3f { x, y, z };
        self
    }

    /// Sets uniform scale across all axes.
    #[must_use]
    pub const fn uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vector3f {
            x: scale,
            y: scale,
            z: scale,
        };
        self
    }

    /// Sets the left rotation quaternion.
    #[must_use]
    pub const fn left_rotation(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.left_rotation = Quaternionf { x, y, z, w };
        self
    }

    /// Sets the right rotation quaternion.
    #[must_use]
    pub const fn right_rotation(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.right_rotation = Quaternionf { x, y, z, w };
        self
    }

    /// Builds the [`DisplayTransformation`].
    #[must_use]
    pub const fn build(self) -> DisplayTransformation {
        DisplayTransformation {
            translation: self.translation,
            scale: self.scale,
            left_rotation: self.left_rotation,
            right_rotation: self.right_rotation,
        }
    }
}

/// Extension trait on generic [`Entity`] for downcasting to specialized Display / Interaction resources.
pub trait EntityDisplayExt {
    /// Attempts to view this entity as a base [`DisplayEntity`].
    fn as_display(&self) -> Option<DisplayEntity>;
    /// Attempts to view this entity as a [`BlockDisplayEntity`].
    fn as_block_display(&self) -> Option<BlockDisplayEntity>;
    /// Attempts to view this entity as an [`ItemDisplayEntity`].
    fn as_item_display(&self) -> Option<ItemDisplayEntity>;
    /// Attempts to view this entity as a [`TextDisplayEntity`].
    fn as_text_display(&self) -> Option<TextDisplayEntity>;
    /// Attempts to view this entity as an [`InteractionEntity`].
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
    fn set_item_stack(&self, item: Option<ItemStack>);
}

impl ItemDisplayEntityExt for ItemDisplayEntity {
    fn set_item_stack(&self, item: Option<ItemStack>) {
        self.set_item(item);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformation_builder() {
        let transform = TransformationBuilder::new()
            .translation(1.0, 2.0, 3.0)
            .scale(2.0, 2.0, 2.0)
            .left_rotation(0.0, 0.7071, 0.0, 0.7071)
            .build();

        assert_eq!(transform.translation.x, 1.0);
        assert_eq!(transform.translation.y, 2.0);
        assert_eq!(transform.translation.z, 3.0);
        assert_eq!(transform.scale.x, 2.0);
        assert_eq!(transform.scale.y, 2.0);
        assert_eq!(transform.scale.z, 2.0);
        assert_eq!(transform.left_rotation.y, 0.7071);
    }
}

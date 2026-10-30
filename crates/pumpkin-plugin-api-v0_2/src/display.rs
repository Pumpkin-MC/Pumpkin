//! Display resources and transformation builders.

pub use crate::wit::pumpkin::plugin::display::{
    BillboardMode, BlockDisplayEntity, DisplayEntity, DisplayTransformation, InteractionEntity,
    ItemDisplayEntity, ItemDisplayMode, Quaternionf, TextAlignment, TextDisplayEntity, Vector3f,
};

pub use crate::ext::display::{
    DisplayEntityExt, EntityDisplayExt, ItemDisplayEntityExt, TextDisplayEntityExt,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformation_builder() {
        let transform = TransformationBuilder::new()
            .translation(1.0, 2.0, 3.0)
            .scale(2.0, 2.0, 2.0)
            .left_rotation(
                0.0,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0,
                std::f32::consts::FRAC_1_SQRT_2,
            )
            .build();

        assert_eq!(transform.translation.x, 1.0);
        assert_eq!(transform.translation.y, 2.0);
        assert_eq!(transform.translation.z, 3.0);
        assert_eq!(transform.scale.x, 2.0);
        assert_eq!(transform.scale.y, 2.0);
        assert_eq!(transform.scale.z, 2.0);
        assert_eq!(transform.left_rotation.y, std::f32::consts::FRAC_1_SQRT_2);
    }
}

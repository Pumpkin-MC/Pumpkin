use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::java::client::play::{Metadata, MetadataSerializer};
use pumpkin_protocol::ser::{NetworkWriteExt, WritingError};
use pumpkin_util::math::atomic_f32::AtomicF32;

use crate::entity::{Entity, NBTStorage, NbtFuture};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
}

impl MetadataSerializer for Vector3f {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_f32(self.x)?;
        writer.write_f32(self.y)?;
        writer.write_f32(self.z)
    }
}

impl From<Vector3f> for NbtTag {
    fn from(val: Vector3f) -> Self {
        Self::List(vec![
            Self::Float(val.x),
            Self::Float(val.y),
            Self::Float(val.z),
        ])
    }
}

impl From<&NbtTag> for Vector3f {
    fn from(tag: &NbtTag) -> Self {
        if let NbtTag::List(list) = tag
            && list.len() == 3
            && let (NbtTag::Float(x), NbtTag::Float(y), NbtTag::Float(z)) =
                (&list[0], &list[1], &list[2])
        {
            return Self {
                x: *x,
                y: *y,
                z: *z,
            };
        }
        Self::ZERO
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };
}

impl MetadataSerializer for Quaternion {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_f32(self.x)?;
        writer.write_f32(self.y)?;
        writer.write_f32(self.z)?;
        writer.write_f32(self.w)
    }
}

impl From<Quaternion> for NbtTag {
    fn from(val: Quaternion) -> Self {
        Self::List(vec![
            Self::Float(val.x),
            Self::Float(val.y),
            Self::Float(val.z),
            Self::Float(val.w),
        ])
    }
}

impl From<&NbtTag> for Quaternion {
    fn from(tag: &NbtTag) -> Self {
        if let NbtTag::List(list) = tag
            && list.len() == 4
            && let (NbtTag::Float(x), NbtTag::Float(y), NbtTag::Float(z), NbtTag::Float(w)) =
                (&list[0], &list[1], &list[2], &list[3])
        {
            return Self {
                x: *x,
                y: *y,
                z: *z,
                w: *w,
            };
        }
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Billboard {
    Fixed,
    Vertical,
    Horizontal,
    Center,
}

impl Billboard {
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Self::Fixed => 0,
            Self::Vertical => 1,
            Self::Horizontal => 2,
            Self::Center => 3,
        }
    }

    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Vertical,
            2 => Self::Horizontal,
            3 => Self::Center,
            _ => Self::Fixed,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Center => "center",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "vertical" => Self::Vertical,
            "horizontal" => Self::Horizontal,
            "center" => Self::Center,
            _ => Self::Fixed,
        }
    }
}

/// Shared fields of `Display.java`.
///
/// Interpolation timing, the affine transform (translation/left-rotation/scale/
/// right-rotation), billboard mode, brightness override, culling extents and glow color
/// override.
pub struct DisplayEntity {
    pub entity: Entity,
    transformation_interpolation_start_delta_ticks: AtomicI32,
    transformation_interpolation_duration: AtomicI32,
    pos_rot_interpolation_duration: AtomicI32,
    translation: AtomicCell<Vector3f>,
    scale: AtomicCell<Vector3f>,
    left_rotation: AtomicCell<Quaternion>,
    right_rotation: AtomicCell<Quaternion>,
    billboard: AtomicU8,
    brightness_override: AtomicI32,
    view_range: AtomicF32,
    shadow_radius: AtomicF32,
    shadow_strength: AtomicF32,
    width: AtomicF32,
    height: AtomicF32,
    glow_color_override: AtomicI32,
}

impl DisplayEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            transformation_interpolation_start_delta_ticks: AtomicI32::new(0),
            transformation_interpolation_duration: AtomicI32::new(0),
            pos_rot_interpolation_duration: AtomicI32::new(0),
            translation: AtomicCell::new(Vector3f::ZERO),
            scale: AtomicCell::new(Vector3f::ONE),
            left_rotation: AtomicCell::new(Quaternion::IDENTITY),
            right_rotation: AtomicCell::new(Quaternion::IDENTITY),
            billboard: AtomicU8::new(Billboard::Fixed.id()),
            brightness_override: AtomicI32::new(-1),
            view_range: AtomicF32::new(1.0),
            shadow_radius: AtomicF32::new(0.0),
            shadow_strength: AtomicF32::new(1.0),
            width: AtomicF32::new(0.0),
            height: AtomicF32::new(0.0),
            glow_color_override: AtomicI32::new(-1),
        }
    }

    /// Sends the full common `Display` tracked-data state.
    ///
    /// Called from each concrete display's `init_data_tracker`, both for
    /// freshly summoned entities (default values) and after NBT load.
    pub fn send_metadata(&self) {
        self.send_int_metadata();
        self.send_vector_and_rotation_metadata();
        self.send_billboard_and_float_metadata();
    }

    fn send_int_metadata(&self) {
        let ints = [
            Metadata::new(
                TrackedData::START_INTERPOLATION,
                MetaDataType::INT,
                self.transformation_interpolation_start_delta_ticks
                    .load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID,
                MetaDataType::INT,
                self.transformation_interpolation_start_delta_ticks
                    .load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::INTERPOLATION_DURATION,
                MetaDataType::INT,
                self.transformation_interpolation_duration
                    .load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::TRANSFORMATION_INTERPOLATION_DURATION_ID,
                MetaDataType::INT,
                self.transformation_interpolation_duration
                    .load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::POS_ROT_INTERPOLATION_DURATION_ID,
                MetaDataType::INT,
                self.pos_rot_interpolation_duration.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::BRIGHTNESS,
                MetaDataType::INT,
                self.brightness_override.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::BRIGHTNESS_OVERRIDE_ID,
                MetaDataType::INT,
                self.brightness_override.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::GLOW_COLOR_OVERRIDE,
                MetaDataType::INT,
                self.glow_color_override.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::GLOW_COLOR_OVERRIDE_ID,
                MetaDataType::INT,
                self.glow_color_override.load(Ordering::Relaxed),
            ),
        ];
        self.entity.send_meta_data(&ints, None);
    }

    fn send_vector_and_rotation_metadata(&self) {
        let translation = self.translation.load();
        let scale = self.scale.load();
        let vectors = [
            Metadata::new(TrackedData::TRANSLATION, MetaDataType::VECTOR3, translation),
            Metadata::new(
                TrackedData::TRANSLATION_ID,
                MetaDataType::VECTOR3,
                translation,
            ),
            Metadata::new(TrackedData::SCALE, MetaDataType::VECTOR3, scale),
            Metadata::new(TrackedData::SCALE_ID, MetaDataType::VECTOR3, scale),
        ];
        self.entity.send_meta_data(&vectors, None);

        let left_rotation = self.left_rotation.load();
        let right_rotation = self.right_rotation.load();
        let quaternions = [
            Metadata::new(
                TrackedData::LEFT_ROTATION,
                MetaDataType::QUATERNION,
                left_rotation,
            ),
            Metadata::new(
                TrackedData::LEFT_ROTATION_ID,
                MetaDataType::QUATERNION,
                left_rotation,
            ),
            Metadata::new(
                TrackedData::RIGHT_ROTATION,
                MetaDataType::QUATERNION,
                right_rotation,
            ),
            Metadata::new(
                TrackedData::RIGHT_ROTATION_ID,
                MetaDataType::QUATERNION,
                right_rotation,
            ),
        ];
        self.entity.send_meta_data(&quaternions, None);
    }

    fn send_billboard_and_float_metadata(&self) {
        let billboard = self.billboard.load(Ordering::Relaxed);
        let bytes = [
            Metadata::new(TrackedData::BILLBOARD, MetaDataType::BYTE, billboard),
            Metadata::new(
                TrackedData::BILLBOARD_RENDER_CONSTRAINTS_ID,
                MetaDataType::BYTE,
                billboard,
            ),
        ];
        self.entity.send_meta_data(&bytes, None);

        let floats = [
            Metadata::new(
                TrackedData::VIEW_RANGE,
                MetaDataType::FLOAT,
                self.view_range.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::VIEW_RANGE_ID,
                MetaDataType::FLOAT,
                self.view_range.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::SHADOW_RADIUS,
                MetaDataType::FLOAT,
                self.shadow_radius.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::SHADOW_RADIUS_ID,
                MetaDataType::FLOAT,
                self.shadow_radius.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::SHADOW_STRENGTH,
                MetaDataType::FLOAT,
                self.shadow_strength.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::SHADOW_STRENGTH_ID,
                MetaDataType::FLOAT,
                self.shadow_strength.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::WIDTH,
                MetaDataType::FLOAT,
                self.width.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::WIDTH_ID,
                MetaDataType::FLOAT,
                self.width.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::HEIGHT,
                MetaDataType::FLOAT,
                self.height.load(Ordering::Relaxed),
            ),
            Metadata::new(
                TrackedData::HEIGHT_ID,
                MetaDataType::FLOAT,
                self.height.load(Ordering::Relaxed),
            ),
        ];
        self.entity.send_meta_data(&floats, None);
    }

    pub fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;

            let mut transformation = NbtCompound::new();
            transformation.put("translation", self.translation.load());
            transformation.put("left_rotation", self.left_rotation.load());
            transformation.put("scale", self.scale.load());
            transformation.put("right_rotation", self.right_rotation.load());
            nbt.put_compound("transformation", transformation);

            nbt.put_int(
                "interpolation_duration",
                self.transformation_interpolation_duration
                    .load(Ordering::Relaxed),
            );
            nbt.put_int(
                "start_interpolation",
                self.transformation_interpolation_start_delta_ticks
                    .load(Ordering::Relaxed),
            );
            nbt.put_int(
                "teleport_duration",
                self.pos_rot_interpolation_duration.load(Ordering::Relaxed),
            );
            nbt.put_string(
                "billboard",
                Billboard::from_id(self.billboard.load(Ordering::Relaxed))
                    .name()
                    .to_string(),
            );
            nbt.put_float("view_range", self.view_range.load(Ordering::Relaxed));
            nbt.put_float("shadow_radius", self.shadow_radius.load(Ordering::Relaxed));
            nbt.put_float(
                "shadow_strength",
                self.shadow_strength.load(Ordering::Relaxed),
            );
            nbt.put_float("width", self.width.load(Ordering::Relaxed));
            nbt.put_float("height", self.height.load(Ordering::Relaxed));
            nbt.put_int(
                "glow_color_override",
                self.glow_color_override.load(Ordering::Relaxed),
            );

            let brightness_override = self.brightness_override.load(Ordering::Relaxed);
            if brightness_override != -1 {
                let mut brightness = NbtCompound::new();
                // Brightness.pack/unpack in Display.java: block << 4 | sky << 20
                brightness.put_int("block", (brightness_override >> 4) & 15);
                brightness.put_int("sky", (brightness_override >> 20) & 15);
                nbt.put_compound("brightness", brightness);
            }
        })
    }

    pub fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.read_nbt_non_mut(nbt).await;

            if let Some(transformation) = nbt.get_compound("transformation") {
                self.translation.store(
                    transformation
                        .get("translation")
                        .map_or(Vector3f::ZERO, Vector3f::from),
                );
                self.left_rotation.store(
                    transformation
                        .get("left_rotation")
                        .map_or(Quaternion::IDENTITY, Quaternion::from),
                );
                self.scale.store(
                    transformation
                        .get("scale")
                        .map_or(Vector3f::ONE, Vector3f::from),
                );
                self.right_rotation.store(
                    transformation
                        .get("right_rotation")
                        .map_or(Quaternion::IDENTITY, Quaternion::from),
                );
            }

            self.transformation_interpolation_duration.store(
                nbt.get_int("interpolation_duration").unwrap_or(0),
                Ordering::Relaxed,
            );
            self.transformation_interpolation_start_delta_ticks.store(
                nbt.get_int("start_interpolation").unwrap_or(0),
                Ordering::Relaxed,
            );
            self.pos_rot_interpolation_duration.store(
                nbt.get_int("teleport_duration").unwrap_or(0).clamp(0, 59),
                Ordering::Relaxed,
            );
            self.billboard.store(
                nbt.get_string("billboard")
                    .map_or(Billboard::Fixed, Billboard::from_name)
                    .id(),
                Ordering::Relaxed,
            );
            self.view_range.store(
                nbt.get_float("view_range").unwrap_or(1.0),
                Ordering::Relaxed,
            );
            self.shadow_radius.store(
                nbt.get_float("shadow_radius").unwrap_or(0.0),
                Ordering::Relaxed,
            );
            self.shadow_strength.store(
                nbt.get_float("shadow_strength").unwrap_or(1.0),
                Ordering::Relaxed,
            );
            self.width
                .store(nbt.get_float("width").unwrap_or(0.0), Ordering::Relaxed);
            self.height
                .store(nbt.get_float("height").unwrap_or(0.0), Ordering::Relaxed);
            self.glow_color_override.store(
                nbt.get_int("glow_color_override").unwrap_or(-1),
                Ordering::Relaxed,
            );

            let brightness_override = nbt.get_compound("brightness").map_or(-1, |brightness| {
                let block = brightness.get_int("block").unwrap_or(0) & 15;
                let sky = brightness.get_int("sky").unwrap_or(0) & 15;
                block << 4 | sky << 20
            });
            self.brightness_override
                .store(brightness_override, Ordering::Relaxed);
        })
    }
}

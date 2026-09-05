/* This file is generated. Do not edit manually. */
use pumpkin_util::math::float_provider::{
    ClampedNormalFloatProvider, ConstantFloatProvider, FloatProvider, NormalFloatProvider,
    TrapezoidFloatProvider, UniformFloatProvider,
};
use pumpkin_util::math::int_provider::{
    BiasedToBottomIntProvider, ConstantIntProvider, IntProvider, NormalIntProvider,
    UniformIntProvider, VeryBiasedToBottomIntProvider,
};
use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
pub enum HeightProvider {
    Uniform(UniformHeightProvider),
    Trapezoid(TrapezoidHeightProvider),
    VeryBiasedToBottom(VeryBiasedToBottomHeightProvider),
}
pub struct UniformHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
}
pub struct TrapezoidHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub plateau: Option<i32>,
}
pub struct VeryBiasedToBottomHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub inner: Option<std::num::NonZero<u32>>,
}
pub struct CaveCarverConfig {
    pub count: IntProvider,
    pub horizontal_radius_multiplier: FloatProvider,
    pub vertical_radius_multiplier: FloatProvider,
    pub floor_level: FloatProvider,
    pub room_vertical_radius_multiplier: FloatProvider,
    pub start_vertical_radius_multiplier: FloatProvider,
    pub thickness: FloatProvider,
    pub weird_thickness_bias: bool,
}
impl CaveCarverConfig {
    #[must_use]
    pub const fn default() -> Self {
        Self {
            count: IntProvider::Constant(1),
            horizontal_radius_multiplier: FloatProvider::Constant(1.0),
            vertical_radius_multiplier: FloatProvider::Constant(1.0),
            floor_level: FloatProvider::Constant(-0.7),
            room_vertical_radius_multiplier: FloatProvider::Constant(0.5),
            start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
            thickness: FloatProvider::Constant(1.0),
            weird_thickness_bias: false,
        }
    }
}
pub struct CanyonShapeConfig {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
    pub y_scale: f32,
}
pub struct CanyonCarverConfig {
    pub vertical_rotation: FloatProvider,
    pub shape: CanyonShapeConfig,
}
pub enum CarverAdditionalConfig {
    Cave(CaveCarverConfig),
    Canyon(CanyonCarverConfig),
}
pub struct CarverConfig {
    pub probability: f32,
    pub y: HeightProvider,
    pub additional: CarverAdditionalConfig,
}
use super::*;
pub const CANYON: CarverConfig = CarverConfig {
    probability: 0.01f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::Absolute(Absolute { absolute: 10i16 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 67i16 }),
    }),
    additional: CarverAdditionalConfig::Canyon(CanyonCarverConfig {
        vertical_rotation: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(-0.125f32, 0.125f32),
        )),
        shape: CanyonShapeConfig {
            distance_factor: FloatProvider::Object(NormalFloatProvider::Uniform(
                UniformFloatProvider::new(0.75f32, 1f32),
            )),
            thickness: FloatProvider::Object(NormalFloatProvider::Trapezoid(
                TrapezoidFloatProvider::new(0f32, 6f32, 2f32),
            )),
            width_smoothness: 3i32,
            horizontal_radius_factor: FloatProvider::Object(NormalFloatProvider::Uniform(
                UniformFloatProvider::new(0.75f32, 1f32),
            )),
            vertical_radius_default_factor: 1f32,
            vertical_radius_center_factor: 0f32,
            y_scale: 3f32,
        },
    }),
};
pub const CAVE: CarverConfig = CarverConfig {
    probability: 0.15f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 8i8 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 180i16 }),
    }),
    additional: CarverAdditionalConfig::Cave(CaveCarverConfig {
        count: IntProvider::Object(NormalIntProvider::VeryBiasedToBottom(
            VeryBiasedToBottomIntProvider::new(0i32, 14i32, 1i32),
        )),
        horizontal_radius_multiplier: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(0.7f32, 1.4f32),
        )),
        vertical_radius_multiplier: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(0.8f32, 1.3f32),
        )),
        floor_level: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(-1f32, -0.4f32),
        )),
        room_vertical_radius_multiplier: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(0.1f32, 0.9f32),
        )),
        start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
        thickness: FloatProvider::Object(NormalFloatProvider::Trapezoid(
            TrapezoidFloatProvider::new(0f32, 3f32, 1f32),
        )),
        weird_thickness_bias: true,
    }),
};
pub const CAVE_EXTRA_UNDERGROUND: CarverConfig = CarverConfig {
    probability: 0.07f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 8i8 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 47i16 }),
    }),
    additional: CarverAdditionalConfig::Cave(CaveCarverConfig {
        count: IntProvider::Object(NormalIntProvider::VeryBiasedToBottom(
            VeryBiasedToBottomIntProvider::new(0i32, 14i32, 1i32),
        )),
        horizontal_radius_multiplier: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(0.7f32, 1.4f32),
        )),
        vertical_radius_multiplier: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(0.8f32, 1.3f32),
        )),
        floor_level: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(-1f32, -0.4f32),
        )),
        room_vertical_radius_multiplier: FloatProvider::Object(NormalFloatProvider::Uniform(
            UniformFloatProvider::new(0.1f32, 0.9f32),
        )),
        start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
        thickness: FloatProvider::Object(NormalFloatProvider::Trapezoid(
            TrapezoidFloatProvider::new(0f32, 3f32, 1f32),
        )),
        weird_thickness_bias: true,
    }),
};
pub const NETHER_CAVE: CarverConfig = CarverConfig {
    probability: 0.2f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 1i8 }),
    }),
    additional: CarverAdditionalConfig::Cave(CaveCarverConfig {
        count: IntProvider::Object(NormalIntProvider::VeryBiasedToBottom(
            VeryBiasedToBottomIntProvider::new(0i32, 9i32, 1i32),
        )),
        horizontal_radius_multiplier: FloatProvider::Constant(1f32),
        vertical_radius_multiplier: FloatProvider::Constant(1f32),
        floor_level: FloatProvider::Constant(-0.7f32),
        room_vertical_radius_multiplier: FloatProvider::Constant(0.5f32),
        start_vertical_radius_multiplier: FloatProvider::Constant(5f32),
        thickness: FloatProvider::Object(NormalFloatProvider::Trapezoid(
            TrapezoidFloatProvider::new(0f32, 6f32, 2f32),
        )),
        weird_thickness_bias: false,
    }),
};

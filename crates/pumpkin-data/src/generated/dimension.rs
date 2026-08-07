/* This file is generated. Do not edit manually. */
use pumpkin_registry::{Registry, error::RegistryInsertError};
use pumpkin_util::{
    identifier::Identifier,
    math::int_provider::{
        BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider,
        ConstantIntProvider, IntProvider, NormalIntProvider, TrapezoidIntProvider,
        UniformIntProvider, WeightedEntry, WeightedListIntProvider,
    },
};
use std::sync::{Arc, LazyLock};
pub static DIMENSIONS: LazyLock<Arc<Registry<Dimension>>> = LazyLock::new(|| {
    let registry = Arc::new(Registry::new());
    initialize(&registry).unwrap();
    pumpkin_registry::ROOT
        .register_arc(
            Identifier::vanilla_static("dimension_type"),
            registry.clone(),
        )
        .unwrap();
    registry
});
#[derive(Debug, Clone)]
pub struct Dimension {
    pub id: u8,
    pub fixed_time: Option<i64>,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: &'static str,
    pub ambient_light: f32,
    pub monster_spawn_light_level: IntProvider,
    pub monster_spawn_block_light_limit: u8,
    pub sky_color: Option<i32>,
    pub fog_color: Option<i32>,
    pub cloud_color: Option<i32>,
    pub timelines: Option<&'static str>,
}
fn initialize(registry: &Arc<Registry<Dimension>>) -> Result<(), RegistryInsertError> {
    registry.register(
        Identifier::parse_static("minecraft:overworld"),
        Dimension {
            id: 0u8,
            fixed_time: None,
            has_skylight: true,
            has_ceiling: false,
            coordinate_scale: 1f64,
            min_y: -64i32,
            height: 384i32,
            logical_height: 384i32,
            infiniburn: "#minecraft:infiniburn_overworld",
            ambient_light: 0f32,
            monster_spawn_light_level: IntProvider::Object(NormalIntProvider::Uniform(
                UniformIntProvider {
                    min_inclusive: 0i32,
                    max_inclusive: 7i32,
                },
            )),
            monster_spawn_block_light_limit: 0u8,
            sky_color: Some(7907327i32),
            fog_color: Some(12638463i32),
            cloud_color: None,
            timelines: Some("#minecraft:in_overworld"),
        },
    )?;
    registry.register(
        Identifier::parse_static("minecraft:overworld_caves"),
        Dimension {
            id: 1u8,
            fixed_time: None,
            has_skylight: true,
            has_ceiling: true,
            coordinate_scale: 1f64,
            min_y: -64i32,
            height: 384i32,
            logical_height: 384i32,
            infiniburn: "#minecraft:infiniburn_overworld",
            ambient_light: 0f32,
            monster_spawn_light_level: IntProvider::Object(NormalIntProvider::Uniform(
                UniformIntProvider {
                    min_inclusive: 0i32,
                    max_inclusive: 7i32,
                },
            )),
            monster_spawn_block_light_limit: 0u8,
            sky_color: Some(7907327i32),
            fog_color: Some(12638463i32),
            cloud_color: None,
            timelines: Some("#minecraft:in_overworld"),
        },
    )?;
    registry.register(
        Identifier::parse_static("minecraft:the_end"),
        Dimension {
            id: 2u8,
            fixed_time: None,
            has_skylight: true,
            has_ceiling: false,
            coordinate_scale: 1f64,
            min_y: 0i32,
            height: 256i32,
            logical_height: 256i32,
            infiniburn: "#minecraft:infiniburn_end",
            ambient_light: 0.25f32,
            monster_spawn_light_level: IntProvider::Constant(15i32),
            monster_spawn_block_light_limit: 0u8,
            sky_color: Some(0i32),
            fog_color: Some(1577752i32),
            cloud_color: None,
            timelines: Some("#minecraft:in_end"),
        },
    )?;
    registry.register(
        Identifier::parse_static("minecraft:the_nether"),
        Dimension {
            id: 3u8,
            fixed_time: None,
            has_skylight: false,
            has_ceiling: true,
            coordinate_scale: 8f64,
            min_y: 0i32,
            height: 256i32,
            logical_height: 128i32,
            infiniburn: "#minecraft:infiniburn_nether",
            ambient_light: 0.1f32,
            monster_spawn_light_level: IntProvider::Constant(7i32),
            monster_spawn_block_light_limit: 15u8,
            sky_color: None,
            fog_color: None,
            cloud_color: None,
            timelines: Some("#minecraft:in_nether"),
        },
    )?;
    Ok(())
}
impl PartialEq for Dimension {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Dimension {}

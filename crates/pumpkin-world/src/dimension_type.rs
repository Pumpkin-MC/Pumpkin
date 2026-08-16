use crate::{
    attributes::EnvironmentAttributeMap,
    cardinal_lighting::CardinalLighting,
    time::{Timeline, WorldClock},
};
use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::optional_field::{OptionalFieldDecode, OptionalFieldEncode},
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_data::{Block, int_provider::IntProviderValue};
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_registry::{RegistryResolvable, RegistryResolvableSet};

#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedIntProvider(NbtTag);

impl UnresolvedIntProvider {
    pub fn resolve(&self) -> DataResult<IntProviderValue> {
        IntProviderValue::parse(self.0.clone(), &NbtOps)
    }
}

impl Encode for UnresolvedIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        DataResult::new_success(NbtOps.convert_to(ops, self.0.clone()))
            .flat_map(|value| ops.merge_into_primitive(prefix, value))
    }
}

impl Decode for UnresolvedIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        DataResult::new_success((Self(ops.convert_to(&NbtOps, input)), ops.empty()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyBox {
    None,
    Overworld,
    End,
}

impl Encode for SkyBox {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        match self {
            Self::None => "none",
            Self::Overworld => "overworld",
            Self::End => "end",
        }
        .to_string()
        .encode(ops, prefix)
    }
}

impl Decode for SkyBox {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        String::decode(input, ops).flat_map(|(value, remaining)| {
            let skybox = match value.as_str() {
                "none" => Self::None,
                "overworld" => Self::Overworld,
                "end" => Self::End,
                _ => return DataResult::new_error(format!("unknown skybox type: {value}")),
            };
            DataResult::new_success((skybox, remaining))
        })
    }
}

#[derive(Debug, Clone)]
pub struct DimensionType {
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: RegistryResolvableSet<Block>,
    pub ambient_light: f32,
    pub monster_spawn_light_level: UnresolvedIntProvider,
    pub monster_spawn_block_light_limit: u8,
    pub attributes: EnvironmentAttributeMap,

    /// Whether the dimension uses a fixed day/night time.
    pub has_fixed_time: bool,

    pub has_ender_dragon_fight: bool,
    pub skybox: Option<SkyBox>,
    pub cardinal_light: Option<CardinalLighting>,

    /// Timeline IDs/tags used by the dimension.
    ///
    /// Vanilla currently uses a single tag, but the registry-entry selector is
    /// intentionally flexible for datapacks.
    pub timelines: Option<RegistryResolvableSet<Timeline>>,

    /// Clock used by `/time` and dimension-specific time markers.
    pub default_clock: Option<RegistryResolvable<WorldClock>>,
}

impl DimensionType {
    #[must_use]
    pub fn is_nether_like(&self) -> bool {
        self.cardinal_light == Some(crate::cardinal_lighting::NETHER)
    }

    #[must_use]
    pub fn is_end_like(&self) -> bool {
        self.skybox == Some(SkyBox::End)
    }

    #[must_use]
    pub fn is_overworld_like(&self) -> bool {
        !self.is_nether_like() && !self.is_end_like()
    }
}

impl Encode for DimensionType {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut builder = ops.map_builder();
        builder = self.has_skylight.encode_field("has_skylight", ops, builder);
        builder = self.has_ceiling.encode_field("has_ceiling", ops, builder);
        builder = self
            .has_ender_dragon_fight
            .encode_field("has_ender_dragon_fight", ops, builder);
        builder = self
            .coordinate_scale
            .encode_field("coordinate_scale", ops, builder);
        builder = self.min_y.encode_field("min_y", ops, builder);
        builder = self.height.encode_field("height", ops, builder);
        builder = self
            .logical_height
            .encode_field("logical_height", ops, builder);
        builder = self.infiniburn.encode_field("infiniburn", ops, builder);
        builder = self
            .ambient_light
            .encode_field("ambient_light", ops, builder);
        builder =
            self.monster_spawn_light_level
                .encode_field("monster_spawn_light_level", ops, builder);
        builder = self.monster_spawn_block_light_limit.encode_field(
            "monster_spawn_block_light_limit",
            ops,
            builder,
        );
        builder = self.attributes.encode_field("attributes", ops, builder);

        if self.has_fixed_time {
            builder = self
                .has_fixed_time
                .encode_field("has_fixed_time", ops, builder);
        }

        builder = self.skybox.encode_optional_field("skybox", ops, builder);
        builder = self
            .cardinal_light
            .encode_optional_field("cardinal_light", ops, builder);
        builder = self
            .timelines
            .encode_optional_field("timelines", ops, builder);
        builder = self
            .default_clock
            .encode_optional_field("default_clock", ops, builder);

        builder.build(prefix)
    }
}

impl Decode for DimensionType {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            let optional_fields =
                Option::<bool>::decode_optional_field::<O>("has_fixed_time", &map, ops, false)
                    .apply_5(
                        |has_fixed_time, skybox, cardinal_light, timelines, default_clock| {
                            (
                                has_fixed_time.unwrap_or(false),
                                skybox,
                                cardinal_light,
                                timelines,
                                default_clock,
                            )
                        },
                        Option::<SkyBox>::decode_optional_field::<O>("skybox", &map, ops, false),
                        Option::<CardinalLighting>::decode_optional_field::<O>(
                            "cardinal_light",
                            &map,
                            ops,
                            false,
                        ),
                        Option::<RegistryResolvableSet<Timeline>>::decode_optional_field::<O>(
                            "timelines",
                            &map,
                            ops,
                            false,
                        ),
                        Option::<RegistryResolvable<WorldClock>>::decode_optional_field::<O>(
                            "default_clock",
                            &map,
                            ops,
                            false,
                        ),
                    );

            bool::decode_field::<O>("has_skylight", &map, ops).apply_13(
                |has_skylight,
                 has_ceiling,
                 coordinate_scale,
                 min_y,
                 height,
                 logical_height,
                 infiniburn,
                 ambient_light,
                 monster_spawn_light_level,
                 monster_spawn_block_light_limit,
                 attributes,
                 has_ender_dragon_fight,
                 (has_fixed_time, skybox, cardinal_light, timelines, default_clock)| {
                    (
                        Self {
                            has_skylight,
                            has_ceiling,
                            coordinate_scale,
                            min_y,
                            height,
                            logical_height,
                            infiniburn,
                            ambient_light,
                            monster_spawn_light_level,
                            monster_spawn_block_light_limit,
                            attributes,
                            has_fixed_time,
                            has_ender_dragon_fight,
                            skybox,
                            cardinal_light,
                            timelines,
                            default_clock,
                        },
                        ops.empty(),
                    )
                },
                bool::decode_field::<O>("has_ceiling", &map, ops),
                f64::decode_field::<O>("coordinate_scale", &map, ops),
                i32::decode_field::<O>("min_y", &map, ops),
                i32::decode_field::<O>("height", &map, ops),
                i32::decode_field::<O>("logical_height", &map, ops),
                RegistryResolvableSet::<Block>::decode_field::<O>("infiniburn", &map, ops),
                f32::decode_field::<O>("ambient_light", &map, ops),
                UnresolvedIntProvider::decode_field::<O>("monster_spawn_light_level", &map, ops),
                u8::decode_field::<O>("monster_spawn_block_light_limit", &map, ops),
                EnvironmentAttributeMap::decode_field::<O>("attributes", &map, ops),
                bool::decode_field::<O>("has_ender_dragon_fight", &map, ops),
                optional_fields,
            )
        })
    }
}

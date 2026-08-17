use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode, MapLike,
    codec::optional_field::{OptionalFieldDecode, OptionalFieldEncode},
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_registry::{Registry, RegistryResolvable};
use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

use crate::attributes::EnvironmentAttributeEntry;

use super::{AttributeTrack, TimeMarker, WorldClock};

#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub clock: RegistryResolvable<WorldClock>,
    pub period_ticks: Option<u32>,
    pub tracks: FxHashMap<Identifier, AttributeTrack>,
    pub time_markers: FxHashMap<Identifier, TimeMarker>,
}

impl Timeline {
    /// Validates period and keyframe/time-marker ordering and bounds without
    /// requiring the registries to have been bootstrapped yet.
    pub fn validate_structure(&self) -> DataResult<()> {
        let mut result = DataResult::new_success(());

        if self.period_ticks == Some(0) {
            result = result.add_message(&DataResult::<()>::new_error(
                "timeline period_ticks must be greater than zero",
            ));
        }

        for (id, marker) in &self.time_markers {
            if let Some(period) = self.period_ticks
                && marker.ticks > period
            {
                result = result.add_message(&DataResult::<()>::new_error(format!(
                    "timeline time marker {id} tick {} exceeds period_ticks {period}",
                    marker.ticks
                )));
            }
        }

        for track in self.tracks.values() {
            result = result.add_message(&track.validate_structure(self.period_ticks));
        }

        result
    }

    /// Validates registry references, modifiers and erased keyframe values
    /// against their environment attribute types.
    pub fn validate(
        &self,
        world_clocks: &dyn Registry,
        environment_attributes: &dyn Registry,
    ) -> DataResult<()> {
        let mut result = self.validate_structure();

        if let Err(error) = self.clock.resolve(world_clocks) {
            result = result.add_message(&DataResult::<()>::new_error(error.to_string()));
        }

        for (attribute_id, track) in &self.tracks {
            let attribute =
                RegistryResolvable::<EnvironmentAttributeEntry>::new(attribute_id.clone());
            match attribute.resolve(environment_attributes) {
                Ok(attribute) => {
                    result = result.add_message(&track.validate(self.period_ticks, &attribute));
                }
                Err(error) => {
                    result = result.add_message(&DataResult::<()>::new_error(error.to_string()));
                }
            }
        }

        result
    }
}

impl Encode for Timeline {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut builder = self.clock.encode_field("clock", ops, ops.map_builder());
        builder = self
            .period_ticks
            .encode_optional_field("period_ticks", ops, builder);

        builder = builder.add_key_result_value_result(
            DataResult::new_success(ops.create_string("tracks")),
            encode_identifier_map(&self.tracks, ops),
        );

        if !self.time_markers.is_empty() {
            builder = builder.add_key_result_value_result(
                DataResult::new_success(ops.create_string("time_markers")),
                encode_identifier_map(&self.time_markers, ops),
            );
        }

        builder.build(prefix)
    }
}

impl Decode for Timeline {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            RegistryResolvable::<WorldClock>::decode_field::<O>("clock", &map, ops).flat_map(
                |clock| {
                    Option::<u32>::decode_optional_field::<O>("period_ticks", &map, ops, false)
                        .flat_map(|period_ticks| {
                            decode_optional_identifier_map_field::<O, AttributeTrack>(
                                "tracks", &map, ops,
                            )
                            .flat_map(|tracks| {
                                decode_optional_identifier_map_field::<O, TimeMarker>(
                                    "time_markers",
                                    &map,
                                    ops,
                                )
                                .map(|time_markers| {
                                    (
                                        Self {
                                            clock,
                                            period_ticks,
                                            tracks,
                                            time_markers,
                                        },
                                        ops.empty(),
                                    )
                                })
                            })
                        })
                },
            )
        })
    }
}

fn encode_identifier_map<O: DynamicOps, V: Encode>(
    values: &FxHashMap<Identifier, V>,
    ops: &'static O,
) -> DataResult<O::Value> {
    let mut builder = ops.map_builder();
    for (identifier, value) in values {
        builder = builder
            .add_key_result_value_result(identifier.encode_start(ops), value.encode_start(ops));
    }
    builder.build(ops.empty())
}

fn decode_optional_identifier_map_field<O: DynamicOps, V: Decode>(
    name: &str,
    map: &impl MapLike<Value = O::Value>,
    ops: &'static O,
) -> DataResult<FxHashMap<Identifier, V>> {
    map.get(&ops.create_string(name)).map_or_else(
        || DataResult::new_success(FxHashMap::default()),
        |value| decode_identifier_map(value.clone(), ops),
    )
}

fn decode_identifier_map<O: DynamicOps, V: Decode>(
    input: O::Value,
    ops: &'static O,
) -> DataResult<FxHashMap<Identifier, V>> {
    ops.get_map(&input).flat_map(|map| {
        let mut values = FxHashMap::default();
        let mut result = DataResult::new_success(());
        for (key, value) in map.iter() {
            let key = <Identifier as Decode>::parse(key, ops);
            let value = V::parse(value.clone(), ops);
            result = result.add_message(&key).add_message(&value);
            if let (Some(key), Some(value)) =
                (key.into_result_or_partial(), value.into_result_or_partial())
            {
                values.insert(key, value);
            }
        }
        result.with_complete_or_partial(values)
    })
}

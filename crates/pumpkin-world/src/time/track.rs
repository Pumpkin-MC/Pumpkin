use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode, MapLike,
    codec::optional_field::OptionalFieldDecode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};

use crate::attributes::{EnvironmentAttributeEntry, attribute_modifier::AttributeModifier};

use super::Easing;

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeTrack {
    pub ease: Easing,
    pub modifier: AttributeModifier,
    pub keyframes: Vec<KeyFrame>,
}

impl AttributeTrack {
    pub(super) fn validate_structure(&self, period_ticks: Option<u32>) -> DataResult<()> {
        let mut result = DataResult::new_success(());
        let mut previous = None;
        let mut repeated_at_tick = false;

        if self.keyframes.is_empty() {
            result = result.add_message(&DataResult::<()>::new_error(
                "timeline attribute track must contain at least one keyframe",
            ));
        }

        for frame in &self.keyframes {
            if let Some(period) = period_ticks
                && frame.ticks > period
            {
                result = result.add_message(&DataResult::<()>::new_error(format!(
                    "timeline keyframe tick {} exceeds period_ticks {period}",
                    frame.ticks
                )));
            }

            if let Some(previous_tick) = previous {
                if frame.ticks < previous_tick {
                    result = result.add_message(&DataResult::<()>::new_error(
                        "timeline keyframes must be ordered by ticks",
                    ));
                }
                if frame.ticks == previous_tick {
                    if repeated_at_tick {
                        result = result.add_message(&DataResult::<()>::new_error(format!(
                            "timeline track contains more than two keyframes at tick {}",
                            frame.ticks
                        )));
                    }
                    repeated_at_tick = true;
                } else {
                    repeated_at_tick = false;
                }
            }
            previous = Some(frame.ticks);
        }

        result
    }

    pub(super) fn validate(
        &self,
        period_ticks: Option<u32>,
        attribute: &EnvironmentAttributeEntry,
    ) -> DataResult<()> {
        let mut result = self.validate_structure(period_ticks);
        for frame in &self.keyframes {
            result = result.add_message(
                &self
                    .modifier
                    .validate_argument(attribute, frame.value.clone()),
            );
        }
        result
    }

    /// Samples the modifier argument for this track at a clock tick.
    ///
    /// Repeating tracks interpolate across the end/start boundary. Non-repeating
    /// tracks clamp to their first/last keyframe outside the keyframe range.
    pub fn sample(
        &self,
        clock_tick: u64,
        period_ticks: Option<u32>,
        attribute: &EnvironmentAttributeEntry,
    ) -> DataResult<Option<NbtTag>> {
        let Some(first) = self.keyframes.first() else {
            return DataResult::new_success(None);
        };
        if self.keyframes.len() == 1 {
            return DataResult::new_success(Some(first.value.clone()));
        }

        let current = period_ticks.map_or(clock_tick, |period| {
            if period == 0 {
                clock_tick
            } else {
                clock_tick % u64::from(period)
            }
        });

        if period_ticks.is_none() {
            if current <= u64::from(first.ticks) {
                return DataResult::new_success(Some(first.value.clone()));
            }
            if let Some(last) = self.keyframes.last()
                && current >= u64::from(last.ticks)
            {
                return DataResult::new_success(Some(last.value.clone()));
            }
        }

        let next_index = self
            .keyframes
            .partition_point(|frame| u64::from(frame.ticks) <= current);

        let (from, from_tick, to, to_tick, sample_tick) = if next_index == 0 {
            let Some(period) = period_ticks.map(u64::from) else {
                return DataResult::new_error(
                    "non-repeating timeline sampled before its first keyframe",
                );
            };
            let Some(from) = self.keyframes.last() else {
                return DataResult::new_error("timeline track has no keyframes");
            };
            (
                from,
                u64::from(from.ticks),
                first,
                u64::from(first.ticks) + period,
                current + period,
            )
        } else if next_index == self.keyframes.len() {
            let from = &self.keyframes[next_index - 1];
            let period = u64::from(period_ticks.unwrap_or(0));
            (
                from,
                u64::from(from.ticks),
                first,
                u64::from(first.ticks) + period,
                current,
            )
        } else {
            let from = &self.keyframes[next_index - 1];
            let to = &self.keyframes[next_index];
            (
                from,
                u64::from(from.ticks),
                to,
                u64::from(to.ticks),
                current,
            )
        };

        if to_tick <= from_tick {
            return DataResult::new_success(Some(to.value.clone()));
        }

        let t = ((sample_tick.saturating_sub(from_tick)) as f32 / (to_tick - from_tick) as f32)
            .clamp(0.0, 1.0);
        let eased = self.ease.apply(t);
        self.modifier
            .interpolate_argument(attribute, eased, from.value.clone(), to.value.clone())
            .map(Some)
    }
}

impl Encode for AttributeTrack {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut builder = ops.map_builder();
        if self.ease != Easing::Linear {
            builder = self.ease.encode_field("ease", ops, builder);
        }
        if !self.modifier.is_override() {
            builder = self.modifier.encode_field("modifier", ops, builder);
        }
        self.keyframes
            .encode_field("keyframes", ops, builder)
            .build(prefix)
    }
}

impl Decode for AttributeTrack {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            Vec::<KeyFrame>::decode_field::<O>("keyframes", &map, ops).flat_map(|keyframes| {
                Option::<Easing>::decode_optional_field::<O>("ease", &map, ops, false).flat_map(
                    |ease| {
                        Option::<AttributeModifier>::decode_optional_field::<O>(
                            "modifier", &map, ops, false,
                        )
                        .map(|modifier| {
                            (
                                Self {
                                    ease: ease.unwrap_or_default(),
                                    modifier: modifier.unwrap_or_default(),
                                    keyframes,
                                },
                                ops.empty(),
                            )
                        })
                    },
                )
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyFrame {
    pub ticks: u32,
    pub value: NbtTag,
}

impl Encode for KeyFrame {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.ticks
            .encode_field("ticks", ops, ops.map_builder())
            .add_key_result_value_result(
                DataResult::new_success(ops.create_string("value")),
                DataResult::new_success(NbtOps.convert_to(ops, self.value.clone())),
            )
            .build(prefix)
    }
}

impl Decode for KeyFrame {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            u32::decode_field::<O>("ticks", &map, ops).flat_map(|ticks| {
                map.get(&ops.create_string("value")).map_or_else(
                    || DataResult::new_error("timeline keyframe is missing value"),
                    |value| {
                        DataResult::new_success((
                            Self {
                                ticks,
                                value: ops.convert_to(&NbtOps, value.clone()),
                            },
                            ops.empty(),
                        ))
                    },
                )
            })
        })
    }
}

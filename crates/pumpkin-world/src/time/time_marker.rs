use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::optional_field::OptionalFieldDecode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeMarker {
    pub ticks: u32,
    pub show_in_commands: bool,
}

impl Encode for TimeMarker {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        if !self.show_in_commands {
            return self.ticks.encode(ops, prefix);
        }

        self.ticks
            .encode_field("ticks", ops, ops.map_builder())
            .pipe(|builder| {
                self.show_in_commands
                    .encode_field("show_in_commands", ops, builder)
            })
            .build(prefix)
    }
}

impl Decode for TimeMarker {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        let compact = u32::parse(input.clone(), ops);
        if let Some(ticks) = compact.into_result_or_partial() {
            return DataResult::new_success((
                Self {
                    ticks,
                    show_in_commands: false,
                },
                ops.empty(),
            ));
        }

        ops.get_map(&input).flat_map(|map| {
            u32::decode_field::<O>("ticks", &map, ops).flat_map(|ticks| {
                Option::<bool>::decode_optional_field::<O>("show_in_commands", &map, ops, false)
                    .map(|show_in_commands| {
                        (
                            Self {
                                ticks,
                                show_in_commands: show_in_commands.unwrap_or(false),
                            },
                            ops.empty(),
                        )
                    })
            })
        })
    }
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

use crate::codec::optional_field::OptionalFieldDecode;
use crate::codec::FieldDecode;
use crate::struct_builder::StructBuilder;
use crate::{DataResult, Decode, DynamicOps, Lifecycle};
use pumpkin_codecs_macros::Encode;

#[derive(Debug, Encode)]
struct Test {
    #[field(name = "a")]
    a: i32,
    #[field(name = "b")]
    b: Vec<String>,
    #[field(name = "c")]
    c: Option<bool>,
}

impl Decode for Test {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input)
            .with_lifecycle(Lifecycle::Stable)
            .flat_map(|map| {
                let a = i32::decode_field::<O>("a", &map, ops);
                let b = Vec::<String>::decode_field::<O>("b", &map, ops);
                let c = Option::<bool>::decode_optional_field::<O>("c", &map, ops, false);
                a.apply_3(|a, b, c| Self { a, b, c }, b, c)
                    .map(|r| (r, input.clone()))
            })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::structure::Test;
    use crate::json_ops::JsonOps;
    use crate::{Decode, Encode};
    use serde_json::json;

    #[test]
    fn lol() {
        let test = Test {
            a: 69,
            b: vec![String::from("hello"), String::from("world")],
            c: Some(true),
        };
        println!("{:?}", test.encode_start(&JsonOps));

        let json = json!({
            "a": 2,
            "b": ["a", "b", "c"],
            "c": true
        });
        println!("{:?}", Test::parse(json, &JsonOps));
    }
}

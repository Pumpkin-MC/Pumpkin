use crate::struct_builder::StructBuilder;
use crate::{DataResult, Decode, DynamicOps, Encode, Lifecycle};
use pumpkin_codecs_macros::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct Test {
    a: i32,
    b: Vec<String>,
    #[field(default = 4)]
    c: u8,
}

#[derive(Debug)]
enum E {
    A(i32),
    B(String, bool)
}

impl Encode for E {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut builder = ops.map_builder();
        match self {
            Self::A(a) => {
                builder = builder.add_string_key_value("type", ops.create_string("a"));
                builder = crate::codec::FieldEncode::encode_field(a, "a", ops, builder);
            }
            Self::B(b, c) => {
                builder = builder.add_string_key_value("type", ops.create_string("b"));
                builder = crate::codec::FieldEncode::encode_field(b, "b", ops, builder);
                builder = crate::codec::FieldEncode::encode_field(c, "c", ops, builder);
            }
        }
        builder.build(prefix)
    }
}

impl Decode for E {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).with_lifecycle(Lifecycle::Stable).flat_map(|map| {
            let ty: DataResult<String> = crate::codec::FieldDecode::decode_field::<O>("type", &map, ops);
            ty.flat_map(|ty| {
                match ty.as_str() {
                    "a" => {
                        let a0 = crate::codec::FieldDecode::decode_field::<O>("a", &map, ops);
                        a0.map(Self::A).map(|r| (r, input.clone()))
                    },
                    "b" => {
                        let a0 = crate::codec::FieldDecode::decode_field::<O>("b", &map, ops);
                        let a1 = crate::codec::FieldDecode::decode_field::<O>("c", &map, ops);
                        a0.apply_2(Self::B, a1).map(|r| (r, input.clone()))
                    },
                    _ => DataResult::new_error(format!("Invalid differentiator key {ty}"))
                }
            })
        })
    }
}

#[derive(Debug, Decode)]
#[tag_key("type")]
enum Foo {
    #[tag("a")]
    A {
        a: i32
    },
    #[tag("b")]
    B {
        b: String,
        c: bool
    },
    #[tag("c")]
    C
}

#[derive(Debug, Encode, Decode)]
struct T;

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
            c: 4,
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

use crate::struct_builder::StructBuilder;
use crate::{DataResult, DynamicOps, Lifecycle};
use pumpkin_codecs_macros::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct Test {
    a: i32,
    b: Vec<String>,
    #[field(default = 4)]
    c: u8
}

#[derive(Debug, Encode, Decode)]
struct T(
    #[field(name = "fg")] i32,
    #[field(name = "hi")] String
);

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

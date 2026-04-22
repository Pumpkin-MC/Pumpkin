use pumpkin_codecs::json_ops::JsonOps;
use pumpkin_codecs::{assert_decode, assert_decode_success, assert_encode_success};
use pumpkin_codecs_macros::{Decode, Encode};
use serde_json::json;

#[test]
fn string_represented() {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
    pub enum Weekday {
        #[codec(tag = "sunday")]
        Sunday,
        #[codec(tag = "monday")]
        Monday,
        #[codec(tag = "tuesday")]
        Tuesday,
        #[codec(tag = "wednesday")]
        Wednesday,
        #[codec(tag = "thursday")]
        Thursday,
        #[codec(tag = "friday")]
        Friday,
        #[codec(tag = "saturday")]
        Saturday,
    }

    assert_encode_success!(Weekday::Monday, JsonOps, json!("monday"));
    assert_encode_success!(Weekday::Thursday, JsonOps, json!("thursday"));
    assert_encode_success!(Weekday::Saturday, JsonOps, json!("saturday"));

    assert_decode_success!(Weekday, json!("wednesday"), JsonOps, Weekday::Wednesday);
    assert_decode_success!(Weekday, json!("sunday"), JsonOps, Weekday::Sunday);

    assert_decode!(Weekday, json!("Sunday"), JsonOps, is_error);
    assert_decode!(Weekday, json!("February"), JsonOps, is_error);
}

#[test]
fn simple() {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
    pub enum Shape {
        #[codec(tag = "rectangle")]
        Rectangle { width: u64, height: u64 },
        #[codec(tag = "circle")]
        Circle { radius: u32 },
        #[codec(tag = "triangle")]
        Triangle { base: u32, height: u32 },
    }

    assert_encode_success!(
        Shape::Rectangle {
            width: 30,
            height: 8
        },
        JsonOps,
        json!({
            "type": "rectangle",
            "width": 30,
            "height": 8
        })
    );

    assert_encode_success!(
        Shape::Circle { radius: 45 },
        JsonOps,
        json!({
            "type": "circle",
            "radius": 45
        })
    );

    assert_encode_success!(
        Shape::Triangle {
            base: 34,
            height: 6534
        },
        JsonOps,
        json!({
            "type": "triangle",
            "base": 34,
            "height": 6534
        })
    );

    assert_decode_success!(
        Shape,
        json!({
            "type": "circle",
            "radius": 190
        }),
        JsonOps,
        Shape::Circle { radius: 190 }
    );

    assert_decode!(
        Shape,
        json!({
            "radius": 190
        }),
        JsonOps,
        is_error
    );

    assert_decode!(
        Shape,
        json!({
            "type": "pentagon",
            "side": 25
        }),
        JsonOps,
        is_error
    );
}

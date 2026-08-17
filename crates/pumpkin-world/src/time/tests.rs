use pumpkin_codecs::{DataResult, Decode, json_ops::JsonOps};
use pumpkin_util::identifier::Identifier;
use serde_json::{Value, json};

use crate::{
    attributes::attribute_modifier::AttributeOperation,
    time::{Easing, Timeline},
};

fn parse_timeline(value: Value) -> Result<Timeline, String> {
    match Timeline::parse(value, &JsonOps) {
        DataResult::Success { result, .. } => Ok(result),
        DataResult::Error { message, .. } => Err(message),
    }
}

#[test]
fn parses_full_timeline() {
    let timeline = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "period_ticks": 24000,
        "tracks": {
            "minecraft:visual/moon_angle": {
                "ease": "in_out_sine",
                "modifier": "maximum",
                "keyframes": [
                    { "ticks": 0, "value": 0.0 },
                    { "ticks": 12000, "value": 180.0 }
                ]
            }
        },
        "time_markers": {
            "minecraft:dawn": 0,
            "minecraft:noon": {
                "ticks": 6000,
                "show_in_commands": true
            }
        }
    }))
    .expect("timeline must parse");

    assert_eq!(timeline.period_ticks, Some(24000));
    assert_eq!(timeline.tracks.len(), 1);
    assert_eq!(timeline.time_markers.len(), 2);

    let track = timeline
        .tracks
        .get(&Identifier::parse_static("minecraft:visual/moon_angle"))
        .expect("moon angle track");
    assert_eq!(track.ease, Easing::InOutSine);
    assert_eq!(track.modifier.operation(), AttributeOperation::Maximum);
    assert_eq!(track.keyframes.len(), 2);
    assert_eq!(track.keyframes[0].ticks, 0);
    assert_eq!(track.keyframes[1].ticks, 12000);

    let dawn = timeline
        .time_markers
        .get(&Identifier::parse_static("minecraft:dawn"))
        .expect("dawn marker");
    assert_eq!(dawn.ticks, 0);
    assert!(!dawn.show_in_commands);

    let noon = timeline
        .time_markers
        .get(&Identifier::parse_static("minecraft:noon"))
        .expect("noon marker");
    assert_eq!(noon.ticks, 6000);
    assert!(noon.show_in_commands);
}

#[test]
fn applies_track_and_collection_defaults() {
    let timeline = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "tracks": {
            "minecraft:visual/sun_angle": {
                "keyframes": [
                    { "ticks": 0, "value": 0.0 }
                ]
            }
        }
    }))
    .expect("timeline must parse");

    assert_eq!(timeline.period_ticks, None);
    assert!(timeline.time_markers.is_empty());

    let track = timeline
        .tracks
        .get(&Identifier::parse_static("minecraft:visual/sun_angle"))
        .expect("sun angle track");
    assert_eq!(track.ease, Easing::Linear);
    assert_eq!(track.modifier.operation(), AttributeOperation::Override);
}

#[test]
fn parses_empty_tracks_and_markers() {
    let timeline = parse_timeline(json!({
        "clock": "minecraft:overworld"
    }))
    .expect("timeline must parse");

    assert!(timeline.tracks.is_empty());
    assert!(timeline.time_markers.is_empty());
}

#[test]
fn parses_cubic_bezier_easing() {
    let timeline = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "tracks": {
            "minecraft:visual/sun_angle": {
                "ease": {
                    "cubic_bezier": [0.42, 0.0, 0.58, 1.0]
                },
                "keyframes": [
                    { "ticks": 0, "value": 0.0 },
                    { "ticks": 12000, "value": 180.0 }
                ]
            }
        }
    }))
    .expect("timeline must parse");

    let track = timeline
        .tracks
        .get(&Identifier::parse_static("minecraft:visual/sun_angle"))
        .expect("sun angle track");
    assert_eq!(track.ease, Easing::CubicBezier([0.42, 0.0, 0.58, 1.0]));
}

#[test]
fn rejects_unknown_modifier() {
    let result = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "tracks": {
            "minecraft:visual/sun_angle": {
                "modifier": "not_a_modifier",
                "keyframes": [
                    { "ticks": 0, "value": 0.0 }
                ]
            }
        }
    }));

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_cubic_bezier_length() {
    let result = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "tracks": {
            "minecraft:visual/sun_angle": {
                "ease": {
                    "cubic_bezier": [0.42, 0.0, 0.58]
                },
                "keyframes": [
                    { "ticks": 0, "value": 0.0 }
                ]
            }
        }
    }));

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_cubic_bezier_x_control_point() {
    let result = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "tracks": {
            "minecraft:visual/sun_angle": {
                "ease": {
                    "cubic_bezier": [1.2, 0.0, 0.58, 1.0]
                },
                "keyframes": [
                    { "ticks": 0, "value": 0.0 }
                ]
            }
        }
    }));

    assert!(result.is_err());
}

#[test]
fn structure_validation_rejects_zero_period() {
    let timeline = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "period_ticks": 0
    }))
    .expect("timeline syntax must parse");

    assert!(matches!(
        timeline.validate_structure(),
        DataResult::Error { .. }
    ));
}

#[test]
fn structure_validation_rejects_three_keyframes_at_same_tick() {
    let timeline = parse_timeline(json!({
        "clock": "minecraft:overworld",
        "period_ticks": 24000,
        "tracks": {
            "minecraft:visual/sun_angle": {
                "keyframes": [
                    { "ticks": 6000, "value": 0.0 },
                    { "ticks": 6000, "value": 1.0 },
                    { "ticks": 6000, "value": 2.0 }
                ]
            }
        }
    }))
    .expect("timeline syntax must parse");

    assert!(matches!(
        timeline.validate_structure(),
        DataResult::Error { .. }
    ));
}

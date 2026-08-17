use pumpkin_codecs::{Decode, json_ops::JsonOps};
use pumpkin_world::dimension_type::DimensionType;
use serde_json::json;

pub fn overworld() -> DimensionType {
    DimensionType::parse(
        json!({
            "ambient_light": 0.0,
            "attributes": {},
            "coordinate_scale": 1.0,
            "has_ceiling": false,
            "has_ender_dragon_fight": false,
            "has_skylight": true,
            "height": 384,
            "infiniburn": "#minecraft:infiniburn_overworld",
            "logical_height": 384,
            "min_y": -64,
            "monster_spawn_block_light_limit": 0,
            "monster_spawn_light_level": 7
        }),
        &JsonOps,
    )
    .into_result()
    .expect("overworld benchmark dimension must decode")
}

use pumpkin_data::packet::clientbound::play::{
    SET_DEFAULT_SPAWN_POSITION, SET_DISPLAY_OBJECTIVE, SET_ENTITY_DATA, SET_ENTITY_LINK,
    SET_ENTITY_MOTION, SET_EQUIPMENT, SET_EXPERIENCE, SET_HEALTH, SET_OBJECTIVE, SET_PASSENGERS,
    SET_PLAYER_TEAM, SET_SCORE,
};
use pumpkin_util::version::JavaMinecraftVersion;

#[test]
fn v1_16_scoreboard_and_entity_packet_ids_match_vanilla_order() {
    let expected = [66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77];

    for version in [JavaMinecraftVersion::V_1_16, JavaMinecraftVersion::V_1_16_1] {
        let actual = [
            SET_DEFAULT_SPAWN_POSITION.to_id(version),
            SET_DISPLAY_OBJECTIVE.to_id(version),
            SET_ENTITY_DATA.to_id(version),
            SET_ENTITY_LINK.to_id(version),
            SET_ENTITY_MOTION.to_id(version),
            SET_EQUIPMENT.to_id(version),
            SET_EXPERIENCE.to_id(version),
            SET_HEALTH.to_id(version),
            SET_OBJECTIVE.to_id(version),
            SET_PASSENGERS.to_id(version),
            SET_PLAYER_TEAM.to_id(version),
            SET_SCORE.to_id(version),
        ];

        assert_eq!(actual, expected, "unexpected packet order for {version:?}");
    }
}

use pumpkin_data::{
    Block,
    block_properties::{
        AttachFace, DaylightDetectorLikeProperties, LeverLikeProperties, NoteBlockLikeProperties,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{
    GameMode, Hand,
    math::{position::BlockPos, vector3::Vector3},
};

pub(super) async fn test_block_tuning_and_invert(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let note_pos = BlockPos::new(1, 1, 2);
    let detector_pos = BlockPos::new(2, 1, 2);
    let lever_pos = BlockPos::new(3, 1, 2);
    test.set_block_default(&note_pos, &Block::NOTE_BLOCK)
        .await?;
    test.set_block_default(&detector_pos, &Block::DAYLIGHT_DETECTOR)
        .await?;
    let mut lever = LeverLikeProperties::default(&Block::LEVER);
    lever.face = AttachFace::Floor;
    lever.powered = false;
    test.set_block(&lever_pos, lever.to_state_id(&Block::LEVER))
        .await?;

    let player = test
        .spawn_simulated_player(
            "BlockUseTester",
            Vector3::new(2.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    player.use_block(&test.absolute_pos(&note_pos), Hand::Right)?;
    player.use_block(&test.absolute_pos(&note_pos), Hand::Right)?;
    test.assert_true(
        NoteBlockLikeProperties::from_state_id(test.block_state_id(&note_pos).await).note == 2,
        "Right clicks did not tune the note block twice",
    )?;

    player.use_block(&test.absolute_pos(&detector_pos), Hand::Right)?;
    test.assert_true(
        DaylightDetectorLikeProperties::from_state_id(test.block_state_id(&detector_pos).await)
            .inverted,
        "Right click did not invert the daylight detector",
    )?;

    player.use_block(&test.absolute_pos(&lever_pos), Hand::Right)?;
    test.assert_true(
        LeverLikeProperties::from_state_id(test.block_state_id(&lever_pos).await).powered,
        "Right click did not power the lever",
    )?;
    test.succeed();
    Ok(())
}

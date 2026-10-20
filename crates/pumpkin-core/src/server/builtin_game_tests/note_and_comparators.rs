use pumpkin_data::{
    Block,
    block_properties::{
        ComparatorLikeProperties, EastRedstone, EndPortalFrameLikeProperties, HorizontalFacing,
        ModeComparator, NorthRedstone, NoteBlockLikeProperties, NoteblockInstrument,
        RedstoneWireLikeProperties, RespawnAnchorLikeProperties, SouthRedstone, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_note_block_instrument_and_power(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let base_pos = BlockPos::new(1, 1, 2);
    let note_pos = BlockPos::new(1, 2, 2);
    let power_pos = BlockPos::new(2, 2, 2);

    helper
        .set_block_default(&note_pos, &Block::NOTE_BLOCK)
        .await?;

    // Placing GOLD_BLOCK beneath the note block updates its instrument to Bell via neighbor shape update
    helper
        .set_block_default(&base_pos, &Block::GOLD_BLOCK)
        .await?;
    let bell_state = helper.block_state_id(&note_pos).await;
    let bell_props = NoteBlockLikeProperties::from_state_id(bell_state);
    helper.assert_true(
        bell_props.instrument == NoteblockInstrument::Bell,
        format!(
            "expected note block instrument Bell above gold_block, found {:?}",
            bell_props.instrument
        ),
    )?;

    // Replacing base with SAND updates the instrument to Snare
    helper.set_block_default(&base_pos, &Block::SAND).await?;
    let snare_state = helper.block_state_id(&note_pos).await;
    let snare_props = NoteBlockLikeProperties::from_state_id(snare_state);
    helper.assert_true(
        snare_props.instrument == NoteblockInstrument::Snare,
        format!(
            "expected note block instrument Snare above sand, found {:?}",
            snare_props.instrument
        ),
    )?;

    // Powering adjacent block sets powered = true
    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;
    let powered_state = helper.block_state_id(&note_pos).await;
    let powered_props = NoteBlockLikeProperties::from_state_id(powered_state);
    helper.assert_true(
        powered_props.powered,
        "expected note block to become powered when adjacent to redstone_block",
    )?;

    // Removing power sets powered = false
    helper.set_block_default(&power_pos, &Block::AIR).await?;
    let unpowered_state = helper.block_state_id(&note_pos).await;
    let unpowered_props = NoteBlockLikeProperties::from_state_id(unpowered_state);
    helper.assert_true(
        !unpowered_props.powered,
        "expected note block to become unpowered after removing redstone_block",
    )?;

    helper.succeed();
    Ok(())
}

pub(super) async fn test_end_portal_frame_and_respawn_anchor_comparators(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let mut wire_props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    wire_props.north = NorthRedstone::Side;
    wire_props.south = SouthRedstone::Side;
    wire_props.east = EastRedstone::None;
    wire_props.west = WestRedstone::None;
    let wire_state = wire_props.to_state_id(&Block::REDSTONE_WIRE);

    let mut comp_props = ComparatorLikeProperties::default(&Block::COMPARATOR);
    comp_props.facing = HorizontalFacing::North;
    comp_props.mode = ModeComparator::Compare;
    let comp_state = comp_props.to_state_id(&Block::COMPARATOR);

    // Circuit 1: End portal frame with eye -> comparator output 15
    let frame_pos = BlockPos::new(1, 1, 1);
    let frame_comp = BlockPos::new(1, 1, 2);
    let frame_wire = BlockPos::new(1, 1, 3);
    helper.set_block(&frame_wire, wire_state).await?;
    helper.set_block(&frame_comp, comp_state).await?;

    let mut frame_props = EndPortalFrameLikeProperties::default(&Block::END_PORTAL_FRAME);
    frame_props.eye = true;
    helper
        .set_block(
            &frame_pos,
            frame_props.to_state_id(&Block::END_PORTAL_FRAME),
        )
        .await?;

    // Circuit 2: Respawn anchor with charges = 2 -> comparator output 7 (2 * 15 / 4)
    let anchor_pos = BlockPos::new(3, 1, 1);
    let anchor_comp = BlockPos::new(3, 1, 2);
    let anchor_wire = BlockPos::new(3, 1, 3);
    helper.set_block(&anchor_wire, wire_state).await?;
    helper.set_block(&anchor_comp, comp_state).await?;

    let mut anchor_props = RespawnAnchorLikeProperties::default(&Block::RESPAWN_ANCHOR);
    anchor_props.charges = 2;
    helper
        .set_block(
            &anchor_pos,
            anchor_props.to_state_id(&Block::RESPAWN_ANCHOR),
        )
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            let frame_wire_state = helper.block_state_id(&frame_wire).await;
            let frame_wire_props = RedstoneWireLikeProperties::from_state_id(frame_wire_state);
            helper.assert_true(
                frame_wire_props.power == 15,
                format!(
                    "expected comparator output 15 from end_portal_frame with eye, found {}",
                    frame_wire_props.power
                ),
            )?;

            let anchor_wire_state = helper.block_state_id(&anchor_wire).await;
            let anchor_wire_props = RedstoneWireLikeProperties::from_state_id(anchor_wire_state);
            helper.assert_true(
                anchor_wire_props.power == 7,
                format!(
                    "expected comparator output 7 from respawn_anchor (charges=2), found {}",
                    anchor_wire_props.power
                ),
            )
        })
        .then_succeed();
    Ok(())
}

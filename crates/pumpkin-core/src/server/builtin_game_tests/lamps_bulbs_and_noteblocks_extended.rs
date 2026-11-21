use pumpkin_data::{
    Block,
    block_properties::{
        CopperBulbLikeProperties, DispenserLikeProperties, Facing, NoteBlockLikeProperties,
        NoteblockInstrument, ObserverLikeProperties, RedstoneOreLikeProperties, TargetProperties,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_copper_bulb_oxidation_variants_toggle(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let bulbs = [
        (1, &Block::COPPER_BULB),
        (3, &Block::EXPOSED_COPPER_BULB),
        (5, &Block::WEATHERED_COPPER_BULB),
        (7, &Block::OXIDIZED_COPPER_BULB),
    ];
    for (x, block) in bulbs {
        let pos = BlockPos::new(x, 1, 2);
        let pwr = BlockPos::new(x, 2, 2);
        helper.set_block_default(&pos, block).await?;
        helper
            .set_block_default(&pwr, &Block::REDSTONE_BLOCK)
            .await?;
        let props = CopperBulbLikeProperties::from_state_id(helper.block_state_id(&pos).await);
        helper.assert_true(
            props.lit && props.powered,
            "Expected copper bulb variant to light and power on rising edge",
        )?;
        helper.set_block_default(&pwr, &Block::AIR).await?;
        let after = CopperBulbLikeProperties::from_state_id(helper.block_state_id(&pos).await);
        helper.assert_true(
            after.lit && !after.powered,
            "Expected copper bulb variant to stay lit on falling edge",
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_waxed_copper_bulb_variants_toggle(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let bulbs = [
        (1, &Block::WAXED_COPPER_BULB),
        (3, &Block::WAXED_EXPOSED_COPPER_BULB),
        (5, &Block::WAXED_WEATHERED_COPPER_BULB),
        (7, &Block::WAXED_OXIDIZED_COPPER_BULB),
    ];
    for (x, block) in bulbs {
        let pos = BlockPos::new(x, 1, 2);
        let pwr = BlockPos::new(x, 2, 2);
        helper.set_block_default(&pos, block).await?;
        helper
            .set_block_default(&pwr, &Block::REDSTONE_BLOCK)
            .await?;
        let props = CopperBulbLikeProperties::from_state_id(helper.block_state_id(&pos).await);
        helper.assert_true(props.lit, "Expected waxed copper bulb variant to light")?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_lamp_stays_lit_while_powered(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let lamp = BlockPos::new(1, 1, 1);
    let pwr = BlockPos::new(2, 1, 1);
    helper
        .set_block_default(&lamp, &Block::REDSTONE_LAMP)
        .await?;
    helper
        .set_block_default(&pwr, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            let lit =
                RedstoneOreLikeProperties::from_state_id(helper.block_state_id(&lamp).await).lit;
            helper.assert_true(
                lit,
                "Expected redstone_lamp to stay lit while power remains",
            )
        })
        .then_succeed();
    Ok(())
}

async fn assert_note_instruments(
    helper: &GameTestHelper,
    pairs: [(i32, &Block, NoteblockInstrument); 4],
) -> GameTestResult<()> {
    build_stone_floor(helper, 8, 4).await?;
    for (x, base_block, expected) in pairs {
        let base_pos = BlockPos::new(x, 1, 2);
        let note_pos = BlockPos::new(x, 2, 2);
        helper
            .set_block_default(&note_pos, &Block::NOTE_BLOCK)
            .await?;
        helper.set_block_default(&base_pos, base_block).await?;
        let actual = NoteBlockLikeProperties::from_state_id(helper.block_state_id(&note_pos).await)
            .instrument;
        helper.assert_true(
            actual == expected,
            format!("Expected note_block instrument {expected:?}, got {actual:?}"),
        )?;
    }
    Ok(())
}

pub(super) async fn test_note_block_instruments_basedrum_snare_hat_bass(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    assert_note_instruments(
        &helper,
        [
            (1, &Block::STONE, NoteblockInstrument::Basedrum),
            (3, &Block::SAND, NoteblockInstrument::Snare),
            (5, &Block::GLASS, NoteblockInstrument::Hat),
            (7, &Block::OAK_PLANKS, NoteblockInstrument::Bass),
        ],
    )
    .await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_note_block_instruments_flute_bell_guitar_chime(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    assert_note_instruments(
        &helper,
        [
            (1, &Block::CLAY, NoteblockInstrument::Flute),
            (3, &Block::GOLD_BLOCK, NoteblockInstrument::Bell),
            (5, &Block::WHITE_WOOL, NoteblockInstrument::Guitar),
            (7, &Block::PACKED_ICE, NoteblockInstrument::Chime),
        ],
    )
    .await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_note_block_instruments_xylophone_iron_cowbell_didgeridoo(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    assert_note_instruments(
        &helper,
        [
            (1, &Block::BONE_BLOCK, NoteblockInstrument::Xylophone),
            (3, &Block::IRON_BLOCK, NoteblockInstrument::IronXylophone),
            (5, &Block::SOUL_SAND, NoteblockInstrument::CowBell),
            (7, &Block::PUMPKIN, NoteblockInstrument::Didgeridoo),
        ],
    )
    .await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_note_block_instruments_bit_banjo_pling_harp(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    assert_note_instruments(
        &helper,
        [
            (1, &Block::EMERALD_BLOCK, NoteblockInstrument::Bit),
            (3, &Block::HAY_BLOCK, NoteblockInstrument::Banjo),
            (5, &Block::GOLD_BLOCK, NoteblockInstrument::Bell),
            (7, &Block::DIRT, NoteblockInstrument::Harp),
        ],
    )
    .await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_note_block_top_mob_head_instruments(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let heads = [
        (1, &Block::ZOMBIE_HEAD, NoteblockInstrument::Zombie),
        (3, &Block::SKELETON_SKULL, NoteblockInstrument::Skeleton),
        (5, &Block::CREEPER_HEAD, NoteblockInstrument::Creeper),
        (7, &Block::PIGLIN_HEAD, NoteblockInstrument::Piglin),
    ];
    for (x, head_block, expected) in heads {
        let note_pos = BlockPos::new(x, 1, 2);
        let head_pos = BlockPos::new(x, 2, 2);
        helper
            .set_block_default(&note_pos, &Block::NOTE_BLOCK)
            .await?;
        helper.set_block_default(&head_pos, head_block).await?;
        let actual = NoteBlockLikeProperties::from_state_id(helper.block_state_id(&note_pos).await)
            .instrument;
        helper.assert_true(
            actual == expected,
            format!("Expected note_block top head instrument {expected:?}, got {actual:?}"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_daylight_detector_signal_calculation(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    use crate::block::blocks::redstone::daylight_detector::DaylightDetectorBlock;
    build_stone_floor(&helper, 3, 3).await?;
    let noon_normal = DaylightDetectorBlock::calculate_signal_strength(15, 0.0, false);
    let noon_inverted = DaylightDetectorBlock::calculate_signal_strength(15, 0.0, true);
    let dark_inverted = DaylightDetectorBlock::calculate_signal_strength(0, 0.0, true);
    helper.assert_true(
        noon_normal == 15 && noon_inverted == 0 && dark_inverted == 15,
        format!(
            "Unexpected daylight detector values: {noon_normal}, {noon_inverted}, {dark_inverted}"
        ),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_observer_detects_repeater_and_lamp_changes(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let target_lamp = BlockPos::new(2, 1, 1);
    let observer = BlockPos::new(2, 1, 2);
    let bulb = BlockPos::new(2, 1, 3);

    helper
        .set_block_default(&target_lamp, &Block::REDSTONE_LAMP)
        .await?;
    let mut obs = ObserverLikeProperties::default(&Block::OBSERVER);
    obs.facing = Facing::North;
    obs.powered = false;
    helper
        .set_block(&observer, obs.to_state_id(&Block::OBSERVER))
        .await?;
    helper.set_block_default(&bulb, &Block::COPPER_BULB).await?;

    // Power target_lamp -> state changes to lit=true -> observer pulses -> copper_bulb latches lit=true!
    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(5)
        .then_execute(move |helper| async move {
            let bulb_lit =
                CopperBulbLikeProperties::from_state_id(helper.block_state_id(&bulb).await).lit;
            helper.assert_true(
                bulb_lit,
                "Expected observer watching redstone_lamp to pulse and toggle copper_bulb",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_observer_chain_propagates_pulse(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 6).await?;
    let mut obs = ObserverLikeProperties::default(&Block::OBSERVER);
    obs.facing = Facing::North;
    obs.powered = false;
    let obs_id = obs.to_state_id(&Block::OBSERVER);

    let o1 = BlockPos::new(2, 1, 2);
    let o2 = BlockPos::new(2, 1, 3);
    let bulb = BlockPos::new(2, 1, 4);
    helper.set_block(&o1, obs_id).await?;
    helper.set_block(&o2, obs_id).await?;
    helper.set_block_default(&bulb, &Block::COPPER_BULB).await?;

    // Trigger o1 by placing stone at (2, 1, 1) -> o1 pulses -> o2 sees o1 change and pulses -> bulb lights
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::STONE)
        .await?;

    helper
        .start_sequence()
        .then_idle(8)
        .then_execute(move |helper| async move {
            let lit =
                CopperBulbLikeProperties::from_state_id(helper.block_state_id(&bulb).await).lit;
            helper.assert_true(
                lit,
                "Expected chained observers to propagate pulse to copper_bulb",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_target_block_arrow_hit_20_tick_pulse(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let target = BlockPos::new(2, 1, 2);
    let disp = BlockPos::new(2, 1, 3);

    helper.set_block_default(&target, &Block::TARGET).await?;
    let mut d_props = DispenserLikeProperties::default(&Block::DISPENSER);
    d_props.facing = Facing::North;
    helper
        .set_block(&disp, d_props.to_state_id(&Block::DISPENSER))
        .await?;
    helper
        .set_container_item(&disp, 0, "minecraft:egg", 1)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 3), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            let p = TargetProperties::from_state_id(helper.block_state_id(&target).await).power;
            helper.assert_true(p > 0, "Expected egg projectile to power target block")
        })
        .then_idle(10)
        .then_execute(move |helper| async move {
            let p = TargetProperties::from_state_id(helper.block_state_id(&target).await).power;
            helper.assert_true(p == 0, "Expected target block to reset to 0 after egg hit")
        })
        .then_succeed();
    Ok(())
}

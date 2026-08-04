//! Port of vanilla's `SculkBehaviour` interface.
//!
//! Source: `net/minecraft/world/level/block/SculkBehaviour.java` (73 lines), plus the
//! parts of `SculkSpreader`/`SculkSpreader.ChargeCursor`
//! (`net/minecraft/world/level/block/SculkSpreader.java`) needed to call it.
//!
//! Design doc: `designs/sculk-and-block-social.md`, Step 3. This is pure, unit-testable
//! algorithm: nothing in this codebase drives `SculkSpreader::update_cursors` yet (that
//! is Step 4, out of scope here), so none of this fires in a running server.

use pumpkin_data::BlockStateId;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{Tag, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;

use crate::block::BlockFuture;
use crate::block::blocks::abstract_multiface::FaceSet;
use crate::block::blocks::multiface_spreader::{self, SpreadTarget};
use crate::block::blocks::sculk_vein::{self, SculkVeinSpreaderConfig};

/// `SculkSpreader.ChargeCursor`.
///
/// Reduced to the three fields `attemptUseCharge` actually reads
/// (`getPos`/`getCharge`/`getDecayDelay`). The cursor list, movement, merging, and
/// persistence (`updateDelay`, `facings`, `CODEC`) are Step 4's job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeCursor {
    pos: BlockPos,
    charge: i32,
    decay_delay: i32,
}

impl ChargeCursor {
    #[must_use]
    pub const fn new(pos: BlockPos, charge: i32, decay_delay: i32) -> Self {
        Self {
            pos,
            charge,
            decay_delay,
        }
    }

    #[must_use]
    pub const fn pos(&self) -> BlockPos {
        self.pos
    }

    #[must_use]
    pub const fn charge(&self) -> i32 {
        self.charge
    }

    #[must_use]
    pub const fn decay_delay(&self) -> i32 {
        self.decay_delay
    }
}

/// `SculkSpreader`'s constructor fields.
///
/// These are the constants a `SculkBehaviour` reads through the `spreader` parameter.
/// The cursor list itself (`addCursors`, `updateCursors`, `MAX_CURSORS`,
/// `MAX_CURSOR_DISTANCE`) is Step 4.
#[derive(Debug, Clone, Copy)]
pub struct SculkSpreaderConfig {
    is_world_generation: bool,
    replaceable_blocks: &'static Tag,
    growth_spawn_cost: i32,
    no_growth_radius: i32,
    charge_decay_rate: i32,
    additional_decay_rate: i32,
}

impl SculkSpreaderConfig {
    /// `SculkSpreader.createLevelSpreader()` (line 68-70).
    #[must_use]
    pub const fn level_spreader() -> Self {
        Self {
            is_world_generation: false,
            replaceable_blocks: &pumpkin_data::tag::Block::MINECRAFT_SCULK_REPLACEABLE,
            growth_spawn_cost: 10,
            no_growth_radius: 4,
            charge_decay_rate: 10,
            additional_decay_rate: 5,
        }
    }

    /// `SculkSpreader.createWorldGenSpreader()` (line 72-74).
    #[must_use]
    pub const fn world_gen_spreader() -> Self {
        Self {
            is_world_generation: true,
            replaceable_blocks: &pumpkin_data::tag::Block::MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN,
            growth_spawn_cost: 50,
            no_growth_radius: 1,
            charge_decay_rate: 5,
            additional_decay_rate: 10,
        }
    }

    #[must_use]
    pub const fn is_world_generation(&self) -> bool {
        self.is_world_generation
    }

    #[must_use]
    pub const fn replaceable_blocks(&self) -> &'static Tag {
        self.replaceable_blocks
    }

    #[must_use]
    pub const fn growth_spawn_cost(&self) -> i32 {
        self.growth_spawn_cost
    }

    #[must_use]
    pub const fn no_growth_radius(&self) -> i32 {
        self.no_growth_radius
    }

    #[must_use]
    pub const fn charge_decay_rate(&self) -> i32 {
        self.charge_decay_rate
    }

    #[must_use]
    pub const fn additional_decay_rate(&self) -> i32 {
        self.additional_decay_rate
    }
}

/// World access a `SculkBehaviour` implementation needs beyond plain `BlockAccessor` reads.
///
/// Arbitrary-position writes, the placement/spread sound, and the vein spreader
/// (`SpreadTarget` supertrait). Mirrors `LevelAccessor.setBlock`/`playSound` plus
/// `MultifaceSpreadeableBlock.getSpreader()`'s write path. Implemented for a live world
/// by `sculk_vein::WorldSpreadTarget` (which already implements `SpreadTarget`), and by
/// in-memory test doubles in this module's and `sculk.rs`'s test modules.
pub trait SculkWorld: SpreadTarget {
    /// `LevelAccessor.setBlock(pos, state, flags)` for a write that isn't a multiface
    /// spread placement (growth spawn, vein->sculk conversion, vein regrow/discharge).
    fn set_block(&self, pos: BlockPos, state_id: BlockStateId) -> BlockFuture<'_, ()>;

    /// `LevelAccessor.playSound(null, pos, sound, SoundSource.BLOCKS, 1.0F, 1.0F)`.
    fn play_block_sound(&self, pos: BlockPos, sound: Sound);

    /// Simplified `Block.pushEntitiesUp`: real vanilla diffs old/new collision shapes;
    /// this codebase's existing ports (`farmland.rs`, `dirt_path.rs`) approximate it as
    /// "teleport entities in this column up by one block", and this reuses that same
    /// simplification rather than inventing shape-diff math for sculk specifically.
    fn push_entities_up(&self, pos: BlockPos);
}

/// `SculkBehaviour` interface. Every default method mirrors the Java default exactly;
/// `attempt_use_charge` has no default, matching vanilla's abstract method.
pub trait SculkBehaviour: Send + Sync {
    /// `getSculkSpreadDelay()`.
    fn sculk_spread_delay(&self) -> u8 {
        1
    }

    /// `onDischarged`. Default no-op (matches the interface default; `SculkBlock` never
    /// overrides it, only `SculkVeinBlock` does).
    fn on_discharged<'a>(
        &'a self,
        _world: &'a dyn SculkWorld,
        _pos: BlockPos,
        _random: &'a mut RandomGenerator,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    /// `depositCharge`. Default `false`; unused by `SculkSpreader` itself in vanilla and
    /// not overridden by any of `SculkBlock`/`SculkVeinBlock`/`DEFAULT`.
    fn deposit_charge(&self, _pos: BlockPos, _random: &mut RandomGenerator) -> bool {
        false
    }

    /// `attemptSpreadVein` interface default: `getSpreader().spreadAll(state, level, pos,
    /// postProcess) > 0`, i.e. spread the *normal* (non-same-space) vein spreader from
    /// `source_faces` (the block currently at `pos`'s own face bits, empty for a
    /// non-vein source). `source_is_vein` distinguishes a real `sculk_vein` source from
    /// a plain block acting as a spread source (e.g. a freshly placed `SCULK` block
    /// during `attemptPlaceSculk`), matching vanilla's
    /// `SculkVeinSpreaderConfig.isOtherBlockValidAsSource = !state.is(SCULK_VEIN)`.
    /// `facings` (the cursor's tracked regrow set) is unused by the plain interface
    /// default — only `SculkBehaviour::DEFAULT`'s override inspects it. `postProcess`
    /// (vanilla: marks the target chunk for worldgen post-processing) has no meaning
    /// for the non-worldgen level spreader this codebase targets and is dropped, matching
    /// `multiface_spreader::spread_all`'s own signature (Step 1/2 already made this
    /// simplification).
    fn attempt_spread_vein<'a>(
        &'a self,
        world: &'a dyn SculkWorld,
        pos: BlockPos,
        source_faces: FaceSet,
        source_is_vein: bool,
        _facings: Option<FaceSet>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let config = SculkVeinSpreaderConfig::vein(source_is_vein);
            multiface_spreader::spread_all(&config, world, source_faces, pos).await > 0
        })
    }

    /// `canChangeBlockStateOnSpread`. Default `true`; `SculkBlock` overrides to `false`.
    fn can_change_block_state_on_spread(&self) -> bool {
        true
    }

    /// `updateDecayDelay`. Default: unchanged (`1`); `DEFAULT` overrides to `max(age-1,0)`.
    fn update_decay_delay(&self, _age: i32) -> i32 {
        1
    }

    /// `attemptUseCharge` — the mandatory method, no default in vanilla either.
    fn attempt_use_charge<'a>(
        &'a self,
        cursor: &'a ChargeCursor,
        world: &'a dyn SculkWorld,
        origin_pos: BlockPos,
        random: &'a mut RandomGenerator,
        spreader: &'a SculkSpreaderConfig,
        spread_veins: bool,
    ) -> BlockFuture<'a, i32>;
}

/// `SculkBehaviour.DEFAULT`.
///
/// The fallback used by `ChargeCursor::update` (Step 4) when the block at a cursor's
/// position doesn't implement `SculkBehaviour` at all (e.g. air, or a non-sculk block
/// the cursor wandered onto).
pub struct DefaultSculkBehaviour;

impl SculkBehaviour for DefaultSculkBehaviour {
    /// Mirrors the three branches of the anonymous `DEFAULT` object's override exactly
    /// (`SculkBehaviour.java` lines 15-25): `facings == null` uses the same-space
    /// spreader off the block's own current faces; a non-empty tracked `facings` set
    /// attempts `SculkVeinBlock.regrow` (only when the position is air or water, per
    /// `!state.isAir() && !state.getFluidState().is(WATER) -> false`); an empty (but
    /// present) set falls through to the interface default (normal vein spreader).
    fn attempt_spread_vein<'a>(
        &'a self,
        world: &'a dyn SculkWorld,
        pos: BlockPos,
        source_faces: FaceSet,
        source_is_vein: bool,
        facings: Option<FaceSet>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            match facings {
                None => {
                    let config = SculkVeinSpreaderConfig::same_space(source_is_vein);
                    multiface_spreader::spread_all(&config, world, source_faces, pos).await > 0
                }
                Some(regrow_faces) if !regrow_faces.is_empty() => {
                    // `!state.isAir() && !state.getFluidState().is(Fluids.WATER) -> false`.
                    let state = world.accessor().get_block_state(&pos);
                    let is_water = world
                        .accessor()
                        .get_fluid(&pos)
                        .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER);
                    if !state.is_air() && !is_water {
                        false
                    } else {
                        sculk_vein::regrow(world, pos, regrow_faces).await
                    }
                }
                Some(_) => {
                    let config = SculkVeinSpreaderConfig::vein(source_is_vein);
                    multiface_spreader::spread_all(&config, world, source_faces, pos).await > 0
                }
            }
        })
    }

    fn update_decay_delay(&self, age: i32) -> i32 {
        (age - 1).max(0)
    }

    fn attempt_use_charge<'a>(
        &'a self,
        cursor: &'a ChargeCursor,
        _world: &'a dyn SculkWorld,
        _origin_pos: BlockPos,
        _random: &'a mut RandomGenerator,
        _spreader: &'a SculkSpreaderConfig,
        _spread_veins: bool,
    ) -> BlockFuture<'a, i32> {
        Box::pin(async move {
            if cursor.decay_delay() > 0 {
                cursor.charge()
            } else {
                0
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_cursor_getters_match_constructor() {
        let cursor = ChargeCursor::new(BlockPos::new(1, 2, 3), 500, 1);
        assert_eq!(cursor.pos(), BlockPos::new(1, 2, 3));
        assert_eq!(cursor.charge(), 500);
        assert_eq!(cursor.decay_delay(), 1);
    }

    #[test]
    fn level_spreader_matches_vanilla_constants() {
        let spreader = SculkSpreaderConfig::level_spreader();
        assert!(!spreader.is_world_generation());
        assert_eq!(spreader.growth_spawn_cost(), 10);
        assert_eq!(spreader.no_growth_radius(), 4);
        assert_eq!(spreader.charge_decay_rate(), 10);
        assert_eq!(spreader.additional_decay_rate(), 5);
    }

    #[test]
    fn world_gen_spreader_matches_vanilla_constants() {
        let spreader = SculkSpreaderConfig::world_gen_spreader();
        assert!(spreader.is_world_generation());
        assert_eq!(spreader.growth_spawn_cost(), 50);
        assert_eq!(spreader.no_growth_radius(), 1);
        assert_eq!(spreader.charge_decay_rate(), 5);
        assert_eq!(spreader.additional_decay_rate(), 10);
    }

    struct NoWrites;
    impl SpreadTarget for NoWrites {
        fn accessor(&self) -> &dyn pumpkin_world::world::BlockAccessor {
            panic!("DEFAULT.attemptUseCharge never touches the world")
        }
        fn place(
            &self,
            _spread_pos: crate::block::blocks::multiface_spreader::SpreadPos,
        ) -> BlockFuture<'_, bool> {
            panic!("unexpected call in DEFAULT.attemptUseCharge test")
        }
    }
    impl SculkWorld for NoWrites {
        fn set_block(&self, _pos: BlockPos, _state_id: BlockStateId) -> BlockFuture<'_, ()> {
            panic!("unexpected call in DEFAULT.attemptUseCharge test")
        }
        fn play_block_sound(&self, _pos: BlockPos, _sound: Sound) {
            panic!("unexpected call in DEFAULT.attemptUseCharge test")
        }
        fn push_entities_up(&self, _pos: BlockPos) {
            panic!("unexpected call in DEFAULT.attemptUseCharge test")
        }
    }

    #[tokio::test]
    async fn default_behaviour_attempt_use_charge_holds_charge_while_decay_delay_positive() {
        let behaviour = DefaultSculkBehaviour;
        let cursor = ChargeCursor::new(BlockPos::new(0, 0, 0), 42, 1);
        let mut random = pumpkin_util::random::RandomGenerator::Xoroshiro(
            pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(0),
        );
        let spreader = SculkSpreaderConfig::level_spreader();

        let new_charge = behaviour
            .attempt_use_charge(
                &cursor,
                &NoWrites,
                BlockPos::new(0, 0, 0),
                &mut random,
                &spreader,
                false,
            )
            .await;
        assert_eq!(new_charge, 42);
    }

    #[tokio::test]
    async fn default_behaviour_attempt_use_charge_discharges_once_decay_delay_is_used_up() {
        let behaviour = DefaultSculkBehaviour;
        let cursor = ChargeCursor::new(BlockPos::new(0, 0, 0), 42, 0);
        let mut random = pumpkin_util::random::RandomGenerator::Xoroshiro(
            pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(0),
        );
        let spreader = SculkSpreaderConfig::level_spreader();

        let new_charge = behaviour
            .attempt_use_charge(
                &cursor,
                &NoWrites,
                BlockPos::new(0, 0, 0),
                &mut random,
                &spreader,
                false,
            )
            .await;
        assert_eq!(new_charge, 0);
    }

    #[test]
    fn default_behaviour_update_decay_delay_matches_vanilla() {
        let behaviour = DefaultSculkBehaviour;
        assert_eq!(behaviour.update_decay_delay(1), 0);
        assert_eq!(behaviour.update_decay_delay(0), 0);
    }
}

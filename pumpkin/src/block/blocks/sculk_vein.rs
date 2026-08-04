//! `SculkVeinBlock` port (`net/minecraft/world/level/block/SculkVeinBlock.java`).
//!
//! The simplest real consumer of `abstract_multiface.rs`'s `MultifaceBlockBase` and
//! `multiface_spreader.rs`'s spread-candidate machinery.
//!
//! Scope (design doc `designs/sculk-and-block-social.md`, Step 2): placement,
//! neighbour-update face removal, and the two spreader configs
//! (`veinSpreader`/`sameSpaceSpreader`) with vanilla's exact `stateCanBeReplaced`/
//! `isOtherBlockValidAsSource` rules. Deliberately does NOT implement `SculkBehaviour`
//! (`attemptUseCharge`/`attemptPlaceSculk`/`onDischarged`/`hasSubstrateAccess`) — that's
//! Step 3. Nothing in this codebase drives `SculkSpreader::update_cursors` yet, so this
//! block is inert with respect to the catalyst; `spread_all`/`same_space_spread_from_random_face`
//! below are real, callable spreading entry points for a future driver, matching
//! vanilla's `attemptSpreadVein`/`performBonemeal`-style direct calls.
//!
//! Verified against vanilla data (`Blocks.java` `SCULK_VEIN` registration): no
//! `.lightLevel(...)`, no `.randomTicks()`, and no `randomTick` override in
//! `SculkVeinBlock.java` — sculk vein growth is driven entirely by
//! `SculkSpreader::update_cursors` (Step 4), never by a block random tick. Neither does
//! `GlowLichenBlock.java` override `randomTick` (also confirmed by reading it in full),
//! so this is not sculk-vein-specific; unlike sculk vein, glow lichen *does* emit light
//! (`.lightLevel(GlowLichenBlock.emission(7))` in `Blocks.java`, confirmed by
//! `pumpkin-data`'s generated `Block::GLOW_LICHEN` states showing `luminance: 7` once
//! any face is present) — `Block::SCULK_VEIN`'s states are all `luminance: 0`.

use std::sync::Arc;

use pumpkin_data::block_properties::{
    BlockProperties, GlowLichenLikeProperties, WaterLikeProperties,
};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId, FacingExt, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::blocks::abstract_multiface::{
    FaceSet, MultifaceBlockBase, MultifaceProperties, has_any_vacant_face,
};
use crate::block::blocks::multiface_spreader::{
    self, DEFAULT_SPREAD_ORDER, SpreadConfig, SpreadPos, SpreadTarget, SpreadType,
};
use crate::block::{
    BlockBehaviour, BlockFuture, BlockIsReplacing, CanPlaceAtArgs, CanUpdateAtArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs,
};
use crate::entity::EntityBase;
use crate::world::World;

/// `SAME_POSITION`-only spread order, matching `SculkVeinBlock`'s `sameSpaceSpreader`.
const SAME_SPACE_SPREAD_ORDER: [SpreadType; 1] = [SpreadType::SamePosition];

#[pumpkin_block("minecraft:sculk_vein")]
pub struct SculkVeinBlock;

impl MultifaceBlockBase for SculkVeinBlock {}

/// Extracts this position's vein faces if (and only if) it currently holds
/// `sculk_vein`, mirroring vanilla's repeated `oldState.is(this)`/`state.is(Blocks.SCULK_VEIN)`
/// guards before treating a state as carrying face data.
fn existing_vein_faces(state: &BlockState) -> Option<FaceSet> {
    (Block::from_state_id(state.id) == &Block::SCULK_VEIN)
        .then(|| GlowLichenLikeProperties::from_state_id(state.id, &Block::SCULK_VEIN).faces())
}

/// `existingState.is(Blocks.WATER) && existingState.getFluidState().isSource()`.
fn is_water_source(state: &BlockState) -> bool {
    let block = Block::from_state_id(state.id);
    block == &Block::WATER && WaterLikeProperties::from_state_id(state.id, block).level == 0
}

/// `MultifaceSpreader.DefaultSpreaderConfig.stateCanBeReplaced`.
fn default_state_can_be_replaced(existing_state: &BlockState) -> bool {
    existing_state.is_air()
        || Block::from_state_id(existing_state.id) == &Block::SCULK_VEIN
        || is_water_source(existing_state)
}

/// `SculkVeinBlock.SculkVeinSpreaderConfig`.
pub struct SculkVeinSpreaderConfig {
    spread_types: &'static [SpreadType],
}

impl SculkVeinSpreaderConfig {
    #[must_use]
    pub const fn vein() -> Self {
        Self {
            spread_types: &DEFAULT_SPREAD_ORDER,
        }
    }

    #[must_use]
    pub const fn same_space() -> Self {
        Self {
            spread_types: &SAME_SPACE_SPREAD_ORDER,
        }
    }
}

impl SpreadConfig for SculkVeinSpreaderConfig {
    fn spread_types(&self) -> &'static [SpreadType] {
        self.spread_types
    }

    // Deliberately `false` (matches `DefaultSpreaderConfig`'s own default): vanilla's
    // override (`!state.is(Blocks.SCULK_VEIN)`) exists so `attemptPlaceSculk` can spread
    // from a freshly-placed plain `SCULK` block (not a vein) as the source — that call
    // path belongs to `SculkBehaviour.attemptUseCharge`, Step 3/4, not in scope here. The
    // only source this Step 2 config is ever driven from is a real, already-placed
    // `sculk_vein` block, for which vanilla's own formula evaluates to `false` anyway.
    fn is_other_block_valid_as_source(&self, _faces: FaceSet) -> bool {
        false
    }

    fn can_spread_into(
        &self,
        accessor: &dyn BlockAccessor,
        source_pos: &BlockPos,
        spread_pos: SpreadPos,
    ) -> bool {
        if !state_can_be_replaced(accessor, *source_pos, spread_pos) {
            return false;
        }
        let existing_state = accessor.get_block_state(&spread_pos.pos);
        let existing_faces = existing_vein_faces(existing_state);
        SculkVeinBlock.is_valid_state_for_placement(
            accessor,
            existing_faces,
            &spread_pos.pos,
            spread_pos.face,
        )
    }
}

/// `SpreadConfig.placeBlock`'s state-computation half (the write itself is the
/// caller's responsibility, since it differs between a live `World` and a test double):
/// given the position's current state and a spread target, returns the new
/// `sculk_vein` state id to write, or `None` if placement is not valid there (mirrors
/// `MultifaceBlock.getStateForPlacement` returning `null`).
fn compute_spread_placement_state(
    accessor: &dyn BlockAccessor,
    existing_state: &BlockState,
    spread_pos: SpreadPos,
) -> Option<BlockStateId> {
    let existing_faces = existing_vein_faces(existing_state);
    let new_faces = SculkVeinBlock.faces_for_placement(
        accessor,
        existing_faces,
        &spread_pos.pos,
        spread_pos.face,
    )?;

    let mut props = if existing_faces.is_some() {
        GlowLichenLikeProperties::from_state_id(existing_state.id, &Block::SCULK_VEIN)
    } else {
        let mut default_props = GlowLichenLikeProperties::default(&Block::SCULK_VEIN);
        default_props.r#waterlogged = is_water_source(existing_state);
        default_props
    };
    props.set_faces(new_faces);
    Some(props.to_state_id(&Block::SCULK_VEIN))
}

/// `SculkVeinBlock.SculkVeinSpreaderConfig#stateCanBeReplaced`.
fn state_can_be_replaced(
    accessor: &dyn BlockAccessor,
    source_pos: BlockPos,
    spread_pos: SpreadPos,
) -> bool {
    let against_pos = spread_pos.pos.offset(spread_pos.face.to_offset());
    let against_block = accessor.get_block(&against_pos);
    if against_block == &Block::SCULK
        || against_block == &Block::SCULK_CATALYST
        || against_block == &Block::MOVING_PISTON
    {
        return false;
    }

    if source_pos.manhattan_distance(spread_pos.pos) == 2 {
        let neighbour_pos = source_pos.offset(spread_pos.face.opposite().to_offset());
        let neighbour_state = accessor.get_block_state(&neighbour_pos);
        if neighbour_state.is_side_solid(spread_pos.face) {
            return false;
        }
    }

    let existing_fluid = accessor.get_fluid(&spread_pos.pos);
    if existing_fluid != pumpkin_data::fluid::Fluid::EMPTY
        && !existing_fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)
    {
        return false;
    }

    let existing_state = accessor.get_block_state(&spread_pos.pos);
    let existing_block = Block::from_state_id(existing_state.id);
    if existing_block.has_tag(&tag::Block::MINECRAFT_FIRE) {
        return false;
    }

    existing_state.replaceable() || default_state_can_be_replaced(existing_state)
}

impl BlockBehaviour for SculkVeinBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let existing_faces = match args.replacing {
                BlockIsReplacing::Itself(state_id) => {
                    Some(GlowLichenLikeProperties::from_state_id(state_id, args.block).faces())
                }
                _ => None,
            };

            let mut candidates: Vec<BlockDirection> = args
                .player
                .get_entity()
                .get_entity_facing_order()
                .into_iter()
                .map(|f| f.to_block_direction())
                .collect();
            if let Some(idx) = candidates.iter().position(|&d| d == args.direction) {
                candidates.remove(idx);
            }
            candidates.insert(0, args.direction);

            for direction in candidates {
                if let Some(new_faces) =
                    self.faces_for_placement(args.world, existing_faces, args.position, direction)
                {
                    let mut props = if existing_faces.is_some() {
                        GlowLichenLikeProperties::from_state_id(
                            args.world.get_block_state_id(args.position),
                            args.block,
                        )
                    } else {
                        let mut default_props = GlowLichenLikeProperties::default(args.block);
                        default_props.r#waterlogged = args.replacing.water_source();
                        default_props
                    };
                    props.set_faces(new_faces);
                    return props.to_state_id(args.block);
                }
            }

            Block::AIR.default_state.id
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(direction) = args.direction
            && self.is_valid_state_for_placement(
                args.block_accessor,
                None,
                args.position,
                direction,
            )
        {
            return true;
        }
        BlockDirection::all().into_iter().any(|direction| {
            self.is_valid_state_for_placement(args.block_accessor, None, args.position, direction)
        })
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        // `SculkVeinBlock` has no `canBeReplaced` override, so it falls back to
        // `MultifaceBlock.canBeReplaced`: `!itemInHand.is(this) || hasAnyVacantFace(state)`.
        // Since this hook only fires when the clicked block already is this block (see
        // `BlockRegistry::place_block`'s `clicked_block == placed_block` guard), the
        // item-in-hand half is always true here, so this reduces to a vacant-face check.
        // Deliberately not direction-specific: `registry.rs`'s two call sites hand this
        // hook the clicked face un-flipped in one case and pre-flipped in the other, so a
        // direction-based gate here would be call-site-dependent rather than matching
        // vanilla. `on_place` (which walks the full facing order) is what actually picks
        // a workable face.
        let existing_faces =
            GlowLichenLikeProperties::from_state_id(args.state_id, args.block).faces();
        has_any_vacant_face(existing_faces)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = GlowLichenLikeProperties::from_state_id(args.state_id, args.block);
            if props.r#waterlogged {
                args.world.schedule_fluid_tick(
                    &pumpkin_data::fluid::Fluid::WATER,
                    *args.position,
                    pumpkin_data::fluid::Fluid::WATER.flow_speed as u8,
                    TickPriority::Normal,
                );
            }

            let faces = props.faces();
            let neighbor_state = args.neighbor_state_id.to_state();
            self.update_faces_for_neighbor(faces, neighbor_state, args.direction)
                .map_or(Block::AIR.default_state.id, |new_faces| {
                    props.set_faces(new_faces);
                    props.to_state_id(args.block)
                })
        })
    }
}

/// A `SpreadTarget` that actually writes into a live `World`, mirroring vanilla's
/// `DefaultSpreaderConfig.placeBlock` (`level.setBlock(spreadPos.pos(), spreadState, 2)`).
pub struct WorldSpreadTarget<'a> {
    pub world: &'a Arc<World>,
}

impl SpreadTarget for WorldSpreadTarget<'_> {
    fn accessor(&self) -> &dyn BlockAccessor {
        self.world.as_ref()
    }

    fn place(&self, spread_pos: SpreadPos) -> BlockFuture<'_, bool> {
        Box::pin(async move {
            let existing_state = self.world.get_block_state(&spread_pos.pos);
            let Some(new_state_id) =
                compute_spread_placement_state(self.world.as_ref(), existing_state, spread_pos)
            else {
                return false;
            };

            self.world
                .set_block_state(&spread_pos.pos, new_state_id, BlockFlags::NOTIFY_LISTENERS)
                .await;
            true
        })
    }
}

impl SculkVeinBlock {
    /// `getSpreader()`.
    #[must_use]
    pub const fn vein_spreader() -> SculkVeinSpreaderConfig {
        SculkVeinSpreaderConfig::vein()
    }

    /// `getSameSpaceSpreader()`.
    #[must_use]
    pub const fn same_space_spreader() -> SculkVeinSpreaderConfig {
        SculkVeinSpreaderConfig::same_space()
    }

    /// `MultifaceSpreader.spreadAll` driven by this block's `veinSpreader`, against a
    /// live world. Real, callable spreading — not wired to any automatic driver yet
    /// (Step 3/4). Returns 0 if `pos` does not currently hold `sculk_vein`.
    pub async fn spread_all(world: &Arc<World>, pos: BlockPos) -> u64 {
        let state = world.get_block_state(&pos);
        let Some(faces) = existing_vein_faces(state) else {
            return 0;
        };
        let config = Self::vein_spreader();
        let target = WorldSpreadTarget { world };
        multiface_spreader::spread_all(&config, &target, faces, pos).await
    }

    /// `MultifaceSpreader.spreadFromRandomFaceTowardRandomDirection` driven by this
    /// block's `sameSpaceSpreader`.
    pub async fn same_space_spread_from_random_face(
        world: &Arc<World>,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) -> Option<SpreadPos> {
        let state = world.get_block_state(&pos);
        let faces = existing_vein_faces(state)?;
        let config = Self::same_space_spreader();
        let target = WorldSpreadTarget { world };
        multiface_spreader::spread_from_random_face_toward_random_direction(
            &config, &target, faces, pos, random,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::blocks::abstract_multiface::can_attach_to;
    use pumpkin_data::fluid::Fluid;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct FakeAccessor {
        states: HashMap<BlockPos, &'static BlockState>,
        fluids: HashMap<BlockPos, Fluid>,
        default: &'static BlockState,
    }

    impl FakeAccessor {
        fn new(default: &'static BlockState) -> Self {
            Self {
                states: HashMap::new(),
                fluids: HashMap::new(),
                default,
            }
        }

        fn with(mut self, pos: BlockPos, state: &'static BlockState) -> Self {
            self.states.insert(pos, state);
            self
        }
    }

    impl BlockAccessor for FakeAccessor {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.states.get(position).copied().unwrap_or(self.default)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, position: &BlockPos) -> Fluid {
            self.fluids.get(position).cloned().unwrap_or(Fluid::EMPTY)
        }
    }

    /// A `SpreadTarget` writing into a shared, in-memory state map, so tests can drive
    /// the real `multiface_spreader` algorithm end-to-end without a live `World`.
    struct RecordingTarget {
        states: Mutex<HashMap<BlockPos, &'static BlockState>>,
        default: &'static BlockState,
    }

    impl RecordingTarget {
        fn new(default: &'static BlockState) -> Self {
            Self {
                states: Mutex::new(HashMap::new()),
                default,
            }
        }

        fn with(self, pos: BlockPos, state: &'static BlockState) -> Self {
            self.states.lock().unwrap().insert(pos, state);
            self
        }

        fn state_at(&self, pos: BlockPos) -> &'static BlockState {
            self.states
                .lock()
                .unwrap()
                .get(&pos)
                .copied()
                .unwrap_or(self.default)
        }
    }

    impl BlockAccessor for RecordingTarget {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.state_at(*position)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, _position: &BlockPos) -> Fluid {
            Fluid::EMPTY
        }
    }

    impl SpreadTarget for RecordingTarget {
        fn accessor(&self) -> &dyn BlockAccessor {
            self
        }

        fn place(&self, spread_pos: SpreadPos) -> BlockFuture<'_, bool> {
            Box::pin(async move {
                let existing = self.state_at(spread_pos.pos);
                let Some(new_state_id) = compute_spread_placement_state(self, existing, spread_pos)
                else {
                    return false;
                };
                self.states
                    .lock()
                    .unwrap()
                    .insert(spread_pos.pos, new_state_id.to_state());
                true
            })
        }
    }

    fn vein_state(faces: &[BlockDirection]) -> &'static BlockState {
        let mut props = GlowLichenLikeProperties::default(&Block::SCULK_VEIN);
        props.set_faces(FaceSet::from_directions(faces.iter().copied()));
        props.to_state_id(&Block::SCULK_VEIN).to_state()
    }

    #[test]
    fn sculk_vein_uses_glow_lichen_like_properties() {
        // Confirms the block's generated state-property shape matches glow lichen's,
        // per the design doc's Step 2 requirement.
        assert!(GlowLichenLikeProperties::handles_block_id(
            Block::SCULK_VEIN.id
        ));
    }

    #[test]
    fn no_light_emission_or_random_ticks_in_vanilla_data() {
        // Verified against generated pumpkin-data (Blocks.java has no .lightLevel(...)
        // or .randomTicks() for SCULK_VEIN) -- unlike glow lichen, sculk vein neither
        // glows nor random-ticks; its growth is cursor-driven only (Step 3/4).
        for state in Block::SCULK_VEIN.states {
            assert_eq!(state.luminance, 0);
            assert!(!state.has_random_ticks());
        }
    }

    #[test]
    fn existing_vein_faces_reads_faces_only_for_vein_blocks() {
        let vein = vein_state(&[BlockDirection::North, BlockDirection::Up]);
        let faces = existing_vein_faces(vein).unwrap();
        assert!(faces.contains(BlockDirection::North));
        assert!(faces.contains(BlockDirection::Up));
        assert!(!faces.contains(BlockDirection::South));

        assert_eq!(existing_vein_faces(Block::STONE.default_state), None);
    }

    #[test]
    fn state_can_be_replaced_rejects_sculk_and_catalyst_and_moving_piston() {
        let pos = BlockPos::new(0, 0, 0);
        let against = pos.offset(BlockDirection::North.to_offset());
        for against_block_state in [
            Block::SCULK.default_state,
            Block::SCULK_CATALYST.default_state,
            Block::MOVING_PISTON.default_state,
        ] {
            let accessor =
                FakeAccessor::new(Block::AIR.default_state).with(against, against_block_state);
            assert!(!state_can_be_replaced(
                &accessor,
                pos,
                SpreadPos {
                    pos,
                    face: BlockDirection::North
                }
            ));
        }
    }

    #[test]
    fn state_can_be_replaced_allows_air_against_sturdy_support() {
        let source = BlockPos::new(0, 0, 0);
        let target = source.offset(BlockDirection::Up.to_offset());
        let against = target.offset(BlockDirection::North.to_offset());
        let accessor =
            FakeAccessor::new(Block::AIR.default_state).with(against, Block::STONE.default_state);

        assert!(state_can_be_replaced(
            &accessor,
            source,
            SpreadPos {
                pos: target,
                face: BlockDirection::North
            }
        ));
    }

    #[test]
    fn state_can_be_replaced_rejects_non_water_fluid() {
        let pos = BlockPos::new(0, 0, 0);
        let mut accessor = FakeAccessor::new(Block::AIR.default_state);
        accessor.fluids.insert(pos, Fluid::LAVA);

        assert!(!state_can_be_replaced(
            &accessor,
            BlockPos::new(5, 5, 5),
            SpreadPos {
                pos,
                face: BlockDirection::North
            }
        ));
    }

    #[test]
    fn state_can_be_replaced_rejects_fire() {
        let pos = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::FIRE.default_state);

        assert!(!state_can_be_replaced(
            &accessor,
            BlockPos::new(5, 5, 5),
            SpreadPos {
                pos,
                face: BlockDirection::North
            }
        ));
    }

    #[test]
    fn state_can_be_replaced_rejects_when_two_step_source_face_is_sturdy() {
        // distManhattan(source, placement) == 2: the "diagonal step" case, blocked when
        // the face of the block adjacent to the source (opposite the placement
        // direction) is sturdy toward that same direction.
        let source = BlockPos::new(0, 0, 0);
        let target = source
            .offset(BlockDirection::Up.to_offset())
            .offset(BlockDirection::North.to_offset());
        let neighbour = source.offset(BlockDirection::South.to_offset());
        let accessor =
            FakeAccessor::new(Block::AIR.default_state).with(neighbour, Block::STONE.default_state);

        assert!(!state_can_be_replaced(
            &accessor,
            source,
            SpreadPos {
                pos: target,
                face: BlockDirection::North
            }
        ));
    }

    #[tokio::test]
    async fn spread_all_actually_grows_a_placed_vein_onto_a_neighbour() {
        // The core "genuinely spreads once placed" check: a vein occupying the North
        // face of the origin, with a sturdy stone block to the East of the origin,
        // should grow a new East-facing vein face at the same position via SAME_POSITION
        // spread (the vanilla-observed common case for a corner block).
        let pos = BlockPos::new(0, 0, 0);
        let east = pos.offset(BlockDirection::East.to_offset());
        let origin_vein = vein_state(&[BlockDirection::North]);

        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, origin_vein)
            .with(east, Block::STONE.default_state);

        let config = SculkVeinBlock::vein_spreader();
        let count = multiface_spreader::spread_all(
            &config,
            &target,
            existing_vein_faces(origin_vein).unwrap(),
            pos,
        )
        .await;

        assert!(count > 0);
        let new_faces = existing_vein_faces(target.state_at(pos)).unwrap();
        assert!(new_faces.contains(BlockDirection::North));
        assert!(new_faces.contains(BlockDirection::East));
    }

    #[tokio::test]
    async fn spread_all_does_nothing_when_no_neighbour_can_support_a_new_face() {
        let pos = BlockPos::new(0, 0, 0);
        let origin_vein = vein_state(&[BlockDirection::North]);
        let target = RecordingTarget::new(Block::AIR.default_state).with(pos, origin_vein);

        let config = SculkVeinBlock::vein_spreader();
        let count = multiface_spreader::spread_all(
            &config,
            &target,
            existing_vein_faces(origin_vein).unwrap(),
            pos,
        )
        .await;

        assert_eq!(count, 0);
        assert_eq!(target.state_at(pos), origin_vein);
    }

    #[test]
    fn can_attach_to_matches_side_solid() {
        assert!(can_attach_to(
            Block::STONE.default_state,
            BlockDirection::North
        ));
        assert!(!can_attach_to(
            Block::AIR.default_state,
            BlockDirection::North
        ));
    }
}

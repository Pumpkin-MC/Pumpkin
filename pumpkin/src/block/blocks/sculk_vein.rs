//! `SculkVeinBlock` port (`net/minecraft/world/level/block/SculkVeinBlock.java`).
//!
//! The simplest real consumer of `abstract_multiface.rs`'s `MultifaceBlockBase` and
//! `multiface_spreader.rs`'s spread-candidate machinery.
//!
//! Scope (design doc `designs/sculk-and-block-social.md`, Step 2): placement,
//! neighbour-update face removal, and the two spreader configs
//! (`veinSpreader`/`sameSpaceSpreader`) with vanilla's exact `stateCanBeReplaced`/
//! `isOtherBlockValidAsSource` rules. `spread_all`/`same_space_spread_from_random_face`
//! below are real, callable spreading entry points for a future driver, matching
//! vanilla's `attemptSpreadVein`/`performBonemeal`-style direct calls.
//!
//! Step 3 (this file's `SculkBehaviour for SculkVeinBlock` impl, `regrow`,
//! `has_substrate_access`, and `attempt_place_sculk`) adds the real charge-consumer
//! behaviour (`attemptUseCharge`/`attemptPlaceSculk`/`onDischarged`/`hasSubstrateAccess`).
//! Nothing in this codebase drives `SculkSpreader::update_cursors` yet (Step 4), so this
//! block is still inert with respect to the catalyst.
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
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId, FacingExt, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::blocks::abstract_multiface::{
    FaceSet, MultifaceBlockBase, MultifaceProperties, can_attach_to_pos, has_any_vacant_face,
};
use crate::block::blocks::multiface_spreader::{
    self, DEFAULT_SPREAD_ORDER, SpreadConfig, SpreadPos, SpreadTarget, SpreadType,
};
use crate::block::sculk_behaviour::{
    ChargeCursor, SculkBehaviour, SculkSpreaderConfig, SculkWorld,
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
///
/// `source_is_vein` mirrors vanilla's `isOtherBlockValidAsSource(BlockState state) =
/// !state.is(Blocks.SCULK_VEIN)` (`SculkVeinBlock.java` lines 193-195): vanilla reads
/// this off the actual source `BlockState` passed into `MultifaceSpreader.spreadAll`.
/// This config is constructed fresh per call, and the source state is fixed for the
/// duration of one `spreadAll`, so passing the already-known fact "is the source a vein"
/// as a constructor argument is equivalent. Step 2 only ever drove this config from a
/// real, already-placed `sculk_vein` block (`source_is_vein = true`); Step 3 adds the
/// `attemptPlaceSculk` call path, which spreads from a freshly-placed plain `SCULK`
/// block instead (`source_is_vein = false`).
pub struct SculkVeinSpreaderConfig {
    spread_types: &'static [SpreadType],
    source_is_vein: bool,
}

impl SculkVeinSpreaderConfig {
    #[must_use]
    pub const fn vein(source_is_vein: bool) -> Self {
        Self {
            spread_types: &DEFAULT_SPREAD_ORDER,
            source_is_vein,
        }
    }

    #[must_use]
    pub const fn same_space(source_is_vein: bool) -> Self {
        Self {
            spread_types: &SAME_SPACE_SPREAD_ORDER,
            source_is_vein,
        }
    }
}

impl SpreadConfig for SculkVeinSpreaderConfig {
    fn spread_types(&self) -> &'static [SpreadType] {
        self.spread_types
    }

    fn is_other_block_valid_as_source(&self, _faces: FaceSet) -> bool {
        !self.source_is_vein
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

impl SculkWorld for WorldSpreadTarget<'_> {
    fn set_block(&self, pos: BlockPos, state_id: BlockStateId) -> BlockFuture<'_, ()> {
        Box::pin(async move {
            self.world
                .set_block_state(&pos, state_id, BlockFlags::NOTIFY_ALL)
                .await;
        })
    }

    fn play_block_sound(&self, pos: BlockPos, sound: Sound) {
        self.world
            .play_block_sound(sound, SoundCategory::Blocks, pos);
    }

    fn push_entities_up(&self, pos: BlockPos) {
        // Simplified `Block.pushEntitiesUp`, matching the existing approximation in
        // `farmland.rs`/`dirt_path.rs`: teleport entities in this 1x1x1 column up by one
        // block rather than diffing old/new collision shapes.
        let min = Vector3::new(f64::from(pos.0.x), f64::from(pos.0.y), f64::from(pos.0.z));
        let max = Vector3::new(min.x + 1.0, min.y + 1.0, min.z + 1.0);
        let aabb = BoundingBox::new(min, max);
        for entity in self.world.get_entities_at_box(&aabb) {
            let entity = entity.get_entity();
            let entity_pos = entity.pos.load();
            entity
                .pos
                .store(Vector3::new(entity_pos.x, entity_pos.y + 1.0, entity_pos.z));
        }
    }
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

/// `SculkVeinBlock.regrow` (lines 46-67): rebuild a vein state at `pos` keeping only the
/// faces from `faces` that still `canAttachTo` their neighbour, or return `false` (no
/// write) if none survive. Reads `pos`'s current state to preserve its waterlogged bit
/// and existing faces (vanilla starts from a fresh `defaultBlockState()` and only sets
/// `WATERLOGGED` from `existing.getFluidState()`, which this matches since a plain
/// vein's default state carries no other faces to begin with).
pub(crate) async fn regrow(world: &dyn SculkWorld, pos: BlockPos, faces: FaceSet) -> bool {
    let mut new_faces = FaceSet::EMPTY;
    for direction in faces.iter() {
        if can_attach_to_pos(world.accessor(), &pos, direction) {
            new_faces = new_faces.with(direction);
        }
    }
    if new_faces.is_empty() {
        return false;
    }

    let existing_state = world.accessor().get_block_state(&pos);
    let mut props = GlowLichenLikeProperties::default(&Block::SCULK_VEIN);
    props.set_faces(new_faces);
    props.r#waterlogged = world.accessor().get_fluid(&pos) != pumpkin_data::fluid::Fluid::EMPTY
        || is_water_source(existing_state);
    world
        .set_block(pos, props.to_state_id(&Block::SCULK_VEIN))
        .await;
    true
}

/// `SculkVeinBlock.hasSubstrateAccess` (lines 139-151): true if any of this vein's own
/// faces points at a `SCULK_REPLACEABLE` neighbour.
#[must_use]
pub fn has_substrate_access(accessor: &dyn BlockAccessor, pos: BlockPos) -> bool {
    let state = accessor.get_block_state(&pos);
    let Some(faces) = existing_vein_faces(state) else {
        return false;
    };
    faces.iter().any(|direction| {
        let neighbour = pos.offset(direction.to_offset());
        accessor
            .get_block(&neighbour)
            .has_tag(&tag::Block::MINECRAFT_SCULK_REPLACEABLE)
    })
}

/// `SculkVeinBlock.attemptPlaceSculk` (lines 105-137).
async fn attempt_place_sculk(
    world: &dyn SculkWorld,
    spreader: &SculkSpreaderConfig,
    pos: BlockPos,
    random: &mut RandomGenerator,
) -> bool {
    let state = world.accessor().get_block_state(&pos);
    let Some(faces) = existing_vein_faces(state) else {
        return false;
    };

    for support in multiface_spreader::shuffled_directions(random) {
        if !faces.contains(support) {
            continue;
        }
        let support_pos = pos.offset(support.to_offset());
        let support_block = world.accessor().get_block(&support_pos);
        if !support_block.has_tag(spreader.replaceable_blocks()) {
            continue;
        }

        world
            .set_block(support_pos, Block::SCULK.default_state.id)
            .await;
        world.push_entities_up(support_pos);
        world.play_block_sound(support_pos, Sound::BlockSculkSpread);

        let vein_config = SculkVeinSpreaderConfig::vein(false);
        multiface_spreader::spread_all(&vein_config, world, FaceSet::EMPTY, support_pos).await;

        let skip = support.opposite();
        for direction in BlockDirection::all() {
            if direction == skip {
                continue;
            }
            let vein_pos = support_pos.offset(direction.to_offset());
            let vein_state = world.accessor().get_block_state(&vein_pos);
            if Block::from_state_id(vein_state.id) == &Block::SCULK_VEIN {
                SculkVeinBlock.on_discharged(world, vein_pos, random).await;
            }
        }

        return true;
    }

    false
}

impl SculkBehaviour for SculkVeinBlock {
    /// `SculkVeinBlock.onDischarged` (lines 69-87): strip any face pointing at a now-
    /// `SCULK` neighbour; revert to air/water if no faces remain.
    fn on_discharged<'a>(
        &'a self,
        world: &'a dyn SculkWorld,
        pos: BlockPos,
        _random: &'a mut RandomGenerator,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let existing_state = world.accessor().get_block_state(&pos);
            let Some(mut faces) = existing_vein_faces(existing_state) else {
                return;
            };

            for direction in BlockDirection::all() {
                if faces.contains(direction) {
                    let neighbour = pos.offset(direction.to_offset());
                    if world.accessor().get_block(&neighbour) == &Block::SCULK {
                        faces = faces.without(direction);
                    }
                }
            }

            let new_state_id = if faces.is_empty() {
                if world.accessor().get_fluid(&pos) == pumpkin_data::fluid::Fluid::EMPTY {
                    Block::AIR.default_state.id
                } else {
                    Block::WATER.default_state.id
                }
            } else {
                let mut props =
                    GlowLichenLikeProperties::from_state_id(existing_state.id, &Block::SCULK_VEIN);
                props.set_faces(faces);
                props.to_state_id(&Block::SCULK_VEIN)
            };
            world.set_block(pos, new_state_id).await;
        })
    }

    /// `SculkVeinBlock.attemptUseCharge` (lines 90-103).
    fn attempt_use_charge<'a>(
        &'a self,
        cursor: &'a ChargeCursor,
        world: &'a dyn SculkWorld,
        _origin_pos: BlockPos,
        random: &'a mut RandomGenerator,
        spreader: &'a SculkSpreaderConfig,
        spread_veins: bool,
    ) -> BlockFuture<'a, i32> {
        Box::pin(async move {
            if spread_veins && attempt_place_sculk(world, spreader, cursor.pos(), random).await {
                cursor.charge() - 1
            } else if random.next_bounded_i32(spreader.charge_decay_rate()) == 0 {
                // `Mth.floor(cursor.getCharge() * 0.5F)`: exact via integer division since
                // charge is never negative here.
                cursor.charge() / 2
            } else {
                cursor.charge()
            }
        })
    }
}

impl SculkVeinBlock {
    /// `getSpreader()`. Always driven from a real, already-placed vein (see
    /// `spread_all`'s `existing_vein_faces` guard), so `source_is_vein = true`.
    #[must_use]
    pub const fn vein_spreader() -> SculkVeinSpreaderConfig {
        SculkVeinSpreaderConfig::vein(true)
    }

    /// `getSameSpaceSpreader()`. Same `source_is_vein = true` reasoning as above.
    #[must_use]
    pub const fn same_space_spreader() -> SculkVeinSpreaderConfig {
        SculkVeinSpreaderConfig::same_space(true)
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
    use pumpkin_util::random::xoroshiro128::Xoroshiro;
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

    /// A `SpreadTarget`/`SculkWorld` writing into a shared, in-memory state map, so
    /// tests can drive the real `multiface_spreader`/`SculkBehaviour` algorithms
    /// end-to-end without a live `World`.
    struct RecordingTarget {
        states: Mutex<HashMap<BlockPos, &'static BlockState>>,
        fluids: Mutex<HashMap<BlockPos, Fluid>>,
        default: &'static BlockState,
        sounds_played: Mutex<Vec<(BlockPos, Sound)>>,
    }

    impl RecordingTarget {
        fn new(default: &'static BlockState) -> Self {
            Self {
                states: Mutex::new(HashMap::new()),
                fluids: Mutex::new(HashMap::new()),
                default,
                sounds_played: Mutex::new(Vec::new()),
            }
        }

        fn with(self, pos: BlockPos, state: &'static BlockState) -> Self {
            self.states.lock().unwrap().insert(pos, state);
            self
        }

        fn with_fluid(self, pos: BlockPos, fluid: Fluid) -> Self {
            self.fluids.lock().unwrap().insert(pos, fluid);
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

        fn get_fluid(&self, position: &BlockPos) -> Fluid {
            self.fluids
                .lock()
                .unwrap()
                .get(position)
                .cloned()
                .unwrap_or(Fluid::EMPTY)
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

    impl SculkWorld for RecordingTarget {
        fn set_block(&self, pos: BlockPos, state_id: BlockStateId) -> BlockFuture<'_, ()> {
            Box::pin(async move {
                self.states.lock().unwrap().insert(pos, state_id.to_state());
            })
        }

        fn play_block_sound(&self, pos: BlockPos, sound: Sound) {
            self.sounds_played.lock().unwrap().push((pos, sound));
        }

        fn push_entities_up(&self, _pos: BlockPos) {}
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

    #[test]
    fn has_substrate_access_true_when_a_face_points_at_a_replaceable_neighbour() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let vein = vein_state(&[BlockDirection::North]);
        // STONE is in `minecraft:sculk_replaceable`.
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::STONE.default_state);
        assert!(has_substrate_access(&target, pos));
    }

    #[test]
    fn has_substrate_access_false_when_no_face_points_at_a_replaceable_neighbour() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let vein = vein_state(&[BlockDirection::North]);
        // OAK_LOG is not in `minecraft:sculk_replaceable`.
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::OAK_LOG.default_state);
        assert!(!has_substrate_access(&target, pos));
    }

    #[test]
    fn has_substrate_access_false_for_a_non_vein_block() {
        let pos = BlockPos::new(0, 0, 0);
        let target = RecordingTarget::new(Block::STONE.default_state);
        assert!(!has_substrate_access(&target, pos));
    }

    #[tokio::test]
    async fn regrow_keeps_only_faces_that_still_attach() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let up = pos.offset(BlockDirection::Up.to_offset());
        // North still has sturdy support, Up does not.
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(north, Block::STONE.default_state)
            .with(up, Block::AIR.default_state);

        let faces = FaceSet::from_directions([BlockDirection::North, BlockDirection::Up]);
        let regrew = regrow(&target, pos, faces).await;
        assert!(regrew);

        let new_faces = existing_vein_faces(target.state_at(pos)).unwrap();
        assert!(new_faces.contains(BlockDirection::North));
        assert!(!new_faces.contains(BlockDirection::Up));
    }

    #[tokio::test]
    async fn regrow_fails_when_no_face_can_reattach() {
        let pos = BlockPos::new(0, 0, 0);
        let target = RecordingTarget::new(Block::AIR.default_state);
        let faces = FaceSet::from_directions([BlockDirection::North]);
        assert!(!regrow(&target, pos, faces).await);
        // No write happened.
        assert_eq!(target.state_at(pos), Block::AIR.default_state);
    }

    #[tokio::test]
    async fn on_discharged_strips_faces_pointing_at_sculk_neighbours() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let up = pos.offset(BlockDirection::Up.to_offset());
        let vein = vein_state(&[BlockDirection::North, BlockDirection::Up]);
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::SCULK.default_state)
            .with(up, Block::STONE.default_state);

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(0));
        SculkVeinBlock
            .on_discharged(&target, pos, &mut random)
            .await;

        let remaining = existing_vein_faces(target.state_at(pos)).unwrap();
        assert!(!remaining.contains(BlockDirection::North));
        assert!(remaining.contains(BlockDirection::Up));
    }

    #[tokio::test]
    async fn on_discharged_reverts_to_air_when_no_faces_remain() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let vein = vein_state(&[BlockDirection::North]);
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::SCULK.default_state);

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(0));
        SculkVeinBlock
            .on_discharged(&target, pos, &mut random)
            .await;

        assert_eq!(target.state_at(pos), Block::AIR.default_state);
    }

    #[tokio::test]
    async fn on_discharged_reverts_to_water_when_waterlogged_and_no_faces_remain() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let vein = vein_state(&[BlockDirection::North]);
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::SCULK.default_state)
            .with_fluid(pos, Fluid::WATER);

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(0));
        SculkVeinBlock
            .on_discharged(&target, pos, &mut random)
            .await;

        assert_eq!(target.state_at(pos), Block::WATER.default_state);
    }

    #[tokio::test]
    async fn attempt_use_charge_places_sculk_and_spends_one_charge_when_a_replaceable_face_exists()
    {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let vein = vein_state(&[BlockDirection::North]);
        // STONE is in `minecraft:sculk_replaceable`, so attemptPlaceSculk always
        // succeeds regardless of the direction shuffle (this vein has only one face).
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::STONE.default_state);

        let cursor = ChargeCursor::new(pos, 100, 1);
        let spreader = SculkSpreaderConfig::level_spreader();
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(0));

        let new_charge = SculkVeinBlock
            .attempt_use_charge(
                &cursor,
                &target,
                BlockPos::new(0, 0, 0),
                &mut random,
                &spreader,
                true,
            )
            .await;

        assert_eq!(new_charge, 99);
        assert_eq!(
            Block::from_state_id(target.state_at(north).id),
            &Block::SCULK
        );
        assert!(
            target
                .sounds_played
                .lock()
                .unwrap()
                .iter()
                .any(|(p, s)| *p == north && *s == Sound::BlockSculkSpread)
        );
    }

    #[tokio::test]
    async fn attempt_use_charge_holds_or_halves_charge_when_spread_veins_is_false() {
        let pos = BlockPos::new(0, 0, 0);
        let vein = vein_state(&[BlockDirection::North]);
        let target = RecordingTarget::new(Block::AIR.default_state).with(pos, vein);
        let spreader = SculkSpreaderConfig::level_spreader();

        // `spread_veins = false`: attemptPlaceSculk is never attempted, so charge only
        // holds or halves via the decay-rate roll, never drops by exactly 1 from a
        // placement.
        for seed in 0..32u64 {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let cursor = ChargeCursor::new(pos, 100, 1);
            let new_charge = SculkVeinBlock
                .attempt_use_charge(
                    &cursor,
                    &target,
                    BlockPos::new(0, 0, 0),
                    &mut random,
                    &spreader,
                    false,
                )
                .await;
            assert!(new_charge == 100 || new_charge == 50);
        }
    }

    #[tokio::test]
    async fn attempt_use_charge_does_nothing_without_a_replaceable_face() {
        let pos = BlockPos::new(0, 0, 0);
        let north = pos.offset(BlockDirection::North.to_offset());
        let vein = vein_state(&[BlockDirection::North]);
        // North is an oak log: not in `minecraft:sculk_replaceable`, so
        // attemptPlaceSculk always fails.
        let target = RecordingTarget::new(Block::AIR.default_state)
            .with(pos, vein)
            .with(north, Block::OAK_LOG.default_state);
        let spreader = SculkSpreaderConfig::level_spreader();

        for seed in 0..32u64 {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let cursor = ChargeCursor::new(pos, 100, 1);
            let new_charge = SculkVeinBlock
                .attempt_use_charge(
                    &cursor,
                    &target,
                    BlockPos::new(0, 0, 0),
                    &mut random,
                    &spreader,
                    true,
                )
                .await;
            assert!(new_charge == 100 || new_charge == 50);
        }
    }
}

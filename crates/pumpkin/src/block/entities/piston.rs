use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::{
    Axis as BlockAxis, BlockProperties, PistonHeadLikeProperties,
    StickyPistonLikeProperties as PistonProps,
};
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::block::{block_state_from_nbt, block_state_to_nbt};

use crate::block::blocks::piston::piston::PistonBlock;
use crate::world::{BlockFlags, World};

use super::BlockEntity;

pub struct PistonBlockEntity {
    pub position: BlockPos,
    pub pushed_block_state: &'static BlockState,
    pub facing: BlockDirection,
    pub current_progress: AtomicCell<f32>,
    pub last_progress: AtomicCell<f32>,
    pub extending: bool,
    pub source: bool,
    /// World age this entity last ticked, `-1` if never. Vanilla `getRetractType` also
    /// checks `lastTicked == level.getGameTime()` (extend can pass 50% in the same tick).
    pub last_ticked: AtomicCell<i64>,
}

impl PistonBlockEntity {
    pub const ID: &'static str = "minecraft:piston";

    /// Animation start: progress 0, `lastTicked` unset (`-1`). Vanilla `PistonMovingBlockEntity` ctor.
    #[must_use]
    pub fn new(
        position: BlockPos,
        facing: BlockDirection,
        pushed_block_state: &'static BlockState,
        extending: bool,
        source: bool,
    ) -> Self {
        Self {
            position,
            facing,
            pushed_block_state,
            current_progress: 0.0.into(),
            last_progress: 0.0.into(),
            extending,
            source,
            last_ticked: (-1i64).into(),
        }
    }

    /// Vanilla `checkIfExtend` `TRIGGER_DROP`: dest still extending and (progress < 0.5,
    /// `lastTicked == gameTime`, or `isHandlingTick`).
    pub fn should_drop_instead_of_pull(&self, world: &World) -> bool {
        self.extending
            && (self.current_progress.load() < 0.5
                || self.last_ticked.load() == world.get_world_age()
                || world.is_handling_tick())
    }

    /// True while this instance is still the live BE at its position. Re-check after every
    /// `.await`: a re-trigger can replace it. Live-map only; `get_block_entity` would rebuild
    /// from NBT and restart the animation.
    fn is_current(&self, world: &World) -> bool {
        world
            .get_live_block_entity(&self.position)
            .is_some_and(|be| {
                be.as_any()
                    .downcast_ref::<Self>()
                    .is_some_and(|piston| std::ptr::eq(piston, self))
            })
    }

    const fn movement_direction(&self) -> BlockDirection {
        if self.extending {
            self.facing
        } else {
            self.facing.opposite()
        }
    }

    /// Vanilla's `getAmountExtended`: how far back from the block's final position
    /// the visual is at a given animation progress. Negative for extending.
    fn amount_extended(&self, progress: f32) -> f32 {
        if self.extending {
            progress - 1.0
        } else {
            1.0 - progress
        }
    }

    fn dir_vec(dir: BlockDirection, scale: f64) -> Vector3<f64> {
        let off = dir.to_offset();
        Vector3::new(
            f64::from(off.x) * scale,
            f64::from(off.y) * scale,
            f64::from(off.z) * scale,
        )
    }

    /// Vanilla `PistonMath.getMovementArea`: slab the leading face of `aabb` sweeps by
    /// `amount` along `dir`. Not `aabb.stretch(motion)` (that includes the block itself).
    fn movement_area(aabb: BoundingBox, dir: BlockDirection, amount: f64) -> BoundingBox {
        let step = f64::from(dir.to_offset().get_axis(dir.to_axis().into()));
        let delta = amount * step;
        let (lo, hi) = (delta.min(0.0), delta.max(0.0));

        let (mut min, mut max) = (aabb.min, aabb.max);
        match dir {
            BlockDirection::West => {
                max.x = aabb.min.x + hi;
                min.x = aabb.min.x + lo;
            }
            BlockDirection::East => {
                min.x = aabb.max.x + lo;
                max.x = aabb.max.x + hi;
            }
            BlockDirection::Down => {
                max.y = aabb.min.y + hi;
                min.y = aabb.min.y + lo;
            }
            BlockDirection::Up => {
                min.y = aabb.max.y + lo;
                max.y = aabb.max.y + hi;
            }
            BlockDirection::North => {
                max.z = aabb.min.z + hi;
                min.z = aabb.min.z + lo;
            }
            BlockDirection::South => {
                min.z = aabb.max.z + lo;
                max.z = aabb.max.z + hi;
            }
        }
        BoundingBox::new(min, max)
    }

    /// Ports vanilla `PistonMovingBlockEntity.moveCollidedEntities`: pushes entities whose
    /// bounding box the moving block's leading face sweeps into during this tick.
    fn push_entities(&self, world: &Arc<World>, new_progress: f32) {
        let last = self.last_progress.load();
        let delta = f64::from(new_progress - last);
        if delta <= 0.0 {
            return;
        }

        let motion_dir = self.movement_direction();

        // Vanilla `moveCollidedEntities`: no collision shape (rail, torch, plant) means no push.
        let shapes = self.animated_shapes(last);
        let Some(block_aabb) = BoundingBox::union_all(&shapes) else {
            return;
        };

        // Vanilla `getMovementArea(aabb, ...).minmax(aabb)`. Overlap is re-measured per sub-box.
        let query = block_aabb.union(&Self::movement_area(block_aabb, motion_dir, delta));

        let launches = Self::launches_entities(self.pushed_block_state);
        let game_time = world.get_world_age();

        for entity in world.get_entities_at_box(&query) {
            let e = entity.get_entity();
            if e.no_physics.load(Ordering::Relaxed) {
                continue;
            }
            // Vanilla: markers, displays, interaction, area effect clouds. No launch, no shove.
            if e.ignores_piston_push() {
                continue;
            }

            if launches {
                // Vanilla skips `ServerPlayer`. The client (`LocalPlayer`) applies the fling.
                if entity.get_player().is_some() {
                    continue;
                }

                // Vanilla sets this as soon as the entity is in the list: replace the movement
                // axis with a full block per tick, leave the other two.
                let axis = motion_dir.to_axis().into();
                let unit = Self::dir_vec(motion_dir, 1.0);
                let mut velocity = e.velocity.load();
                velocity.set_axis(axis, unit.get_axis(axis));
                e.velocity.store(velocity);
                e.send_velocity();
            }

            // Player position is client-authoritative. Vanilla uses `Entity.move(PISTON)`.
            if entity.get_player().is_some() {
                continue;
            }

            // Vanilla: max overlap per sub-box swept slab, not the union (fence/stair gaps).
            // Cap at `delta`.
            let entity_aabb = e.bounding_box.load();
            let mut overlap = 0.0f64;
            for shape in &shapes {
                let swept_shape = Self::movement_area(*shape, motion_dir, delta);
                if swept_shape.intersects(&entity_aabb) {
                    overlap = overlap.max(Self::intersection_size(
                        swept_shape,
                        motion_dir,
                        entity_aabb,
                    ));
                    if overlap >= delta {
                        break;
                    }
                }
            }
            if overlap <= 0.0 {
                continue;
            }
            let push_amount = overlap.min(delta) + 0.01;
            Self::move_entity(&entity, motion_dir, push_amount, motion_dir, game_time);

            // Vanilla `push`: retracting head also shoves out of the piston body cube.
            if !self.extending && self.source {
                Self::push_out_of_piston_body(
                    &entity,
                    &self.position,
                    motion_dir,
                    delta,
                    game_time,
                );
            }
        }
    }

    /// Vanilla `PistonMovingBlockEntity.moveStuckEntities`. Honey only
    /// (`isStickyForEntities`). Horizontal movement only.
    fn move_stuck_entities(&self, world: &Arc<World>, new_progress: f32) {
        if Block::from_state_id(self.pushed_block_state.id) != &Block::HONEY_BLOCK {
            return;
        }

        let motion_dir = self.movement_direction();
        if motion_dir.to_axis() == BlockAxis::Y {
            return;
        }

        let last = self.last_progress.load();
        let delta_progress = f64::from(new_progress - last);
        if delta_progress <= 0.0 {
            return;
        }

        // Vanilla: `movedState.getCollisionShape().max(Y)`, then local
        // `AABB(0, stickyTop, 0, 1, 1.500001, 1)` shifted by `getExtendedProgress`.
        let sticky_top = self
            .pushed_block_state
            .get_block_collision_shapes()
            .map(|shape| shape.max.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if !sticky_top.is_finite() {
            return;
        }
        let query = BoundingBox::new(
            Vector3::new(0.0, sticky_top, 0.0),
            Vector3::new(1.0, 1.500_001_000_000_000_1, 1.0),
        )
        .at_pos(self.position)
        .shift(Self::dir_vec(
            self.facing,
            f64::from(self.amount_extended(last)),
        ));

        let game_time = world.get_world_age();
        for entity in world.get_entities_at_box(&query) {
            let e = entity.get_entity();
            // Vanilla `matchesStickyCritera`: `PushReaction.NORMAL` (IGNORE is not).
            if e.ignores_piston_push() {
                continue;
            }
            // Player position is client-authoritative. Vanilla still `Entity.move(PISTON)`.
            if entity.get_player().is_some() {
                continue;
            }
            if !e.on_ground.load(Ordering::Relaxed) {
                continue;
            }
            // `isSupportedBy(pos)` or entity x/z inside the band, not the AABB.
            let pos = e.pos.load();
            let supported_here = e.supporting_block_pos.load() == Some(self.position);
            let within_footprint = pos.x >= query.min.x
                && pos.x <= query.max.x
                && pos.z >= query.min.z
                && pos.z <= query.max.z;
            if !supported_here && !within_footprint {
                continue;
            }

            Self::move_entity(&entity, motion_dir, delta_progress, motion_dir, game_time);
        }
    }

    /// Vanilla `getIntersectionSize`: how much `entity` overlaps `swept` along
    /// `motion_dir`. Positive means the entity is in the path of the moving block.
    fn intersection_size(
        swept: BoundingBox,
        motion_dir: BlockDirection,
        entity: BoundingBox,
    ) -> f64 {
        match motion_dir {
            BlockDirection::East => swept.max.x - entity.min.x,
            BlockDirection::West => entity.max.x - swept.min.x,
            BlockDirection::Up => swept.max.y - entity.min.y,
            BlockDirection::Down => entity.max.y - swept.min.y,
            BlockDirection::South => swept.max.z - entity.min.z,
            BlockDirection::North => entity.max.z - swept.min.z,
        }
    }

    /// Slime only. Not `PistonHandler::is_block_sticky` (honey drags, does not launch).
    /// Vanilla asks the pushed block, so a `source` BE never launches.
    fn launches_entities(pushed: &BlockState) -> bool {
        Block::from_state_id(pushed.id) == &Block::SLIME_BLOCK
    }

    /// Vanilla `movedState.getBlock() instanceof PistonBaseBlock`.
    fn is_piston_base(state: &BlockState) -> bool {
        PistonBlock::is_base(Block::from_state_id(state.id))
    }

    /// The piston head state this placeholder animates, with `short` as given. Used wherever
    /// vanilla substitutes a `PISTON_HEAD` for the stored `movedState`.
    fn head_state(&self, short: bool) -> &'static BlockState {
        let mut props = PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
        props.facing = self.facing.to_facing();
        props.short = short;
        props.r#type = PistonBlock::piston_type(Block::from_state_id(self.pushed_block_state.id));
        BlockState::from_id(props.to_state_id(&Block::PISTON_HEAD))
    }

    /// Vanilla `getCollisionRelatedBlockState`. Retracting `source` uses the head shape;
    /// the stored state is the piston body it replaced.
    fn collision_related_state(&self, progress: f32) -> &'static BlockState {
        if !self.extending && self.source && Self::is_piston_base(self.pushed_block_state) {
            self.head_state(progress > 0.25)
        } else {
            self.pushed_block_state
        }
    }

    /// `state` boxes at this cell, shifted by the current animation offset. Individual
    /// boxes, not the union (vanilla `MovingPistonBlock` collision query).
    fn shifted_shapes(&self, state: &'static BlockState, progress: f32) -> Vec<BoundingBox> {
        let offset = Self::dir_vec(self.facing, f64::from(self.amount_extended(progress)));
        state
            .get_block_collision_shapes_at(&self.position)
            .map(|shape| shape.at_pos(self.position).shift(offset))
            .collect()
    }

    /// The moving boxes this placeholder pushes entities with at a given animation progress.
    fn animated_shapes(&self, progress: f32) -> Vec<BoundingBox> {
        self.shifted_shapes(self.collision_related_state(progress), progress)
    }

    /// Retracting `source`: stationary piston body. Vanilla `pistonHeadShape` in
    /// `getCollisionShape` (the base, not the head).
    fn stationary_base_shapes(&self) -> Vec<BoundingBox> {
        if !(!self.extending && self.source && Self::is_piston_base(self.pushed_block_state)) {
            return Vec::new();
        }

        let block = Block::from_state_id(self.pushed_block_state.id);
        let mut props = PistonProps::from_state_id(self.pushed_block_state.id, block);
        props.extended = true;
        BlockState::from_id(props.to_state_id(block))
            .get_block_collision_shapes_at(&self.position)
            .map(|shape| shape.at_pos(self.position))
            .collect()
    }

    /// Vanilla `PistonMovingBlockEntity.getCollisionShape`. `noclip` matching this cell's
    /// motion reports only the stationary part (`Entity::piston_noclip`).
    pub fn collision_shapes(&self, noclip: Option<BlockDirection>) -> Vec<BoundingBox> {
        let progress = self.current_progress.load();
        let mut shapes = self.stationary_base_shapes();

        if progress < 1.0 && noclip == Some(self.movement_direction()) {
            return shapes;
        }

        // `source` always animates the head. Vanilla `short`: `extending != (1.0 - progress < 0.25)`.
        let state = if self.source {
            self.head_state(self.extending != (1.0 - progress < 0.25))
        } else {
            self.pushed_block_state
        };
        shapes.extend(self.shifted_shapes(state, progress));
        shapes
    }

    /// Collision-clipped move, not a teleport. `move_entity_external`: do not write into
    /// `velocity` (a minecart would re-integrate it).
    fn move_entity(
        entity: &Arc<dyn crate::entity::EntityBase>,
        dir: BlockDirection,
        distance: f64,
        piston_direction: BlockDirection,
        game_time: i64,
    ) {
        let e = entity.get_entity();
        e.move_entity_piston(
            entity,
            Self::dir_vec(dir, distance),
            piston_direction,
            game_time,
        );
        e.send_pos();
    }

    /// Vanilla `push`: when a piston head retracts, shove entities that ended up
    /// inside the piston-body cube back out the opposite direction (slightly past
    /// the move they just got, so the net motion is essentially zero).
    fn push_out_of_piston_body(
        entity: &Arc<dyn crate::entity::EntityBase>,
        piston_pos: &BlockPos,
        motion_dir: BlockDirection,
        amount: f64,
        game_time: i64,
    ) {
        let body_aabb = BoundingBox::from_block(piston_pos);
        let entity_aabb = entity.get_entity().bounding_box.load();
        if !body_aabb.intersects(&entity_aabb) {
            return;
        }
        let back = motion_dir.opposite();
        let e = Self::intersection_size(body_aabb, back, entity_aabb) + 0.01;
        let f = Self::intersection_size(
            body_aabb,
            back,
            Self::aabb_intersection(body_aabb, entity_aabb),
        ) + 0.01;
        if (e - f).abs() < 0.01 {
            let distance = e.min(amount) + 0.01;
            // Vanilla `moveEntityByPiston(direction, entity, delta, opposite)`: noclip stays
            // `motion_dir` so the entity can leave the retracting head's cell.
            Self::move_entity(entity, back, distance, motion_dir, game_time);
        }
    }

    const fn aabb_intersection(a: BoundingBox, b: BoundingBox) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(
                a.min.x.max(b.min.x),
                a.min.y.max(b.min.y),
                a.min.z.max(b.min.z),
            ),
            Vector3::new(
                a.max.x.min(b.max.x),
                a.max.y.min(b.max.y),
                a.max.z.min(b.max.z),
            ),
        )
    }

    pub fn finish(&self, world: &Arc<World>) {
        if self.last_progress.load() < 1.0 {
            // Vanilla parks the animation at its end before doing anything else, so a second
            // `finish` on the same instance is a no-op.
            self.current_progress.store(1.0);
            self.last_progress.store(1.0);

            let pos = self.position;
            if !self.is_current(world) {
                return;
            }
            // Vanilla `finalTick`: drop the BE first, then `setBlock` if the cell is
            // still `MOVING_PISTON`. Neighbour callbacks must not see the placeholder.
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                let state = if self.source {
                    Block::AIR.default_state.id
                } else {
                    // Vanilla `finalTick` does not strip `waterlogged`; `tick` does.
                    world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos)
                };
                world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL);
                world.update_neighbor(&pos, Block::from_state_id(state));
                Self::queue_delivered_block(world, pos);
            }
        }
    }

    /// Vanilla `ChunkHolder.broadcastChanges` sends the live cell. Re-queue after
    /// neighbour callbacks: the delivered block, even if `set_block_state` was a no-op.
    fn queue_delivered_block(world: &World, pos: BlockPos) {
        let live = world.get_block_state_id(&pos);
        if Block::from_state_id(live) != &Block::MOVING_PISTON {
            world.defer_block_change(pos, live);
        }
    }

    /// Vanilla `PistonMovingBlockEntity.saveAdditional`. Client codec names and types:
    /// `blockState` as `{Name, Properties}`, `facing` as byte (`to_index`), `progress` is
    /// `progressO` (last tick) so the client interpolates from there.
    fn write_fields(&self, nbt: &mut NbtCompound) {
        nbt.put_compound(BLOCK_STATE, block_state_to_nbt(self.pushed_block_state.id));
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
    }
}

const BLOCK_STATE: &str = "blockState";
const FACING: &str = "facing";
const LAST_PROGRESS: &str = "progress";
const EXTENDING: &str = "extending";
const SOURCE: &str = "source";

impl BlockEntity for PistonBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    /// Placeholders place their block when the animation ends. Tick in vanilla creation order.
    fn is_tick_order_sensitive(&self) -> bool {
        true
    }

    fn tick(&self, world: &Arc<World>) {
        // Superseded by a re-trigger: further `finish` would clobber the new animation.
        if !self.is_current(world) {
            return;
        }

        self.last_ticked.store(world.get_world_age());

        let current_progress = self.current_progress.load();
        self.last_progress.store(current_progress);
        if current_progress >= 1.0 {
            let pos = self.position;
            // Vanilla `PistonMovingBlockEntity.tick`: `removeBlockEntity` first, then
            // `setBlock` if the cell is still `MOVING_PISTON`.
            if !self.is_current(world) {
                return;
            }
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                // Vanilla uses the post-processed state. Unsurvivable (rail, torch) becomes
                // air: place then break so it drops.
                let updated_state =
                    world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos);

                if BlockState::from_id(updated_state).is_air() {
                    world.set_block_state(
                        &pos,
                        self.pushed_block_state.id,
                        BlockFlags::FORCE_STATE | BlockFlags::MOVED,
                    );
                    // No-op when the pushed block was air to begin with.
                    world.break_block(&pos, None, BlockFlags::NOTIFY_ALL);
                } else {
                    let updated_state = Block::from_state_id(updated_state)
                        .without_waterlogged(updated_state)
                        .map_or(updated_state, |state| state.id);
                    world.set_block_state(
                        &pos,
                        updated_state,
                        BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                    );
                    // Vanilla `updateNeighbor(pos, block, pos)`: the delivered block
                    // re-examines its support. Neighbours already got `NOTIFY_NEIGHBORS`.
                    world.update_neighbor(&pos, Block::from_state_id(updated_state));
                }
                Self::queue_delivered_block(world, pos);
            }
            return;
        }
        let new_progress = (current_progress + 0.5).min(1.0);
        self.push_entities(world, new_progress);
        self.move_stuck_entities(world, new_progress);
        self.current_progress.store(new_progress);
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        // Vanilla `DEFAULT_BLOCK_STATE` (air) for a missing or unknown `blockState` tag.
        // Pumpkin has no data-fixers. The placeholder still animates and delivers air.
        let pushed_block_state = nbt
            .get_compound(BLOCK_STATE)
            .and_then(block_state_from_nbt)
            .map_or(Block::AIR.default_state, BlockStateId::to_state);
        let facing = nbt.get_byte(FACING).unwrap_or(0);
        let last_progress = nbt.get_float(LAST_PROGRESS).unwrap_or(0.0);
        let extending = nbt.get_bool(EXTENDING).unwrap_or(false);
        let source = nbt.get_bool(SOURCE).unwrap_or(false);
        Self {
            pushed_block_state,
            position,
            facing: BlockDirection::from_index(facing as u8).unwrap_or(BlockDirection::Down),
            current_progress: last_progress.into(),
            last_progress: last_progress.into(),
            extending,
            source,
            last_ticked: (-1i64).into(),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_fields(nbt);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_fields(&mut nbt);
        Some(nbt)
    }

    fn sends_update_packet(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

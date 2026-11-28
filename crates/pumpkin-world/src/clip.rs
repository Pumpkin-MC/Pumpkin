use pumpkin_data::block_properties::ScaffoldingLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use crate::world::BlockAccessor;

/// Block shape selection mode for [`ClipContext`], matching vanilla `ClipContext.Block`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipBlock {
    /// Tests physical collision shapes (`BlockState::collision_shapes`).
    Collider,
    /// Tests outline selection shapes (`BlockState::outline_shapes`).
    Outline,
    /// Tests vanilla's visual occlusion shapes, which can differ from physical collision.
    Visual,
    /// Tests full block shape for blocks in `#minecraft:fall_damage_resetting`.
    FallDamageResetting,
}

/// Fluid selection mode for [`ClipContext`], matching vanilla `ClipContext.Fluid`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipFluid {
    /// Ignores all fluids.
    None,
    /// Hits source fluid blocks and waterlogged blocks.
    SourceOnly,
    /// Hits any non-empty fluid.
    Any,
    /// Hits water (source or flowing) and waterlogged blocks.
    Water,
}

/// Context for raycasting / line segment clipping against the world, matching vanilla `ClipContext`.
#[derive(Clone, Debug)]
pub struct ClipContext {
    pub from: Vector3<f64>,
    pub to: Vector3<f64>,
    pub block: ClipBlock,
    pub fluid: ClipFluid,
    visual_viewer: Option<VisualViewer>,
}

#[derive(Clone, Copy, Debug)]
struct VisualViewer {
    bottom: f64,
    descending: bool,
}

impl VisualViewer {
    fn is_above(self, height: f64) -> bool {
        self.bottom > height - f64::from(1.0e-5f32)
    }
}

impl ClipContext {
    #[must_use]
    pub const fn new(
        from: Vector3<f64>,
        to: Vector3<f64>,
        block: ClipBlock,
        fluid: ClipFluid,
    ) -> Self {
        Self {
            from,
            to,
            block,
            fluid,
            visual_viewer: None,
        }
    }

    /// Applies viewer-dependent visual shapes using the viewer's feet and descent input.
    #[must_use]
    pub const fn with_visual_viewer(mut self, bottom: f64, descending: bool) -> Self {
        self.visual_viewer = Some(VisualViewer { bottom, descending });
        self
    }
}

/// Result of clipping a ray against blocks and fluids in the world, matching vanilla `BlockHitResult`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockHitResult {
    /// The exact world coordinate of the hit, or the end position if the ray missed.
    pub location: Vector3<f64>,
    /// The block face that was hit, or the nearest face to the incoming ray if missed.
    pub direction: BlockDirection,
    /// The coordinates of the block that was hit, or the block containing the end position if missed.
    pub block_pos: BlockPos,
    /// True if no block or fluid was hit along the ray.
    pub is_miss: bool,
    /// True if the ray began inside the shape.
    pub is_inside: bool,
}

impl BlockHitResult {
    #[must_use]
    pub const fn hit(
        location: Vector3<f64>,
        direction: BlockDirection,
        block_pos: BlockPos,
        is_inside: bool,
    ) -> Self {
        Self {
            location,
            direction,
            block_pos,
            is_miss: false,
            is_inside,
        }
    }

    #[must_use]
    pub const fn miss(
        location: Vector3<f64>,
        direction: BlockDirection,
        block_pos: BlockPos,
    ) -> Self {
        Self {
            location,
            direction,
            block_pos,
            is_miss: true,
            is_inside: false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_miss(&self) -> bool {
        self.is_miss
    }

    #[inline]
    #[must_use]
    pub const fn is_hit(&self) -> bool {
        !self.is_miss
    }

    /// Returns the cursor position within the block `[0.0, 1.0]`, matching vanilla.
    #[must_use]
    pub fn cursor_pos(&self) -> Vector3<f32> {
        Vector3::new(
            (self.location.x - f64::from(self.block_pos.0.x)) as f32,
            (self.location.y - f64::from(self.block_pos.0.y)) as f32,
            (self.location.z - f64::from(self.block_pos.0.z)) as f32,
        )
    }

    /// Vanilla `withDirection`: returns a copy of this result with an overridden face direction.
    #[must_use]
    pub const fn with_direction(mut self, direction: BlockDirection) -> Self {
        self.direction = direction;
        self
    }
}

/// Intersects a ray segment from `from` to `to` with an AABB defined by `min` and `max`.
///
/// Returns `Some((t, hit_direction, hit_pos, is_inside))` where `t` is in `[0.0, 1.0]`,
/// or `None` if the ray does not intersect the box.
#[must_use]
pub fn intersects_aabb_with_hit(
    from: Vector3<f64>,
    to: Vector3<f64>,
    min: Vector3<f64>,
    max: Vector3<f64>,
) -> Option<(f64, BlockDirection, Vector3<f64>, bool)> {
    let dir = to.sub(&from);
    let mut tmin: f64 = 0.0;
    let mut tmax: f64 = 1.0;

    let mut hit_axis = None;
    let mut hit_is_min = false;

    let is_inside = from.x >= min.x
        && from.x <= max.x
        && from.y >= min.y
        && from.y <= max.y
        && from.z >= min.z
        && from.z <= max.z;

    macro_rules! check_axis {
        ($axis:ident, $dir_axis:ident, $min_axis:ident, $max_axis:ident) => {{
            if dir.$dir_axis.abs() < 1e-8 {
                if from.$dir_axis < min.$min_axis || from.$dir_axis > max.$max_axis {
                    return None;
                }
            } else {
                let inv_d = 1.0 / dir.$dir_axis;
                let t_near = (min.$min_axis - from.$dir_axis) * inv_d;
                let t_far = (max.$max_axis - from.$dir_axis) * inv_d;

                let (t_entry, t_exit, is_min_face) = if inv_d >= 0.0 {
                    (t_near, t_far, true)
                } else {
                    (t_far, t_near, false)
                };

                if t_entry > tmin {
                    tmin = t_entry;
                    hit_axis = Some(stringify!($axis));
                    hit_is_min = is_min_face;
                }
                tmax = tmax.min(t_exit);
                if tmax < tmin {
                    return None;
                }
            }
        }};
    }

    check_axis!(x, x, x, x);
    check_axis!(y, y, y, y);
    check_axis!(z, z, z, z);

    if tmax < 0.0 || tmin > 1.0 {
        return None;
    }

    let direction = if is_inside {
        BlockDirection::get_nearest(-dir.x, -dir.y, -dir.z)
    } else {
        match (hit_axis, hit_is_min) {
            (Some("x"), true) => BlockDirection::West,
            (Some("x"), false) => BlockDirection::East,
            (Some("y"), true) => BlockDirection::Down,
            (Some("y"), false) => BlockDirection::Up,
            (Some("z"), true) => BlockDirection::North,
            (Some("z"), false) => BlockDirection::South,
            _ => BlockDirection::get_nearest(-dir.x, -dir.y, -dir.z),
        }
    };

    let t_hit = tmin.max(0.0);
    let hit_pos = from + dir * t_hit;
    Some((t_hit, direction, hit_pos, is_inside))
}

fn clip_block_shapes(
    shapes: impl Iterator<Item = BoundingBox>,
    _state: &BlockState,
    block_pos: &BlockPos,
    from: Vector3<f64>,
    to: Vector3<f64>,
) -> Option<(f64, BlockDirection, Vector3<f64>, bool)> {
    let mut closest_hit: Option<(f64, BlockDirection, Vector3<f64>, bool)> = None;

    for shape in shapes {
        let world_min = shape.min.add(&block_pos.0.to_f64());
        let world_max = shape.max.add(&block_pos.0.to_f64());

        if let Some((t, dir, hit_pos, is_inside)) =
            intersects_aabb_with_hit(from, to, world_min, world_max)
            && closest_hit
                .as_ref()
                .is_none_or(|(closest_t, _, _, _)| t < *closest_t)
        {
            closest_hit = Some((t, dir, hit_pos, is_inside));
        }
    }

    closest_hit
}

fn clip_fluid(
    state: &BlockState,
    block_pos: &BlockPos,
    from: Vector3<f64>,
    to: Vector3<f64>,
    fluid_mode: ClipFluid,
) -> Option<(f64, BlockDirection, Vector3<f64>, bool)> {
    let should_clip = match fluid_mode {
        ClipFluid::None => false,
        ClipFluid::SourceOnly => {
            state.is_waterlogged()
                || Fluid::from_state_id(state.id)
                    .is_some_and(|f| f.id == Fluid::WATER.id || f.id == Fluid::LAVA.id)
        }
        ClipFluid::Any => {
            state.is_waterlogged()
                || Fluid::from_state_id(state.id).is_some_and(|f| f != &Fluid::EMPTY)
        }
        ClipFluid::Water => {
            state.is_waterlogged()
                || Fluid::from_state_id(state.id).is_some_and(|f| f.matches_type(&Fluid::WATER))
        }
    };

    if !should_clip {
        return None;
    }

    let world_min = block_pos.0.to_f64();
    let world_max = world_min.add_raw(1.0, 1.0, 1.0);
    intersects_aabb_with_hit(from, to, world_min, world_max)
}

fn clip_visual_shape(
    state: &BlockState,
    block_pos: &BlockPos,
    context: &ClipContext,
) -> Option<(f64, BlockDirection, Vector3<f64>, bool)> {
    let block_y = f64::from(block_pos.0.y);
    if let Some(viewer) = context.visual_viewer
        && state.id.to_block() == &Block::SCAFFOLDING
        && (viewer.descending || !viewer.is_above(block_y + 1.0))
    {
        // ScaffoldingBlock.getCollisionShape selects the bottom slab for this context.
        const SHAPE_UNSTABLE_BOTTOM: BoundingBox = BoundingBox {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(1.0, 0.125, 1.0),
        };
        let props = ScaffoldingLikeProperties::from_state_id(state.id);
        if props.distance != 0 && props.bottom && viewer.is_above(block_y) {
            return clip_block_shapes(
                std::iter::once(SHAPE_UNSTABLE_BOTTOM),
                state,
                block_pos,
                context.from,
                context.to,
            );
        }
        return None;
    }

    clip_block_shapes(
        state.get_block_visual_shapes_at(block_pos),
        state,
        block_pos,
        context.from,
        context.to,
    )
}

fn check_block(
    accessor: &(impl BlockAccessor + ?Sized),
    block_pos: &BlockPos,
    from: Vector3<f64>,
    to: Vector3<f64>,
    context: &ClipContext,
) -> Option<BlockHitResult> {
    let state = accessor.get_block_state(block_pos);

    let block_hit = match context.block {
        ClipBlock::Collider => clip_block_shapes(
            state.get_block_collision_shapes_at(block_pos),
            state,
            block_pos,
            from,
            to,
        ),
        ClipBlock::Outline => clip_block_shapes(
            state.get_block_outline_shapes_at(block_pos),
            state,
            block_pos,
            from,
            to,
        ),
        ClipBlock::Visual => clip_visual_shape(state, block_pos, context),
        ClipBlock::FallDamageResetting => {
            if Block::from_state_id(state.id)
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_FALL_DAMAGE_RESETTING)
            {
                let world_min = block_pos.0.to_f64();
                let world_max = world_min.add_raw(1.0, 1.0, 1.0);
                intersects_aabb_with_hit(from, to, world_min, world_max)
            } else {
                None
            }
        }
    };

    let fluid_hit = clip_fluid(state, block_pos, from, to, context.fluid);

    match (block_hit, fluid_hit) {
        (Some((bt, bdir, bpos, binside)), Some((ft, fdir, fpos, finside))) => {
            if bt <= ft {
                Some(BlockHitResult::hit(bpos, bdir, *block_pos, binside))
            } else {
                Some(BlockHitResult::hit(fpos, fdir, *block_pos, finside))
            }
        }
        (Some((_, bdir, bpos, binside)), None) => {
            Some(BlockHitResult::hit(bpos, bdir, *block_pos, binside))
        }
        (None, Some((_, fdir, fpos, finside))) => {
            Some(BlockHitResult::hit(fpos, fdir, *block_pos, finside))
        }
        (None, None) => None,
    }
}

/// Traces a ray segment through the block grid from `context.from` to `context.to`,
/// testing voxel shapes and fluid shapes according to `context`.
/// Matches vanilla `BlockGetter.clip(ClipContext)`.
pub fn clip(accessor: &(impl BlockAccessor + ?Sized), context: &ClipContext) -> BlockHitResult {
    let from = context.from;
    let to = context.to;
    traverse_blocks(from, to, |block| {
        check_block(accessor, block, from, to, context)
    })
    .unwrap_or_else(|| {
        let miss_vec = from.sub(&to);
        let miss_dir = if from == to {
            BlockDirection::Up
        } else {
            BlockDirection::get_nearest(miss_vec.x, miss_vec.y, miss_vec.z)
        };
        BlockHitResult::miss(to, miss_dir, BlockPos::floored_v(to))
    })
}

pub fn traverse_blocks<T>(
    from: Vector3<f64>,
    to: Vector3<f64>,
    mut visit: impl FnMut(&BlockPos) -> Option<T>,
) -> Option<T> {
    // A non-finite end never advances the traversal and would loop forever.
    if from == to
        || ![from.x, from.y, from.z, to.x, to.y, to.z]
            .iter()
            .all(|value| value.is_finite())
    {
        return None;
    }

    let adjust = -1.0e-7f64;
    let d_to = to.lerp(&from, adjust);
    let d_from = from.lerp(&to, adjust);

    let mut block = BlockPos::floored(d_from.x, d_from.y, d_from.z);

    if let Some(hit) = visit(&block) {
        return Some(hit);
    }

    let difference = d_to.sub(&d_from);
    let step = difference.sign();

    let delta = Vector3::new(
        if step.x == 0 {
            f64::MAX
        } else {
            f64::from(step.x) / difference.x
        },
        if step.y == 0 {
            f64::MAX
        } else {
            f64::from(step.y) / difference.y
        },
        if step.z == 0 {
            f64::MAX
        } else {
            f64::from(step.z) / difference.z
        },
    );

    let mut next = Vector3::new(
        delta.x
            * if step.x > 0 {
                1.0 - (d_from.x - d_from.x.floor())
            } else {
                d_from.x - d_from.x.floor()
            },
        delta.y
            * if step.y > 0 {
                1.0 - (d_from.y - d_from.y.floor())
            } else {
                d_from.y - d_from.y.floor()
            },
        delta.z
            * if step.z > 0 {
                1.0 - (d_from.z - d_from.z.floor())
            } else {
                d_from.z - d_from.z.floor()
            },
    );

    while next.x <= 1.0 || next.y <= 1.0 || next.z <= 1.0 {
        if next.x < next.y {
            if next.x < next.z {
                block.0.x += step.x;
                next.x += delta.x;
            } else {
                block.0.z += step.z;
                next.z += delta.z;
            }
        } else if next.y < next.z {
            block.0.y += step.y;
            next.y += delta.y;
        } else {
            block.0.z += step.z;
            next.z += delta.z;
        }

        if let Some(hit) = visit(&block) {
            return Some(hit);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::BlockStateId;
    use rustc_hash::FxHashMap;

    struct TestWorld {
        blocks: FxHashMap<BlockPos, BlockStateId>,
    }

    impl TestWorld {
        fn new() -> Self {
            Self {
                blocks: FxHashMap::default(),
            }
        }

        fn set_block(&mut self, pos: BlockPos, block: &'static Block) {
            self.blocks.insert(pos, block.default_state.id);
        }
    }

    impl BlockAccessor for TestWorld {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            let state_id = self.get_block_state_id(position);
            Block::from_state_id(state_id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.get_block_state_id(position).to_state()
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.blocks
                .get(position)
                .copied()
                .unwrap_or(Block::AIR.default_state.id)
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state_id = self.get_block_state_id(position);
            BlockState::from_id_with_block(state_id)
        }
    }

    #[test]
    fn ray_miss_in_air() {
        let world = TestWorld::new();
        let context = ClipContext::new(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(5.5, 0.5, 0.5),
            ClipBlock::Outline,
            ClipFluid::None,
        );
        let hit = world.clip(&context);
        assert!(hit.is_miss());
        assert!(!hit.is_hit());
        assert_eq!(hit.block_pos, BlockPos::new(5, 0, 0));
    }

    #[test]
    fn ray_hits_solid_block() {
        let mut world = TestWorld::new();
        world.set_block(BlockPos::new(3, 0, 0), &Block::STONE);

        let context = ClipContext::new(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(5.5, 0.5, 0.5),
            ClipBlock::Outline,
            ClipFluid::None,
        );
        let hit = world.clip(&context);
        assert!(hit.is_hit());
        assert!(!hit.is_miss());
        assert_eq!(hit.block_pos, BlockPos::new(3, 0, 0));
        assert_eq!(hit.direction, BlockDirection::West);
        assert!((hit.location.x - 3.0).abs() < 1e-5);
        assert!((hit.location.y - 0.5).abs() < 1e-5);
        assert!((hit.location.z - 0.5).abs() < 1e-5);
        assert!(!hit.is_inside);
    }

    #[test]
    fn ray_fluid_handling() {
        let mut world = TestWorld::new();
        world.set_block(BlockPos::new(3, 0, 0), &Block::WATER);

        // Fluid NONE should pass right through water
        let hit_none = world.clip(&ClipContext::new(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(5.5, 0.5, 0.5),
            ClipBlock::Outline,
            ClipFluid::None,
        ));
        assert!(hit_none.is_miss());

        // Fluid SOURCE_ONLY should hit water
        let hit_source = world.clip(&ClipContext::new(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(5.5, 0.5, 0.5),
            ClipBlock::Outline,
            ClipFluid::SourceOnly,
        ));
        assert!(hit_source.is_hit());
        assert_eq!(hit_source.block_pos, BlockPos::new(3, 0, 0));
        assert_eq!(hit_source.direction, BlockDirection::West);

        // Fluid ANY should hit water
        let hit_any = world.clip(&ClipContext::new(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(5.5, 0.5, 0.5),
            ClipBlock::Collider,
            ClipFluid::Any,
        ));
        assert!(hit_any.is_hit());
        assert_eq!(hit_any.block_pos, BlockPos::new(3, 0, 0));
    }

    #[test]
    fn ray_inside_block() {
        let mut world = TestWorld::new();
        world.set_block(BlockPos::new(0, 0, 0), &Block::STONE);

        let hit = world.clip(&ClipContext::new(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(0.5, 2.5, 0.5),
            ClipBlock::Collider,
            ClipFluid::None,
        ));
        assert!(hit.is_hit());
        assert!(hit.is_inside);
        assert_eq!(hit.block_pos, BlockPos::new(0, 0, 0));
    }

    #[test]
    fn visual_shapes_line_of_sight() {
        let mut world = TestWorld::new();
        let from = Vector3::new(0.5, 0.5, 0.5);
        let to = Vector3::new(5.5, 0.5, 0.5);

        // Clear line of sight in air
        let hit_air = world.clip(&ClipContext::new(
            from,
            to,
            ClipBlock::Visual,
            ClipFluid::None,
        ));
        assert!(hit_air.is_miss());

        // Blocked line of sight with stone
        world.set_block(BlockPos::new(2, 0, 0), &Block::STONE);
        let hit_stone = world.clip(&ClipContext::new(
            from,
            to,
            ClipBlock::Visual,
            ClipFluid::None,
        ));
        assert!(hit_stone.is_hit());
        assert_eq!(hit_stone.block_pos, BlockPos::new(2, 0, 0));

        world.set_block(BlockPos::new(2, 0, 0), &Block::GLASS);
        let visual_glass = world.clip(&ClipContext::new(
            from,
            to,
            ClipBlock::Visual,
            ClipFluid::None,
        ));
        let collider_glass = world.clip(&ClipContext::new(
            from,
            to,
            ClipBlock::Collider,
            ClipFluid::None,
        ));
        assert!(visual_glass.is_miss());
        assert_eq!(collider_glass.block_pos, BlockPos::new(2, 0, 0));
        assert!(collider_glass.is_hit());
    }

    #[test]
    fn scaffolding_visual_shape_uses_viewer_context() {
        let mut world = TestWorld::new();
        let position = BlockPos::new(2, 10, 0);
        world.set_block(position, &Block::SCAFFOLDING);
        let across = |y| {
            ClipContext::new(
                Vector3::new(0.5, y, 0.5),
                Vector3::new(5.5, y, 0.5),
                ClipBlock::Visual,
                ClipFluid::None,
            )
        };

        let top = 10.9375;
        assert!(world.clip(&across(top)).is_hit());
        assert!(
            world
                .clip(&across(top).with_visual_viewer(10.0, false))
                .is_miss()
        );
        assert!(
            world
                .clip(&across(top).with_visual_viewer(11.0, false))
                .is_hit()
        );
        assert!(
            world
                .clip(&across(top).with_visual_viewer(11.0, true))
                .is_miss()
        );

        let mut props = ScaffoldingLikeProperties::default(&Block::SCAFFOLDING);
        props.distance = 1;
        props.bottom = true;
        world
            .blocks
            .insert(position, props.to_state_id(&Block::SCAFFOLDING));
        let bottom = 10.0625;
        assert!(
            world
                .clip(&across(bottom).with_visual_viewer(10.0, false))
                .is_hit()
        );
        assert!(
            world
                .clip(&across(bottom).with_visual_viewer(9.5, false))
                .is_miss()
        );
    }
}

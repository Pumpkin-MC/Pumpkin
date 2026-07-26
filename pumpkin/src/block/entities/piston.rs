use std::sync::atomic::Ordering;
use std::{pin::Pin, sync::Arc};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::{
    BlockProperties, PistonHeadLikeProperties, PistonType, StickyPistonLikeProperties,
};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

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
}

impl PistonBlockEntity {
    pub const ID: &'static str = "minecraft:piston";

    /// Vanilla `PistonMovingBlockEntity.PUSH_OFFSET` (`PistonMovingBlockEntity.java:41`).
    const PUSH_OFFSET: f64 = 0.01;

    const fn movement_direction(&self) -> BlockDirection {
        if self.extending {
            self.facing
        } else {
            self.facing.opposite()
        }
    }

    /// Vanilla's `getExtendedProgress` (`PistonMovingBlockEntity.java:105-107`):
    /// how far back from the block's final position the visual is at a given
    /// animation progress. Negative for extending.
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

    /// Union of two AABBs, vanilla `AABB.minmax`.
    fn min_max(a: BoundingBox, b: BoundingBox) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(
                a.min.x.min(b.min.x),
                a.min.y.min(b.min.y),
                a.min.z.min(b.min.z),
            ),
            Vector3::new(
                a.max.x.max(b.max.x),
                a.max.y.max(b.max.y),
                a.max.z.max(b.max.z),
            ),
        )
    }

    /// Vanilla `PistonMath.getMovementArea` (`PistonMath.java:10-33`): the slab
    /// swept by the leading face of `aabb` when it moves `amount` along `dir`.
    fn movement_area(aabb: BoundingBox, dir: BlockDirection, amount: f64) -> BoundingBox {
        let off = dir.to_offset();
        let step = f64::from(off.x + off.y + off.z);
        let delta = amount * step;
        let min = delta.min(0.0);
        let max = delta.max(0.0);
        match dir {
            BlockDirection::West => BoundingBox::new(
                Vector3::new(aabb.min.x + min, aabb.min.y, aabb.min.z),
                Vector3::new(aabb.min.x + max, aabb.max.y, aabb.max.z),
            ),
            BlockDirection::East => BoundingBox::new(
                Vector3::new(aabb.max.x + min, aabb.min.y, aabb.min.z),
                Vector3::new(aabb.max.x + max, aabb.max.y, aabb.max.z),
            ),
            BlockDirection::Down => BoundingBox::new(
                Vector3::new(aabb.min.x, aabb.min.y + min, aabb.min.z),
                Vector3::new(aabb.max.x, aabb.min.y + max, aabb.max.z),
            ),
            BlockDirection::Up => BoundingBox::new(
                Vector3::new(aabb.min.x, aabb.max.y + min, aabb.min.z),
                Vector3::new(aabb.max.x, aabb.max.y + max, aabb.max.z),
            ),
            BlockDirection::North => BoundingBox::new(
                Vector3::new(aabb.min.x, aabb.min.y, aabb.min.z + min),
                Vector3::new(aabb.max.x, aabb.max.y, aabb.min.z + max),
            ),
            BlockDirection::South => BoundingBox::new(
                Vector3::new(aabb.min.x, aabb.min.y, aabb.max.z + min),
                Vector3::new(aabb.max.x, aabb.max.y, aabb.max.z + max),
            ),
        }
    }

    /// Vanilla `moveByPositionAndProgress` (`PistonMovingBlockEntity.java:228-231`):
    /// a shape-local AABB placed at the block's current animated position.
    fn move_by_position_and_progress(&self, aabb: BoundingBox, progress: f32) -> BoundingBox {
        aabb.at_pos(self.position).shift(Self::dir_vec(
            self.facing,
            f64::from(self.amount_extended(progress)),
        ))
    }

    /// Vanilla `getCollisionRelatedBlockState` (`PistonMovingBlockEntity.java:109-114`):
    /// a retracting source piston pushes entities with the piston head's shape
    /// (`SHORT` once progress passes 0.25), everything else with the moved state.
    fn collision_related_block_state(&self, progress: f32) -> &'static BlockState {
        let block = Block::from_state_id(self.pushed_block_state.id);
        if !self.extending
            && self.source
            && (block == &Block::PISTON || block == &Block::STICKY_PISTON)
        {
            let mut head = PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
            head.r#short = progress > 0.25;
            head.r#type = if block == &Block::STICKY_PISTON {
                PistonType::Sticky
            } else {
                PistonType::Normal
            };
            head.facing =
                StickyPistonLikeProperties::from_state_id(self.pushed_block_state.id, block).facing;
            BlockState::from_id(head.to_state_id(&Block::PISTON_HEAD))
        } else {
            self.pushed_block_state
        }
    }

    /// World-space collision boxes of the moving block, used by
    /// `World::get_block_collisions` so entities collide with the in-flight block.
    ///
    /// Port of vanilla `PistonMovingBlockEntity.getCollisionShape`
    /// (`PistonMovingBlockEntity.java:325-337`): the shape of the moved state (or
    /// the piston head for a source piston) offset by the progress-interpolated
    /// position, unioned with the piston base body while a source piston retracts.
    ///
    /// Vanilla's `NOCLIP` thread-local (`java:327-330`) suppresses this shape while
    /// the piston itself moves an entity via `Entity.move(PISTON, ...)`; Pumpkin's
    /// push writes positions directly with `set_pos` and never resolves block
    /// collisions during the push, so no equivalent is needed here.
    pub fn collision_shapes(&self) -> Vec<BoundingBox> {
        let progress = self.current_progress.load();
        let block = Block::from_state_id(self.pushed_block_state.id);
        let mut shapes = Vec::new();

        // PistonMovingBlockEntity.java:326: while a source piston retracts, the
        // piston base (with EXTENDED=true, i.e. the 12-pixel body) still occupies
        // the block space.
        if !self.extending
            && self.source
            && (block == &Block::PISTON || block == &Block::STICKY_PISTON)
        {
            let mut base =
                StickyPistonLikeProperties::from_state_id(self.pushed_block_state.id, block);
            base.extended = true;
            let base_state = BlockState::from_id(base.to_state_id(block));
            shapes.extend(
                base_state
                    .get_block_collision_shapes()
                    .map(|s| s.at_pos(self.position)),
            );
        }

        // PistonMovingBlockEntity.java:331: a source piston moves as a piston
        // head (SHORT while more than a quarter block away from rest), anything
        // else moves as the pushed state.
        let moving_state = if self.source {
            let mut head = PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
            head.facing = self.facing.to_facing();
            head.r#short = self.extending != ((1.0 - progress) < 0.25);
            BlockState::from_id(head.to_state_id(&Block::PISTON_HEAD))
        } else {
            self.pushed_block_state
        };

        // PistonMovingBlockEntity.java:332-336: offset by the animated position.
        let offset = Self::dir_vec(self.facing, f64::from(self.amount_extended(progress)));
        shapes.extend(
            moving_state
                .get_block_collision_shapes()
                .map(|s| s.at_pos(self.position).shift(offset)),
        );
        shapes
    }

    /// Ports vanilla `PistonMovingBlockEntity.moveCollidedEntities`
    /// (`PistonMovingBlockEntity.java:116-166`): pushes every entity in the path
    /// of the moving block by its overlap with the leading edge, capped at this
    /// tick's progress delta plus `PUSH_OFFSET` (the 0.51 leading-edge push),
    /// and launches entities on a moving slime block.
    fn move_collided_entities(&self, world: &Arc<World>, new_progress: f32) {
        let progress = self.current_progress.load();
        let movement = self.movement_direction();
        let delta_progress = f64::from(new_progress - progress);
        if delta_progress <= 0.0 {
            return;
        }

        let shapes: Vec<BoundingBox> = self
            .collision_related_block_state(progress)
            .get_block_collision_shapes()
            .collect();
        let Some(first) = shapes.first() else {
            // PistonMovingBlockEntity.java:120-122: nothing to push with.
            return;
        };

        // PistonMovingBlockEntity.java:123-124: search the union of the block's
        // current animated bounds and the area swept by its leading face.
        let bounds = shapes
            .iter()
            .skip(1)
            .fold(*first, |acc, aabb| Self::min_max(acc, *aabb));
        let aabb = self.move_by_position_and_progress(bounds, progress);
        let search = Self::min_max(Self::movement_area(aabb, movement, delta_progress), aabb);
        let entities = world.get_all_at_box(&search);
        if entities.is_empty() {
            return;
        }

        // PistonMovingBlockEntity.java:129: the launch applies when the moved
        // block itself is slime.
        let cause_bounce = Block::from_state_id(self.pushed_block_state.id) == &Block::SLIME_BLOCK;

        for entity in entities {
            let e = entity.get_entity();
            // PistonMovingBlockEntity.java:134: PushReaction.IGNORE — spectators
            // (and Pumpkin's no-clip marker entities) are never pushed.
            if entity.is_spectator() || e.no_clip.load(Ordering::Relaxed) {
                continue;
            }
            let is_player = entity.get_player().is_some();

            if cause_bounce {
                // PistonMovingBlockEntity.java:135-155: slime launch — the
                // velocity component along the push axis is set to the movement
                // step. Players are skipped entirely (java:136); their client
                // simulates both launch and push locally.
                if is_player {
                    continue;
                }
                let step = movement.to_offset();
                let mut velocity = e.velocity.load();
                if step.x != 0 {
                    velocity.x = f64::from(step.x);
                } else if step.y != 0 {
                    velocity.y = f64::from(step.y);
                } else {
                    velocity.z = f64::from(step.z);
                }
                e.velocity.store(velocity);
                e.send_velocity();
            }

            // PistonMovingBlockEntity.java:156-159: the push distance is the
            // largest overlap between the entity and any leading-edge slab of
            // the shape, early-exiting once a full tick of movement is reached.
            let entity_aabb = e.bounding_box.load();
            let mut delta = 0.0f64;
            for shape in &shapes {
                let moving_aabb = Self::movement_area(
                    self.move_by_position_and_progress(*shape, progress),
                    movement,
                    delta_progress,
                );
                if moving_aabb.intersects(&entity_aabb) {
                    delta = delta.max(Self::get_movement(moving_aabb, movement, entity_aabb));
                    if delta >= delta_progress {
                        break;
                    }
                }
            }
            if delta <= 0.0 {
                continue;
            }
            // PistonMovingBlockEntity.java:161: min(delta, deltaProgress) + 0.01.
            delta = delta.min(delta_progress) + Self::PUSH_OFFSET;
            Self::move_entity_by_piston(e, movement, delta, is_player);

            // PistonMovingBlockEntity.java:163-164.
            if !self.extending && self.source {
                Self::fix_entity_within_piston_base(
                    e,
                    &self.position,
                    movement,
                    delta_progress,
                    is_player,
                );
            }
        }
    }

    /// Ports vanilla `moveStuckEntities` (`PistonMovingBlockEntity.java:177-196`):
    /// entities standing on a horizontally moving honey block travel with it.
    fn move_stuck_entities(&self, world: &Arc<World>, new_progress: f32) {
        // isStickyForEntities, PistonMovingBlockEntity.java:198-200.
        if Block::from_state_id(self.pushed_block_state.id) != &Block::HONEY_BLOCK {
            return;
        }
        let movement = self.movement_direction();
        // PistonMovingBlockEntity.java:182-184: only horizontal movement drags.
        if matches!(movement, BlockDirection::Up | BlockDirection::Down) {
            return;
        }
        let progress = self.current_progress.load();
        let delta_progress = f64::from(new_progress - progress);
        if delta_progress <= 0.0 {
            return;
        }

        // PistonMovingBlockEntity.java:185: top of the moved block's collision shape.
        let sticky_top = self
            .pushed_block_state
            .get_block_collision_shapes()
            .map(|s| s.max.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if !sticky_top.is_finite() {
            return;
        }
        // PistonMovingBlockEntity.java:186: the vanilla constant 1.5000010000000001.
        let aabb = self.move_by_position_and_progress(
            BoundingBox::new(
                Vector3::new(0.0, sticky_top, 0.0),
                Vector3::new(1.0, 1.500_001_000_000_000_1, 1.0),
            ),
            progress,
        );

        for entity in world.get_all_at_box(&aabb) {
            let e = entity.get_entity();
            // matchesStickyCritera, PistonMovingBlockEntity.java:194-196:
            // PushReaction.NORMAL, on ground, and either supported by this block
            // or centered above the sticky surface.
            if entity.is_spectator() || e.no_clip.load(Ordering::Relaxed) {
                continue;
            }
            if !e.on_ground.load(Ordering::Relaxed) {
                continue;
            }
            let pos = e.pos.load();
            let supported = e.get_supporting_block_pos() == Some(self.position);
            let centered_on_top = pos.x >= aabb.min.x
                && pos.x <= aabb.max.x
                && pos.z >= aabb.min.z
                && pos.z <= aabb.max.z;
            if !(supported || centered_on_top) {
                continue;
            }
            // PistonMovingBlockEntity.java:189-191: stuck entities move exactly
            // one progress delta, without the push offset.
            let is_player = entity.get_player().is_some();
            Self::move_entity_by_piston(e, movement, delta_progress, is_player);
        }
    }

    /// Vanilla `getMovement` (`PistonMovingBlockEntity.java:206-226`): how much
    /// `entity` overlaps `aabb` along `movement`. Positive means the entity is in
    /// the path of the moving block.
    fn get_movement(aabb: BoundingBox, movement: BlockDirection, entity: BoundingBox) -> f64 {
        match movement {
            BlockDirection::East => aabb.max.x - entity.min.x,
            BlockDirection::West => entity.max.x - aabb.min.x,
            BlockDirection::Up => aabb.max.y - entity.min.y,
            BlockDirection::Down => entity.max.y - aabb.min.y,
            BlockDirection::South => aabb.max.z - entity.min.z,
            BlockDirection::North => entity.max.z - aabb.min.z,
        }
    }

    /// Vanilla `moveEntityByPiston` (`PistonMovingBlockEntity.java:168-175`).
    ///
    /// Non-players get their server position moved and broadcast. For players
    /// only the server-side position is updated: the vanilla client ticks the
    /// moving piston block entity itself and applies the identical push locally,
    /// and player movement is otherwise client-authoritative, so echoing a move
    /// packet back would fight the client's own movement stream.
    fn move_entity_by_piston(
        entity: &crate::entity::Entity,
        dir: BlockDirection,
        distance: f64,
        is_player: bool,
    ) {
        let new_pos = entity.pos.load() + Self::dir_vec(dir, distance);
        entity.set_pos(new_pos);
        if !is_player {
            entity.send_pos();
        }
    }

    /// Vanilla `fixEntityWithinPistonBase` (`PistonMovingBlockEntity.java:233-243`):
    /// when a piston head retracts, shove entities that ended up inside the
    /// piston-body cube back out the opposite direction (slightly past the move
    /// they just got, so the net motion is essentially zero).
    fn fix_entity_within_piston_base(
        entity: &crate::entity::Entity,
        piston_pos: &BlockPos,
        movement: BlockDirection,
        delta_progress: f64,
        is_player: bool,
    ) {
        let body_aabb = BoundingBox::from_block(piston_pos);
        let entity_aabb = entity.bounding_box.load();
        if !body_aabb.intersects(&entity_aabb) {
            return;
        }
        let back = movement.opposite();
        let delta = Self::get_movement(body_aabb, back, entity_aabb) + Self::PUSH_OFFSET;
        let delta_intersected = Self::get_movement(
            body_aabb,
            back,
            Self::aabb_intersection(body_aabb, entity_aabb),
        ) + Self::PUSH_OFFSET;
        if (delta - delta_intersected).abs() < 0.01 {
            let distance = delta.min(delta_progress) + Self::PUSH_OFFSET;
            Self::move_entity_by_piston(entity, back, distance, is_player);
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

    pub async fn finish(&self, world: Arc<World>) {
        if self.last_progress.load() < 1.0 {
            let pos = self.position;
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                let state = if self.source {
                    Block::AIR.default_state.id
                } else {
                    world
                        .clone()
                        .update_from_neighbor_shapes(self.pushed_block_state.id, &pos)
                        .await
                };
                world
                    .clone()
                    .set_block_state(&pos, state, BlockFlags::NOTIFY_ALL)
                    .await;
                world
                    .update_neighbor(&pos, Block::from_state_id(state))
                    .await;
            }
        }
    }
}

const FACING: &str = "facing";
const LAST_PROGRESS: &str = "progress";
const EXTENDING: &str = "extending";
const SOURCE: &str = "source";
/// Pumpkin-native key: global block state palette id of the pushed block.
const BLOCK_STATE_ID: &str = "blockStateId";

fn write_piston_fields(entity: &PistonBlockEntity, nbt: &mut NbtCompound) {
    nbt.put_byte(FACING, entity.facing.to_index() as i8);
    nbt.put_float(LAST_PROGRESS, entity.last_progress.load());
    nbt.put_bool(EXTENDING, entity.extending);
    nbt.put_bool(SOURCE, entity.source);
    nbt.put_int(
        BLOCK_STATE_ID,
        i32::from(entity.pushed_block_state.id.as_u16()),
    );
    write_block_state(entity.pushed_block_state, nbt);
}

/// Writes the vanilla block-state codec payload used by piston block-entity updates.
///
/// The client renders a moving piston from `blockState`; the native palette id is
/// useful for Pumpkin saves but is unknown to an unmodified Java client.
fn write_block_state(state: &BlockState, nbt: &mut NbtCompound) {
    let block = Block::from_state_id(state.id);
    let mut block_state = NbtCompound::new();
    block_state.put_string("Name", format!("minecraft:{}", block.name));

    if let Some(properties) = block.properties(state.id) {
        let mut state_properties = NbtCompound::new();
        for (name, value) in properties.to_props() {
            state_properties.put_string(name, value.to_string());
        }
        if !state_properties.is_empty() {
            block_state.put_compound("Properties", state_properties);
        }
    }

    nbt.put_compound("blockState", block_state);
}

fn read_pushed_block_state(nbt: &NbtCompound) -> &'static BlockState {
    if let Some(state) = read_vanilla_block_state(nbt) {
        return state;
    }

    if let Some(id) = nbt.get_int(BLOCK_STATE_ID)
        && let Ok(id) = u16::try_from(id)
        && let Some(state_id) = pumpkin_data::BlockStateId::new(id)
    {
        return BlockState::from_id(state_id);
    }
    Block::AIR.default_state
}

/// Decodes the vanilla `BlockState.CODEC` payload used by moving piston block entities.
///
/// `Properties` and each individual property are optional in the vanilla codec, so any
/// omitted values inherit from the block's default state. Matching existing states instead
/// of feeding arbitrary saved NBT into generated property parsers also keeps malformed or
/// newer-world data from panicking the server.
fn read_vanilla_block_state(nbt: &NbtCompound) -> Option<&'static BlockState> {
    let block_state = nbt.get_compound("blockState")?;
    let name = block_state.get_string("Name")?;
    let block = Block::from_name(name.strip_prefix("minecraft:").unwrap_or(name))?;
    let Some(properties) = block_state.get_compound("Properties") else {
        return Some(block.default_state);
    };
    let Some(default_properties) = block.properties(block.default_state.id) else {
        return Some(block.default_state);
    };
    let default_properties = default_properties.to_props();

    block
        .states
        .iter()
        .find(|state| {
            block.properties(state.id).is_some_and(|state_properties| {
                state_properties.to_props().iter().all(|(name, value)| {
                    properties.get_string(name).or_else(|| {
                        default_properties
                            .iter()
                            .find_map(|(default_name, default_value)| {
                                (*default_name == *name).then_some(*default_value)
                            })
                    }) == Some(*value)
                })
            })
        })
        .or(Some(block.default_state))
}

impl BlockEntity for PistonBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let current_progress = self.current_progress.load();
            self.last_progress.store(current_progress);
            if current_progress >= 1.0 {
                let pos = self.position;
                world.remove_block_entity(&pos);
                if world.get_block(&pos) == &Block::MOVING_PISTON {
                    if self.pushed_block_state.is_air() {
                        world
                            .clone()
                            .set_block_state(
                                &pos,
                                self.pushed_block_state.id,
                                BlockFlags::FORCE_STATE | BlockFlags::MOVED,
                            )
                            .await;
                    } else {
                        let updated_state = world
                            .clone()
                            .update_from_neighbor_shapes(self.pushed_block_state.id, &pos)
                            .await;
                        world
                            .clone()
                            .set_block_state(
                                &pos,
                                updated_state,
                                BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                            )
                            .await;
                        world
                            .clone()
                            .update_neighbor(&pos, Block::from_state_id(updated_state))
                            .await;
                    }
                }
                return;
            }
            // PistonMovingBlockEntity.java:296-302: entities are moved with the
            // unclamped new progress, which is clamped only when stored.
            let new_progress = current_progress + 0.5;
            self.move_collided_entities(world, new_progress);
            self.move_stuck_entities(world, new_progress);
            self.current_progress.store(new_progress.min(1.0));
        })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let pushed_block_state = read_pushed_block_state(nbt);
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
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            write_piston_fields(self, nbt);
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        write_piston_fields(self, &mut nbt);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

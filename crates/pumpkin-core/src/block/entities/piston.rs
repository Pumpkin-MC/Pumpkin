use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::{
    PistonHeadLikeProperties, StickyPistonLikeProperties as PistonProps,
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

    /// Ports vanilla `PistonBlockEntity.pushEntities`: pushes entities whose
    /// bounding box intersects the moving block's swept volume during this tick.
    fn push_entities(&self, world: &Arc<World>, new_progress: f32) {
        let last = self.last_progress.load();
        let delta = f64::from(new_progress - last);
        if delta <= 0.0 {
            return;
        }

        let motion_dir = self.movement_direction();
        let amount = f64::from(self.amount_extended(last));

        // Block AABB at the current visual position (start of this tick).
        // For source=true (the piston head BE), vanilla uses the piston-head
        // collision shape — a 4-pixel-thick slab at the front face — rather than
        // the full cube. Without this, the head "sweeps" too much volume and
        // drags entities (e.g. end crystals) back with it during retraction.
        let raw_block_aabb = BoundingBox::from_block(&self.position);
        let block_aabb = if self.source {
            Self::head_shape_aabb(raw_block_aabb, self.facing)
        } else {
            raw_block_aabb
        }
        .shift(Self::dir_vec(self.facing, amount));

        // Stretch by motion to get the swept volume (one tick of animation).
        let motion = Self::dir_vec(motion_dir, delta);
        let swept = block_aabb.stretch(motion);

        for entity in world.get_entities_at_box(&swept) {
            let e = entity.get_entity();
            if e.no_physics.load(Ordering::Relaxed) {
                continue;
            }
            // Player movement is client-authoritative; vanilla still nudges them
            // via Entity.move(PISTON), but we skip them here to avoid teleport jank.
            if entity.get_player().is_some() {
                continue;
            }

            let entity_aabb = e.bounding_box.load();
            let intersection = Self::intersection_size(swept, motion_dir, entity_aabb);
            if intersection <= 0.0 {
                continue;
            }
            let push_amount = intersection.min(delta) + 0.01;
            Self::move_entity(e, motion_dir, push_amount);

            // For a retracting head, vanilla also shoves the entity OUT of the
            // piston body cube. Without this, entities get pulled into the
            // piston and "stick" to it (looks like a sticky-piston drag).
            // For a retract-head BE, `self.position` already IS the piston
            // block position (it replaces the piston during animation).
            if !self.extending && self.source {
                Self::push_out_of_piston_body(e, &self.position, motion_dir, delta);
            }
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

    fn move_entity(entity: &crate::entity::Entity, dir: BlockDirection, distance: f64) {
        let new_pos = entity.pos.load() + Self::dir_vec(dir, distance);
        entity.set_pos(new_pos);
    }

    /// Vanilla `push`: when a piston head retracts, shove entities that ended up
    /// inside the piston-body cube back out the opposite direction (slightly past
    /// the move they just got, so the net motion is essentially zero).
    fn push_out_of_piston_body(
        entity: &crate::entity::Entity,
        piston_pos: &BlockPos,
        motion_dir: BlockDirection,
        amount: f64,
    ) {
        let body_aabb = BoundingBox::from_block(piston_pos);
        let entity_aabb = entity.bounding_box.load();
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
            Self::move_entity(entity, back, distance);
        }
    }

    /// Approximation of vanilla `PistonHeadBlock` collision shape: a 4-pixel
    /// (0.25 block) slab at the front face of the block in the facing direction.
    fn head_shape_aabb(block_aabb: BoundingBox, facing: BlockDirection) -> BoundingBox {
        const HEAD_THICKNESS: f64 = 0.25;
        let mut min = block_aabb.min;
        let mut max = block_aabb.max;
        match facing {
            BlockDirection::East => min.x = max.x - HEAD_THICKNESS,
            BlockDirection::West => max.x = min.x + HEAD_THICKNESS,
            BlockDirection::Up => min.y = max.y - HEAD_THICKNESS,
            BlockDirection::Down => max.y = min.y + HEAD_THICKNESS,
            BlockDirection::South => min.z = max.z - HEAD_THICKNESS,
            BlockDirection::North => max.z = min.z + HEAD_THICKNESS,
        }
        BoundingBox::new(min, max)
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

    /// Vanilla `PistonMovingBlockEntity.getCollisionShape`, in world coordinates: the moved
    /// block (the piston head for the source) where the animation has it this tick, plus the
    /// extended piston base while a source piston retracts. Without it a `moving_piston` has
    /// no collision and entities fall into the block for the whole animation.
    pub fn collision_shapes(&self) -> Vec<BoundingBox> {
        let pos = self.position;
        let mut shapes = Vec::new();
        let pushed = Block::from_state_id(self.pushed_block_state.id);
        if !self.extending
            && self.source
            && (pushed == &Block::PISTON || pushed == &Block::STICKY_PISTON)
        {
            let mut base = PistonProps::from_state_id(self.pushed_block_state.id);
            base.extended = true;
            let base = BlockState::from_id(base.to_state_id(pushed));
            shapes.extend(
                base.get_block_collision_shapes_at(&pos)
                    .map(|s| s.at_pos(pos)),
            );
        }
        let progress = self.current_progress.load();
        let moved = if self.source {
            let mut head = if pushed == &Block::PISTON_HEAD {
                PistonHeadLikeProperties::from_state_id(self.pushed_block_state.id)
            } else {
                PistonHeadLikeProperties::default(&Block::PISTON_HEAD)
            };
            head.facing = self.facing.to_facing();
            head.short = self.extending != (1.0 - progress < 0.25);
            BlockState::from_id(head.to_state_id(&Block::PISTON_HEAD))
        } else {
            self.pushed_block_state
        };
        let offset = Self::dir_vec(self.facing, f64::from(self.amount_extended(progress)));
        shapes.extend(
            moved
                .get_block_collision_shapes_at(&pos)
                .map(|s| s.at_pos(pos).shift(offset)),
        );
        shapes
    }

    pub fn finish(&self, world: &Arc<World>) {
        if self.last_progress.load() < 1.0 {
            let pos = self.position;
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                let state = if self.source {
                    Block::AIR.default_state.id
                } else {
                    world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos)
                };
                world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL);
                world.update_neighbors(&pos, None);
            }
        }
    }
}

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

    fn tick(&self, world: &Arc<World>) {
        let current_progress = self.current_progress.load();
        self.last_progress.store(current_progress);
        if current_progress >= 1.0 {
            let pos = self.position;
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                if self.pushed_block_state.is_air() {
                    world.set_block_state(
                        &pos,
                        self.pushed_block_state.id,
                        BlockFlags::FORCE_STATE | BlockFlags::MOVED,
                    );
                } else {
                    let updated_state =
                        world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos);
                    world.set_block_state(
                        &pos,
                        updated_state,
                        BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                    );
                    world.update_neighbors(&pos, None);
                }
            }
            return;
        }
        let new_progress = (current_progress + 0.5).min(1.0);
        self.push_entities(world, new_progress);
        self.current_progress.store(new_progress);
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        // TODO
        let pushed_block_state = Block::AIR.default_state;
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

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        // TODO: pushed_block_state
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        // TODO: pushed_block_state
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
        // TODO: duplicated code because of async :c
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::{Block, BlockDirection, BlockState, entity::EntityType};
    use pumpkin_util::math::{
        boundingbox::BoundingBox, position::BlockPos, vector2::Vector2, vector3::Vector3,
    };

    use super::{PistonBlockEntity, PistonProps};
    use crate::entity::{Entity, EntityBase, decoration::armor_stand::ArmorStandEntity};
    use crate::world::{BlockFlags, World};

    fn moving(
        position: BlockPos,
        pushed: &'static BlockState,
        facing: BlockDirection,
        extending: bool,
        source: bool,
    ) -> PistonBlockEntity {
        PistonBlockEntity {
            position,
            pushed_block_state: pushed,
            facing,
            current_progress: 0.0.into(),
            last_progress: 0.0.into(),
            extending,
            source,
        }
    }

    fn top(shapes: &[BoundingBox]) -> f64 {
        shapes
            .iter()
            .map(|s| s.max.y)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    #[test]
    fn a_moved_block_collides_where_the_animation_draws_it() {
        let pos = BlockPos::new(0, 64, 0);
        // Pushed up into `pos`: it starts a block lower and rises with the progress.
        let pushed = moving(
            pos,
            Block::STONE.default_state,
            BlockDirection::Up,
            true,
            false,
        );
        assert_eq!(top(&pushed.collision_shapes()), 64.0);
        pushed.current_progress.store(0.5);
        assert_eq!(top(&pushed.collision_shapes()), 64.5);
        // Pulled down into `pos`: it starts a block higher and sinks.
        let pulled = moving(
            pos,
            Block::STONE.default_state,
            BlockDirection::Up,
            false,
            false,
        );
        assert_eq!(top(&pulled.collision_shapes()), 66.0);
        pulled.current_progress.store(0.5);
        assert_eq!(top(&pulled.collision_shapes()), 65.5);
        // A moved block with no collision stays without one.
        let air = moving(
            pos,
            Block::AIR.default_state,
            BlockDirection::Up,
            false,
            false,
        );
        assert!(air.collision_shapes().is_empty());
    }

    #[test]
    fn a_retracting_piston_keeps_its_base_and_its_moving_head() {
        let pos = BlockPos::new(0, 64, 0);
        let mut props = PistonProps::default(&Block::STICKY_PISTON);
        props.facing = BlockDirection::Up.to_facing();
        let piston = BlockState::from_id(props.to_state_id(&Block::STICKY_PISTON));
        let retracting = moving(pos, piston, BlockDirection::Up, false, true);
        let shapes = retracting.collision_shapes();
        // The 12-pixel extended base stays in place...
        assert!(
            shapes
                .iter()
                .any(|s| s.min.y == 64.0 && (s.max.y - 64.75).abs() < 1e-9 && s.max.x == 1.0)
        );
        // ...and the head starts where it was, a block above.
        assert_eq!(top(&shapes), 66.0);
        retracting.current_progress.store(0.5);
        assert_eq!(top(&retracting.collision_shapes()), 65.5);
    }

    fn load_chunk(world: &Arc<World>) {
        world.level.loaded_chunks.insert(
            Vector2::new(0, 0),
            pumpkin_world::chunk::ChunkData::empty_sync(0, 0),
        );
    }

    fn fall(stand: &ArmorStandEntity) -> f64 {
        stand
            .get_entity()
            .move_entity(stand, Vector3::new(0.0, -1.0, 0.0));
        stand.get_entity().pos.load().y
    }

    /// A sticky piston pulls the stone an armor stand rests on down a block: the stand rides
    /// it down instead of falling into the `moving_piston`, and ends on the stone.
    #[tokio::test]
    async fn an_entity_rides_a_block_pulled_down_instead_of_falling_into_it()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        load_chunk(&world);
        // The stone was at y = 66 and is being pulled down to y = 65.
        let target = BlockPos::new(0, 65, 0);
        world.set_block_state(
            &target,
            Block::MOVING_PISTON.default_state.id,
            BlockFlags::FORCE_STATE,
        );
        world.add_block_entity(Arc::new(moving(
            target,
            Block::STONE.default_state,
            BlockDirection::Up,
            false,
            false,
        )));
        let stand = ArmorStandEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 67.0, 0.5),
            &EntityType::ARMOR_STAND,
        ));

        // Its feet are two blocks above the `moving_piston`, out of the usual scan range.
        assert_eq!(fall(&stand), 67.0, "the stone still starts at y = 66");
        assert!(
            !world.is_space_empty(
                stand
                    .get_entity()
                    .bounding_box
                    .load()
                    .shift(Vector3::new(0.0, -0.25, 0.0))
            )
        );
        let block_entity = world
            .get_block_entity(&target)
            .ok_or("no piston block entity")?;
        block_entity.tick(&world);
        assert_eq!(fall(&stand), 66.5, "half way down");
        block_entity.tick(&world);
        block_entity.tick(&world);
        assert_eq!(world.get_block(&target), &Block::STONE);
        assert_eq!(fall(&stand), 66.0, "on the stone at its new place");
        world.level.shutdown().await;
        Ok(())
    }

    /// A pulled fence is 1.5 blocks tall, so at the start of its move it reaches two blocks
    /// above its `moving_piston`: every collision query must still find it.
    #[tokio::test]
    async fn a_tall_moving_shape_is_found_two_blocks_above_its_piston()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        load_chunk(&world);
        let target = BlockPos::new(0, 64, 0);
        world.set_block_state(
            &target,
            Block::MOVING_PISTON.default_state.id,
            BlockFlags::FORCE_STATE,
        );
        world.add_block_entity(Arc::new(moving(
            target,
            Block::OAK_FENCE.default_state,
            BlockDirection::Up,
            false,
            false,
        )));
        // The fence is drawn from y = 65 to 66.5; this box sits in its top half.
        let query = BoundingBox::new(Vector3::new(0.4, 66.1, 0.4), Vector3::new(0.6, 66.3, 0.6));
        assert!(!world.is_space_empty(query));
        assert!(!world.no_block_collision(query));
        let stand = ArmorStandEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 66.2, 0.5),
            &EntityType::ARMOR_STAND,
        ));
        assert!(!world.get_block_collisions(query, &stand).0.is_empty());
        // Beside the fence's post there is room.
        let beside = BoundingBox::new(Vector3::new(0.8, 66.1, 0.8), Vector3::new(0.9, 66.3, 0.9));
        assert!(world.is_space_empty(beside));
        assert!(world.no_block_collision(beside));
        world.level.shutdown().await;
        Ok(())
    }

    /// An entity landing on a block pushed up into the air lands on it mid-animation instead of
    /// falling through the `moving_piston` it still is.
    #[tokio::test]
    async fn an_entity_lands_on_a_block_still_moving() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        load_chunk(&world);
        let target = BlockPos::new(0, 65, 0);
        world.set_block_state(
            &target,
            Block::MOVING_PISTON.default_state.id,
            BlockFlags::FORCE_STATE,
        );
        let pushed = Arc::new(moving(
            target,
            Block::STONE.default_state,
            BlockDirection::Up,
            true,
            false,
        ));
        pushed.current_progress.store(0.5);
        world.add_block_entity(pushed);
        let stand = ArmorStandEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 66.2, 0.5),
            &EntityType::ARMOR_STAND,
        ));
        assert_eq!(fall(&stand), 65.5);
        world.level.shutdown().await;
        Ok(())
    }
}

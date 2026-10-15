use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::generation::structure::template::{BlockStateResolver, PaletteEntry};

use crate::world::{BlockFlags, World};

use super::BlockEntity;

pub struct PistonBlockEntity {
    pub position: BlockPos,
    pub pushed_block_state: &'static BlockState,
    pub facing: BlockDirection,
    pub current_progress: AtomicCell<f32>,
    pub last_progress: AtomicCell<f32>,
    pub last_ticked: AtomicCell<i64>,
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

        let is_slime = self.pushed_block_state.id.to_block() == &Block::SLIME_BLOCK;

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

            if is_slime {
                let off = motion_dir.to_offset();
                let mut vel = e.velocity.load();
                if off.x != 0 {
                    vel.x = f64::from(off.x);
                }
                if off.y != 0 {
                    vel.y = f64::from(off.y);
                }
                if off.z != 0 {
                    vel.z = f64::from(off.z);
                }
                e.velocity.store(vel);
            }

            // For a retracting head, vanilla also shoves the entity OUT of the
            // piston body cube. Without this, entities get pulled into the
            // piston and "stick" to it (looks like a sticky-piston drag).
            // For a retract-head BE, `self.position` already IS the piston
            // block position (it replaces the piston during animation).
            if !self.extending && self.source {
                Self::push_out_of_piston_body(e, &self.position, motion_dir, delta);
            }
        }

        if is_slime {
            let off = motion_dir.to_offset();
            for player in world.get_players_at_box(&swept) {
                let mut vel = player.living_entity.entity.velocity.load();
                if off.x != 0 {
                    vel.x = f64::from(off.x);
                }
                if off.y != 0 {
                    vel.y = f64::from(off.y);
                }
                if off.z != 0 {
                    vel.z = f64::from(off.z);
                }
                player.set_velocity(vel);
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

    pub fn finish(&self, world: &Arc<World>) {
        if self.last_progress.load() < 1.0 {
            self.current_progress.store(1.0);
            self.last_progress.store(1.0);
            let pos = self.position;
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                let state = if self.source {
                    if self.extending {
                        Block::AIR.default_state.id
                    } else {
                        self.pushed_block_state.id
                    }
                } else {
                    world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos)
                };
                let new_block = Block::from_state_id(state);
                let state = new_block.set_waterlogged(state, false).unwrap_or(state);
                world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL | BlockFlags::MOVED);
                world.update_neighbor(&pos, new_block);
            }
        }
    }
}

pub fn block_state_to_nbt(state: &'static BlockState) -> NbtCompound {
    let block = Block::from_state_id(state.id);
    let mut comp = NbtCompound::new();
    let name = if block.name.starts_with("minecraft:") {
        block.name.to_string()
    } else {
        format!("minecraft:{}", block.name)
    };
    comp.put_string("id", name.clone());
    comp.put_string("Name", name);
    if let Some(props) = block.properties(state.id) {
        let prop_vec = props.to_props();
        if !prop_vec.is_empty() {
            let mut props_comp = NbtCompound::new();
            for (k, v) in prop_vec {
                props_comp.put_string(k, v.to_string());
            }
            comp.put_compound("properties", props_comp.clone());
            comp.put_compound("Properties", props_comp);
        }
    }
    comp
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
        self.last_ticked.store(world.get_world_age());
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
                        BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                    );
                    world.update_neighbors(&pos, None);
                } else {
                    let updated_state =
                        world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos);
                    let updated_block = Block::from_state_id(updated_state);
                    if updated_block == &Block::AIR {
                        world.set_block_state(
                            &pos,
                            self.pushed_block_state.id,
                            BlockFlags::NOTIFY_LISTENERS
                                | BlockFlags::MOVED
                                | BlockFlags::UPDATE_KNOWN_SHAPE,
                        );
                        world.break_block(&pos, None, BlockFlags::NOTIFY_ALL);
                    } else {
                        let new_state = updated_block
                            .set_waterlogged(updated_state, false)
                            .unwrap_or(updated_state);
                        world.set_block_state(
                            &pos,
                            new_state,
                            BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                        );
                        world.update_neighbor(&pos, updated_block);
                    }
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
        let pushed_block_state = if let Some(bs_compound) = nbt.get_compound("blockState") {
            if let Ok(entry) = PaletteEntry::from_nbt_compound(bs_compound) {
                BlockStateResolver::resolve_simple(&entry).unwrap_or(Block::AIR.default_state)
            } else {
                Block::AIR.default_state
            }
        } else {
            Block::AIR.default_state
        };
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
            last_ticked: 0.into(),
            extending,
            source,
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_compound("blockState", block_state_to_nbt(self.pushed_block_state));
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_compound("blockState", block_state_to_nbt(self.pushed_block_state));
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
        Some(nbt)
    }

    // The client animates moving blocks itself from the piston block event.
    fn update_packet_nbt(&self) -> Option<NbtCompound> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

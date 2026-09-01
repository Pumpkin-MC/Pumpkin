use rand::Rng;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::block_properties::{JigsawLikeProperties, Orientation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, xoroshiro128::Xoroshiro},
};
use pumpkin_world::generation::structure::structures::{
    StructureGeneratorContext, StructurePosition,
    jigsaw::{JigsawJointType, PoolElementStructurePiece},
    jigsaw_placement::{
        DimensionPadding, JigsawPlacement, LiquidSettings, MaxDistance, PoolAliasLookup,
    },
};

use crate::block::blocks::jigsaw::JigsawBlock;
use crate::world::World;

use super::BlockEntity;

pub struct JigsawBlockEntity {
    pub position: BlockPos,
    pub name: Mutex<String>,
    pub target: Mutex<String>,
    pub pool: Mutex<String>,
    pub final_state: Mutex<String>,
    pub joint: Mutex<JigsawJointType>,
    pub selection_priority: AtomicI32,
    pub placement_priority: AtomicI32,
    pub dirty: AtomicBool,
}

impl JigsawBlockEntity {
    pub const ID: &'static str = "minecraft:jigsaw";
    pub const EMPTY_ID: &'static str = "minecraft:empty";
    pub const DEFAULT_FINAL_STATE: &'static str = "minecraft:air";
    pub const DEFAULT_PLACEMENT_PRIORITY: i32 = 0;
    pub const DEFAULT_SELECTION_PRIORITY: i32 = 0;
    pub const NAME: &'static str = "name";
    pub const TARGET: &'static str = "target";
    pub const POOL: &'static str = "pool";
    pub const FINAL_STATE: &'static str = "final_state";
    pub const JOINT: &'static str = "joint";
    pub const PLACEMENT_PRIORITY: &'static str = "placement_priority";
    pub const SELECTION_PRIORITY: &'static str = "selection_priority";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            name: Mutex::new(Self::EMPTY_ID.to_string()),
            target: Mutex::new(Self::EMPTY_ID.to_string()),
            pool: Mutex::new(Self::EMPTY_ID.to_string()),
            final_state: Mutex::new(Self::DEFAULT_FINAL_STATE.to_string()),
            joint: Mutex::new(JigsawJointType::Rollable),
            selection_priority: AtomicI32::new(Self::DEFAULT_SELECTION_PRIORITY),
            placement_priority: AtomicI32::new(Self::DEFAULT_PLACEMENT_PRIORITY),
            dirty: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub const fn get_default_joint_type(orientation: Orientation) -> JigsawJointType {
        let front = JigsawBlock::get_front_facing(orientation);
        if front.is_horizontal() {
            JigsawJointType::Aligned
        } else {
            JigsawJointType::Rollable
        }
    }

    pub fn generate(&self, world: &Arc<World>, levels: i32, keep_jigsaws: bool) {
        let pool = self
            .pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let target = self
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        let block_state = world.get_block_state(&self.position);
        let props = JigsawLikeProperties::from_state_id(block_state.id);
        let front = JigsawBlock::get_front_facing(props.r#orientation);

        let position = self.position.offset(front.to_offset());

        let structure = {
            let mut context = StructureGeneratorContext {
                seed: world.level_info.load().world_gen_settings.seed,
                chunk_x: position.chunk_position().x,
                chunk_z: position.chunk_position().y,
                random: RandomGenerator::Xoroshiro(Xoroshiro::from_seed(rand::rng().next_u64())),
                sea_level: 63,
                min_y: -64,
                height_sampler: None,
                structure_key: None,
            };

            JigsawPlacement::add_pieces(
                &mut context,
                &pool,
                Some(&target),
                levels,
                position,
                false,
                false,
                &MaxDistance::new(128),
                DimensionPadding::ZERO,
                LiquidSettings::ApplyWaterlog,
                &PoolAliasLookup::default(),
            )
        };

        if let Some(structure) = structure {
            Self::place_structure(world, &structure, keep_jigsaws);
        }
    }

    fn place_structure(world: &Arc<World>, structure: &StructurePosition, keep_jigsaws: bool) {
        let mut pieces = std::mem::take(
            &mut structure
                .collector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pieces,
        );
        let mut placer = crate::world::block_placer::WorldBlockPlacer::new(world);
        for piece in &mut pieces {
            if let Some(pool_piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() {
                pumpkin_world::generation::structure::structures::jigsaw::place_pool_element_templates(
                    pool_piece,
                    &mut placer,
                    None,
                    keep_jigsaws,
                );
            }
        }
        placer.finalize();
        world.queue_block_updates(&placer.changed_positions);
        world.flush_block_updates();
    }
}

impl BlockEntity for JigsawBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }
    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let name = Mutex::new(
            nbt.get_string(Self::NAME)
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
        );
        let target = Mutex::new(
            nbt.get_string(Self::TARGET)
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
        );
        let pool = Mutex::new(
            nbt.get_string(Self::POOL)
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
        );
        let final_state = Mutex::new(
            nbt.get_string(Self::FINAL_STATE)
                .unwrap_or(Self::DEFAULT_FINAL_STATE)
                .to_string(),
        );
        let joint = Mutex::new(
            nbt.get_string(Self::JOINT)
                .map_or(JigsawJointType::Rollable, JigsawJointType::from_str),
        );
        let selection_priority = AtomicI32::new(
            nbt.get_int(Self::SELECTION_PRIORITY)
                .unwrap_or(Self::DEFAULT_SELECTION_PRIORITY),
        );
        let placement_priority = AtomicI32::new(
            nbt.get_int(Self::PLACEMENT_PRIORITY)
                .unwrap_or(Self::DEFAULT_PLACEMENT_PRIORITY),
        );

        Self {
            position,
            name,
            target,
            pool,
            final_state,
            joint,
            selection_priority,
            placement_priority,
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(name) = self.name.lock() {
            nbt.put_string(Self::NAME, name.clone());
        }
        if let Ok(target) = self.target.lock() {
            nbt.put_string(Self::TARGET, target.clone());
        }
        if let Ok(pool) = self.pool.lock() {
            nbt.put_string(Self::POOL, pool.clone());
        }
        if let Ok(final_state) = self.final_state.lock() {
            nbt.put_string(Self::FINAL_STATE, final_state.clone());
        }
        if let Ok(joint) = self.joint.lock() {
            nbt.put_string(Self::JOINT, joint.as_str().to_string());
        }
        nbt.put_int(
            Self::PLACEMENT_PRIORITY,
            self.placement_priority.load(Ordering::SeqCst),
        );
        nbt.put_int(
            Self::SELECTION_PRIORITY,
            self.selection_priority.load(Ordering::SeqCst),
        );
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_string(Self::NAME, self.name.try_lock().ok()?.clone());
        nbt.put_string(Self::TARGET, self.target.try_lock().ok()?.clone());
        nbt.put_string(Self::POOL, self.pool.try_lock().ok()?.clone());
        nbt.put_string(Self::FINAL_STATE, self.final_state.try_lock().ok()?.clone());
        let joint = *self.joint.try_lock().ok()?;
        nbt.put_string(Self::JOINT, joint.as_str().to_string());
        nbt.put_int(
            Self::PLACEMENT_PRIORITY,
            self.placement_priority.load(Ordering::SeqCst),
        );
        nbt.put_int(
            Self::SELECTION_PRIORITY,
            self.selection_priority.load(Ordering::SeqCst),
        );
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

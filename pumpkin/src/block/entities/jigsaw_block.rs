use rand::Rng;
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
};

use futures::Future;
use pumpkin_data::block_properties::{BlockProperties, JigsawLikeProperties, Orientation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection,
    math::position::BlockPos,
    random::{RandomGenerator, xoroshiro128::Xoroshiro},
};
use pumpkin_world::generation::structure::{
    structures::{
        StructureGeneratorContext, StructurePosition,
        jigsaw::{JigsawJointType, PoolElementStructurePiece},
        jigsaw_placement::{
            DimensionPadding, JigsawPlacement, LiquidSettings, MaxDistance, PoolAliasLookup,
        },
    },
    template::BlockStateResolver,
};

use tokio::sync::Mutex;

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
    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            name: Mutex::new("minecraft:empty".to_string()),
            target: Mutex::new("minecraft:empty".to_string()),
            pool: Mutex::new("minecraft:empty".to_string()),
            final_state: Mutex::new("minecraft:air".to_string()),
            joint: Mutex::new(JigsawJointType::Rollable),
            selection_priority: AtomicI32::new(0),
            placement_priority: AtomicI32::new(0),
            dirty: AtomicBool::new(false),
        }
    }

    pub async fn generate(&self, world: &Arc<World>, levels: i32, keep_jigsaws: bool) {
        let pool = self.pool.lock().await.clone();
        let target = self.target.lock().await.clone();

        let block_state = world.get_block_state(&self.position);
        let props =
            JigsawLikeProperties::from_state_id(block_state.id, &pumpkin_data::Block::JIGSAW);
        let (front, _top) = Self::to_front_top(props.r#orientation);

        let position = self.position.offset(front.to_vector());

        let structure = {
            let mut context = StructureGeneratorContext {
                seed: world.level_info.load().world_gen_settings.seed,
                chunk_x: self.position.chunk_position().x,
                chunk_z: self.position.chunk_position().y,
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
                &MaxDistance::new(80),
                &DimensionPadding::ZERO,
                LiquidSettings::ApplyWaterlog,
                &PoolAliasLookup,
            )
        };

        if let Some(structure) = structure {
            self.place_structure(world, structure, keep_jigsaws).await;
        }
    }

    async fn place_structure(
        &self,
        world: &Arc<World>,
        structure: StructurePosition,
        keep_jigsaws: bool,
    ) {
        let mut pieces = std::mem::take(&mut structure.collector.lock().unwrap().pieces);
        for piece in &mut pieces {
            if let Some(pool_piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() {
                let origin = pool_piece.pos;
                let rotation = pool_piece.rotation;
                let mut templates = Vec::new();
                pool_piece
                    .element
                    .for_each_template(|_, _, template| templates.push(template));

                for template in templates {
                    for block in &template.blocks {
                        let palette_entry = &template.palette[block.state as usize];
                        if palette_entry.name == "minecraft:structure_void" {
                            continue;
                        }

                        if !keep_jigsaws && palette_entry.name == "minecraft:jigsaw" {
                            let final_state_str = block
                                .nbt
                                .as_ref()
                                .and_then(|n| n.get_string("final_state"))
                                .unwrap_or("minecraft:air");

                            let entry =
                            pumpkin_world::generation::structure::template::PaletteEntry::from_string(
                                final_state_str,
                            );
                            let final_state =
                            pumpkin_world::generation::structure::template::BlockStateResolver::resolve(
                                &entry,
                                rotation,
                                pumpkin_data::Mirror::default(),
                            )
                            .unwrap_or(pumpkin_data::Block::AIR.default_state);

                            let local_pos = rotation.transform_pos(block.pos, template.size);
                            let world_pos = origin.add(local_pos.x, local_pos.y, local_pos.z);

                            world
                                .set_block_state(
                                    &world_pos,
                                    final_state.id,
                                    pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                                )
                                .await;
                            continue;
                        }

                        let Some(state) = BlockStateResolver::resolve(
                            palette_entry,
                            rotation,
                            pumpkin_data::Mirror::default(),
                        ) else {
                            continue;
                        };

                        let local_pos = rotation.transform_pos(block.pos, template.size);
                        let world_pos = origin.add(local_pos.x, local_pos.y, local_pos.z);

                        world
                            .set_block_state(
                                &world_pos,
                                state.id,
                                pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                            )
                            .await;

                        // Handle block entities
                        if let Some(nbt) = &block.nbt {
                            let mut block_entity_nbt = nbt.clone();
                            block_entity_nbt.put_int("x", world_pos.0.x);
                            block_entity_nbt.put_int("y", world_pos.0.y);
                            block_entity_nbt.put_int("z", world_pos.0.z);
                            if let Some(block_entity) =
                                crate::block::entities::block_entity_from_nbt(&block_entity_nbt)
                            {
                                world.add_block_entity(block_entity);
                            }
                        }
                    }
                }
            }
        }
    }

    const fn to_front_top(orientation: Orientation) -> (BlockDirection, BlockDirection) {
        match orientation {
            Orientation::DownEast => (BlockDirection::Down, BlockDirection::East),
            Orientation::DownNorth => (BlockDirection::Down, BlockDirection::North),
            Orientation::DownSouth => (BlockDirection::Down, BlockDirection::South),
            Orientation::DownWest => (BlockDirection::Down, BlockDirection::West),
            Orientation::UpEast => (BlockDirection::Up, BlockDirection::East),
            Orientation::UpNorth => (BlockDirection::Up, BlockDirection::North),
            Orientation::UpSouth => (BlockDirection::Up, BlockDirection::South),
            Orientation::UpWest => (BlockDirection::Up, BlockDirection::West),
            Orientation::WestUp => (BlockDirection::West, BlockDirection::Up),
            Orientation::EastUp => (BlockDirection::East, BlockDirection::Up),
            Orientation::NorthUp => (BlockDirection::North, BlockDirection::Up),
            Orientation::SouthUp => (BlockDirection::South, BlockDirection::Up),
        }
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
            nbt.get_string("name")
                .unwrap_or("minecraft:empty")
                .to_string(),
        );
        let target = Mutex::new(
            nbt.get_string("target")
                .unwrap_or("minecraft:empty")
                .to_string(),
        );
        let pool = Mutex::new(
            nbt.get_string("pool")
                .unwrap_or("minecraft:empty")
                .to_string(),
        );
        let final_state = Mutex::new(
            nbt.get_string("final_state")
                .unwrap_or("minecraft:air")
                .to_string(),
        );
        let joint = Mutex::new(JigsawJointType::from_str(
            nbt.get_string("joint").unwrap_or("rollable"),
        ));
        let selection_priority = AtomicI32::new(nbt.get_int("selection_priority").unwrap_or(0));
        let placement_priority = AtomicI32::new(nbt.get_int("placement_priority").unwrap_or(0));

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

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            nbt.put_string("name", self.name.lock().await.to_string());
            nbt.put_string("target", self.target.lock().await.to_string());
            nbt.put_string("pool", self.pool.lock().await.to_string());
            nbt.put_string("final_state", self.final_state.lock().await.to_string());
            let joint = *self.joint.lock().await;
            nbt.put_string(
                "joint",
                match joint {
                    JigsawJointType::Rollable => "rollable".to_string(),
                    JigsawJointType::Aligned => "aligned".to_string(),
                },
            );
            nbt.put_int(
                "selection_priority",
                self.selection_priority.load(Ordering::SeqCst),
            );
            nbt.put_int(
                "placement_priority",
                self.placement_priority.load(Ordering::SeqCst),
            );
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        futures::executor::block_on(async {
            self.write_nbt(&mut nbt).await;
        });
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

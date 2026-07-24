use crate::chunk::{
    ChunkData, ChunkHeightmapType, ChunkHeightmaps, ChunkLight, ChunkSections,
    palette::{BiomePalette, BlockPalette},
};
use crate::generation::biome_coords;
use crate::tick::scheduler::ChunkTickScheduler;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::dimension::Dimension;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::ProtoChunk;
use crate::level::SyncChunk;

use pumpkin_data::chunk::ChunkStatus;
use std::sync::Mutex;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum StagedChunkEnum {
    None,
    /// Initial empty chunk, ready for biome population
    Empty = 1, // EMPTY STRUCTURE_STARTS STRUCTURE_REFERENCES
    /// Chunk with biomes populated, ready for noise generation
    Biomes,
    StructureStart,
    StructureReferences,
    /// Chunk with terrain noise generated, ready for surface building
    Noise,
    /// Chunk with surface built, ready for carvers
    Surface,
    /// Chunk with carvers applied, ready for features and structures
    Carvers,
    /// Chunk with features and structures, ready for lighting
    Features, // FEATURES
    /// Chunk with lighting calculated, ready for spawning
    Lighting, // INITIALIZE LIGHT
    /// Chunk with mobs spawned, ready for finalization
    Spawn, // SPAWN
    /// Fully generated chunk
    Full,
}

#[expect(clippy::fallible_impl_from)]
impl From<u8> for StagedChunkEnum {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Empty,
            2 => Self::Biomes,
            3 => Self::StructureStart,
            4 => Self::StructureReferences,
            5 => Self::Noise,
            6 => Self::Surface,
            7 => Self::Carvers,
            8 => Self::Features,
            9 => Self::Lighting,
            10 => Self::Spawn,
            11 => Self::Full,
            _ => panic!(),
        }
    }
}

impl From<ChunkStatus> for StagedChunkEnum {
    fn from(status: ChunkStatus) -> Self {
        match status {
            ChunkStatus::Empty => Self::Empty,
            ChunkStatus::StructureStarts => Self::StructureStart,
            ChunkStatus::StructureReferences => Self::StructureReferences,
            ChunkStatus::Biomes => Self::Biomes,
            ChunkStatus::Noise => Self::Noise,
            ChunkStatus::Surface => Self::Surface,
            ChunkStatus::Carvers => Self::Carvers,
            ChunkStatus::Features => Self::Features,
            ChunkStatus::InitializeLight | ChunkStatus::Light => Self::Lighting,
            ChunkStatus::Spawn => Self::Spawn,
            ChunkStatus::Full => Self::Full,
        }
    }
}

#[expect(clippy::fallible_impl_from)]
impl From<StagedChunkEnum> for ChunkStatus {
    fn from(status: StagedChunkEnum) -> Self {
        match status {
            StagedChunkEnum::Empty => Self::Empty,
            StagedChunkEnum::StructureStart => Self::StructureStarts,
            StagedChunkEnum::StructureReferences => Self::StructureReferences,
            StagedChunkEnum::Biomes => Self::Biomes,
            StagedChunkEnum::Noise => Self::Noise,
            StagedChunkEnum::Surface => Self::Surface,
            StagedChunkEnum::Carvers => Self::Carvers,
            StagedChunkEnum::Features => Self::Features,
            StagedChunkEnum::Lighting => Self::Light,
            StagedChunkEnum::Spawn => Self::Spawn,
            StagedChunkEnum::Full => Self::Full,
            StagedChunkEnum::None => panic!(),
        }
    }
}

impl StagedChunkEnum {
    #[must_use]
    pub const fn level_to_stage(level: i8) -> Self {
        if level <= 43 {
            Self::Full
        } else if level <= 44 {
            Self::Spawn
        } else if level <= 45 {
            Self::Lighting
        } else if level <= 46 {
            Self::Features
        } else if level <= 47 {
            Self::Carvers
        } else if level <= 48 {
            Self::Surface
        } else {
            Self::None
        }
    }

    /// Total number of state values (0 = None … 11 = Full).
    pub const COUNT: usize = Self::Full as usize + 1;
    pub const FULL_DEPENDENCIES: &'static [Self] = &[
        Self::Full,
        Self::Spawn,
        Self::Lighting,
        Self::Features,
        Self::Carvers,
        Self::Surface,
    ];
    pub const FULL_RADIUS: i32 = 4;
    #[must_use]
    pub const fn get_direct_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Features | Self::Lighting | Self::Spawn | Self::Full => 1,
            _ => 0,
        }
    }
    #[must_use]
    pub const fn get_write_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Features | Self::Lighting | Self::Spawn => 1,
            _ => 0,
        }
    }
    #[must_use]
    pub const fn get_direct_dependencies(self) -> &'static [Self] {
        match self {
            // In vanilla StructureStart is first, but since it needs the biome in Vanilla it gets computed in StructureStart and
            // the Biome Step, this should be more efficient
            Self::Biomes => &[Self::Empty],
            Self::StructureStart => &[Self::Biomes],
            Self::StructureReferences => &[
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
                Self::StructureStart,
            ],
            Self::Noise => &[Self::StructureReferences],
            Self::Surface => &[Self::Noise],
            Self::Carvers => &[Self::Surface],
            Self::Features => &[Self::Carvers, Self::Carvers],
            Self::Lighting => &[Self::Features, Self::Features],
            Self::Spawn => &[Self::Lighting, Self::Lighting],
            Self::Full => &[Self::Spawn, Self::Spawn],
            _ => panic!(),
        }
    }
}

pub enum Chunk {
    Level(SyncChunk),
    Proto(Box<ProtoChunk>),
}

impl Chunk {
    #[must_use]
    pub fn get_stage_id(&self) -> u8 {
        match self {
            Self::Proto(data) => data.stage_id(),
            Self::Level(_) => StagedChunkEnum::Full as u8,
        }
    }
    pub fn get_proto_chunk_mut(&mut self) -> &mut ProtoChunk {
        match self {
            Self::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Self::Proto(chunk) => chunk,
        }
    }
    #[must_use]
    pub fn get_proto_chunk(&self) -> &ProtoChunk {
        match self {
            Self::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Self::Proto(chunk) => chunk,
        }
    }

    fn build_level_sections(proto_chunk: &ProtoChunk, dimension: &Dimension) -> ChunkSections {
        let total_sections = dimension.height as usize / BlockPalette::SIZE;
        let biome_min_y = biome_coords::from_block(dimension.min_y);
        let block_sections = (0..total_sections)
            .map(|section_index| {
                BlockPalette::from_fn(|x, y, z| {
                    let y = section_index * BlockPalette::SIZE + y;
                    proto_chunk.get_block_state_raw(x as i32, y as i32, z as i32)
                })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let biome_sections = (0..total_sections)
            .map(|section_index| {
                BiomePalette::from_fn(|x, y, z| {
                    let y = section_index * BiomePalette::SIZE + y;
                    proto_chunk.get_biome_id(x as i32, biome_min_y + y as i32, z as i32)
                })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        ChunkSections::from_palettes(block_sections, biome_sections, dimension.min_y)
    }

    fn build_level_heightmaps(proto_chunk: &ProtoChunk, min_y: i32) -> ChunkHeightmaps {
        let mut heightmaps = ChunkHeightmaps::default();
        for x in 0..16 {
            for z in 0..16 {
                let source_index = x * 16 + z;
                for (heightmap_type, height) in [
                    (
                        ChunkHeightmapType::WorldSurface,
                        proto_chunk.flat_surface_height_map[source_index],
                    ),
                    (
                        ChunkHeightmapType::MotionBlocking,
                        proto_chunk.flat_motion_blocking_height_map[source_index],
                    ),
                    (
                        ChunkHeightmapType::MotionBlockingNoLeaves,
                        proto_chunk.flat_motion_blocking_no_leaves_height_map[source_index],
                    ),
                ] {
                    heightmaps.set(heightmap_type, x as i32, z as i32, i32::from(height), min_y);
                }
            }
        }
        heightmaps
    }

    pub fn upgrade_to_level_chunk(
        &mut self,
        dimension: &Dimension,
        lighting_config: &LightingEngineConfig,
    ) {
        // Take ownership of the ProtoChunk by temporarily replacing with a dummy value
        // This allows us to move the light data instead of cloning it
        let proto_chunk_box = match std::mem::replace(
            self,
            Self::Level(Arc::new(ChunkData {
                section: ChunkSections::new(0, 0),
                heightmap: Mutex::default(),
                x: 0,
                z: 0,
                block_ticks: ChunkTickScheduler::default(),
                fluid_ticks: ChunkTickScheduler::default(),
                pending_block_entities: Mutex::default(),
                light_engine: Mutex::new(ChunkLight::default()),
                light_populated: AtomicBool::new(false),
                status: ChunkStatus::Empty,
                blending_data: None,
                dirty: AtomicBool::new(false),
            })),
        ) {
            Self::Proto(proto) => proto,
            Self::Level(_) => panic!("Cannot upgrade a Level chunk"),
        };

        let proto_chunk = *proto_chunk_box;

        let sections = Self::build_level_sections(&proto_chunk, dimension);
        let heightmaps = Self::build_level_heightmaps(&proto_chunk, dimension.min_y);

        // Move the light data instead of cloning it
        // By taking ownership of proto_chunk, we can move the light data directly
        // This prevents keeping duplicate lighting data in memory
        let light_data = proto_chunk.light;

        // Only mark lit if past the lighting stage, and the lighting config is "default" ("full" and "dark" modes skip proper lighting)
        let is_lit = proto_chunk.stage >= StagedChunkEnum::Lighting
            && *lighting_config == LightingEngineConfig::Default;

        // Convert pending block entities from structure generation to actual block entities
        let mut pending_block_entities = FxHashMap::default();
        for nbt in proto_chunk.pending_block_entities {
            if let Some(x) = nbt.get_int("x")
                && let Some(y) = nbt.get_int("y")
                && let Some(z) = nbt.get_int("z")
            {
                pending_block_entities
                    .insert(pumpkin_util::math::position::BlockPos::new(x, y, z), nbt);
            }
        }

        let chunk = ChunkData {
            light_engine: Mutex::new(light_data),
            light_populated: AtomicBool::new(is_lit),
            section: sections,
            heightmap: Mutex::new(heightmaps),
            x: proto_chunk.x,
            z: proto_chunk.z,
            dirty: AtomicBool::new(true),
            block_ticks: ChunkTickScheduler::default(),
            fluid_ticks: ChunkTickScheduler::from_iter(proto_chunk.fluid_ticks),
            pending_block_entities: Mutex::new(pending_block_entities),
            status: proto_chunk.stage.into(),
            blending_data: proto_chunk.blending_data,
        };

        *self = Self::Level(Arc::new(chunk));
    }
}

use std::path::PathBuf;

use pumpkin_data::chunk::ChunkStatus;

pub struct LevelFolder {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
    pub entities_folder: PathBuf,
}

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
            ChunkStatus::InitializeLight => Self::Lighting,
            ChunkStatus::Light => Self::Lighting,
            ChunkStatus::Spawn => Self::Spawn,
            ChunkStatus::Full => Self::Full,
        }
    }
}

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
            _ => panic!(),
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
            Self::Empty => 0,
            Self::StructureStart => 0,
            Self::StructureReferences => 0,
            Self::Biomes => 0,
            Self::Noise => 0,
            Self::Surface => 0,
            Self::Carvers => 0,
            Self::Features => 1,
            Self::Lighting => 1,
            Self::Spawn => 1,
            Self::Full => 1,
            _ => panic!(),
        }
    }
    #[must_use]
    pub const fn get_write_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Empty => 0,
            Self::StructureStart => 0,
            Self::StructureReferences => 0,
            Self::Biomes => 0,
            Self::Noise => 0,
            Self::Surface => 0,
            Self::Carvers => 0,
            Self::Features => 1,
            Self::Lighting => 1,
            Self::Spawn => 1,
            Self::Full => 0,
            _ => panic!(),
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
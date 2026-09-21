/* This file is generated. Do not edit manually. */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MetaDataType {
    pub id: i32,
}
impl MetaDataType {
    pub const ARMADILLO_STATE: MetaDataType = MetaDataType { id: 36i32 };
    pub const BLOCK_POS: MetaDataType = MetaDataType { id: 10i32 };
    pub const BLOCK_STATE: MetaDataType = MetaDataType { id: 14i32 };
    pub const BOOLEAN: MetaDataType = MetaDataType { id: 8i32 };
    pub const BYTE: MetaDataType = MetaDataType { id: 0i32 };
    pub const CAT_SOUND_VARIANT: MetaDataType = MetaDataType { id: 22i32 };
    pub const CAT_VARIANT: MetaDataType = MetaDataType { id: 21i32 };
    pub const CHICKEN_SOUND_VARIANT: MetaDataType = MetaDataType { id: 31i32 };
    pub const CHICKEN_VARIANT: MetaDataType = MetaDataType { id: 30i32 };
    pub const COMPONENT: MetaDataType = MetaDataType { id: 5i32 };
    pub const COW_SOUND_VARIANT: MetaDataType = MetaDataType { id: 24i32 };
    pub const COW_VARIANT: MetaDataType = MetaDataType { id: 23i32 };
    pub const DIRECTION: MetaDataType = MetaDataType { id: 12i32 };
    pub const DYE_COLOR: MetaDataType = MetaDataType { id: 43i32 };
    pub const FLOAT: MetaDataType = MetaDataType { id: 3i32 };
    pub const FROG_VARIANT: MetaDataType = MetaDataType { id: 27i32 };
    pub const HUMANOID_ARM: MetaDataType = MetaDataType { id: 42i32 };
    pub const INT: MetaDataType = MetaDataType { id: 1i32 };
    pub const ITEM_STACK: MetaDataType = MetaDataType { id: 7i32 };
    pub const LONG: MetaDataType = MetaDataType { id: 2i32 };
    pub const OPTIONAL_BLOCK_POS: MetaDataType = MetaDataType { id: 11i32 };
    pub const OPTIONAL_BLOCK_STATE: MetaDataType = MetaDataType { id: 15i32 };
    pub const OPTIONAL_COMPONENT: MetaDataType = MetaDataType { id: 6i32 };
    pub const OPTIONAL_GLOBAL_POS: MetaDataType = MetaDataType { id: 33i32 };
    pub const OPTIONAL_LIVING_ENTITY_REFERENCE: MetaDataType = MetaDataType { id: 13i32 };
    pub const OPTIONAL_UNSIGNED_INT: MetaDataType = MetaDataType { id: 19i32 };
    pub const PAINTING_VARIANT: MetaDataType = MetaDataType { id: 34i32 };
    pub const PARTICLE: MetaDataType = MetaDataType { id: 16i32 };
    pub const PARTICLES: MetaDataType = MetaDataType { id: 17i32 };
    pub const PIG_SOUND_VARIANT: MetaDataType = MetaDataType { id: 29i32 };
    pub const PIG_VARIANT: MetaDataType = MetaDataType { id: 28i32 };
    pub const POSE: MetaDataType = MetaDataType { id: 20i32 };
    pub const QUATERNION: MetaDataType = MetaDataType { id: 40i32 };
    pub const RESOLVABLE_PROFILE: MetaDataType = MetaDataType { id: 41i32 };
    pub const ROTATIONS: MetaDataType = MetaDataType { id: 9i32 };
    pub const SNIFFER_STATE: MetaDataType = MetaDataType { id: 35i32 };
    pub const STRING: MetaDataType = MetaDataType { id: 4i32 };
    pub const VECTOR3: MetaDataType = MetaDataType { id: 39i32 };
    pub const VILLAGER_DATA: MetaDataType = MetaDataType { id: 18i32 };
    pub const WEATHERING_COPPER_STATE: MetaDataType = MetaDataType { id: 38i32 };
    pub const WOLF_SOUND_VARIANT: MetaDataType = MetaDataType { id: 26i32 };
    pub const WOLF_VARIANT: MetaDataType = MetaDataType { id: 25i32 };
    pub const ZOMBIE_NAUTILUS_VARIANT: MetaDataType = MetaDataType { id: 32i32 };
    pub const INTEGER: MetaDataType = Self::INT;
    pub const ENTITY_POSE: MetaDataType = Self::POSE;
    pub const FACING: MetaDataType = Self::DIRECTION;
    pub const TEXT_COMPONENT: MetaDataType = Self::COMPONENT;
    pub const OPTIONAL_TEXT_COMPONENT: MetaDataType = Self::OPTIONAL_COMPONENT;
    pub const OPTIONAL_INT: MetaDataType = Self::OPTIONAL_UNSIGNED_INT;
    pub const VECTOR_3F: MetaDataType = Self::VECTOR3;
    pub const QUATERNION_F: MetaDataType = Self::QUATERNION;
    pub const ROTATION: MetaDataType = Self::ROTATIONS;
    pub const PARTICLE_LIST: MetaDataType = Self::PARTICLES;
    pub const COPPER_GOLEM_STATE: MetaDataType = Self::WEATHERING_COPPER_STATE;
    pub const PROFILE: MetaDataType = Self::RESOLVABLE_PROFILE;
    pub const ARM: MetaDataType = Self::HUMANOID_ARM;
    #[must_use]
    pub const fn id(&self, _version: pumpkin_util::version::JavaMinecraftVersion) -> i32 {
        self.id
    }
}

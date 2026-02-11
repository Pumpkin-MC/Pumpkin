/* This file is generated. Do not edit manually. */
use crate::tag::RegistryKey;
use crate::tag::Taggable;
use pumpkin_util::HeightMap;
use pumpkin_util::loot_table::*;
use std::hash::Hash;
#[derive(Debug)]
pub struct EntityType {
    pub id: u16,
    pub max_health: Option<f32>,
    pub attackable: Option<bool>,
    pub mob: bool,
    pub saveable: bool,
    pub limit_per_chunk: i32,
    pub summonable: bool,
    pub fire_immune: bool,
    pub category: &'static MobCategory,
    pub can_spawn_far_from_player: bool,
    pub loot_table: Option<LootTable>,
    pub dimension: [f32; 2],
    pub eye_height: f32,
    pub spawn_restriction: SpawnRestriction,
    pub resource_name: &'static str,
}
impl Hash for EntityType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for EntityType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for EntityType {}
impl Taggable for EntityType {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::EntityType
    }
    #[inline]
    fn registry_key(&self) -> &str {
        self.resource_name
    }
    #[inline]
    fn registry_id(&self) -> u16 {
        self.id
    }
}
#[derive(Debug)]
pub struct SpawnRestriction {
    pub location: SpawnLocation,
    pub heightmap: HeightMap,
}
#[derive(Debug)]
pub enum SpawnLocation {
    InLava,
    InWater,
    OnGround,
    Unrestricted,
}
#[derive(Debug)]
pub struct MobCategory {
    pub id: usize,
    pub max: i32,
    pub is_friendly: bool,
    pub is_persistent: bool,
    pub despawn_distance: i32,
}
impl PartialEq for MobCategory {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl MobCategory {
    pub const NO_DESPAWN_DISTANCE: i32 = 32;
    pub const MONSTER: MobCategory = MobCategory {
        id: 0,
        max: 70,
        is_friendly: false,
        is_persistent: false,
        despawn_distance: 128,
    };
    pub const CREATURE: MobCategory = MobCategory {
        id: 1,
        max: 10,
        is_friendly: true,
        is_persistent: true,
        despawn_distance: 128,
    };
    pub const AMBIENT: MobCategory = MobCategory {
        id: 2,
        max: 15,
        is_friendly: true,
        is_persistent: false,
        despawn_distance: 128,
    };
    pub const AXOLOTLS: MobCategory = MobCategory {
        id: 3,
        max: 5,
        is_friendly: true,
        is_persistent: false,
        despawn_distance: 128,
    };
    pub const UNDERGROUND_WATER_CREATURE: MobCategory = MobCategory {
        id: 4,
        max: 5,
        is_friendly: true,
        is_persistent: false,
        despawn_distance: 128,
    };
    pub const WATER_CREATURE: MobCategory = MobCategory {
        id: 5,
        max: 5,
        is_friendly: true,
        is_persistent: true,
        despawn_distance: 128,
    };
    pub const WATER_AMBIENT: MobCategory = MobCategory {
        id: 6,
        max: 20,
        is_friendly: true,
        is_persistent: false,
        despawn_distance: 64,
    };
    pub const MISC: MobCategory = MobCategory {
        id: 7,
        max: -1,
        is_friendly: true,
        is_persistent: true,
        despawn_distance: 128,
    };
    pub const SPAWNING_CATEGORIES: [&'static Self; 8] = [
        &Self::MONSTER,
        &Self::CREATURE,
        &Self::AMBIENT,
        &Self::AXOLOTLS,
        &Self::UNDERGROUND_WATER_CREATURE,
        &Self::WATER_CREATURE,
        &Self::WATER_AMBIENT,
        &Self::MISC,
    ];
}
impl EntityType {
    pub const ACACIA_BOAT: EntityType = EntityType {
        id: 0,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "acacia_boat",
    };
    pub const ACACIA_CHEST_BOAT: EntityType = EntityType {
        id: 1,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "acacia_chest_boat",
    };
    pub const ALLAY: EntityType = EntityType {
        id: 2,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/allay"),
            pools: None,
        }),
        dimension: [0.35f32, 0.6f32],
        eye_height: 0.36f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "allay",
    };
    pub const AREA_EFFECT_CLOUD: EntityType = EntityType {
        id: 3,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [6f32, 0.5f32],
        eye_height: 0.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "area_effect_cloud",
    };
    pub const ARMADILLO: EntityType = EntityType {
        id: 4,
        max_health: Some(12f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/armadillo"),
            pools: None,
        }),
        dimension: [0.7f32, 0.65f32],
        eye_height: 0.26f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "armadillo",
    };
    pub const ARMOR_STAND: EntityType = EntityType {
        id: 5,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/armor_stand"),
            pools: None,
        }),
        dimension: [0.5f32, 1.975f32],
        eye_height: 1.7775f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "armor_stand",
    };
    pub const ARROW: EntityType = EntityType {
        id: 6,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0.13f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "arrow",
    };
    pub const AXOLOTL: EntityType = EntityType {
        id: 7,
        max_health: Some(14f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::AXOLOTLS,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/axolotl"),
            pools: None,
        }),
        dimension: [0.75f32, 0.42f32],
        eye_height: 0.2751f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "axolotl",
    };
    pub const BAMBOO_CHEST_RAFT: EntityType = EntityType {
        id: 8,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "bamboo_chest_raft",
    };
    pub const BAMBOO_RAFT: EntityType = EntityType {
        id: 9,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "bamboo_raft",
    };
    pub const BAT: EntityType = EntityType {
        id: 10,
        max_health: Some(6f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::AMBIENT,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/bat"),
            pools: None,
        }),
        dimension: [0.5f32, 0.9f32],
        eye_height: 0.45f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "bat",
    };
    pub const BEE: EntityType = EntityType {
        id: 11,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/bee"),
            pools: None,
        }),
        dimension: [0.7f32, 0.6f32],
        eye_height: 0.3f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "bee",
    };
    pub const BIRCH_BOAT: EntityType = EntityType {
        id: 12,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "birch_boat",
    };
    pub const BIRCH_CHEST_BOAT: EntityType = EntityType {
        id: 13,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "birch_chest_boat",
    };
    pub const BLAZE: EntityType = EntityType {
        id: 14,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/blaze"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:blaze_rod",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 1f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::KilledByPlayer]),
                functions: None,
            }]),
        }),
        dimension: [0.6f32, 1.8f32],
        eye_height: 1.53f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "blaze",
    };
    pub const BLOCK_DISPLAY: EntityType = EntityType {
        id: 15,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0f32, 0f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "block_display",
    };
    pub const BOGGED: EntityType = EntityType {
        id: 16,
        max_health: Some(16f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/bogged"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:tipped_arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::SetPotion,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.99f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "bogged",
    };
    pub const BREEZE: EntityType = EntityType {
        id: 17,
        max_health: Some(30f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/breeze"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:breeze_rod",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::KilledByPlayer]),
                functions: None,
            }]),
        }),
        dimension: [0.6f32, 1.77f32],
        eye_height: 1.3452f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "breeze",
    };
    pub const BREEZE_WIND_CHARGE: EntityType = EntityType {
        id: 18,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.3125f32, 0.3125f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "breeze_wind_charge",
    };
    pub const CAMEL: EntityType = EntityType {
        id: 19,
        max_health: Some(32f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/camel"),
            pools: None,
        }),
        dimension: [1.7f32, 2.375f32],
        eye_height: 2.275f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "camel",
    };
    pub const CAMEL_HUSK: EntityType = EntityType {
        id: 20,
        max_health: Some(32f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/camel_husk"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:rotten_flesh",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 2f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.7f32, 2.375f32],
        eye_height: 2.275f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "camel_husk",
    };
    pub const CAT: EntityType = EntityType {
        id: 21,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/cat"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:string",
                    }),
                    conditions: None,
                    functions: Some(&[LootFunction {
                        content: LootFunctionTypes::SetCount {
                            count: LootFunctionNumberProvider::Uniform {
                                min: 0f32,
                                max: 2f32,
                            },
                            add: false,
                        },
                        conditions: None,
                    }]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.6f32, 0.7f32],
        eye_height: 0.35f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "cat",
    };
    pub const CAVE_SPIDER: EntityType = EntityType {
        id: 22,
        max_health: Some(12f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/cave_spider"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:string",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:spider_eye",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: -1f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.7f32, 0.5f32],
        eye_height: 0.45f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "cave_spider",
    };
    pub const CHERRY_BOAT: EntityType = EntityType {
        id: 23,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "cherry_boat",
    };
    pub const CHERRY_CHEST_BOAT: EntityType = EntityType {
        id: 24,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "cherry_chest_boat",
    };
    pub const CHEST_MINECART: EntityType = EntityType {
        id: 25,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "chest_minecart",
    };
    pub const CHICKEN: EntityType = EntityType {
        id: 26,
        max_health: Some(4f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/chicken"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:feather",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:chicken",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [0.4f32, 0.7f32],
        eye_height: 0.644f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "chicken",
    };
    pub const COD: EntityType = EntityType {
        id: 27,
        max_health: Some(3f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 8i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_AMBIENT,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/cod"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:cod",
                        }),
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::FurnaceSmelt,
                            conditions: Some(&[LootCondition::AnyOf]),
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone_meal",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::RandomChance]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.5f32, 0.3f32],
        eye_height: 0.195f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "cod",
    };
    pub const COMMAND_BLOCK_MINECART: EntityType = EntityType {
        id: 29,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "command_block_minecart",
    };
    pub const COPPER_GOLEM: EntityType = EntityType {
        id: 28,
        max_health: Some(12f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/copper_golem"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:copper_ingot",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.49f32, 0.98f32],
        eye_height: 0.8125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "copper_golem",
    };
    pub const COW: EntityType = EntityType {
        id: 30,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/cow"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:leather",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:beef",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 1f32,
                                        max: 3f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [0.9f32, 1.4f32],
        eye_height: 1.3f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "cow",
    };
    pub const CREAKING: EntityType = EntityType {
        id: 31,
        max_health: Some(1f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/creaking"),
            pools: None,
        }),
        dimension: [0.9f32, 2.7f32],
        eye_height: 2.3f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "creaking",
    };
    pub const CREEPER: EntityType = EntityType {
        id: 32,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/creeper"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:gunpowder",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Tag,
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::EntityProperties]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.7f32],
        eye_height: 1.445f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "creeper",
    };
    pub const DARK_OAK_BOAT: EntityType = EntityType {
        id: 33,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "dark_oak_boat",
    };
    pub const DARK_OAK_CHEST_BOAT: EntityType = EntityType {
        id: 34,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "dark_oak_chest_boat",
    };
    pub const DOLPHIN: EntityType = EntityType {
        id: 35,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_CREATURE,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/dolphin"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:cod",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 1f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::FurnaceSmelt,
                            conditions: Some(&[LootCondition::AnyOf]),
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 0.6f32],
        eye_height: 0.3f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "dolphin",
    };
    pub const DONKEY: EntityType = EntityType {
        id: 36,
        max_health: Some(53f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/donkey"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:leather",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3964844f32, 1.5f32],
        eye_height: 1.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "donkey",
    };
    pub const DRAGON_FIREBALL: EntityType = EntityType {
        id: 37,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1f32, 1f32],
        eye_height: 0.85f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "dragon_fireball",
    };
    pub const DROWNED: EntityType = EntityType {
        id: 38,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/drowned"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rotten_flesh",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:copper_ingot",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "drowned",
    };
    pub const EGG: EntityType = EntityType {
        id: 39,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "egg",
    };
    pub const ELDER_GUARDIAN: EntityType = EntityType {
        id: 40,
        max_health: Some(80f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/elder_guardian"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:prismarine_shard",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:cod",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::FurnaceSmelt,
                                    conditions: Some(&[LootCondition::AnyOf]),
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:prismarine_crystals",
                            }),
                            conditions: None,
                            functions: Some(&[LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            }]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Empty,
                            conditions: None,
                            functions: None,
                        },
                    ],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:wet_sponge",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::LootTable,
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::FurnaceSmelt,
                            conditions: Some(&[LootCondition::AnyOf]),
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Empty,
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:tide_armor_trim_smithing_template",
                            }),
                            conditions: None,
                            functions: None,
                        },
                    ],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [1.9975f32, 1.9975f32],
        eye_height: 0.99875f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "elder_guardian",
    };
    pub const END_CRYSTAL: EntityType = EntityType {
        id: 45,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [2f32, 2f32],
        eye_height: 1.7f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "end_crystal",
    };
    pub const ENDER_DRAGON: EntityType = EntityType {
        id: 43,
        max_health: Some(200f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/ender_dragon"),
            pools: None,
        }),
        dimension: [16f32, 8f32],
        eye_height: 6.8f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "ender_dragon",
    };
    pub const ENDER_PEARL: EntityType = EntityType {
        id: 44,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "ender_pearl",
    };
    pub const ENDERMAN: EntityType = EntityType {
        id: 41,
        max_health: Some(40f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/enderman"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:ender_pearl",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 1f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.6f32, 2.9f32],
        eye_height: 2.55f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "enderman",
    };
    pub const ENDERMITE: EntityType = EntityType {
        id: 42,
        max_health: Some(8f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/endermite"),
            pools: None,
        }),
        dimension: [0.4f32, 0.3f32],
        eye_height: 0.13f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "endermite",
    };
    pub const EVOKER: EntityType = EntityType {
        id: 46,
        max_health: Some(24f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/evoker"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:totem_of_undying",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:emerald",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.6575f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "evoker",
    };
    pub const EVOKER_FANGS: EntityType = EntityType {
        id: 47,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.8f32],
        eye_height: 0.68f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "evoker_fangs",
    };
    pub const EXPERIENCE_BOTTLE: EntityType = EntityType {
        id: 48,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "experience_bottle",
    };
    pub const EXPERIENCE_ORB: EntityType = EntityType {
        id: 49,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "experience_orb",
    };
    pub const EYE_OF_ENDER: EntityType = EntityType {
        id: 50,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "eye_of_ender",
    };
    pub const FALLING_BLOCK: EntityType = EntityType {
        id: 51,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.98f32],
        eye_height: 0.83300006f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "falling_block",
    };
    pub const FIREBALL: EntityType = EntityType {
        id: 52,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1f32, 1f32],
        eye_height: 0.85f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "fireball",
    };
    pub const FIREWORK_ROCKET: EntityType = EntityType {
        id: 53,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "firework_rocket",
    };
    pub const FISHING_BOBBER: EntityType = EntityType {
        id: 156,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: false,
        limit_per_chunk: 0i32,
        summonable: false,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "fishing_bobber",
    };
    pub const FOX: EntityType = EntityType {
        id: 54,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/fox"),
            pools: None,
        }),
        dimension: [0.6f32, 0.7f32],
        eye_height: 0.4f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "fox",
    };
    pub const FROG: EntityType = EntityType {
        id: 55,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/frog"),
            pools: None,
        }),
        dimension: [0.5f32, 0.5f32],
        eye_height: 0.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "frog",
    };
    pub const FURNACE_MINECART: EntityType = EntityType {
        id: 56,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "furnace_minecart",
    };
    pub const GHAST: EntityType = EntityType {
        id: 57,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 1i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/ghast"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:ghast_tear",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:gunpowder",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:music_disc_tears",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::DamageSourceProperties,
                        LootCondition::KilledByPlayer,
                    ]),
                    functions: Some(&[LootFunction {
                        content: LootFunctionTypes::SetCount {
                            count: LootFunctionNumberProvider::Constant { value: 1f32 },
                            add: false,
                        },
                        conditions: None,
                    }]),
                },
            ]),
        }),
        dimension: [4f32, 4f32],
        eye_height: 2.6f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "ghast",
    };
    pub const GIANT: EntityType = EntityType {
        id: 59,
        max_health: Some(100f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/giant"),
            pools: None,
        }),
        dimension: [3.6f32, 12f32],
        eye_height: 10.44f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "giant",
    };
    pub const GLOW_ITEM_FRAME: EntityType = EntityType {
        id: 60,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "glow_item_frame",
    };
    pub const GLOW_SQUID: EntityType = EntityType {
        id: 61,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::UNDERGROUND_WATER_CREATURE,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/glow_squid"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:glow_ink_sac",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.8f32, 0.8f32],
        eye_height: 0.4f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "glow_squid",
    };
    pub const GOAT: EntityType = EntityType {
        id: 62,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/goat"),
            pools: None,
        }),
        dimension: [0.9f32, 1.3f32],
        eye_height: 1.105f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "goat",
    };
    pub const GUARDIAN: EntityType = EntityType {
        id: 63,
        max_health: Some(30f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/guardian"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:prismarine_shard",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:cod",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::FurnaceSmelt,
                                    conditions: Some(&[LootCondition::AnyOf]),
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:prismarine_crystals",
                            }),
                            conditions: None,
                            functions: Some(&[LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            }]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Empty,
                            conditions: None,
                            functions: None,
                        },
                    ],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::LootTable,
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::FurnaceSmelt,
                            conditions: Some(&[LootCondition::AnyOf]),
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.85f32, 0.85f32],
        eye_height: 0.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "guardian",
    };
    pub const HAPPY_GHAST: EntityType = EntityType {
        id: 58,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 1i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/happy_ghast"),
            pools: None,
        }),
        dimension: [4f32, 4f32],
        eye_height: 2.6f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "happy_ghast",
    };
    pub const HOGLIN: EntityType = EntityType {
        id: 64,
        max_health: Some(40f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/hoglin"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:porkchop",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 2f32,
                                        max: 4f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:leather",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [1.3964844f32, 1.4f32],
        eye_height: 1.19f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "hoglin",
    };
    pub const HOPPER_MINECART: EntityType = EntityType {
        id: 65,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "hopper_minecart",
    };
    pub const HORSE: EntityType = EntityType {
        id: 66,
        max_health: Some(53f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/horse"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:leather",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3964844f32, 1.6f32],
        eye_height: 1.52f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "horse",
    };
    pub const HUSK: EntityType = EntityType {
        id: 67,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/husk"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rotten_flesh",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rabbit_foot",
                        }),
                        conditions: Some(&[LootCondition::EntityProperties]),
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:iron_ingot",
                            }),
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:carrot",
                            }),
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:potato",
                            }),
                            conditions: None,
                            functions: Some(&[LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            }]),
                        },
                    ],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "husk",
    };
    pub const ILLUSIONER: EntityType = EntityType {
        id: 68,
        max_health: Some(32f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/illusioner"),
            pools: None,
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.6575f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "illusioner",
    };
    pub const INTERACTION: EntityType = EntityType {
        id: 69,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0f32, 0f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "interaction",
    };
    pub const IRON_GOLEM: EntityType = EntityType {
        id: 70,
        max_health: Some(100f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/iron_golem"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:poppy",
                        }),
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:iron_ingot",
                        }),
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 3f32,
                                    max: 5f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [1.4f32, 2.7f32],
        eye_height: 2.295f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "iron_golem",
    };
    pub const ITEM: EntityType = EntityType {
        id: 71,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "item",
    };
    pub const ITEM_DISPLAY: EntityType = EntityType {
        id: 72,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0f32, 0f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "item_display",
    };
    pub const ITEM_FRAME: EntityType = EntityType {
        id: 73,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "item_frame",
    };
    pub const JUNGLE_BOAT: EntityType = EntityType {
        id: 74,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "jungle_boat",
    };
    pub const JUNGLE_CHEST_BOAT: EntityType = EntityType {
        id: 75,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "jungle_chest_boat",
    };
    pub const LEASH_KNOT: EntityType = EntityType {
        id: 76,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: false,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.375f32, 0.5f32],
        eye_height: 0.0625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "leash_knot",
    };
    pub const LIGHTNING_BOLT: EntityType = EntityType {
        id: 77,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: false,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0f32, 0f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "lightning_bolt",
    };
    pub const LINGERING_POTION: EntityType = EntityType {
        id: 106,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "lingering_potion",
    };
    pub const LLAMA: EntityType = EntityType {
        id: 78,
        max_health: Some(53f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/llama"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:leather",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 1.87f32],
        eye_height: 1.7765f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "llama",
    };
    pub const LLAMA_SPIT: EntityType = EntityType {
        id: 79,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "llama_spit",
    };
    pub const MAGMA_CUBE: EntityType = EntityType {
        id: 80,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/magma_cube"),
            pools: Some(&[LootPool {
                entries: &[
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:magma_cream",
                        }),
                        conditions: Some(&[
                            LootCondition::Inverted,
                            LootCondition::EntityProperties,
                        ]),
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: -2f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    },
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:pearlescent_froglight",
                        }),
                        conditions: Some(&[LootCondition::DamageSourceProperties]),
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    },
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:verdant_froglight",
                        }),
                        conditions: Some(&[LootCondition::DamageSourceProperties]),
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    },
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:ochre_froglight",
                        }),
                        conditions: Some(&[LootCondition::DamageSourceProperties]),
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    },
                ],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.52f32, 0.52f32],
        eye_height: 0.325f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "magma_cube",
    };
    pub const MANGROVE_BOAT: EntityType = EntityType {
        id: 81,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "mangrove_boat",
    };
    pub const MANGROVE_CHEST_BOAT: EntityType = EntityType {
        id: 82,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "mangrove_chest_boat",
    };
    pub const MANNEQUIN: EntityType = EntityType {
        id: 83,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/mannequin"),
            pools: None,
        }),
        dimension: [0.6f32, 1.8f32],
        eye_height: 1.62f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "mannequin",
    };
    pub const MARKER: EntityType = EntityType {
        id: 84,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0f32, 0f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "marker",
    };
    pub const MINECART: EntityType = EntityType {
        id: 85,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "minecart",
    };
    pub const MOOSHROOM: EntityType = EntityType {
        id: 86,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/mooshroom"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:leather",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:beef",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 1f32,
                                        max: 3f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [0.9f32, 1.4f32],
        eye_height: 1.3f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "mooshroom",
    };
    pub const MULE: EntityType = EntityType {
        id: 87,
        max_health: Some(53f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/mule"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:leather",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3964844f32, 1.6f32],
        eye_height: 1.52f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "mule",
    };
    pub const NAUTILUS: EntityType = EntityType {
        id: 88,
        max_health: Some(15f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_CREATURE,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/nautilus"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:nautilus_shell",
                    }),
                    conditions: None,
                    functions: None,
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[
                    LootCondition::KilledByPlayer,
                    LootCondition::RandomChanceWithEnchantedBonus,
                ]),
                functions: None,
            }]),
        }),
        dimension: [0.875f32, 0.95f32],
        eye_height: 0.2751f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "nautilus",
    };
    pub const OAK_BOAT: EntityType = EntityType {
        id: 89,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "oak_boat",
    };
    pub const OAK_CHEST_BOAT: EntityType = EntityType {
        id: 90,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "oak_chest_boat",
    };
    pub const OCELOT: EntityType = EntityType {
        id: 91,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/ocelot"),
            pools: None,
        }),
        dimension: [0.6f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlocking,
        },
        resource_name: "ocelot",
    };
    pub const OMINOUS_ITEM_SPAWNER: EntityType = EntityType {
        id: 92,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "ominous_item_spawner",
    };
    pub const PAINTING: EntityType = EntityType {
        id: 93,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "painting",
    };
    pub const PALE_OAK_BOAT: EntityType = EntityType {
        id: 94,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "pale_oak_boat",
    };
    pub const PALE_OAK_CHEST_BOAT: EntityType = EntityType {
        id: 95,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "pale_oak_chest_boat",
    };
    pub const PANDA: EntityType = EntityType {
        id: 96,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/panda"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:bamboo",
                    }),
                    conditions: None,
                    functions: Some(&[LootFunction {
                        content: LootFunctionTypes::SetCount {
                            count: LootFunctionNumberProvider::Constant { value: 1f32 },
                            add: false,
                        },
                        conditions: None,
                    }]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3f32, 1.25f32],
        eye_height: 1.0625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "panda",
    };
    pub const PARCHED: EntityType = EntityType {
        id: 97,
        max_health: Some(16f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/parched"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:tipped_arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::SetPotion,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.99f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "parched",
    };
    pub const PARROT: EntityType = EntityType {
        id: 98,
        max_health: Some(6f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/parrot"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:feather",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.5f32, 0.9f32],
        eye_height: 0.54f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlocking,
        },
        resource_name: "parrot",
    };
    pub const PHANTOM: EntityType = EntityType {
        id: 99,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/phantom"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:phantom_membrane",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 1f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::KilledByPlayer]),
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 0.5f32],
        eye_height: 0.175f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "phantom",
    };
    pub const PIG: EntityType = EntityType {
        id: 100,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/pig"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:porkchop",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::FurnaceSmelt,
                            conditions: Some(&[LootCondition::AnyOf]),
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 0.9f32],
        eye_height: 0.765f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "pig",
    };
    pub const PIGLIN: EntityType = EntityType {
        id: 101,
        max_health: Some(16f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/piglin"),
            pools: None,
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.79f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "piglin",
    };
    pub const PIGLIN_BRUTE: EntityType = EntityType {
        id: 102,
        max_health: Some(50f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/piglin_brute"),
            pools: None,
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.79f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "piglin_brute",
    };
    pub const PILLAGER: EntityType = EntityType {
        id: 103,
        max_health: Some(24f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 1i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/pillager"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:ominous_bottle",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::SetOminousBottleAmplifier,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::EntityProperties]),
                functions: None,
            }]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.6575f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "pillager",
    };
    pub const PLAYER: EntityType = EntityType {
        id: 155,
        max_health: None,
        attackable: None,
        mob: false,
        saveable: false,
        limit_per_chunk: 0i32,
        summonable: false,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/player"),
            pools: None,
        }),
        dimension: [0.6f32, 1.8f32],
        eye_height: 1.62f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "player",
    };
    pub const POLAR_BEAR: EntityType = EntityType {
        id: 104,
        max_health: Some(30f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/polar_bear"),
            pools: Some(&[LootPool {
                entries: &[
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:cod",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    },
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:salmon",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    },
                ],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.4f32, 1.4f32],
        eye_height: 1.19f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "polar_bear",
    };
    pub const PUFFERFISH: EntityType = EntityType {
        id: 107,
        max_health: Some(3f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 8i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_AMBIENT,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/pufferfish"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:pufferfish",
                        }),
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone_meal",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::RandomChance]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.7f32, 0.7f32],
        eye_height: 0.455f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "pufferfish",
    };
    pub const RABBIT: EntityType = EntityType {
        id: 108,
        max_health: Some(3f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/rabbit"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rabbit_hide",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rabbit",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rabbit_foot",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.4f32, 0.5f32],
        eye_height: 0.425f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "rabbit",
    };
    pub const RAVAGER: EntityType = EntityType {
        id: 109,
        max_health: Some(100f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/ravager"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:saddle",
                    }),
                    conditions: None,
                    functions: Some(&[LootFunction {
                        content: LootFunctionTypes::SetCount {
                            count: LootFunctionNumberProvider::Constant { value: 1f32 },
                            add: false,
                        },
                        conditions: None,
                    }]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.95f32, 2.2f32],
        eye_height: 1.8700001f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "ravager",
    };
    pub const SALMON: EntityType = EntityType {
        id: 110,
        max_health: Some(3f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 5i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_AMBIENT,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/salmon"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:salmon",
                        }),
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::FurnaceSmelt,
                            conditions: Some(&[LootCondition::AnyOf]),
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone_meal",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::RandomChance]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.7f32, 0.4f32],
        eye_height: 0.26f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "salmon",
    };
    pub const SHEEP: EntityType = EntityType {
        id: 111,
        max_health: Some(8f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/sheep"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:mutton",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 1f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Alternatives(AlternativeEntry {
                            children: &[
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                                LootPoolEntry {
                                    content: LootPoolEntryTypes::LootTable,
                                    conditions: Some(&[LootCondition::EntityProperties]),
                                    functions: None,
                                },
                            ],
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [0.9f32, 1.3f32],
        eye_height: 1.235f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "sheep",
    };
    pub const SHULKER: EntityType = EntityType {
        id: 112,
        max_health: Some(30f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/shulker"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:shulker_shell",
                    }),
                    conditions: None,
                    functions: None,
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::RandomChanceWithEnchantedBonus]),
                functions: None,
            }]),
        }),
        dimension: [1f32, 1f32],
        eye_height: 0.5f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "shulker",
    };
    pub const SHULKER_BULLET: EntityType = EntityType {
        id: 113,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.3125f32, 0.3125f32],
        eye_height: 0.265625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "shulker_bullet",
    };
    pub const SILVERFISH: EntityType = EntityType {
        id: 114,
        max_health: Some(8f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/silverfish"),
            pools: None,
        }),
        dimension: [0.4f32, 0.3f32],
        eye_height: 0.13f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "silverfish",
    };
    pub const SKELETON: EntityType = EntityType {
        id: 115,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/skeleton"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.99f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "skeleton",
    };
    pub const SKELETON_HORSE: EntityType = EntityType {
        id: 116,
        max_health: Some(15f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/skeleton_horse"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:bone",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3964844f32, 1.6f32],
        eye_height: 1.52f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "skeleton_horse",
    };
    pub const SLIME: EntityType = EntityType {
        id: 117,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/slime"),
            pools: Some(&[LootPool {
                entries: &[
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:slime_ball",
                        }),
                        conditions: Some(&[LootCondition::Inverted]),
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    },
                    LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:slime_ball",
                        }),
                        conditions: Some(&[LootCondition::DamageSourceProperties]),
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    },
                ],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::EntityProperties]),
                functions: None,
            }]),
        }),
        dimension: [0.52f32, 0.52f32],
        eye_height: 0.325f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "slime",
    };
    pub const SMALL_FIREBALL: EntityType = EntityType {
        id: 118,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.3125f32, 0.3125f32],
        eye_height: 0.265625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "small_fireball",
    };
    pub const SNIFFER: EntityType = EntityType {
        id: 119,
        max_health: Some(14f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/sniffer"),
            pools: None,
        }),
        dimension: [1.9f32, 1.75f32],
        eye_height: 1.05f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "sniffer",
    };
    pub const SNOW_GOLEM: EntityType = EntityType {
        id: 121,
        max_health: Some(4f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/snow_golem"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:snowball",
                    }),
                    conditions: None,
                    functions: Some(&[LootFunction {
                        content: LootFunctionTypes::SetCount {
                            count: LootFunctionNumberProvider::Uniform {
                                min: 0f32,
                                max: 15f32,
                            },
                            add: false,
                        },
                        conditions: None,
                    }]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.7f32, 1.9f32],
        eye_height: 1.7f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "snow_golem",
    };
    pub const SNOWBALL: EntityType = EntityType {
        id: 120,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "snowball",
    };
    pub const SPAWNER_MINECART: EntityType = EntityType {
        id: 122,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "spawner_minecart",
    };
    pub const SPECTRAL_ARROW: EntityType = EntityType {
        id: 123,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0.13f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "spectral_arrow",
    };
    pub const SPIDER: EntityType = EntityType {
        id: 124,
        max_health: Some(16f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/spider"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:string",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:spider_eye",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: -1f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
            ]),
        }),
        dimension: [1.4f32, 0.9f32],
        eye_height: 0.65f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "spider",
    };
    pub const SPLASH_POTION: EntityType = EntityType {
        id: 105,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.25f32, 0.25f32],
        eye_height: 0.2125f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "splash_potion",
    };
    pub const SPRUCE_BOAT: EntityType = EntityType {
        id: 125,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "spruce_boat",
    };
    pub const SPRUCE_CHEST_BOAT: EntityType = EntityType {
        id: 126,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [1.375f32, 0.5625f32],
        eye_height: 0.5625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "spruce_chest_boat",
    };
    pub const SQUID: EntityType = EntityType {
        id: 127,
        max_health: Some(10f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_CREATURE,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/squid"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:ink_sac",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.8f32, 0.8f32],
        eye_height: 0.4f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "squid",
    };
    pub const STRAY: EntityType = EntityType {
        id: 128,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/stray"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:tipped_arrow",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::SetPotion,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::KilledByPlayer]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.99f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "stray",
    };
    pub const STRIDER: EntityType = EntityType {
        id: 129,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/strider"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:string",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 2f32,
                                    max: 5f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 1.7f32],
        eye_height: 1.445f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InLava,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "strider",
    };
    pub const TADPOLE: EntityType = EntityType {
        id: 130,
        max_health: Some(6f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 8i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/tadpole"),
            pools: None,
        }),
        dimension: [0.4f32, 0.3f32],
        eye_height: 0.19500001f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "tadpole",
    };
    pub const TEXT_DISPLAY: EntityType = EntityType {
        id: 131,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0f32, 0f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "text_display",
    };
    pub const TNT: EntityType = EntityType {
        id: 132,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.98f32],
        eye_height: 0.15f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "tnt",
    };
    pub const TNT_MINECART: EntityType = EntityType {
        id: 133,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.98f32, 0.7f32],
        eye_height: 0.595f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "tnt_minecart",
    };
    pub const TRADER_LLAMA: EntityType = EntityType {
        id: 134,
        max_health: Some(53f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/trader_llama"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:leather",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 2f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 1.87f32],
        eye_height: 1.7765f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "trader_llama",
    };
    pub const TRIDENT: EntityType = EntityType {
        id: 135,
        max_health: None,
        attackable: Some(false),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.5f32, 0.5f32],
        eye_height: 0.13f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "trident",
    };
    pub const TROPICAL_FISH: EntityType = EntityType {
        id: 136,
        max_health: Some(3f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 8i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::WATER_AMBIENT,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/tropical_fish"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:tropical_fish",
                        }),
                        conditions: None,
                        functions: Some(&[LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Constant { value: 1f32 },
                                add: false,
                            },
                            conditions: None,
                        }]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone_meal",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::RandomChance]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.5f32, 0.4f32],
        eye_height: 0.26f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::InWater,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "tropical_fish",
    };
    pub const TURTLE: EntityType = EntityType {
        id: 137,
        max_health: Some(30f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/turtle"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:seagrass",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bowl",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[LootCondition::DamageSourceProperties]),
                    functions: None,
                },
            ]),
        }),
        dimension: [1.2f32, 0.4f32],
        eye_height: 0.34f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "turtle",
    };
    pub const VEX: EntityType = EntityType {
        id: 138,
        max_health: Some(14f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/vex"),
            pools: None,
        }),
        dimension: [0.4f32, 0.8f32],
        eye_height: 0.51875f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "vex",
    };
    pub const VILLAGER: EntityType = EntityType {
        id: 139,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/villager"),
            pools: None,
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.62f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "villager",
    };
    pub const VINDICATOR: EntityType = EntityType {
        id: 140,
        max_health: Some(24f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/vindicator"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:emerald",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 1f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::KilledByPlayer]),
                functions: None,
            }]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.6575f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "vindicator",
    };
    pub const WANDERING_TRADER: EntityType = EntityType {
        id: 141,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/wandering_trader"),
            pools: None,
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.62f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "wandering_trader",
    };
    pub const WARDEN: EntityType = EntityType {
        id: 142,
        max_health: Some(500f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/warden"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:sculk_catalyst",
                    }),
                    conditions: None,
                    functions: None,
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [0.9f32, 2.9f32],
        eye_height: 2.4650002f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "warden",
    };
    pub const WIND_CHARGE: EntityType = EntityType {
        id: 143,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.3125f32, 0.3125f32],
        eye_height: 0f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "wind_charge",
    };
    pub const WITCH: EntityType = EntityType {
        id: 144,
        max_health: Some(26f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/witch"),
            pools: Some(&[
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:glowstone_dust",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::SetCount {
                                        count: LootFunctionNumberProvider::Uniform {
                                            min: 0f32,
                                            max: 2f32,
                                        },
                                        add: false,
                                    },
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:sugar",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::SetCount {
                                        count: LootFunctionNumberProvider::Uniform {
                                            min: 0f32,
                                            max: 2f32,
                                        },
                                        add: false,
                                    },
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:spider_eye",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::SetCount {
                                        count: LootFunctionNumberProvider::Uniform {
                                            min: 0f32,
                                            max: 2f32,
                                        },
                                        add: false,
                                    },
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:glass_bottle",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::SetCount {
                                        count: LootFunctionNumberProvider::Uniform {
                                            min: 0f32,
                                            max: 2f32,
                                        },
                                        add: false,
                                    },
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:gunpowder",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::SetCount {
                                        count: LootFunctionNumberProvider::Uniform {
                                            min: 0f32,
                                            max: 2f32,
                                        },
                                        add: false,
                                    },
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                            ]),
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:stick",
                            }),
                            conditions: None,
                            functions: Some(&[
                                LootFunction {
                                    content: LootFunctionTypes::SetCount {
                                        count: LootFunctionNumberProvider::Uniform {
                                            min: 0f32,
                                            max: 2f32,
                                        },
                                        add: false,
                                    },
                                    conditions: None,
                                },
                                LootFunction {
                                    content: LootFunctionTypes::EnchantedCountIncrease,
                                    conditions: None,
                                },
                            ]),
                        },
                    ],
                    rolls: LootNumberProviderTypes::Object(
                        LootNumberProviderTypesProvider::Uniform(UniformLootNumberProvider {
                            min: 1f32,
                            max: 3f32,
                        }),
                    ),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:redstone",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 4f32,
                                        max: 8f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.62f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "witch",
    };
    pub const WITHER: EntityType = EntityType {
        id: 145,
        max_health: Some(300f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/wither"),
            pools: None,
        }),
        dimension: [0.9f32, 3.5f32],
        eye_height: 2.9750001f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "wither",
    };
    pub const WITHER_SKELETON: EntityType = EntityType {
        id: 146,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/wither_skeleton"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:coal",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: -1f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:bone",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:wither_skeleton_skull",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.7f32, 2.4f32],
        eye_height: 2.1f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "wither_skeleton",
    };
    pub const WITHER_SKULL: EntityType = EntityType {
        id: 147,
        max_health: None,
        attackable: Some(true),
        mob: false,
        saveable: true,
        limit_per_chunk: 0i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MISC,
        can_spawn_far_from_player: true,
        loot_table: None,
        dimension: [0.3125f32, 0.3125f32],
        eye_height: 0.265625f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "wither_skull",
    };
    pub const WOLF: EntityType = EntityType {
        id: 148,
        max_health: Some(8f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 8i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::CREATURE,
        can_spawn_far_from_player: true,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/wolf"),
            pools: None,
        }),
        dimension: [0.6f32, 0.85f32],
        eye_height: 0.68f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "wolf",
    };
    pub const ZOGLIN: EntityType = EntityType {
        id: 149,
        max_health: Some(40f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/zoglin"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:rotten_flesh",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 1f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3964844f32, 1.4f32],
        eye_height: 1.19f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "zoglin",
    };
    pub const ZOMBIE: EntityType = EntityType {
        id: 150,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/zombie"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rotten_flesh",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:red_mushroom",
                        }),
                        conditions: Some(&[LootCondition::EntityProperties]),
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:iron_ingot",
                            }),
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:carrot",
                            }),
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:potato",
                            }),
                            conditions: None,
                            functions: Some(&[LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            }]),
                        },
                    ],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:music_disc_lava_chicken",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::EntityProperties,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "zombie",
    };
    pub const ZOMBIE_HORSE: EntityType = EntityType {
        id: 151,
        max_health: Some(25f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 6i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/zombie_horse"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:rotten_flesh",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 2f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: None,
                functions: None,
            }]),
        }),
        dimension: [1.3964844f32, 1.6f32],
        eye_height: 1.52f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "zombie_horse",
    };
    pub const ZOMBIE_NAUTILUS: EntityType = EntityType {
        id: 152,
        max_health: Some(15f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/zombie_nautilus"),
            pools: Some(&[LootPool {
                entries: &[LootPoolEntry {
                    content: LootPoolEntryTypes::Item(ItemEntry {
                        name: "minecraft:rotten_flesh",
                    }),
                    conditions: None,
                    functions: Some(&[
                        LootFunction {
                            content: LootFunctionTypes::SetCount {
                                count: LootFunctionNumberProvider::Uniform {
                                    min: 0f32,
                                    max: 3f32,
                                },
                                add: false,
                            },
                            conditions: None,
                        },
                        LootFunction {
                            content: LootFunctionTypes::EnchantedCountIncrease,
                            conditions: None,
                        },
                    ]),
                }],
                rolls: LootNumberProviderTypes::Constant(1f32),
                bonus_rolls: 0f32,
                conditions: Some(&[LootCondition::KilledByPlayer]),
                functions: None,
            }]),
        }),
        dimension: [0.875f32, 0.95f32],
        eye_height: 0.2751f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::Unrestricted,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "zombie_nautilus",
    };
    pub const ZOMBIE_VILLAGER: EntityType = EntityType {
        id: 153,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: false,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/zombie_villager"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rotten_flesh",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 2f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:iron_ingot",
                            }),
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:carrot",
                            }),
                            conditions: None,
                            functions: None,
                        },
                        LootPoolEntry {
                            content: LootPoolEntryTypes::Item(ItemEntry {
                                name: "minecraft:potato",
                            }),
                            conditions: None,
                            functions: Some(&[LootFunction {
                                content: LootFunctionTypes::FurnaceSmelt,
                                conditions: Some(&[LootCondition::AnyOf]),
                            }]),
                        },
                    ],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.74f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "zombie_villager",
    };
    pub const ZOMBIFIED_PIGLIN: EntityType = EntityType {
        id: 154,
        max_health: Some(20f32),
        attackable: Some(true),
        mob: true,
        saveable: true,
        limit_per_chunk: 4i32,
        summonable: true,
        fire_immune: true,
        category: &MobCategory::MONSTER,
        can_spawn_far_from_player: false,
        loot_table: Some(LootTable {
            r#type: LootTableType::Entity,
            random_sequence: Some("minecraft:entities/zombified_piglin"),
            pools: Some(&[
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:rotten_flesh",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:gold_nugget",
                        }),
                        conditions: None,
                        functions: Some(&[
                            LootFunction {
                                content: LootFunctionTypes::SetCount {
                                    count: LootFunctionNumberProvider::Uniform {
                                        min: 0f32,
                                        max: 1f32,
                                    },
                                    add: false,
                                },
                                conditions: None,
                            },
                            LootFunction {
                                content: LootFunctionTypes::EnchantedCountIncrease,
                                conditions: None,
                            },
                        ]),
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: None,
                    functions: None,
                },
                LootPool {
                    entries: &[LootPoolEntry {
                        content: LootPoolEntryTypes::Item(ItemEntry {
                            name: "minecraft:gold_ingot",
                        }),
                        conditions: None,
                        functions: None,
                    }],
                    rolls: LootNumberProviderTypes::Constant(1f32),
                    bonus_rolls: 0f32,
                    conditions: Some(&[
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChanceWithEnchantedBonus,
                    ]),
                    functions: None,
                },
            ]),
        }),
        dimension: [0.6f32, 1.95f32],
        eye_height: 1.79f32,
        spawn_restriction: SpawnRestriction {
            location: SpawnLocation::OnGround,
            heightmap: HeightMap::MotionBlockingNoLeaves,
        },
        resource_name: "zombified_piglin",
    };
    pub const fn from_raw(id: u16) -> Option<&'static Self> {
        match id {
            0 => Some(&Self::ACACIA_BOAT),
            1 => Some(&Self::ACACIA_CHEST_BOAT),
            2 => Some(&Self::ALLAY),
            3 => Some(&Self::AREA_EFFECT_CLOUD),
            4 => Some(&Self::ARMADILLO),
            5 => Some(&Self::ARMOR_STAND),
            6 => Some(&Self::ARROW),
            7 => Some(&Self::AXOLOTL),
            8 => Some(&Self::BAMBOO_CHEST_RAFT),
            9 => Some(&Self::BAMBOO_RAFT),
            10 => Some(&Self::BAT),
            11 => Some(&Self::BEE),
            12 => Some(&Self::BIRCH_BOAT),
            13 => Some(&Self::BIRCH_CHEST_BOAT),
            14 => Some(&Self::BLAZE),
            15 => Some(&Self::BLOCK_DISPLAY),
            16 => Some(&Self::BOGGED),
            17 => Some(&Self::BREEZE),
            18 => Some(&Self::BREEZE_WIND_CHARGE),
            19 => Some(&Self::CAMEL),
            20 => Some(&Self::CAMEL_HUSK),
            21 => Some(&Self::CAT),
            22 => Some(&Self::CAVE_SPIDER),
            23 => Some(&Self::CHERRY_BOAT),
            24 => Some(&Self::CHERRY_CHEST_BOAT),
            25 => Some(&Self::CHEST_MINECART),
            26 => Some(&Self::CHICKEN),
            27 => Some(&Self::COD),
            29 => Some(&Self::COMMAND_BLOCK_MINECART),
            28 => Some(&Self::COPPER_GOLEM),
            30 => Some(&Self::COW),
            31 => Some(&Self::CREAKING),
            32 => Some(&Self::CREEPER),
            33 => Some(&Self::DARK_OAK_BOAT),
            34 => Some(&Self::DARK_OAK_CHEST_BOAT),
            35 => Some(&Self::DOLPHIN),
            36 => Some(&Self::DONKEY),
            37 => Some(&Self::DRAGON_FIREBALL),
            38 => Some(&Self::DROWNED),
            39 => Some(&Self::EGG),
            40 => Some(&Self::ELDER_GUARDIAN),
            45 => Some(&Self::END_CRYSTAL),
            43 => Some(&Self::ENDER_DRAGON),
            44 => Some(&Self::ENDER_PEARL),
            41 => Some(&Self::ENDERMAN),
            42 => Some(&Self::ENDERMITE),
            46 => Some(&Self::EVOKER),
            47 => Some(&Self::EVOKER_FANGS),
            48 => Some(&Self::EXPERIENCE_BOTTLE),
            49 => Some(&Self::EXPERIENCE_ORB),
            50 => Some(&Self::EYE_OF_ENDER),
            51 => Some(&Self::FALLING_BLOCK),
            52 => Some(&Self::FIREBALL),
            53 => Some(&Self::FIREWORK_ROCKET),
            156 => Some(&Self::FISHING_BOBBER),
            54 => Some(&Self::FOX),
            55 => Some(&Self::FROG),
            56 => Some(&Self::FURNACE_MINECART),
            57 => Some(&Self::GHAST),
            59 => Some(&Self::GIANT),
            60 => Some(&Self::GLOW_ITEM_FRAME),
            61 => Some(&Self::GLOW_SQUID),
            62 => Some(&Self::GOAT),
            63 => Some(&Self::GUARDIAN),
            58 => Some(&Self::HAPPY_GHAST),
            64 => Some(&Self::HOGLIN),
            65 => Some(&Self::HOPPER_MINECART),
            66 => Some(&Self::HORSE),
            67 => Some(&Self::HUSK),
            68 => Some(&Self::ILLUSIONER),
            69 => Some(&Self::INTERACTION),
            70 => Some(&Self::IRON_GOLEM),
            71 => Some(&Self::ITEM),
            72 => Some(&Self::ITEM_DISPLAY),
            73 => Some(&Self::ITEM_FRAME),
            74 => Some(&Self::JUNGLE_BOAT),
            75 => Some(&Self::JUNGLE_CHEST_BOAT),
            76 => Some(&Self::LEASH_KNOT),
            77 => Some(&Self::LIGHTNING_BOLT),
            106 => Some(&Self::LINGERING_POTION),
            78 => Some(&Self::LLAMA),
            79 => Some(&Self::LLAMA_SPIT),
            80 => Some(&Self::MAGMA_CUBE),
            81 => Some(&Self::MANGROVE_BOAT),
            82 => Some(&Self::MANGROVE_CHEST_BOAT),
            83 => Some(&Self::MANNEQUIN),
            84 => Some(&Self::MARKER),
            85 => Some(&Self::MINECART),
            86 => Some(&Self::MOOSHROOM),
            87 => Some(&Self::MULE),
            88 => Some(&Self::NAUTILUS),
            89 => Some(&Self::OAK_BOAT),
            90 => Some(&Self::OAK_CHEST_BOAT),
            91 => Some(&Self::OCELOT),
            92 => Some(&Self::OMINOUS_ITEM_SPAWNER),
            93 => Some(&Self::PAINTING),
            94 => Some(&Self::PALE_OAK_BOAT),
            95 => Some(&Self::PALE_OAK_CHEST_BOAT),
            96 => Some(&Self::PANDA),
            97 => Some(&Self::PARCHED),
            98 => Some(&Self::PARROT),
            99 => Some(&Self::PHANTOM),
            100 => Some(&Self::PIG),
            101 => Some(&Self::PIGLIN),
            102 => Some(&Self::PIGLIN_BRUTE),
            103 => Some(&Self::PILLAGER),
            155 => Some(&Self::PLAYER),
            104 => Some(&Self::POLAR_BEAR),
            107 => Some(&Self::PUFFERFISH),
            108 => Some(&Self::RABBIT),
            109 => Some(&Self::RAVAGER),
            110 => Some(&Self::SALMON),
            111 => Some(&Self::SHEEP),
            112 => Some(&Self::SHULKER),
            113 => Some(&Self::SHULKER_BULLET),
            114 => Some(&Self::SILVERFISH),
            115 => Some(&Self::SKELETON),
            116 => Some(&Self::SKELETON_HORSE),
            117 => Some(&Self::SLIME),
            118 => Some(&Self::SMALL_FIREBALL),
            119 => Some(&Self::SNIFFER),
            121 => Some(&Self::SNOW_GOLEM),
            120 => Some(&Self::SNOWBALL),
            122 => Some(&Self::SPAWNER_MINECART),
            123 => Some(&Self::SPECTRAL_ARROW),
            124 => Some(&Self::SPIDER),
            105 => Some(&Self::SPLASH_POTION),
            125 => Some(&Self::SPRUCE_BOAT),
            126 => Some(&Self::SPRUCE_CHEST_BOAT),
            127 => Some(&Self::SQUID),
            128 => Some(&Self::STRAY),
            129 => Some(&Self::STRIDER),
            130 => Some(&Self::TADPOLE),
            131 => Some(&Self::TEXT_DISPLAY),
            132 => Some(&Self::TNT),
            133 => Some(&Self::TNT_MINECART),
            134 => Some(&Self::TRADER_LLAMA),
            135 => Some(&Self::TRIDENT),
            136 => Some(&Self::TROPICAL_FISH),
            137 => Some(&Self::TURTLE),
            138 => Some(&Self::VEX),
            139 => Some(&Self::VILLAGER),
            140 => Some(&Self::VINDICATOR),
            141 => Some(&Self::WANDERING_TRADER),
            142 => Some(&Self::WARDEN),
            143 => Some(&Self::WIND_CHARGE),
            144 => Some(&Self::WITCH),
            145 => Some(&Self::WITHER),
            146 => Some(&Self::WITHER_SKELETON),
            147 => Some(&Self::WITHER_SKULL),
            148 => Some(&Self::WOLF),
            149 => Some(&Self::ZOGLIN),
            150 => Some(&Self::ZOMBIE),
            151 => Some(&Self::ZOMBIE_HORSE),
            152 => Some(&Self::ZOMBIE_NAUTILUS),
            153 => Some(&Self::ZOMBIE_VILLAGER),
            154 => Some(&Self::ZOMBIFIED_PIGLIN),
            _ => None,
        }
    }
    pub fn from_name(name: &str) -> Option<&'static Self> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        match name {
            "acacia_boat" => Some(&Self::ACACIA_BOAT),
            "acacia_chest_boat" => Some(&Self::ACACIA_CHEST_BOAT),
            "allay" => Some(&Self::ALLAY),
            "area_effect_cloud" => Some(&Self::AREA_EFFECT_CLOUD),
            "armadillo" => Some(&Self::ARMADILLO),
            "armor_stand" => Some(&Self::ARMOR_STAND),
            "arrow" => Some(&Self::ARROW),
            "axolotl" => Some(&Self::AXOLOTL),
            "bamboo_chest_raft" => Some(&Self::BAMBOO_CHEST_RAFT),
            "bamboo_raft" => Some(&Self::BAMBOO_RAFT),
            "bat" => Some(&Self::BAT),
            "bee" => Some(&Self::BEE),
            "birch_boat" => Some(&Self::BIRCH_BOAT),
            "birch_chest_boat" => Some(&Self::BIRCH_CHEST_BOAT),
            "blaze" => Some(&Self::BLAZE),
            "block_display" => Some(&Self::BLOCK_DISPLAY),
            "bogged" => Some(&Self::BOGGED),
            "breeze" => Some(&Self::BREEZE),
            "breeze_wind_charge" => Some(&Self::BREEZE_WIND_CHARGE),
            "camel" => Some(&Self::CAMEL),
            "camel_husk" => Some(&Self::CAMEL_HUSK),
            "cat" => Some(&Self::CAT),
            "cave_spider" => Some(&Self::CAVE_SPIDER),
            "cherry_boat" => Some(&Self::CHERRY_BOAT),
            "cherry_chest_boat" => Some(&Self::CHERRY_CHEST_BOAT),
            "chest_minecart" => Some(&Self::CHEST_MINECART),
            "chicken" => Some(&Self::CHICKEN),
            "cod" => Some(&Self::COD),
            "command_block_minecart" => Some(&Self::COMMAND_BLOCK_MINECART),
            "copper_golem" => Some(&Self::COPPER_GOLEM),
            "cow" => Some(&Self::COW),
            "creaking" => Some(&Self::CREAKING),
            "creeper" => Some(&Self::CREEPER),
            "dark_oak_boat" => Some(&Self::DARK_OAK_BOAT),
            "dark_oak_chest_boat" => Some(&Self::DARK_OAK_CHEST_BOAT),
            "dolphin" => Some(&Self::DOLPHIN),
            "donkey" => Some(&Self::DONKEY),
            "dragon_fireball" => Some(&Self::DRAGON_FIREBALL),
            "drowned" => Some(&Self::DROWNED),
            "egg" => Some(&Self::EGG),
            "elder_guardian" => Some(&Self::ELDER_GUARDIAN),
            "end_crystal" => Some(&Self::END_CRYSTAL),
            "ender_dragon" => Some(&Self::ENDER_DRAGON),
            "ender_pearl" => Some(&Self::ENDER_PEARL),
            "enderman" => Some(&Self::ENDERMAN),
            "endermite" => Some(&Self::ENDERMITE),
            "evoker" => Some(&Self::EVOKER),
            "evoker_fangs" => Some(&Self::EVOKER_FANGS),
            "experience_bottle" => Some(&Self::EXPERIENCE_BOTTLE),
            "experience_orb" => Some(&Self::EXPERIENCE_ORB),
            "eye_of_ender" => Some(&Self::EYE_OF_ENDER),
            "falling_block" => Some(&Self::FALLING_BLOCK),
            "fireball" => Some(&Self::FIREBALL),
            "firework_rocket" => Some(&Self::FIREWORK_ROCKET),
            "fishing_bobber" => Some(&Self::FISHING_BOBBER),
            "fox" => Some(&Self::FOX),
            "frog" => Some(&Self::FROG),
            "furnace_minecart" => Some(&Self::FURNACE_MINECART),
            "ghast" => Some(&Self::GHAST),
            "giant" => Some(&Self::GIANT),
            "glow_item_frame" => Some(&Self::GLOW_ITEM_FRAME),
            "glow_squid" => Some(&Self::GLOW_SQUID),
            "goat" => Some(&Self::GOAT),
            "guardian" => Some(&Self::GUARDIAN),
            "happy_ghast" => Some(&Self::HAPPY_GHAST),
            "hoglin" => Some(&Self::HOGLIN),
            "hopper_minecart" => Some(&Self::HOPPER_MINECART),
            "horse" => Some(&Self::HORSE),
            "husk" => Some(&Self::HUSK),
            "illusioner" => Some(&Self::ILLUSIONER),
            "interaction" => Some(&Self::INTERACTION),
            "iron_golem" => Some(&Self::IRON_GOLEM),
            "item" => Some(&Self::ITEM),
            "item_display" => Some(&Self::ITEM_DISPLAY),
            "item_frame" => Some(&Self::ITEM_FRAME),
            "jungle_boat" => Some(&Self::JUNGLE_BOAT),
            "jungle_chest_boat" => Some(&Self::JUNGLE_CHEST_BOAT),
            "leash_knot" => Some(&Self::LEASH_KNOT),
            "lightning_bolt" => Some(&Self::LIGHTNING_BOLT),
            "lingering_potion" => Some(&Self::LINGERING_POTION),
            "llama" => Some(&Self::LLAMA),
            "llama_spit" => Some(&Self::LLAMA_SPIT),
            "magma_cube" => Some(&Self::MAGMA_CUBE),
            "mangrove_boat" => Some(&Self::MANGROVE_BOAT),
            "mangrove_chest_boat" => Some(&Self::MANGROVE_CHEST_BOAT),
            "mannequin" => Some(&Self::MANNEQUIN),
            "marker" => Some(&Self::MARKER),
            "minecart" => Some(&Self::MINECART),
            "mooshroom" => Some(&Self::MOOSHROOM),
            "mule" => Some(&Self::MULE),
            "nautilus" => Some(&Self::NAUTILUS),
            "oak_boat" => Some(&Self::OAK_BOAT),
            "oak_chest_boat" => Some(&Self::OAK_CHEST_BOAT),
            "ocelot" => Some(&Self::OCELOT),
            "ominous_item_spawner" => Some(&Self::OMINOUS_ITEM_SPAWNER),
            "painting" => Some(&Self::PAINTING),
            "pale_oak_boat" => Some(&Self::PALE_OAK_BOAT),
            "pale_oak_chest_boat" => Some(&Self::PALE_OAK_CHEST_BOAT),
            "panda" => Some(&Self::PANDA),
            "parched" => Some(&Self::PARCHED),
            "parrot" => Some(&Self::PARROT),
            "phantom" => Some(&Self::PHANTOM),
            "pig" => Some(&Self::PIG),
            "piglin" => Some(&Self::PIGLIN),
            "piglin_brute" => Some(&Self::PIGLIN_BRUTE),
            "pillager" => Some(&Self::PILLAGER),
            "player" => Some(&Self::PLAYER),
            "polar_bear" => Some(&Self::POLAR_BEAR),
            "pufferfish" => Some(&Self::PUFFERFISH),
            "rabbit" => Some(&Self::RABBIT),
            "ravager" => Some(&Self::RAVAGER),
            "salmon" => Some(&Self::SALMON),
            "sheep" => Some(&Self::SHEEP),
            "shulker" => Some(&Self::SHULKER),
            "shulker_bullet" => Some(&Self::SHULKER_BULLET),
            "silverfish" => Some(&Self::SILVERFISH),
            "skeleton" => Some(&Self::SKELETON),
            "skeleton_horse" => Some(&Self::SKELETON_HORSE),
            "slime" => Some(&Self::SLIME),
            "small_fireball" => Some(&Self::SMALL_FIREBALL),
            "sniffer" => Some(&Self::SNIFFER),
            "snow_golem" => Some(&Self::SNOW_GOLEM),
            "snowball" => Some(&Self::SNOWBALL),
            "spawner_minecart" => Some(&Self::SPAWNER_MINECART),
            "spectral_arrow" => Some(&Self::SPECTRAL_ARROW),
            "spider" => Some(&Self::SPIDER),
            "splash_potion" => Some(&Self::SPLASH_POTION),
            "spruce_boat" => Some(&Self::SPRUCE_BOAT),
            "spruce_chest_boat" => Some(&Self::SPRUCE_CHEST_BOAT),
            "squid" => Some(&Self::SQUID),
            "stray" => Some(&Self::STRAY),
            "strider" => Some(&Self::STRIDER),
            "tadpole" => Some(&Self::TADPOLE),
            "text_display" => Some(&Self::TEXT_DISPLAY),
            "tnt" => Some(&Self::TNT),
            "tnt_minecart" => Some(&Self::TNT_MINECART),
            "trader_llama" => Some(&Self::TRADER_LLAMA),
            "trident" => Some(&Self::TRIDENT),
            "tropical_fish" => Some(&Self::TROPICAL_FISH),
            "turtle" => Some(&Self::TURTLE),
            "vex" => Some(&Self::VEX),
            "villager" => Some(&Self::VILLAGER),
            "vindicator" => Some(&Self::VINDICATOR),
            "wandering_trader" => Some(&Self::WANDERING_TRADER),
            "warden" => Some(&Self::WARDEN),
            "wind_charge" => Some(&Self::WIND_CHARGE),
            "witch" => Some(&Self::WITCH),
            "wither" => Some(&Self::WITHER),
            "wither_skeleton" => Some(&Self::WITHER_SKELETON),
            "wither_skull" => Some(&Self::WITHER_SKULL),
            "wolf" => Some(&Self::WOLF),
            "zoglin" => Some(&Self::ZOGLIN),
            "zombie" => Some(&Self::ZOMBIE),
            "zombie_horse" => Some(&Self::ZOMBIE_HORSE),
            "zombie_nautilus" => Some(&Self::ZOMBIE_NAUTILUS),
            "zombie_villager" => Some(&Self::ZOMBIE_VILLAGER),
            "zombified_piglin" => Some(&Self::ZOMBIFIED_PIGLIN),
            _ => None,
        }
    }
}

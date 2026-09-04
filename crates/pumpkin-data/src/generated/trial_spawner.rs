/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrialSpawnerConfigData {
    pub spawn_range: i32,
    pub total_mobs: f32,
    pub simultaneous_mobs: f32,
    pub total_mobs_added_per_player: f32,
    pub simultaneous_mobs_added_per_player: f32,
    pub ticks_between_spawn: i32,
    pub spawn_potentials: &'static [WeightedSpawnData],
    pub loot_tables_to_eject: &'static [WeightedLootTable],
    pub items_to_drop_when_ominous: Option<&'static str>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeightedSpawnData {
    pub entity_id: &'static str,
    pub equipment_loot_table: Option<&'static str>,
    pub equipment_slot_drop_chances: Option<f32>,
    pub weight: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeightedLootTable {
    pub loot_table: &'static str,
    pub weight: i32,
}
pub const TRIAL_CHAMBER_BREEZE_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 2f32,
    simultaneous_mobs: 1f32,
    total_mobs_added_per_player: 1f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:breeze",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_BREEZE_OMINOUS: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 4f32,
    simultaneous_mobs: 2f32,
    total_mobs_added_per_player: 1f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:breeze",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/key",
            weight: 3i32,
        },
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
            weight: 7i32,
        },
    ],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_MELEE_HUSK_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:husk",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_MELEE_HUSK_OMINOUS: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:husk",
        equipment_loot_table: Some("minecraft:equipment/trial_chamber_melee"),
        equipment_slot_drop_chances: Some(0f32),
        weight: 1i32,
    }],
    loot_tables_to_eject: &[
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/key",
            weight: 3i32,
        },
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
            weight: 7i32,
        },
    ],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_MELEE_SPIDER_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:spider",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_MELEE_SPIDER_OMINOUS: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 12f32,
    simultaneous_mobs: 4f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:spider",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/key",
            weight: 3i32,
        },
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
            weight: 7i32,
        },
    ],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_MELEE_ZOMBIE_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:zombie",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_MELEE_ZOMBIE_OMINOUS: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:zombie",
        equipment_loot_table: Some("minecraft:equipment/trial_chamber_melee"),
        equipment_slot_drop_chances: Some(0f32),
        weight: 1i32,
    }],
    loot_tables_to_eject: &[
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/key",
            weight: 3i32,
        },
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
            weight: 7i32,
        },
    ],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_RANGED_POISON_SKELETON_NORMAL: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 3f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:bogged",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_RANGED_POISON_SKELETON_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 3f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:bogged",
            equipment_loot_table: Some("minecraft:equipment/trial_chamber_ranged"),
            equipment_slot_drop_chances: Some(0f32),
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_RANGED_SKELETON_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:skeleton",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_RANGED_SKELETON_OMINOUS: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:skeleton",
        equipment_loot_table: Some("minecraft:equipment/trial_chamber_ranged"),
        equipment_slot_drop_chances: Some(0f32),
        weight: 1i32,
    }],
    loot_tables_to_eject: &[
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/key",
            weight: 3i32,
        },
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
            weight: 7i32,
        },
    ],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_RANGED_STRAY_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:stray",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_RANGED_STRAY_OMINOUS: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:stray",
        equipment_loot_table: Some("minecraft:equipment/trial_chamber_ranged"),
        equipment_slot_drop_chances: Some(0f32),
        weight: 1i32,
    }],
    loot_tables_to_eject: &[
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/key",
            weight: 3i32,
        },
        WeightedLootTable {
            loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
            weight: 7i32,
        },
    ],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_SLOW_RANGED_POISON_SKELETON_NORMAL: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 2f32,
        ticks_between_spawn: 160i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:bogged",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SLOW_RANGED_POISON_SKELETON_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 2f32,
        ticks_between_spawn: 160i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:bogged",
            equipment_loot_table: Some("minecraft:equipment/trial_chamber_ranged"),
            equipment_slot_drop_chances: Some(0f32),
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SLOW_RANGED_SKELETON_NORMAL: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 2f32,
        ticks_between_spawn: 160i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:skeleton",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SLOW_RANGED_SKELETON_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 2f32,
        ticks_between_spawn: 160i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:skeleton",
            equipment_loot_table: Some("minecraft:equipment/trial_chamber_ranged"),
            equipment_slot_drop_chances: Some(0f32),
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SLOW_RANGED_STRAY_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 4f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 2f32,
    ticks_between_spawn: 160i32,
    spawn_potentials: &[WeightedSpawnData {
        entity_id: "minecraft:stray",
        equipment_loot_table: None,
        equipment_slot_drop_chances: None,
        weight: 1i32,
    }],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_SLOW_RANGED_STRAY_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 2f32,
        ticks_between_spawn: 160i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:stray",
            equipment_loot_table: Some("minecraft:equipment/trial_chamber_ranged"),
            equipment_slot_drop_chances: Some(0f32),
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_BABY_ZOMBIE_NORMAL: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 2f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:zombie",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_BABY_ZOMBIE_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 2f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:zombie",
            equipment_loot_table: Some("minecraft:equipment/trial_chamber_melee"),
            equipment_slot_drop_chances: Some(0f32),
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_CAVE_SPIDER_NORMAL: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 3f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:cave_spider",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_CAVE_SPIDER_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 12f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:cave_spider",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_SILVERFISH_NORMAL: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 6f32,
        simultaneous_mobs: 3f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:silverfish",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_SILVERFISH_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 12f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[WeightedSpawnData {
            entity_id: "minecraft:silverfish",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        }],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
pub const TRIAL_CHAMBER_SMALL_MELEE_SLIME_NORMAL: TrialSpawnerConfigData = TrialSpawnerConfigData {
    spawn_range: 4i32,
    total_mobs: 6f32,
    simultaneous_mobs: 3f32,
    total_mobs_added_per_player: 2f32,
    simultaneous_mobs_added_per_player: 0.5f32,
    ticks_between_spawn: 20i32,
    spawn_potentials: &[
        WeightedSpawnData {
            entity_id: "minecraft:slime",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 3i32,
        },
        WeightedSpawnData {
            entity_id: "minecraft:slime",
            equipment_loot_table: None,
            equipment_slot_drop_chances: None,
            weight: 1i32,
        },
    ],
    loot_tables_to_eject: &[],
    items_to_drop_when_ominous: None,
};
pub const TRIAL_CHAMBER_SMALL_MELEE_SLIME_OMINOUS: TrialSpawnerConfigData =
    TrialSpawnerConfigData {
        spawn_range: 4i32,
        total_mobs: 12f32,
        simultaneous_mobs: 4f32,
        total_mobs_added_per_player: 2f32,
        simultaneous_mobs_added_per_player: 0.5f32,
        ticks_between_spawn: 20i32,
        spawn_potentials: &[
            WeightedSpawnData {
                entity_id: "minecraft:slime",
                equipment_loot_table: None,
                equipment_slot_drop_chances: None,
                weight: 3i32,
            },
            WeightedSpawnData {
                entity_id: "minecraft:slime",
                equipment_loot_table: None,
                equipment_slot_drop_chances: None,
                weight: 1i32,
            },
        ],
        loot_tables_to_eject: &[
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/key",
                weight: 3i32,
            },
            WeightedLootTable {
                loot_table: "minecraft:spawners/ominous/trial_chamber/consumables",
                weight: 7i32,
            },
        ],
        items_to_drop_when_ominous: None,
    };
#[must_use]
pub fn get_trial_spawner_config(key: &str) -> Option<&'static TrialSpawnerConfigData> {
    match key {
        "trial_chamber/breeze/normal" | "minecraft:trial_chamber/breeze/normal" => {
            Some(&TRIAL_CHAMBER_BREEZE_NORMAL)
        }
        "trial_chamber/breeze/ominous" | "minecraft:trial_chamber/breeze/ominous" => {
            Some(&TRIAL_CHAMBER_BREEZE_OMINOUS)
        }
        "trial_chamber/melee/husk/normal" | "minecraft:trial_chamber/melee/husk/normal" => {
            Some(&TRIAL_CHAMBER_MELEE_HUSK_NORMAL)
        }
        "trial_chamber/melee/husk/ominous" | "minecraft:trial_chamber/melee/husk/ominous" => {
            Some(&TRIAL_CHAMBER_MELEE_HUSK_OMINOUS)
        }
        "trial_chamber/melee/spider/normal" | "minecraft:trial_chamber/melee/spider/normal" => {
            Some(&TRIAL_CHAMBER_MELEE_SPIDER_NORMAL)
        }
        "trial_chamber/melee/spider/ominous" | "minecraft:trial_chamber/melee/spider/ominous" => {
            Some(&TRIAL_CHAMBER_MELEE_SPIDER_OMINOUS)
        }
        "trial_chamber/melee/zombie/normal" | "minecraft:trial_chamber/melee/zombie/normal" => {
            Some(&TRIAL_CHAMBER_MELEE_ZOMBIE_NORMAL)
        }
        "trial_chamber/melee/zombie/ominous" | "minecraft:trial_chamber/melee/zombie/ominous" => {
            Some(&TRIAL_CHAMBER_MELEE_ZOMBIE_OMINOUS)
        }
        "trial_chamber/ranged/poison_skeleton/normal"
        | "minecraft:trial_chamber/ranged/poison_skeleton/normal" => {
            Some(&TRIAL_CHAMBER_RANGED_POISON_SKELETON_NORMAL)
        }
        "trial_chamber/ranged/poison_skeleton/ominous"
        | "minecraft:trial_chamber/ranged/poison_skeleton/ominous" => {
            Some(&TRIAL_CHAMBER_RANGED_POISON_SKELETON_OMINOUS)
        }
        "trial_chamber/ranged/skeleton/normal"
        | "minecraft:trial_chamber/ranged/skeleton/normal" => {
            Some(&TRIAL_CHAMBER_RANGED_SKELETON_NORMAL)
        }
        "trial_chamber/ranged/skeleton/ominous"
        | "minecraft:trial_chamber/ranged/skeleton/ominous" => {
            Some(&TRIAL_CHAMBER_RANGED_SKELETON_OMINOUS)
        }
        "trial_chamber/ranged/stray/normal" | "minecraft:trial_chamber/ranged/stray/normal" => {
            Some(&TRIAL_CHAMBER_RANGED_STRAY_NORMAL)
        }
        "trial_chamber/ranged/stray/ominous" | "minecraft:trial_chamber/ranged/stray/ominous" => {
            Some(&TRIAL_CHAMBER_RANGED_STRAY_OMINOUS)
        }
        "trial_chamber/slow_ranged/poison_skeleton/normal"
        | "minecraft:trial_chamber/slow_ranged/poison_skeleton/normal" => {
            Some(&TRIAL_CHAMBER_SLOW_RANGED_POISON_SKELETON_NORMAL)
        }
        "trial_chamber/slow_ranged/poison_skeleton/ominous"
        | "minecraft:trial_chamber/slow_ranged/poison_skeleton/ominous" => {
            Some(&TRIAL_CHAMBER_SLOW_RANGED_POISON_SKELETON_OMINOUS)
        }
        "trial_chamber/slow_ranged/skeleton/normal"
        | "minecraft:trial_chamber/slow_ranged/skeleton/normal" => {
            Some(&TRIAL_CHAMBER_SLOW_RANGED_SKELETON_NORMAL)
        }
        "trial_chamber/slow_ranged/skeleton/ominous"
        | "minecraft:trial_chamber/slow_ranged/skeleton/ominous" => {
            Some(&TRIAL_CHAMBER_SLOW_RANGED_SKELETON_OMINOUS)
        }
        "trial_chamber/slow_ranged/stray/normal"
        | "minecraft:trial_chamber/slow_ranged/stray/normal" => {
            Some(&TRIAL_CHAMBER_SLOW_RANGED_STRAY_NORMAL)
        }
        "trial_chamber/slow_ranged/stray/ominous"
        | "minecraft:trial_chamber/slow_ranged/stray/ominous" => {
            Some(&TRIAL_CHAMBER_SLOW_RANGED_STRAY_OMINOUS)
        }
        "trial_chamber/small_melee/baby_zombie/normal"
        | "minecraft:trial_chamber/small_melee/baby_zombie/normal" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_BABY_ZOMBIE_NORMAL)
        }
        "trial_chamber/small_melee/baby_zombie/ominous"
        | "minecraft:trial_chamber/small_melee/baby_zombie/ominous" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_BABY_ZOMBIE_OMINOUS)
        }
        "trial_chamber/small_melee/cave_spider/normal"
        | "minecraft:trial_chamber/small_melee/cave_spider/normal" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_CAVE_SPIDER_NORMAL)
        }
        "trial_chamber/small_melee/cave_spider/ominous"
        | "minecraft:trial_chamber/small_melee/cave_spider/ominous" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_CAVE_SPIDER_OMINOUS)
        }
        "trial_chamber/small_melee/silverfish/normal"
        | "minecraft:trial_chamber/small_melee/silverfish/normal" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_SILVERFISH_NORMAL)
        }
        "trial_chamber/small_melee/silverfish/ominous"
        | "minecraft:trial_chamber/small_melee/silverfish/ominous" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_SILVERFISH_OMINOUS)
        }
        "trial_chamber/small_melee/slime/normal"
        | "minecraft:trial_chamber/small_melee/slime/normal" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_SLIME_NORMAL)
        }
        "trial_chamber/small_melee/slime/ominous"
        | "minecraft:trial_chamber/small_melee/slime/ominous" => {
            Some(&TRIAL_CHAMBER_SMALL_MELEE_SLIME_OMINOUS)
        }
        _ => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrialSpawnerKey {
    Breeze,
    MeleeHusk,
    MeleeSpider,
    MeleeZombie,
    RangedPoisonSkeleton,
    RangedSkeleton,
    RangedStray,
    SlowRangedPoisonSkeleton,
    SlowRangedSkeleton,
    SlowRangedStray,
    SmallMeleeBabyZombie,
    SmallMeleeCaveSpider,
    SmallMeleeSilverfish,
    SmallMeleeSlime,
}
impl TrialSpawnerKey {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Breeze => "trial_chamber/breeze",
            Self::MeleeHusk => "trial_chamber/melee/husk",
            Self::MeleeSpider => "trial_chamber/melee/spider",
            Self::MeleeZombie => "trial_chamber/melee/zombie",
            Self::RangedPoisonSkeleton => "trial_chamber/ranged/poison_skeleton",
            Self::RangedSkeleton => "trial_chamber/ranged/skeleton",
            Self::RangedStray => "trial_chamber/ranged/stray",
            Self::SlowRangedPoisonSkeleton => "trial_chamber/slow_ranged/poison_skeleton",
            Self::SlowRangedSkeleton => "trial_chamber/slow_ranged/skeleton",
            Self::SlowRangedStray => "trial_chamber/slow_ranged/stray",
            Self::SmallMeleeBabyZombie => "trial_chamber/small_melee/baby_zombie",
            Self::SmallMeleeCaveSpider => "trial_chamber/small_melee/cave_spider",
            Self::SmallMeleeSilverfish => "trial_chamber/small_melee/silverfish",
            Self::SmallMeleeSlime => "trial_chamber/small_melee/slime",
        }
    }
    #[must_use]
    pub const fn normal_config(&self) -> &'static TrialSpawnerConfigData {
        match self {
            Self::Breeze => &TRIAL_CHAMBER_BREEZE_NORMAL,
            Self::MeleeHusk => &TRIAL_CHAMBER_MELEE_HUSK_NORMAL,
            Self::MeleeSpider => &TRIAL_CHAMBER_MELEE_SPIDER_NORMAL,
            Self::MeleeZombie => &TRIAL_CHAMBER_MELEE_ZOMBIE_NORMAL,
            Self::RangedPoisonSkeleton => &TRIAL_CHAMBER_RANGED_POISON_SKELETON_NORMAL,
            Self::RangedSkeleton => &TRIAL_CHAMBER_RANGED_SKELETON_NORMAL,
            Self::RangedStray => &TRIAL_CHAMBER_RANGED_STRAY_NORMAL,
            Self::SlowRangedPoisonSkeleton => &TRIAL_CHAMBER_SLOW_RANGED_POISON_SKELETON_NORMAL,
            Self::SlowRangedSkeleton => &TRIAL_CHAMBER_SLOW_RANGED_SKELETON_NORMAL,
            Self::SlowRangedStray => &TRIAL_CHAMBER_SLOW_RANGED_STRAY_NORMAL,
            Self::SmallMeleeBabyZombie => &TRIAL_CHAMBER_SMALL_MELEE_BABY_ZOMBIE_NORMAL,
            Self::SmallMeleeCaveSpider => &TRIAL_CHAMBER_SMALL_MELEE_CAVE_SPIDER_NORMAL,
            Self::SmallMeleeSilverfish => &TRIAL_CHAMBER_SMALL_MELEE_SILVERFISH_NORMAL,
            Self::SmallMeleeSlime => &TRIAL_CHAMBER_SMALL_MELEE_SLIME_NORMAL,
        }
    }
    #[must_use]
    pub const fn ominous_config(&self) -> &'static TrialSpawnerConfigData {
        match self {
            Self::Breeze => &TRIAL_CHAMBER_BREEZE_OMINOUS,
            Self::MeleeHusk => &TRIAL_CHAMBER_MELEE_HUSK_OMINOUS,
            Self::MeleeSpider => &TRIAL_CHAMBER_MELEE_SPIDER_OMINOUS,
            Self::MeleeZombie => &TRIAL_CHAMBER_MELEE_ZOMBIE_OMINOUS,
            Self::RangedPoisonSkeleton => &TRIAL_CHAMBER_RANGED_POISON_SKELETON_OMINOUS,
            Self::RangedSkeleton => &TRIAL_CHAMBER_RANGED_SKELETON_OMINOUS,
            Self::RangedStray => &TRIAL_CHAMBER_RANGED_STRAY_OMINOUS,
            Self::SlowRangedPoisonSkeleton => &TRIAL_CHAMBER_SLOW_RANGED_POISON_SKELETON_OMINOUS,
            Self::SlowRangedSkeleton => &TRIAL_CHAMBER_SLOW_RANGED_SKELETON_OMINOUS,
            Self::SlowRangedStray => &TRIAL_CHAMBER_SLOW_RANGED_STRAY_OMINOUS,
            Self::SmallMeleeBabyZombie => &TRIAL_CHAMBER_SMALL_MELEE_BABY_ZOMBIE_OMINOUS,
            Self::SmallMeleeCaveSpider => &TRIAL_CHAMBER_SMALL_MELEE_CAVE_SPIDER_OMINOUS,
            Self::SmallMeleeSilverfish => &TRIAL_CHAMBER_SMALL_MELEE_SILVERFISH_OMINOUS,
            Self::SmallMeleeSlime => &TRIAL_CHAMBER_SMALL_MELEE_SLIME_OMINOUS,
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "trial_chamber/breeze" | "minecraft:trial_chamber/breeze" => Some(Self::Breeze),
            "trial_chamber/melee/husk" | "minecraft:trial_chamber/melee/husk" => {
                Some(Self::MeleeHusk)
            }
            "trial_chamber/melee/spider" | "minecraft:trial_chamber/melee/spider" => {
                Some(Self::MeleeSpider)
            }
            "trial_chamber/melee/zombie" | "minecraft:trial_chamber/melee/zombie" => {
                Some(Self::MeleeZombie)
            }
            "trial_chamber/ranged/poison_skeleton"
            | "minecraft:trial_chamber/ranged/poison_skeleton" => Some(Self::RangedPoisonSkeleton),
            "trial_chamber/ranged/skeleton" | "minecraft:trial_chamber/ranged/skeleton" => {
                Some(Self::RangedSkeleton)
            }
            "trial_chamber/ranged/stray" | "minecraft:trial_chamber/ranged/stray" => {
                Some(Self::RangedStray)
            }
            "trial_chamber/slow_ranged/poison_skeleton"
            | "minecraft:trial_chamber/slow_ranged/poison_skeleton" => {
                Some(Self::SlowRangedPoisonSkeleton)
            }
            "trial_chamber/slow_ranged/skeleton"
            | "minecraft:trial_chamber/slow_ranged/skeleton" => Some(Self::SlowRangedSkeleton),
            "trial_chamber/slow_ranged/stray" | "minecraft:trial_chamber/slow_ranged/stray" => {
                Some(Self::SlowRangedStray)
            }
            "trial_chamber/small_melee/baby_zombie"
            | "minecraft:trial_chamber/small_melee/baby_zombie" => Some(Self::SmallMeleeBabyZombie),
            "trial_chamber/small_melee/cave_spider"
            | "minecraft:trial_chamber/small_melee/cave_spider" => Some(Self::SmallMeleeCaveSpider),
            "trial_chamber/small_melee/silverfish"
            | "minecraft:trial_chamber/small_melee/silverfish" => Some(Self::SmallMeleeSilverfish),
            "trial_chamber/small_melee/slime" | "minecraft:trial_chamber/small_melee/slime" => {
                Some(Self::SmallMeleeSlime)
            }
            _ => None,
        }
    }
}

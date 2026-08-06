use crate::entity::player::Player;

#[derive(Debug, Clone)]
pub enum AdvancementTrigger {
    InventoryChanged,
    PlayerKilledEntity { entity_type_resource: String },
    EnterBlock { block_id: String },
    PlacedBlock { block_id: String },
    ConsumeItem { item_id: String },
    SleptInBed,
    EnterDimension { dimension: String },
    PlayerKilled,
    DeflectedDamage,
    LaunchedEyeOfEnder,
    GlowedSign,
    BredAnimal { parent_type: String },
    DealtOverkillDamage,
    SniperDuel,
    TwoBirdsOneArrow,
    Arbalistic,
    Bullseye,
    CuredZombieVillager,
}

impl Player {
    #[allow(clippy::collapsible_if, clippy::too_many_lines)]
    pub async fn trigger_advancement(&self, trigger: AdvancementTrigger) {
        use pumpkin_data::advancement::Advancement;
        use pumpkin_data::item::Item;

        match trigger {
            AdvancementTrigger::InventoryChanged => {
                if !self.has_advancement(Advancement::STORY_ROOT).await {
                    if self.has_item_in_inventory(&Item::CRAFTING_TABLE).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ROOT,
                            "crafting_table",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_MINE_STONE).await {
                    let stone_items = [
                        &Item::COBBLESTONE,
                        &Item::STONE,
                        &Item::ANDESITE,
                        &Item::DIORITE,
                        &Item::GRANITE,
                        &Item::DEEPSLATE,
                        &Item::COBBLED_DEEPSLATE,
                        &Item::BLACKSTONE,
                    ];
                    for item in stone_items {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::STORY_MINE_STONE,
                                "get_stone",
                            )
                            .await;
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::STORY_UPGRADE_TOOLS).await {
                    if self.has_item_in_inventory(&Item::STONE_PICKAXE).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_UPGRADE_TOOLS,
                            "stone_pickaxe",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_SMELT_IRON).await {
                    if self.has_item_in_inventory(&Item::IRON_INGOT).await {
                        self.trigger_advancement_criterion(Advancement::STORY_SMELT_IRON, "iron")
                            .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_IRON_TOOLS).await {
                    if self.has_item_in_inventory(&Item::IRON_PICKAXE).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_IRON_TOOLS,
                            "iron_pickaxe",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_MINE_DIAMOND).await {
                    if self.has_item_in_inventory(&Item::DIAMOND).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_MINE_DIAMOND,
                            "diamond",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_LAVA_BUCKET).await {
                    if self.has_item_in_inventory(&Item::LAVA_BUCKET).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_LAVA_BUCKET,
                            "lava_bucket",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_OBTAIN_ARMOR).await {
                    let armor = [
                        (&Item::IRON_HELMET, "iron_helmet"),
                        (&Item::IRON_CHESTPLATE, "iron_chestplate"),
                        (&Item::IRON_LEGGINGS, "iron_leggings"),
                        (&Item::IRON_BOOTS, "iron_boots"),
                    ];
                    for (item, criterion) in armor {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::STORY_OBTAIN_ARMOR,
                                criterion,
                            )
                            .await;
                        }
                    }
                }

                if !self.has_advancement(Advancement::STORY_SHINY_GEAR).await {
                    let armor = [
                        (&Item::DIAMOND_HELMET, "diamond_helmet"),
                        (&Item::DIAMOND_CHESTPLATE, "diamond_chestplate"),
                        (&Item::DIAMOND_LEGGINGS, "diamond_leggings"),
                        (&Item::DIAMOND_BOOTS, "diamond_boots"),
                    ];
                    for (item, criterion) in armor {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::STORY_SHINY_GEAR,
                                criterion,
                            )
                            .await;
                        }
                    }
                }

                if !self.has_advancement(Advancement::STORY_FORM_OBSIDIAN).await {
                    if self.has_item_in_inventory(&Item::OBSIDIAN).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_FORM_OBSIDIAN,
                            "obsidian",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_GET_WITHER_SKULL)
                    .await
                {
                    if self
                        .has_item_in_inventory(&Item::WITHER_SKELETON_SKULL)
                        .await
                    {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_GET_WITHER_SKULL,
                            "wither_skull",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_OBTAIN_ANCIENT_DEBRIS)
                    .await
                {
                    if self.has_item_in_inventory(&Item::ANCIENT_DEBRIS).await {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_OBTAIN_ANCIENT_DEBRIS,
                            "ancient_debris",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_OBTAIN_BLAZE_ROD)
                    .await
                {
                    if self.has_item_in_inventory(&Item::BLAZE_ROD).await {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_OBTAIN_BLAZE_ROD,
                            "blaze_rod",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_OBTAIN_CRYING_OBSIDIAN)
                    .await
                {
                    if self.has_item_in_inventory(&Item::CRYING_OBSIDIAN).await {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_OBTAIN_CRYING_OBSIDIAN,
                            "crying_obsidian",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_NETHERITE_ARMOR)
                    .await
                {
                    if self.has_item_in_inventory(&Item::NETHERITE_HELMET).await
                        && self
                            .has_item_in_inventory(&Item::NETHERITE_CHESTPLATE)
                            .await
                        && self.has_item_in_inventory(&Item::NETHERITE_LEGGINGS).await
                        && self.has_item_in_inventory(&Item::NETHERITE_BOOTS).await
                    {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_NETHERITE_ARMOR,
                            "netherite_armor",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::HUSBANDRY_FISHY_BUSINESS)
                    .await
                {
                    let fishes = [
                        (&Item::COD, "cod"),
                        (&Item::SALMON, "salmon"),
                        (&Item::PUFFERFISH, "pufferfish"),
                        (&Item::TROPICAL_FISH, "tropical_fish"),
                    ];
                    for (item, criterion) in fishes {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_FISHY_BUSINESS,
                                criterion,
                            )
                            .await;
                        }
                    }
                }

                if !self
                    .has_advancement(Advancement::HUSBANDRY_TACTICAL_FISHING)
                    .await
                {
                    let fish_buckets = [
                        (&Item::COD_BUCKET, "cod_bucket"),
                        (&Item::SALMON_BUCKET, "salmon_bucket"),
                        (&Item::PUFFERFISH_BUCKET, "pufferfish_bucket"),
                        (&Item::TROPICAL_FISH_BUCKET, "tropical_fish_bucket"),
                    ];
                    for (item, criterion) in fish_buckets {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_TACTICAL_FISHING,
                                criterion,
                            )
                            .await;
                        }
                    }
                }

                if !self
                    .has_advancement(Advancement::HUSBANDRY_AXOLOTL_IN_A_BUCKET)
                    .await
                {
                    if self.has_item_in_inventory(&Item::AXOLOTL_BUCKET).await {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_AXOLOTL_IN_A_BUCKET,
                            "axolotl_bucket",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::HUSBANDRY_TADPOLE_IN_A_BUCKET)
                    .await
                {
                    if self.has_item_in_inventory(&Item::TADPOLE_BUCKET).await {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_TADPOLE_IN_A_BUCKET,
                            "tadpole_bucket",
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::HUSBANDRY_OBTAIN_NETHERITE_HOE)
                    .await
                {
                    if self.has_item_in_inventory(&Item::NETHERITE_HOE).await {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_OBTAIN_NETHERITE_HOE,
                            "netherite_hoe",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::STORY_ENCHANT_ITEM).await {
                    let mut has_enchanted = false;
                    for item in &self.inventory().main_inventory {
                        let lock = item.lock().await;
                        if !lock.is_empty() && lock.has_enchantments() {
                            has_enchanted = true;
                            break;
                        }
                    }
                    if has_enchanted {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ENCHANT_ITEM,
                            "enchanted_item",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::NETHER_BREW_POTION).await {
                    let potions = [&Item::POTION, &Item::SPLASH_POTION, &Item::LINGERING_POTION];
                    for item in potions {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::NETHER_BREW_POTION,
                                "potion",
                            )
                            .await;
                            break;
                        }
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_CREATE_BEACON)
                    .await
                {
                    if self.has_item_in_inventory(&Item::BEACON).await {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_CREATE_BEACON,
                            "beacon",
                        )
                        .await;
                    }
                }
                if !self
                    .has_advancement(Advancement::NETHER_CREATE_FULL_BEACON)
                    .await
                {
                    if self.has_item_in_inventory(&Item::BEACON).await {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_CREATE_FULL_BEACON,
                            "beacon",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::END_ELYTRA).await {
                    if self.has_item_in_inventory(&Item::ELYTRA).await {
                        self.trigger_advancement_criterion(Advancement::END_ELYTRA, "elytra")
                            .await;
                    }
                }

                if !self.has_advancement(Advancement::END_DRAGON_EGG).await {
                    if self.has_item_in_inventory(&Item::DRAGON_EGG).await {
                        self.trigger_advancement_criterion(
                            Advancement::END_DRAGON_EGG,
                            "dragon_egg",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::END_DRAGON_BREATH).await {
                    if self.has_item_in_inventory(&Item::DRAGON_BREATH).await {
                        self.trigger_advancement_criterion(
                            Advancement::END_DRAGON_BREATH,
                            "dragon_breath",
                        )
                        .await;
                    }
                }

                if !self.has_advancement(Advancement::END_FIND_END_CITY).await {
                    let city_items = [&Item::SHULKER_SHELL, &Item::CHORUS_FRUIT];
                    for item in city_items {
                        if self.has_item_in_inventory(item).await {
                            self.trigger_advancement_criterion(
                                Advancement::END_FIND_END_CITY,
                                "in_city",
                            )
                            .await;
                            break;
                        }
                    }
                }

                if !self
                    .has_advancement(Advancement::NETHER_EXPLORE_NETHER)
                    .await
                {
                    let pos = self.position().to_block_pos();
                    let biome = self.world().level.get_rough_biome(&pos);
                    let biome_resource = format!("minecraft:{}", biome.registry_id);
                    let nether_biomes = [
                        "minecraft:basalt_deltas",
                        "minecraft:crimson_forest",
                        "minecraft:nether_wastes",
                        "minecraft:soul_sand_valley",
                        "minecraft:warped_forest",
                    ];
                    if nether_biomes.contains(&biome_resource.as_str()) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_EXPLORE_NETHER,
                            &biome_resource,
                        )
                        .await;
                    }
                }

                if !self
                    .has_advancement(Advancement::ADVENTURE_ADVENTURING_TIME)
                    .await
                {
                    let pos = self.position().to_block_pos();
                    let biome = self.world().level.get_rough_biome(&pos);
                    let biome_resource = format!("minecraft:{}", biome.registry_id);
                    let overworld_biomes = [
                        "minecraft:badlands",
                        "minecraft:bamboo_jungle",
                        "minecraft:beach",
                        "minecraft:birch_forest",
                        "minecraft:cherry_grove",
                        "minecraft:cold_ocean",
                        "minecraft:dark_forest",
                        "minecraft:deep_cold_ocean",
                        "minecraft:deep_dark",
                        "minecraft:deep_frozen_ocean",
                        "minecraft:deep_lukewarm_ocean",
                        "minecraft:deep_ocean",
                        "minecraft:desert",
                        "minecraft:dripstone_caves",
                        "minecraft:eroded_badlands",
                        "minecraft:flower_forest",
                        "minecraft:forest",
                        "minecraft:frozen_ocean",
                        "minecraft:frozen_peaks",
                        "minecraft:frozen_river",
                        "minecraft:grove",
                        "minecraft:ice_spikes",
                        "minecraft:jagged_peaks",
                        "minecraft:jungle",
                        "minecraft:lukewarm_ocean",
                        "minecraft:lush_caves",
                        "minecraft:mangrove_swamp",
                        "minecraft:meadow",
                        "minecraft:mushroom_fields",
                        "minecraft:ocean",
                        "minecraft:old_growth_birch_forest",
                        "minecraft:old_growth_pine_taiga",
                        "minecraft:old_growth_spruce_taiga",
                        "minecraft:pale_garden",
                        "minecraft:plains",
                        "minecraft:river",
                        "minecraft:savanna",
                        "minecraft:savanna_plateau",
                        "minecraft:snowy_beach",
                        "minecraft:snowy_plains",
                        "minecraft:snowy_slopes",
                        "minecraft:snowy_taiga",
                        "minecraft:sparse_jungle",
                        "minecraft:stony_peaks",
                        "minecraft:stony_shore",
                        "minecraft:sulfur_caves",
                        "minecraft:sunflower_plains",
                        "minecraft:swamp",
                        "minecraft:taiga",
                        "minecraft:warm_ocean",
                        "minecraft:windswept_forest",
                        "minecraft:windswept_gravelly_hills",
                        "minecraft:windswept_hills",
                        "minecraft:windswept_savanna",
                        "minecraft:wooded_badlands",
                    ];
                    if overworld_biomes.contains(&biome_resource.as_str()) {
                        self.trigger_advancement_criterion(
                            Advancement::ADVENTURE_ADVENTURING_TIME,
                            &biome_resource,
                        )
                        .await;
                    }
                }
            }
            AdvancementTrigger::PlayerKilledEntity {
                entity_type_resource,
            } => {
                self.trigger_advancement_criterion(
                    Advancement::ADVENTURE_KILL_A_MOB,
                    &entity_type_resource,
                )
                .await;
                self.trigger_advancement_criterion(
                    Advancement::ADVENTURE_KILL_ALL_MOBS,
                    &entity_type_resource,
                )
                .await;
                if !self.has_advancement(Advancement::ADVENTURE_ROOT).await {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_ROOT,
                        "killed_something",
                    )
                    .await;
                }
            }
            AdvancementTrigger::SleptInBed => {
                if !self
                    .has_advancement(Advancement::ADVENTURE_SLEEP_IN_BED)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_SLEEP_IN_BED,
                        "slept_in_bed",
                    )
                    .await;
                }
            }
            AdvancementTrigger::PlacedBlock { block_id } => {
                if !self
                    .has_advancement(Advancement::HUSBANDRY_PLANT_SEED)
                    .await
                {
                    let seed_blocks = [
                        ("minecraft:wheat", "wheat"),
                        ("minecraft:pumpkin_stem", "pumpkin_stem"),
                        ("minecraft:melon_stem", "melon_stem"),
                        ("minecraft:beetroots", "beetroots"),
                        ("minecraft:nether_wart", "nether_wart"),
                        ("minecraft:torchflower", "torchflower"),
                        ("minecraft:pitcher_pod", "pitcher_pod"),
                    ];
                    for (block, criterion) in seed_blocks {
                        if block_id == block {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_PLANT_SEED,
                                criterion,
                            )
                            .await;
                            break;
                        }
                    }
                }
            }
            AdvancementTrigger::EnterDimension { dimension } => {
                if dimension == "the_nether" {
                    if !self
                        .has_advancement(Advancement::STORY_ENTER_THE_NETHER)
                        .await
                    {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ENTER_THE_NETHER,
                            "entered_nether",
                        )
                        .await;
                    }
                    if !self.has_advancement(Advancement::NETHER_ROOT).await {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_ROOT,
                            "entered_nether",
                        )
                        .await;
                    }
                } else if dimension == "the_end" {
                    if !self.has_advancement(Advancement::STORY_ENTER_THE_END).await {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ENTER_THE_END,
                            "entered_end",
                        )
                        .await;
                    }
                    if !self.has_advancement(Advancement::END_ROOT).await {
                        self.trigger_advancement_criterion(Advancement::END_ROOT, "entered_end")
                            .await;
                    }
                }
            }
            AdvancementTrigger::ConsumeItem { item_id } => {
                if !self.has_advancement(Advancement::HUSBANDRY_ROOT).await {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_ROOT,
                        "consumed_item",
                    )
                    .await;
                }
                let food_name = item_id.strip_prefix("minecraft:").unwrap_or(&item_id);
                if Advancement::HUSBANDRY_BALANCED_DIET
                    .criteria
                    .contains(&food_name)
                {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_BALANCED_DIET,
                        food_name,
                    )
                    .await;
                }
            }
            AdvancementTrigger::PlayerKilled => {
                if !self.has_advancement(Advancement::ADVENTURE_ROOT).await {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_ROOT,
                        "killed_by_something",
                    )
                    .await;
                }
            }
            AdvancementTrigger::DeflectedDamage => {
                if !self.has_advancement(Advancement::STORY_DEFLECT_ARROW).await {
                    self.trigger_advancement_criterion(
                        Advancement::STORY_DEFLECT_ARROW,
                        "deflected_projectile",
                    )
                    .await;
                }
            }
            AdvancementTrigger::LaunchedEyeOfEnder => {
                if !self
                    .has_advancement(Advancement::STORY_FOLLOW_ENDER_EYE)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::STORY_FOLLOW_ENDER_EYE,
                        "in_stronghold",
                    )
                    .await;
                }
            }
            AdvancementTrigger::GlowedSign => {
                if !self
                    .has_advancement(Advancement::HUSBANDRY_MAKE_A_SIGN_GLOW)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_MAKE_A_SIGN_GLOW,
                        "make_a_sign_glow",
                    )
                    .await;
                }
            }
            AdvancementTrigger::BredAnimal { parent_type } => {
                self.trigger_advancement_criterion(Advancement::HUSBANDRY_BREED_AN_ANIMAL, "bred")
                    .await;
                self.trigger_advancement_criterion(
                    Advancement::HUSBANDRY_BRED_ALL_ANIMALS,
                    &parent_type,
                )
                .await;
            }
            AdvancementTrigger::EnterBlock { block_id: _ } => {}
            AdvancementTrigger::DealtOverkillDamage => {
                if !self
                    .has_advancement(Advancement::ADVENTURE_OVEROVERKILL)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_OVEROVERKILL,
                        "overoverkill",
                    )
                    .await;
                }
            }
            AdvancementTrigger::SniperDuel => {
                if !self
                    .has_advancement(Advancement::ADVENTURE_SNIPER_DUEL)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_SNIPER_DUEL,
                        "killed_skeleton",
                    )
                    .await;
                }
            }
            AdvancementTrigger::TwoBirdsOneArrow => {
                if !self
                    .has_advancement(Advancement::ADVENTURE_TWO_BIRDS_ONE_ARROW)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_TWO_BIRDS_ONE_ARROW,
                        "two_birds",
                    )
                    .await;
                }
            }
            AdvancementTrigger::Arbalistic => {
                if !self
                    .has_advancement(Advancement::ADVENTURE_ARBALISTIC)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_ARBALISTIC,
                        "arbalistic",
                    )
                    .await;
                }
            }
            AdvancementTrigger::Bullseye => {
                if !self.has_advancement(Advancement::ADVENTURE_BULLSEYE).await {
                    self.trigger_advancement_criterion(Advancement::ADVENTURE_BULLSEYE, "bullseye")
                        .await;
                }
            }
            AdvancementTrigger::CuredZombieVillager => {
                if !self
                    .has_advancement(Advancement::STORY_CURE_ZOMBIE_VILLAGER)
                    .await
                {
                    self.trigger_advancement_criterion(
                        Advancement::STORY_CURE_ZOMBIE_VILLAGER,
                        "cured_zombie",
                    )
                    .await;
                }
            }
        }
    }
}

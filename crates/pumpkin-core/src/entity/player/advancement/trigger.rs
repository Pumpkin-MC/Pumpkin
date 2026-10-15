use crate::entity::player::Player;

#[derive(Debug, Clone)]
pub enum AdvancementTrigger {
    InventoryChanged,
    PlayerKilledEntity { entity_type_resource: String },
    EnterBlock { block_id: String },
    PlacedBlock { block_id: String },
    ConsumeItem { item_id: String },
    SleptInBed,
    FishedItem { item_id: String },
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
    TradedWithVillager,
    UsedTotem,
    BrewedPotion,
    ConstructBeacon { level: i32 },
    TamedAnimal { animal_type: String, variant: Option<String> },
    AvoidVibration,
    KillMobNearSculkCatalyst,
    FallAfterExplosion,
    Blowback,
    RideEntityInLava { in_overworld: bool },
    UsingItem { item_id: String, target_entity: Option<String> },
    ShotCrossbow,
    ShotTrident,
    ChanneledLightning,
    Levitated,
    FallFromHeight,
    RideBoatWithGoat,
    UnderLockAndKey,
    Revaulting,
    CrafterCraftedCrafter,
    LeashedFrog { variant: String },
    AllayDeliverItemToPlayer,
    AllayDeliverCakeToNoteBlock,
    RemoveWolfArmor,
    RepairWolfArmor,
    FeedSnifflet,
    WaxOn,
    WaxOff,
    SafelyHarvestHoney,
    SilkTouchNest,
    KillAxolotlTarget,
    UhOh,
    ChargeRespawnAnchor,
    SummonWither,
    DistractPiglin,
    SummonDragon,
    NetherTravel,
    GenerateLoot { loot_table: String },
    EffectsChanged,
    WhosThePillagerNow,
    PlayJukeboxInMeadows,
    WalkOnPowderSnowWithLeatherBoots,
    LightenUp,
}

impl Player {
    #[allow(clippy::collapsible_if, clippy::too_many_lines)]
    pub fn trigger_advancement(&self, trigger: AdvancementTrigger) {
        use pumpkin_data::advancement::Advancement;
        use pumpkin_data::item::Item;

        match trigger {
            AdvancementTrigger::InventoryChanged => {
                if !self.has_advancement(Advancement::STORY_ROOT) {
                    if self.has_item_in_inventory(&Item::CRAFTING_TABLE) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ROOT,
                            "crafting_table",
                        );
                    }
                }

                if !self.has_advancement(Advancement::STORY_MINE_STONE) {
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
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::STORY_MINE_STONE,
                                "get_stone",
                            );
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::STORY_UPGRADE_TOOLS) {
                    if self.has_item_in_inventory(&Item::STONE_PICKAXE) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_UPGRADE_TOOLS,
                            "stone_pickaxe",
                        );
                    }
                }

                if !self.has_advancement(Advancement::STORY_SMELT_IRON) {
                    if self.has_item_in_inventory(&Item::IRON_INGOT) {
                        self.trigger_advancement_criterion(Advancement::STORY_SMELT_IRON, "iron");
                    }
                }

                if !self.has_advancement(Advancement::STORY_IRON_TOOLS) {
                    if self.has_item_in_inventory(&Item::IRON_PICKAXE) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_IRON_TOOLS,
                            "iron_pickaxe",
                        );
                    }
                }

                if !self.has_advancement(Advancement::STORY_MINE_DIAMOND) {
                    if self.has_item_in_inventory(&Item::DIAMOND) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_MINE_DIAMOND,
                            "diamond",
                        );
                    }
                }

                if !self.has_advancement(Advancement::STORY_LAVA_BUCKET) {
                    if self.has_item_in_inventory(&Item::LAVA_BUCKET) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_LAVA_BUCKET,
                            "lava_bucket",
                        );
                    }
                }

                if !self.has_advancement(Advancement::STORY_OBTAIN_ARMOR) {
                    let armor = [
                        (&Item::IRON_HELMET, "iron_helmet"),
                        (&Item::IRON_CHESTPLATE, "iron_chestplate"),
                        (&Item::IRON_LEGGINGS, "iron_leggings"),
                        (&Item::IRON_BOOTS, "iron_boots"),
                    ];
                    for (item, criterion) in armor {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::STORY_OBTAIN_ARMOR,
                                criterion,
                            );
                        }
                    }
                }

                if !self.has_advancement(Advancement::STORY_SHINY_GEAR) {
                    let armor = [
                        (&Item::DIAMOND_HELMET, "diamond_helmet"),
                        (&Item::DIAMOND_CHESTPLATE, "diamond_chestplate"),
                        (&Item::DIAMOND_LEGGINGS, "diamond_leggings"),
                        (&Item::DIAMOND_BOOTS, "diamond_boots"),
                    ];
                    for (item, criterion) in armor {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::STORY_SHINY_GEAR,
                                criterion,
                            );
                        }
                    }
                }

                if !self.has_advancement(Advancement::STORY_FORM_OBSIDIAN) {
                    if self.has_item_in_inventory(&Item::OBSIDIAN) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_FORM_OBSIDIAN,
                            "obsidian",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_GET_WITHER_SKULL) {
                    if self.has_item_in_inventory(&Item::WITHER_SKELETON_SKULL) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_GET_WITHER_SKULL,
                            "wither_skull",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_OBTAIN_ANCIENT_DEBRIS) {
                    if self.has_item_in_inventory(&Item::ANCIENT_DEBRIS) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_OBTAIN_ANCIENT_DEBRIS,
                            "ancient_debris",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_OBTAIN_BLAZE_ROD) {
                    if self.has_item_in_inventory(&Item::BLAZE_ROD) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_OBTAIN_BLAZE_ROD,
                            "blaze_rod",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_OBTAIN_CRYING_OBSIDIAN) {
                    if self.has_item_in_inventory(&Item::CRYING_OBSIDIAN) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_OBTAIN_CRYING_OBSIDIAN,
                            "crying_obsidian",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_NETHERITE_ARMOR) {
                    if self.has_item_in_inventory(&Item::NETHERITE_HELMET)
                        && self.has_item_in_inventory(&Item::NETHERITE_CHESTPLATE)
                        && self.has_item_in_inventory(&Item::NETHERITE_LEGGINGS)
                        && self.has_item_in_inventory(&Item::NETHERITE_BOOTS)
                    {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_NETHERITE_ARMOR,
                            "netherite_armor",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_FIND_FORTRESS) {
                    let fortress_items = [
                        &Item::NETHER_BRICKS,
                        &Item::NETHER_BRICK,
                        &Item::BLAZE_ROD,
                        &Item::WITHER_SKELETON_SKULL,
                    ];
                    for item in fortress_items {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::NETHER_FIND_FORTRESS,
                                "fortress",
                            );
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::NETHER_FIND_BASTION) {
                    let bastion_items = [
                        &Item::PIGLIN_BANNER_PATTERN,
                        &Item::SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE,
                        &Item::NETHERITE_UPGRADE_SMITHING_TEMPLATE,
                        &Item::GILDED_BLACKSTONE,
                    ];
                    for item in bastion_items {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::NETHER_FIND_BASTION,
                                "bastion",
                            );
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_TACTICAL_FISHING) {
                    let fish_buckets = [
                        (&Item::COD_BUCKET, "cod_bucket"),
                        (&Item::SALMON_BUCKET, "salmon_bucket"),
                        (&Item::PUFFERFISH_BUCKET, "pufferfish_bucket"),
                        (&Item::TROPICAL_FISH_BUCKET, "tropical_fish_bucket"),
                    ];
                    for (item, criterion) in fish_buckets {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_TACTICAL_FISHING,
                                criterion,
                            );
                        }
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_AXOLOTL_IN_A_BUCKET) {
                    if self.has_item_in_inventory(&Item::AXOLOTL_BUCKET) {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_AXOLOTL_IN_A_BUCKET,
                            "axolotl_bucket",
                        );
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_TADPOLE_IN_A_BUCKET) {
                    if self.has_item_in_inventory(&Item::TADPOLE_BUCKET) {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_TADPOLE_IN_A_BUCKET,
                            "tadpole_bucket",
                        );
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_OBTAIN_NETHERITE_HOE) {
                    if self.has_item_in_inventory(&Item::NETHERITE_HOE) {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_OBTAIN_NETHERITE_HOE,
                            "netherite_hoe",
                        );
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_FROGLIGHTS) {
                    if self.has_item_in_inventory(&Item::OCHRE_FROGLIGHT)
                        && self.has_item_in_inventory(&Item::PEARLESCENT_FROGLIGHT)
                        && self.has_item_in_inventory(&Item::VERDANT_FROGLIGHT)
                    {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_FROGLIGHTS,
                            "froglights",
                        );
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_OBTAIN_SNIFFER_EGG) {
                    if self.has_item_in_inventory(&Item::SNIFFER_EGG) {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_OBTAIN_SNIFFER_EGG,
                            "obtain_sniffer_egg",
                        );
                    }
                }

                if !self.has_advancement(Advancement::STORY_ENCHANT_ITEM) {
                    let has_enchanted = {
                        let main_inv = self
                            .inventory()
                            .main_inventory
                            .read()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        main_inv
                            .iter()
                            .any(|stack| !stack.is_empty() && stack.has_enchantments())
                    };
                    if has_enchanted {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ENCHANT_ITEM,
                            "enchanted_item",
                        );
                    }
                }

                if !self.has_advancement(Advancement::NETHER_BREW_POTION) {
                    let potions = [&Item::POTION, &Item::SPLASH_POTION, &Item::LINGERING_POTION];
                    for item in potions {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::NETHER_BREW_POTION,
                                "potion",
                            );
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::NETHER_CREATE_BEACON) {
                    if self.has_item_in_inventory(&Item::BEACON) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_CREATE_BEACON,
                            "beacon",
                        );
                    }
                }
                if !self.has_advancement(Advancement::NETHER_CREATE_FULL_BEACON) {
                    if self.has_item_in_inventory(&Item::BEACON) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_CREATE_FULL_BEACON,
                            "beacon",
                        );
                    }
                }

                if !self.has_advancement(Advancement::END_ELYTRA) {
                    if self.has_item_in_inventory(&Item::ELYTRA) {
                        self.trigger_advancement_criterion(Advancement::END_ELYTRA, "elytra");
                    }
                }

                if !self.has_advancement(Advancement::END_DRAGON_EGG) {
                    if self.has_item_in_inventory(&Item::DRAGON_EGG) {
                        self.trigger_advancement_criterion(
                            Advancement::END_DRAGON_EGG,
                            "dragon_egg",
                        );
                    }
                }

                if !self.has_advancement(Advancement::END_DRAGON_BREATH) {
                    if self.has_item_in_inventory(&Item::DRAGON_BREATH) {
                        self.trigger_advancement_criterion(
                            Advancement::END_DRAGON_BREATH,
                            "dragon_breath",
                        );
                    }
                }

                if !self.has_advancement(Advancement::END_FIND_END_CITY) {
                    let city_items = [&Item::SHULKER_SHELL, &Item::CHORUS_FRUIT];
                    for item in city_items {
                        if self.has_item_in_inventory(item) {
                            self.trigger_advancement_criterion(
                                Advancement::END_FIND_END_CITY,
                                "in_city",
                            );
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::NETHER_EXPLORE_NETHER) {
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
                        );
                    }
                }

                if !self.has_advancement(Advancement::ADVENTURE_ADVENTURING_TIME) {
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
                        );
                    }
                }
            }
            AdvancementTrigger::PlayerKilledEntity {
                entity_type_resource,
            } => {
                self.trigger_advancement_criterion(
                    Advancement::ADVENTURE_KILL_A_MOB,
                    &entity_type_resource,
                );
                self.trigger_advancement_criterion(
                    Advancement::ADVENTURE_KILL_ALL_MOBS,
                    &entity_type_resource,
                );
                if !self.has_advancement(Advancement::ADVENTURE_ROOT) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_ROOT,
                        "killed_something",
                    );
                }
                if entity_type_resource == "minecraft:ghast" {
                    if !self.has_advancement(Advancement::NETHER_RETURN_TO_SENDER) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_RETURN_TO_SENDER,
                            "killed_ghast",
                        );
                    }
                    if self.world().dimension == pumpkin_data::dimension::Dimension::OVERWORLD
                        && !self.has_advancement(Advancement::NETHER_UNEASY_ALLIANCE)
                    {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_UNEASY_ALLIANCE,
                            "killed_ghast",
                        );
                    }
                }
                if entity_type_resource == "minecraft:ender_dragon" {
                    if !self.has_advancement(Advancement::END_KILL_DRAGON) {
                        self.trigger_advancement_criterion(
                            Advancement::END_KILL_DRAGON,
                            "killed_dragon",
                        );
                    }
                }
                if entity_type_resource == "minecraft:blaze"
                    || entity_type_resource == "minecraft:wither_skeleton"
                {
                    if !self.has_advancement(Advancement::NETHER_FIND_FORTRESS) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_FIND_FORTRESS,
                            "fortress",
                        );
                    }
                }
                if entity_type_resource == "minecraft:piglin_brute" {
                    if !self.has_advancement(Advancement::NETHER_FIND_BASTION) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_FIND_BASTION,
                            "bastion",
                        );
                    }
                }
            }
            AdvancementTrigger::SleptInBed => {
                if !self.has_advancement(Advancement::ADVENTURE_SLEEP_IN_BED) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_SLEEP_IN_BED,
                        "slept_in_bed",
                    );
                }
            }
            AdvancementTrigger::FishedItem { item_id } => {
                if !self.has_advancement(Advancement::HUSBANDRY_FISHY_BUSINESS) {
                    let fishes = [
                        ("minecraft:cod", "cod"),
                        ("minecraft:salmon", "salmon"),
                        ("minecraft:pufferfish", "pufferfish"),
                        ("minecraft:tropical_fish", "tropical_fish"),
                    ];
                    for (fish, criterion) in fishes {
                        if item_id == fish {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_FISHY_BUSINESS,
                                criterion,
                            );
                            break;
                        }
                    }
                }
            }
            AdvancementTrigger::PlacedBlock { block_id } => {
                if !self.has_advancement(Advancement::HUSBANDRY_PLANT_SEED) {
                    let seed_blocks = [
                        ("minecraft:wheat", "wheat"),
                        ("minecraft:pumpkin_stem", "pumpkin_stem"),
                        ("minecraft:melon_stem", "melon_stem"),
                        ("minecraft:beetroots", "beetroots"),
                        ("minecraft:nether_wart", "nether_wart"),
                        ("minecraft:torchflower", "torchflower"),
                        ("minecraft:torchflower_crop", "torchflower"),
                        ("minecraft:pitcher_pod", "pitcher_pod"),
                        ("minecraft:pitcher_crop", "pitcher_pod"),
                    ];
                    for (block, criterion) in seed_blocks {
                        if block_id == block {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_PLANT_SEED,
                                criterion,
                            );
                            break;
                        }
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_PLANT_ANY_SNIFFER_SEED) {
                    if block_id == "minecraft:torchflower_crop" || block_id == "minecraft:torchflower" {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_PLANT_ANY_SNIFFER_SEED,
                            "torchflower",
                        );
                    } else if block_id == "minecraft:pitcher_crop" || block_id == "minecraft:pitcher_pod" {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_PLANT_ANY_SNIFFER_SEED,
                            "pitcher_pod",
                        );
                    }
                }

                if !self.has_advancement(Advancement::HUSBANDRY_PLACE_DRIED_GHAST_IN_WATER) {
                    if block_id == "minecraft:dried_ghast" {
                        self.trigger_advancement_criterion(
                            Advancement::HUSBANDRY_PLACE_DRIED_GHAST_IN_WATER,
                            "place_dried_ghast_in_water",
                        );
                    }
                }
            }
            AdvancementTrigger::EnterDimension { dimension } => {
                if dimension == "the_nether" {
                    if !self.has_advancement(Advancement::STORY_ENTER_THE_NETHER) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ENTER_THE_NETHER,
                            "entered_nether",
                        );
                    }
                    if !self.has_advancement(Advancement::NETHER_ROOT) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_ROOT,
                            "entered_nether",
                        );
                    }
                } else if dimension == "the_end" {
                    if !self.has_advancement(Advancement::STORY_ENTER_THE_END) {
                        self.trigger_advancement_criterion(
                            Advancement::STORY_ENTER_THE_END,
                            "entered_end",
                        );
                    }
                    if !self.has_advancement(Advancement::END_ROOT) {
                        self.trigger_advancement_criterion(Advancement::END_ROOT, "entered_end");
                    }
                }
            }
            AdvancementTrigger::ConsumeItem { item_id } => {
                if !self.has_advancement(Advancement::HUSBANDRY_ROOT) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_ROOT,
                        "consumed_item",
                    );
                }
                let food_name = item_id.strip_prefix("minecraft:").unwrap_or(&item_id);
                if Advancement::HUSBANDRY_BALANCED_DIET
                    .criteria
                    .contains(&food_name)
                {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_BALANCED_DIET,
                        food_name,
                    );
                }
            }
            AdvancementTrigger::PlayerKilled => {
                if !self.has_advancement(Advancement::ADVENTURE_ROOT) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_ROOT,
                        "killed_by_something",
                    );
                }
            }
            AdvancementTrigger::DeflectedDamage => {
                if !self.has_advancement(Advancement::STORY_DEFLECT_ARROW) {
                    self.trigger_advancement_criterion(
                        Advancement::STORY_DEFLECT_ARROW,
                        "deflected_projectile",
                    );
                }
            }
            AdvancementTrigger::LaunchedEyeOfEnder => {
                if !self.has_advancement(Advancement::STORY_FOLLOW_ENDER_EYE) {
                    self.trigger_advancement_criterion(
                        Advancement::STORY_FOLLOW_ENDER_EYE,
                        "in_stronghold",
                    );
                }
            }
            AdvancementTrigger::GlowedSign => {
                if !self.has_advancement(Advancement::HUSBANDRY_MAKE_A_SIGN_GLOW) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_MAKE_A_SIGN_GLOW,
                        "make_a_sign_glow",
                    );
                }
            }
            AdvancementTrigger::BredAnimal { parent_type } => {
                self.trigger_advancement_criterion(Advancement::HUSBANDRY_BREED_AN_ANIMAL, "bred");
                self.trigger_advancement_criterion(
                    Advancement::HUSBANDRY_BRED_ALL_ANIMALS,
                    &parent_type,
                );
            }
            AdvancementTrigger::EnterBlock { block_id } => {
                if (block_id == "minecraft:end_gateway" || block_id == "end_gateway")
                    && !self.has_advancement(Advancement::END_ENTER_END_GATEWAY)
                {
                    self.trigger_advancement_criterion(
                        Advancement::END_ENTER_END_GATEWAY,
                        "entered_end_gateway",
                    );
                }
            }
            AdvancementTrigger::DealtOverkillDamage => {
                if !self.has_advancement(Advancement::ADVENTURE_OVEROVERKILL) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_OVEROVERKILL,
                        "overoverkill",
                    );
                }
            }
            AdvancementTrigger::SniperDuel => {
                if !self.has_advancement(Advancement::ADVENTURE_SNIPER_DUEL) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_SNIPER_DUEL,
                        "killed_skeleton",
                    );
                }
            }
            AdvancementTrigger::TwoBirdsOneArrow => {
                if !self.has_advancement(Advancement::ADVENTURE_TWO_BIRDS_ONE_ARROW) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_TWO_BIRDS_ONE_ARROW,
                        "two_birds",
                    );
                }
            }
            AdvancementTrigger::Arbalistic => {
                if !self.has_advancement(Advancement::ADVENTURE_ARBALISTIC) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_ARBALISTIC,
                        "arbalistic",
                    );
                }
            }
            AdvancementTrigger::Bullseye => {
                if !self.has_advancement(Advancement::ADVENTURE_BULLSEYE) {
                    self.trigger_advancement_criterion(Advancement::ADVENTURE_BULLSEYE, "bullseye");
                }
            }
            AdvancementTrigger::CuredZombieVillager => {
                if !self.has_advancement(Advancement::STORY_CURE_ZOMBIE_VILLAGER) {
                    self.trigger_advancement_criterion(
                        Advancement::STORY_CURE_ZOMBIE_VILLAGER,
                        "cured_zombie",
                    );
                }
            }
            AdvancementTrigger::TradedWithVillager => {
                if !self.has_advancement(Advancement::ADVENTURE_TRADE) {
                    self.trigger_advancement_criterion(Advancement::ADVENTURE_TRADE, "traded");
                }
                if self.living_entity.entity.pos.load().y >= 319.0
                    && !self.has_advancement(Advancement::ADVENTURE_TRADE_AT_WORLD_HEIGHT)
                {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_TRADE_AT_WORLD_HEIGHT,
                        "trade_at_world_height",
                    );
                }
            }
            AdvancementTrigger::UsedTotem => {
                if !self.has_advancement(Advancement::ADVENTURE_TOTEM_OF_UNDYING) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_TOTEM_OF_UNDYING,
                        "used_totem",
                    );
                }
            }
            AdvancementTrigger::BrewedPotion => {
                if !self.has_advancement(Advancement::NETHER_BREW_POTION) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_BREW_POTION,
                        "potion",
                    );
                }
            }
            AdvancementTrigger::ConstructBeacon { level } => {
                if !self.has_advancement(Advancement::NETHER_CREATE_BEACON) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_CREATE_BEACON,
                        "beacon",
                    );
                }
                if level >= 4 && !self.has_advancement(Advancement::NETHER_CREATE_FULL_BEACON) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_CREATE_FULL_BEACON,
                        "beacon",
                    );
                }
            }
            AdvancementTrigger::TamedAnimal {
                animal_type,
                variant,
            } => {
                if !self.has_advancement(Advancement::HUSBANDRY_TAME_AN_ANIMAL) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_TAME_AN_ANIMAL,
                        "tamed_animal",
                    );
                }
                if let Some(variant_str) = variant {
                    if animal_type == "minecraft:cat" {
                        if !self.has_advancement(Advancement::HUSBANDRY_COMPLETE_CATALOGUE) {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_COMPLETE_CATALOGUE,
                                &variant_str,
                            );
                        }
                    } else if animal_type == "minecraft:wolf" {
                        if !self.has_advancement(Advancement::HUSBANDRY_WHOLE_PACK) {
                            self.trigger_advancement_criterion(
                                Advancement::HUSBANDRY_WHOLE_PACK,
                                &variant_str,
                            );
                        }
                    }
                }
            }
            AdvancementTrigger::AvoidVibration => {
                if !self.has_advancement(Advancement::ADVENTURE_AVOID_VIBRATION) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_AVOID_VIBRATION,
                        "avoid_vibration",
                    );
                }
            }
            AdvancementTrigger::KillMobNearSculkCatalyst => {
                if !self.has_advancement(Advancement::ADVENTURE_KILL_MOB_NEAR_SCULK_CATALYST) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_KILL_MOB_NEAR_SCULK_CATALYST,
                        "kill_mob_near_sculk_catalyst",
                    );
                }
            }
            AdvancementTrigger::FallAfterExplosion => {
                if !self.has_advancement(Advancement::ADVENTURE_WHO_NEEDS_ROCKETS) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_WHO_NEEDS_ROCKETS,
                        "who_needs_rockets",
                    );
                }
            }
            AdvancementTrigger::Blowback => {
                if !self.has_advancement(Advancement::ADVENTURE_BLOWBACK) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_BLOWBACK,
                        "blowback",
                    );
                }
            }
            AdvancementTrigger::RideEntityInLava { in_overworld } => {
                if !self.has_advancement(Advancement::NETHER_RIDE_STRIDER) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_RIDE_STRIDER,
                        "used_warped_fungus_on_a_stick",
                    );
                }
                if in_overworld
                    && !self.has_advancement(Advancement::NETHER_RIDE_STRIDER_IN_OVERWORLD_LAVA)
                {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_RIDE_STRIDER_IN_OVERWORLD_LAVA,
                        "ride_entity_distance",
                    );
                }
            }
            AdvancementTrigger::UsingItem {
                item_id,
                target_entity,
            } => {
                if item_id == "minecraft:spyglass" {
                    if let Some(target) = target_entity.as_deref() {
                        match target {
                            "minecraft:parrot" => {
                                if !self.has_advancement(Advancement::ADVENTURE_SPYGLASS_AT_PARROT) {
                                    self.trigger_advancement_criterion(
                                        Advancement::ADVENTURE_SPYGLASS_AT_PARROT,
                                        "spyglass_at_parrot",
                                    );
                                }
                            }
                            "minecraft:ghast" => {
                                if !self.has_advancement(Advancement::ADVENTURE_SPYGLASS_AT_GHAST) {
                                    self.trigger_advancement_criterion(
                                        Advancement::ADVENTURE_SPYGLASS_AT_GHAST,
                                        "spyglass_at_ghast",
                                    );
                                }
                            }
                            "minecraft:ender_dragon" => {
                                if !self.has_advancement(Advancement::ADVENTURE_SPYGLASS_AT_DRAGON) {
                                    self.trigger_advancement_criterion(
                                        Advancement::ADVENTURE_SPYGLASS_AT_DRAGON,
                                        "spyglass_at_dragon",
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            AdvancementTrigger::ShotCrossbow => {
                if !self.has_advancement(Advancement::ADVENTURE_OL_BETSY) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_OL_BETSY,
                        "shot_crossbow",
                    );
                }
            }
            AdvancementTrigger::ShotTrident => {
                if !self.has_advancement(Advancement::ADVENTURE_THROW_TRIDENT) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_THROW_TRIDENT,
                        "shot_trident",
                    );
                }
            }
            AdvancementTrigger::ChanneledLightning => {
                if !self.has_advancement(Advancement::ADVENTURE_VERY_VERY_FRIGHTENING) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_VERY_VERY_FRIGHTENING,
                        "struck_villager",
                    );
                }
            }
            AdvancementTrigger::Levitated => {
                if !self.has_advancement(Advancement::END_LEVITATE) {
                    self.trigger_advancement_criterion(
                        Advancement::END_LEVITATE,
                        "levitated",
                    );
                }
            }
            AdvancementTrigger::FallFromHeight => {
                if !self.has_advancement(Advancement::ADVENTURE_FALL_FROM_WORLD_HEIGHT) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_FALL_FROM_WORLD_HEIGHT,
                        "fall_from_world_height",
                    );
                }
            }
            AdvancementTrigger::RideBoatWithGoat => {
                if !self.has_advancement(Advancement::HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT,
                        "ride_a_boat_with_a_goat",
                    );
                }
            }
            AdvancementTrigger::UnderLockAndKey => {
                if !self.has_advancement(Advancement::ADVENTURE_UNDER_LOCK_AND_KEY) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_UNDER_LOCK_AND_KEY,
                        "under_lock_and_key",
                    );
                }
            }
            AdvancementTrigger::Revaulting => {
                if !self.has_advancement(Advancement::ADVENTURE_REVAULTING) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_REVAULTING,
                        "revaulting",
                    );
                }
            }
            AdvancementTrigger::CrafterCraftedCrafter => {
                if !self.has_advancement(Advancement::ADVENTURE_CRAFTERS_CRAFTING_CRAFTERS) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_CRAFTERS_CRAFTING_CRAFTERS,
                        "crafter_crafted_crafter",
                    );
                }
            }
            AdvancementTrigger::LeashedFrog { variant } => {
                self.trigger_advancement_criterion(
                    Advancement::HUSBANDRY_LEASH_ALL_FROG_VARIANTS,
                    &variant,
                );
            }
            AdvancementTrigger::AllayDeliverItemToPlayer => {
                if !self.has_advancement(Advancement::HUSBANDRY_ALLAY_DELIVER_ITEM_TO_PLAYER) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_ALLAY_DELIVER_ITEM_TO_PLAYER,
                        "allay_deliver_item_to_player",
                    );
                }
            }
            AdvancementTrigger::AllayDeliverCakeToNoteBlock => {
                if !self.has_advancement(Advancement::HUSBANDRY_ALLAY_DELIVER_CAKE_TO_NOTE_BLOCK) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_ALLAY_DELIVER_CAKE_TO_NOTE_BLOCK,
                        "allay_deliver_cake_to_note_block",
                    );
                }
            }
            AdvancementTrigger::RemoveWolfArmor => {
                if !self.has_advancement(Advancement::HUSBANDRY_REMOVE_WOLF_ARMOR) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_REMOVE_WOLF_ARMOR,
                        "remove_wolf_armor",
                    );
                }
            }
            AdvancementTrigger::RepairWolfArmor => {
                if !self.has_advancement(Advancement::HUSBANDRY_REPAIR_WOLF_ARMOR) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_REPAIR_WOLF_ARMOR,
                        "repair_wolf_armor",
                    );
                }
            }
            AdvancementTrigger::FeedSnifflet => {
                if !self.has_advancement(Advancement::HUSBANDRY_FEED_SNIFFLET) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_FEED_SNIFFLET,
                        "feed_snifflet",
                    );
                }
            }
            AdvancementTrigger::WaxOn => {
                if !self.has_advancement(Advancement::HUSBANDRY_WAX_ON) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_WAX_ON,
                        "wax_on",
                    );
                }
            }
            AdvancementTrigger::WaxOff => {
                if !self.has_advancement(Advancement::HUSBANDRY_WAX_OFF) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_WAX_OFF,
                        "wax_off",
                    );
                }
            }
            AdvancementTrigger::SafelyHarvestHoney => {
                if !self.has_advancement(Advancement::HUSBANDRY_SAFELY_HARVEST_HONEY) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_SAFELY_HARVEST_HONEY,
                        "safely_harvest_honey",
                    );
                }
            }
            AdvancementTrigger::SilkTouchNest => {
                if !self.has_advancement(Advancement::HUSBANDRY_SILK_TOUCH_NEST) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_SILK_TOUCH_NEST,
                        "silk_touch_nest",
                    );
                }
            }
            AdvancementTrigger::KillAxolotlTarget => {
                if !self.has_advancement(Advancement::HUSBANDRY_KILL_AXOLOTL_TARGET) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_KILL_AXOLOTL_TARGET,
                        "kill_axolotl_target",
                    );
                }
            }
            AdvancementTrigger::UhOh => {
                if !self.has_advancement(Advancement::HUSBANDRY_UH_OH) {
                    self.trigger_advancement_criterion(
                        Advancement::HUSBANDRY_UH_OH,
                        "give_tnt_directly",
                    );
                }
            }
            AdvancementTrigger::ChargeRespawnAnchor => {
                if !self.has_advancement(Advancement::NETHER_CHARGE_RESPAWN_ANCHOR) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_CHARGE_RESPAWN_ANCHOR,
                        "charge_respawn_anchor",
                    );
                }
            }
            AdvancementTrigger::SummonWither => {
                if !self.has_advancement(Advancement::NETHER_SUMMON_WITHER) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_SUMMON_WITHER,
                        "summoned",
                    );
                }
            }
            AdvancementTrigger::DistractPiglin => {
                if !self.has_advancement(Advancement::NETHER_DISTRACT_PIGLIN) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_DISTRACT_PIGLIN,
                        "distract_piglin",
                    );
                }
            }
            AdvancementTrigger::SummonDragon => {
                if !self.has_advancement(Advancement::END_RESPAWN_DRAGON) {
                    self.trigger_advancement_criterion(
                        Advancement::END_RESPAWN_DRAGON,
                        "summoned_dragon",
                    );
                }
            }
            AdvancementTrigger::NetherTravel => {
                if !self.has_advancement(Advancement::NETHER_FAST_TRAVEL) {
                    self.trigger_advancement_criterion(
                        Advancement::NETHER_FAST_TRAVEL,
                        "travelled",
                    );
                }
            }
            AdvancementTrigger::GenerateLoot { loot_table } => {
                let bastion_tables = [
                    ("minecraft:chests/bastion_other", "loot_bastion_other"),
                    ("minecraft:chests/bastion_treasure", "loot_bastion_treasure"),
                    ("minecraft:chests/bastion_hoglin_stable", "loot_bastion_hoglin_stable"),
                    ("minecraft:chests/bastion_bridge", "loot_bastion_bridge"),
                ];
                for (table, criterion) in bastion_tables {
                    let short_name = table.strip_prefix("minecraft:").unwrap_or(table);
                    if loot_table == table || loot_table == short_name {
                        if !self.has_advancement(Advancement::NETHER_LOOT_BASTION) {
                            self.trigger_advancement_criterion(
                                Advancement::NETHER_LOOT_BASTION,
                                criterion,
                            );
                        }
                        if !self.has_advancement(Advancement::NETHER_FIND_BASTION) {
                            self.trigger_advancement_criterion(
                                Advancement::NETHER_FIND_BASTION,
                                "bastion",
                            );
                        }
                        break;
                    }
                }
            }
            AdvancementTrigger::EffectsChanged => {
                use pumpkin_data::effect::StatusEffect;

                if !self.has_advancement(Advancement::NETHER_ALL_POTIONS) {
                    let potion_effects = [
                        &StatusEffect::FIRE_RESISTANCE,
                        &StatusEffect::INFESTED,
                        &StatusEffect::INVISIBILITY,
                        &StatusEffect::JUMP_BOOST,
                        &StatusEffect::NIGHT_VISION,
                        &StatusEffect::OOZING,
                        &StatusEffect::POISON,
                        &StatusEffect::REGENERATION,
                        &StatusEffect::RESISTANCE,
                        &StatusEffect::SLOW_FALLING,
                        &StatusEffect::SLOWNESS,
                        &StatusEffect::SPEED,
                        &StatusEffect::STRENGTH,
                        &StatusEffect::WATER_BREATHING,
                        &StatusEffect::WEAKNESS,
                        &StatusEffect::WEAVING,
                        &StatusEffect::WIND_CHARGED,
                    ];
                    if potion_effects.iter().all(|eff| self.has_effect(eff)) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_ALL_POTIONS,
                            "all_effects",
                        );
                    }
                }
                if !self.has_advancement(Advancement::NETHER_ALL_EFFECTS) {
                    let all_effects = [
                        &StatusEffect::ABSORPTION,
                        &StatusEffect::BAD_OMEN,
                        &StatusEffect::BLINDNESS,
                        &StatusEffect::BREATH_OF_THE_NAUTILUS,
                        &StatusEffect::CONDUIT_POWER,
                        &StatusEffect::DARKNESS,
                        &StatusEffect::DOLPHINS_GRACE,
                        &StatusEffect::FIRE_RESISTANCE,
                        &StatusEffect::GLOWING,
                        &StatusEffect::HASTE,
                        &StatusEffect::HERO_OF_THE_VILLAGE,
                        &StatusEffect::HUNGER,
                        &StatusEffect::INFESTED,
                        &StatusEffect::INVISIBILITY,
                        &StatusEffect::JUMP_BOOST,
                        &StatusEffect::LEVITATION,
                        &StatusEffect::MINING_FATIGUE,
                        &StatusEffect::NAUSEA,
                        &StatusEffect::NIGHT_VISION,
                        &StatusEffect::OOZING,
                        &StatusEffect::POISON,
                        &StatusEffect::RAID_OMEN,
                        &StatusEffect::REGENERATION,
                        &StatusEffect::RESISTANCE,
                        &StatusEffect::SLOW_FALLING,
                        &StatusEffect::SLOWNESS,
                        &StatusEffect::SPEED,
                        &StatusEffect::STRENGTH,
                        &StatusEffect::TRIAL_OMEN,
                        &StatusEffect::WATER_BREATHING,
                        &StatusEffect::WEAKNESS,
                        &StatusEffect::WEAVING,
                        &StatusEffect::WIND_CHARGED,
                        &StatusEffect::WITHER,
                    ];
                    if all_effects.iter().all(|eff| self.has_effect(eff)) {
                        self.trigger_advancement_criterion(
                            Advancement::NETHER_ALL_EFFECTS,
                            "all_effects",
                        );
                    }
                }
            }
            AdvancementTrigger::WhosThePillagerNow => {
                if !self.has_advancement(Advancement::ADVENTURE_WHOS_THE_PILLAGER_NOW) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_WHOS_THE_PILLAGER_NOW,
                        "kill_pillager",
                    );
                }
            }
            AdvancementTrigger::PlayJukeboxInMeadows => {
                if !self.has_advancement(Advancement::ADVENTURE_PLAY_JUKEBOX_IN_MEADOWS) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_PLAY_JUKEBOX_IN_MEADOWS,
                        "play_jukebox_in_meadows",
                    );
                }
            }
            AdvancementTrigger::WalkOnPowderSnowWithLeatherBoots => {
                if !self.has_advancement(Advancement::ADVENTURE_WALK_ON_POWDER_SNOW_WITH_LEATHER_BOOTS) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_WALK_ON_POWDER_SNOW_WITH_LEATHER_BOOTS,
                        "walk_on_powder_snow_with_leather_boots",
                    );
                }
            }
            AdvancementTrigger::LightenUp => {
                if !self.has_advancement(Advancement::ADVENTURE_LIGHTEN_UP) {
                    self.trigger_advancement_criterion(
                        Advancement::ADVENTURE_LIGHTEN_UP,
                        "lighten_up",
                    );
                }
            }
        }
    }
}

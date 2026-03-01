/* This file is generated. Do not edit manually. */
pub struct Advancement {
    pub id: ResourceLocation,
    pub parent: Option<ResourceLocation>,
    pub send_telemetry: bool,
    pub display_name: Option<TextComponent>,
}
impl Advancement {
    pub const ADVENTURE_ADVENTURING_TIME: Self = Self {
        id: "adventure/adventuring_time",
        parent: Some("minecraft:adventure/sleep_in_bed"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_ARBALISTIC: Self = Self {
        id: "adventure/arbalistic",
        parent: Some("minecraft:adventure/ol_betsy"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_AVOID_VIBRATION: Self = Self {
        id: "adventure/avoid_vibration",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_BLOWBACK: Self = Self {
        id: "adventure/blowback",
        parent: Some("minecraft:adventure/minecraft_trials_edition"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_BRUSH_ARMADILLO: Self = Self {
        id: "adventure/brush_armadillo",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_BULLSEYE: Self = Self {
        id: "adventure/bullseye",
        parent: Some("minecraft:adventure/shoot_arrow"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_CRAFT_DECORATED_POT_USING_ONLY_SHERDS: Self = Self {
        id: "adventure/craft_decorated_pot_using_only_sherds",
        parent: Some("minecraft:adventure/salvage_sherd"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_CRAFTERS_CRAFTING_CRAFTERS: Self = Self {
        id: "adventure/crafters_crafting_crafters",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_FALL_FROM_WORLD_HEIGHT: Self = Self {
        id: "adventure/fall_from_world_height",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_HEART_TRANSPLANTER: Self = Self {
        id: "adventure/heart_transplanter",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_HERO_OF_THE_VILLAGE: Self = Self {
        id: "adventure/hero_of_the_village",
        parent: Some("minecraft:adventure/voluntary_exile"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_HONEY_BLOCK_SLIDE: Self = Self {
        id: "adventure/honey_block_slide",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_KILL_A_MOB: Self = Self {
        id: "adventure/kill_a_mob",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_KILL_ALL_MOBS: Self = Self {
        id: "adventure/kill_all_mobs",
        parent: Some("minecraft:adventure/kill_a_mob"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_KILL_MOB_NEAR_SCULK_CATALYST: Self = Self {
        id: "adventure/kill_mob_near_sculk_catalyst",
        parent: Some("minecraft:adventure/kill_a_mob"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_LIGHTEN_UP: Self = Self {
        id: "adventure/lighten_up",
        parent: Some("minecraft:adventure/minecraft_trials_edition"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_LIGHTNING_ROD_WITH_VILLAGER_NO_FIRE: Self = Self {
        id: "adventure/lightning_rod_with_villager_no_fire",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_MINECRAFT_TRIALS_EDITION: Self = Self {
        id: "adventure/minecraft_trials_edition",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_OL_BETSY: Self = Self {
        id: "adventure/ol_betsy",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_OVEROVERKILL: Self = Self {
        id: "adventure/overoverkill",
        parent: Some("minecraft:adventure/minecraft_trials_edition"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_PLAY_JUKEBOX_IN_MEADOWS: Self = Self {
        id: "adventure/play_jukebox_in_meadows",
        parent: Some("minecraft:adventure/sleep_in_bed"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_READ_POWER_OF_CHISELED_BOOKSHELF: Self = Self {
        id: "adventure/read_power_of_chiseled_bookshelf",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_REVAULTING: Self = Self {
        id: "adventure/revaulting",
        parent: Some("minecraft:adventure/under_lock_and_key"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_ROOT: Self = Self {
        id: "adventure/root",
        parent: None,
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SALVAGE_SHERD: Self = Self {
        id: "adventure/salvage_sherd",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SHOOT_ARROW: Self = Self {
        id: "adventure/shoot_arrow",
        parent: Some("minecraft:adventure/kill_a_mob"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SLEEP_IN_BED: Self = Self {
        id: "adventure/sleep_in_bed",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SNIPER_DUEL: Self = Self {
        id: "adventure/sniper_duel",
        parent: Some("minecraft:adventure/shoot_arrow"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SPEAR_MANY_MOBS: Self = Self {
        id: "adventure/spear_many_mobs",
        parent: Some("minecraft:adventure/kill_a_mob"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SPYGLASS_AT_DRAGON: Self = Self {
        id: "adventure/spyglass_at_dragon",
        parent: Some("minecraft:adventure/spyglass_at_ghast"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SPYGLASS_AT_GHAST: Self = Self {
        id: "adventure/spyglass_at_ghast",
        parent: Some("minecraft:adventure/spyglass_at_parrot"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SPYGLASS_AT_PARROT: Self = Self {
        id: "adventure/spyglass_at_parrot",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_SUMMON_IRON_GOLEM: Self = Self {
        id: "adventure/summon_iron_golem",
        parent: Some("minecraft:adventure/trade"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_THROW_TRIDENT: Self = Self {
        id: "adventure/throw_trident",
        parent: Some("minecraft:adventure/kill_a_mob"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_TOTEM_OF_UNDYING: Self = Self {
        id: "adventure/totem_of_undying",
        parent: Some("minecraft:adventure/kill_a_mob"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_TRADE: Self = Self {
        id: "adventure/trade",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_TRADE_AT_WORLD_HEIGHT: Self = Self {
        id: "adventure/trade_at_world_height",
        parent: Some("minecraft:adventure/trade"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_TRIM_WITH_ALL_EXCLUSIVE_ARMOR_PATTERNS: Self = Self {
        id: "adventure/trim_with_all_exclusive_armor_patterns",
        parent: Some("minecraft:adventure/trim_with_any_armor_pattern"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_TRIM_WITH_ANY_ARMOR_PATTERN: Self = Self {
        id: "adventure/trim_with_any_armor_pattern",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_TWO_BIRDS_ONE_ARROW: Self = Self {
        id: "adventure/two_birds_one_arrow",
        parent: Some("minecraft:adventure/ol_betsy"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_UNDER_LOCK_AND_KEY: Self = Self {
        id: "adventure/under_lock_and_key",
        parent: Some("minecraft:adventure/minecraft_trials_edition"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_USE_LODESTONE: Self = Self {
        id: "adventure/use_lodestone",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_VERY_VERY_FRIGHTENING: Self = Self {
        id: "adventure/very_very_frightening",
        parent: Some("minecraft:adventure/throw_trident"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_VOLUNTARY_EXILE: Self = Self {
        id: "adventure/voluntary_exile",
        parent: Some("minecraft:adventure/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_WALK_ON_POWDER_SNOW_WITH_LEATHER_BOOTS: Self = Self {
        id: "adventure/walk_on_powder_snow_with_leather_boots",
        parent: Some("minecraft:adventure/sleep_in_bed"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_WHO_NEEDS_ROCKETS: Self = Self {
        id: "adventure/who_needs_rockets",
        parent: Some("minecraft:adventure/minecraft_trials_edition"),
        send_telemetry: false,
        display_name: None,
    };
    pub const ADVENTURE_WHOS_THE_PILLAGER_NOW: Self = Self {
        id: "adventure/whos_the_pillager_now",
        parent: Some("minecraft:adventure/ol_betsy"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_DRAGON_BREATH: Self = Self {
        id: "end/dragon_breath",
        parent: Some("minecraft:end/kill_dragon"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_DRAGON_EGG: Self = Self {
        id: "end/dragon_egg",
        parent: Some("minecraft:end/kill_dragon"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_ELYTRA: Self = Self {
        id: "end/elytra",
        parent: Some("minecraft:end/find_end_city"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_ENTER_END_GATEWAY: Self = Self {
        id: "end/enter_end_gateway",
        parent: Some("minecraft:end/kill_dragon"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_FIND_END_CITY: Self = Self {
        id: "end/find_end_city",
        parent: Some("minecraft:end/enter_end_gateway"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_KILL_DRAGON: Self = Self {
        id: "end/kill_dragon",
        parent: Some("minecraft:end/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_LEVITATE: Self = Self {
        id: "end/levitate",
        parent: Some("minecraft:end/find_end_city"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_RESPAWN_DRAGON: Self = Self {
        id: "end/respawn_dragon",
        parent: Some("minecraft:end/kill_dragon"),
        send_telemetry: false,
        display_name: None,
    };
    pub const END_ROOT: Self = Self {
        id: "end/root",
        parent: None,
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_ALLAY_DELIVER_CAKE_TO_NOTE_BLOCK: Self = Self {
        id: "husbandry/allay_deliver_cake_to_note_block",
        parent: Some("minecraft:husbandry/allay_deliver_item_to_player"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_ALLAY_DELIVER_ITEM_TO_PLAYER: Self = Self {
        id: "husbandry/allay_deliver_item_to_player",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_AXOLOTL_IN_A_BUCKET: Self = Self {
        id: "husbandry/axolotl_in_a_bucket",
        parent: Some("minecraft:husbandry/tactical_fishing"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_BALANCED_DIET: Self = Self {
        id: "husbandry/balanced_diet",
        parent: Some("minecraft:husbandry/plant_seed"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_BRED_ALL_ANIMALS: Self = Self {
        id: "husbandry/bred_all_animals",
        parent: Some("minecraft:husbandry/breed_an_animal"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_BREED_AN_ANIMAL: Self = Self {
        id: "husbandry/breed_an_animal",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_COMPLETE_CATALOGUE: Self = Self {
        id: "husbandry/complete_catalogue",
        parent: Some("minecraft:husbandry/tame_an_animal"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_FEED_SNIFFLET: Self = Self {
        id: "husbandry/feed_snifflet",
        parent: Some("minecraft:husbandry/obtain_sniffer_egg"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_FISHY_BUSINESS: Self = Self {
        id: "husbandry/fishy_business",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_FROGLIGHTS: Self = Self {
        id: "husbandry/froglights",
        parent: Some("minecraft:husbandry/leash_all_frog_variants"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_KILL_AXOLOTL_TARGET: Self = Self {
        id: "husbandry/kill_axolotl_target",
        parent: Some("minecraft:husbandry/axolotl_in_a_bucket"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_LEASH_ALL_FROG_VARIANTS: Self = Self {
        id: "husbandry/leash_all_frog_variants",
        parent: Some("minecraft:husbandry/tadpole_in_a_bucket"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_MAKE_A_SIGN_GLOW: Self = Self {
        id: "husbandry/make_a_sign_glow",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_OBTAIN_NETHERITE_HOE: Self = Self {
        id: "husbandry/obtain_netherite_hoe",
        parent: Some("minecraft:husbandry/plant_seed"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_OBTAIN_SNIFFER_EGG: Self = Self {
        id: "husbandry/obtain_sniffer_egg",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_PLACE_DRIED_GHAST_IN_WATER: Self = Self {
        id: "husbandry/place_dried_ghast_in_water",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_PLANT_ANY_SNIFFER_SEED: Self = Self {
        id: "husbandry/plant_any_sniffer_seed",
        parent: Some("minecraft:husbandry/feed_snifflet"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_PLANT_SEED: Self = Self {
        id: "husbandry/plant_seed",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_REMOVE_WOLF_ARMOR: Self = Self {
        id: "husbandry/remove_wolf_armor",
        parent: Some("minecraft:husbandry/tame_an_animal"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_REPAIR_WOLF_ARMOR: Self = Self {
        id: "husbandry/repair_wolf_armor",
        parent: Some("minecraft:husbandry/tame_an_animal"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT: Self = Self {
        id: "husbandry/ride_a_boat_with_a_goat",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_ROOT: Self = Self {
        id: "husbandry/root",
        parent: None,
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_SAFELY_HARVEST_HONEY: Self = Self {
        id: "husbandry/safely_harvest_honey",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_SILK_TOUCH_NEST: Self = Self {
        id: "husbandry/silk_touch_nest",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_TACTICAL_FISHING: Self = Self {
        id: "husbandry/tactical_fishing",
        parent: Some("minecraft:husbandry/fishy_business"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_TADPOLE_IN_A_BUCKET: Self = Self {
        id: "husbandry/tadpole_in_a_bucket",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_TAME_AN_ANIMAL: Self = Self {
        id: "husbandry/tame_an_animal",
        parent: Some("minecraft:husbandry/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_WAX_OFF: Self = Self {
        id: "husbandry/wax_off",
        parent: Some("minecraft:husbandry/wax_on"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_WAX_ON: Self = Self {
        id: "husbandry/wax_on",
        parent: Some("minecraft:husbandry/safely_harvest_honey"),
        send_telemetry: false,
        display_name: None,
    };
    pub const HUSBANDRY_WHOLE_PACK: Self = Self {
        id: "husbandry/whole_pack",
        parent: Some("minecraft:husbandry/tame_an_animal"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_ALL_EFFECTS: Self = Self {
        id: "nether/all_effects",
        parent: Some("minecraft:nether/all_potions"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_ALL_POTIONS: Self = Self {
        id: "nether/all_potions",
        parent: Some("minecraft:nether/brew_potion"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_BREW_POTION: Self = Self {
        id: "nether/brew_potion",
        parent: Some("minecraft:nether/obtain_blaze_rod"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_CHARGE_RESPAWN_ANCHOR: Self = Self {
        id: "nether/charge_respawn_anchor",
        parent: Some("minecraft:nether/obtain_crying_obsidian"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_CREATE_BEACON: Self = Self {
        id: "nether/create_beacon",
        parent: Some("minecraft:nether/summon_wither"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_CREATE_FULL_BEACON: Self = Self {
        id: "nether/create_full_beacon",
        parent: Some("minecraft:nether/create_beacon"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_DISTRACT_PIGLIN: Self = Self {
        id: "nether/distract_piglin",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_EXPLORE_NETHER: Self = Self {
        id: "nether/explore_nether",
        parent: Some("minecraft:nether/ride_strider"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_FAST_TRAVEL: Self = Self {
        id: "nether/fast_travel",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_FIND_BASTION: Self = Self {
        id: "nether/find_bastion",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_FIND_FORTRESS: Self = Self {
        id: "nether/find_fortress",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_GET_WITHER_SKULL: Self = Self {
        id: "nether/get_wither_skull",
        parent: Some("minecraft:nether/find_fortress"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_LOOT_BASTION: Self = Self {
        id: "nether/loot_bastion",
        parent: Some("minecraft:nether/find_bastion"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_NETHERITE_ARMOR: Self = Self {
        id: "nether/netherite_armor",
        parent: Some("minecraft:nether/obtain_ancient_debris"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_OBTAIN_ANCIENT_DEBRIS: Self = Self {
        id: "nether/obtain_ancient_debris",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_OBTAIN_BLAZE_ROD: Self = Self {
        id: "nether/obtain_blaze_rod",
        parent: Some("minecraft:nether/find_fortress"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_OBTAIN_CRYING_OBSIDIAN: Self = Self {
        id: "nether/obtain_crying_obsidian",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_RETURN_TO_SENDER: Self = Self {
        id: "nether/return_to_sender",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_RIDE_STRIDER: Self = Self {
        id: "nether/ride_strider",
        parent: Some("minecraft:nether/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_RIDE_STRIDER_IN_OVERWORLD_LAVA: Self = Self {
        id: "nether/ride_strider_in_overworld_lava",
        parent: Some("minecraft:nether/ride_strider"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_ROOT: Self = Self {
        id: "nether/root",
        parent: None,
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_SUMMON_WITHER: Self = Self {
        id: "nether/summon_wither",
        parent: Some("minecraft:nether/get_wither_skull"),
        send_telemetry: false,
        display_name: None,
    };
    pub const NETHER_UNEASY_ALLIANCE: Self = Self {
        id: "nether/uneasy_alliance",
        parent: Some("minecraft:nether/return_to_sender"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_BLAZE_POWDER: Self = Self {
        id: "recipes/brewing/blaze_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_BREWING_STAND: Self = Self {
        id: "recipes/brewing/brewing_stand",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_CAULDRON: Self = Self {
        id: "recipes/brewing/cauldron",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_FERMENTED_SPIDER_EYE: Self = Self {
        id: "recipes/brewing/fermented_spider_eye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_GLASS_BOTTLE: Self = Self {
        id: "recipes/brewing/glass_bottle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_GLISTERING_MELON_SLICE: Self = Self {
        id: "recipes/brewing/glistering_melon_slice",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_GOLDEN_CARROT: Self = Self {
        id: "recipes/brewing/golden_carrot",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BREWING_MAGMA_CREAM: Self = Self {
        id: "recipes/brewing/magma_cream",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ACACIA_PLANKS: Self = Self {
        id: "recipes/building_blocks/acacia_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ACACIA_SLAB: Self = Self {
        id: "recipes/building_blocks/acacia_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ACACIA_STAIRS: Self = Self {
        id: "recipes/building_blocks/acacia_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ACACIA_WOOD: Self = Self {
        id: "recipes/building_blocks/acacia_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_AMETHYST_BLOCK: Self = Self {
        id: "recipes/building_blocks/amethyst_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ANDESITE: Self = Self {
        id: "recipes/building_blocks/andesite",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB: Self = Self {
        id: "recipes/building_blocks/andesite_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/andesite_slab_from_andesite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS: Self = Self {
        id: "recipes/building_blocks/andesite_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/andesite_stairs_from_andesite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BAMBOO_BLOCK: Self = Self {
        id: "recipes/building_blocks/bamboo_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_SLAB: Self = Self {
        id: "recipes/building_blocks/bamboo_mosaic_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_STAIRS: Self = Self {
        id: "recipes/building_blocks/bamboo_mosaic_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BAMBOO_PLANKS: Self = Self {
        id: "recipes/building_blocks/bamboo_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BAMBOO_SLAB: Self = Self {
        id: "recipes/building_blocks/bamboo_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BAMBOO_STAIRS: Self = Self {
        id: "recipes/building_blocks/bamboo_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BIRCH_PLANKS: Self = Self {
        id: "recipes/building_blocks/birch_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BIRCH_SLAB: Self = Self {
        id: "recipes/building_blocks/birch_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BIRCH_STAIRS: Self = Self {
        id: "recipes/building_blocks/birch_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BIRCH_WOOD: Self = Self {
        id: "recipes/building_blocks/birch_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACK_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/black_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACK_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/black_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACK_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/black_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/blackstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/blackstone_slab_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/blackstone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/blackstone_stairs_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLUE_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/blue_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLUE_ICE: Self = Self {
        id: "recipes/building_blocks/blue_ice",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLUE_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/blue_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BLUE_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/blue_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BONE_BLOCK: Self = Self {
        id: "recipes/building_blocks/bone_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BOOKSHELF: Self = Self {
        id: "recipes/building_blocks/bookshelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BRICK_SLAB_FROM_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/brick_slab_from_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BRICK_STAIRS_FROM_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/brick_stairs_from_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BRICKS: Self = Self {
        id: "recipes/building_blocks/bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BROWN_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/brown_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BROWN_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/brown_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_BROWN_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/brown_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHERRY_PLANKS: Self = Self {
        id: "recipes/building_blocks/cherry_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHERRY_SLAB: Self = Self {
        id: "recipes/building_blocks/cherry_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHERRY_STAIRS: Self = Self {
        id: "recipes/building_blocks/cherry_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHERRY_WOOD: Self = Self {
        id: "recipes/building_blocks/cherry_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_BOOKSHELF: Self = Self {
        id: "recipes/building_blocks/chiseled_bookshelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_COPPER_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/chiseled_copper_from_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_CUT_COPPER_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/chiseled_copper_from_cut_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE: Self = Self {
        id: "recipes/building_blocks/chiseled_deepslate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_deepslate_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS: Self = Self {
        id: "recipes/building_blocks/chiseled_nether_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS_FROM_NETHER_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_nether_bricks_from_nether_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE: Self = Self {
        id: "recipes/building_blocks/chiseled_polished_blackstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/chiseled_polished_blackstone_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/chiseled_polished_blackstone_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK: Self = Self {
        id: "recipes/building_blocks/chiseled_quartz_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK_FROM_QUARTZ_BLOCK_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_quartz_block_from_quartz_block_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/chiseled_red_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_red_sandstone_from_red_sandstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS: Self = Self {
        id: "recipes/building_blocks/chiseled_resin_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS_FROM_RESIN_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_resin_bricks_from_resin_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/chiseled_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE_FROM_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/chiseled_sandstone_from_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS: Self = Self {
        id: "recipes/building_blocks/chiseled_stone_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_FROM_STONE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_stone_bricks_from_stone_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_STONE_FROM_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/chiseled_stone_bricks_stone_from_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_TUFF: Self = Self {
        id: "recipes/building_blocks/chiseled_tuff",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS: Self = Self {
        id: "recipes/building_blocks/chiseled_tuff_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_tuff_bricks_from_polished_tuff_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/chiseled_tuff_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CLAY: Self = Self {
        id: "recipes/building_blocks/clay",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COAL_BLOCK: Self = Self {
        id: "recipes/building_blocks/coal_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COARSE_DIRT: Self = Self {
        id: "recipes/building_blocks/coarse_dirt",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB: Self = Self {
        id: "recipes/building_blocks/cobbled_deepslate_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/cobbled_deepslate_slab_from_cobbled_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS: Self = Self {
        id: "recipes/building_blocks/cobbled_deepslate_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING : Self = Self { id : "recipes/building_blocks/cobbled_deepslate_stairs_from_cobbled_deepslate_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/cobblestone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB_FROM_COBBLESTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cobblestone_slab_from_cobblestone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/cobblestone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS_FROM_COBBLESTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/cobblestone_stairs_from_cobblestone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_COPPER_BLOCK: Self = Self {
        id: "recipes/building_blocks/copper_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_COPPER_GRATE_FROM_COPPER_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/copper_grate_from_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_BRICKS: Self = Self {
        id: "recipes/building_blocks/cracked_deepslate_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_TILES: Self = Self {
        id: "recipes/building_blocks/cracked_deepslate_tiles",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRACKED_NETHER_BRICKS: Self = Self {
        id: "recipes/building_blocks/cracked_nether_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRACKED_POLISHED_BLACKSTONE_BRICKS: Self = Self {
        id: "recipes/building_blocks/cracked_polished_blackstone_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRACKED_STONE_BRICKS: Self = Self {
        id: "recipes/building_blocks/cracked_stone_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRIMSON_HYPHAE: Self = Self {
        id: "recipes/building_blocks/crimson_hyphae",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRIMSON_PLANKS: Self = Self {
        id: "recipes/building_blocks/crimson_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRIMSON_SLAB: Self = Self {
        id: "recipes/building_blocks/crimson_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CRIMSON_STAIRS: Self = Self {
        id: "recipes/building_blocks/crimson_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_FROM_COPPER_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cut_copper_from_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_COPPER_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cut_copper_slab_from_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_CUT_COPPER_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cut_copper_slab_from_cut_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_COPPER_BLOCK_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/cut_copper_stairs_from_copper_block_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_CUT_COPPER_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cut_copper_stairs_from_cut_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/cut_red_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/cut_red_sandstone_from_red_sandstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/cut_red_sandstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_CUT_RED_SANDSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/cut_red_sandstone_slab_from_cut_red_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/cut_red_sandstone_slab_from_red_sandstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/cut_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_FROM_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cut_sandstone_from_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/cut_sandstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_CUT_SANDSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/cut_sandstone_slab_from_cut_sandstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/cut_sandstone_slab_from_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CYAN_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/cyan_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CYAN_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/cyan_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_CYAN_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/cyan_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_OAK_PLANKS: Self = Self {
        id: "recipes/building_blocks/dark_oak_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_OAK_SLAB: Self = Self {
        id: "recipes/building_blocks/dark_oak_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_OAK_STAIRS: Self = Self {
        id: "recipes/building_blocks/dark_oak_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_OAK_WOOD: Self = Self {
        id: "recipes/building_blocks/dark_oak_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE: Self = Self {
        id: "recipes/building_blocks/dark_prismarine",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB: Self = Self {
        id: "recipes/building_blocks/dark_prismarine_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB_FROM_DARK_PRISMARINE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/dark_prismarine_slab_from_dark_prismarine_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS: Self = Self {
        id: "recipes/building_blocks/dark_prismarine_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS_FROM_DARK_PRISMARINE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/dark_prismarine_stairs_from_dark_prismarine_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE: Self = Self {
        id: "recipes/building_blocks/deepslate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/deepslate_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_brick_slab_from_cobbled_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_brick_slab_from_deepslate_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_brick_slab_from_polished_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/deepslate_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_brick_stairs_from_cobbled_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_brick_stairs_from_deepslate_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING : Self = Self { id : "recipes/building_blocks/deepslate_brick_stairs_from_polished_deepslate_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS: Self = Self {
        id: "recipes/building_blocks/deepslate_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_bricks_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_POLISHED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_bricks_from_polished_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB: Self = Self {
        id: "recipes/building_blocks/deepslate_tile_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_tile_slab_from_cobbled_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_tile_slab_from_deepslate_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_TILES_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_tile_slab_from_deepslate_tiles_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_tile_slab_from_polished_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS: Self = Self {
        id: "recipes/building_blocks/deepslate_tile_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_tile_stairs_from_cobbled_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_TILES_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_tiles_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/deepslate_tile_stairs_from_polished_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES: Self = Self {
        id: "recipes/building_blocks/deepslate_tiles",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_tiles_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_DEEPSLATE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_tiles_from_deepslate_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_POLISHED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/deepslate_tiles_from_polished_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_DIAMOND_BLOCK: Self = Self {
        id: "recipes/building_blocks/diamond_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DIORITE: Self = Self {
        id: "recipes/building_blocks/diorite",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DIORITE_SLAB: Self = Self {
        id: "recipes/building_blocks/diorite_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DIORITE_SLAB_FROM_DIORITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/diorite_slab_from_diorite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS: Self = Self {
        id: "recipes/building_blocks/diorite_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/diorite_stairs_from_diorite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DRIED_GHAST: Self = Self {
        id: "recipes/building_blocks/dried_ghast",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DRIED_KELP_BLOCK: Self = Self {
        id: "recipes/building_blocks/dried_kelp_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DRIPSTONE_BLOCK: Self = Self {
        id: "recipes/building_blocks/dripstone_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_BLACK_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_black_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_BLUE_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_blue_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_BROWN_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_brown_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_CYAN_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_cyan_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_GRAY_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_gray_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_GREEN_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_green_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_LIGHT_BLUE_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_light_blue_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_LIGHT_GRAY_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_light_gray_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_LIME_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_lime_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_MAGENTA_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_magenta_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_ORANGE_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_orange_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_PINK_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_pink_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_PURPLE_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_purple_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_RED_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_red_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_WHITE_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_white_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_DYE_YELLOW_WOOL: Self = Self {
        id: "recipes/building_blocks/dye_yellow_wool",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EMERALD_BLOCK: Self = Self {
        id: "recipes/building_blocks/emerald_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/end_stone_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_BRICK_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/end_stone_brick_slab_from_end_stone_brick_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/end_stone_brick_slab_from_end_stone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/end_stone_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_BRICK_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_brick_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS: Self = Self {
        id: "recipes/building_blocks/end_stone_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS_FROM_END_STONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/end_stone_bricks_from_end_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/exposed_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/exposed_chiseled_copper_from_exposed_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/exposed_chiseled_copper_from_exposed_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/exposed_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE_FROM_EXPOSED_COPPER_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/exposed_copper_grate_from_exposed_copper_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/exposed_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_FROM_EXPOSED_COPPER_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/exposed_cut_copper_from_exposed_copper_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/exposed_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/exposed_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_GLASS: Self = Self {
        id: "recipes/building_blocks/glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GLOWSTONE: Self = Self {
        id: "recipes/building_blocks/glowstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GOLD_BLOCK: Self = Self {
        id: "recipes/building_blocks/gold_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRANITE: Self = Self {
        id: "recipes/building_blocks/granite",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRANITE_SLAB: Self = Self {
        id: "recipes/building_blocks/granite_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRANITE_SLAB_FROM_GRANITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/granite_slab_from_granite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS: Self = Self {
        id: "recipes/building_blocks/granite_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/granite_stairs_from_granite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRAY_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/gray_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRAY_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/gray_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GRAY_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/gray_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GREEN_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/green_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GREEN_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/green_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_GREEN_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/green_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_HAY_BLOCK: Self = Self {
        id: "recipes/building_blocks/hay_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_IRON_BLOCK: Self = Self {
        id: "recipes/building_blocks/iron_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_JACK_O_LANTERN: Self = Self {
        id: "recipes/building_blocks/jack_o_lantern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_JUNGLE_PLANKS: Self = Self {
        id: "recipes/building_blocks/jungle_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_JUNGLE_SLAB: Self = Self {
        id: "recipes/building_blocks/jungle_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_JUNGLE_STAIRS: Self = Self {
        id: "recipes/building_blocks/jungle_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_JUNGLE_WOOD: Self = Self {
        id: "recipes/building_blocks/jungle_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LAPIS_BLOCK: Self = Self {
        id: "recipes/building_blocks/lapis_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/light_blue_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/light_blue_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/light_blue_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/light_gray_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/light_gray_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/light_gray_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIME_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/lime_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIME_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/lime_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_LIME_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/lime_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MAGENTA_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/magenta_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MAGENTA_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/magenta_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MAGENTA_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/magenta_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MAGMA_BLOCK: Self = Self {
        id: "recipes/building_blocks/magma_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MANGROVE_PLANKS: Self = Self {
        id: "recipes/building_blocks/mangrove_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MANGROVE_SLAB: Self = Self {
        id: "recipes/building_blocks/mangrove_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MANGROVE_STAIRS: Self = Self {
        id: "recipes/building_blocks/mangrove_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MANGROVE_WOOD: Self = Self {
        id: "recipes/building_blocks/mangrove_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MELON: Self = Self {
        id: "recipes/building_blocks/melon",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_MOSS_BLOCK: Self = Self {
        id: "recipes/building_blocks/mossy_cobblestone_from_moss_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_VINE: Self = Self {
        id: "recipes/building_blocks/mossy_cobblestone_from_vine",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/mossy_cobblestone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB_FROM_MOSSY_COBBLESTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/mossy_cobblestone_slab_from_mossy_cobblestone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/mossy_cobblestone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS_FROM_MOSSY_COBBLESTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/mossy_cobblestone_stairs_from_mossy_cobblestone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/mossy_stone_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB_FROM_MOSSY_STONE_BRICK_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/mossy_stone_brick_slab_from_mossy_stone_brick_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/mossy_stone_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS_FROM_MOSSY_STONE_BRICK_STONECUTTING : Self = Self { id : "recipes/building_blocks/mossy_stone_brick_stairs_from_mossy_stone_brick_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_MOSS_BLOCK: Self = Self {
        id: "recipes/building_blocks/mossy_stone_bricks_from_moss_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_VINE: Self = Self {
        id: "recipes/building_blocks/mossy_stone_bricks_from_vine",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/mud_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB_FROM_MUD_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/mud_brick_slab_from_mud_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/mud_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS_FROM_MUD_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/mud_brick_stairs_from_mud_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MUD_BRICKS: Self = Self {
        id: "recipes/building_blocks/mud_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_MUDDY_MANGROVE_ROOTS: Self = Self {
        id: "recipes/building_blocks/muddy_mangrove_roots",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/nether_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB_FROM_NETHER_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/nether_brick_slab_from_nether_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/nether_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS_FROM_NETHER_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/nether_brick_stairs_from_nether_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_NETHER_BRICKS: Self = Self {
        id: "recipes/building_blocks/nether_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_NETHER_WART_BLOCK: Self = Self {
        id: "recipes/building_blocks/nether_wart_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_NETHERITE_BLOCK: Self = Self {
        id: "recipes/building_blocks/netherite_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OAK_PLANKS: Self = Self {
        id: "recipes/building_blocks/oak_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OAK_SLAB: Self = Self {
        id: "recipes/building_blocks/oak_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OAK_STAIRS: Self = Self {
        id: "recipes/building_blocks/oak_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OAK_WOOD: Self = Self {
        id: "recipes/building_blocks/oak_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ORANGE_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/orange_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ORANGE_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/orange_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_ORANGE_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/orange_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/oxidized_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/oxidized_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE_FROM_OXIDIZED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/oxidized_copper_grate_from_oxidized_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/oxidized_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/oxidized_cut_copper_from_oxidized_copper_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/oxidized_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/oxidized_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_PACKED_ICE: Self = Self {
        id: "recipes/building_blocks/packed_ice",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PACKED_MUD: Self = Self {
        id: "recipes/building_blocks/packed_mud",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PALE_OAK_PLANKS: Self = Self {
        id: "recipes/building_blocks/pale_oak_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PALE_OAK_SLAB: Self = Self {
        id: "recipes/building_blocks/pale_oak_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PALE_OAK_STAIRS: Self = Self {
        id: "recipes/building_blocks/pale_oak_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PALE_OAK_WOOD: Self = Self {
        id: "recipes/building_blocks/pale_oak_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PINK_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/pink_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PINK_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/pink_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PINK_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/pink_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE: Self = Self {
        id: "recipes/building_blocks/polished_andesite",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_FROM_ANDESITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_andesite_from_andesite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_andesite_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_andesite_slab_from_andesite_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_POLISHED_ANDESITE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_andesite_slab_from_polished_andesite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_andesite_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_andesite_stairs_from_andesite_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_POLISHED_ANDESITE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_andesite_stairs_from_polished_andesite_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BASALT: Self = Self {
        id: "recipes/building_blocks/polished_basalt",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BASALT_FROM_BASALT_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_basalt_from_basalt_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE: Self = Self {
        id: "recipes/building_blocks/polished_blackstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_blackstone_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_brick_slab_from_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_bricks_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_blackstone_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_brick_stairs_from_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_bricks_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS: Self = Self {
        id: "recipes/building_blocks/polished_blackstone_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_BLACKSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_blackstone_bricks_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_bricks_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_blackstone_from_blackstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_blackstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_blackstone_slab_from_blackstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_slab_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_blackstone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_blackstone_stairs_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_blackstone_stairs_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE: Self = Self {
        id: "recipes/building_blocks/polished_deepslate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_deepslate_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_deepslate_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_deepslate_slab_from_cobbled_deepslate_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_deepslate_slab_from_polished_deepslate_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_deepslate_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_deepslate_stairs_from_cobbled_deepslate_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING : Self = Self { id : "recipes/building_blocks/polished_deepslate_stairs_from_polished_deepslate_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE: Self = Self {
        id: "recipes/building_blocks/polished_diorite",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_FROM_DIORITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_diorite_from_diorite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_diorite_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_DIORITE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_diorite_slab_from_diorite_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_POLISHED_DIORITE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_diorite_slab_from_polished_diorite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_diorite_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_diorite_stairs_from_diorite_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_POLISHED_DIORITE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_diorite_stairs_from_polished_diorite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE: Self = Self {
        id: "recipes/building_blocks/polished_granite",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_FROM_GRANITE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_granite_from_granite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_granite_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_GRANITE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_granite_slab_from_granite_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_POLISHED_GRANITE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_granite_slab_from_polished_granite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_granite_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_granite_stairs_from_granite_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_POLISHED_GRANITE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/polished_granite_stairs_from_polished_granite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF: Self = Self {
        id: "recipes/building_blocks/polished_tuff",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_tuff_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB: Self = Self {
        id: "recipes/building_blocks/polished_tuff_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_POLISHED_TUFF_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_tuff_slab_from_polished_tuff_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_tuff_slab_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS: Self = Self {
        id: "recipes/building_blocks/polished_tuff_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_POLISHED_TUFF_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/polished_tuff_stairs_from_polished_tuff_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/polished_tuff_stairs_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE: Self = Self {
        id: "recipes/building_blocks/prismarine",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/prismarine_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB_FROM_PRISMARINE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/prismarine_brick_slab_from_prismarine_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/prismarine_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS_FROM_PRISMARINE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/prismarine_brick_stairs_from_prismarine_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICKS: Self = Self {
        id: "recipes/building_blocks/prismarine_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB: Self = Self {
        id: "recipes/building_blocks/prismarine_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB_FROM_PRISMARINE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/prismarine_slab_from_prismarine_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS: Self = Self {
        id: "recipes/building_blocks/prismarine_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS_FROM_PRISMARINE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/prismarine_stairs_from_prismarine_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPLE_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/purple_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPLE_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/purple_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPLE_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/purple_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_BLOCK: Self = Self {
        id: "recipes/building_blocks/purpur_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR: Self = Self {
        id: "recipes/building_blocks/purpur_pillar",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR_FROM_PURPUR_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/purpur_pillar_from_purpur_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_SLAB: Self = Self {
        id: "recipes/building_blocks/purpur_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_SLAB_FROM_PURPUR_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/purpur_slab_from_purpur_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS: Self = Self {
        id: "recipes/building_blocks/purpur_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS_FROM_PURPUR_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/purpur_stairs_from_purpur_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_BLOCK: Self = Self {
        id: "recipes/building_blocks/quartz_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS: Self = Self {
        id: "recipes/building_blocks/quartz_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS_FROM_QUARTZ_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/quartz_bricks_from_quartz_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR: Self = Self {
        id: "recipes/building_blocks/quartz_pillar",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR_FROM_QUARTZ_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/quartz_pillar_from_quartz_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB: Self = Self {
        id: "recipes/building_blocks/quartz_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB_FROM_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/quartz_slab_from_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS: Self = Self {
        id: "recipes/building_blocks/quartz_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS_FROM_QUARTZ_BLOCK_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/quartz_stairs_from_quartz_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RAW_COPPER_BLOCK: Self = Self {
        id: "recipes/building_blocks/raw_copper_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RAW_GOLD_BLOCK: Self = Self {
        id: "recipes/building_blocks/raw_gold_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RAW_IRON_BLOCK: Self = Self {
        id: "recipes/building_blocks/raw_iron_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/red_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/red_nether_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB_FROM_RED_NETHER_BRICKS_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/red_nether_brick_slab_from_red_nether_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/red_nether_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS_FROM_RED_NETHER_BRICKS_STONECUTTING : Self = Self { id : "recipes/building_blocks/red_nether_brick_stairs_from_red_nether_bricks_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICKS: Self = Self {
        id: "recipes/building_blocks/red_nether_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/red_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/red_sandstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/red_sandstone_slab_from_red_sandstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/red_sandstone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS_FROM_RED_SANDSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/red_sandstone_stairs_from_red_sandstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_RED_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/red_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RED_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/red_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RESIN_BLOCK: Self = Self {
        id: "recipes/building_blocks/resin_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/resin_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB_FROM_RESIN_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/resin_brick_slab_from_resin_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/resin_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS_FROM_RESIN_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/resin_brick_stairs_from_resin_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_RESIN_BRICKS: Self = Self {
        id: "recipes/building_blocks/resin_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/sandstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/sandstone_slab_from_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/sandstone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS_FROM_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/sandstone_stairs_from_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SEA_LANTERN: Self = Self {
        id: "recipes/building_blocks/sea_lantern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_BASALT: Self = Self {
        id: "recipes/building_blocks/smooth_basalt",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ: Self = Self {
        id: "recipes/building_blocks/smooth_quartz",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB: Self = Self {
        id: "recipes/building_blocks/smooth_quartz_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB_FROM_SMOOTH_QUARTZ_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/smooth_quartz_slab_from_smooth_quartz_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS: Self = Self {
        id: "recipes/building_blocks/smooth_quartz_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS_FROM_SMOOTH_QUARTZ_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/smooth_quartz_stairs_from_smooth_quartz_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/smooth_red_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/smooth_red_sandstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/smooth_red_sandstone_slab_from_smooth_red_sandstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/smooth_red_sandstone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING : Self = Self { id : "recipes/building_blocks/smooth_red_sandstone_stairs_from_smooth_red_sandstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE: Self = Self {
        id: "recipes/building_blocks/smooth_sandstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB: Self = Self {
        id: "recipes/building_blocks/smooth_sandstone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB_FROM_SMOOTH_SANDSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/smooth_sandstone_slab_from_smooth_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/smooth_sandstone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS_FROM_SMOOTH_SANDSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/smooth_sandstone_stairs_from_smooth_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_STONE: Self = Self {
        id: "recipes/building_blocks/smooth_stone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB: Self = Self {
        id: "recipes/building_blocks/smooth_stone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB_FROM_SMOOTH_STONE_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/smooth_stone_slab_from_smooth_stone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_SNOW_BLOCK: Self = Self {
        id: "recipes/building_blocks/snow_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SPONGE: Self = Self {
        id: "recipes/building_blocks/sponge",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SPRUCE_PLANKS: Self = Self {
        id: "recipes/building_blocks/spruce_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SPRUCE_SLAB: Self = Self {
        id: "recipes/building_blocks/spruce_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SPRUCE_STAIRS: Self = Self {
        id: "recipes/building_blocks/spruce_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_SPRUCE_WOOD: Self = Self {
        id: "recipes/building_blocks/spruce_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE: Self = Self {
        id: "recipes/building_blocks/stone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/stone_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/stone_brick_slab_from_stone_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/stone_brick_slab_from_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/stone_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/stone_brick_stairs_from_stone_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/stone_brick_stairs_from_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICKS: Self = Self {
        id: "recipes/building_blocks/stone_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_BRICKS_FROM_STONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/stone_bricks_from_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_SLAB: Self = Self {
        id: "recipes/building_blocks/stone_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_SLAB_FROM_STONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/stone_slab_from_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_STAIRS: Self = Self {
        id: "recipes/building_blocks/stone_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STONE_STAIRS_FROM_STONE_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/stone_stairs_from_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_ACACIA_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_acacia_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_BIRCH_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_birch_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_CHERRY_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_cherry_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_CRIMSON_HYPHAE: Self = Self {
        id: "recipes/building_blocks/stripped_crimson_hyphae",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_DARK_OAK_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_dark_oak_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_JUNGLE_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_jungle_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_MANGROVE_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_mangrove_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_OAK_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_oak_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_PALE_OAK_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_pale_oak_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_SPRUCE_WOOD: Self = Self {
        id: "recipes/building_blocks/stripped_spruce_wood",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_STRIPPED_WARPED_HYPHAE: Self = Self {
        id: "recipes/building_blocks/stripped_warped_hyphae",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TINTED_GLASS: Self = Self {
        id: "recipes/building_blocks/tinted_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB: Self = Self {
        id: "recipes/building_blocks/tuff_brick_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_POLISHED_TUFF_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/tuff_brick_slab_from_polished_tuff_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_brick_slab_from_tuff_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_brick_slab_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS: Self = Self {
        id: "recipes/building_blocks/tuff_brick_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_POLISHED_TUFF_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/tuff_brick_stairs_from_polished_tuff_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/tuff_brick_stairs_from_tuff_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_brick_stairs_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICKS: Self = Self {
        id: "recipes/building_blocks/tuff_bricks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_bricks_from_polished_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_bricks_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_SLAB: Self = Self {
        id: "recipes/building_blocks/tuff_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_SLAB_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_slab_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_STAIRS: Self = Self {
        id: "recipes/building_blocks/tuff_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_TUFF_STAIRS_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/building_blocks/tuff_stairs_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WARPED_HYPHAE: Self = Self {
        id: "recipes/building_blocks/warped_hyphae",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WARPED_PLANKS: Self = Self {
        id: "recipes/building_blocks/warped_planks",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WARPED_SLAB: Self = Self {
        id: "recipes/building_blocks/warped_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WARPED_STAIRS: Self = Self {
        id: "recipes/building_blocks/warped_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_chiseled_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/waxed_chiseled_copper_from_waxed_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_CUT_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/waxed_chiseled_copper_from_waxed_cut_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BARS_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_bars_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BLOCK_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_block_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHAIN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_chain_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHEST_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_chest_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_golem_statue_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/waxed_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_grate_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_WAXED_COPPER_BLOCK_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/waxed_copper_grate_from_waxed_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_COPPER_LANTERN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_copper_lantern_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING: Self =
        Self {
            id: "recipes/building_blocks/waxed_cut_copper_from_waxed_copper_block_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_slab_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_COPPER_BLOCK_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_copper_block_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_CUT_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_cut_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_stairs_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_COPPER_BLOCK_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_CUT_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_chiseled_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_BARS_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_bars_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHAIN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_chain_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHEST_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_chest_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB: Self =
        Self {
            id: "recipes/building_blocks/waxed_exposed_copper_golem_statue_from_honeycomb",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_grate_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_WAXED_EXPOSED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_copper_grate_from_waxed_exposed_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_LANTERN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_copper_lantern_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_cut_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_LIGHTNING_ROD_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_exposed_lightning_rod_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_LIGHTNING_ROD_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_lightning_rod_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_BARS_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_bars_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHAIN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_chain_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHEST_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_chest_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB: Self =
        Self {
            id: "recipes/building_blocks/waxed_oxidized_copper_golem_statue_from_honeycomb",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_grate_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_copper_grate_from_waxed_oxidized_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_LANTERN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_copper_lantern_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_cut_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_cut_copper_from_waxed_oxidized_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_HONEYCOMB: Self =
        Self {
            id: "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_honeycomb",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_LIGHTNING_ROD_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_oxidized_lightning_rod_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_chiseled_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_BARS_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_bars_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHAIN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_chain_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHEST_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_chest_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB: Self =
        Self {
            id: "recipes/building_blocks/waxed_weathered_copper_golem_statue_from_honeycomb",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_grate_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_WAXED_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_copper_grate_from_waxed_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_LANTERN_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_copper_lantern_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_cut_copper_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_cut_copper_from_waxed_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_HONEYCOMB: Self =
        Self {
            id: "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_honeycomb",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_LIGHTNING_ROD_FROM_HONEYCOMB: Self = Self {
        id: "recipes/building_blocks/waxed_weathered_lightning_rod_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER: Self = Self {
        id: "recipes/building_blocks/weathered_chiseled_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/weathered_chiseled_copper_from_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/weathered_chiseled_copper_from_weathered_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE: Self = Self {
        id: "recipes/building_blocks/weathered_copper_grate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE_FROM_WEATHERED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/weathered_copper_grate_from_weathered_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER: Self = Self {
        id: "recipes/building_blocks/weathered_cut_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_FROM_WEATHERED_COPPER_STONECUTTING:
        Self = Self {
        id: "recipes/building_blocks/weathered_cut_copper_from_weathered_copper_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB: Self = Self {
        id: "recipes/building_blocks/weathered_cut_copper_slab",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS: Self = Self {
        id: "recipes/building_blocks/weathered_cut_copper_stairs",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_CUT_COPPER_STONECUTTING : Self = Self { id : "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_cut_copper_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_BUILDING_BLOCKS_WHITE_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/white_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WHITE_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/white_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WHITE_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/white_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_WHITE_WOOL_FROM_STRING: Self = Self {
        id: "recipes/building_blocks/white_wool_from_string",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_YELLOW_CONCRETE_POWDER: Self = Self {
        id: "recipes/building_blocks/yellow_concrete_powder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_YELLOW_STAINED_GLASS: Self = Self {
        id: "recipes/building_blocks/yellow_stained_glass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_BUILDING_BLOCKS_YELLOW_TERRACOTTA: Self = Self {
        id: "recipes/building_blocks/yellow_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_ARROW: Self = Self {
        id: "recipes/combat/arrow",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_BLACK_HARNESS: Self = Self {
        id: "recipes/combat/black_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_BLUE_HARNESS: Self = Self {
        id: "recipes/combat/blue_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_BOW: Self = Self {
        id: "recipes/combat/bow",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_BROWN_HARNESS: Self = Self {
        id: "recipes/combat/brown_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_COPPER_BOOTS: Self = Self {
        id: "recipes/combat/copper_boots",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_COPPER_CHESTPLATE: Self = Self {
        id: "recipes/combat/copper_chestplate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_COPPER_HELMET: Self = Self {
        id: "recipes/combat/copper_helmet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_COPPER_LEGGINGS: Self = Self {
        id: "recipes/combat/copper_leggings",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_COPPER_SPEAR: Self = Self {
        id: "recipes/combat/copper_spear",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_COPPER_SWORD: Self = Self {
        id: "recipes/combat/copper_sword",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_CROSSBOW: Self = Self {
        id: "recipes/combat/crossbow",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_CYAN_HARNESS: Self = Self {
        id: "recipes/combat/cyan_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DIAMOND_BOOTS: Self = Self {
        id: "recipes/combat/diamond_boots",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DIAMOND_CHESTPLATE: Self = Self {
        id: "recipes/combat/diamond_chestplate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DIAMOND_HELMET: Self = Self {
        id: "recipes/combat/diamond_helmet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DIAMOND_LEGGINGS: Self = Self {
        id: "recipes/combat/diamond_leggings",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DIAMOND_SPEAR: Self = Self {
        id: "recipes/combat/diamond_spear",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DIAMOND_SWORD: Self = Self {
        id: "recipes/combat/diamond_sword",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_BLACK_HARNESS: Self = Self {
        id: "recipes/combat/dye_black_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_BLUE_HARNESS: Self = Self {
        id: "recipes/combat/dye_blue_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_BROWN_HARNESS: Self = Self {
        id: "recipes/combat/dye_brown_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_CYAN_HARNESS: Self = Self {
        id: "recipes/combat/dye_cyan_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_GRAY_HARNESS: Self = Self {
        id: "recipes/combat/dye_gray_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_GREEN_HARNESS: Self = Self {
        id: "recipes/combat/dye_green_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_LIGHT_BLUE_HARNESS: Self = Self {
        id: "recipes/combat/dye_light_blue_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_LIGHT_GRAY_HARNESS: Self = Self {
        id: "recipes/combat/dye_light_gray_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_LIME_HARNESS: Self = Self {
        id: "recipes/combat/dye_lime_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_MAGENTA_HARNESS: Self = Self {
        id: "recipes/combat/dye_magenta_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_ORANGE_HARNESS: Self = Self {
        id: "recipes/combat/dye_orange_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_PINK_HARNESS: Self = Self {
        id: "recipes/combat/dye_pink_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_PURPLE_HARNESS: Self = Self {
        id: "recipes/combat/dye_purple_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_RED_HARNESS: Self = Self {
        id: "recipes/combat/dye_red_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_WHITE_HARNESS: Self = Self {
        id: "recipes/combat/dye_white_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_DYE_YELLOW_HARNESS: Self = Self {
        id: "recipes/combat/dye_yellow_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GOLDEN_BOOTS: Self = Self {
        id: "recipes/combat/golden_boots",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GOLDEN_CHESTPLATE: Self = Self {
        id: "recipes/combat/golden_chestplate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GOLDEN_HELMET: Self = Self {
        id: "recipes/combat/golden_helmet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GOLDEN_LEGGINGS: Self = Self {
        id: "recipes/combat/golden_leggings",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GOLDEN_SPEAR: Self = Self {
        id: "recipes/combat/golden_spear",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GOLDEN_SWORD: Self = Self {
        id: "recipes/combat/golden_sword",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GRAY_HARNESS: Self = Self {
        id: "recipes/combat/gray_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_GREEN_HARNESS: Self = Self {
        id: "recipes/combat/green_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_IRON_BOOTS: Self = Self {
        id: "recipes/combat/iron_boots",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_IRON_CHESTPLATE: Self = Self {
        id: "recipes/combat/iron_chestplate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_IRON_HELMET: Self = Self {
        id: "recipes/combat/iron_helmet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_IRON_LEGGINGS: Self = Self {
        id: "recipes/combat/iron_leggings",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_IRON_SPEAR: Self = Self {
        id: "recipes/combat/iron_spear",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_IRON_SWORD: Self = Self {
        id: "recipes/combat/iron_sword",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LEATHER_BOOTS: Self = Self {
        id: "recipes/combat/leather_boots",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LEATHER_CHESTPLATE: Self = Self {
        id: "recipes/combat/leather_chestplate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LEATHER_HELMET: Self = Self {
        id: "recipes/combat/leather_helmet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LEATHER_LEGGINGS: Self = Self {
        id: "recipes/combat/leather_leggings",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LIGHT_BLUE_HARNESS: Self = Self {
        id: "recipes/combat/light_blue_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LIGHT_GRAY_HARNESS: Self = Self {
        id: "recipes/combat/light_gray_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_LIME_HARNESS: Self = Self {
        id: "recipes/combat/lime_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_MACE: Self = Self {
        id: "recipes/combat/mace",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_MAGENTA_HARNESS: Self = Self {
        id: "recipes/combat/magenta_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_BOOTS_SMITHING: Self = Self {
        id: "recipes/combat/netherite_boots_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_CHESTPLATE_SMITHING: Self = Self {
        id: "recipes/combat/netherite_chestplate_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_HELMET_SMITHING: Self = Self {
        id: "recipes/combat/netherite_helmet_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_HORSE_ARMOR_SMITHING: Self = Self {
        id: "recipes/combat/netherite_horse_armor_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_LEGGINGS_SMITHING: Self = Self {
        id: "recipes/combat/netherite_leggings_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_NAUTILUS_ARMOR_SMITHING: Self = Self {
        id: "recipes/combat/netherite_nautilus_armor_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_SPEAR_SMITHING: Self = Self {
        id: "recipes/combat/netherite_spear_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_NETHERITE_SWORD_SMITHING: Self = Self {
        id: "recipes/combat/netherite_sword_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_ORANGE_HARNESS: Self = Self {
        id: "recipes/combat/orange_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_PINK_HARNESS: Self = Self {
        id: "recipes/combat/pink_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_PURPLE_HARNESS: Self = Self {
        id: "recipes/combat/purple_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_RED_HARNESS: Self = Self {
        id: "recipes/combat/red_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_SADDLE: Self = Self {
        id: "recipes/combat/saddle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_SHIELD: Self = Self {
        id: "recipes/combat/shield",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_SPECTRAL_ARROW: Self = Self {
        id: "recipes/combat/spectral_arrow",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_STONE_SPEAR: Self = Self {
        id: "recipes/combat/stone_spear",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_STONE_SWORD: Self = Self {
        id: "recipes/combat/stone_sword",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_TURTLE_HELMET: Self = Self {
        id: "recipes/combat/turtle_helmet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_WHITE_HARNESS: Self = Self {
        id: "recipes/combat/white_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_WOLF_ARMOR: Self = Self {
        id: "recipes/combat/wolf_armor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_WOODEN_SPEAR: Self = Self {
        id: "recipes/combat/wooden_spear",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_WOODEN_SWORD: Self = Self {
        id: "recipes/combat/wooden_sword",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_COMBAT_YELLOW_HARNESS: Self = Self {
        id: "recipes/combat/yellow_harness",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ACACIA_FENCE: Self = Self {
        id: "recipes/decorations/acacia_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ACACIA_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/acacia_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ACACIA_SHELF: Self = Self {
        id: "recipes/decorations/acacia_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ACACIA_SIGN: Self = Self {
        id: "recipes/decorations/acacia_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ANDESITE_WALL: Self = Self {
        id: "recipes/decorations/andesite_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ANDESITE_WALL_FROM_ANDESITE_STONECUTTING: Self = Self {
        id: "recipes/decorations/andesite_wall_from_andesite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ANVIL: Self = Self {
        id: "recipes/decorations/anvil",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ARMOR_STAND: Self = Self {
        id: "recipes/decorations/armor_stand",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BAMBOO_FENCE: Self = Self {
        id: "recipes/decorations/bamboo_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BAMBOO_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/bamboo_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BAMBOO_MOSAIC: Self = Self {
        id: "recipes/decorations/bamboo_mosaic",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BAMBOO_SHELF: Self = Self {
        id: "recipes/decorations/bamboo_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BAMBOO_SIGN: Self = Self {
        id: "recipes/decorations/bamboo_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BARREL: Self = Self {
        id: "recipes/decorations/barrel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BEEHIVE: Self = Self {
        id: "recipes/decorations/beehive",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BIRCH_FENCE: Self = Self {
        id: "recipes/decorations/birch_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BIRCH_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/birch_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BIRCH_SHELF: Self = Self {
        id: "recipes/decorations/birch_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BIRCH_SIGN: Self = Self {
        id: "recipes/decorations/birch_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_BANNER: Self = Self {
        id: "recipes/decorations/black_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_BED: Self = Self {
        id: "recipes/decorations/black_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_CANDLE: Self = Self {
        id: "recipes/decorations/black_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_CARPET: Self = Self {
        id: "recipes/decorations/black_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/black_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/black_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/black_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/black_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACKSTONE_WALL: Self = Self {
        id: "recipes/decorations/blackstone_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING: Self = Self {
        id: "recipes/decorations/blackstone_wall_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLAST_FURNACE: Self = Self {
        id: "recipes/decorations/blast_furnace",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_BANNER: Self = Self {
        id: "recipes/decorations/blue_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_BED: Self = Self {
        id: "recipes/decorations/blue_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_CANDLE: Self = Self {
        id: "recipes/decorations/blue_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_CARPET: Self = Self {
        id: "recipes/decorations/blue_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/blue_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/blue_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/blue_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/blue_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BRICK_WALL: Self = Self {
        id: "recipes/decorations/brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BRICK_WALL_FROM_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/decorations/brick_wall_from_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_BANNER: Self = Self {
        id: "recipes/decorations/brown_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_BED: Self = Self {
        id: "recipes/decorations/brown_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_CANDLE: Self = Self {
        id: "recipes/decorations/brown_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_CARPET: Self = Self {
        id: "recipes/decorations/brown_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/brown_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/brown_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/brown_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/brown_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CAMPFIRE: Self = Self {
        id: "recipes/decorations/campfire",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CANDLE: Self = Self {
        id: "recipes/decorations/candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CARTOGRAPHY_TABLE: Self = Self {
        id: "recipes/decorations/cartography_table",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CHERRY_FENCE: Self = Self {
        id: "recipes/decorations/cherry_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CHERRY_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/cherry_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CHERRY_SHELF: Self = Self {
        id: "recipes/decorations/cherry_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CHERRY_SIGN: Self = Self {
        id: "recipes/decorations/cherry_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CHEST: Self = Self {
        id: "recipes/decorations/chest",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL: Self = Self {
        id: "recipes/decorations/cobbled_deepslate_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/cobbled_deepslate_wall_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_COBBLESTONE_WALL: Self = Self {
        id: "recipes/decorations/cobblestone_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COBBLESTONE_WALL_FROM_COBBLESTONE_STONECUTTING: Self = Self {
        id: "recipes/decorations/cobblestone_wall_from_cobblestone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COMPOSTER: Self = Self {
        id: "recipes/decorations/composter",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COPPER_BARS: Self = Self {
        id: "recipes/decorations/copper_bars",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COPPER_CHAIN: Self = Self {
        id: "recipes/decorations/copper_chain",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COPPER_CHEST: Self = Self {
        id: "recipes/decorations/copper_chest",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COPPER_LANTERN: Self = Self {
        id: "recipes/decorations/copper_lantern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_COPPER_TORCH: Self = Self {
        id: "recipes/decorations/copper_torch",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CRAFTING_TABLE: Self = Self {
        id: "recipes/decorations/crafting_table",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CRIMSON_FENCE: Self = Self {
        id: "recipes/decorations/crimson_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CRIMSON_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/crimson_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CRIMSON_SHELF: Self = Self {
        id: "recipes/decorations/crimson_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CRIMSON_SIGN: Self = Self {
        id: "recipes/decorations/crimson_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_BANNER: Self = Self {
        id: "recipes/decorations/cyan_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_BED: Self = Self {
        id: "recipes/decorations/cyan_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_CANDLE: Self = Self {
        id: "recipes/decorations/cyan_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_CARPET: Self = Self {
        id: "recipes/decorations/cyan_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/cyan_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/cyan_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/cyan_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/cyan_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DARK_OAK_FENCE: Self = Self {
        id: "recipes/decorations/dark_oak_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DARK_OAK_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/dark_oak_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DARK_OAK_SHELF: Self = Self {
        id: "recipes/decorations/dark_oak_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DARK_OAK_SIGN: Self = Self {
        id: "recipes/decorations/dark_oak_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DECORATED_POT_SIMPLE: Self = Self {
        id: "recipes/decorations/decorated_pot_simple",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL: Self = Self {
        id: "recipes/decorations/deepslate_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_brick_wall_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_brick_wall_from_deepslate_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_brick_wall_from_polished_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL: Self = Self {
        id: "recipes/decorations/deepslate_tile_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_tile_wall_from_cobbled_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_tile_wall_from_deepslate_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_TILES_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_tile_wall_from_deepslate_tiles_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/deepslate_tile_wall_from_polished_deepslate_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_DIORITE_WALL: Self = Self {
        id: "recipes/decorations/diorite_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DIORITE_WALL_FROM_DIORITE_STONECUTTING: Self = Self {
        id: "recipes/decorations/diorite_wall_from_diorite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_BLACK_BED: Self = Self {
        id: "recipes/decorations/dye_black_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_BLACK_CARPET: Self = Self {
        id: "recipes/decorations/dye_black_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_BLUE_BED: Self = Self {
        id: "recipes/decorations/dye_blue_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_BLUE_CARPET: Self = Self {
        id: "recipes/decorations/dye_blue_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_BROWN_BED: Self = Self {
        id: "recipes/decorations/dye_brown_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_BROWN_CARPET: Self = Self {
        id: "recipes/decorations/dye_brown_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_CYAN_BED: Self = Self {
        id: "recipes/decorations/dye_cyan_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_CYAN_CARPET: Self = Self {
        id: "recipes/decorations/dye_cyan_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_GRAY_BED: Self = Self {
        id: "recipes/decorations/dye_gray_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_GRAY_CARPET: Self = Self {
        id: "recipes/decorations/dye_gray_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_GREEN_BED: Self = Self {
        id: "recipes/decorations/dye_green_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_GREEN_CARPET: Self = Self {
        id: "recipes/decorations/dye_green_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_LIGHT_BLUE_BED: Self = Self {
        id: "recipes/decorations/dye_light_blue_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_LIGHT_BLUE_CARPET: Self = Self {
        id: "recipes/decorations/dye_light_blue_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_LIGHT_GRAY_BED: Self = Self {
        id: "recipes/decorations/dye_light_gray_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_LIGHT_GRAY_CARPET: Self = Self {
        id: "recipes/decorations/dye_light_gray_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_LIME_BED: Self = Self {
        id: "recipes/decorations/dye_lime_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_LIME_CARPET: Self = Self {
        id: "recipes/decorations/dye_lime_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_MAGENTA_BED: Self = Self {
        id: "recipes/decorations/dye_magenta_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_MAGENTA_CARPET: Self = Self {
        id: "recipes/decorations/dye_magenta_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_ORANGE_BED: Self = Self {
        id: "recipes/decorations/dye_orange_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_ORANGE_CARPET: Self = Self {
        id: "recipes/decorations/dye_orange_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_PINK_BED: Self = Self {
        id: "recipes/decorations/dye_pink_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_PINK_CARPET: Self = Self {
        id: "recipes/decorations/dye_pink_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_PURPLE_BED: Self = Self {
        id: "recipes/decorations/dye_purple_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_PURPLE_CARPET: Self = Self {
        id: "recipes/decorations/dye_purple_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_RED_BED: Self = Self {
        id: "recipes/decorations/dye_red_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_RED_CARPET: Self = Self {
        id: "recipes/decorations/dye_red_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_WHITE_BED: Self = Self {
        id: "recipes/decorations/dye_white_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_WHITE_CARPET: Self = Self {
        id: "recipes/decorations/dye_white_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_YELLOW_BED: Self = Self {
        id: "recipes/decorations/dye_yellow_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_DYE_YELLOW_CARPET: Self = Self {
        id: "recipes/decorations/dye_yellow_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ENCHANTING_TABLE: Self = Self {
        id: "recipes/decorations/enchanting_table",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_END_CRYSTAL: Self = Self {
        id: "recipes/decorations/end_crystal",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_END_ROD: Self = Self {
        id: "recipes/decorations/end_rod",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_END_STONE_BRICK_WALL: Self = Self {
        id: "recipes/decorations/end_stone_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_BRICK_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/end_stone_brick_wall_from_end_stone_brick_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_STONECUTTING: Self = Self {
        id: "recipes/decorations/end_stone_brick_wall_from_end_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ENDER_CHEST: Self = Self {
        id: "recipes/decorations/ender_chest",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_FLETCHING_TABLE: Self = Self {
        id: "recipes/decorations/fletching_table",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_FLOWER_POT: Self = Self {
        id: "recipes/decorations/flower_pot",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_FURNACE: Self = Self {
        id: "recipes/decorations/furnace",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GLASS_PANE: Self = Self {
        id: "recipes/decorations/glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GLOW_ITEM_FRAME: Self = Self {
        id: "recipes/decorations/glow_item_frame",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRANITE_WALL: Self = Self {
        id: "recipes/decorations/granite_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRANITE_WALL_FROM_GRANITE_STONECUTTING: Self = Self {
        id: "recipes/decorations/granite_wall_from_granite_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_BANNER: Self = Self {
        id: "recipes/decorations/gray_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_BED: Self = Self {
        id: "recipes/decorations/gray_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_CANDLE: Self = Self {
        id: "recipes/decorations/gray_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_CARPET: Self = Self {
        id: "recipes/decorations/gray_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/gray_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/gray_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/gray_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/gray_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_BANNER: Self = Self {
        id: "recipes/decorations/green_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_BED: Self = Self {
        id: "recipes/decorations/green_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_CANDLE: Self = Self {
        id: "recipes/decorations/green_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_CARPET: Self = Self {
        id: "recipes/decorations/green_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/green_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/green_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/green_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/green_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_GRINDSTONE: Self = Self {
        id: "recipes/decorations/grindstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_HONEYCOMB_BLOCK: Self = Self {
        id: "recipes/decorations/honeycomb_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_IRON_BARS: Self = Self {
        id: "recipes/decorations/iron_bars",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_IRON_CHAIN: Self = Self {
        id: "recipes/decorations/iron_chain",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ITEM_FRAME: Self = Self {
        id: "recipes/decorations/item_frame",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_JUKEBOX: Self = Self {
        id: "recipes/decorations/jukebox",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_JUNGLE_FENCE: Self = Self {
        id: "recipes/decorations/jungle_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_JUNGLE_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/jungle_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_JUNGLE_SHELF: Self = Self {
        id: "recipes/decorations/jungle_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_JUNGLE_SIGN: Self = Self {
        id: "recipes/decorations/jungle_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LADDER: Self = Self {
        id: "recipes/decorations/ladder",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LANTERN: Self = Self {
        id: "recipes/decorations/lantern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_BANNER: Self = Self {
        id: "recipes/decorations/light_blue_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_BED: Self = Self {
        id: "recipes/decorations/light_blue_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_CANDLE: Self = Self {
        id: "recipes/decorations/light_blue_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_CARPET: Self = Self {
        id: "recipes/decorations/light_blue_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/light_blue_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/light_blue_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/light_blue_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/light_blue_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_BANNER: Self = Self {
        id: "recipes/decorations/light_gray_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_BED: Self = Self {
        id: "recipes/decorations/light_gray_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_CANDLE: Self = Self {
        id: "recipes/decorations/light_gray_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_CARPET: Self = Self {
        id: "recipes/decorations/light_gray_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/light_gray_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/light_gray_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/light_gray_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/light_gray_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_BANNER: Self = Self {
        id: "recipes/decorations/lime_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_BED: Self = Self {
        id: "recipes/decorations/lime_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_CANDLE: Self = Self {
        id: "recipes/decorations/lime_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_CARPET: Self = Self {
        id: "recipes/decorations/lime_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/lime_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/lime_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/lime_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/lime_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LODESTONE: Self = Self {
        id: "recipes/decorations/lodestone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_LOOM: Self = Self {
        id: "recipes/decorations/loom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_BANNER: Self = Self {
        id: "recipes/decorations/magenta_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_BED: Self = Self {
        id: "recipes/decorations/magenta_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_CANDLE: Self = Self {
        id: "recipes/decorations/magenta_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_CARPET: Self = Self {
        id: "recipes/decorations/magenta_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/magenta_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/magenta_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/magenta_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/magenta_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MANGROVE_FENCE: Self = Self {
        id: "recipes/decorations/mangrove_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MANGROVE_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/mangrove_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MANGROVE_SHELF: Self = Self {
        id: "recipes/decorations/mangrove_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MANGROVE_SIGN: Self = Self {
        id: "recipes/decorations/mangrove_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MOSS_CARPET: Self = Self {
        id: "recipes/decorations/moss_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL: Self = Self {
        id: "recipes/decorations/mossy_cobblestone_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL_FROM_MOSSY_COBBLESTONE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/mossy_cobblestone_wall_from_mossy_cobblestone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL: Self = Self {
        id: "recipes/decorations/mossy_stone_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL_FROM_MOSSY_STONE_BRICK_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/mossy_stone_brick_wall_from_mossy_stone_brick_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_MUD_BRICK_WALL: Self = Self {
        id: "recipes/decorations/mud_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_MUD_BRICK_WALL_FROM_MUD_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/decorations/mud_brick_wall_from_mud_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_NETHER_BRICK_FENCE: Self = Self {
        id: "recipes/decorations/nether_brick_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_NETHER_BRICK_WALL: Self = Self {
        id: "recipes/decorations/nether_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_NETHER_BRICK_WALL_FROM_NETHER_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/decorations/nether_brick_wall_from_nether_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_OAK_FENCE: Self = Self {
        id: "recipes/decorations/oak_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_OAK_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/oak_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_OAK_SHELF: Self = Self {
        id: "recipes/decorations/oak_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_OAK_SIGN: Self = Self {
        id: "recipes/decorations/oak_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_BANNER: Self = Self {
        id: "recipes/decorations/orange_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_BED: Self = Self {
        id: "recipes/decorations/orange_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_CANDLE: Self = Self {
        id: "recipes/decorations/orange_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_CARPET: Self = Self {
        id: "recipes/decorations/orange_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/orange_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/orange_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/orange_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/orange_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PAINTING: Self = Self {
        id: "recipes/decorations/painting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PALE_MOSS_CARPET: Self = Self {
        id: "recipes/decorations/pale_moss_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PALE_OAK_FENCE: Self = Self {
        id: "recipes/decorations/pale_oak_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PALE_OAK_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/pale_oak_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PALE_OAK_SHELF: Self = Self {
        id: "recipes/decorations/pale_oak_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PALE_OAK_SIGN: Self = Self {
        id: "recipes/decorations/pale_oak_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_BANNER: Self = Self {
        id: "recipes/decorations/pink_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_BED: Self = Self {
        id: "recipes/decorations/pink_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_CANDLE: Self = Self {
        id: "recipes/decorations/pink_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_CARPET: Self = Self {
        id: "recipes/decorations/pink_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/pink_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/pink_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/pink_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/pink_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL: Self = Self {
        id: "recipes/decorations/polished_blackstone_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_BLACKSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/decorations/polished_blackstone_brick_wall_from_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING : Self = Self { id : "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_bricks_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING : Self = Self { id : "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_stonecutting" , parent : Some ("minecraft:recipes/root") , send_telemetry : false , display_name : None , } ;
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL: Self = Self {
        id: "recipes/decorations/polished_blackstone_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/polished_blackstone_wall_from_blackstone_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING:
        Self = Self {
        id: "recipes/decorations/polished_blackstone_wall_from_polished_blackstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL: Self = Self {
        id: "recipes/decorations/polished_deepslate_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/decorations/polished_deepslate_wall_from_cobbled_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING:
        Self = Self {
        id: "recipes/decorations/polished_deepslate_wall_from_polished_deepslate_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_TUFF_WALL: Self = Self {
        id: "recipes/decorations/polished_tuff_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_POLISHED_TUFF_STONECUTTING: Self = Self {
        id: "recipes/decorations/polished_tuff_wall_from_polished_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/decorations/polished_tuff_wall_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PRISMARINE_WALL: Self = Self {
        id: "recipes/decorations/prismarine_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PRISMARINE_WALL_FROM_PRISMARINE_STONECUTTING: Self = Self {
        id: "recipes/decorations/prismarine_wall_from_prismarine_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_BANNER: Self = Self {
        id: "recipes/decorations/purple_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_BED: Self = Self {
        id: "recipes/decorations/purple_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_CANDLE: Self = Self {
        id: "recipes/decorations/purple_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_CARPET: Self = Self {
        id: "recipes/decorations/purple_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/purple_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/purple_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/purple_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/purple_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_BANNER: Self = Self {
        id: "recipes/decorations/red_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_BED: Self = Self {
        id: "recipes/decorations/red_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_CANDLE: Self = Self {
        id: "recipes/decorations/red_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_CARPET: Self = Self {
        id: "recipes/decorations/red_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/red_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL: Self = Self {
        id: "recipes/decorations/red_nether_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL_FROM_RED_NETHER_BRICKS_STONECUTTING: Self =
        Self {
            id: "recipes/decorations/red_nether_brick_wall_from_red_nether_bricks_stonecutting",
            parent: Some("minecraft:recipes/root"),
            send_telemetry: false,
            display_name: None,
        };
    pub const RECIPES_DECORATIONS_RED_SANDSTONE_WALL: Self = Self {
        id: "recipes/decorations/red_sandstone_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_SANDSTONE_WALL_FROM_RED_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/decorations/red_sandstone_wall_from_red_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/red_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/red_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/red_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RESIN_BRICK_WALL: Self = Self {
        id: "recipes/decorations/resin_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RESIN_BRICK_WALL_FROM_RESIN_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/decorations/resin_brick_wall_from_resin_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_RESPAWN_ANCHOR: Self = Self {
        id: "recipes/decorations/respawn_anchor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SANDSTONE_WALL: Self = Self {
        id: "recipes/decorations/sandstone_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SANDSTONE_WALL_FROM_SANDSTONE_STONECUTTING: Self = Self {
        id: "recipes/decorations/sandstone_wall_from_sandstone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SCAFFOLDING: Self = Self {
        id: "recipes/decorations/scaffolding",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SMITHING_TABLE: Self = Self {
        id: "recipes/decorations/smithing_table",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SMOKER: Self = Self {
        id: "recipes/decorations/smoker",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SNOW: Self = Self {
        id: "recipes/decorations/snow",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SOUL_CAMPFIRE: Self = Self {
        id: "recipes/decorations/soul_campfire",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SOUL_LANTERN: Self = Self {
        id: "recipes/decorations/soul_lantern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SOUL_TORCH: Self = Self {
        id: "recipes/decorations/soul_torch",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SPRUCE_FENCE: Self = Self {
        id: "recipes/decorations/spruce_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SPRUCE_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/spruce_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SPRUCE_SHELF: Self = Self {
        id: "recipes/decorations/spruce_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_SPRUCE_SIGN: Self = Self {
        id: "recipes/decorations/spruce_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_STONE_BRICK_WALL: Self = Self {
        id: "recipes/decorations/stone_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_STONE_BRICK_WALL_FROM_STONE_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/decorations/stone_brick_wall_from_stone_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_STONE_BRICK_WALLS_FROM_STONE_STONECUTTING: Self = Self {
        id: "recipes/decorations/stone_brick_walls_from_stone_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_STONECUTTER: Self = Self {
        id: "recipes/decorations/stonecutter",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TORCH: Self = Self {
        id: "recipes/decorations/torch",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TUFF_BRICK_WALL: Self = Self {
        id: "recipes/decorations/tuff_brick_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_POLISHED_TUFF_STONECUTTING: Self = Self {
        id: "recipes/decorations/tuff_brick_wall_from_polished_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_BRICKS_STONECUTTING: Self = Self {
        id: "recipes/decorations/tuff_brick_wall_from_tuff_bricks_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/decorations/tuff_brick_wall_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TUFF_WALL: Self = Self {
        id: "recipes/decorations/tuff_wall",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_TUFF_WALL_FROM_TUFF_STONECUTTING: Self = Self {
        id: "recipes/decorations/tuff_wall_from_tuff_stonecutting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WARPED_FENCE: Self = Self {
        id: "recipes/decorations/warped_fence",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WARPED_HANGING_SIGN: Self = Self {
        id: "recipes/decorations/warped_hanging_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WARPED_SHELF: Self = Self {
        id: "recipes/decorations/warped_shelf",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WARPED_SIGN: Self = Self {
        id: "recipes/decorations/warped_sign",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_BANNER: Self = Self {
        id: "recipes/decorations/white_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_BED: Self = Self {
        id: "recipes/decorations/white_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_CANDLE: Self = Self {
        id: "recipes/decorations/white_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_CARPET: Self = Self {
        id: "recipes/decorations/white_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/white_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/white_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/white_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/white_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_BANNER: Self = Self {
        id: "recipes/decorations/yellow_banner",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_BED: Self = Self {
        id: "recipes/decorations/yellow_bed",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_CANDLE: Self = Self {
        id: "recipes/decorations/yellow_candle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_CARPET: Self = Self {
        id: "recipes/decorations/yellow_carpet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_GLAZED_TERRACOTTA: Self = Self {
        id: "recipes/decorations/yellow_glazed_terracotta",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_SHULKER_BOX: Self = Self {
        id: "recipes/decorations/yellow_shulker_box",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE: Self = Self {
        id: "recipes/decorations/yellow_stained_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE_FROM_GLASS_PANE: Self = Self {
        id: "recipes/decorations/yellow_stained_glass_pane_from_glass_pane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_BAKED_POTATO: Self = Self {
        id: "recipes/food/baked_potato",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_BAKED_POTATO_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/baked_potato_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_BAKED_POTATO_FROM_SMOKING: Self = Self {
        id: "recipes/food/baked_potato_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_BEETROOT_SOUP: Self = Self {
        id: "recipes/food/beetroot_soup",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_BREAD: Self = Self {
        id: "recipes/food/bread",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_CAKE: Self = Self {
        id: "recipes/food/cake",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_BEEF: Self = Self {
        id: "recipes/food/cooked_beef",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_BEEF_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_beef_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_BEEF_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_beef_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_CHICKEN: Self = Self {
        id: "recipes/food/cooked_chicken",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_CHICKEN_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_chicken_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_CHICKEN_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_chicken_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_COD: Self = Self {
        id: "recipes/food/cooked_cod",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_COD_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_cod_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_COD_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_cod_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_MUTTON: Self = Self {
        id: "recipes/food/cooked_mutton",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_MUTTON_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_mutton_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_MUTTON_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_mutton_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_PORKCHOP: Self = Self {
        id: "recipes/food/cooked_porkchop",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_PORKCHOP_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_porkchop_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_PORKCHOP_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_porkchop_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_RABBIT: Self = Self {
        id: "recipes/food/cooked_rabbit",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_RABBIT_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_rabbit_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_RABBIT_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_rabbit_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_SALMON: Self = Self {
        id: "recipes/food/cooked_salmon",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_SALMON_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/cooked_salmon_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKED_SALMON_FROM_SMOKING: Self = Self {
        id: "recipes/food/cooked_salmon_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_COOKIE: Self = Self {
        id: "recipes/food/cookie",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_DRIED_KELP: Self = Self {
        id: "recipes/food/dried_kelp",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_DRIED_KELP_FROM_CAMPFIRE_COOKING: Self = Self {
        id: "recipes/food/dried_kelp_from_campfire_cooking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_DRIED_KELP_FROM_SMELTING: Self = Self {
        id: "recipes/food/dried_kelp_from_smelting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_DRIED_KELP_FROM_SMOKING: Self = Self {
        id: "recipes/food/dried_kelp_from_smoking",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_GOLDEN_APPLE: Self = Self {
        id: "recipes/food/golden_apple",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_HONEY_BOTTLE: Self = Self {
        id: "recipes/food/honey_bottle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_MUSHROOM_STEW: Self = Self {
        id: "recipes/food/mushroom_stew",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_PUMPKIN_PIE: Self = Self {
        id: "recipes/food/pumpkin_pie",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_RABBIT_STEW_FROM_BROWN_MUSHROOM: Self = Self {
        id: "recipes/food/rabbit_stew_from_brown_mushroom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_RABBIT_STEW_FROM_RED_MUSHROOM: Self = Self {
        id: "recipes/food/rabbit_stew_from_red_mushroom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ALLIUM: Self = Self {
        id: "recipes/food/suspicious_stew_from_allium",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_AZURE_BLUET: Self = Self {
        id: "recipes/food/suspicious_stew_from_azure_bluet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_BLUE_ORCHID: Self = Self {
        id: "recipes/food/suspicious_stew_from_blue_orchid",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CLOSED_EYEBLOSSOM: Self = Self {
        id: "recipes/food/suspicious_stew_from_closed_eyeblossom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CORNFLOWER: Self = Self {
        id: "recipes/food/suspicious_stew_from_cornflower",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_DANDELION: Self = Self {
        id: "recipes/food/suspicious_stew_from_dandelion",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_LILY_OF_THE_VALLEY: Self = Self {
        id: "recipes/food/suspicious_stew_from_lily_of_the_valley",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OPEN_EYEBLOSSOM: Self = Self {
        id: "recipes/food/suspicious_stew_from_open_eyeblossom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ORANGE_TULIP: Self = Self {
        id: "recipes/food/suspicious_stew_from_orange_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OXEYE_DAISY: Self = Self {
        id: "recipes/food/suspicious_stew_from_oxeye_daisy",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_PINK_TULIP: Self = Self {
        id: "recipes/food/suspicious_stew_from_pink_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_POPPY: Self = Self {
        id: "recipes/food/suspicious_stew_from_poppy",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_RED_TULIP: Self = Self {
        id: "recipes/food/suspicious_stew_from_red_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_TORCHFLOWER: Self = Self {
        id: "recipes/food/suspicious_stew_from_torchflower",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WHITE_TULIP: Self = Self {
        id: "recipes/food/suspicious_stew_from_white_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WITHER_ROSE: Self = Self {
        id: "recipes/food/suspicious_stew_from_wither_rose",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BEACON: Self = Self {
        id: "recipes/misc/beacon",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BLACK_DYE: Self = Self {
        id: "recipes/misc/black_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BLACK_DYE_FROM_WITHER_ROSE: Self = Self {
        id: "recipes/misc/black_dye_from_wither_rose",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BLUE_DYE: Self = Self {
        id: "recipes/misc/blue_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BLUE_DYE_FROM_CORNFLOWER: Self = Self {
        id: "recipes/misc/blue_dye_from_cornflower",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/bolt_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/bolt_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BONE_MEAL: Self = Self {
        id: "recipes/misc/bone_meal",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BONE_MEAL_FROM_BONE_BLOCK: Self = Self {
        id: "recipes/misc/bone_meal_from_bone_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BOOK: Self = Self {
        id: "recipes/misc/book",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BORDURE_INDENTED_BANNER_PATTERN: Self = Self {
        id: "recipes/misc/bordure_indented_banner_pattern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BOWL: Self = Self {
        id: "recipes/misc/bowl",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BRICK: Self = Self {
        id: "recipes/misc/brick",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BROWN_DYE: Self = Self {
        id: "recipes/misc/brown_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_BUCKET: Self = Self {
        id: "recipes/misc/bucket",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_CHARCOAL: Self = Self {
        id: "recipes/misc/charcoal",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAL: Self = Self {
        id: "recipes/misc/coal",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAL_FROM_BLASTING_COAL_ORE: Self = Self {
        id: "recipes/misc/coal_from_blasting_coal_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAL_FROM_BLASTING_DEEPSLATE_COAL_ORE: Self = Self {
        id: "recipes/misc/coal_from_blasting_deepslate_coal_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAL_FROM_SMELTING_COAL_ORE: Self = Self {
        id: "recipes/misc/coal_from_smelting_coal_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAL_FROM_SMELTING_DEEPSLATE_COAL_ORE: Self = Self {
        id: "recipes/misc/coal_from_smelting_deepslate_coal_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/coast_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/coast_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_CONDUIT: Self = Self {
        id: "recipes/misc/conduit",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT: Self = Self {
        id: "recipes/misc/copper_ingot",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_COPPER_ORE: Self = Self {
        id: "recipes/misc/copper_ingot_from_blasting_copper_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_DEEPSLATE_COPPER_ORE: Self = Self {
        id: "recipes/misc/copper_ingot_from_blasting_deepslate_copper_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_RAW_COPPER: Self = Self {
        id: "recipes/misc/copper_ingot_from_blasting_raw_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_NUGGETS: Self = Self {
        id: "recipes/misc/copper_ingot_from_nuggets",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_COPPER_ORE: Self = Self {
        id: "recipes/misc/copper_ingot_from_smelting_copper_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_DEEPSLATE_COPPER_ORE: Self = Self {
        id: "recipes/misc/copper_ingot_from_smelting_deepslate_copper_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_RAW_COPPER: Self = Self {
        id: "recipes/misc/copper_ingot_from_smelting_raw_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_INGOT_FROM_WAXED_COPPER_BLOCK: Self = Self {
        id: "recipes/misc/copper_ingot_from_waxed_copper_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_NUGGET: Self = Self {
        id: "recipes/misc/copper_nugget",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_NUGGET_FROM_BLASTING: Self = Self {
        id: "recipes/misc/copper_nugget_from_blasting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_COPPER_NUGGET_FROM_SMELTING: Self = Self {
        id: "recipes/misc/copper_nugget_from_smelting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_CREAKING_HEART: Self = Self {
        id: "recipes/misc/creaking_heart",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_CREEPER_BANNER_PATTERN: Self = Self {
        id: "recipes/misc/creeper_banner_pattern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_CYAN_DYE: Self = Self {
        id: "recipes/misc/cyan_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_CYAN_DYE_FROM_PITCHER_PLANT: Self = Self {
        id: "recipes/misc/cyan_dye_from_pitcher_plant",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DIAMOND: Self = Self {
        id: "recipes/misc/diamond",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DIAMOND_FROM_BLASTING_DEEPSLATE_DIAMOND_ORE: Self = Self {
        id: "recipes/misc/diamond_from_blasting_deepslate_diamond_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DIAMOND_FROM_BLASTING_DIAMOND_ORE: Self = Self {
        id: "recipes/misc/diamond_from_blasting_diamond_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DIAMOND_FROM_SMELTING_DEEPSLATE_DIAMOND_ORE: Self = Self {
        id: "recipes/misc/diamond_from_smelting_deepslate_diamond_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DIAMOND_FROM_SMELTING_DIAMOND_ORE: Self = Self {
        id: "recipes/misc/diamond_from_smelting_diamond_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/dune_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/dune_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EMERALD: Self = Self {
        id: "recipes/misc/emerald",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EMERALD_FROM_BLASTING_DEEPSLATE_EMERALD_ORE: Self = Self {
        id: "recipes/misc/emerald_from_blasting_deepslate_emerald_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EMERALD_FROM_BLASTING_EMERALD_ORE: Self = Self {
        id: "recipes/misc/emerald_from_blasting_emerald_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EMERALD_FROM_SMELTING_DEEPSLATE_EMERALD_ORE: Self = Self {
        id: "recipes/misc/emerald_from_smelting_deepslate_emerald_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EMERALD_FROM_SMELTING_EMERALD_ORE: Self = Self {
        id: "recipes/misc/emerald_from_smelting_emerald_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_ENDER_EYE: Self = Self {
        id: "recipes/misc/ender_eye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/eye_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/eye_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_FIELD_MASONED_BANNER_PATTERN: Self = Self {
        id: "recipes/misc/field_masoned_banner_pattern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_FIRE_CHARGE: Self = Self {
        id: "recipes/misc/fire_charge",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_FIREWORK_ROCKET_SIMPLE: Self = Self {
        id: "recipes/misc/firework_rocket_simple",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/flow_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/flow_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_FLOWER_BANNER_PATTERN: Self = Self {
        id: "recipes/misc/flower_banner_pattern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_DEEPSLATE_GOLD_ORE: Self = Self {
        id: "recipes/misc/gold_ingot_from_blasting_deepslate_gold_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_GOLD_ORE: Self = Self {
        id: "recipes/misc/gold_ingot_from_blasting_gold_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_NETHER_GOLD_ORE: Self = Self {
        id: "recipes/misc/gold_ingot_from_blasting_nether_gold_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_RAW_GOLD: Self = Self {
        id: "recipes/misc/gold_ingot_from_blasting_raw_gold",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_GOLD_BLOCK: Self = Self {
        id: "recipes/misc/gold_ingot_from_gold_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_NUGGETS: Self = Self {
        id: "recipes/misc/gold_ingot_from_nuggets",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_DEEPSLATE_GOLD_ORE: Self = Self {
        id: "recipes/misc/gold_ingot_from_smelting_deepslate_gold_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_GOLD_ORE: Self = Self {
        id: "recipes/misc/gold_ingot_from_smelting_gold_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_NETHER_GOLD_ORE: Self = Self {
        id: "recipes/misc/gold_ingot_from_smelting_nether_gold_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_RAW_GOLD: Self = Self {
        id: "recipes/misc/gold_ingot_from_smelting_raw_gold",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_NUGGET: Self = Self {
        id: "recipes/misc/gold_nugget",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_NUGGET_FROM_BLASTING: Self = Self {
        id: "recipes/misc/gold_nugget_from_blasting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GOLD_NUGGET_FROM_SMELTING: Self = Self {
        id: "recipes/misc/gold_nugget_from_smelting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GRAY_DYE: Self = Self {
        id: "recipes/misc/gray_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GRAY_DYE_FROM_CLOSED_EYEBLOSSOM: Self = Self {
        id: "recipes/misc/gray_dye_from_closed_eyeblossom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_GREEN_DYE: Self = Self {
        id: "recipes/misc/green_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/host_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/host_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_BLASTING_DEEPSLATE_IRON_ORE: Self = Self {
        id: "recipes/misc/iron_ingot_from_blasting_deepslate_iron_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_BLASTING_IRON_ORE: Self = Self {
        id: "recipes/misc/iron_ingot_from_blasting_iron_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_BLASTING_RAW_IRON: Self = Self {
        id: "recipes/misc/iron_ingot_from_blasting_raw_iron",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_IRON_BLOCK: Self = Self {
        id: "recipes/misc/iron_ingot_from_iron_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_NUGGETS: Self = Self {
        id: "recipes/misc/iron_ingot_from_nuggets",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_SMELTING_DEEPSLATE_IRON_ORE: Self = Self {
        id: "recipes/misc/iron_ingot_from_smelting_deepslate_iron_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_SMELTING_IRON_ORE: Self = Self {
        id: "recipes/misc/iron_ingot_from_smelting_iron_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_INGOT_FROM_SMELTING_RAW_IRON: Self = Self {
        id: "recipes/misc/iron_ingot_from_smelting_raw_iron",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_NUGGET: Self = Self {
        id: "recipes/misc/iron_nugget",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_NUGGET_FROM_BLASTING: Self = Self {
        id: "recipes/misc/iron_nugget_from_blasting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_IRON_NUGGET_FROM_SMELTING: Self = Self {
        id: "recipes/misc/iron_nugget_from_smelting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LAPIS_LAZULI: Self = Self {
        id: "recipes/misc/lapis_lazuli",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_DEEPSLATE_LAPIS_ORE: Self = Self {
        id: "recipes/misc/lapis_lazuli_from_blasting_deepslate_lapis_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_LAPIS_ORE: Self = Self {
        id: "recipes/misc/lapis_lazuli_from_blasting_lapis_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_DEEPSLATE_LAPIS_ORE: Self = Self {
        id: "recipes/misc/lapis_lazuli_from_smelting_deepslate_lapis_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_LAPIS_ORE: Self = Self {
        id: "recipes/misc/lapis_lazuli_from_smelting_lapis_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LEAF_LITTER: Self = Self {
        id: "recipes/misc/leaf_litter",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LEATHER: Self = Self {
        id: "recipes/misc/leather",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LEATHER_HORSE_ARMOR: Self = Self {
        id: "recipes/misc/leather_horse_armor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_ORCHID: Self = Self {
        id: "recipes/misc/light_blue_dye_from_blue_orchid",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_WHITE_DYE: Self = Self {
        id: "recipes/misc/light_blue_dye_from_blue_white_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_GRAY_DYE_FROM_AZURE_BLUET: Self = Self {
        id: "recipes/misc/light_gray_dye_from_azure_bluet",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_GRAY_DYE_FROM_BLACK_WHITE_DYE: Self = Self {
        id: "recipes/misc/light_gray_dye_from_black_white_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_GRAY_DYE_FROM_GRAY_WHITE_DYE: Self = Self {
        id: "recipes/misc/light_gray_dye_from_gray_white_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_GRAY_DYE_FROM_OXEYE_DAISY: Self = Self {
        id: "recipes/misc/light_gray_dye_from_oxeye_daisy",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIGHT_GRAY_DYE_FROM_WHITE_TULIP: Self = Self {
        id: "recipes/misc/light_gray_dye_from_white_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIME_DYE: Self = Self {
        id: "recipes/misc/lime_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_LIME_DYE_FROM_SMELTING: Self = Self {
        id: "recipes/misc/lime_dye_from_smelting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MAGENTA_DYE_FROM_ALLIUM: Self = Self {
        id: "recipes/misc/magenta_dye_from_allium",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_PINK: Self = Self {
        id: "recipes/misc/magenta_dye_from_blue_red_pink",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_WHITE_DYE: Self = Self {
        id: "recipes/misc/magenta_dye_from_blue_red_white_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MAGENTA_DYE_FROM_LILAC: Self = Self {
        id: "recipes/misc/magenta_dye_from_lilac",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MAGENTA_DYE_FROM_PURPLE_AND_PINK: Self = Self {
        id: "recipes/misc/magenta_dye_from_purple_and_pink",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MAP: Self = Self {
        id: "recipes/misc/map",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MELON_SEEDS: Self = Self {
        id: "recipes/misc/melon_seeds",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MOJANG_BANNER_PATTERN: Self = Self {
        id: "recipes/misc/mojang_banner_pattern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_MUSIC_DISC_5: Self = Self {
        id: "recipes/misc/music_disc_5",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_NETHER_BRICK: Self = Self {
        id: "recipes/misc/nether_brick",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_NETHERITE_INGOT: Self = Self {
        id: "recipes/misc/netherite_ingot",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_NETHERITE_INGOT_FROM_NETHERITE_BLOCK: Self = Self {
        id: "recipes/misc/netherite_ingot_from_netherite_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_NETHERITE_SCRAP: Self = Self {
        id: "recipes/misc/netherite_scrap",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_NETHERITE_SCRAP_FROM_BLASTING: Self = Self {
        id: "recipes/misc/netherite_scrap_from_blasting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_NETHERITE_UPGRADE_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/netherite_upgrade_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_ORANGE_DYE_FROM_OPEN_EYEBLOSSOM: Self = Self {
        id: "recipes/misc/orange_dye_from_open_eyeblossom",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_ORANGE_DYE_FROM_ORANGE_TULIP: Self = Self {
        id: "recipes/misc/orange_dye_from_orange_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_ORANGE_DYE_FROM_RED_YELLOW: Self = Self {
        id: "recipes/misc/orange_dye_from_red_yellow",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_ORANGE_DYE_FROM_TORCHFLOWER: Self = Self {
        id: "recipes/misc/orange_dye_from_torchflower",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PAPER: Self = Self {
        id: "recipes/misc/paper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PINK_DYE_FROM_CACTUS_FLOWER: Self = Self {
        id: "recipes/misc/pink_dye_from_cactus_flower",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PINK_DYE_FROM_PEONY: Self = Self {
        id: "recipes/misc/pink_dye_from_peony",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PINK_DYE_FROM_PINK_PETALS: Self = Self {
        id: "recipes/misc/pink_dye_from_pink_petals",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PINK_DYE_FROM_PINK_TULIP: Self = Self {
        id: "recipes/misc/pink_dye_from_pink_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PINK_DYE_FROM_RED_WHITE_DYE: Self = Self {
        id: "recipes/misc/pink_dye_from_red_white_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_POPPED_CHORUS_FRUIT: Self = Self {
        id: "recipes/misc/popped_chorus_fruit",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PUMPKIN_SEEDS: Self = Self {
        id: "recipes/misc/pumpkin_seeds",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_PURPLE_DYE: Self = Self {
        id: "recipes/misc/purple_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_QUARTZ: Self = Self {
        id: "recipes/misc/quartz",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_QUARTZ_FROM_BLASTING: Self = Self {
        id: "recipes/misc/quartz_from_blasting",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/raiser_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/raiser_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RAW_COPPER: Self = Self {
        id: "recipes/misc/raw_copper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RAW_GOLD: Self = Self {
        id: "recipes/misc/raw_gold",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RAW_IRON: Self = Self {
        id: "recipes/misc/raw_iron",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RED_DYE_FROM_BEETROOT: Self = Self {
        id: "recipes/misc/red_dye_from_beetroot",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RED_DYE_FROM_POPPY: Self = Self {
        id: "recipes/misc/red_dye_from_poppy",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RED_DYE_FROM_ROSE_BUSH: Self = Self {
        id: "recipes/misc/red_dye_from_rose_bush",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RED_DYE_FROM_TULIP: Self = Self {
        id: "recipes/misc/red_dye_from_tulip",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RESIN_BRICK: Self = Self {
        id: "recipes/misc/resin_brick",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RESIN_CLUMP: Self = Self {
        id: "recipes/misc/resin_clump",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/rib_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/rib_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/sentry_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/sentry_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/shaper_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/shaper_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/silence_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/silence_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SKULL_BANNER_PATTERN: Self = Self {
        id: "recipes/misc/skull_banner_pattern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SLIME_BALL: Self = Self {
        id: "recipes/misc/slime_ball",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/snout_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/snout_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/spire_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/spire_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_STICK: Self = Self {
        id: "recipes/misc/stick",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_STICK_FROM_BAMBOO_ITEM: Self = Self {
        id: "recipes/misc/stick_from_bamboo_item",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SUGAR_FROM_HONEY_BOTTLE: Self = Self {
        id: "recipes/misc/sugar_from_honey_bottle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_SUGAR_FROM_SUGAR_CANE: Self = Self {
        id: "recipes/misc/sugar_from_sugar_cane",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/tide_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/tide_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/vex_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/vex_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/ward_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/ward_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/wayfinder_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/wayfinder_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WHEAT: Self = Self {
        id: "recipes/misc/wheat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WHITE_DYE: Self = Self {
        id: "recipes/misc/white_dye",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WHITE_DYE_FROM_LILY_OF_THE_VALLEY: Self = Self {
        id: "recipes/misc/white_dye_from_lily_of_the_valley",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: "recipes/misc/wild_armor_trim_smithing_template",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM: Self = Self {
        id: "recipes/misc/wild_armor_trim_smithing_template_smithing_trim",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WIND_CHARGE: Self = Self {
        id: "recipes/misc/wind_charge",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_WRITABLE_BOOK: Self = Self {
        id: "recipes/misc/writable_book",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_YELLOW_DYE_FROM_DANDELION: Self = Self {
        id: "recipes/misc/yellow_dye_from_dandelion",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_YELLOW_DYE_FROM_SUNFLOWER: Self = Self {
        id: "recipes/misc/yellow_dye_from_sunflower",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_MISC_YELLOW_DYE_FROM_WILDFLOWERS: Self = Self {
        id: "recipes/misc/yellow_dye_from_wildflowers",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_ACACIA_BUTTON: Self = Self {
        id: "recipes/redstone/acacia_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_ACACIA_DOOR: Self = Self {
        id: "recipes/redstone/acacia_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_ACACIA_FENCE_GATE: Self = Self {
        id: "recipes/redstone/acacia_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_ACACIA_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/acacia_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_ACACIA_TRAPDOOR: Self = Self {
        id: "recipes/redstone/acacia_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BAMBOO_BUTTON: Self = Self {
        id: "recipes/redstone/bamboo_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BAMBOO_DOOR: Self = Self {
        id: "recipes/redstone/bamboo_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BAMBOO_FENCE_GATE: Self = Self {
        id: "recipes/redstone/bamboo_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BAMBOO_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/bamboo_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BAMBOO_TRAPDOOR: Self = Self {
        id: "recipes/redstone/bamboo_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BIRCH_BUTTON: Self = Self {
        id: "recipes/redstone/birch_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BIRCH_DOOR: Self = Self {
        id: "recipes/redstone/birch_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BIRCH_FENCE_GATE: Self = Self {
        id: "recipes/redstone/birch_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BIRCH_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/birch_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_BIRCH_TRAPDOOR: Self = Self {
        id: "recipes/redstone/birch_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CALIBRATED_SCULK_SENSOR: Self = Self {
        id: "recipes/redstone/calibrated_sculk_sensor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CHERRY_BUTTON: Self = Self {
        id: "recipes/redstone/cherry_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CHERRY_DOOR: Self = Self {
        id: "recipes/redstone/cherry_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CHERRY_FENCE_GATE: Self = Self {
        id: "recipes/redstone/cherry_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CHERRY_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/cherry_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CHERRY_TRAPDOOR: Self = Self {
        id: "recipes/redstone/cherry_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_COMPARATOR: Self = Self {
        id: "recipes/redstone/comparator",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_COPPER_BULB: Self = Self {
        id: "recipes/redstone/copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_COPPER_DOOR: Self = Self {
        id: "recipes/redstone/copper_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_COPPER_TRAPDOOR: Self = Self {
        id: "recipes/redstone/copper_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CRAFTER: Self = Self {
        id: "recipes/redstone/crafter",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CRIMSON_BUTTON: Self = Self {
        id: "recipes/redstone/crimson_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CRIMSON_DOOR: Self = Self {
        id: "recipes/redstone/crimson_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CRIMSON_FENCE_GATE: Self = Self {
        id: "recipes/redstone/crimson_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CRIMSON_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/crimson_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_CRIMSON_TRAPDOOR: Self = Self {
        id: "recipes/redstone/crimson_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DARK_OAK_BUTTON: Self = Self {
        id: "recipes/redstone/dark_oak_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DARK_OAK_DOOR: Self = Self {
        id: "recipes/redstone/dark_oak_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DARK_OAK_FENCE_GATE: Self = Self {
        id: "recipes/redstone/dark_oak_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DARK_OAK_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/dark_oak_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DARK_OAK_TRAPDOOR: Self = Self {
        id: "recipes/redstone/dark_oak_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DAYLIGHT_DETECTOR: Self = Self {
        id: "recipes/redstone/daylight_detector",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DISPENSER: Self = Self {
        id: "recipes/redstone/dispenser",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_DROPPER: Self = Self {
        id: "recipes/redstone/dropper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_EXPOSED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/exposed_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_HEAVY_WEIGHTED_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/heavy_weighted_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_HONEY_BLOCK: Self = Self {
        id: "recipes/redstone/honey_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_HOPPER: Self = Self {
        id: "recipes/redstone/hopper",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_IRON_DOOR: Self = Self {
        id: "recipes/redstone/iron_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_IRON_TRAPDOOR: Self = Self {
        id: "recipes/redstone/iron_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_JUNGLE_BUTTON: Self = Self {
        id: "recipes/redstone/jungle_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_JUNGLE_DOOR: Self = Self {
        id: "recipes/redstone/jungle_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_JUNGLE_FENCE_GATE: Self = Self {
        id: "recipes/redstone/jungle_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_JUNGLE_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/jungle_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_JUNGLE_TRAPDOOR: Self = Self {
        id: "recipes/redstone/jungle_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_LECTERN: Self = Self {
        id: "recipes/redstone/lectern",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_LEVER: Self = Self {
        id: "recipes/redstone/lever",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_LIGHT_WEIGHTED_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/light_weighted_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_LIGHTNING_ROD: Self = Self {
        id: "recipes/redstone/lightning_rod",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_MANGROVE_BUTTON: Self = Self {
        id: "recipes/redstone/mangrove_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_MANGROVE_DOOR: Self = Self {
        id: "recipes/redstone/mangrove_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_MANGROVE_FENCE_GATE: Self = Self {
        id: "recipes/redstone/mangrove_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_MANGROVE_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/mangrove_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_MANGROVE_TRAPDOOR: Self = Self {
        id: "recipes/redstone/mangrove_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_NOTE_BLOCK: Self = Self {
        id: "recipes/redstone/note_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OAK_BUTTON: Self = Self {
        id: "recipes/redstone/oak_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OAK_DOOR: Self = Self {
        id: "recipes/redstone/oak_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OAK_FENCE_GATE: Self = Self {
        id: "recipes/redstone/oak_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OAK_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/oak_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OAK_TRAPDOOR: Self = Self {
        id: "recipes/redstone/oak_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OBSERVER: Self = Self {
        id: "recipes/redstone/observer",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_OXIDIZED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/oxidized_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_PALE_OAK_BUTTON: Self = Self {
        id: "recipes/redstone/pale_oak_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_PALE_OAK_DOOR: Self = Self {
        id: "recipes/redstone/pale_oak_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_PALE_OAK_FENCE_GATE: Self = Self {
        id: "recipes/redstone/pale_oak_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_PALE_OAK_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/pale_oak_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_PALE_OAK_TRAPDOOR: Self = Self {
        id: "recipes/redstone/pale_oak_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_PISTON: Self = Self {
        id: "recipes/redstone/piston",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_POLISHED_BLACKSTONE_BUTTON: Self = Self {
        id: "recipes/redstone/polished_blackstone_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_POLISHED_BLACKSTONE_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/polished_blackstone_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE: Self = Self {
        id: "recipes/redstone/redstone",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_BLOCK: Self = Self {
        id: "recipes/redstone/redstone_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_DEEPSLATE_REDSTONE_ORE: Self = Self {
        id: "recipes/redstone/redstone_from_blasting_deepslate_redstone_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_REDSTONE_ORE: Self = Self {
        id: "recipes/redstone/redstone_from_blasting_redstone_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_DEEPSLATE_REDSTONE_ORE: Self = Self {
        id: "recipes/redstone/redstone_from_smelting_deepslate_redstone_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_REDSTONE_ORE: Self = Self {
        id: "recipes/redstone/redstone_from_smelting_redstone_ore",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_LAMP: Self = Self {
        id: "recipes/redstone/redstone_lamp",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REDSTONE_TORCH: Self = Self {
        id: "recipes/redstone/redstone_torch",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_REPEATER: Self = Self {
        id: "recipes/redstone/repeater",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_SLIME_BLOCK: Self = Self {
        id: "recipes/redstone/slime_block",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_SPRUCE_BUTTON: Self = Self {
        id: "recipes/redstone/spruce_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_SPRUCE_DOOR: Self = Self {
        id: "recipes/redstone/spruce_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_SPRUCE_FENCE_GATE: Self = Self {
        id: "recipes/redstone/spruce_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_SPRUCE_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/spruce_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_SPRUCE_TRAPDOOR: Self = Self {
        id: "recipes/redstone/spruce_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_STICKY_PISTON: Self = Self {
        id: "recipes/redstone/sticky_piston",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_STONE_BUTTON: Self = Self {
        id: "recipes/redstone/stone_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_STONE_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/stone_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_TARGET: Self = Self {
        id: "recipes/redstone/target",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_TNT: Self = Self {
        id: "recipes/redstone/tnt",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_TRAPPED_CHEST: Self = Self {
        id: "recipes/redstone/trapped_chest",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_TRIPWIRE_HOOK: Self = Self {
        id: "recipes/redstone/tripwire_hook",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WARPED_BUTTON: Self = Self {
        id: "recipes/redstone/warped_button",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WARPED_DOOR: Self = Self {
        id: "recipes/redstone/warped_door",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WARPED_FENCE_GATE: Self = Self {
        id: "recipes/redstone/warped_fence_gate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WARPED_PRESSURE_PLATE: Self = Self {
        id: "recipes/redstone/warped_pressure_plate",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WARPED_TRAPDOOR: Self = Self {
        id: "recipes/redstone/warped_trapdoor",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/waxed_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_COPPER_BULB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_copper_bulb_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_COPPER_DOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_copper_door_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_COPPER_TRAPDOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_copper_trapdoor_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/waxed_exposed_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_exposed_copper_bulb_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_DOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_exposed_copper_door_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_TRAPDOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_exposed_copper_trapdoor_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/waxed_oxidized_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_oxidized_copper_bulb_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_DOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_oxidized_copper_door_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_TRAPDOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_oxidized_copper_trapdoor_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/waxed_weathered_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_weathered_copper_bulb_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_DOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_weathered_copper_door_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_TRAPDOOR_FROM_HONEYCOMB: Self = Self {
        id: "recipes/redstone/waxed_weathered_copper_trapdoor_from_honeycomb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_REDSTONE_WEATHERED_COPPER_BULB: Self = Self {
        id: "recipes/redstone/weathered_copper_bulb",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_ROOT: Self = Self {
        id: "recipes/root",
        parent: None,
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_BLACK_BUNDLE: Self = Self {
        id: "recipes/tools/black_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_BLUE_BUNDLE: Self = Self {
        id: "recipes/tools/blue_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_BROWN_BUNDLE: Self = Self {
        id: "recipes/tools/brown_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_BRUSH: Self = Self {
        id: "recipes/tools/brush",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_BUNDLE: Self = Self {
        id: "recipes/tools/bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_CLOCK: Self = Self {
        id: "recipes/tools/clock",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_COMPASS: Self = Self {
        id: "recipes/tools/compass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_COPPER_AXE: Self = Self {
        id: "recipes/tools/copper_axe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_COPPER_HOE: Self = Self {
        id: "recipes/tools/copper_hoe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_COPPER_PICKAXE: Self = Self {
        id: "recipes/tools/copper_pickaxe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_COPPER_SHOVEL: Self = Self {
        id: "recipes/tools/copper_shovel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_CYAN_BUNDLE: Self = Self {
        id: "recipes/tools/cyan_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_DIAMOND_AXE: Self = Self {
        id: "recipes/tools/diamond_axe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_DIAMOND_HOE: Self = Self {
        id: "recipes/tools/diamond_hoe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_DIAMOND_PICKAXE: Self = Self {
        id: "recipes/tools/diamond_pickaxe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_DIAMOND_SHOVEL: Self = Self {
        id: "recipes/tools/diamond_shovel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_FISHING_ROD: Self = Self {
        id: "recipes/tools/fishing_rod",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_FLINT_AND_STEEL: Self = Self {
        id: "recipes/tools/flint_and_steel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_GOLDEN_AXE: Self = Self {
        id: "recipes/tools/golden_axe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_GOLDEN_HOE: Self = Self {
        id: "recipes/tools/golden_hoe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_GOLDEN_PICKAXE: Self = Self {
        id: "recipes/tools/golden_pickaxe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_GOLDEN_SHOVEL: Self = Self {
        id: "recipes/tools/golden_shovel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_GRAY_BUNDLE: Self = Self {
        id: "recipes/tools/gray_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_GREEN_BUNDLE: Self = Self {
        id: "recipes/tools/green_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_IRON_AXE: Self = Self {
        id: "recipes/tools/iron_axe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_IRON_HOE: Self = Self {
        id: "recipes/tools/iron_hoe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_IRON_PICKAXE: Self = Self {
        id: "recipes/tools/iron_pickaxe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_IRON_SHOVEL: Self = Self {
        id: "recipes/tools/iron_shovel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_LEAD: Self = Self {
        id: "recipes/tools/lead",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_LIGHT_BLUE_BUNDLE: Self = Self {
        id: "recipes/tools/light_blue_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_LIGHT_GRAY_BUNDLE: Self = Self {
        id: "recipes/tools/light_gray_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_LIME_BUNDLE: Self = Self {
        id: "recipes/tools/lime_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_MAGENTA_BUNDLE: Self = Self {
        id: "recipes/tools/magenta_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_NETHERITE_AXE_SMITHING: Self = Self {
        id: "recipes/tools/netherite_axe_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_NETHERITE_HOE_SMITHING: Self = Self {
        id: "recipes/tools/netherite_hoe_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_NETHERITE_PICKAXE_SMITHING: Self = Self {
        id: "recipes/tools/netherite_pickaxe_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_NETHERITE_SHOVEL_SMITHING: Self = Self {
        id: "recipes/tools/netherite_shovel_smithing",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_ORANGE_BUNDLE: Self = Self {
        id: "recipes/tools/orange_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_PINK_BUNDLE: Self = Self {
        id: "recipes/tools/pink_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_PURPLE_BUNDLE: Self = Self {
        id: "recipes/tools/purple_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_RECOVERY_COMPASS: Self = Self {
        id: "recipes/tools/recovery_compass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_RED_BUNDLE: Self = Self {
        id: "recipes/tools/red_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_SHEARS: Self = Self {
        id: "recipes/tools/shears",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_SPYGLASS: Self = Self {
        id: "recipes/tools/spyglass",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_STONE_AXE: Self = Self {
        id: "recipes/tools/stone_axe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_STONE_HOE: Self = Self {
        id: "recipes/tools/stone_hoe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_STONE_PICKAXE: Self = Self {
        id: "recipes/tools/stone_pickaxe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_STONE_SHOVEL: Self = Self {
        id: "recipes/tools/stone_shovel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_WHITE_BUNDLE: Self = Self {
        id: "recipes/tools/white_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_WOODEN_AXE: Self = Self {
        id: "recipes/tools/wooden_axe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_WOODEN_HOE: Self = Self {
        id: "recipes/tools/wooden_hoe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_WOODEN_PICKAXE: Self = Self {
        id: "recipes/tools/wooden_pickaxe",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_WOODEN_SHOVEL: Self = Self {
        id: "recipes/tools/wooden_shovel",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TOOLS_YELLOW_BUNDLE: Self = Self {
        id: "recipes/tools/yellow_bundle",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_ACACIA_BOAT: Self = Self {
        id: "recipes/transportation/acacia_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_ACACIA_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/acacia_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_ACTIVATOR_RAIL: Self = Self {
        id: "recipes/transportation/activator_rail",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_BAMBOO_CHEST_RAFT: Self = Self {
        id: "recipes/transportation/bamboo_chest_raft",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_BAMBOO_RAFT: Self = Self {
        id: "recipes/transportation/bamboo_raft",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_BIRCH_BOAT: Self = Self {
        id: "recipes/transportation/birch_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_BIRCH_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/birch_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_CARROT_ON_A_STICK: Self = Self {
        id: "recipes/transportation/carrot_on_a_stick",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_CHERRY_BOAT: Self = Self {
        id: "recipes/transportation/cherry_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_CHERRY_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/cherry_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_CHEST_MINECART: Self = Self {
        id: "recipes/transportation/chest_minecart",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_DARK_OAK_BOAT: Self = Self {
        id: "recipes/transportation/dark_oak_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_DARK_OAK_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/dark_oak_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_DETECTOR_RAIL: Self = Self {
        id: "recipes/transportation/detector_rail",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_FURNACE_MINECART: Self = Self {
        id: "recipes/transportation/furnace_minecart",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_HOPPER_MINECART: Self = Self {
        id: "recipes/transportation/hopper_minecart",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_JUNGLE_BOAT: Self = Self {
        id: "recipes/transportation/jungle_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_JUNGLE_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/jungle_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_MANGROVE_BOAT: Self = Self {
        id: "recipes/transportation/mangrove_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_MANGROVE_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/mangrove_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_MINECART: Self = Self {
        id: "recipes/transportation/minecart",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_OAK_BOAT: Self = Self {
        id: "recipes/transportation/oak_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_OAK_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/oak_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_PALE_OAK_BOAT: Self = Self {
        id: "recipes/transportation/pale_oak_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_PALE_OAK_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/pale_oak_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_POWERED_RAIL: Self = Self {
        id: "recipes/transportation/powered_rail",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_RAIL: Self = Self {
        id: "recipes/transportation/rail",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_SPRUCE_BOAT: Self = Self {
        id: "recipes/transportation/spruce_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_SPRUCE_CHEST_BOAT: Self = Self {
        id: "recipes/transportation/spruce_chest_boat",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_TNT_MINECART: Self = Self {
        id: "recipes/transportation/tnt_minecart",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const RECIPES_TRANSPORTATION_WARPED_FUNGUS_ON_A_STICK: Self = Self {
        id: "recipes/transportation/warped_fungus_on_a_stick",
        parent: Some("minecraft:recipes/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_CURE_ZOMBIE_VILLAGER: Self = Self {
        id: "story/cure_zombie_villager",
        parent: Some("minecraft:story/enter_the_nether"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_DEFLECT_ARROW: Self = Self {
        id: "story/deflect_arrow",
        parent: Some("minecraft:story/obtain_armor"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_ENCHANT_ITEM: Self = Self {
        id: "story/enchant_item",
        parent: Some("minecraft:story/mine_diamond"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_ENTER_THE_END: Self = Self {
        id: "story/enter_the_end",
        parent: Some("minecraft:story/follow_ender_eye"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_ENTER_THE_NETHER: Self = Self {
        id: "story/enter_the_nether",
        parent: Some("minecraft:story/form_obsidian"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_FOLLOW_ENDER_EYE: Self = Self {
        id: "story/follow_ender_eye",
        parent: Some("minecraft:story/enter_the_nether"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_FORM_OBSIDIAN: Self = Self {
        id: "story/form_obsidian",
        parent: Some("minecraft:story/lava_bucket"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_IRON_TOOLS: Self = Self {
        id: "story/iron_tools",
        parent: Some("minecraft:story/smelt_iron"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_LAVA_BUCKET: Self = Self {
        id: "story/lava_bucket",
        parent: Some("minecraft:story/smelt_iron"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_MINE_DIAMOND: Self = Self {
        id: "story/mine_diamond",
        parent: Some("minecraft:story/iron_tools"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_MINE_STONE: Self = Self {
        id: "story/mine_stone",
        parent: Some("minecraft:story/root"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_OBTAIN_ARMOR: Self = Self {
        id: "story/obtain_armor",
        parent: Some("minecraft:story/smelt_iron"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_ROOT: Self = Self {
        id: "story/root",
        parent: None,
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_SHINY_GEAR: Self = Self {
        id: "story/shiny_gear",
        parent: Some("minecraft:story/mine_diamond"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_SMELT_IRON: Self = Self {
        id: "story/smelt_iron",
        parent: Some("minecraft:story/upgrade_tools"),
        send_telemetry: false,
        display_name: None,
    };
    pub const STORY_UPGRADE_TOOLS: Self = Self {
        id: "story/upgrade_tools",
        parent: Some("minecraft:story/mine_stone"),
        send_telemetry: false,
        display_name: None,
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name { "adventure/adventuring_time" => Some (& Self :: ADVENTURE_ADVENTURING_TIME) , "adventure/arbalistic" => Some (& Self :: ADVENTURE_ARBALISTIC) , "adventure/avoid_vibration" => Some (& Self :: ADVENTURE_AVOID_VIBRATION) , "adventure/blowback" => Some (& Self :: ADVENTURE_BLOWBACK) , "adventure/brush_armadillo" => Some (& Self :: ADVENTURE_BRUSH_ARMADILLO) , "adventure/bullseye" => Some (& Self :: ADVENTURE_BULLSEYE) , "adventure/craft_decorated_pot_using_only_sherds" => Some (& Self :: ADVENTURE_CRAFT_DECORATED_POT_USING_ONLY_SHERDS) , "adventure/crafters_crafting_crafters" => Some (& Self :: ADVENTURE_CRAFTERS_CRAFTING_CRAFTERS) , "adventure/fall_from_world_height" => Some (& Self :: ADVENTURE_FALL_FROM_WORLD_HEIGHT) , "adventure/heart_transplanter" => Some (& Self :: ADVENTURE_HEART_TRANSPLANTER) , "adventure/hero_of_the_village" => Some (& Self :: ADVENTURE_HERO_OF_THE_VILLAGE) , "adventure/honey_block_slide" => Some (& Self :: ADVENTURE_HONEY_BLOCK_SLIDE) , "adventure/kill_a_mob" => Some (& Self :: ADVENTURE_KILL_A_MOB) , "adventure/kill_all_mobs" => Some (& Self :: ADVENTURE_KILL_ALL_MOBS) , "adventure/kill_mob_near_sculk_catalyst" => Some (& Self :: ADVENTURE_KILL_MOB_NEAR_SCULK_CATALYST) , "adventure/lighten_up" => Some (& Self :: ADVENTURE_LIGHTEN_UP) , "adventure/lightning_rod_with_villager_no_fire" => Some (& Self :: ADVENTURE_LIGHTNING_ROD_WITH_VILLAGER_NO_FIRE) , "adventure/minecraft_trials_edition" => Some (& Self :: ADVENTURE_MINECRAFT_TRIALS_EDITION) , "adventure/ol_betsy" => Some (& Self :: ADVENTURE_OL_BETSY) , "adventure/overoverkill" => Some (& Self :: ADVENTURE_OVEROVERKILL) , "adventure/play_jukebox_in_meadows" => Some (& Self :: ADVENTURE_PLAY_JUKEBOX_IN_MEADOWS) , "adventure/read_power_of_chiseled_bookshelf" => Some (& Self :: ADVENTURE_READ_POWER_OF_CHISELED_BOOKSHELF) , "adventure/revaulting" => Some (& Self :: ADVENTURE_REVAULTING) , "adventure/root" => Some (& Self :: ADVENTURE_ROOT) , "adventure/salvage_sherd" => Some (& Self :: ADVENTURE_SALVAGE_SHERD) , "adventure/shoot_arrow" => Some (& Self :: ADVENTURE_SHOOT_ARROW) , "adventure/sleep_in_bed" => Some (& Self :: ADVENTURE_SLEEP_IN_BED) , "adventure/sniper_duel" => Some (& Self :: ADVENTURE_SNIPER_DUEL) , "adventure/spear_many_mobs" => Some (& Self :: ADVENTURE_SPEAR_MANY_MOBS) , "adventure/spyglass_at_dragon" => Some (& Self :: ADVENTURE_SPYGLASS_AT_DRAGON) , "adventure/spyglass_at_ghast" => Some (& Self :: ADVENTURE_SPYGLASS_AT_GHAST) , "adventure/spyglass_at_parrot" => Some (& Self :: ADVENTURE_SPYGLASS_AT_PARROT) , "adventure/summon_iron_golem" => Some (& Self :: ADVENTURE_SUMMON_IRON_GOLEM) , "adventure/throw_trident" => Some (& Self :: ADVENTURE_THROW_TRIDENT) , "adventure/totem_of_undying" => Some (& Self :: ADVENTURE_TOTEM_OF_UNDYING) , "adventure/trade" => Some (& Self :: ADVENTURE_TRADE) , "adventure/trade_at_world_height" => Some (& Self :: ADVENTURE_TRADE_AT_WORLD_HEIGHT) , "adventure/trim_with_all_exclusive_armor_patterns" => Some (& Self :: ADVENTURE_TRIM_WITH_ALL_EXCLUSIVE_ARMOR_PATTERNS) , "adventure/trim_with_any_armor_pattern" => Some (& Self :: ADVENTURE_TRIM_WITH_ANY_ARMOR_PATTERN) , "adventure/two_birds_one_arrow" => Some (& Self :: ADVENTURE_TWO_BIRDS_ONE_ARROW) , "adventure/under_lock_and_key" => Some (& Self :: ADVENTURE_UNDER_LOCK_AND_KEY) , "adventure/use_lodestone" => Some (& Self :: ADVENTURE_USE_LODESTONE) , "adventure/very_very_frightening" => Some (& Self :: ADVENTURE_VERY_VERY_FRIGHTENING) , "adventure/voluntary_exile" => Some (& Self :: ADVENTURE_VOLUNTARY_EXILE) , "adventure/walk_on_powder_snow_with_leather_boots" => Some (& Self :: ADVENTURE_WALK_ON_POWDER_SNOW_WITH_LEATHER_BOOTS) , "adventure/who_needs_rockets" => Some (& Self :: ADVENTURE_WHO_NEEDS_ROCKETS) , "adventure/whos_the_pillager_now" => Some (& Self :: ADVENTURE_WHOS_THE_PILLAGER_NOW) , "end/dragon_breath" => Some (& Self :: END_DRAGON_BREATH) , "end/dragon_egg" => Some (& Self :: END_DRAGON_EGG) , "end/elytra" => Some (& Self :: END_ELYTRA) , "end/enter_end_gateway" => Some (& Self :: END_ENTER_END_GATEWAY) , "end/find_end_city" => Some (& Self :: END_FIND_END_CITY) , "end/kill_dragon" => Some (& Self :: END_KILL_DRAGON) , "end/levitate" => Some (& Self :: END_LEVITATE) , "end/respawn_dragon" => Some (& Self :: END_RESPAWN_DRAGON) , "end/root" => Some (& Self :: END_ROOT) , "husbandry/allay_deliver_cake_to_note_block" => Some (& Self :: HUSBANDRY_ALLAY_DELIVER_CAKE_TO_NOTE_BLOCK) , "husbandry/allay_deliver_item_to_player" => Some (& Self :: HUSBANDRY_ALLAY_DELIVER_ITEM_TO_PLAYER) , "husbandry/axolotl_in_a_bucket" => Some (& Self :: HUSBANDRY_AXOLOTL_IN_A_BUCKET) , "husbandry/balanced_diet" => Some (& Self :: HUSBANDRY_BALANCED_DIET) , "husbandry/bred_all_animals" => Some (& Self :: HUSBANDRY_BRED_ALL_ANIMALS) , "husbandry/breed_an_animal" => Some (& Self :: HUSBANDRY_BREED_AN_ANIMAL) , "husbandry/complete_catalogue" => Some (& Self :: HUSBANDRY_COMPLETE_CATALOGUE) , "husbandry/feed_snifflet" => Some (& Self :: HUSBANDRY_FEED_SNIFFLET) , "husbandry/fishy_business" => Some (& Self :: HUSBANDRY_FISHY_BUSINESS) , "husbandry/froglights" => Some (& Self :: HUSBANDRY_FROGLIGHTS) , "husbandry/kill_axolotl_target" => Some (& Self :: HUSBANDRY_KILL_AXOLOTL_TARGET) , "husbandry/leash_all_frog_variants" => Some (& Self :: HUSBANDRY_LEASH_ALL_FROG_VARIANTS) , "husbandry/make_a_sign_glow" => Some (& Self :: HUSBANDRY_MAKE_A_SIGN_GLOW) , "husbandry/obtain_netherite_hoe" => Some (& Self :: HUSBANDRY_OBTAIN_NETHERITE_HOE) , "husbandry/obtain_sniffer_egg" => Some (& Self :: HUSBANDRY_OBTAIN_SNIFFER_EGG) , "husbandry/place_dried_ghast_in_water" => Some (& Self :: HUSBANDRY_PLACE_DRIED_GHAST_IN_WATER) , "husbandry/plant_any_sniffer_seed" => Some (& Self :: HUSBANDRY_PLANT_ANY_SNIFFER_SEED) , "husbandry/plant_seed" => Some (& Self :: HUSBANDRY_PLANT_SEED) , "husbandry/remove_wolf_armor" => Some (& Self :: HUSBANDRY_REMOVE_WOLF_ARMOR) , "husbandry/repair_wolf_armor" => Some (& Self :: HUSBANDRY_REPAIR_WOLF_ARMOR) , "husbandry/ride_a_boat_with_a_goat" => Some (& Self :: HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT) , "husbandry/root" => Some (& Self :: HUSBANDRY_ROOT) , "husbandry/safely_harvest_honey" => Some (& Self :: HUSBANDRY_SAFELY_HARVEST_HONEY) , "husbandry/silk_touch_nest" => Some (& Self :: HUSBANDRY_SILK_TOUCH_NEST) , "husbandry/tactical_fishing" => Some (& Self :: HUSBANDRY_TACTICAL_FISHING) , "husbandry/tadpole_in_a_bucket" => Some (& Self :: HUSBANDRY_TADPOLE_IN_A_BUCKET) , "husbandry/tame_an_animal" => Some (& Self :: HUSBANDRY_TAME_AN_ANIMAL) , "husbandry/wax_off" => Some (& Self :: HUSBANDRY_WAX_OFF) , "husbandry/wax_on" => Some (& Self :: HUSBANDRY_WAX_ON) , "husbandry/whole_pack" => Some (& Self :: HUSBANDRY_WHOLE_PACK) , "nether/all_effects" => Some (& Self :: NETHER_ALL_EFFECTS) , "nether/all_potions" => Some (& Self :: NETHER_ALL_POTIONS) , "nether/brew_potion" => Some (& Self :: NETHER_BREW_POTION) , "nether/charge_respawn_anchor" => Some (& Self :: NETHER_CHARGE_RESPAWN_ANCHOR) , "nether/create_beacon" => Some (& Self :: NETHER_CREATE_BEACON) , "nether/create_full_beacon" => Some (& Self :: NETHER_CREATE_FULL_BEACON) , "nether/distract_piglin" => Some (& Self :: NETHER_DISTRACT_PIGLIN) , "nether/explore_nether" => Some (& Self :: NETHER_EXPLORE_NETHER) , "nether/fast_travel" => Some (& Self :: NETHER_FAST_TRAVEL) , "nether/find_bastion" => Some (& Self :: NETHER_FIND_BASTION) , "nether/find_fortress" => Some (& Self :: NETHER_FIND_FORTRESS) , "nether/get_wither_skull" => Some (& Self :: NETHER_GET_WITHER_SKULL) , "nether/loot_bastion" => Some (& Self :: NETHER_LOOT_BASTION) , "nether/netherite_armor" => Some (& Self :: NETHER_NETHERITE_ARMOR) , "nether/obtain_ancient_debris" => Some (& Self :: NETHER_OBTAIN_ANCIENT_DEBRIS) , "nether/obtain_blaze_rod" => Some (& Self :: NETHER_OBTAIN_BLAZE_ROD) , "nether/obtain_crying_obsidian" => Some (& Self :: NETHER_OBTAIN_CRYING_OBSIDIAN) , "nether/return_to_sender" => Some (& Self :: NETHER_RETURN_TO_SENDER) , "nether/ride_strider" => Some (& Self :: NETHER_RIDE_STRIDER) , "nether/ride_strider_in_overworld_lava" => Some (& Self :: NETHER_RIDE_STRIDER_IN_OVERWORLD_LAVA) , "nether/root" => Some (& Self :: NETHER_ROOT) , "nether/summon_wither" => Some (& Self :: NETHER_SUMMON_WITHER) , "nether/uneasy_alliance" => Some (& Self :: NETHER_UNEASY_ALLIANCE) , "recipes/brewing/blaze_powder" => Some (& Self :: RECIPES_BREWING_BLAZE_POWDER) , "recipes/brewing/brewing_stand" => Some (& Self :: RECIPES_BREWING_BREWING_STAND) , "recipes/brewing/cauldron" => Some (& Self :: RECIPES_BREWING_CAULDRON) , "recipes/brewing/fermented_spider_eye" => Some (& Self :: RECIPES_BREWING_FERMENTED_SPIDER_EYE) , "recipes/brewing/glass_bottle" => Some (& Self :: RECIPES_BREWING_GLASS_BOTTLE) , "recipes/brewing/glistering_melon_slice" => Some (& Self :: RECIPES_BREWING_GLISTERING_MELON_SLICE) , "recipes/brewing/golden_carrot" => Some (& Self :: RECIPES_BREWING_GOLDEN_CARROT) , "recipes/brewing/magma_cream" => Some (& Self :: RECIPES_BREWING_MAGMA_CREAM) , "recipes/building_blocks/acacia_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_PLANKS) , "recipes/building_blocks/acacia_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_SLAB) , "recipes/building_blocks/acacia_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_STAIRS) , "recipes/building_blocks/acacia_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_WOOD) , "recipes/building_blocks/amethyst_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_AMETHYST_BLOCK) , "recipes/building_blocks/andesite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE) , "recipes/building_blocks/andesite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB) , "recipes/building_blocks/andesite_slab_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/andesite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS) , "recipes/building_blocks/andesite_stairs_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/bamboo_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_BLOCK) , "recipes/building_blocks/bamboo_mosaic_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_SLAB) , "recipes/building_blocks/bamboo_mosaic_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_STAIRS) , "recipes/building_blocks/bamboo_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_PLANKS) , "recipes/building_blocks/bamboo_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_SLAB) , "recipes/building_blocks/bamboo_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_STAIRS) , "recipes/building_blocks/birch_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_PLANKS) , "recipes/building_blocks/birch_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_SLAB) , "recipes/building_blocks/birch_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_STAIRS) , "recipes/building_blocks/birch_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_WOOD) , "recipes/building_blocks/black_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACK_CONCRETE_POWDER) , "recipes/building_blocks/black_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACK_STAINED_GLASS) , "recipes/building_blocks/black_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACK_TERRACOTTA) , "recipes/building_blocks/blackstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB) , "recipes/building_blocks/blackstone_slab_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/blackstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS) , "recipes/building_blocks/blackstone_stairs_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/blue_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_CONCRETE_POWDER) , "recipes/building_blocks/blue_ice" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_ICE) , "recipes/building_blocks/blue_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_STAINED_GLASS) , "recipes/building_blocks/blue_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_TERRACOTTA) , "recipes/building_blocks/bone_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BONE_BLOCK) , "recipes/building_blocks/bookshelf" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BOOKSHELF) , "recipes/building_blocks/brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_SLAB) , "recipes/building_blocks/brick_slab_from_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_SLAB_FROM_BRICKS_STONECUTTING) , "recipes/building_blocks/brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_STAIRS) , "recipes/building_blocks/brick_stairs_from_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_STAIRS_FROM_BRICKS_STONECUTTING) , "recipes/building_blocks/bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICKS) , "recipes/building_blocks/brown_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BROWN_CONCRETE_POWDER) , "recipes/building_blocks/brown_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BROWN_STAINED_GLASS) , "recipes/building_blocks/brown_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BROWN_TERRACOTTA) , "recipes/building_blocks/cherry_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_PLANKS) , "recipes/building_blocks/cherry_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_SLAB) , "recipes/building_blocks/cherry_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_STAIRS) , "recipes/building_blocks/cherry_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_WOOD) , "recipes/building_blocks/chiseled_bookshelf" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_BOOKSHELF) , "recipes/building_blocks/chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER) , "recipes/building_blocks/chiseled_copper_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/chiseled_copper_from_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/chiseled_deepslate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE) , "recipes/building_blocks/chiseled_deepslate_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/chiseled_nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS) , "recipes/building_blocks/chiseled_nether_bricks_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_polished_blackstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE) , "recipes/building_blocks/chiseled_polished_blackstone_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_polished_blackstone_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_quartz_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK) , "recipes/building_blocks/chiseled_quartz_block_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/chiseled_red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE) , "recipes/building_blocks/chiseled_red_sandstone_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_resin_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS) , "recipes/building_blocks/chiseled_resin_bricks_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE) , "recipes/building_blocks/chiseled_sandstone_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS) , "recipes/building_blocks/chiseled_stone_bricks_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_FROM_STONE_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_stone_bricks_stone_from_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_STONE_FROM_STONECUTTING) , "recipes/building_blocks/chiseled_tuff" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF) , "recipes/building_blocks/chiseled_tuff_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS) , "recipes/building_blocks/chiseled_tuff_bricks_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/chiseled_tuff_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/clay" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CLAY) , "recipes/building_blocks/coal_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COAL_BLOCK) , "recipes/building_blocks/coarse_dirt" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COARSE_DIRT) , "recipes/building_blocks/cobbled_deepslate_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB) , "recipes/building_blocks/cobbled_deepslate_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/cobbled_deepslate_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS) , "recipes/building_blocks/cobbled_deepslate_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/cobblestone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB) , "recipes/building_blocks/cobblestone_slab_from_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB_FROM_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/cobblestone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS) , "recipes/building_blocks/cobblestone_stairs_from_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS_FROM_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/copper_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COPPER_BLOCK) , "recipes/building_blocks/copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COPPER_GRATE) , "recipes/building_blocks/copper_grate_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COPPER_GRATE_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cracked_deepslate_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_BRICKS) , "recipes/building_blocks/cracked_deepslate_tiles" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_TILES) , "recipes/building_blocks/cracked_nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_NETHER_BRICKS) , "recipes/building_blocks/cracked_polished_blackstone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_POLISHED_BLACKSTONE_BRICKS) , "recipes/building_blocks/cracked_stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_STONE_BRICKS) , "recipes/building_blocks/crimson_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_HYPHAE) , "recipes/building_blocks/crimson_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_PLANKS) , "recipes/building_blocks/crimson_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_SLAB) , "recipes/building_blocks/crimson_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_STAIRS) , "recipes/building_blocks/cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER) , "recipes/building_blocks/cut_copper_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB) , "recipes/building_blocks/cut_copper_slab_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cut_copper_slab_from_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS) , "recipes/building_blocks/cut_copper_stairs_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cut_copper_stairs_from_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/cut_red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE) , "recipes/building_blocks/cut_red_sandstone_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_red_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB) , "recipes/building_blocks/cut_red_sandstone_slab_from_cut_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_CUT_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_red_sandstone_slab_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE) , "recipes/building_blocks/cut_sandstone_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB) , "recipes/building_blocks/cut_sandstone_slab_from_cut_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_CUT_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_sandstone_slab_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cyan_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CYAN_CONCRETE_POWDER) , "recipes/building_blocks/cyan_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CYAN_STAINED_GLASS) , "recipes/building_blocks/cyan_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CYAN_TERRACOTTA) , "recipes/building_blocks/dark_oak_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_PLANKS) , "recipes/building_blocks/dark_oak_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_SLAB) , "recipes/building_blocks/dark_oak_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_STAIRS) , "recipes/building_blocks/dark_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_WOOD) , "recipes/building_blocks/dark_prismarine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE) , "recipes/building_blocks/dark_prismarine_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB) , "recipes/building_blocks/dark_prismarine_slab_from_dark_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB_FROM_DARK_PRISMARINE_STONECUTTING) , "recipes/building_blocks/dark_prismarine_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS) , "recipes/building_blocks/dark_prismarine_stairs_from_dark_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS_FROM_DARK_PRISMARINE_STONECUTTING) , "recipes/building_blocks/deepslate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE) , "recipes/building_blocks/deepslate_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB) , "recipes/building_blocks/deepslate_brick_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_brick_slab_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_brick_slab_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS) , "recipes/building_blocks/deepslate_brick_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_brick_stairs_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_brick_stairs_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS) , "recipes/building_blocks/deepslate_bricks_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_bricks_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB) , "recipes/building_blocks/deepslate_tile_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab_from_deepslate_tiles_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_TILES_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS) , "recipes/building_blocks/deepslate_tile_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_tiles_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_TILES_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tiles" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES) , "recipes/building_blocks/deepslate_tiles_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tiles_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_tiles_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/diamond_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIAMOND_BLOCK) , "recipes/building_blocks/diorite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE) , "recipes/building_blocks/diorite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_SLAB) , "recipes/building_blocks/diorite_slab_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_SLAB_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/diorite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS) , "recipes/building_blocks/diorite_stairs_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/dried_ghast" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DRIED_GHAST) , "recipes/building_blocks/dried_kelp_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DRIED_KELP_BLOCK) , "recipes/building_blocks/dripstone_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DRIPSTONE_BLOCK) , "recipes/building_blocks/dye_black_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_BLACK_WOOL) , "recipes/building_blocks/dye_blue_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_BLUE_WOOL) , "recipes/building_blocks/dye_brown_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_BROWN_WOOL) , "recipes/building_blocks/dye_cyan_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_CYAN_WOOL) , "recipes/building_blocks/dye_gray_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_GRAY_WOOL) , "recipes/building_blocks/dye_green_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_GREEN_WOOL) , "recipes/building_blocks/dye_light_blue_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_LIGHT_BLUE_WOOL) , "recipes/building_blocks/dye_light_gray_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_LIGHT_GRAY_WOOL) , "recipes/building_blocks/dye_lime_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_LIME_WOOL) , "recipes/building_blocks/dye_magenta_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_MAGENTA_WOOL) , "recipes/building_blocks/dye_orange_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_ORANGE_WOOL) , "recipes/building_blocks/dye_pink_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_PINK_WOOL) , "recipes/building_blocks/dye_purple_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_PURPLE_WOOL) , "recipes/building_blocks/dye_red_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_RED_WOOL) , "recipes/building_blocks/dye_white_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_WHITE_WOOL) , "recipes/building_blocks/dye_yellow_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_YELLOW_WOOL) , "recipes/building_blocks/emerald_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EMERALD_BLOCK) , "recipes/building_blocks/end_stone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB) , "recipes/building_blocks/end_stone_brick_slab_from_end_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/end_stone_brick_slab_from_end_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_STONECUTTING) , "recipes/building_blocks/end_stone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS) , "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_STONECUTTING) , "recipes/building_blocks/end_stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS) , "recipes/building_blocks/end_stone_bricks_from_end_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS_FROM_END_STONE_STONECUTTING) , "recipes/building_blocks/exposed_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER) , "recipes/building_blocks/exposed_chiseled_copper_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_chiseled_copper_from_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE) , "recipes/building_blocks/exposed_copper_grate_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER) , "recipes/building_blocks/exposed_cut_copper_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB) , "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS) , "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GLASS) , "recipes/building_blocks/glowstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GLOWSTONE) , "recipes/building_blocks/gold_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GOLD_BLOCK) , "recipes/building_blocks/granite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE) , "recipes/building_blocks/granite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_SLAB) , "recipes/building_blocks/granite_slab_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_SLAB_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/granite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS) , "recipes/building_blocks/granite_stairs_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/gray_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRAY_CONCRETE_POWDER) , "recipes/building_blocks/gray_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRAY_STAINED_GLASS) , "recipes/building_blocks/gray_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRAY_TERRACOTTA) , "recipes/building_blocks/green_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GREEN_CONCRETE_POWDER) , "recipes/building_blocks/green_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GREEN_STAINED_GLASS) , "recipes/building_blocks/green_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GREEN_TERRACOTTA) , "recipes/building_blocks/hay_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_HAY_BLOCK) , "recipes/building_blocks/iron_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_IRON_BLOCK) , "recipes/building_blocks/jack_o_lantern" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JACK_O_LANTERN) , "recipes/building_blocks/jungle_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_PLANKS) , "recipes/building_blocks/jungle_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_SLAB) , "recipes/building_blocks/jungle_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_STAIRS) , "recipes/building_blocks/jungle_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_WOOD) , "recipes/building_blocks/lapis_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LAPIS_BLOCK) , "recipes/building_blocks/light_blue_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_CONCRETE_POWDER) , "recipes/building_blocks/light_blue_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_STAINED_GLASS) , "recipes/building_blocks/light_blue_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_TERRACOTTA) , "recipes/building_blocks/light_gray_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_CONCRETE_POWDER) , "recipes/building_blocks/light_gray_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_STAINED_GLASS) , "recipes/building_blocks/light_gray_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_TERRACOTTA) , "recipes/building_blocks/lime_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIME_CONCRETE_POWDER) , "recipes/building_blocks/lime_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIME_STAINED_GLASS) , "recipes/building_blocks/lime_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIME_TERRACOTTA) , "recipes/building_blocks/magenta_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGENTA_CONCRETE_POWDER) , "recipes/building_blocks/magenta_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGENTA_STAINED_GLASS) , "recipes/building_blocks/magenta_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGENTA_TERRACOTTA) , "recipes/building_blocks/magma_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGMA_BLOCK) , "recipes/building_blocks/mangrove_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_PLANKS) , "recipes/building_blocks/mangrove_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_SLAB) , "recipes/building_blocks/mangrove_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_STAIRS) , "recipes/building_blocks/mangrove_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_WOOD) , "recipes/building_blocks/melon" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MELON) , "recipes/building_blocks/mossy_cobblestone_from_moss_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_MOSS_BLOCK) , "recipes/building_blocks/mossy_cobblestone_from_vine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_VINE) , "recipes/building_blocks/mossy_cobblestone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB) , "recipes/building_blocks/mossy_cobblestone_slab_from_mossy_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB_FROM_MOSSY_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/mossy_cobblestone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS) , "recipes/building_blocks/mossy_cobblestone_stairs_from_mossy_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS_FROM_MOSSY_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/mossy_stone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB) , "recipes/building_blocks/mossy_stone_brick_slab_from_mossy_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB_FROM_MOSSY_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/mossy_stone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS) , "recipes/building_blocks/mossy_stone_brick_stairs_from_mossy_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS_FROM_MOSSY_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/mossy_stone_bricks_from_moss_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_MOSS_BLOCK) , "recipes/building_blocks/mossy_stone_bricks_from_vine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_VINE) , "recipes/building_blocks/mud_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB) , "recipes/building_blocks/mud_brick_slab_from_mud_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB_FROM_MUD_BRICKS_STONECUTTING) , "recipes/building_blocks/mud_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS) , "recipes/building_blocks/mud_brick_stairs_from_mud_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS_FROM_MUD_BRICKS_STONECUTTING) , "recipes/building_blocks/mud_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICKS) , "recipes/building_blocks/muddy_mangrove_roots" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUDDY_MANGROVE_ROOTS) , "recipes/building_blocks/nether_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB) , "recipes/building_blocks/nether_brick_slab_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/nether_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS) , "recipes/building_blocks/nether_brick_stairs_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICKS) , "recipes/building_blocks/nether_wart_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_WART_BLOCK) , "recipes/building_blocks/netherite_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHERITE_BLOCK) , "recipes/building_blocks/oak_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_PLANKS) , "recipes/building_blocks/oak_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_SLAB) , "recipes/building_blocks/oak_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_STAIRS) , "recipes/building_blocks/oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_WOOD) , "recipes/building_blocks/orange_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ORANGE_CONCRETE_POWDER) , "recipes/building_blocks/orange_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ORANGE_STAINED_GLASS) , "recipes/building_blocks/orange_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ORANGE_TERRACOTTA) , "recipes/building_blocks/oxidized_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER) , "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE) , "recipes/building_blocks/oxidized_copper_grate_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER) , "recipes/building_blocks/oxidized_cut_copper_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB) , "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS) , "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/packed_ice" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PACKED_ICE) , "recipes/building_blocks/packed_mud" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PACKED_MUD) , "recipes/building_blocks/pale_oak_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_PLANKS) , "recipes/building_blocks/pale_oak_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_SLAB) , "recipes/building_blocks/pale_oak_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_STAIRS) , "recipes/building_blocks/pale_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_WOOD) , "recipes/building_blocks/pink_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PINK_CONCRETE_POWDER) , "recipes/building_blocks/pink_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PINK_STAINED_GLASS) , "recipes/building_blocks/pink_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PINK_TERRACOTTA) , "recipes/building_blocks/polished_andesite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE) , "recipes/building_blocks/polished_andesite_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB) , "recipes/building_blocks/polished_andesite_slab_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_slab_from_polished_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_POLISHED_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS) , "recipes/building_blocks/polished_andesite_stairs_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_stairs_from_polished_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_POLISHED_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_basalt" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BASALT) , "recipes/building_blocks/polished_basalt_from_basalt_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BASALT_FROM_BASALT_STONECUTTING) , "recipes/building_blocks/polished_blackstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE) , "recipes/building_blocks/polished_blackstone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB) , "recipes/building_blocks/polished_blackstone_brick_slab_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS) , "recipes/building_blocks/polished_blackstone_brick_stairs_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS) , "recipes/building_blocks/polished_blackstone_bricks_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_bricks_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB) , "recipes/building_blocks/polished_blackstone_slab_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_slab_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS) , "recipes/building_blocks/polished_blackstone_stairs_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_stairs_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_deepslate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE) , "recipes/building_blocks/polished_deepslate_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB) , "recipes/building_blocks/polished_deepslate_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_slab_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS) , "recipes/building_blocks/polished_deepslate_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_stairs_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_diorite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE) , "recipes/building_blocks/polished_diorite_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB) , "recipes/building_blocks/polished_diorite_slab_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_slab_from_polished_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_POLISHED_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS) , "recipes/building_blocks/polished_diorite_stairs_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_stairs_from_polished_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_POLISHED_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_granite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE) , "recipes/building_blocks/polished_granite_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB) , "recipes/building_blocks/polished_granite_slab_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_slab_from_polished_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_POLISHED_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS) , "recipes/building_blocks/polished_granite_stairs_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_stairs_from_polished_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_POLISHED_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_tuff" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF) , "recipes/building_blocks/polished_tuff_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB) , "recipes/building_blocks/polished_tuff_slab_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_slab_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS) , "recipes/building_blocks/polished_tuff_stairs_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_stairs_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/prismarine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE) , "recipes/building_blocks/prismarine_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB) , "recipes/building_blocks/prismarine_brick_slab_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/prismarine_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS) , "recipes/building_blocks/prismarine_brick_stairs_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/prismarine_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICKS) , "recipes/building_blocks/prismarine_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB) , "recipes/building_blocks/prismarine_slab_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/prismarine_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS) , "recipes/building_blocks/prismarine_stairs_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/purple_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPLE_CONCRETE_POWDER) , "recipes/building_blocks/purple_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPLE_STAINED_GLASS) , "recipes/building_blocks/purple_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPLE_TERRACOTTA) , "recipes/building_blocks/purpur_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_BLOCK) , "recipes/building_blocks/purpur_pillar" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR) , "recipes/building_blocks/purpur_pillar_from_purpur_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR_FROM_PURPUR_BLOCK_STONECUTTING) , "recipes/building_blocks/purpur_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_SLAB) , "recipes/building_blocks/purpur_slab_from_purpur_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_SLAB_FROM_PURPUR_BLOCK_STONECUTTING) , "recipes/building_blocks/purpur_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS) , "recipes/building_blocks/purpur_stairs_from_purpur_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS_FROM_PURPUR_BLOCK_STONECUTTING) , "recipes/building_blocks/quartz_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_BLOCK) , "recipes/building_blocks/quartz_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS) , "recipes/building_blocks/quartz_bricks_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/quartz_pillar" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR) , "recipes/building_blocks/quartz_pillar_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/quartz_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB) , "recipes/building_blocks/quartz_slab_from_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB_FROM_STONECUTTING) , "recipes/building_blocks/quartz_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS) , "recipes/building_blocks/quartz_stairs_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/raw_copper_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RAW_COPPER_BLOCK) , "recipes/building_blocks/raw_gold_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RAW_GOLD_BLOCK) , "recipes/building_blocks/raw_iron_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RAW_IRON_BLOCK) , "recipes/building_blocks/red_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_CONCRETE_POWDER) , "recipes/building_blocks/red_nether_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB) , "recipes/building_blocks/red_nether_brick_slab_from_red_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB_FROM_RED_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/red_nether_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS) , "recipes/building_blocks/red_nether_brick_stairs_from_red_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS_FROM_RED_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/red_nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICKS) , "recipes/building_blocks/red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE) , "recipes/building_blocks/red_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB) , "recipes/building_blocks/red_sandstone_slab_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/red_sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS) , "recipes/building_blocks/red_sandstone_stairs_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/red_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_STAINED_GLASS) , "recipes/building_blocks/red_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_TERRACOTTA) , "recipes/building_blocks/resin_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BLOCK) , "recipes/building_blocks/resin_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB) , "recipes/building_blocks/resin_brick_slab_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/building_blocks/resin_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS) , "recipes/building_blocks/resin_brick_stairs_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/building_blocks/resin_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICKS) , "recipes/building_blocks/sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE) , "recipes/building_blocks/sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB) , "recipes/building_blocks/sandstone_slab_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS) , "recipes/building_blocks/sandstone_stairs_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/sea_lantern" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SEA_LANTERN) , "recipes/building_blocks/smooth_basalt" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_BASALT) , "recipes/building_blocks/smooth_quartz" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ) , "recipes/building_blocks/smooth_quartz_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB) , "recipes/building_blocks/smooth_quartz_slab_from_smooth_quartz_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB_FROM_SMOOTH_QUARTZ_STONECUTTING) , "recipes/building_blocks/smooth_quartz_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS) , "recipes/building_blocks/smooth_quartz_stairs_from_smooth_quartz_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS_FROM_SMOOTH_QUARTZ_STONECUTTING) , "recipes/building_blocks/smooth_red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE) , "recipes/building_blocks/smooth_red_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB) , "recipes/building_blocks/smooth_red_sandstone_slab_from_smooth_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_red_sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS) , "recipes/building_blocks/smooth_red_sandstone_stairs_from_smooth_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE) , "recipes/building_blocks/smooth_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB) , "recipes/building_blocks/smooth_sandstone_slab_from_smooth_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB_FROM_SMOOTH_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS) , "recipes/building_blocks/smooth_sandstone_stairs_from_smooth_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS_FROM_SMOOTH_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_stone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE) , "recipes/building_blocks/smooth_stone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB) , "recipes/building_blocks/smooth_stone_slab_from_smooth_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB_FROM_SMOOTH_STONE_STONECUTTING) , "recipes/building_blocks/snow_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SNOW_BLOCK) , "recipes/building_blocks/sponge" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPONGE) , "recipes/building_blocks/spruce_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_PLANKS) , "recipes/building_blocks/spruce_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_SLAB) , "recipes/building_blocks/spruce_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_STAIRS) , "recipes/building_blocks/spruce_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_WOOD) , "recipes/building_blocks/stone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE) , "recipes/building_blocks/stone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB) , "recipes/building_blocks/stone_brick_slab_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_BRICKS_STONECUTTING) , "recipes/building_blocks/stone_brick_slab_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS) , "recipes/building_blocks/stone_brick_stairs_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_BRICKS_STONECUTTING) , "recipes/building_blocks/stone_brick_stairs_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICKS) , "recipes/building_blocks/stone_bricks_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICKS_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_SLAB) , "recipes/building_blocks/stone_slab_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_SLAB_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_STAIRS) , "recipes/building_blocks/stone_stairs_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_STAIRS_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stripped_acacia_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_ACACIA_WOOD) , "recipes/building_blocks/stripped_birch_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_BIRCH_WOOD) , "recipes/building_blocks/stripped_cherry_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_CHERRY_WOOD) , "recipes/building_blocks/stripped_crimson_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_CRIMSON_HYPHAE) , "recipes/building_blocks/stripped_dark_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_DARK_OAK_WOOD) , "recipes/building_blocks/stripped_jungle_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_JUNGLE_WOOD) , "recipes/building_blocks/stripped_mangrove_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_MANGROVE_WOOD) , "recipes/building_blocks/stripped_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_OAK_WOOD) , "recipes/building_blocks/stripped_pale_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_PALE_OAK_WOOD) , "recipes/building_blocks/stripped_spruce_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_SPRUCE_WOOD) , "recipes/building_blocks/stripped_warped_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_WARPED_HYPHAE) , "recipes/building_blocks/terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TERRACOTTA) , "recipes/building_blocks/tinted_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TINTED_GLASS) , "recipes/building_blocks/tuff_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB) , "recipes/building_blocks/tuff_brick_slab_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_brick_slab_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/building_blocks/tuff_brick_slab_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS) , "recipes/building_blocks/tuff_brick_stairs_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_brick_stairs_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/building_blocks/tuff_brick_stairs_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS) , "recipes/building_blocks/tuff_bricks_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_bricks_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_SLAB) , "recipes/building_blocks/tuff_slab_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_SLAB_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_STAIRS) , "recipes/building_blocks/tuff_stairs_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_STAIRS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/warped_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_HYPHAE) , "recipes/building_blocks/warped_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_PLANKS) , "recipes/building_blocks/warped_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_SLAB) , "recipes/building_blocks/warped_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_STAIRS) , "recipes/building_blocks/waxed_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER) , "recipes/building_blocks/waxed_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_chiseled_copper_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_chiseled_copper_from_waxed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_block_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BLOCK_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE) , "recipes/building_blocks/waxed_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_grate_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER) , "recipes/building_blocks/waxed_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER) , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE) , "recipes/building_blocks/waxed_exposed_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_grate_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER) , "recipes/building_blocks/waxed_exposed_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER) , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE) , "recipes/building_blocks/waxed_oxidized_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_grate_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER) , "recipes/building_blocks/waxed_oxidized_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER) , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE) , "recipes/building_blocks/waxed_weathered_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_grate_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER) , "recipes/building_blocks/waxed_weathered_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/weathered_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER) , "recipes/building_blocks/weathered_chiseled_copper_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_chiseled_copper_from_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE) , "recipes/building_blocks/weathered_copper_grate_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER) , "recipes/building_blocks/weathered_cut_copper_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB) , "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS) , "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/white_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_CONCRETE_POWDER) , "recipes/building_blocks/white_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_STAINED_GLASS) , "recipes/building_blocks/white_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_TERRACOTTA) , "recipes/building_blocks/white_wool_from_string" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_WOOL_FROM_STRING) , "recipes/building_blocks/yellow_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_YELLOW_CONCRETE_POWDER) , "recipes/building_blocks/yellow_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_YELLOW_STAINED_GLASS) , "recipes/building_blocks/yellow_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_YELLOW_TERRACOTTA) , "recipes/combat/arrow" => Some (& Self :: RECIPES_COMBAT_ARROW) , "recipes/combat/black_harness" => Some (& Self :: RECIPES_COMBAT_BLACK_HARNESS) , "recipes/combat/blue_harness" => Some (& Self :: RECIPES_COMBAT_BLUE_HARNESS) , "recipes/combat/bow" => Some (& Self :: RECIPES_COMBAT_BOW) , "recipes/combat/brown_harness" => Some (& Self :: RECIPES_COMBAT_BROWN_HARNESS) , "recipes/combat/copper_boots" => Some (& Self :: RECIPES_COMBAT_COPPER_BOOTS) , "recipes/combat/copper_chestplate" => Some (& Self :: RECIPES_COMBAT_COPPER_CHESTPLATE) , "recipes/combat/copper_helmet" => Some (& Self :: RECIPES_COMBAT_COPPER_HELMET) , "recipes/combat/copper_leggings" => Some (& Self :: RECIPES_COMBAT_COPPER_LEGGINGS) , "recipes/combat/copper_spear" => Some (& Self :: RECIPES_COMBAT_COPPER_SPEAR) , "recipes/combat/copper_sword" => Some (& Self :: RECIPES_COMBAT_COPPER_SWORD) , "recipes/combat/crossbow" => Some (& Self :: RECIPES_COMBAT_CROSSBOW) , "recipes/combat/cyan_harness" => Some (& Self :: RECIPES_COMBAT_CYAN_HARNESS) , "recipes/combat/diamond_boots" => Some (& Self :: RECIPES_COMBAT_DIAMOND_BOOTS) , "recipes/combat/diamond_chestplate" => Some (& Self :: RECIPES_COMBAT_DIAMOND_CHESTPLATE) , "recipes/combat/diamond_helmet" => Some (& Self :: RECIPES_COMBAT_DIAMOND_HELMET) , "recipes/combat/diamond_leggings" => Some (& Self :: RECIPES_COMBAT_DIAMOND_LEGGINGS) , "recipes/combat/diamond_spear" => Some (& Self :: RECIPES_COMBAT_DIAMOND_SPEAR) , "recipes/combat/diamond_sword" => Some (& Self :: RECIPES_COMBAT_DIAMOND_SWORD) , "recipes/combat/dye_black_harness" => Some (& Self :: RECIPES_COMBAT_DYE_BLACK_HARNESS) , "recipes/combat/dye_blue_harness" => Some (& Self :: RECIPES_COMBAT_DYE_BLUE_HARNESS) , "recipes/combat/dye_brown_harness" => Some (& Self :: RECIPES_COMBAT_DYE_BROWN_HARNESS) , "recipes/combat/dye_cyan_harness" => Some (& Self :: RECIPES_COMBAT_DYE_CYAN_HARNESS) , "recipes/combat/dye_gray_harness" => Some (& Self :: RECIPES_COMBAT_DYE_GRAY_HARNESS) , "recipes/combat/dye_green_harness" => Some (& Self :: RECIPES_COMBAT_DYE_GREEN_HARNESS) , "recipes/combat/dye_light_blue_harness" => Some (& Self :: RECIPES_COMBAT_DYE_LIGHT_BLUE_HARNESS) , "recipes/combat/dye_light_gray_harness" => Some (& Self :: RECIPES_COMBAT_DYE_LIGHT_GRAY_HARNESS) , "recipes/combat/dye_lime_harness" => Some (& Self :: RECIPES_COMBAT_DYE_LIME_HARNESS) , "recipes/combat/dye_magenta_harness" => Some (& Self :: RECIPES_COMBAT_DYE_MAGENTA_HARNESS) , "recipes/combat/dye_orange_harness" => Some (& Self :: RECIPES_COMBAT_DYE_ORANGE_HARNESS) , "recipes/combat/dye_pink_harness" => Some (& Self :: RECIPES_COMBAT_DYE_PINK_HARNESS) , "recipes/combat/dye_purple_harness" => Some (& Self :: RECIPES_COMBAT_DYE_PURPLE_HARNESS) , "recipes/combat/dye_red_harness" => Some (& Self :: RECIPES_COMBAT_DYE_RED_HARNESS) , "recipes/combat/dye_white_harness" => Some (& Self :: RECIPES_COMBAT_DYE_WHITE_HARNESS) , "recipes/combat/dye_yellow_harness" => Some (& Self :: RECIPES_COMBAT_DYE_YELLOW_HARNESS) , "recipes/combat/golden_boots" => Some (& Self :: RECIPES_COMBAT_GOLDEN_BOOTS) , "recipes/combat/golden_chestplate" => Some (& Self :: RECIPES_COMBAT_GOLDEN_CHESTPLATE) , "recipes/combat/golden_helmet" => Some (& Self :: RECIPES_COMBAT_GOLDEN_HELMET) , "recipes/combat/golden_leggings" => Some (& Self :: RECIPES_COMBAT_GOLDEN_LEGGINGS) , "recipes/combat/golden_spear" => Some (& Self :: RECIPES_COMBAT_GOLDEN_SPEAR) , "recipes/combat/golden_sword" => Some (& Self :: RECIPES_COMBAT_GOLDEN_SWORD) , "recipes/combat/gray_harness" => Some (& Self :: RECIPES_COMBAT_GRAY_HARNESS) , "recipes/combat/green_harness" => Some (& Self :: RECIPES_COMBAT_GREEN_HARNESS) , "recipes/combat/iron_boots" => Some (& Self :: RECIPES_COMBAT_IRON_BOOTS) , "recipes/combat/iron_chestplate" => Some (& Self :: RECIPES_COMBAT_IRON_CHESTPLATE) , "recipes/combat/iron_helmet" => Some (& Self :: RECIPES_COMBAT_IRON_HELMET) , "recipes/combat/iron_leggings" => Some (& Self :: RECIPES_COMBAT_IRON_LEGGINGS) , "recipes/combat/iron_spear" => Some (& Self :: RECIPES_COMBAT_IRON_SPEAR) , "recipes/combat/iron_sword" => Some (& Self :: RECIPES_COMBAT_IRON_SWORD) , "recipes/combat/leather_boots" => Some (& Self :: RECIPES_COMBAT_LEATHER_BOOTS) , "recipes/combat/leather_chestplate" => Some (& Self :: RECIPES_COMBAT_LEATHER_CHESTPLATE) , "recipes/combat/leather_helmet" => Some (& Self :: RECIPES_COMBAT_LEATHER_HELMET) , "recipes/combat/leather_leggings" => Some (& Self :: RECIPES_COMBAT_LEATHER_LEGGINGS) , "recipes/combat/light_blue_harness" => Some (& Self :: RECIPES_COMBAT_LIGHT_BLUE_HARNESS) , "recipes/combat/light_gray_harness" => Some (& Self :: RECIPES_COMBAT_LIGHT_GRAY_HARNESS) , "recipes/combat/lime_harness" => Some (& Self :: RECIPES_COMBAT_LIME_HARNESS) , "recipes/combat/mace" => Some (& Self :: RECIPES_COMBAT_MACE) , "recipes/combat/magenta_harness" => Some (& Self :: RECIPES_COMBAT_MAGENTA_HARNESS) , "recipes/combat/netherite_boots_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_BOOTS_SMITHING) , "recipes/combat/netherite_chestplate_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_CHESTPLATE_SMITHING) , "recipes/combat/netherite_helmet_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_HELMET_SMITHING) , "recipes/combat/netherite_horse_armor_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_HORSE_ARMOR_SMITHING) , "recipes/combat/netherite_leggings_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_LEGGINGS_SMITHING) , "recipes/combat/netherite_nautilus_armor_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_NAUTILUS_ARMOR_SMITHING) , "recipes/combat/netherite_spear_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_SPEAR_SMITHING) , "recipes/combat/netherite_sword_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_SWORD_SMITHING) , "recipes/combat/orange_harness" => Some (& Self :: RECIPES_COMBAT_ORANGE_HARNESS) , "recipes/combat/pink_harness" => Some (& Self :: RECIPES_COMBAT_PINK_HARNESS) , "recipes/combat/purple_harness" => Some (& Self :: RECIPES_COMBAT_PURPLE_HARNESS) , "recipes/combat/red_harness" => Some (& Self :: RECIPES_COMBAT_RED_HARNESS) , "recipes/combat/saddle" => Some (& Self :: RECIPES_COMBAT_SADDLE) , "recipes/combat/shield" => Some (& Self :: RECIPES_COMBAT_SHIELD) , "recipes/combat/spectral_arrow" => Some (& Self :: RECIPES_COMBAT_SPECTRAL_ARROW) , "recipes/combat/stone_spear" => Some (& Self :: RECIPES_COMBAT_STONE_SPEAR) , "recipes/combat/stone_sword" => Some (& Self :: RECIPES_COMBAT_STONE_SWORD) , "recipes/combat/turtle_helmet" => Some (& Self :: RECIPES_COMBAT_TURTLE_HELMET) , "recipes/combat/white_harness" => Some (& Self :: RECIPES_COMBAT_WHITE_HARNESS) , "recipes/combat/wolf_armor" => Some (& Self :: RECIPES_COMBAT_WOLF_ARMOR) , "recipes/combat/wooden_spear" => Some (& Self :: RECIPES_COMBAT_WOODEN_SPEAR) , "recipes/combat/wooden_sword" => Some (& Self :: RECIPES_COMBAT_WOODEN_SWORD) , "recipes/combat/yellow_harness" => Some (& Self :: RECIPES_COMBAT_YELLOW_HARNESS) , "recipes/decorations/acacia_fence" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_FENCE) , "recipes/decorations/acacia_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_HANGING_SIGN) , "recipes/decorations/acacia_shelf" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_SHELF) , "recipes/decorations/acacia_sign" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_SIGN) , "recipes/decorations/andesite_wall" => Some (& Self :: RECIPES_DECORATIONS_ANDESITE_WALL) , "recipes/decorations/andesite_wall_from_andesite_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_ANDESITE_WALL_FROM_ANDESITE_STONECUTTING) , "recipes/decorations/anvil" => Some (& Self :: RECIPES_DECORATIONS_ANVIL) , "recipes/decorations/armor_stand" => Some (& Self :: RECIPES_DECORATIONS_ARMOR_STAND) , "recipes/decorations/bamboo_fence" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_FENCE) , "recipes/decorations/bamboo_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_HANGING_SIGN) , "recipes/decorations/bamboo_mosaic" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_MOSAIC) , "recipes/decorations/bamboo_shelf" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_SHELF) , "recipes/decorations/bamboo_sign" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_SIGN) , "recipes/decorations/barrel" => Some (& Self :: RECIPES_DECORATIONS_BARREL) , "recipes/decorations/beehive" => Some (& Self :: RECIPES_DECORATIONS_BEEHIVE) , "recipes/decorations/birch_fence" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_FENCE) , "recipes/decorations/birch_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_HANGING_SIGN) , "recipes/decorations/birch_shelf" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_SHELF) , "recipes/decorations/birch_sign" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_SIGN) , "recipes/decorations/black_banner" => Some (& Self :: RECIPES_DECORATIONS_BLACK_BANNER) , "recipes/decorations/black_bed" => Some (& Self :: RECIPES_DECORATIONS_BLACK_BED) , "recipes/decorations/black_candle" => Some (& Self :: RECIPES_DECORATIONS_BLACK_CANDLE) , "recipes/decorations/black_carpet" => Some (& Self :: RECIPES_DECORATIONS_BLACK_CARPET) , "recipes/decorations/black_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_BLACK_GLAZED_TERRACOTTA) , "recipes/decorations/black_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_BLACK_SHULKER_BOX) , "recipes/decorations/black_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE) , "recipes/decorations/black_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/blackstone_wall" => Some (& Self :: RECIPES_DECORATIONS_BLACKSTONE_WALL) , "recipes/decorations/blackstone_wall_from_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING) , "recipes/decorations/blast_furnace" => Some (& Self :: RECIPES_DECORATIONS_BLAST_FURNACE) , "recipes/decorations/blue_banner" => Some (& Self :: RECIPES_DECORATIONS_BLUE_BANNER) , "recipes/decorations/blue_bed" => Some (& Self :: RECIPES_DECORATIONS_BLUE_BED) , "recipes/decorations/blue_candle" => Some (& Self :: RECIPES_DECORATIONS_BLUE_CANDLE) , "recipes/decorations/blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_BLUE_CARPET) , "recipes/decorations/blue_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_BLUE_GLAZED_TERRACOTTA) , "recipes/decorations/blue_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_BLUE_SHULKER_BOX) , "recipes/decorations/blue_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE) , "recipes/decorations/blue_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/brick_wall" => Some (& Self :: RECIPES_DECORATIONS_BRICK_WALL) , "recipes/decorations/brick_wall_from_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_BRICK_WALL_FROM_BRICKS_STONECUTTING) , "recipes/decorations/brown_banner" => Some (& Self :: RECIPES_DECORATIONS_BROWN_BANNER) , "recipes/decorations/brown_bed" => Some (& Self :: RECIPES_DECORATIONS_BROWN_BED) , "recipes/decorations/brown_candle" => Some (& Self :: RECIPES_DECORATIONS_BROWN_CANDLE) , "recipes/decorations/brown_carpet" => Some (& Self :: RECIPES_DECORATIONS_BROWN_CARPET) , "recipes/decorations/brown_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_BROWN_GLAZED_TERRACOTTA) , "recipes/decorations/brown_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_BROWN_SHULKER_BOX) , "recipes/decorations/brown_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE) , "recipes/decorations/brown_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/campfire" => Some (& Self :: RECIPES_DECORATIONS_CAMPFIRE) , "recipes/decorations/candle" => Some (& Self :: RECIPES_DECORATIONS_CANDLE) , "recipes/decorations/cartography_table" => Some (& Self :: RECIPES_DECORATIONS_CARTOGRAPHY_TABLE) , "recipes/decorations/cherry_fence" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_FENCE) , "recipes/decorations/cherry_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_HANGING_SIGN) , "recipes/decorations/cherry_shelf" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_SHELF) , "recipes/decorations/cherry_sign" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_SIGN) , "recipes/decorations/chest" => Some (& Self :: RECIPES_DECORATIONS_CHEST) , "recipes/decorations/cobbled_deepslate_wall" => Some (& Self :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL) , "recipes/decorations/cobbled_deepslate_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/cobblestone_wall" => Some (& Self :: RECIPES_DECORATIONS_COBBLESTONE_WALL) , "recipes/decorations/cobblestone_wall_from_cobblestone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_COBBLESTONE_WALL_FROM_COBBLESTONE_STONECUTTING) , "recipes/decorations/composter" => Some (& Self :: RECIPES_DECORATIONS_COMPOSTER) , "recipes/decorations/copper_bars" => Some (& Self :: RECIPES_DECORATIONS_COPPER_BARS) , "recipes/decorations/copper_chain" => Some (& Self :: RECIPES_DECORATIONS_COPPER_CHAIN) , "recipes/decorations/copper_chest" => Some (& Self :: RECIPES_DECORATIONS_COPPER_CHEST) , "recipes/decorations/copper_lantern" => Some (& Self :: RECIPES_DECORATIONS_COPPER_LANTERN) , "recipes/decorations/copper_torch" => Some (& Self :: RECIPES_DECORATIONS_COPPER_TORCH) , "recipes/decorations/crafting_table" => Some (& Self :: RECIPES_DECORATIONS_CRAFTING_TABLE) , "recipes/decorations/crimson_fence" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_FENCE) , "recipes/decorations/crimson_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_HANGING_SIGN) , "recipes/decorations/crimson_shelf" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_SHELF) , "recipes/decorations/crimson_sign" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_SIGN) , "recipes/decorations/cyan_banner" => Some (& Self :: RECIPES_DECORATIONS_CYAN_BANNER) , "recipes/decorations/cyan_bed" => Some (& Self :: RECIPES_DECORATIONS_CYAN_BED) , "recipes/decorations/cyan_candle" => Some (& Self :: RECIPES_DECORATIONS_CYAN_CANDLE) , "recipes/decorations/cyan_carpet" => Some (& Self :: RECIPES_DECORATIONS_CYAN_CARPET) , "recipes/decorations/cyan_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_CYAN_GLAZED_TERRACOTTA) , "recipes/decorations/cyan_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_CYAN_SHULKER_BOX) , "recipes/decorations/cyan_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE) , "recipes/decorations/cyan_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/dark_oak_fence" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_FENCE) , "recipes/decorations/dark_oak_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_HANGING_SIGN) , "recipes/decorations/dark_oak_shelf" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_SHELF) , "recipes/decorations/dark_oak_sign" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_SIGN) , "recipes/decorations/decorated_pot_simple" => Some (& Self :: RECIPES_DECORATIONS_DECORATED_POT_SIMPLE) , "recipes/decorations/deepslate_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL) , "recipes/decorations/deepslate_brick_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/deepslate_brick_wall_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/decorations/deepslate_brick_wall_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/decorations/deepslate_tile_wall" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL) , "recipes/decorations/deepslate_tile_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/deepslate_tile_wall_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/decorations/deepslate_tile_wall_from_deepslate_tiles_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_TILES_STONECUTTING) , "recipes/decorations/deepslate_tile_wall_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/decorations/diorite_wall" => Some (& Self :: RECIPES_DECORATIONS_DIORITE_WALL) , "recipes/decorations/diorite_wall_from_diorite_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DIORITE_WALL_FROM_DIORITE_STONECUTTING) , "recipes/decorations/dye_black_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLACK_BED) , "recipes/decorations/dye_black_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLACK_CARPET) , "recipes/decorations/dye_blue_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLUE_BED) , "recipes/decorations/dye_blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLUE_CARPET) , "recipes/decorations/dye_brown_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_BROWN_BED) , "recipes/decorations/dye_brown_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_BROWN_CARPET) , "recipes/decorations/dye_cyan_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_CYAN_BED) , "recipes/decorations/dye_cyan_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_CYAN_CARPET) , "recipes/decorations/dye_gray_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_GRAY_BED) , "recipes/decorations/dye_gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_GRAY_CARPET) , "recipes/decorations/dye_green_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_GREEN_BED) , "recipes/decorations/dye_green_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_GREEN_CARPET) , "recipes/decorations/dye_light_blue_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_BLUE_BED) , "recipes/decorations/dye_light_blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_BLUE_CARPET) , "recipes/decorations/dye_light_gray_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_GRAY_BED) , "recipes/decorations/dye_light_gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_GRAY_CARPET) , "recipes/decorations/dye_lime_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIME_BED) , "recipes/decorations/dye_lime_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIME_CARPET) , "recipes/decorations/dye_magenta_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_MAGENTA_BED) , "recipes/decorations/dye_magenta_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_MAGENTA_CARPET) , "recipes/decorations/dye_orange_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_ORANGE_BED) , "recipes/decorations/dye_orange_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_ORANGE_CARPET) , "recipes/decorations/dye_pink_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_PINK_BED) , "recipes/decorations/dye_pink_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_PINK_CARPET) , "recipes/decorations/dye_purple_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_PURPLE_BED) , "recipes/decorations/dye_purple_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_PURPLE_CARPET) , "recipes/decorations/dye_red_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_RED_BED) , "recipes/decorations/dye_red_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_RED_CARPET) , "recipes/decorations/dye_white_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_WHITE_BED) , "recipes/decorations/dye_white_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_WHITE_CARPET) , "recipes/decorations/dye_yellow_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_YELLOW_BED) , "recipes/decorations/dye_yellow_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_YELLOW_CARPET) , "recipes/decorations/enchanting_table" => Some (& Self :: RECIPES_DECORATIONS_ENCHANTING_TABLE) , "recipes/decorations/end_crystal" => Some (& Self :: RECIPES_DECORATIONS_END_CRYSTAL) , "recipes/decorations/end_rod" => Some (& Self :: RECIPES_DECORATIONS_END_ROD) , "recipes/decorations/end_stone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL) , "recipes/decorations/end_stone_brick_wall_from_end_stone_brick_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_BRICK_STONECUTTING) , "recipes/decorations/end_stone_brick_wall_from_end_stone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_STONECUTTING) , "recipes/decorations/ender_chest" => Some (& Self :: RECIPES_DECORATIONS_ENDER_CHEST) , "recipes/decorations/fletching_table" => Some (& Self :: RECIPES_DECORATIONS_FLETCHING_TABLE) , "recipes/decorations/flower_pot" => Some (& Self :: RECIPES_DECORATIONS_FLOWER_POT) , "recipes/decorations/furnace" => Some (& Self :: RECIPES_DECORATIONS_FURNACE) , "recipes/decorations/glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GLASS_PANE) , "recipes/decorations/glow_item_frame" => Some (& Self :: RECIPES_DECORATIONS_GLOW_ITEM_FRAME) , "recipes/decorations/granite_wall" => Some (& Self :: RECIPES_DECORATIONS_GRANITE_WALL) , "recipes/decorations/granite_wall_from_granite_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_GRANITE_WALL_FROM_GRANITE_STONECUTTING) , "recipes/decorations/gray_banner" => Some (& Self :: RECIPES_DECORATIONS_GRAY_BANNER) , "recipes/decorations/gray_bed" => Some (& Self :: RECIPES_DECORATIONS_GRAY_BED) , "recipes/decorations/gray_candle" => Some (& Self :: RECIPES_DECORATIONS_GRAY_CANDLE) , "recipes/decorations/gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_GRAY_CARPET) , "recipes/decorations/gray_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_GRAY_GLAZED_TERRACOTTA) , "recipes/decorations/gray_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_GRAY_SHULKER_BOX) , "recipes/decorations/gray_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE) , "recipes/decorations/gray_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/green_banner" => Some (& Self :: RECIPES_DECORATIONS_GREEN_BANNER) , "recipes/decorations/green_bed" => Some (& Self :: RECIPES_DECORATIONS_GREEN_BED) , "recipes/decorations/green_candle" => Some (& Self :: RECIPES_DECORATIONS_GREEN_CANDLE) , "recipes/decorations/green_carpet" => Some (& Self :: RECIPES_DECORATIONS_GREEN_CARPET) , "recipes/decorations/green_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_GREEN_GLAZED_TERRACOTTA) , "recipes/decorations/green_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_GREEN_SHULKER_BOX) , "recipes/decorations/green_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE) , "recipes/decorations/green_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/grindstone" => Some (& Self :: RECIPES_DECORATIONS_GRINDSTONE) , "recipes/decorations/honeycomb_block" => Some (& Self :: RECIPES_DECORATIONS_HONEYCOMB_BLOCK) , "recipes/decorations/iron_bars" => Some (& Self :: RECIPES_DECORATIONS_IRON_BARS) , "recipes/decorations/iron_chain" => Some (& Self :: RECIPES_DECORATIONS_IRON_CHAIN) , "recipes/decorations/item_frame" => Some (& Self :: RECIPES_DECORATIONS_ITEM_FRAME) , "recipes/decorations/jukebox" => Some (& Self :: RECIPES_DECORATIONS_JUKEBOX) , "recipes/decorations/jungle_fence" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_FENCE) , "recipes/decorations/jungle_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_HANGING_SIGN) , "recipes/decorations/jungle_shelf" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_SHELF) , "recipes/decorations/jungle_sign" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_SIGN) , "recipes/decorations/ladder" => Some (& Self :: RECIPES_DECORATIONS_LADDER) , "recipes/decorations/lantern" => Some (& Self :: RECIPES_DECORATIONS_LANTERN) , "recipes/decorations/light_blue_banner" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_BANNER) , "recipes/decorations/light_blue_bed" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_BED) , "recipes/decorations/light_blue_candle" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_CANDLE) , "recipes/decorations/light_blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_CARPET) , "recipes/decorations/light_blue_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_GLAZED_TERRACOTTA) , "recipes/decorations/light_blue_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_SHULKER_BOX) , "recipes/decorations/light_blue_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE) , "recipes/decorations/light_blue_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/light_gray_banner" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_BANNER) , "recipes/decorations/light_gray_bed" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_BED) , "recipes/decorations/light_gray_candle" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_CANDLE) , "recipes/decorations/light_gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_CARPET) , "recipes/decorations/light_gray_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_GLAZED_TERRACOTTA) , "recipes/decorations/light_gray_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_SHULKER_BOX) , "recipes/decorations/light_gray_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE) , "recipes/decorations/light_gray_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/lime_banner" => Some (& Self :: RECIPES_DECORATIONS_LIME_BANNER) , "recipes/decorations/lime_bed" => Some (& Self :: RECIPES_DECORATIONS_LIME_BED) , "recipes/decorations/lime_candle" => Some (& Self :: RECIPES_DECORATIONS_LIME_CANDLE) , "recipes/decorations/lime_carpet" => Some (& Self :: RECIPES_DECORATIONS_LIME_CARPET) , "recipes/decorations/lime_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_LIME_GLAZED_TERRACOTTA) , "recipes/decorations/lime_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_LIME_SHULKER_BOX) , "recipes/decorations/lime_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE) , "recipes/decorations/lime_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/lodestone" => Some (& Self :: RECIPES_DECORATIONS_LODESTONE) , "recipes/decorations/loom" => Some (& Self :: RECIPES_DECORATIONS_LOOM) , "recipes/decorations/magenta_banner" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_BANNER) , "recipes/decorations/magenta_bed" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_BED) , "recipes/decorations/magenta_candle" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_CANDLE) , "recipes/decorations/magenta_carpet" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_CARPET) , "recipes/decorations/magenta_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_GLAZED_TERRACOTTA) , "recipes/decorations/magenta_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_SHULKER_BOX) , "recipes/decorations/magenta_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE) , "recipes/decorations/magenta_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/mangrove_fence" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_FENCE) , "recipes/decorations/mangrove_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_HANGING_SIGN) , "recipes/decorations/mangrove_shelf" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_SHELF) , "recipes/decorations/mangrove_sign" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_SIGN) , "recipes/decorations/moss_carpet" => Some (& Self :: RECIPES_DECORATIONS_MOSS_CARPET) , "recipes/decorations/mossy_cobblestone_wall" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL) , "recipes/decorations/mossy_cobblestone_wall_from_mossy_cobblestone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL_FROM_MOSSY_COBBLESTONE_STONECUTTING) , "recipes/decorations/mossy_stone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL) , "recipes/decorations/mossy_stone_brick_wall_from_mossy_stone_brick_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL_FROM_MOSSY_STONE_BRICK_STONECUTTING) , "recipes/decorations/mud_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_MUD_BRICK_WALL) , "recipes/decorations/mud_brick_wall_from_mud_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_MUD_BRICK_WALL_FROM_MUD_BRICKS_STONECUTTING) , "recipes/decorations/nether_brick_fence" => Some (& Self :: RECIPES_DECORATIONS_NETHER_BRICK_FENCE) , "recipes/decorations/nether_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_NETHER_BRICK_WALL) , "recipes/decorations/nether_brick_wall_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_NETHER_BRICK_WALL_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/decorations/oak_fence" => Some (& Self :: RECIPES_DECORATIONS_OAK_FENCE) , "recipes/decorations/oak_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_OAK_HANGING_SIGN) , "recipes/decorations/oak_shelf" => Some (& Self :: RECIPES_DECORATIONS_OAK_SHELF) , "recipes/decorations/oak_sign" => Some (& Self :: RECIPES_DECORATIONS_OAK_SIGN) , "recipes/decorations/orange_banner" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_BANNER) , "recipes/decorations/orange_bed" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_BED) , "recipes/decorations/orange_candle" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_CANDLE) , "recipes/decorations/orange_carpet" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_CARPET) , "recipes/decorations/orange_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_GLAZED_TERRACOTTA) , "recipes/decorations/orange_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_SHULKER_BOX) , "recipes/decorations/orange_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE) , "recipes/decorations/orange_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/painting" => Some (& Self :: RECIPES_DECORATIONS_PAINTING) , "recipes/decorations/pale_moss_carpet" => Some (& Self :: RECIPES_DECORATIONS_PALE_MOSS_CARPET) , "recipes/decorations/pale_oak_fence" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_FENCE) , "recipes/decorations/pale_oak_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_HANGING_SIGN) , "recipes/decorations/pale_oak_shelf" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_SHELF) , "recipes/decorations/pale_oak_sign" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_SIGN) , "recipes/decorations/pink_banner" => Some (& Self :: RECIPES_DECORATIONS_PINK_BANNER) , "recipes/decorations/pink_bed" => Some (& Self :: RECIPES_DECORATIONS_PINK_BED) , "recipes/decorations/pink_candle" => Some (& Self :: RECIPES_DECORATIONS_PINK_CANDLE) , "recipes/decorations/pink_carpet" => Some (& Self :: RECIPES_DECORATIONS_PINK_CARPET) , "recipes/decorations/pink_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_PINK_GLAZED_TERRACOTTA) , "recipes/decorations/pink_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_PINK_SHULKER_BOX) , "recipes/decorations/pink_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE) , "recipes/decorations/pink_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/polished_blackstone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL) , "recipes/decorations/polished_blackstone_brick_wall_from_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING) , "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_blackstone_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL) , "recipes/decorations/polished_blackstone_wall_from_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_blackstone_wall_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_deepslate_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL) , "recipes/decorations/polished_deepslate_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/polished_deepslate_wall_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/decorations/polished_tuff_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL) , "recipes/decorations/polished_tuff_wall_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/decorations/polished_tuff_wall_from_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_TUFF_STONECUTTING) , "recipes/decorations/prismarine_wall" => Some (& Self :: RECIPES_DECORATIONS_PRISMARINE_WALL) , "recipes/decorations/prismarine_wall_from_prismarine_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_PRISMARINE_WALL_FROM_PRISMARINE_STONECUTTING) , "recipes/decorations/purple_banner" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_BANNER) , "recipes/decorations/purple_bed" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_BED) , "recipes/decorations/purple_candle" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_CANDLE) , "recipes/decorations/purple_carpet" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_CARPET) , "recipes/decorations/purple_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_GLAZED_TERRACOTTA) , "recipes/decorations/purple_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_SHULKER_BOX) , "recipes/decorations/purple_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE) , "recipes/decorations/purple_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/red_banner" => Some (& Self :: RECIPES_DECORATIONS_RED_BANNER) , "recipes/decorations/red_bed" => Some (& Self :: RECIPES_DECORATIONS_RED_BED) , "recipes/decorations/red_candle" => Some (& Self :: RECIPES_DECORATIONS_RED_CANDLE) , "recipes/decorations/red_carpet" => Some (& Self :: RECIPES_DECORATIONS_RED_CARPET) , "recipes/decorations/red_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_RED_GLAZED_TERRACOTTA) , "recipes/decorations/red_nether_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL) , "recipes/decorations/red_nether_brick_wall_from_red_nether_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL_FROM_RED_NETHER_BRICKS_STONECUTTING) , "recipes/decorations/red_sandstone_wall" => Some (& Self :: RECIPES_DECORATIONS_RED_SANDSTONE_WALL) , "recipes/decorations/red_sandstone_wall_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_RED_SANDSTONE_WALL_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/decorations/red_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_RED_SHULKER_BOX) , "recipes/decorations/red_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE) , "recipes/decorations/red_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/resin_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_RESIN_BRICK_WALL) , "recipes/decorations/resin_brick_wall_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_RESIN_BRICK_WALL_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/decorations/respawn_anchor" => Some (& Self :: RECIPES_DECORATIONS_RESPAWN_ANCHOR) , "recipes/decorations/sandstone_wall" => Some (& Self :: RECIPES_DECORATIONS_SANDSTONE_WALL) , "recipes/decorations/sandstone_wall_from_sandstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_SANDSTONE_WALL_FROM_SANDSTONE_STONECUTTING) , "recipes/decorations/scaffolding" => Some (& Self :: RECIPES_DECORATIONS_SCAFFOLDING) , "recipes/decorations/shulker_box" => Some (& Self :: RECIPES_DECORATIONS_SHULKER_BOX) , "recipes/decorations/smithing_table" => Some (& Self :: RECIPES_DECORATIONS_SMITHING_TABLE) , "recipes/decorations/smoker" => Some (& Self :: RECIPES_DECORATIONS_SMOKER) , "recipes/decorations/snow" => Some (& Self :: RECIPES_DECORATIONS_SNOW) , "recipes/decorations/soul_campfire" => Some (& Self :: RECIPES_DECORATIONS_SOUL_CAMPFIRE) , "recipes/decorations/soul_lantern" => Some (& Self :: RECIPES_DECORATIONS_SOUL_LANTERN) , "recipes/decorations/soul_torch" => Some (& Self :: RECIPES_DECORATIONS_SOUL_TORCH) , "recipes/decorations/spruce_fence" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_FENCE) , "recipes/decorations/spruce_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_HANGING_SIGN) , "recipes/decorations/spruce_shelf" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_SHELF) , "recipes/decorations/spruce_sign" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_SIGN) , "recipes/decorations/stone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_STONE_BRICK_WALL) , "recipes/decorations/stone_brick_wall_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_STONE_BRICK_WALL_FROM_STONE_BRICKS_STONECUTTING) , "recipes/decorations/stone_brick_walls_from_stone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_STONE_BRICK_WALLS_FROM_STONE_STONECUTTING) , "recipes/decorations/stonecutter" => Some (& Self :: RECIPES_DECORATIONS_STONECUTTER) , "recipes/decorations/torch" => Some (& Self :: RECIPES_DECORATIONS_TORCH) , "recipes/decorations/tuff_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL) , "recipes/decorations/tuff_brick_wall_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/decorations/tuff_brick_wall_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/decorations/tuff_brick_wall_from_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_STONECUTTING) , "recipes/decorations/tuff_wall" => Some (& Self :: RECIPES_DECORATIONS_TUFF_WALL) , "recipes/decorations/tuff_wall_from_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_WALL_FROM_TUFF_STONECUTTING) , "recipes/decorations/warped_fence" => Some (& Self :: RECIPES_DECORATIONS_WARPED_FENCE) , "recipes/decorations/warped_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_WARPED_HANGING_SIGN) , "recipes/decorations/warped_shelf" => Some (& Self :: RECIPES_DECORATIONS_WARPED_SHELF) , "recipes/decorations/warped_sign" => Some (& Self :: RECIPES_DECORATIONS_WARPED_SIGN) , "recipes/decorations/white_banner" => Some (& Self :: RECIPES_DECORATIONS_WHITE_BANNER) , "recipes/decorations/white_bed" => Some (& Self :: RECIPES_DECORATIONS_WHITE_BED) , "recipes/decorations/white_candle" => Some (& Self :: RECIPES_DECORATIONS_WHITE_CANDLE) , "recipes/decorations/white_carpet" => Some (& Self :: RECIPES_DECORATIONS_WHITE_CARPET) , "recipes/decorations/white_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_WHITE_GLAZED_TERRACOTTA) , "recipes/decorations/white_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_WHITE_SHULKER_BOX) , "recipes/decorations/white_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE) , "recipes/decorations/white_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/yellow_banner" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_BANNER) , "recipes/decorations/yellow_bed" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_BED) , "recipes/decorations/yellow_candle" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_CANDLE) , "recipes/decorations/yellow_carpet" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_CARPET) , "recipes/decorations/yellow_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_GLAZED_TERRACOTTA) , "recipes/decorations/yellow_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_SHULKER_BOX) , "recipes/decorations/yellow_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE) , "recipes/decorations/yellow_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/food/baked_potato" => Some (& Self :: RECIPES_FOOD_BAKED_POTATO) , "recipes/food/baked_potato_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_BAKED_POTATO_FROM_CAMPFIRE_COOKING) , "recipes/food/baked_potato_from_smoking" => Some (& Self :: RECIPES_FOOD_BAKED_POTATO_FROM_SMOKING) , "recipes/food/beetroot_soup" => Some (& Self :: RECIPES_FOOD_BEETROOT_SOUP) , "recipes/food/bread" => Some (& Self :: RECIPES_FOOD_BREAD) , "recipes/food/cake" => Some (& Self :: RECIPES_FOOD_CAKE) , "recipes/food/cooked_beef" => Some (& Self :: RECIPES_FOOD_COOKED_BEEF) , "recipes/food/cooked_beef_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_BEEF_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_beef_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_BEEF_FROM_SMOKING) , "recipes/food/cooked_chicken" => Some (& Self :: RECIPES_FOOD_COOKED_CHICKEN) , "recipes/food/cooked_chicken_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_CHICKEN_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_chicken_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_CHICKEN_FROM_SMOKING) , "recipes/food/cooked_cod" => Some (& Self :: RECIPES_FOOD_COOKED_COD) , "recipes/food/cooked_cod_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_COD_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_cod_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_COD_FROM_SMOKING) , "recipes/food/cooked_mutton" => Some (& Self :: RECIPES_FOOD_COOKED_MUTTON) , "recipes/food/cooked_mutton_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_MUTTON_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_mutton_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_MUTTON_FROM_SMOKING) , "recipes/food/cooked_porkchop" => Some (& Self :: RECIPES_FOOD_COOKED_PORKCHOP) , "recipes/food/cooked_porkchop_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_PORKCHOP_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_porkchop_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_PORKCHOP_FROM_SMOKING) , "recipes/food/cooked_rabbit" => Some (& Self :: RECIPES_FOOD_COOKED_RABBIT) , "recipes/food/cooked_rabbit_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_RABBIT_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_rabbit_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_RABBIT_FROM_SMOKING) , "recipes/food/cooked_salmon" => Some (& Self :: RECIPES_FOOD_COOKED_SALMON) , "recipes/food/cooked_salmon_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_SALMON_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_salmon_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_SALMON_FROM_SMOKING) , "recipes/food/cookie" => Some (& Self :: RECIPES_FOOD_COOKIE) , "recipes/food/dried_kelp" => Some (& Self :: RECIPES_FOOD_DRIED_KELP) , "recipes/food/dried_kelp_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_DRIED_KELP_FROM_CAMPFIRE_COOKING) , "recipes/food/dried_kelp_from_smelting" => Some (& Self :: RECIPES_FOOD_DRIED_KELP_FROM_SMELTING) , "recipes/food/dried_kelp_from_smoking" => Some (& Self :: RECIPES_FOOD_DRIED_KELP_FROM_SMOKING) , "recipes/food/golden_apple" => Some (& Self :: RECIPES_FOOD_GOLDEN_APPLE) , "recipes/food/honey_bottle" => Some (& Self :: RECIPES_FOOD_HONEY_BOTTLE) , "recipes/food/mushroom_stew" => Some (& Self :: RECIPES_FOOD_MUSHROOM_STEW) , "recipes/food/pumpkin_pie" => Some (& Self :: RECIPES_FOOD_PUMPKIN_PIE) , "recipes/food/rabbit_stew_from_brown_mushroom" => Some (& Self :: RECIPES_FOOD_RABBIT_STEW_FROM_BROWN_MUSHROOM) , "recipes/food/rabbit_stew_from_red_mushroom" => Some (& Self :: RECIPES_FOOD_RABBIT_STEW_FROM_RED_MUSHROOM) , "recipes/food/suspicious_stew_from_allium" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ALLIUM) , "recipes/food/suspicious_stew_from_azure_bluet" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_AZURE_BLUET) , "recipes/food/suspicious_stew_from_blue_orchid" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_BLUE_ORCHID) , "recipes/food/suspicious_stew_from_closed_eyeblossom" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CLOSED_EYEBLOSSOM) , "recipes/food/suspicious_stew_from_cornflower" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CORNFLOWER) , "recipes/food/suspicious_stew_from_dandelion" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_DANDELION) , "recipes/food/suspicious_stew_from_lily_of_the_valley" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_LILY_OF_THE_VALLEY) , "recipes/food/suspicious_stew_from_open_eyeblossom" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OPEN_EYEBLOSSOM) , "recipes/food/suspicious_stew_from_orange_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ORANGE_TULIP) , "recipes/food/suspicious_stew_from_oxeye_daisy" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OXEYE_DAISY) , "recipes/food/suspicious_stew_from_pink_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_PINK_TULIP) , "recipes/food/suspicious_stew_from_poppy" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_POPPY) , "recipes/food/suspicious_stew_from_red_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_RED_TULIP) , "recipes/food/suspicious_stew_from_torchflower" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_TORCHFLOWER) , "recipes/food/suspicious_stew_from_white_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WHITE_TULIP) , "recipes/food/suspicious_stew_from_wither_rose" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WITHER_ROSE) , "recipes/misc/beacon" => Some (& Self :: RECIPES_MISC_BEACON) , "recipes/misc/black_dye" => Some (& Self :: RECIPES_MISC_BLACK_DYE) , "recipes/misc/black_dye_from_wither_rose" => Some (& Self :: RECIPES_MISC_BLACK_DYE_FROM_WITHER_ROSE) , "recipes/misc/blue_dye" => Some (& Self :: RECIPES_MISC_BLUE_DYE) , "recipes/misc/blue_dye_from_cornflower" => Some (& Self :: RECIPES_MISC_BLUE_DYE_FROM_CORNFLOWER) , "recipes/misc/bolt_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/bolt_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/bone_meal" => Some (& Self :: RECIPES_MISC_BONE_MEAL) , "recipes/misc/bone_meal_from_bone_block" => Some (& Self :: RECIPES_MISC_BONE_MEAL_FROM_BONE_BLOCK) , "recipes/misc/book" => Some (& Self :: RECIPES_MISC_BOOK) , "recipes/misc/bordure_indented_banner_pattern" => Some (& Self :: RECIPES_MISC_BORDURE_INDENTED_BANNER_PATTERN) , "recipes/misc/bowl" => Some (& Self :: RECIPES_MISC_BOWL) , "recipes/misc/brick" => Some (& Self :: RECIPES_MISC_BRICK) , "recipes/misc/brown_dye" => Some (& Self :: RECIPES_MISC_BROWN_DYE) , "recipes/misc/bucket" => Some (& Self :: RECIPES_MISC_BUCKET) , "recipes/misc/charcoal" => Some (& Self :: RECIPES_MISC_CHARCOAL) , "recipes/misc/coal" => Some (& Self :: RECIPES_MISC_COAL) , "recipes/misc/coal_from_blasting_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_BLASTING_COAL_ORE) , "recipes/misc/coal_from_blasting_deepslate_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_BLASTING_DEEPSLATE_COAL_ORE) , "recipes/misc/coal_from_smelting_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_SMELTING_COAL_ORE) , "recipes/misc/coal_from_smelting_deepslate_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_SMELTING_DEEPSLATE_COAL_ORE) , "recipes/misc/coast_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/coast_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/conduit" => Some (& Self :: RECIPES_MISC_CONDUIT) , "recipes/misc/copper_ingot" => Some (& Self :: RECIPES_MISC_COPPER_INGOT) , "recipes/misc/copper_ingot_from_blasting_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_COPPER_ORE) , "recipes/misc/copper_ingot_from_blasting_deepslate_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_DEEPSLATE_COPPER_ORE) , "recipes/misc/copper_ingot_from_blasting_raw_copper" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_RAW_COPPER) , "recipes/misc/copper_ingot_from_nuggets" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_NUGGETS) , "recipes/misc/copper_ingot_from_smelting_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_COPPER_ORE) , "recipes/misc/copper_ingot_from_smelting_deepslate_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_DEEPSLATE_COPPER_ORE) , "recipes/misc/copper_ingot_from_smelting_raw_copper" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_RAW_COPPER) , "recipes/misc/copper_ingot_from_waxed_copper_block" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_WAXED_COPPER_BLOCK) , "recipes/misc/copper_nugget" => Some (& Self :: RECIPES_MISC_COPPER_NUGGET) , "recipes/misc/copper_nugget_from_blasting" => Some (& Self :: RECIPES_MISC_COPPER_NUGGET_FROM_BLASTING) , "recipes/misc/copper_nugget_from_smelting" => Some (& Self :: RECIPES_MISC_COPPER_NUGGET_FROM_SMELTING) , "recipes/misc/creaking_heart" => Some (& Self :: RECIPES_MISC_CREAKING_HEART) , "recipes/misc/creeper_banner_pattern" => Some (& Self :: RECIPES_MISC_CREEPER_BANNER_PATTERN) , "recipes/misc/cyan_dye" => Some (& Self :: RECIPES_MISC_CYAN_DYE) , "recipes/misc/cyan_dye_from_pitcher_plant" => Some (& Self :: RECIPES_MISC_CYAN_DYE_FROM_PITCHER_PLANT) , "recipes/misc/diamond" => Some (& Self :: RECIPES_MISC_DIAMOND) , "recipes/misc/diamond_from_blasting_deepslate_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_BLASTING_DEEPSLATE_DIAMOND_ORE) , "recipes/misc/diamond_from_blasting_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_BLASTING_DIAMOND_ORE) , "recipes/misc/diamond_from_smelting_deepslate_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_SMELTING_DEEPSLATE_DIAMOND_ORE) , "recipes/misc/diamond_from_smelting_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_SMELTING_DIAMOND_ORE) , "recipes/misc/dune_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/dune_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/emerald" => Some (& Self :: RECIPES_MISC_EMERALD) , "recipes/misc/emerald_from_blasting_deepslate_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_BLASTING_DEEPSLATE_EMERALD_ORE) , "recipes/misc/emerald_from_blasting_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_BLASTING_EMERALD_ORE) , "recipes/misc/emerald_from_smelting_deepslate_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_SMELTING_DEEPSLATE_EMERALD_ORE) , "recipes/misc/emerald_from_smelting_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_SMELTING_EMERALD_ORE) , "recipes/misc/ender_eye" => Some (& Self :: RECIPES_MISC_ENDER_EYE) , "recipes/misc/eye_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/eye_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/field_masoned_banner_pattern" => Some (& Self :: RECIPES_MISC_FIELD_MASONED_BANNER_PATTERN) , "recipes/misc/fire_charge" => Some (& Self :: RECIPES_MISC_FIRE_CHARGE) , "recipes/misc/firework_rocket_simple" => Some (& Self :: RECIPES_MISC_FIREWORK_ROCKET_SIMPLE) , "recipes/misc/flow_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/flow_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/flower_banner_pattern" => Some (& Self :: RECIPES_MISC_FLOWER_BANNER_PATTERN) , "recipes/misc/gold_ingot_from_blasting_deepslate_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_DEEPSLATE_GOLD_ORE) , "recipes/misc/gold_ingot_from_blasting_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_GOLD_ORE) , "recipes/misc/gold_ingot_from_blasting_nether_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_NETHER_GOLD_ORE) , "recipes/misc/gold_ingot_from_blasting_raw_gold" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_RAW_GOLD) , "recipes/misc/gold_ingot_from_gold_block" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_GOLD_BLOCK) , "recipes/misc/gold_ingot_from_nuggets" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_NUGGETS) , "recipes/misc/gold_ingot_from_smelting_deepslate_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_DEEPSLATE_GOLD_ORE) , "recipes/misc/gold_ingot_from_smelting_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_GOLD_ORE) , "recipes/misc/gold_ingot_from_smelting_nether_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_NETHER_GOLD_ORE) , "recipes/misc/gold_ingot_from_smelting_raw_gold" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_RAW_GOLD) , "recipes/misc/gold_nugget" => Some (& Self :: RECIPES_MISC_GOLD_NUGGET) , "recipes/misc/gold_nugget_from_blasting" => Some (& Self :: RECIPES_MISC_GOLD_NUGGET_FROM_BLASTING) , "recipes/misc/gold_nugget_from_smelting" => Some (& Self :: RECIPES_MISC_GOLD_NUGGET_FROM_SMELTING) , "recipes/misc/gray_dye" => Some (& Self :: RECIPES_MISC_GRAY_DYE) , "recipes/misc/gray_dye_from_closed_eyeblossom" => Some (& Self :: RECIPES_MISC_GRAY_DYE_FROM_CLOSED_EYEBLOSSOM) , "recipes/misc/green_dye" => Some (& Self :: RECIPES_MISC_GREEN_DYE) , "recipes/misc/host_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/host_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/iron_ingot_from_blasting_deepslate_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_DEEPSLATE_IRON_ORE) , "recipes/misc/iron_ingot_from_blasting_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_IRON_ORE) , "recipes/misc/iron_ingot_from_blasting_raw_iron" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_RAW_IRON) , "recipes/misc/iron_ingot_from_iron_block" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_IRON_BLOCK) , "recipes/misc/iron_ingot_from_nuggets" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_NUGGETS) , "recipes/misc/iron_ingot_from_smelting_deepslate_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_DEEPSLATE_IRON_ORE) , "recipes/misc/iron_ingot_from_smelting_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_IRON_ORE) , "recipes/misc/iron_ingot_from_smelting_raw_iron" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_RAW_IRON) , "recipes/misc/iron_nugget" => Some (& Self :: RECIPES_MISC_IRON_NUGGET) , "recipes/misc/iron_nugget_from_blasting" => Some (& Self :: RECIPES_MISC_IRON_NUGGET_FROM_BLASTING) , "recipes/misc/iron_nugget_from_smelting" => Some (& Self :: RECIPES_MISC_IRON_NUGGET_FROM_SMELTING) , "recipes/misc/lapis_lazuli" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI) , "recipes/misc/lapis_lazuli_from_blasting_deepslate_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_DEEPSLATE_LAPIS_ORE) , "recipes/misc/lapis_lazuli_from_blasting_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_LAPIS_ORE) , "recipes/misc/lapis_lazuli_from_smelting_deepslate_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_DEEPSLATE_LAPIS_ORE) , "recipes/misc/lapis_lazuli_from_smelting_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_LAPIS_ORE) , "recipes/misc/leaf_litter" => Some (& Self :: RECIPES_MISC_LEAF_LITTER) , "recipes/misc/leather" => Some (& Self :: RECIPES_MISC_LEATHER) , "recipes/misc/leather_horse_armor" => Some (& Self :: RECIPES_MISC_LEATHER_HORSE_ARMOR) , "recipes/misc/light_blue_dye_from_blue_orchid" => Some (& Self :: RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_ORCHID) , "recipes/misc/light_blue_dye_from_blue_white_dye" => Some (& Self :: RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_WHITE_DYE) , "recipes/misc/light_gray_dye_from_azure_bluet" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_AZURE_BLUET) , "recipes/misc/light_gray_dye_from_black_white_dye" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_BLACK_WHITE_DYE) , "recipes/misc/light_gray_dye_from_gray_white_dye" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_GRAY_WHITE_DYE) , "recipes/misc/light_gray_dye_from_oxeye_daisy" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_OXEYE_DAISY) , "recipes/misc/light_gray_dye_from_white_tulip" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_WHITE_TULIP) , "recipes/misc/lime_dye" => Some (& Self :: RECIPES_MISC_LIME_DYE) , "recipes/misc/lime_dye_from_smelting" => Some (& Self :: RECIPES_MISC_LIME_DYE_FROM_SMELTING) , "recipes/misc/magenta_dye_from_allium" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_ALLIUM) , "recipes/misc/magenta_dye_from_blue_red_pink" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_PINK) , "recipes/misc/magenta_dye_from_blue_red_white_dye" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_WHITE_DYE) , "recipes/misc/magenta_dye_from_lilac" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_LILAC) , "recipes/misc/magenta_dye_from_purple_and_pink" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_PURPLE_AND_PINK) , "recipes/misc/map" => Some (& Self :: RECIPES_MISC_MAP) , "recipes/misc/melon_seeds" => Some (& Self :: RECIPES_MISC_MELON_SEEDS) , "recipes/misc/mojang_banner_pattern" => Some (& Self :: RECIPES_MISC_MOJANG_BANNER_PATTERN) , "recipes/misc/music_disc_5" => Some (& Self :: RECIPES_MISC_MUSIC_DISC_5) , "recipes/misc/nether_brick" => Some (& Self :: RECIPES_MISC_NETHER_BRICK) , "recipes/misc/netherite_ingot" => Some (& Self :: RECIPES_MISC_NETHERITE_INGOT) , "recipes/misc/netherite_ingot_from_netherite_block" => Some (& Self :: RECIPES_MISC_NETHERITE_INGOT_FROM_NETHERITE_BLOCK) , "recipes/misc/netherite_scrap" => Some (& Self :: RECIPES_MISC_NETHERITE_SCRAP) , "recipes/misc/netherite_scrap_from_blasting" => Some (& Self :: RECIPES_MISC_NETHERITE_SCRAP_FROM_BLASTING) , "recipes/misc/netherite_upgrade_smithing_template" => Some (& Self :: RECIPES_MISC_NETHERITE_UPGRADE_SMITHING_TEMPLATE) , "recipes/misc/orange_dye_from_open_eyeblossom" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_OPEN_EYEBLOSSOM) , "recipes/misc/orange_dye_from_orange_tulip" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_ORANGE_TULIP) , "recipes/misc/orange_dye_from_red_yellow" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_RED_YELLOW) , "recipes/misc/orange_dye_from_torchflower" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_TORCHFLOWER) , "recipes/misc/paper" => Some (& Self :: RECIPES_MISC_PAPER) , "recipes/misc/pink_dye_from_cactus_flower" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_CACTUS_FLOWER) , "recipes/misc/pink_dye_from_peony" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_PEONY) , "recipes/misc/pink_dye_from_pink_petals" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_PINK_PETALS) , "recipes/misc/pink_dye_from_pink_tulip" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_PINK_TULIP) , "recipes/misc/pink_dye_from_red_white_dye" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_RED_WHITE_DYE) , "recipes/misc/popped_chorus_fruit" => Some (& Self :: RECIPES_MISC_POPPED_CHORUS_FRUIT) , "recipes/misc/pumpkin_seeds" => Some (& Self :: RECIPES_MISC_PUMPKIN_SEEDS) , "recipes/misc/purple_dye" => Some (& Self :: RECIPES_MISC_PURPLE_DYE) , "recipes/misc/quartz" => Some (& Self :: RECIPES_MISC_QUARTZ) , "recipes/misc/quartz_from_blasting" => Some (& Self :: RECIPES_MISC_QUARTZ_FROM_BLASTING) , "recipes/misc/raiser_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/raiser_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/raw_copper" => Some (& Self :: RECIPES_MISC_RAW_COPPER) , "recipes/misc/raw_gold" => Some (& Self :: RECIPES_MISC_RAW_GOLD) , "recipes/misc/raw_iron" => Some (& Self :: RECIPES_MISC_RAW_IRON) , "recipes/misc/red_dye_from_beetroot" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_BEETROOT) , "recipes/misc/red_dye_from_poppy" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_POPPY) , "recipes/misc/red_dye_from_rose_bush" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_ROSE_BUSH) , "recipes/misc/red_dye_from_tulip" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_TULIP) , "recipes/misc/resin_brick" => Some (& Self :: RECIPES_MISC_RESIN_BRICK) , "recipes/misc/resin_clump" => Some (& Self :: RECIPES_MISC_RESIN_CLUMP) , "recipes/misc/rib_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/rib_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/sentry_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/sentry_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/shaper_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/shaper_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/silence_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/silence_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/skull_banner_pattern" => Some (& Self :: RECIPES_MISC_SKULL_BANNER_PATTERN) , "recipes/misc/slime_ball" => Some (& Self :: RECIPES_MISC_SLIME_BALL) , "recipes/misc/snout_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/snout_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/spire_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/spire_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/stick" => Some (& Self :: RECIPES_MISC_STICK) , "recipes/misc/stick_from_bamboo_item" => Some (& Self :: RECIPES_MISC_STICK_FROM_BAMBOO_ITEM) , "recipes/misc/sugar_from_honey_bottle" => Some (& Self :: RECIPES_MISC_SUGAR_FROM_HONEY_BOTTLE) , "recipes/misc/sugar_from_sugar_cane" => Some (& Self :: RECIPES_MISC_SUGAR_FROM_SUGAR_CANE) , "recipes/misc/tide_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/tide_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/vex_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/vex_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/ward_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/ward_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/wayfinder_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/wayfinder_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/wheat" => Some (& Self :: RECIPES_MISC_WHEAT) , "recipes/misc/white_dye" => Some (& Self :: RECIPES_MISC_WHITE_DYE) , "recipes/misc/white_dye_from_lily_of_the_valley" => Some (& Self :: RECIPES_MISC_WHITE_DYE_FROM_LILY_OF_THE_VALLEY) , "recipes/misc/wild_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/wild_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/wind_charge" => Some (& Self :: RECIPES_MISC_WIND_CHARGE) , "recipes/misc/writable_book" => Some (& Self :: RECIPES_MISC_WRITABLE_BOOK) , "recipes/misc/yellow_dye_from_dandelion" => Some (& Self :: RECIPES_MISC_YELLOW_DYE_FROM_DANDELION) , "recipes/misc/yellow_dye_from_sunflower" => Some (& Self :: RECIPES_MISC_YELLOW_DYE_FROM_SUNFLOWER) , "recipes/misc/yellow_dye_from_wildflowers" => Some (& Self :: RECIPES_MISC_YELLOW_DYE_FROM_WILDFLOWERS) , "recipes/redstone/acacia_button" => Some (& Self :: RECIPES_REDSTONE_ACACIA_BUTTON) , "recipes/redstone/acacia_door" => Some (& Self :: RECIPES_REDSTONE_ACACIA_DOOR) , "recipes/redstone/acacia_fence_gate" => Some (& Self :: RECIPES_REDSTONE_ACACIA_FENCE_GATE) , "recipes/redstone/acacia_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_ACACIA_PRESSURE_PLATE) , "recipes/redstone/acacia_trapdoor" => Some (& Self :: RECIPES_REDSTONE_ACACIA_TRAPDOOR) , "recipes/redstone/bamboo_button" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_BUTTON) , "recipes/redstone/bamboo_door" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_DOOR) , "recipes/redstone/bamboo_fence_gate" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_FENCE_GATE) , "recipes/redstone/bamboo_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_PRESSURE_PLATE) , "recipes/redstone/bamboo_trapdoor" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_TRAPDOOR) , "recipes/redstone/birch_button" => Some (& Self :: RECIPES_REDSTONE_BIRCH_BUTTON) , "recipes/redstone/birch_door" => Some (& Self :: RECIPES_REDSTONE_BIRCH_DOOR) , "recipes/redstone/birch_fence_gate" => Some (& Self :: RECIPES_REDSTONE_BIRCH_FENCE_GATE) , "recipes/redstone/birch_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_BIRCH_PRESSURE_PLATE) , "recipes/redstone/birch_trapdoor" => Some (& Self :: RECIPES_REDSTONE_BIRCH_TRAPDOOR) , "recipes/redstone/calibrated_sculk_sensor" => Some (& Self :: RECIPES_REDSTONE_CALIBRATED_SCULK_SENSOR) , "recipes/redstone/cherry_button" => Some (& Self :: RECIPES_REDSTONE_CHERRY_BUTTON) , "recipes/redstone/cherry_door" => Some (& Self :: RECIPES_REDSTONE_CHERRY_DOOR) , "recipes/redstone/cherry_fence_gate" => Some (& Self :: RECIPES_REDSTONE_CHERRY_FENCE_GATE) , "recipes/redstone/cherry_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_CHERRY_PRESSURE_PLATE) , "recipes/redstone/cherry_trapdoor" => Some (& Self :: RECIPES_REDSTONE_CHERRY_TRAPDOOR) , "recipes/redstone/comparator" => Some (& Self :: RECIPES_REDSTONE_COMPARATOR) , "recipes/redstone/copper_bulb" => Some (& Self :: RECIPES_REDSTONE_COPPER_BULB) , "recipes/redstone/copper_door" => Some (& Self :: RECIPES_REDSTONE_COPPER_DOOR) , "recipes/redstone/copper_trapdoor" => Some (& Self :: RECIPES_REDSTONE_COPPER_TRAPDOOR) , "recipes/redstone/crafter" => Some (& Self :: RECIPES_REDSTONE_CRAFTER) , "recipes/redstone/crimson_button" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_BUTTON) , "recipes/redstone/crimson_door" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_DOOR) , "recipes/redstone/crimson_fence_gate" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_FENCE_GATE) , "recipes/redstone/crimson_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_PRESSURE_PLATE) , "recipes/redstone/crimson_trapdoor" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_TRAPDOOR) , "recipes/redstone/dark_oak_button" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_BUTTON) , "recipes/redstone/dark_oak_door" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_DOOR) , "recipes/redstone/dark_oak_fence_gate" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_FENCE_GATE) , "recipes/redstone/dark_oak_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_PRESSURE_PLATE) , "recipes/redstone/dark_oak_trapdoor" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_TRAPDOOR) , "recipes/redstone/daylight_detector" => Some (& Self :: RECIPES_REDSTONE_DAYLIGHT_DETECTOR) , "recipes/redstone/dispenser" => Some (& Self :: RECIPES_REDSTONE_DISPENSER) , "recipes/redstone/dropper" => Some (& Self :: RECIPES_REDSTONE_DROPPER) , "recipes/redstone/exposed_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_EXPOSED_COPPER_BULB) , "recipes/redstone/heavy_weighted_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_HEAVY_WEIGHTED_PRESSURE_PLATE) , "recipes/redstone/honey_block" => Some (& Self :: RECIPES_REDSTONE_HONEY_BLOCK) , "recipes/redstone/hopper" => Some (& Self :: RECIPES_REDSTONE_HOPPER) , "recipes/redstone/iron_door" => Some (& Self :: RECIPES_REDSTONE_IRON_DOOR) , "recipes/redstone/iron_trapdoor" => Some (& Self :: RECIPES_REDSTONE_IRON_TRAPDOOR) , "recipes/redstone/jungle_button" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_BUTTON) , "recipes/redstone/jungle_door" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_DOOR) , "recipes/redstone/jungle_fence_gate" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_FENCE_GATE) , "recipes/redstone/jungle_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_PRESSURE_PLATE) , "recipes/redstone/jungle_trapdoor" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_TRAPDOOR) , "recipes/redstone/lectern" => Some (& Self :: RECIPES_REDSTONE_LECTERN) , "recipes/redstone/lever" => Some (& Self :: RECIPES_REDSTONE_LEVER) , "recipes/redstone/light_weighted_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_LIGHT_WEIGHTED_PRESSURE_PLATE) , "recipes/redstone/lightning_rod" => Some (& Self :: RECIPES_REDSTONE_LIGHTNING_ROD) , "recipes/redstone/mangrove_button" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_BUTTON) , "recipes/redstone/mangrove_door" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_DOOR) , "recipes/redstone/mangrove_fence_gate" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_FENCE_GATE) , "recipes/redstone/mangrove_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_PRESSURE_PLATE) , "recipes/redstone/mangrove_trapdoor" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_TRAPDOOR) , "recipes/redstone/note_block" => Some (& Self :: RECIPES_REDSTONE_NOTE_BLOCK) , "recipes/redstone/oak_button" => Some (& Self :: RECIPES_REDSTONE_OAK_BUTTON) , "recipes/redstone/oak_door" => Some (& Self :: RECIPES_REDSTONE_OAK_DOOR) , "recipes/redstone/oak_fence_gate" => Some (& Self :: RECIPES_REDSTONE_OAK_FENCE_GATE) , "recipes/redstone/oak_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_OAK_PRESSURE_PLATE) , "recipes/redstone/oak_trapdoor" => Some (& Self :: RECIPES_REDSTONE_OAK_TRAPDOOR) , "recipes/redstone/observer" => Some (& Self :: RECIPES_REDSTONE_OBSERVER) , "recipes/redstone/oxidized_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_OXIDIZED_COPPER_BULB) , "recipes/redstone/pale_oak_button" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_BUTTON) , "recipes/redstone/pale_oak_door" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_DOOR) , "recipes/redstone/pale_oak_fence_gate" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_FENCE_GATE) , "recipes/redstone/pale_oak_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_PRESSURE_PLATE) , "recipes/redstone/pale_oak_trapdoor" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_TRAPDOOR) , "recipes/redstone/piston" => Some (& Self :: RECIPES_REDSTONE_PISTON) , "recipes/redstone/polished_blackstone_button" => Some (& Self :: RECIPES_REDSTONE_POLISHED_BLACKSTONE_BUTTON) , "recipes/redstone/polished_blackstone_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_POLISHED_BLACKSTONE_PRESSURE_PLATE) , "recipes/redstone/redstone" => Some (& Self :: RECIPES_REDSTONE_REDSTONE) , "recipes/redstone/redstone_block" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_BLOCK) , "recipes/redstone/redstone_from_blasting_deepslate_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_DEEPSLATE_REDSTONE_ORE) , "recipes/redstone/redstone_from_blasting_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_REDSTONE_ORE) , "recipes/redstone/redstone_from_smelting_deepslate_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_DEEPSLATE_REDSTONE_ORE) , "recipes/redstone/redstone_from_smelting_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_REDSTONE_ORE) , "recipes/redstone/redstone_lamp" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_LAMP) , "recipes/redstone/redstone_torch" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_TORCH) , "recipes/redstone/repeater" => Some (& Self :: RECIPES_REDSTONE_REPEATER) , "recipes/redstone/slime_block" => Some (& Self :: RECIPES_REDSTONE_SLIME_BLOCK) , "recipes/redstone/spruce_button" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_BUTTON) , "recipes/redstone/spruce_door" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_DOOR) , "recipes/redstone/spruce_fence_gate" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_FENCE_GATE) , "recipes/redstone/spruce_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_PRESSURE_PLATE) , "recipes/redstone/spruce_trapdoor" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_TRAPDOOR) , "recipes/redstone/sticky_piston" => Some (& Self :: RECIPES_REDSTONE_STICKY_PISTON) , "recipes/redstone/stone_button" => Some (& Self :: RECIPES_REDSTONE_STONE_BUTTON) , "recipes/redstone/stone_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_STONE_PRESSURE_PLATE) , "recipes/redstone/target" => Some (& Self :: RECIPES_REDSTONE_TARGET) , "recipes/redstone/tnt" => Some (& Self :: RECIPES_REDSTONE_TNT) , "recipes/redstone/trapped_chest" => Some (& Self :: RECIPES_REDSTONE_TRAPPED_CHEST) , "recipes/redstone/tripwire_hook" => Some (& Self :: RECIPES_REDSTONE_TRIPWIRE_HOOK) , "recipes/redstone/warped_button" => Some (& Self :: RECIPES_REDSTONE_WARPED_BUTTON) , "recipes/redstone/warped_door" => Some (& Self :: RECIPES_REDSTONE_WARPED_DOOR) , "recipes/redstone/warped_fence_gate" => Some (& Self :: RECIPES_REDSTONE_WARPED_FENCE_GATE) , "recipes/redstone/warped_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_WARPED_PRESSURE_PLATE) , "recipes/redstone/warped_trapdoor" => Some (& Self :: RECIPES_REDSTONE_WARPED_TRAPDOOR) , "recipes/redstone/waxed_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_BULB) , "recipes/redstone/waxed_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_exposed_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB) , "recipes/redstone/waxed_exposed_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_exposed_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_exposed_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_oxidized_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB) , "recipes/redstone/waxed_oxidized_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_oxidized_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_oxidized_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_weathered_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB) , "recipes/redstone/waxed_weathered_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_weathered_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_weathered_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/weathered_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WEATHERED_COPPER_BULB) , "recipes/root" => Some (& Self :: RECIPES_ROOT) , "recipes/tools/black_bundle" => Some (& Self :: RECIPES_TOOLS_BLACK_BUNDLE) , "recipes/tools/blue_bundle" => Some (& Self :: RECIPES_TOOLS_BLUE_BUNDLE) , "recipes/tools/brown_bundle" => Some (& Self :: RECIPES_TOOLS_BROWN_BUNDLE) , "recipes/tools/brush" => Some (& Self :: RECIPES_TOOLS_BRUSH) , "recipes/tools/bundle" => Some (& Self :: RECIPES_TOOLS_BUNDLE) , "recipes/tools/clock" => Some (& Self :: RECIPES_TOOLS_CLOCK) , "recipes/tools/compass" => Some (& Self :: RECIPES_TOOLS_COMPASS) , "recipes/tools/copper_axe" => Some (& Self :: RECIPES_TOOLS_COPPER_AXE) , "recipes/tools/copper_hoe" => Some (& Self :: RECIPES_TOOLS_COPPER_HOE) , "recipes/tools/copper_pickaxe" => Some (& Self :: RECIPES_TOOLS_COPPER_PICKAXE) , "recipes/tools/copper_shovel" => Some (& Self :: RECIPES_TOOLS_COPPER_SHOVEL) , "recipes/tools/cyan_bundle" => Some (& Self :: RECIPES_TOOLS_CYAN_BUNDLE) , "recipes/tools/diamond_axe" => Some (& Self :: RECIPES_TOOLS_DIAMOND_AXE) , "recipes/tools/diamond_hoe" => Some (& Self :: RECIPES_TOOLS_DIAMOND_HOE) , "recipes/tools/diamond_pickaxe" => Some (& Self :: RECIPES_TOOLS_DIAMOND_PICKAXE) , "recipes/tools/diamond_shovel" => Some (& Self :: RECIPES_TOOLS_DIAMOND_SHOVEL) , "recipes/tools/fishing_rod" => Some (& Self :: RECIPES_TOOLS_FISHING_ROD) , "recipes/tools/flint_and_steel" => Some (& Self :: RECIPES_TOOLS_FLINT_AND_STEEL) , "recipes/tools/golden_axe" => Some (& Self :: RECIPES_TOOLS_GOLDEN_AXE) , "recipes/tools/golden_hoe" => Some (& Self :: RECIPES_TOOLS_GOLDEN_HOE) , "recipes/tools/golden_pickaxe" => Some (& Self :: RECIPES_TOOLS_GOLDEN_PICKAXE) , "recipes/tools/golden_shovel" => Some (& Self :: RECIPES_TOOLS_GOLDEN_SHOVEL) , "recipes/tools/gray_bundle" => Some (& Self :: RECIPES_TOOLS_GRAY_BUNDLE) , "recipes/tools/green_bundle" => Some (& Self :: RECIPES_TOOLS_GREEN_BUNDLE) , "recipes/tools/iron_axe" => Some (& Self :: RECIPES_TOOLS_IRON_AXE) , "recipes/tools/iron_hoe" => Some (& Self :: RECIPES_TOOLS_IRON_HOE) , "recipes/tools/iron_pickaxe" => Some (& Self :: RECIPES_TOOLS_IRON_PICKAXE) , "recipes/tools/iron_shovel" => Some (& Self :: RECIPES_TOOLS_IRON_SHOVEL) , "recipes/tools/lead" => Some (& Self :: RECIPES_TOOLS_LEAD) , "recipes/tools/light_blue_bundle" => Some (& Self :: RECIPES_TOOLS_LIGHT_BLUE_BUNDLE) , "recipes/tools/light_gray_bundle" => Some (& Self :: RECIPES_TOOLS_LIGHT_GRAY_BUNDLE) , "recipes/tools/lime_bundle" => Some (& Self :: RECIPES_TOOLS_LIME_BUNDLE) , "recipes/tools/magenta_bundle" => Some (& Self :: RECIPES_TOOLS_MAGENTA_BUNDLE) , "recipes/tools/netherite_axe_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_AXE_SMITHING) , "recipes/tools/netherite_hoe_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_HOE_SMITHING) , "recipes/tools/netherite_pickaxe_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_PICKAXE_SMITHING) , "recipes/tools/netherite_shovel_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_SHOVEL_SMITHING) , "recipes/tools/orange_bundle" => Some (& Self :: RECIPES_TOOLS_ORANGE_BUNDLE) , "recipes/tools/pink_bundle" => Some (& Self :: RECIPES_TOOLS_PINK_BUNDLE) , "recipes/tools/purple_bundle" => Some (& Self :: RECIPES_TOOLS_PURPLE_BUNDLE) , "recipes/tools/recovery_compass" => Some (& Self :: RECIPES_TOOLS_RECOVERY_COMPASS) , "recipes/tools/red_bundle" => Some (& Self :: RECIPES_TOOLS_RED_BUNDLE) , "recipes/tools/shears" => Some (& Self :: RECIPES_TOOLS_SHEARS) , "recipes/tools/spyglass" => Some (& Self :: RECIPES_TOOLS_SPYGLASS) , "recipes/tools/stone_axe" => Some (& Self :: RECIPES_TOOLS_STONE_AXE) , "recipes/tools/stone_hoe" => Some (& Self :: RECIPES_TOOLS_STONE_HOE) , "recipes/tools/stone_pickaxe" => Some (& Self :: RECIPES_TOOLS_STONE_PICKAXE) , "recipes/tools/stone_shovel" => Some (& Self :: RECIPES_TOOLS_STONE_SHOVEL) , "recipes/tools/white_bundle" => Some (& Self :: RECIPES_TOOLS_WHITE_BUNDLE) , "recipes/tools/wooden_axe" => Some (& Self :: RECIPES_TOOLS_WOODEN_AXE) , "recipes/tools/wooden_hoe" => Some (& Self :: RECIPES_TOOLS_WOODEN_HOE) , "recipes/tools/wooden_pickaxe" => Some (& Self :: RECIPES_TOOLS_WOODEN_PICKAXE) , "recipes/tools/wooden_shovel" => Some (& Self :: RECIPES_TOOLS_WOODEN_SHOVEL) , "recipes/tools/yellow_bundle" => Some (& Self :: RECIPES_TOOLS_YELLOW_BUNDLE) , "recipes/transportation/acacia_boat" => Some (& Self :: RECIPES_TRANSPORTATION_ACACIA_BOAT) , "recipes/transportation/acacia_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_ACACIA_CHEST_BOAT) , "recipes/transportation/activator_rail" => Some (& Self :: RECIPES_TRANSPORTATION_ACTIVATOR_RAIL) , "recipes/transportation/bamboo_chest_raft" => Some (& Self :: RECIPES_TRANSPORTATION_BAMBOO_CHEST_RAFT) , "recipes/transportation/bamboo_raft" => Some (& Self :: RECIPES_TRANSPORTATION_BAMBOO_RAFT) , "recipes/transportation/birch_boat" => Some (& Self :: RECIPES_TRANSPORTATION_BIRCH_BOAT) , "recipes/transportation/birch_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_BIRCH_CHEST_BOAT) , "recipes/transportation/carrot_on_a_stick" => Some (& Self :: RECIPES_TRANSPORTATION_CARROT_ON_A_STICK) , "recipes/transportation/cherry_boat" => Some (& Self :: RECIPES_TRANSPORTATION_CHERRY_BOAT) , "recipes/transportation/cherry_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_CHERRY_CHEST_BOAT) , "recipes/transportation/chest_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_CHEST_MINECART) , "recipes/transportation/dark_oak_boat" => Some (& Self :: RECIPES_TRANSPORTATION_DARK_OAK_BOAT) , "recipes/transportation/dark_oak_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_DARK_OAK_CHEST_BOAT) , "recipes/transportation/detector_rail" => Some (& Self :: RECIPES_TRANSPORTATION_DETECTOR_RAIL) , "recipes/transportation/furnace_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_FURNACE_MINECART) , "recipes/transportation/hopper_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_HOPPER_MINECART) , "recipes/transportation/jungle_boat" => Some (& Self :: RECIPES_TRANSPORTATION_JUNGLE_BOAT) , "recipes/transportation/jungle_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_JUNGLE_CHEST_BOAT) , "recipes/transportation/mangrove_boat" => Some (& Self :: RECIPES_TRANSPORTATION_MANGROVE_BOAT) , "recipes/transportation/mangrove_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_MANGROVE_CHEST_BOAT) , "recipes/transportation/minecart" => Some (& Self :: RECIPES_TRANSPORTATION_MINECART) , "recipes/transportation/oak_boat" => Some (& Self :: RECIPES_TRANSPORTATION_OAK_BOAT) , "recipes/transportation/oak_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_OAK_CHEST_BOAT) , "recipes/transportation/pale_oak_boat" => Some (& Self :: RECIPES_TRANSPORTATION_PALE_OAK_BOAT) , "recipes/transportation/pale_oak_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_PALE_OAK_CHEST_BOAT) , "recipes/transportation/powered_rail" => Some (& Self :: RECIPES_TRANSPORTATION_POWERED_RAIL) , "recipes/transportation/rail" => Some (& Self :: RECIPES_TRANSPORTATION_RAIL) , "recipes/transportation/spruce_boat" => Some (& Self :: RECIPES_TRANSPORTATION_SPRUCE_BOAT) , "recipes/transportation/spruce_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_SPRUCE_CHEST_BOAT) , "recipes/transportation/tnt_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_TNT_MINECART) , "recipes/transportation/warped_fungus_on_a_stick" => Some (& Self :: RECIPES_TRANSPORTATION_WARPED_FUNGUS_ON_A_STICK) , "story/cure_zombie_villager" => Some (& Self :: STORY_CURE_ZOMBIE_VILLAGER) , "story/deflect_arrow" => Some (& Self :: STORY_DEFLECT_ARROW) , "story/enchant_item" => Some (& Self :: STORY_ENCHANT_ITEM) , "story/enter_the_end" => Some (& Self :: STORY_ENTER_THE_END) , "story/enter_the_nether" => Some (& Self :: STORY_ENTER_THE_NETHER) , "story/follow_ender_eye" => Some (& Self :: STORY_FOLLOW_ENDER_EYE) , "story/form_obsidian" => Some (& Self :: STORY_FORM_OBSIDIAN) , "story/iron_tools" => Some (& Self :: STORY_IRON_TOOLS) , "story/lava_bucket" => Some (& Self :: STORY_LAVA_BUCKET) , "story/mine_diamond" => Some (& Self :: STORY_MINE_DIAMOND) , "story/mine_stone" => Some (& Self :: STORY_MINE_STONE) , "story/obtain_armor" => Some (& Self :: STORY_OBTAIN_ARMOR) , "story/root" => Some (& Self :: STORY_ROOT) , "story/shiny_gear" => Some (& Self :: STORY_SHINY_GEAR) , "story/smelt_iron" => Some (& Self :: STORY_SMELT_IRON) , "story/upgrade_tools" => Some (& Self :: STORY_UPGRADE_TOOLS) , _ => None }
    }
    pub fn from_minecraft_name(name: &str) -> Option<&'static Self> {
        match name { "adventure/adventuring_time" => Some (& Self :: ADVENTURE_ADVENTURING_TIME) , "adventure/arbalistic" => Some (& Self :: ADVENTURE_ARBALISTIC) , "adventure/avoid_vibration" => Some (& Self :: ADVENTURE_AVOID_VIBRATION) , "adventure/blowback" => Some (& Self :: ADVENTURE_BLOWBACK) , "adventure/brush_armadillo" => Some (& Self :: ADVENTURE_BRUSH_ARMADILLO) , "adventure/bullseye" => Some (& Self :: ADVENTURE_BULLSEYE) , "adventure/craft_decorated_pot_using_only_sherds" => Some (& Self :: ADVENTURE_CRAFT_DECORATED_POT_USING_ONLY_SHERDS) , "adventure/crafters_crafting_crafters" => Some (& Self :: ADVENTURE_CRAFTERS_CRAFTING_CRAFTERS) , "adventure/fall_from_world_height" => Some (& Self :: ADVENTURE_FALL_FROM_WORLD_HEIGHT) , "adventure/heart_transplanter" => Some (& Self :: ADVENTURE_HEART_TRANSPLANTER) , "adventure/hero_of_the_village" => Some (& Self :: ADVENTURE_HERO_OF_THE_VILLAGE) , "adventure/honey_block_slide" => Some (& Self :: ADVENTURE_HONEY_BLOCK_SLIDE) , "adventure/kill_a_mob" => Some (& Self :: ADVENTURE_KILL_A_MOB) , "adventure/kill_all_mobs" => Some (& Self :: ADVENTURE_KILL_ALL_MOBS) , "adventure/kill_mob_near_sculk_catalyst" => Some (& Self :: ADVENTURE_KILL_MOB_NEAR_SCULK_CATALYST) , "adventure/lighten_up" => Some (& Self :: ADVENTURE_LIGHTEN_UP) , "adventure/lightning_rod_with_villager_no_fire" => Some (& Self :: ADVENTURE_LIGHTNING_ROD_WITH_VILLAGER_NO_FIRE) , "adventure/minecraft_trials_edition" => Some (& Self :: ADVENTURE_MINECRAFT_TRIALS_EDITION) , "adventure/ol_betsy" => Some (& Self :: ADVENTURE_OL_BETSY) , "adventure/overoverkill" => Some (& Self :: ADVENTURE_OVEROVERKILL) , "adventure/play_jukebox_in_meadows" => Some (& Self :: ADVENTURE_PLAY_JUKEBOX_IN_MEADOWS) , "adventure/read_power_of_chiseled_bookshelf" => Some (& Self :: ADVENTURE_READ_POWER_OF_CHISELED_BOOKSHELF) , "adventure/revaulting" => Some (& Self :: ADVENTURE_REVAULTING) , "adventure/root" => Some (& Self :: ADVENTURE_ROOT) , "adventure/salvage_sherd" => Some (& Self :: ADVENTURE_SALVAGE_SHERD) , "adventure/shoot_arrow" => Some (& Self :: ADVENTURE_SHOOT_ARROW) , "adventure/sleep_in_bed" => Some (& Self :: ADVENTURE_SLEEP_IN_BED) , "adventure/sniper_duel" => Some (& Self :: ADVENTURE_SNIPER_DUEL) , "adventure/spear_many_mobs" => Some (& Self :: ADVENTURE_SPEAR_MANY_MOBS) , "adventure/spyglass_at_dragon" => Some (& Self :: ADVENTURE_SPYGLASS_AT_DRAGON) , "adventure/spyglass_at_ghast" => Some (& Self :: ADVENTURE_SPYGLASS_AT_GHAST) , "adventure/spyglass_at_parrot" => Some (& Self :: ADVENTURE_SPYGLASS_AT_PARROT) , "adventure/summon_iron_golem" => Some (& Self :: ADVENTURE_SUMMON_IRON_GOLEM) , "adventure/throw_trident" => Some (& Self :: ADVENTURE_THROW_TRIDENT) , "adventure/totem_of_undying" => Some (& Self :: ADVENTURE_TOTEM_OF_UNDYING) , "adventure/trade" => Some (& Self :: ADVENTURE_TRADE) , "adventure/trade_at_world_height" => Some (& Self :: ADVENTURE_TRADE_AT_WORLD_HEIGHT) , "adventure/trim_with_all_exclusive_armor_patterns" => Some (& Self :: ADVENTURE_TRIM_WITH_ALL_EXCLUSIVE_ARMOR_PATTERNS) , "adventure/trim_with_any_armor_pattern" => Some (& Self :: ADVENTURE_TRIM_WITH_ANY_ARMOR_PATTERN) , "adventure/two_birds_one_arrow" => Some (& Self :: ADVENTURE_TWO_BIRDS_ONE_ARROW) , "adventure/under_lock_and_key" => Some (& Self :: ADVENTURE_UNDER_LOCK_AND_KEY) , "adventure/use_lodestone" => Some (& Self :: ADVENTURE_USE_LODESTONE) , "adventure/very_very_frightening" => Some (& Self :: ADVENTURE_VERY_VERY_FRIGHTENING) , "adventure/voluntary_exile" => Some (& Self :: ADVENTURE_VOLUNTARY_EXILE) , "adventure/walk_on_powder_snow_with_leather_boots" => Some (& Self :: ADVENTURE_WALK_ON_POWDER_SNOW_WITH_LEATHER_BOOTS) , "adventure/who_needs_rockets" => Some (& Self :: ADVENTURE_WHO_NEEDS_ROCKETS) , "adventure/whos_the_pillager_now" => Some (& Self :: ADVENTURE_WHOS_THE_PILLAGER_NOW) , "end/dragon_breath" => Some (& Self :: END_DRAGON_BREATH) , "end/dragon_egg" => Some (& Self :: END_DRAGON_EGG) , "end/elytra" => Some (& Self :: END_ELYTRA) , "end/enter_end_gateway" => Some (& Self :: END_ENTER_END_GATEWAY) , "end/find_end_city" => Some (& Self :: END_FIND_END_CITY) , "end/kill_dragon" => Some (& Self :: END_KILL_DRAGON) , "end/levitate" => Some (& Self :: END_LEVITATE) , "end/respawn_dragon" => Some (& Self :: END_RESPAWN_DRAGON) , "end/root" => Some (& Self :: END_ROOT) , "husbandry/allay_deliver_cake_to_note_block" => Some (& Self :: HUSBANDRY_ALLAY_DELIVER_CAKE_TO_NOTE_BLOCK) , "husbandry/allay_deliver_item_to_player" => Some (& Self :: HUSBANDRY_ALLAY_DELIVER_ITEM_TO_PLAYER) , "husbandry/axolotl_in_a_bucket" => Some (& Self :: HUSBANDRY_AXOLOTL_IN_A_BUCKET) , "husbandry/balanced_diet" => Some (& Self :: HUSBANDRY_BALANCED_DIET) , "husbandry/bred_all_animals" => Some (& Self :: HUSBANDRY_BRED_ALL_ANIMALS) , "husbandry/breed_an_animal" => Some (& Self :: HUSBANDRY_BREED_AN_ANIMAL) , "husbandry/complete_catalogue" => Some (& Self :: HUSBANDRY_COMPLETE_CATALOGUE) , "husbandry/feed_snifflet" => Some (& Self :: HUSBANDRY_FEED_SNIFFLET) , "husbandry/fishy_business" => Some (& Self :: HUSBANDRY_FISHY_BUSINESS) , "husbandry/froglights" => Some (& Self :: HUSBANDRY_FROGLIGHTS) , "husbandry/kill_axolotl_target" => Some (& Self :: HUSBANDRY_KILL_AXOLOTL_TARGET) , "husbandry/leash_all_frog_variants" => Some (& Self :: HUSBANDRY_LEASH_ALL_FROG_VARIANTS) , "husbandry/make_a_sign_glow" => Some (& Self :: HUSBANDRY_MAKE_A_SIGN_GLOW) , "husbandry/obtain_netherite_hoe" => Some (& Self :: HUSBANDRY_OBTAIN_NETHERITE_HOE) , "husbandry/obtain_sniffer_egg" => Some (& Self :: HUSBANDRY_OBTAIN_SNIFFER_EGG) , "husbandry/place_dried_ghast_in_water" => Some (& Self :: HUSBANDRY_PLACE_DRIED_GHAST_IN_WATER) , "husbandry/plant_any_sniffer_seed" => Some (& Self :: HUSBANDRY_PLANT_ANY_SNIFFER_SEED) , "husbandry/plant_seed" => Some (& Self :: HUSBANDRY_PLANT_SEED) , "husbandry/remove_wolf_armor" => Some (& Self :: HUSBANDRY_REMOVE_WOLF_ARMOR) , "husbandry/repair_wolf_armor" => Some (& Self :: HUSBANDRY_REPAIR_WOLF_ARMOR) , "husbandry/ride_a_boat_with_a_goat" => Some (& Self :: HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT) , "husbandry/root" => Some (& Self :: HUSBANDRY_ROOT) , "husbandry/safely_harvest_honey" => Some (& Self :: HUSBANDRY_SAFELY_HARVEST_HONEY) , "husbandry/silk_touch_nest" => Some (& Self :: HUSBANDRY_SILK_TOUCH_NEST) , "husbandry/tactical_fishing" => Some (& Self :: HUSBANDRY_TACTICAL_FISHING) , "husbandry/tadpole_in_a_bucket" => Some (& Self :: HUSBANDRY_TADPOLE_IN_A_BUCKET) , "husbandry/tame_an_animal" => Some (& Self :: HUSBANDRY_TAME_AN_ANIMAL) , "husbandry/wax_off" => Some (& Self :: HUSBANDRY_WAX_OFF) , "husbandry/wax_on" => Some (& Self :: HUSBANDRY_WAX_ON) , "husbandry/whole_pack" => Some (& Self :: HUSBANDRY_WHOLE_PACK) , "nether/all_effects" => Some (& Self :: NETHER_ALL_EFFECTS) , "nether/all_potions" => Some (& Self :: NETHER_ALL_POTIONS) , "nether/brew_potion" => Some (& Self :: NETHER_BREW_POTION) , "nether/charge_respawn_anchor" => Some (& Self :: NETHER_CHARGE_RESPAWN_ANCHOR) , "nether/create_beacon" => Some (& Self :: NETHER_CREATE_BEACON) , "nether/create_full_beacon" => Some (& Self :: NETHER_CREATE_FULL_BEACON) , "nether/distract_piglin" => Some (& Self :: NETHER_DISTRACT_PIGLIN) , "nether/explore_nether" => Some (& Self :: NETHER_EXPLORE_NETHER) , "nether/fast_travel" => Some (& Self :: NETHER_FAST_TRAVEL) , "nether/find_bastion" => Some (& Self :: NETHER_FIND_BASTION) , "nether/find_fortress" => Some (& Self :: NETHER_FIND_FORTRESS) , "nether/get_wither_skull" => Some (& Self :: NETHER_GET_WITHER_SKULL) , "nether/loot_bastion" => Some (& Self :: NETHER_LOOT_BASTION) , "nether/netherite_armor" => Some (& Self :: NETHER_NETHERITE_ARMOR) , "nether/obtain_ancient_debris" => Some (& Self :: NETHER_OBTAIN_ANCIENT_DEBRIS) , "nether/obtain_blaze_rod" => Some (& Self :: NETHER_OBTAIN_BLAZE_ROD) , "nether/obtain_crying_obsidian" => Some (& Self :: NETHER_OBTAIN_CRYING_OBSIDIAN) , "nether/return_to_sender" => Some (& Self :: NETHER_RETURN_TO_SENDER) , "nether/ride_strider" => Some (& Self :: NETHER_RIDE_STRIDER) , "nether/ride_strider_in_overworld_lava" => Some (& Self :: NETHER_RIDE_STRIDER_IN_OVERWORLD_LAVA) , "nether/root" => Some (& Self :: NETHER_ROOT) , "nether/summon_wither" => Some (& Self :: NETHER_SUMMON_WITHER) , "nether/uneasy_alliance" => Some (& Self :: NETHER_UNEASY_ALLIANCE) , "recipes/brewing/blaze_powder" => Some (& Self :: RECIPES_BREWING_BLAZE_POWDER) , "recipes/brewing/brewing_stand" => Some (& Self :: RECIPES_BREWING_BREWING_STAND) , "recipes/brewing/cauldron" => Some (& Self :: RECIPES_BREWING_CAULDRON) , "recipes/brewing/fermented_spider_eye" => Some (& Self :: RECIPES_BREWING_FERMENTED_SPIDER_EYE) , "recipes/brewing/glass_bottle" => Some (& Self :: RECIPES_BREWING_GLASS_BOTTLE) , "recipes/brewing/glistering_melon_slice" => Some (& Self :: RECIPES_BREWING_GLISTERING_MELON_SLICE) , "recipes/brewing/golden_carrot" => Some (& Self :: RECIPES_BREWING_GOLDEN_CARROT) , "recipes/brewing/magma_cream" => Some (& Self :: RECIPES_BREWING_MAGMA_CREAM) , "recipes/building_blocks/acacia_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_PLANKS) , "recipes/building_blocks/acacia_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_SLAB) , "recipes/building_blocks/acacia_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_STAIRS) , "recipes/building_blocks/acacia_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ACACIA_WOOD) , "recipes/building_blocks/amethyst_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_AMETHYST_BLOCK) , "recipes/building_blocks/andesite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE) , "recipes/building_blocks/andesite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB) , "recipes/building_blocks/andesite_slab_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/andesite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS) , "recipes/building_blocks/andesite_stairs_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/bamboo_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_BLOCK) , "recipes/building_blocks/bamboo_mosaic_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_SLAB) , "recipes/building_blocks/bamboo_mosaic_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_STAIRS) , "recipes/building_blocks/bamboo_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_PLANKS) , "recipes/building_blocks/bamboo_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_SLAB) , "recipes/building_blocks/bamboo_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BAMBOO_STAIRS) , "recipes/building_blocks/birch_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_PLANKS) , "recipes/building_blocks/birch_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_SLAB) , "recipes/building_blocks/birch_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_STAIRS) , "recipes/building_blocks/birch_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BIRCH_WOOD) , "recipes/building_blocks/black_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACK_CONCRETE_POWDER) , "recipes/building_blocks/black_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACK_STAINED_GLASS) , "recipes/building_blocks/black_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACK_TERRACOTTA) , "recipes/building_blocks/blackstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB) , "recipes/building_blocks/blackstone_slab_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/blackstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS) , "recipes/building_blocks/blackstone_stairs_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/blue_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_CONCRETE_POWDER) , "recipes/building_blocks/blue_ice" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_ICE) , "recipes/building_blocks/blue_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_STAINED_GLASS) , "recipes/building_blocks/blue_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BLUE_TERRACOTTA) , "recipes/building_blocks/bone_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BONE_BLOCK) , "recipes/building_blocks/bookshelf" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BOOKSHELF) , "recipes/building_blocks/brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_SLAB) , "recipes/building_blocks/brick_slab_from_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_SLAB_FROM_BRICKS_STONECUTTING) , "recipes/building_blocks/brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_STAIRS) , "recipes/building_blocks/brick_stairs_from_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICK_STAIRS_FROM_BRICKS_STONECUTTING) , "recipes/building_blocks/bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BRICKS) , "recipes/building_blocks/brown_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BROWN_CONCRETE_POWDER) , "recipes/building_blocks/brown_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BROWN_STAINED_GLASS) , "recipes/building_blocks/brown_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_BROWN_TERRACOTTA) , "recipes/building_blocks/cherry_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_PLANKS) , "recipes/building_blocks/cherry_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_SLAB) , "recipes/building_blocks/cherry_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_STAIRS) , "recipes/building_blocks/cherry_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHERRY_WOOD) , "recipes/building_blocks/chiseled_bookshelf" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_BOOKSHELF) , "recipes/building_blocks/chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER) , "recipes/building_blocks/chiseled_copper_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/chiseled_copper_from_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/chiseled_deepslate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE) , "recipes/building_blocks/chiseled_deepslate_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/chiseled_nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS) , "recipes/building_blocks/chiseled_nether_bricks_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_polished_blackstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE) , "recipes/building_blocks/chiseled_polished_blackstone_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_polished_blackstone_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_quartz_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK) , "recipes/building_blocks/chiseled_quartz_block_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/chiseled_red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE) , "recipes/building_blocks/chiseled_red_sandstone_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_resin_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS) , "recipes/building_blocks/chiseled_resin_bricks_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE) , "recipes/building_blocks/chiseled_sandstone_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/chiseled_stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS) , "recipes/building_blocks/chiseled_stone_bricks_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_FROM_STONE_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_stone_bricks_stone_from_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_STONE_FROM_STONECUTTING) , "recipes/building_blocks/chiseled_tuff" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF) , "recipes/building_blocks/chiseled_tuff_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS) , "recipes/building_blocks/chiseled_tuff_bricks_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/chiseled_tuff_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/clay" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CLAY) , "recipes/building_blocks/coal_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COAL_BLOCK) , "recipes/building_blocks/coarse_dirt" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COARSE_DIRT) , "recipes/building_blocks/cobbled_deepslate_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB) , "recipes/building_blocks/cobbled_deepslate_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/cobbled_deepslate_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS) , "recipes/building_blocks/cobbled_deepslate_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/cobblestone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB) , "recipes/building_blocks/cobblestone_slab_from_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB_FROM_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/cobblestone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS) , "recipes/building_blocks/cobblestone_stairs_from_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS_FROM_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/copper_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COPPER_BLOCK) , "recipes/building_blocks/copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COPPER_GRATE) , "recipes/building_blocks/copper_grate_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_COPPER_GRATE_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cracked_deepslate_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_BRICKS) , "recipes/building_blocks/cracked_deepslate_tiles" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_TILES) , "recipes/building_blocks/cracked_nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_NETHER_BRICKS) , "recipes/building_blocks/cracked_polished_blackstone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_POLISHED_BLACKSTONE_BRICKS) , "recipes/building_blocks/cracked_stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRACKED_STONE_BRICKS) , "recipes/building_blocks/crimson_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_HYPHAE) , "recipes/building_blocks/crimson_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_PLANKS) , "recipes/building_blocks/crimson_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_SLAB) , "recipes/building_blocks/crimson_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CRIMSON_STAIRS) , "recipes/building_blocks/cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER) , "recipes/building_blocks/cut_copper_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB) , "recipes/building_blocks/cut_copper_slab_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cut_copper_slab_from_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS) , "recipes/building_blocks/cut_copper_stairs_from_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/cut_copper_stairs_from_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/cut_red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE) , "recipes/building_blocks/cut_red_sandstone_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_red_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB) , "recipes/building_blocks/cut_red_sandstone_slab_from_cut_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_CUT_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_red_sandstone_slab_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE) , "recipes/building_blocks/cut_sandstone_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB) , "recipes/building_blocks/cut_sandstone_slab_from_cut_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_CUT_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cut_sandstone_slab_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/cyan_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CYAN_CONCRETE_POWDER) , "recipes/building_blocks/cyan_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CYAN_STAINED_GLASS) , "recipes/building_blocks/cyan_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_CYAN_TERRACOTTA) , "recipes/building_blocks/dark_oak_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_PLANKS) , "recipes/building_blocks/dark_oak_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_SLAB) , "recipes/building_blocks/dark_oak_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_STAIRS) , "recipes/building_blocks/dark_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_OAK_WOOD) , "recipes/building_blocks/dark_prismarine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE) , "recipes/building_blocks/dark_prismarine_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB) , "recipes/building_blocks/dark_prismarine_slab_from_dark_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB_FROM_DARK_PRISMARINE_STONECUTTING) , "recipes/building_blocks/dark_prismarine_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS) , "recipes/building_blocks/dark_prismarine_stairs_from_dark_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS_FROM_DARK_PRISMARINE_STONECUTTING) , "recipes/building_blocks/deepslate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE) , "recipes/building_blocks/deepslate_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB) , "recipes/building_blocks/deepslate_brick_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_brick_slab_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_brick_slab_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS) , "recipes/building_blocks/deepslate_brick_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_brick_stairs_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_brick_stairs_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS) , "recipes/building_blocks/deepslate_bricks_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_bricks_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB) , "recipes/building_blocks/deepslate_tile_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab_from_deepslate_tiles_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_TILES_STONECUTTING) , "recipes/building_blocks/deepslate_tile_slab_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS) , "recipes/building_blocks/deepslate_tile_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_tiles_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_TILES_STONECUTTING) , "recipes/building_blocks/deepslate_tile_stairs_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tiles" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES) , "recipes/building_blocks/deepslate_tiles_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/deepslate_tiles_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/building_blocks/deepslate_tiles_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/diamond_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIAMOND_BLOCK) , "recipes/building_blocks/diorite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE) , "recipes/building_blocks/diorite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_SLAB) , "recipes/building_blocks/diorite_slab_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_SLAB_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/diorite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS) , "recipes/building_blocks/diorite_stairs_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/dried_ghast" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DRIED_GHAST) , "recipes/building_blocks/dried_kelp_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DRIED_KELP_BLOCK) , "recipes/building_blocks/dripstone_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DRIPSTONE_BLOCK) , "recipes/building_blocks/dye_black_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_BLACK_WOOL) , "recipes/building_blocks/dye_blue_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_BLUE_WOOL) , "recipes/building_blocks/dye_brown_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_BROWN_WOOL) , "recipes/building_blocks/dye_cyan_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_CYAN_WOOL) , "recipes/building_blocks/dye_gray_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_GRAY_WOOL) , "recipes/building_blocks/dye_green_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_GREEN_WOOL) , "recipes/building_blocks/dye_light_blue_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_LIGHT_BLUE_WOOL) , "recipes/building_blocks/dye_light_gray_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_LIGHT_GRAY_WOOL) , "recipes/building_blocks/dye_lime_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_LIME_WOOL) , "recipes/building_blocks/dye_magenta_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_MAGENTA_WOOL) , "recipes/building_blocks/dye_orange_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_ORANGE_WOOL) , "recipes/building_blocks/dye_pink_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_PINK_WOOL) , "recipes/building_blocks/dye_purple_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_PURPLE_WOOL) , "recipes/building_blocks/dye_red_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_RED_WOOL) , "recipes/building_blocks/dye_white_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_WHITE_WOOL) , "recipes/building_blocks/dye_yellow_wool" => Some (& Self :: RECIPES_BUILDING_BLOCKS_DYE_YELLOW_WOOL) , "recipes/building_blocks/emerald_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EMERALD_BLOCK) , "recipes/building_blocks/end_stone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB) , "recipes/building_blocks/end_stone_brick_slab_from_end_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/end_stone_brick_slab_from_end_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_STONECUTTING) , "recipes/building_blocks/end_stone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS) , "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_STONECUTTING) , "recipes/building_blocks/end_stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS) , "recipes/building_blocks/end_stone_bricks_from_end_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS_FROM_END_STONE_STONECUTTING) , "recipes/building_blocks/exposed_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER) , "recipes/building_blocks/exposed_chiseled_copper_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_chiseled_copper_from_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE) , "recipes/building_blocks/exposed_copper_grate_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER) , "recipes/building_blocks/exposed_cut_copper_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB) , "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS) , "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GLASS) , "recipes/building_blocks/glowstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GLOWSTONE) , "recipes/building_blocks/gold_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GOLD_BLOCK) , "recipes/building_blocks/granite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE) , "recipes/building_blocks/granite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_SLAB) , "recipes/building_blocks/granite_slab_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_SLAB_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/granite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS) , "recipes/building_blocks/granite_stairs_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/gray_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRAY_CONCRETE_POWDER) , "recipes/building_blocks/gray_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRAY_STAINED_GLASS) , "recipes/building_blocks/gray_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GRAY_TERRACOTTA) , "recipes/building_blocks/green_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GREEN_CONCRETE_POWDER) , "recipes/building_blocks/green_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GREEN_STAINED_GLASS) , "recipes/building_blocks/green_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_GREEN_TERRACOTTA) , "recipes/building_blocks/hay_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_HAY_BLOCK) , "recipes/building_blocks/iron_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_IRON_BLOCK) , "recipes/building_blocks/jack_o_lantern" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JACK_O_LANTERN) , "recipes/building_blocks/jungle_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_PLANKS) , "recipes/building_blocks/jungle_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_SLAB) , "recipes/building_blocks/jungle_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_STAIRS) , "recipes/building_blocks/jungle_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_JUNGLE_WOOD) , "recipes/building_blocks/lapis_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LAPIS_BLOCK) , "recipes/building_blocks/light_blue_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_CONCRETE_POWDER) , "recipes/building_blocks/light_blue_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_STAINED_GLASS) , "recipes/building_blocks/light_blue_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_TERRACOTTA) , "recipes/building_blocks/light_gray_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_CONCRETE_POWDER) , "recipes/building_blocks/light_gray_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_STAINED_GLASS) , "recipes/building_blocks/light_gray_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_TERRACOTTA) , "recipes/building_blocks/lime_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIME_CONCRETE_POWDER) , "recipes/building_blocks/lime_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIME_STAINED_GLASS) , "recipes/building_blocks/lime_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_LIME_TERRACOTTA) , "recipes/building_blocks/magenta_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGENTA_CONCRETE_POWDER) , "recipes/building_blocks/magenta_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGENTA_STAINED_GLASS) , "recipes/building_blocks/magenta_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGENTA_TERRACOTTA) , "recipes/building_blocks/magma_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MAGMA_BLOCK) , "recipes/building_blocks/mangrove_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_PLANKS) , "recipes/building_blocks/mangrove_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_SLAB) , "recipes/building_blocks/mangrove_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_STAIRS) , "recipes/building_blocks/mangrove_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MANGROVE_WOOD) , "recipes/building_blocks/melon" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MELON) , "recipes/building_blocks/mossy_cobblestone_from_moss_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_MOSS_BLOCK) , "recipes/building_blocks/mossy_cobblestone_from_vine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_VINE) , "recipes/building_blocks/mossy_cobblestone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB) , "recipes/building_blocks/mossy_cobblestone_slab_from_mossy_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB_FROM_MOSSY_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/mossy_cobblestone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS) , "recipes/building_blocks/mossy_cobblestone_stairs_from_mossy_cobblestone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS_FROM_MOSSY_COBBLESTONE_STONECUTTING) , "recipes/building_blocks/mossy_stone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB) , "recipes/building_blocks/mossy_stone_brick_slab_from_mossy_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB_FROM_MOSSY_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/mossy_stone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS) , "recipes/building_blocks/mossy_stone_brick_stairs_from_mossy_stone_brick_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS_FROM_MOSSY_STONE_BRICK_STONECUTTING) , "recipes/building_blocks/mossy_stone_bricks_from_moss_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_MOSS_BLOCK) , "recipes/building_blocks/mossy_stone_bricks_from_vine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_VINE) , "recipes/building_blocks/mud_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB) , "recipes/building_blocks/mud_brick_slab_from_mud_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB_FROM_MUD_BRICKS_STONECUTTING) , "recipes/building_blocks/mud_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS) , "recipes/building_blocks/mud_brick_stairs_from_mud_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS_FROM_MUD_BRICKS_STONECUTTING) , "recipes/building_blocks/mud_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUD_BRICKS) , "recipes/building_blocks/muddy_mangrove_roots" => Some (& Self :: RECIPES_BUILDING_BLOCKS_MUDDY_MANGROVE_ROOTS) , "recipes/building_blocks/nether_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB) , "recipes/building_blocks/nether_brick_slab_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/nether_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS) , "recipes/building_blocks/nether_brick_stairs_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_BRICKS) , "recipes/building_blocks/nether_wart_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHER_WART_BLOCK) , "recipes/building_blocks/netherite_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_NETHERITE_BLOCK) , "recipes/building_blocks/oak_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_PLANKS) , "recipes/building_blocks/oak_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_SLAB) , "recipes/building_blocks/oak_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_STAIRS) , "recipes/building_blocks/oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OAK_WOOD) , "recipes/building_blocks/orange_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ORANGE_CONCRETE_POWDER) , "recipes/building_blocks/orange_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ORANGE_STAINED_GLASS) , "recipes/building_blocks/orange_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_ORANGE_TERRACOTTA) , "recipes/building_blocks/oxidized_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER) , "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE) , "recipes/building_blocks/oxidized_copper_grate_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER) , "recipes/building_blocks/oxidized_cut_copper_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB) , "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS) , "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/packed_ice" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PACKED_ICE) , "recipes/building_blocks/packed_mud" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PACKED_MUD) , "recipes/building_blocks/pale_oak_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_PLANKS) , "recipes/building_blocks/pale_oak_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_SLAB) , "recipes/building_blocks/pale_oak_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_STAIRS) , "recipes/building_blocks/pale_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PALE_OAK_WOOD) , "recipes/building_blocks/pink_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PINK_CONCRETE_POWDER) , "recipes/building_blocks/pink_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PINK_STAINED_GLASS) , "recipes/building_blocks/pink_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PINK_TERRACOTTA) , "recipes/building_blocks/polished_andesite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE) , "recipes/building_blocks/polished_andesite_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB) , "recipes/building_blocks/polished_andesite_slab_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_slab_from_polished_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_POLISHED_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS) , "recipes/building_blocks/polished_andesite_stairs_from_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_andesite_stairs_from_polished_andesite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_POLISHED_ANDESITE_STONECUTTING) , "recipes/building_blocks/polished_basalt" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BASALT) , "recipes/building_blocks/polished_basalt_from_basalt_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BASALT_FROM_BASALT_STONECUTTING) , "recipes/building_blocks/polished_blackstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE) , "recipes/building_blocks/polished_blackstone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB) , "recipes/building_blocks/polished_blackstone_brick_slab_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS) , "recipes/building_blocks/polished_blackstone_brick_stairs_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING) , "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS) , "recipes/building_blocks/polished_blackstone_bricks_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_bricks_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB) , "recipes/building_blocks/polished_blackstone_slab_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_slab_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS) , "recipes/building_blocks/polished_blackstone_stairs_from_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_blackstone_stairs_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/building_blocks/polished_deepslate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE) , "recipes/building_blocks/polished_deepslate_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB) , "recipes/building_blocks/polished_deepslate_slab_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_slab_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS) , "recipes/building_blocks/polished_deepslate_stairs_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_deepslate_stairs_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/building_blocks/polished_diorite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE) , "recipes/building_blocks/polished_diorite_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB) , "recipes/building_blocks/polished_diorite_slab_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_slab_from_polished_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_POLISHED_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS) , "recipes/building_blocks/polished_diorite_stairs_from_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_diorite_stairs_from_polished_diorite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_POLISHED_DIORITE_STONECUTTING) , "recipes/building_blocks/polished_granite" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE) , "recipes/building_blocks/polished_granite_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB) , "recipes/building_blocks/polished_granite_slab_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_slab_from_polished_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_POLISHED_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS) , "recipes/building_blocks/polished_granite_stairs_from_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_granite_stairs_from_polished_granite_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_POLISHED_GRANITE_STONECUTTING) , "recipes/building_blocks/polished_tuff" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF) , "recipes/building_blocks/polished_tuff_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB) , "recipes/building_blocks/polished_tuff_slab_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_slab_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS) , "recipes/building_blocks/polished_tuff_stairs_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/polished_tuff_stairs_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/prismarine" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE) , "recipes/building_blocks/prismarine_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB) , "recipes/building_blocks/prismarine_brick_slab_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/prismarine_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS) , "recipes/building_blocks/prismarine_brick_stairs_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/prismarine_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICKS) , "recipes/building_blocks/prismarine_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB) , "recipes/building_blocks/prismarine_slab_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/prismarine_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS) , "recipes/building_blocks/prismarine_stairs_from_prismarine_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS_FROM_PRISMARINE_STONECUTTING) , "recipes/building_blocks/purple_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPLE_CONCRETE_POWDER) , "recipes/building_blocks/purple_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPLE_STAINED_GLASS) , "recipes/building_blocks/purple_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPLE_TERRACOTTA) , "recipes/building_blocks/purpur_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_BLOCK) , "recipes/building_blocks/purpur_pillar" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR) , "recipes/building_blocks/purpur_pillar_from_purpur_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR_FROM_PURPUR_BLOCK_STONECUTTING) , "recipes/building_blocks/purpur_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_SLAB) , "recipes/building_blocks/purpur_slab_from_purpur_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_SLAB_FROM_PURPUR_BLOCK_STONECUTTING) , "recipes/building_blocks/purpur_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS) , "recipes/building_blocks/purpur_stairs_from_purpur_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS_FROM_PURPUR_BLOCK_STONECUTTING) , "recipes/building_blocks/quartz_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_BLOCK) , "recipes/building_blocks/quartz_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS) , "recipes/building_blocks/quartz_bricks_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/quartz_pillar" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR) , "recipes/building_blocks/quartz_pillar_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/quartz_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB) , "recipes/building_blocks/quartz_slab_from_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB_FROM_STONECUTTING) , "recipes/building_blocks/quartz_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS) , "recipes/building_blocks/quartz_stairs_from_quartz_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS_FROM_QUARTZ_BLOCK_STONECUTTING) , "recipes/building_blocks/raw_copper_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RAW_COPPER_BLOCK) , "recipes/building_blocks/raw_gold_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RAW_GOLD_BLOCK) , "recipes/building_blocks/raw_iron_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RAW_IRON_BLOCK) , "recipes/building_blocks/red_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_CONCRETE_POWDER) , "recipes/building_blocks/red_nether_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB) , "recipes/building_blocks/red_nether_brick_slab_from_red_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB_FROM_RED_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/red_nether_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS) , "recipes/building_blocks/red_nether_brick_stairs_from_red_nether_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS_FROM_RED_NETHER_BRICKS_STONECUTTING) , "recipes/building_blocks/red_nether_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICKS) , "recipes/building_blocks/red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE) , "recipes/building_blocks/red_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB) , "recipes/building_blocks/red_sandstone_slab_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/red_sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS) , "recipes/building_blocks/red_sandstone_stairs_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/red_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_STAINED_GLASS) , "recipes/building_blocks/red_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RED_TERRACOTTA) , "recipes/building_blocks/resin_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BLOCK) , "recipes/building_blocks/resin_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB) , "recipes/building_blocks/resin_brick_slab_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/building_blocks/resin_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS) , "recipes/building_blocks/resin_brick_stairs_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/building_blocks/resin_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_RESIN_BRICKS) , "recipes/building_blocks/sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE) , "recipes/building_blocks/sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB) , "recipes/building_blocks/sandstone_slab_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS) , "recipes/building_blocks/sandstone_stairs_from_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS_FROM_SANDSTONE_STONECUTTING) , "recipes/building_blocks/sea_lantern" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SEA_LANTERN) , "recipes/building_blocks/smooth_basalt" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_BASALT) , "recipes/building_blocks/smooth_quartz" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ) , "recipes/building_blocks/smooth_quartz_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB) , "recipes/building_blocks/smooth_quartz_slab_from_smooth_quartz_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB_FROM_SMOOTH_QUARTZ_STONECUTTING) , "recipes/building_blocks/smooth_quartz_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS) , "recipes/building_blocks/smooth_quartz_stairs_from_smooth_quartz_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS_FROM_SMOOTH_QUARTZ_STONECUTTING) , "recipes/building_blocks/smooth_red_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE) , "recipes/building_blocks/smooth_red_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB) , "recipes/building_blocks/smooth_red_sandstone_slab_from_smooth_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_red_sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS) , "recipes/building_blocks/smooth_red_sandstone_stairs_from_smooth_red_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_sandstone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE) , "recipes/building_blocks/smooth_sandstone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB) , "recipes/building_blocks/smooth_sandstone_slab_from_smooth_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB_FROM_SMOOTH_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_sandstone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS) , "recipes/building_blocks/smooth_sandstone_stairs_from_smooth_sandstone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS_FROM_SMOOTH_SANDSTONE_STONECUTTING) , "recipes/building_blocks/smooth_stone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE) , "recipes/building_blocks/smooth_stone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB) , "recipes/building_blocks/smooth_stone_slab_from_smooth_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB_FROM_SMOOTH_STONE_STONECUTTING) , "recipes/building_blocks/snow_block" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SNOW_BLOCK) , "recipes/building_blocks/sponge" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPONGE) , "recipes/building_blocks/spruce_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_PLANKS) , "recipes/building_blocks/spruce_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_SLAB) , "recipes/building_blocks/spruce_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_STAIRS) , "recipes/building_blocks/spruce_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_SPRUCE_WOOD) , "recipes/building_blocks/stone" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE) , "recipes/building_blocks/stone_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB) , "recipes/building_blocks/stone_brick_slab_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_BRICKS_STONECUTTING) , "recipes/building_blocks/stone_brick_slab_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS) , "recipes/building_blocks/stone_brick_stairs_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_BRICKS_STONECUTTING) , "recipes/building_blocks/stone_brick_stairs_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICKS) , "recipes/building_blocks/stone_bricks_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_BRICKS_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_SLAB) , "recipes/building_blocks/stone_slab_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_SLAB_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stone_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_STAIRS) , "recipes/building_blocks/stone_stairs_from_stone_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STONE_STAIRS_FROM_STONE_STONECUTTING) , "recipes/building_blocks/stripped_acacia_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_ACACIA_WOOD) , "recipes/building_blocks/stripped_birch_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_BIRCH_WOOD) , "recipes/building_blocks/stripped_cherry_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_CHERRY_WOOD) , "recipes/building_blocks/stripped_crimson_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_CRIMSON_HYPHAE) , "recipes/building_blocks/stripped_dark_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_DARK_OAK_WOOD) , "recipes/building_blocks/stripped_jungle_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_JUNGLE_WOOD) , "recipes/building_blocks/stripped_mangrove_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_MANGROVE_WOOD) , "recipes/building_blocks/stripped_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_OAK_WOOD) , "recipes/building_blocks/stripped_pale_oak_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_PALE_OAK_WOOD) , "recipes/building_blocks/stripped_spruce_wood" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_SPRUCE_WOOD) , "recipes/building_blocks/stripped_warped_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_STRIPPED_WARPED_HYPHAE) , "recipes/building_blocks/terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TERRACOTTA) , "recipes/building_blocks/tinted_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TINTED_GLASS) , "recipes/building_blocks/tuff_brick_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB) , "recipes/building_blocks/tuff_brick_slab_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_brick_slab_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/building_blocks/tuff_brick_slab_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_brick_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS) , "recipes/building_blocks/tuff_brick_stairs_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_brick_stairs_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/building_blocks/tuff_brick_stairs_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_bricks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS) , "recipes/building_blocks/tuff_bricks_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_bricks_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_SLAB) , "recipes/building_blocks/tuff_slab_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_SLAB_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/tuff_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_STAIRS) , "recipes/building_blocks/tuff_stairs_from_tuff_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_TUFF_STAIRS_FROM_TUFF_STONECUTTING) , "recipes/building_blocks/warped_hyphae" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_HYPHAE) , "recipes/building_blocks/warped_planks" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_PLANKS) , "recipes/building_blocks/warped_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_SLAB) , "recipes/building_blocks/warped_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WARPED_STAIRS) , "recipes/building_blocks/waxed_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER) , "recipes/building_blocks/waxed_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_chiseled_copper_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_chiseled_copper_from_waxed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_block_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BLOCK_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE) , "recipes/building_blocks/waxed_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_copper_grate_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER) , "recipes/building_blocks/waxed_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_COPPER_BLOCK_STONECUTTING) , "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER) , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE) , "recipes/building_blocks/waxed_exposed_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_copper_grate_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER) , "recipes/building_blocks/waxed_exposed_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_exposed_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER) , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE) , "recipes/building_blocks/waxed_oxidized_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_copper_grate_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER) , "recipes/building_blocks/waxed_oxidized_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_oxidized_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER) , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_copper_bars_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_BARS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_chain_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHAIN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_chest_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHEST_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_golem_statue_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE) , "recipes/building_blocks/waxed_weathered_copper_grate_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_copper_grate_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_copper_lantern_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_LANTERN_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER) , "recipes/building_blocks/waxed_weathered_cut_copper_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB) , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_HONEYCOMB) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/waxed_weathered_lightning_rod_from_honeycomb" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_LIGHTNING_ROD_FROM_HONEYCOMB) , "recipes/building_blocks/weathered_chiseled_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER) , "recipes/building_blocks/weathered_chiseled_copper_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_chiseled_copper_from_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_copper_grate" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE) , "recipes/building_blocks/weathered_copper_grate_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER) , "recipes/building_blocks/weathered_cut_copper_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_slab" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB) , "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_stairs" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS) , "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_COPPER_STONECUTTING) , "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_cut_copper_stonecutting" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_CUT_COPPER_STONECUTTING) , "recipes/building_blocks/white_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_CONCRETE_POWDER) , "recipes/building_blocks/white_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_STAINED_GLASS) , "recipes/building_blocks/white_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_TERRACOTTA) , "recipes/building_blocks/white_wool_from_string" => Some (& Self :: RECIPES_BUILDING_BLOCKS_WHITE_WOOL_FROM_STRING) , "recipes/building_blocks/yellow_concrete_powder" => Some (& Self :: RECIPES_BUILDING_BLOCKS_YELLOW_CONCRETE_POWDER) , "recipes/building_blocks/yellow_stained_glass" => Some (& Self :: RECIPES_BUILDING_BLOCKS_YELLOW_STAINED_GLASS) , "recipes/building_blocks/yellow_terracotta" => Some (& Self :: RECIPES_BUILDING_BLOCKS_YELLOW_TERRACOTTA) , "recipes/combat/arrow" => Some (& Self :: RECIPES_COMBAT_ARROW) , "recipes/combat/black_harness" => Some (& Self :: RECIPES_COMBAT_BLACK_HARNESS) , "recipes/combat/blue_harness" => Some (& Self :: RECIPES_COMBAT_BLUE_HARNESS) , "recipes/combat/bow" => Some (& Self :: RECIPES_COMBAT_BOW) , "recipes/combat/brown_harness" => Some (& Self :: RECIPES_COMBAT_BROWN_HARNESS) , "recipes/combat/copper_boots" => Some (& Self :: RECIPES_COMBAT_COPPER_BOOTS) , "recipes/combat/copper_chestplate" => Some (& Self :: RECIPES_COMBAT_COPPER_CHESTPLATE) , "recipes/combat/copper_helmet" => Some (& Self :: RECIPES_COMBAT_COPPER_HELMET) , "recipes/combat/copper_leggings" => Some (& Self :: RECIPES_COMBAT_COPPER_LEGGINGS) , "recipes/combat/copper_spear" => Some (& Self :: RECIPES_COMBAT_COPPER_SPEAR) , "recipes/combat/copper_sword" => Some (& Self :: RECIPES_COMBAT_COPPER_SWORD) , "recipes/combat/crossbow" => Some (& Self :: RECIPES_COMBAT_CROSSBOW) , "recipes/combat/cyan_harness" => Some (& Self :: RECIPES_COMBAT_CYAN_HARNESS) , "recipes/combat/diamond_boots" => Some (& Self :: RECIPES_COMBAT_DIAMOND_BOOTS) , "recipes/combat/diamond_chestplate" => Some (& Self :: RECIPES_COMBAT_DIAMOND_CHESTPLATE) , "recipes/combat/diamond_helmet" => Some (& Self :: RECIPES_COMBAT_DIAMOND_HELMET) , "recipes/combat/diamond_leggings" => Some (& Self :: RECIPES_COMBAT_DIAMOND_LEGGINGS) , "recipes/combat/diamond_spear" => Some (& Self :: RECIPES_COMBAT_DIAMOND_SPEAR) , "recipes/combat/diamond_sword" => Some (& Self :: RECIPES_COMBAT_DIAMOND_SWORD) , "recipes/combat/dye_black_harness" => Some (& Self :: RECIPES_COMBAT_DYE_BLACK_HARNESS) , "recipes/combat/dye_blue_harness" => Some (& Self :: RECIPES_COMBAT_DYE_BLUE_HARNESS) , "recipes/combat/dye_brown_harness" => Some (& Self :: RECIPES_COMBAT_DYE_BROWN_HARNESS) , "recipes/combat/dye_cyan_harness" => Some (& Self :: RECIPES_COMBAT_DYE_CYAN_HARNESS) , "recipes/combat/dye_gray_harness" => Some (& Self :: RECIPES_COMBAT_DYE_GRAY_HARNESS) , "recipes/combat/dye_green_harness" => Some (& Self :: RECIPES_COMBAT_DYE_GREEN_HARNESS) , "recipes/combat/dye_light_blue_harness" => Some (& Self :: RECIPES_COMBAT_DYE_LIGHT_BLUE_HARNESS) , "recipes/combat/dye_light_gray_harness" => Some (& Self :: RECIPES_COMBAT_DYE_LIGHT_GRAY_HARNESS) , "recipes/combat/dye_lime_harness" => Some (& Self :: RECIPES_COMBAT_DYE_LIME_HARNESS) , "recipes/combat/dye_magenta_harness" => Some (& Self :: RECIPES_COMBAT_DYE_MAGENTA_HARNESS) , "recipes/combat/dye_orange_harness" => Some (& Self :: RECIPES_COMBAT_DYE_ORANGE_HARNESS) , "recipes/combat/dye_pink_harness" => Some (& Self :: RECIPES_COMBAT_DYE_PINK_HARNESS) , "recipes/combat/dye_purple_harness" => Some (& Self :: RECIPES_COMBAT_DYE_PURPLE_HARNESS) , "recipes/combat/dye_red_harness" => Some (& Self :: RECIPES_COMBAT_DYE_RED_HARNESS) , "recipes/combat/dye_white_harness" => Some (& Self :: RECIPES_COMBAT_DYE_WHITE_HARNESS) , "recipes/combat/dye_yellow_harness" => Some (& Self :: RECIPES_COMBAT_DYE_YELLOW_HARNESS) , "recipes/combat/golden_boots" => Some (& Self :: RECIPES_COMBAT_GOLDEN_BOOTS) , "recipes/combat/golden_chestplate" => Some (& Self :: RECIPES_COMBAT_GOLDEN_CHESTPLATE) , "recipes/combat/golden_helmet" => Some (& Self :: RECIPES_COMBAT_GOLDEN_HELMET) , "recipes/combat/golden_leggings" => Some (& Self :: RECIPES_COMBAT_GOLDEN_LEGGINGS) , "recipes/combat/golden_spear" => Some (& Self :: RECIPES_COMBAT_GOLDEN_SPEAR) , "recipes/combat/golden_sword" => Some (& Self :: RECIPES_COMBAT_GOLDEN_SWORD) , "recipes/combat/gray_harness" => Some (& Self :: RECIPES_COMBAT_GRAY_HARNESS) , "recipes/combat/green_harness" => Some (& Self :: RECIPES_COMBAT_GREEN_HARNESS) , "recipes/combat/iron_boots" => Some (& Self :: RECIPES_COMBAT_IRON_BOOTS) , "recipes/combat/iron_chestplate" => Some (& Self :: RECIPES_COMBAT_IRON_CHESTPLATE) , "recipes/combat/iron_helmet" => Some (& Self :: RECIPES_COMBAT_IRON_HELMET) , "recipes/combat/iron_leggings" => Some (& Self :: RECIPES_COMBAT_IRON_LEGGINGS) , "recipes/combat/iron_spear" => Some (& Self :: RECIPES_COMBAT_IRON_SPEAR) , "recipes/combat/iron_sword" => Some (& Self :: RECIPES_COMBAT_IRON_SWORD) , "recipes/combat/leather_boots" => Some (& Self :: RECIPES_COMBAT_LEATHER_BOOTS) , "recipes/combat/leather_chestplate" => Some (& Self :: RECIPES_COMBAT_LEATHER_CHESTPLATE) , "recipes/combat/leather_helmet" => Some (& Self :: RECIPES_COMBAT_LEATHER_HELMET) , "recipes/combat/leather_leggings" => Some (& Self :: RECIPES_COMBAT_LEATHER_LEGGINGS) , "recipes/combat/light_blue_harness" => Some (& Self :: RECIPES_COMBAT_LIGHT_BLUE_HARNESS) , "recipes/combat/light_gray_harness" => Some (& Self :: RECIPES_COMBAT_LIGHT_GRAY_HARNESS) , "recipes/combat/lime_harness" => Some (& Self :: RECIPES_COMBAT_LIME_HARNESS) , "recipes/combat/mace" => Some (& Self :: RECIPES_COMBAT_MACE) , "recipes/combat/magenta_harness" => Some (& Self :: RECIPES_COMBAT_MAGENTA_HARNESS) , "recipes/combat/netherite_boots_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_BOOTS_SMITHING) , "recipes/combat/netherite_chestplate_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_CHESTPLATE_SMITHING) , "recipes/combat/netherite_helmet_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_HELMET_SMITHING) , "recipes/combat/netherite_horse_armor_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_HORSE_ARMOR_SMITHING) , "recipes/combat/netherite_leggings_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_LEGGINGS_SMITHING) , "recipes/combat/netherite_nautilus_armor_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_NAUTILUS_ARMOR_SMITHING) , "recipes/combat/netherite_spear_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_SPEAR_SMITHING) , "recipes/combat/netherite_sword_smithing" => Some (& Self :: RECIPES_COMBAT_NETHERITE_SWORD_SMITHING) , "recipes/combat/orange_harness" => Some (& Self :: RECIPES_COMBAT_ORANGE_HARNESS) , "recipes/combat/pink_harness" => Some (& Self :: RECIPES_COMBAT_PINK_HARNESS) , "recipes/combat/purple_harness" => Some (& Self :: RECIPES_COMBAT_PURPLE_HARNESS) , "recipes/combat/red_harness" => Some (& Self :: RECIPES_COMBAT_RED_HARNESS) , "recipes/combat/saddle" => Some (& Self :: RECIPES_COMBAT_SADDLE) , "recipes/combat/shield" => Some (& Self :: RECIPES_COMBAT_SHIELD) , "recipes/combat/spectral_arrow" => Some (& Self :: RECIPES_COMBAT_SPECTRAL_ARROW) , "recipes/combat/stone_spear" => Some (& Self :: RECIPES_COMBAT_STONE_SPEAR) , "recipes/combat/stone_sword" => Some (& Self :: RECIPES_COMBAT_STONE_SWORD) , "recipes/combat/turtle_helmet" => Some (& Self :: RECIPES_COMBAT_TURTLE_HELMET) , "recipes/combat/white_harness" => Some (& Self :: RECIPES_COMBAT_WHITE_HARNESS) , "recipes/combat/wolf_armor" => Some (& Self :: RECIPES_COMBAT_WOLF_ARMOR) , "recipes/combat/wooden_spear" => Some (& Self :: RECIPES_COMBAT_WOODEN_SPEAR) , "recipes/combat/wooden_sword" => Some (& Self :: RECIPES_COMBAT_WOODEN_SWORD) , "recipes/combat/yellow_harness" => Some (& Self :: RECIPES_COMBAT_YELLOW_HARNESS) , "recipes/decorations/acacia_fence" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_FENCE) , "recipes/decorations/acacia_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_HANGING_SIGN) , "recipes/decorations/acacia_shelf" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_SHELF) , "recipes/decorations/acacia_sign" => Some (& Self :: RECIPES_DECORATIONS_ACACIA_SIGN) , "recipes/decorations/andesite_wall" => Some (& Self :: RECIPES_DECORATIONS_ANDESITE_WALL) , "recipes/decorations/andesite_wall_from_andesite_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_ANDESITE_WALL_FROM_ANDESITE_STONECUTTING) , "recipes/decorations/anvil" => Some (& Self :: RECIPES_DECORATIONS_ANVIL) , "recipes/decorations/armor_stand" => Some (& Self :: RECIPES_DECORATIONS_ARMOR_STAND) , "recipes/decorations/bamboo_fence" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_FENCE) , "recipes/decorations/bamboo_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_HANGING_SIGN) , "recipes/decorations/bamboo_mosaic" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_MOSAIC) , "recipes/decorations/bamboo_shelf" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_SHELF) , "recipes/decorations/bamboo_sign" => Some (& Self :: RECIPES_DECORATIONS_BAMBOO_SIGN) , "recipes/decorations/barrel" => Some (& Self :: RECIPES_DECORATIONS_BARREL) , "recipes/decorations/beehive" => Some (& Self :: RECIPES_DECORATIONS_BEEHIVE) , "recipes/decorations/birch_fence" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_FENCE) , "recipes/decorations/birch_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_HANGING_SIGN) , "recipes/decorations/birch_shelf" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_SHELF) , "recipes/decorations/birch_sign" => Some (& Self :: RECIPES_DECORATIONS_BIRCH_SIGN) , "recipes/decorations/black_banner" => Some (& Self :: RECIPES_DECORATIONS_BLACK_BANNER) , "recipes/decorations/black_bed" => Some (& Self :: RECIPES_DECORATIONS_BLACK_BED) , "recipes/decorations/black_candle" => Some (& Self :: RECIPES_DECORATIONS_BLACK_CANDLE) , "recipes/decorations/black_carpet" => Some (& Self :: RECIPES_DECORATIONS_BLACK_CARPET) , "recipes/decorations/black_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_BLACK_GLAZED_TERRACOTTA) , "recipes/decorations/black_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_BLACK_SHULKER_BOX) , "recipes/decorations/black_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE) , "recipes/decorations/black_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/blackstone_wall" => Some (& Self :: RECIPES_DECORATIONS_BLACKSTONE_WALL) , "recipes/decorations/blackstone_wall_from_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING) , "recipes/decorations/blast_furnace" => Some (& Self :: RECIPES_DECORATIONS_BLAST_FURNACE) , "recipes/decorations/blue_banner" => Some (& Self :: RECIPES_DECORATIONS_BLUE_BANNER) , "recipes/decorations/blue_bed" => Some (& Self :: RECIPES_DECORATIONS_BLUE_BED) , "recipes/decorations/blue_candle" => Some (& Self :: RECIPES_DECORATIONS_BLUE_CANDLE) , "recipes/decorations/blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_BLUE_CARPET) , "recipes/decorations/blue_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_BLUE_GLAZED_TERRACOTTA) , "recipes/decorations/blue_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_BLUE_SHULKER_BOX) , "recipes/decorations/blue_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE) , "recipes/decorations/blue_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/brick_wall" => Some (& Self :: RECIPES_DECORATIONS_BRICK_WALL) , "recipes/decorations/brick_wall_from_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_BRICK_WALL_FROM_BRICKS_STONECUTTING) , "recipes/decorations/brown_banner" => Some (& Self :: RECIPES_DECORATIONS_BROWN_BANNER) , "recipes/decorations/brown_bed" => Some (& Self :: RECIPES_DECORATIONS_BROWN_BED) , "recipes/decorations/brown_candle" => Some (& Self :: RECIPES_DECORATIONS_BROWN_CANDLE) , "recipes/decorations/brown_carpet" => Some (& Self :: RECIPES_DECORATIONS_BROWN_CARPET) , "recipes/decorations/brown_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_BROWN_GLAZED_TERRACOTTA) , "recipes/decorations/brown_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_BROWN_SHULKER_BOX) , "recipes/decorations/brown_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE) , "recipes/decorations/brown_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/campfire" => Some (& Self :: RECIPES_DECORATIONS_CAMPFIRE) , "recipes/decorations/candle" => Some (& Self :: RECIPES_DECORATIONS_CANDLE) , "recipes/decorations/cartography_table" => Some (& Self :: RECIPES_DECORATIONS_CARTOGRAPHY_TABLE) , "recipes/decorations/cherry_fence" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_FENCE) , "recipes/decorations/cherry_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_HANGING_SIGN) , "recipes/decorations/cherry_shelf" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_SHELF) , "recipes/decorations/cherry_sign" => Some (& Self :: RECIPES_DECORATIONS_CHERRY_SIGN) , "recipes/decorations/chest" => Some (& Self :: RECIPES_DECORATIONS_CHEST) , "recipes/decorations/cobbled_deepslate_wall" => Some (& Self :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL) , "recipes/decorations/cobbled_deepslate_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/cobblestone_wall" => Some (& Self :: RECIPES_DECORATIONS_COBBLESTONE_WALL) , "recipes/decorations/cobblestone_wall_from_cobblestone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_COBBLESTONE_WALL_FROM_COBBLESTONE_STONECUTTING) , "recipes/decorations/composter" => Some (& Self :: RECIPES_DECORATIONS_COMPOSTER) , "recipes/decorations/copper_bars" => Some (& Self :: RECIPES_DECORATIONS_COPPER_BARS) , "recipes/decorations/copper_chain" => Some (& Self :: RECIPES_DECORATIONS_COPPER_CHAIN) , "recipes/decorations/copper_chest" => Some (& Self :: RECIPES_DECORATIONS_COPPER_CHEST) , "recipes/decorations/copper_lantern" => Some (& Self :: RECIPES_DECORATIONS_COPPER_LANTERN) , "recipes/decorations/copper_torch" => Some (& Self :: RECIPES_DECORATIONS_COPPER_TORCH) , "recipes/decorations/crafting_table" => Some (& Self :: RECIPES_DECORATIONS_CRAFTING_TABLE) , "recipes/decorations/crimson_fence" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_FENCE) , "recipes/decorations/crimson_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_HANGING_SIGN) , "recipes/decorations/crimson_shelf" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_SHELF) , "recipes/decorations/crimson_sign" => Some (& Self :: RECIPES_DECORATIONS_CRIMSON_SIGN) , "recipes/decorations/cyan_banner" => Some (& Self :: RECIPES_DECORATIONS_CYAN_BANNER) , "recipes/decorations/cyan_bed" => Some (& Self :: RECIPES_DECORATIONS_CYAN_BED) , "recipes/decorations/cyan_candle" => Some (& Self :: RECIPES_DECORATIONS_CYAN_CANDLE) , "recipes/decorations/cyan_carpet" => Some (& Self :: RECIPES_DECORATIONS_CYAN_CARPET) , "recipes/decorations/cyan_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_CYAN_GLAZED_TERRACOTTA) , "recipes/decorations/cyan_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_CYAN_SHULKER_BOX) , "recipes/decorations/cyan_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE) , "recipes/decorations/cyan_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/dark_oak_fence" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_FENCE) , "recipes/decorations/dark_oak_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_HANGING_SIGN) , "recipes/decorations/dark_oak_shelf" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_SHELF) , "recipes/decorations/dark_oak_sign" => Some (& Self :: RECIPES_DECORATIONS_DARK_OAK_SIGN) , "recipes/decorations/decorated_pot_simple" => Some (& Self :: RECIPES_DECORATIONS_DECORATED_POT_SIMPLE) , "recipes/decorations/deepslate_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL) , "recipes/decorations/deepslate_brick_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/deepslate_brick_wall_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/decorations/deepslate_brick_wall_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/decorations/deepslate_tile_wall" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL) , "recipes/decorations/deepslate_tile_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/deepslate_tile_wall_from_deepslate_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING) , "recipes/decorations/deepslate_tile_wall_from_deepslate_tiles_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_TILES_STONECUTTING) , "recipes/decorations/deepslate_tile_wall_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/decorations/diorite_wall" => Some (& Self :: RECIPES_DECORATIONS_DIORITE_WALL) , "recipes/decorations/diorite_wall_from_diorite_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_DIORITE_WALL_FROM_DIORITE_STONECUTTING) , "recipes/decorations/dye_black_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLACK_BED) , "recipes/decorations/dye_black_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLACK_CARPET) , "recipes/decorations/dye_blue_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLUE_BED) , "recipes/decorations/dye_blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_BLUE_CARPET) , "recipes/decorations/dye_brown_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_BROWN_BED) , "recipes/decorations/dye_brown_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_BROWN_CARPET) , "recipes/decorations/dye_cyan_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_CYAN_BED) , "recipes/decorations/dye_cyan_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_CYAN_CARPET) , "recipes/decorations/dye_gray_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_GRAY_BED) , "recipes/decorations/dye_gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_GRAY_CARPET) , "recipes/decorations/dye_green_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_GREEN_BED) , "recipes/decorations/dye_green_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_GREEN_CARPET) , "recipes/decorations/dye_light_blue_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_BLUE_BED) , "recipes/decorations/dye_light_blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_BLUE_CARPET) , "recipes/decorations/dye_light_gray_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_GRAY_BED) , "recipes/decorations/dye_light_gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIGHT_GRAY_CARPET) , "recipes/decorations/dye_lime_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIME_BED) , "recipes/decorations/dye_lime_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_LIME_CARPET) , "recipes/decorations/dye_magenta_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_MAGENTA_BED) , "recipes/decorations/dye_magenta_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_MAGENTA_CARPET) , "recipes/decorations/dye_orange_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_ORANGE_BED) , "recipes/decorations/dye_orange_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_ORANGE_CARPET) , "recipes/decorations/dye_pink_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_PINK_BED) , "recipes/decorations/dye_pink_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_PINK_CARPET) , "recipes/decorations/dye_purple_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_PURPLE_BED) , "recipes/decorations/dye_purple_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_PURPLE_CARPET) , "recipes/decorations/dye_red_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_RED_BED) , "recipes/decorations/dye_red_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_RED_CARPET) , "recipes/decorations/dye_white_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_WHITE_BED) , "recipes/decorations/dye_white_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_WHITE_CARPET) , "recipes/decorations/dye_yellow_bed" => Some (& Self :: RECIPES_DECORATIONS_DYE_YELLOW_BED) , "recipes/decorations/dye_yellow_carpet" => Some (& Self :: RECIPES_DECORATIONS_DYE_YELLOW_CARPET) , "recipes/decorations/enchanting_table" => Some (& Self :: RECIPES_DECORATIONS_ENCHANTING_TABLE) , "recipes/decorations/end_crystal" => Some (& Self :: RECIPES_DECORATIONS_END_CRYSTAL) , "recipes/decorations/end_rod" => Some (& Self :: RECIPES_DECORATIONS_END_ROD) , "recipes/decorations/end_stone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL) , "recipes/decorations/end_stone_brick_wall_from_end_stone_brick_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_BRICK_STONECUTTING) , "recipes/decorations/end_stone_brick_wall_from_end_stone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_STONECUTTING) , "recipes/decorations/ender_chest" => Some (& Self :: RECIPES_DECORATIONS_ENDER_CHEST) , "recipes/decorations/fletching_table" => Some (& Self :: RECIPES_DECORATIONS_FLETCHING_TABLE) , "recipes/decorations/flower_pot" => Some (& Self :: RECIPES_DECORATIONS_FLOWER_POT) , "recipes/decorations/furnace" => Some (& Self :: RECIPES_DECORATIONS_FURNACE) , "recipes/decorations/glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GLASS_PANE) , "recipes/decorations/glow_item_frame" => Some (& Self :: RECIPES_DECORATIONS_GLOW_ITEM_FRAME) , "recipes/decorations/granite_wall" => Some (& Self :: RECIPES_DECORATIONS_GRANITE_WALL) , "recipes/decorations/granite_wall_from_granite_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_GRANITE_WALL_FROM_GRANITE_STONECUTTING) , "recipes/decorations/gray_banner" => Some (& Self :: RECIPES_DECORATIONS_GRAY_BANNER) , "recipes/decorations/gray_bed" => Some (& Self :: RECIPES_DECORATIONS_GRAY_BED) , "recipes/decorations/gray_candle" => Some (& Self :: RECIPES_DECORATIONS_GRAY_CANDLE) , "recipes/decorations/gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_GRAY_CARPET) , "recipes/decorations/gray_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_GRAY_GLAZED_TERRACOTTA) , "recipes/decorations/gray_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_GRAY_SHULKER_BOX) , "recipes/decorations/gray_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE) , "recipes/decorations/gray_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/green_banner" => Some (& Self :: RECIPES_DECORATIONS_GREEN_BANNER) , "recipes/decorations/green_bed" => Some (& Self :: RECIPES_DECORATIONS_GREEN_BED) , "recipes/decorations/green_candle" => Some (& Self :: RECIPES_DECORATIONS_GREEN_CANDLE) , "recipes/decorations/green_carpet" => Some (& Self :: RECIPES_DECORATIONS_GREEN_CARPET) , "recipes/decorations/green_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_GREEN_GLAZED_TERRACOTTA) , "recipes/decorations/green_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_GREEN_SHULKER_BOX) , "recipes/decorations/green_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE) , "recipes/decorations/green_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/grindstone" => Some (& Self :: RECIPES_DECORATIONS_GRINDSTONE) , "recipes/decorations/honeycomb_block" => Some (& Self :: RECIPES_DECORATIONS_HONEYCOMB_BLOCK) , "recipes/decorations/iron_bars" => Some (& Self :: RECIPES_DECORATIONS_IRON_BARS) , "recipes/decorations/iron_chain" => Some (& Self :: RECIPES_DECORATIONS_IRON_CHAIN) , "recipes/decorations/item_frame" => Some (& Self :: RECIPES_DECORATIONS_ITEM_FRAME) , "recipes/decorations/jukebox" => Some (& Self :: RECIPES_DECORATIONS_JUKEBOX) , "recipes/decorations/jungle_fence" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_FENCE) , "recipes/decorations/jungle_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_HANGING_SIGN) , "recipes/decorations/jungle_shelf" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_SHELF) , "recipes/decorations/jungle_sign" => Some (& Self :: RECIPES_DECORATIONS_JUNGLE_SIGN) , "recipes/decorations/ladder" => Some (& Self :: RECIPES_DECORATIONS_LADDER) , "recipes/decorations/lantern" => Some (& Self :: RECIPES_DECORATIONS_LANTERN) , "recipes/decorations/light_blue_banner" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_BANNER) , "recipes/decorations/light_blue_bed" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_BED) , "recipes/decorations/light_blue_candle" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_CANDLE) , "recipes/decorations/light_blue_carpet" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_CARPET) , "recipes/decorations/light_blue_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_GLAZED_TERRACOTTA) , "recipes/decorations/light_blue_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_SHULKER_BOX) , "recipes/decorations/light_blue_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE) , "recipes/decorations/light_blue_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/light_gray_banner" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_BANNER) , "recipes/decorations/light_gray_bed" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_BED) , "recipes/decorations/light_gray_candle" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_CANDLE) , "recipes/decorations/light_gray_carpet" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_CARPET) , "recipes/decorations/light_gray_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_GLAZED_TERRACOTTA) , "recipes/decorations/light_gray_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_SHULKER_BOX) , "recipes/decorations/light_gray_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE) , "recipes/decorations/light_gray_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/lime_banner" => Some (& Self :: RECIPES_DECORATIONS_LIME_BANNER) , "recipes/decorations/lime_bed" => Some (& Self :: RECIPES_DECORATIONS_LIME_BED) , "recipes/decorations/lime_candle" => Some (& Self :: RECIPES_DECORATIONS_LIME_CANDLE) , "recipes/decorations/lime_carpet" => Some (& Self :: RECIPES_DECORATIONS_LIME_CARPET) , "recipes/decorations/lime_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_LIME_GLAZED_TERRACOTTA) , "recipes/decorations/lime_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_LIME_SHULKER_BOX) , "recipes/decorations/lime_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE) , "recipes/decorations/lime_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/lodestone" => Some (& Self :: RECIPES_DECORATIONS_LODESTONE) , "recipes/decorations/loom" => Some (& Self :: RECIPES_DECORATIONS_LOOM) , "recipes/decorations/magenta_banner" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_BANNER) , "recipes/decorations/magenta_bed" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_BED) , "recipes/decorations/magenta_candle" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_CANDLE) , "recipes/decorations/magenta_carpet" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_CARPET) , "recipes/decorations/magenta_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_GLAZED_TERRACOTTA) , "recipes/decorations/magenta_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_SHULKER_BOX) , "recipes/decorations/magenta_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE) , "recipes/decorations/magenta_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/mangrove_fence" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_FENCE) , "recipes/decorations/mangrove_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_HANGING_SIGN) , "recipes/decorations/mangrove_shelf" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_SHELF) , "recipes/decorations/mangrove_sign" => Some (& Self :: RECIPES_DECORATIONS_MANGROVE_SIGN) , "recipes/decorations/moss_carpet" => Some (& Self :: RECIPES_DECORATIONS_MOSS_CARPET) , "recipes/decorations/mossy_cobblestone_wall" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL) , "recipes/decorations/mossy_cobblestone_wall_from_mossy_cobblestone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL_FROM_MOSSY_COBBLESTONE_STONECUTTING) , "recipes/decorations/mossy_stone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL) , "recipes/decorations/mossy_stone_brick_wall_from_mossy_stone_brick_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL_FROM_MOSSY_STONE_BRICK_STONECUTTING) , "recipes/decorations/mud_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_MUD_BRICK_WALL) , "recipes/decorations/mud_brick_wall_from_mud_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_MUD_BRICK_WALL_FROM_MUD_BRICKS_STONECUTTING) , "recipes/decorations/nether_brick_fence" => Some (& Self :: RECIPES_DECORATIONS_NETHER_BRICK_FENCE) , "recipes/decorations/nether_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_NETHER_BRICK_WALL) , "recipes/decorations/nether_brick_wall_from_nether_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_NETHER_BRICK_WALL_FROM_NETHER_BRICKS_STONECUTTING) , "recipes/decorations/oak_fence" => Some (& Self :: RECIPES_DECORATIONS_OAK_FENCE) , "recipes/decorations/oak_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_OAK_HANGING_SIGN) , "recipes/decorations/oak_shelf" => Some (& Self :: RECIPES_DECORATIONS_OAK_SHELF) , "recipes/decorations/oak_sign" => Some (& Self :: RECIPES_DECORATIONS_OAK_SIGN) , "recipes/decorations/orange_banner" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_BANNER) , "recipes/decorations/orange_bed" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_BED) , "recipes/decorations/orange_candle" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_CANDLE) , "recipes/decorations/orange_carpet" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_CARPET) , "recipes/decorations/orange_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_GLAZED_TERRACOTTA) , "recipes/decorations/orange_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_SHULKER_BOX) , "recipes/decorations/orange_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE) , "recipes/decorations/orange_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/painting" => Some (& Self :: RECIPES_DECORATIONS_PAINTING) , "recipes/decorations/pale_moss_carpet" => Some (& Self :: RECIPES_DECORATIONS_PALE_MOSS_CARPET) , "recipes/decorations/pale_oak_fence" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_FENCE) , "recipes/decorations/pale_oak_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_HANGING_SIGN) , "recipes/decorations/pale_oak_shelf" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_SHELF) , "recipes/decorations/pale_oak_sign" => Some (& Self :: RECIPES_DECORATIONS_PALE_OAK_SIGN) , "recipes/decorations/pink_banner" => Some (& Self :: RECIPES_DECORATIONS_PINK_BANNER) , "recipes/decorations/pink_bed" => Some (& Self :: RECIPES_DECORATIONS_PINK_BED) , "recipes/decorations/pink_candle" => Some (& Self :: RECIPES_DECORATIONS_PINK_CANDLE) , "recipes/decorations/pink_carpet" => Some (& Self :: RECIPES_DECORATIONS_PINK_CARPET) , "recipes/decorations/pink_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_PINK_GLAZED_TERRACOTTA) , "recipes/decorations/pink_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_PINK_SHULKER_BOX) , "recipes/decorations/pink_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE) , "recipes/decorations/pink_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/polished_blackstone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL) , "recipes/decorations/polished_blackstone_brick_wall_from_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING) , "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_blackstone_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL) , "recipes/decorations/polished_blackstone_wall_from_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_blackstone_wall_from_polished_blackstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING) , "recipes/decorations/polished_deepslate_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL) , "recipes/decorations/polished_deepslate_wall_from_cobbled_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING) , "recipes/decorations/polished_deepslate_wall_from_polished_deepslate_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING) , "recipes/decorations/polished_tuff_wall" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL) , "recipes/decorations/polished_tuff_wall_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/decorations/polished_tuff_wall_from_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_TUFF_STONECUTTING) , "recipes/decorations/prismarine_wall" => Some (& Self :: RECIPES_DECORATIONS_PRISMARINE_WALL) , "recipes/decorations/prismarine_wall_from_prismarine_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_PRISMARINE_WALL_FROM_PRISMARINE_STONECUTTING) , "recipes/decorations/purple_banner" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_BANNER) , "recipes/decorations/purple_bed" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_BED) , "recipes/decorations/purple_candle" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_CANDLE) , "recipes/decorations/purple_carpet" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_CARPET) , "recipes/decorations/purple_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_GLAZED_TERRACOTTA) , "recipes/decorations/purple_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_SHULKER_BOX) , "recipes/decorations/purple_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE) , "recipes/decorations/purple_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/red_banner" => Some (& Self :: RECIPES_DECORATIONS_RED_BANNER) , "recipes/decorations/red_bed" => Some (& Self :: RECIPES_DECORATIONS_RED_BED) , "recipes/decorations/red_candle" => Some (& Self :: RECIPES_DECORATIONS_RED_CANDLE) , "recipes/decorations/red_carpet" => Some (& Self :: RECIPES_DECORATIONS_RED_CARPET) , "recipes/decorations/red_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_RED_GLAZED_TERRACOTTA) , "recipes/decorations/red_nether_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL) , "recipes/decorations/red_nether_brick_wall_from_red_nether_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL_FROM_RED_NETHER_BRICKS_STONECUTTING) , "recipes/decorations/red_sandstone_wall" => Some (& Self :: RECIPES_DECORATIONS_RED_SANDSTONE_WALL) , "recipes/decorations/red_sandstone_wall_from_red_sandstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_RED_SANDSTONE_WALL_FROM_RED_SANDSTONE_STONECUTTING) , "recipes/decorations/red_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_RED_SHULKER_BOX) , "recipes/decorations/red_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE) , "recipes/decorations/red_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/resin_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_RESIN_BRICK_WALL) , "recipes/decorations/resin_brick_wall_from_resin_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_RESIN_BRICK_WALL_FROM_RESIN_BRICKS_STONECUTTING) , "recipes/decorations/respawn_anchor" => Some (& Self :: RECIPES_DECORATIONS_RESPAWN_ANCHOR) , "recipes/decorations/sandstone_wall" => Some (& Self :: RECIPES_DECORATIONS_SANDSTONE_WALL) , "recipes/decorations/sandstone_wall_from_sandstone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_SANDSTONE_WALL_FROM_SANDSTONE_STONECUTTING) , "recipes/decorations/scaffolding" => Some (& Self :: RECIPES_DECORATIONS_SCAFFOLDING) , "recipes/decorations/shulker_box" => Some (& Self :: RECIPES_DECORATIONS_SHULKER_BOX) , "recipes/decorations/smithing_table" => Some (& Self :: RECIPES_DECORATIONS_SMITHING_TABLE) , "recipes/decorations/smoker" => Some (& Self :: RECIPES_DECORATIONS_SMOKER) , "recipes/decorations/snow" => Some (& Self :: RECIPES_DECORATIONS_SNOW) , "recipes/decorations/soul_campfire" => Some (& Self :: RECIPES_DECORATIONS_SOUL_CAMPFIRE) , "recipes/decorations/soul_lantern" => Some (& Self :: RECIPES_DECORATIONS_SOUL_LANTERN) , "recipes/decorations/soul_torch" => Some (& Self :: RECIPES_DECORATIONS_SOUL_TORCH) , "recipes/decorations/spruce_fence" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_FENCE) , "recipes/decorations/spruce_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_HANGING_SIGN) , "recipes/decorations/spruce_shelf" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_SHELF) , "recipes/decorations/spruce_sign" => Some (& Self :: RECIPES_DECORATIONS_SPRUCE_SIGN) , "recipes/decorations/stone_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_STONE_BRICK_WALL) , "recipes/decorations/stone_brick_wall_from_stone_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_STONE_BRICK_WALL_FROM_STONE_BRICKS_STONECUTTING) , "recipes/decorations/stone_brick_walls_from_stone_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_STONE_BRICK_WALLS_FROM_STONE_STONECUTTING) , "recipes/decorations/stonecutter" => Some (& Self :: RECIPES_DECORATIONS_STONECUTTER) , "recipes/decorations/torch" => Some (& Self :: RECIPES_DECORATIONS_TORCH) , "recipes/decorations/tuff_brick_wall" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL) , "recipes/decorations/tuff_brick_wall_from_polished_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_POLISHED_TUFF_STONECUTTING) , "recipes/decorations/tuff_brick_wall_from_tuff_bricks_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_BRICKS_STONECUTTING) , "recipes/decorations/tuff_brick_wall_from_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_STONECUTTING) , "recipes/decorations/tuff_wall" => Some (& Self :: RECIPES_DECORATIONS_TUFF_WALL) , "recipes/decorations/tuff_wall_from_tuff_stonecutting" => Some (& Self :: RECIPES_DECORATIONS_TUFF_WALL_FROM_TUFF_STONECUTTING) , "recipes/decorations/warped_fence" => Some (& Self :: RECIPES_DECORATIONS_WARPED_FENCE) , "recipes/decorations/warped_hanging_sign" => Some (& Self :: RECIPES_DECORATIONS_WARPED_HANGING_SIGN) , "recipes/decorations/warped_shelf" => Some (& Self :: RECIPES_DECORATIONS_WARPED_SHELF) , "recipes/decorations/warped_sign" => Some (& Self :: RECIPES_DECORATIONS_WARPED_SIGN) , "recipes/decorations/white_banner" => Some (& Self :: RECIPES_DECORATIONS_WHITE_BANNER) , "recipes/decorations/white_bed" => Some (& Self :: RECIPES_DECORATIONS_WHITE_BED) , "recipes/decorations/white_candle" => Some (& Self :: RECIPES_DECORATIONS_WHITE_CANDLE) , "recipes/decorations/white_carpet" => Some (& Self :: RECIPES_DECORATIONS_WHITE_CARPET) , "recipes/decorations/white_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_WHITE_GLAZED_TERRACOTTA) , "recipes/decorations/white_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_WHITE_SHULKER_BOX) , "recipes/decorations/white_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE) , "recipes/decorations/white_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/decorations/yellow_banner" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_BANNER) , "recipes/decorations/yellow_bed" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_BED) , "recipes/decorations/yellow_candle" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_CANDLE) , "recipes/decorations/yellow_carpet" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_CARPET) , "recipes/decorations/yellow_glazed_terracotta" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_GLAZED_TERRACOTTA) , "recipes/decorations/yellow_shulker_box" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_SHULKER_BOX) , "recipes/decorations/yellow_stained_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE) , "recipes/decorations/yellow_stained_glass_pane_from_glass_pane" => Some (& Self :: RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE_FROM_GLASS_PANE) , "recipes/food/baked_potato" => Some (& Self :: RECIPES_FOOD_BAKED_POTATO) , "recipes/food/baked_potato_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_BAKED_POTATO_FROM_CAMPFIRE_COOKING) , "recipes/food/baked_potato_from_smoking" => Some (& Self :: RECIPES_FOOD_BAKED_POTATO_FROM_SMOKING) , "recipes/food/beetroot_soup" => Some (& Self :: RECIPES_FOOD_BEETROOT_SOUP) , "recipes/food/bread" => Some (& Self :: RECIPES_FOOD_BREAD) , "recipes/food/cake" => Some (& Self :: RECIPES_FOOD_CAKE) , "recipes/food/cooked_beef" => Some (& Self :: RECIPES_FOOD_COOKED_BEEF) , "recipes/food/cooked_beef_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_BEEF_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_beef_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_BEEF_FROM_SMOKING) , "recipes/food/cooked_chicken" => Some (& Self :: RECIPES_FOOD_COOKED_CHICKEN) , "recipes/food/cooked_chicken_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_CHICKEN_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_chicken_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_CHICKEN_FROM_SMOKING) , "recipes/food/cooked_cod" => Some (& Self :: RECIPES_FOOD_COOKED_COD) , "recipes/food/cooked_cod_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_COD_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_cod_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_COD_FROM_SMOKING) , "recipes/food/cooked_mutton" => Some (& Self :: RECIPES_FOOD_COOKED_MUTTON) , "recipes/food/cooked_mutton_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_MUTTON_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_mutton_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_MUTTON_FROM_SMOKING) , "recipes/food/cooked_porkchop" => Some (& Self :: RECIPES_FOOD_COOKED_PORKCHOP) , "recipes/food/cooked_porkchop_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_PORKCHOP_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_porkchop_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_PORKCHOP_FROM_SMOKING) , "recipes/food/cooked_rabbit" => Some (& Self :: RECIPES_FOOD_COOKED_RABBIT) , "recipes/food/cooked_rabbit_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_RABBIT_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_rabbit_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_RABBIT_FROM_SMOKING) , "recipes/food/cooked_salmon" => Some (& Self :: RECIPES_FOOD_COOKED_SALMON) , "recipes/food/cooked_salmon_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_COOKED_SALMON_FROM_CAMPFIRE_COOKING) , "recipes/food/cooked_salmon_from_smoking" => Some (& Self :: RECIPES_FOOD_COOKED_SALMON_FROM_SMOKING) , "recipes/food/cookie" => Some (& Self :: RECIPES_FOOD_COOKIE) , "recipes/food/dried_kelp" => Some (& Self :: RECIPES_FOOD_DRIED_KELP) , "recipes/food/dried_kelp_from_campfire_cooking" => Some (& Self :: RECIPES_FOOD_DRIED_KELP_FROM_CAMPFIRE_COOKING) , "recipes/food/dried_kelp_from_smelting" => Some (& Self :: RECIPES_FOOD_DRIED_KELP_FROM_SMELTING) , "recipes/food/dried_kelp_from_smoking" => Some (& Self :: RECIPES_FOOD_DRIED_KELP_FROM_SMOKING) , "recipes/food/golden_apple" => Some (& Self :: RECIPES_FOOD_GOLDEN_APPLE) , "recipes/food/honey_bottle" => Some (& Self :: RECIPES_FOOD_HONEY_BOTTLE) , "recipes/food/mushroom_stew" => Some (& Self :: RECIPES_FOOD_MUSHROOM_STEW) , "recipes/food/pumpkin_pie" => Some (& Self :: RECIPES_FOOD_PUMPKIN_PIE) , "recipes/food/rabbit_stew_from_brown_mushroom" => Some (& Self :: RECIPES_FOOD_RABBIT_STEW_FROM_BROWN_MUSHROOM) , "recipes/food/rabbit_stew_from_red_mushroom" => Some (& Self :: RECIPES_FOOD_RABBIT_STEW_FROM_RED_MUSHROOM) , "recipes/food/suspicious_stew_from_allium" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ALLIUM) , "recipes/food/suspicious_stew_from_azure_bluet" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_AZURE_BLUET) , "recipes/food/suspicious_stew_from_blue_orchid" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_BLUE_ORCHID) , "recipes/food/suspicious_stew_from_closed_eyeblossom" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CLOSED_EYEBLOSSOM) , "recipes/food/suspicious_stew_from_cornflower" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CORNFLOWER) , "recipes/food/suspicious_stew_from_dandelion" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_DANDELION) , "recipes/food/suspicious_stew_from_lily_of_the_valley" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_LILY_OF_THE_VALLEY) , "recipes/food/suspicious_stew_from_open_eyeblossom" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OPEN_EYEBLOSSOM) , "recipes/food/suspicious_stew_from_orange_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ORANGE_TULIP) , "recipes/food/suspicious_stew_from_oxeye_daisy" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OXEYE_DAISY) , "recipes/food/suspicious_stew_from_pink_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_PINK_TULIP) , "recipes/food/suspicious_stew_from_poppy" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_POPPY) , "recipes/food/suspicious_stew_from_red_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_RED_TULIP) , "recipes/food/suspicious_stew_from_torchflower" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_TORCHFLOWER) , "recipes/food/suspicious_stew_from_white_tulip" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WHITE_TULIP) , "recipes/food/suspicious_stew_from_wither_rose" => Some (& Self :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WITHER_ROSE) , "recipes/misc/beacon" => Some (& Self :: RECIPES_MISC_BEACON) , "recipes/misc/black_dye" => Some (& Self :: RECIPES_MISC_BLACK_DYE) , "recipes/misc/black_dye_from_wither_rose" => Some (& Self :: RECIPES_MISC_BLACK_DYE_FROM_WITHER_ROSE) , "recipes/misc/blue_dye" => Some (& Self :: RECIPES_MISC_BLUE_DYE) , "recipes/misc/blue_dye_from_cornflower" => Some (& Self :: RECIPES_MISC_BLUE_DYE_FROM_CORNFLOWER) , "recipes/misc/bolt_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/bolt_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/bone_meal" => Some (& Self :: RECIPES_MISC_BONE_MEAL) , "recipes/misc/bone_meal_from_bone_block" => Some (& Self :: RECIPES_MISC_BONE_MEAL_FROM_BONE_BLOCK) , "recipes/misc/book" => Some (& Self :: RECIPES_MISC_BOOK) , "recipes/misc/bordure_indented_banner_pattern" => Some (& Self :: RECIPES_MISC_BORDURE_INDENTED_BANNER_PATTERN) , "recipes/misc/bowl" => Some (& Self :: RECIPES_MISC_BOWL) , "recipes/misc/brick" => Some (& Self :: RECIPES_MISC_BRICK) , "recipes/misc/brown_dye" => Some (& Self :: RECIPES_MISC_BROWN_DYE) , "recipes/misc/bucket" => Some (& Self :: RECIPES_MISC_BUCKET) , "recipes/misc/charcoal" => Some (& Self :: RECIPES_MISC_CHARCOAL) , "recipes/misc/coal" => Some (& Self :: RECIPES_MISC_COAL) , "recipes/misc/coal_from_blasting_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_BLASTING_COAL_ORE) , "recipes/misc/coal_from_blasting_deepslate_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_BLASTING_DEEPSLATE_COAL_ORE) , "recipes/misc/coal_from_smelting_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_SMELTING_COAL_ORE) , "recipes/misc/coal_from_smelting_deepslate_coal_ore" => Some (& Self :: RECIPES_MISC_COAL_FROM_SMELTING_DEEPSLATE_COAL_ORE) , "recipes/misc/coast_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/coast_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/conduit" => Some (& Self :: RECIPES_MISC_CONDUIT) , "recipes/misc/copper_ingot" => Some (& Self :: RECIPES_MISC_COPPER_INGOT) , "recipes/misc/copper_ingot_from_blasting_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_COPPER_ORE) , "recipes/misc/copper_ingot_from_blasting_deepslate_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_DEEPSLATE_COPPER_ORE) , "recipes/misc/copper_ingot_from_blasting_raw_copper" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_RAW_COPPER) , "recipes/misc/copper_ingot_from_nuggets" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_NUGGETS) , "recipes/misc/copper_ingot_from_smelting_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_COPPER_ORE) , "recipes/misc/copper_ingot_from_smelting_deepslate_copper_ore" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_DEEPSLATE_COPPER_ORE) , "recipes/misc/copper_ingot_from_smelting_raw_copper" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_RAW_COPPER) , "recipes/misc/copper_ingot_from_waxed_copper_block" => Some (& Self :: RECIPES_MISC_COPPER_INGOT_FROM_WAXED_COPPER_BLOCK) , "recipes/misc/copper_nugget" => Some (& Self :: RECIPES_MISC_COPPER_NUGGET) , "recipes/misc/copper_nugget_from_blasting" => Some (& Self :: RECIPES_MISC_COPPER_NUGGET_FROM_BLASTING) , "recipes/misc/copper_nugget_from_smelting" => Some (& Self :: RECIPES_MISC_COPPER_NUGGET_FROM_SMELTING) , "recipes/misc/creaking_heart" => Some (& Self :: RECIPES_MISC_CREAKING_HEART) , "recipes/misc/creeper_banner_pattern" => Some (& Self :: RECIPES_MISC_CREEPER_BANNER_PATTERN) , "recipes/misc/cyan_dye" => Some (& Self :: RECIPES_MISC_CYAN_DYE) , "recipes/misc/cyan_dye_from_pitcher_plant" => Some (& Self :: RECIPES_MISC_CYAN_DYE_FROM_PITCHER_PLANT) , "recipes/misc/diamond" => Some (& Self :: RECIPES_MISC_DIAMOND) , "recipes/misc/diamond_from_blasting_deepslate_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_BLASTING_DEEPSLATE_DIAMOND_ORE) , "recipes/misc/diamond_from_blasting_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_BLASTING_DIAMOND_ORE) , "recipes/misc/diamond_from_smelting_deepslate_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_SMELTING_DEEPSLATE_DIAMOND_ORE) , "recipes/misc/diamond_from_smelting_diamond_ore" => Some (& Self :: RECIPES_MISC_DIAMOND_FROM_SMELTING_DIAMOND_ORE) , "recipes/misc/dune_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/dune_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/emerald" => Some (& Self :: RECIPES_MISC_EMERALD) , "recipes/misc/emerald_from_blasting_deepslate_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_BLASTING_DEEPSLATE_EMERALD_ORE) , "recipes/misc/emerald_from_blasting_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_BLASTING_EMERALD_ORE) , "recipes/misc/emerald_from_smelting_deepslate_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_SMELTING_DEEPSLATE_EMERALD_ORE) , "recipes/misc/emerald_from_smelting_emerald_ore" => Some (& Self :: RECIPES_MISC_EMERALD_FROM_SMELTING_EMERALD_ORE) , "recipes/misc/ender_eye" => Some (& Self :: RECIPES_MISC_ENDER_EYE) , "recipes/misc/eye_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/eye_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/field_masoned_banner_pattern" => Some (& Self :: RECIPES_MISC_FIELD_MASONED_BANNER_PATTERN) , "recipes/misc/fire_charge" => Some (& Self :: RECIPES_MISC_FIRE_CHARGE) , "recipes/misc/firework_rocket_simple" => Some (& Self :: RECIPES_MISC_FIREWORK_ROCKET_SIMPLE) , "recipes/misc/flow_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/flow_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/flower_banner_pattern" => Some (& Self :: RECIPES_MISC_FLOWER_BANNER_PATTERN) , "recipes/misc/gold_ingot_from_blasting_deepslate_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_DEEPSLATE_GOLD_ORE) , "recipes/misc/gold_ingot_from_blasting_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_GOLD_ORE) , "recipes/misc/gold_ingot_from_blasting_nether_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_NETHER_GOLD_ORE) , "recipes/misc/gold_ingot_from_blasting_raw_gold" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_RAW_GOLD) , "recipes/misc/gold_ingot_from_gold_block" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_GOLD_BLOCK) , "recipes/misc/gold_ingot_from_nuggets" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_NUGGETS) , "recipes/misc/gold_ingot_from_smelting_deepslate_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_DEEPSLATE_GOLD_ORE) , "recipes/misc/gold_ingot_from_smelting_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_GOLD_ORE) , "recipes/misc/gold_ingot_from_smelting_nether_gold_ore" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_NETHER_GOLD_ORE) , "recipes/misc/gold_ingot_from_smelting_raw_gold" => Some (& Self :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_RAW_GOLD) , "recipes/misc/gold_nugget" => Some (& Self :: RECIPES_MISC_GOLD_NUGGET) , "recipes/misc/gold_nugget_from_blasting" => Some (& Self :: RECIPES_MISC_GOLD_NUGGET_FROM_BLASTING) , "recipes/misc/gold_nugget_from_smelting" => Some (& Self :: RECIPES_MISC_GOLD_NUGGET_FROM_SMELTING) , "recipes/misc/gray_dye" => Some (& Self :: RECIPES_MISC_GRAY_DYE) , "recipes/misc/gray_dye_from_closed_eyeblossom" => Some (& Self :: RECIPES_MISC_GRAY_DYE_FROM_CLOSED_EYEBLOSSOM) , "recipes/misc/green_dye" => Some (& Self :: RECIPES_MISC_GREEN_DYE) , "recipes/misc/host_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/host_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/iron_ingot_from_blasting_deepslate_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_DEEPSLATE_IRON_ORE) , "recipes/misc/iron_ingot_from_blasting_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_IRON_ORE) , "recipes/misc/iron_ingot_from_blasting_raw_iron" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_RAW_IRON) , "recipes/misc/iron_ingot_from_iron_block" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_IRON_BLOCK) , "recipes/misc/iron_ingot_from_nuggets" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_NUGGETS) , "recipes/misc/iron_ingot_from_smelting_deepslate_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_DEEPSLATE_IRON_ORE) , "recipes/misc/iron_ingot_from_smelting_iron_ore" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_IRON_ORE) , "recipes/misc/iron_ingot_from_smelting_raw_iron" => Some (& Self :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_RAW_IRON) , "recipes/misc/iron_nugget" => Some (& Self :: RECIPES_MISC_IRON_NUGGET) , "recipes/misc/iron_nugget_from_blasting" => Some (& Self :: RECIPES_MISC_IRON_NUGGET_FROM_BLASTING) , "recipes/misc/iron_nugget_from_smelting" => Some (& Self :: RECIPES_MISC_IRON_NUGGET_FROM_SMELTING) , "recipes/misc/lapis_lazuli" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI) , "recipes/misc/lapis_lazuli_from_blasting_deepslate_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_DEEPSLATE_LAPIS_ORE) , "recipes/misc/lapis_lazuli_from_blasting_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_LAPIS_ORE) , "recipes/misc/lapis_lazuli_from_smelting_deepslate_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_DEEPSLATE_LAPIS_ORE) , "recipes/misc/lapis_lazuli_from_smelting_lapis_ore" => Some (& Self :: RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_LAPIS_ORE) , "recipes/misc/leaf_litter" => Some (& Self :: RECIPES_MISC_LEAF_LITTER) , "recipes/misc/leather" => Some (& Self :: RECIPES_MISC_LEATHER) , "recipes/misc/leather_horse_armor" => Some (& Self :: RECIPES_MISC_LEATHER_HORSE_ARMOR) , "recipes/misc/light_blue_dye_from_blue_orchid" => Some (& Self :: RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_ORCHID) , "recipes/misc/light_blue_dye_from_blue_white_dye" => Some (& Self :: RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_WHITE_DYE) , "recipes/misc/light_gray_dye_from_azure_bluet" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_AZURE_BLUET) , "recipes/misc/light_gray_dye_from_black_white_dye" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_BLACK_WHITE_DYE) , "recipes/misc/light_gray_dye_from_gray_white_dye" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_GRAY_WHITE_DYE) , "recipes/misc/light_gray_dye_from_oxeye_daisy" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_OXEYE_DAISY) , "recipes/misc/light_gray_dye_from_white_tulip" => Some (& Self :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_WHITE_TULIP) , "recipes/misc/lime_dye" => Some (& Self :: RECIPES_MISC_LIME_DYE) , "recipes/misc/lime_dye_from_smelting" => Some (& Self :: RECIPES_MISC_LIME_DYE_FROM_SMELTING) , "recipes/misc/magenta_dye_from_allium" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_ALLIUM) , "recipes/misc/magenta_dye_from_blue_red_pink" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_PINK) , "recipes/misc/magenta_dye_from_blue_red_white_dye" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_WHITE_DYE) , "recipes/misc/magenta_dye_from_lilac" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_LILAC) , "recipes/misc/magenta_dye_from_purple_and_pink" => Some (& Self :: RECIPES_MISC_MAGENTA_DYE_FROM_PURPLE_AND_PINK) , "recipes/misc/map" => Some (& Self :: RECIPES_MISC_MAP) , "recipes/misc/melon_seeds" => Some (& Self :: RECIPES_MISC_MELON_SEEDS) , "recipes/misc/mojang_banner_pattern" => Some (& Self :: RECIPES_MISC_MOJANG_BANNER_PATTERN) , "recipes/misc/music_disc_5" => Some (& Self :: RECIPES_MISC_MUSIC_DISC_5) , "recipes/misc/nether_brick" => Some (& Self :: RECIPES_MISC_NETHER_BRICK) , "recipes/misc/netherite_ingot" => Some (& Self :: RECIPES_MISC_NETHERITE_INGOT) , "recipes/misc/netherite_ingot_from_netherite_block" => Some (& Self :: RECIPES_MISC_NETHERITE_INGOT_FROM_NETHERITE_BLOCK) , "recipes/misc/netherite_scrap" => Some (& Self :: RECIPES_MISC_NETHERITE_SCRAP) , "recipes/misc/netherite_scrap_from_blasting" => Some (& Self :: RECIPES_MISC_NETHERITE_SCRAP_FROM_BLASTING) , "recipes/misc/netherite_upgrade_smithing_template" => Some (& Self :: RECIPES_MISC_NETHERITE_UPGRADE_SMITHING_TEMPLATE) , "recipes/misc/orange_dye_from_open_eyeblossom" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_OPEN_EYEBLOSSOM) , "recipes/misc/orange_dye_from_orange_tulip" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_ORANGE_TULIP) , "recipes/misc/orange_dye_from_red_yellow" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_RED_YELLOW) , "recipes/misc/orange_dye_from_torchflower" => Some (& Self :: RECIPES_MISC_ORANGE_DYE_FROM_TORCHFLOWER) , "recipes/misc/paper" => Some (& Self :: RECIPES_MISC_PAPER) , "recipes/misc/pink_dye_from_cactus_flower" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_CACTUS_FLOWER) , "recipes/misc/pink_dye_from_peony" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_PEONY) , "recipes/misc/pink_dye_from_pink_petals" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_PINK_PETALS) , "recipes/misc/pink_dye_from_pink_tulip" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_PINK_TULIP) , "recipes/misc/pink_dye_from_red_white_dye" => Some (& Self :: RECIPES_MISC_PINK_DYE_FROM_RED_WHITE_DYE) , "recipes/misc/popped_chorus_fruit" => Some (& Self :: RECIPES_MISC_POPPED_CHORUS_FRUIT) , "recipes/misc/pumpkin_seeds" => Some (& Self :: RECIPES_MISC_PUMPKIN_SEEDS) , "recipes/misc/purple_dye" => Some (& Self :: RECIPES_MISC_PURPLE_DYE) , "recipes/misc/quartz" => Some (& Self :: RECIPES_MISC_QUARTZ) , "recipes/misc/quartz_from_blasting" => Some (& Self :: RECIPES_MISC_QUARTZ_FROM_BLASTING) , "recipes/misc/raiser_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/raiser_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/raw_copper" => Some (& Self :: RECIPES_MISC_RAW_COPPER) , "recipes/misc/raw_gold" => Some (& Self :: RECIPES_MISC_RAW_GOLD) , "recipes/misc/raw_iron" => Some (& Self :: RECIPES_MISC_RAW_IRON) , "recipes/misc/red_dye_from_beetroot" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_BEETROOT) , "recipes/misc/red_dye_from_poppy" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_POPPY) , "recipes/misc/red_dye_from_rose_bush" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_ROSE_BUSH) , "recipes/misc/red_dye_from_tulip" => Some (& Self :: RECIPES_MISC_RED_DYE_FROM_TULIP) , "recipes/misc/resin_brick" => Some (& Self :: RECIPES_MISC_RESIN_BRICK) , "recipes/misc/resin_clump" => Some (& Self :: RECIPES_MISC_RESIN_CLUMP) , "recipes/misc/rib_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/rib_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/sentry_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/sentry_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/shaper_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/shaper_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/silence_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/silence_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/skull_banner_pattern" => Some (& Self :: RECIPES_MISC_SKULL_BANNER_PATTERN) , "recipes/misc/slime_ball" => Some (& Self :: RECIPES_MISC_SLIME_BALL) , "recipes/misc/snout_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/snout_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/spire_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/spire_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/stick" => Some (& Self :: RECIPES_MISC_STICK) , "recipes/misc/stick_from_bamboo_item" => Some (& Self :: RECIPES_MISC_STICK_FROM_BAMBOO_ITEM) , "recipes/misc/sugar_from_honey_bottle" => Some (& Self :: RECIPES_MISC_SUGAR_FROM_HONEY_BOTTLE) , "recipes/misc/sugar_from_sugar_cane" => Some (& Self :: RECIPES_MISC_SUGAR_FROM_SUGAR_CANE) , "recipes/misc/tide_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/tide_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/vex_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/vex_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/ward_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/ward_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/wayfinder_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/wayfinder_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/wheat" => Some (& Self :: RECIPES_MISC_WHEAT) , "recipes/misc/white_dye" => Some (& Self :: RECIPES_MISC_WHITE_DYE) , "recipes/misc/white_dye_from_lily_of_the_valley" => Some (& Self :: RECIPES_MISC_WHITE_DYE_FROM_LILY_OF_THE_VALLEY) , "recipes/misc/wild_armor_trim_smithing_template" => Some (& Self :: RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE) , "recipes/misc/wild_armor_trim_smithing_template_smithing_trim" => Some (& Self :: RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM) , "recipes/misc/wind_charge" => Some (& Self :: RECIPES_MISC_WIND_CHARGE) , "recipes/misc/writable_book" => Some (& Self :: RECIPES_MISC_WRITABLE_BOOK) , "recipes/misc/yellow_dye_from_dandelion" => Some (& Self :: RECIPES_MISC_YELLOW_DYE_FROM_DANDELION) , "recipes/misc/yellow_dye_from_sunflower" => Some (& Self :: RECIPES_MISC_YELLOW_DYE_FROM_SUNFLOWER) , "recipes/misc/yellow_dye_from_wildflowers" => Some (& Self :: RECIPES_MISC_YELLOW_DYE_FROM_WILDFLOWERS) , "recipes/redstone/acacia_button" => Some (& Self :: RECIPES_REDSTONE_ACACIA_BUTTON) , "recipes/redstone/acacia_door" => Some (& Self :: RECIPES_REDSTONE_ACACIA_DOOR) , "recipes/redstone/acacia_fence_gate" => Some (& Self :: RECIPES_REDSTONE_ACACIA_FENCE_GATE) , "recipes/redstone/acacia_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_ACACIA_PRESSURE_PLATE) , "recipes/redstone/acacia_trapdoor" => Some (& Self :: RECIPES_REDSTONE_ACACIA_TRAPDOOR) , "recipes/redstone/bamboo_button" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_BUTTON) , "recipes/redstone/bamboo_door" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_DOOR) , "recipes/redstone/bamboo_fence_gate" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_FENCE_GATE) , "recipes/redstone/bamboo_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_PRESSURE_PLATE) , "recipes/redstone/bamboo_trapdoor" => Some (& Self :: RECIPES_REDSTONE_BAMBOO_TRAPDOOR) , "recipes/redstone/birch_button" => Some (& Self :: RECIPES_REDSTONE_BIRCH_BUTTON) , "recipes/redstone/birch_door" => Some (& Self :: RECIPES_REDSTONE_BIRCH_DOOR) , "recipes/redstone/birch_fence_gate" => Some (& Self :: RECIPES_REDSTONE_BIRCH_FENCE_GATE) , "recipes/redstone/birch_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_BIRCH_PRESSURE_PLATE) , "recipes/redstone/birch_trapdoor" => Some (& Self :: RECIPES_REDSTONE_BIRCH_TRAPDOOR) , "recipes/redstone/calibrated_sculk_sensor" => Some (& Self :: RECIPES_REDSTONE_CALIBRATED_SCULK_SENSOR) , "recipes/redstone/cherry_button" => Some (& Self :: RECIPES_REDSTONE_CHERRY_BUTTON) , "recipes/redstone/cherry_door" => Some (& Self :: RECIPES_REDSTONE_CHERRY_DOOR) , "recipes/redstone/cherry_fence_gate" => Some (& Self :: RECIPES_REDSTONE_CHERRY_FENCE_GATE) , "recipes/redstone/cherry_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_CHERRY_PRESSURE_PLATE) , "recipes/redstone/cherry_trapdoor" => Some (& Self :: RECIPES_REDSTONE_CHERRY_TRAPDOOR) , "recipes/redstone/comparator" => Some (& Self :: RECIPES_REDSTONE_COMPARATOR) , "recipes/redstone/copper_bulb" => Some (& Self :: RECIPES_REDSTONE_COPPER_BULB) , "recipes/redstone/copper_door" => Some (& Self :: RECIPES_REDSTONE_COPPER_DOOR) , "recipes/redstone/copper_trapdoor" => Some (& Self :: RECIPES_REDSTONE_COPPER_TRAPDOOR) , "recipes/redstone/crafter" => Some (& Self :: RECIPES_REDSTONE_CRAFTER) , "recipes/redstone/crimson_button" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_BUTTON) , "recipes/redstone/crimson_door" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_DOOR) , "recipes/redstone/crimson_fence_gate" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_FENCE_GATE) , "recipes/redstone/crimson_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_PRESSURE_PLATE) , "recipes/redstone/crimson_trapdoor" => Some (& Self :: RECIPES_REDSTONE_CRIMSON_TRAPDOOR) , "recipes/redstone/dark_oak_button" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_BUTTON) , "recipes/redstone/dark_oak_door" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_DOOR) , "recipes/redstone/dark_oak_fence_gate" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_FENCE_GATE) , "recipes/redstone/dark_oak_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_PRESSURE_PLATE) , "recipes/redstone/dark_oak_trapdoor" => Some (& Self :: RECIPES_REDSTONE_DARK_OAK_TRAPDOOR) , "recipes/redstone/daylight_detector" => Some (& Self :: RECIPES_REDSTONE_DAYLIGHT_DETECTOR) , "recipes/redstone/dispenser" => Some (& Self :: RECIPES_REDSTONE_DISPENSER) , "recipes/redstone/dropper" => Some (& Self :: RECIPES_REDSTONE_DROPPER) , "recipes/redstone/exposed_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_EXPOSED_COPPER_BULB) , "recipes/redstone/heavy_weighted_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_HEAVY_WEIGHTED_PRESSURE_PLATE) , "recipes/redstone/honey_block" => Some (& Self :: RECIPES_REDSTONE_HONEY_BLOCK) , "recipes/redstone/hopper" => Some (& Self :: RECIPES_REDSTONE_HOPPER) , "recipes/redstone/iron_door" => Some (& Self :: RECIPES_REDSTONE_IRON_DOOR) , "recipes/redstone/iron_trapdoor" => Some (& Self :: RECIPES_REDSTONE_IRON_TRAPDOOR) , "recipes/redstone/jungle_button" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_BUTTON) , "recipes/redstone/jungle_door" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_DOOR) , "recipes/redstone/jungle_fence_gate" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_FENCE_GATE) , "recipes/redstone/jungle_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_PRESSURE_PLATE) , "recipes/redstone/jungle_trapdoor" => Some (& Self :: RECIPES_REDSTONE_JUNGLE_TRAPDOOR) , "recipes/redstone/lectern" => Some (& Self :: RECIPES_REDSTONE_LECTERN) , "recipes/redstone/lever" => Some (& Self :: RECIPES_REDSTONE_LEVER) , "recipes/redstone/light_weighted_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_LIGHT_WEIGHTED_PRESSURE_PLATE) , "recipes/redstone/lightning_rod" => Some (& Self :: RECIPES_REDSTONE_LIGHTNING_ROD) , "recipes/redstone/mangrove_button" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_BUTTON) , "recipes/redstone/mangrove_door" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_DOOR) , "recipes/redstone/mangrove_fence_gate" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_FENCE_GATE) , "recipes/redstone/mangrove_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_PRESSURE_PLATE) , "recipes/redstone/mangrove_trapdoor" => Some (& Self :: RECIPES_REDSTONE_MANGROVE_TRAPDOOR) , "recipes/redstone/note_block" => Some (& Self :: RECIPES_REDSTONE_NOTE_BLOCK) , "recipes/redstone/oak_button" => Some (& Self :: RECIPES_REDSTONE_OAK_BUTTON) , "recipes/redstone/oak_door" => Some (& Self :: RECIPES_REDSTONE_OAK_DOOR) , "recipes/redstone/oak_fence_gate" => Some (& Self :: RECIPES_REDSTONE_OAK_FENCE_GATE) , "recipes/redstone/oak_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_OAK_PRESSURE_PLATE) , "recipes/redstone/oak_trapdoor" => Some (& Self :: RECIPES_REDSTONE_OAK_TRAPDOOR) , "recipes/redstone/observer" => Some (& Self :: RECIPES_REDSTONE_OBSERVER) , "recipes/redstone/oxidized_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_OXIDIZED_COPPER_BULB) , "recipes/redstone/pale_oak_button" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_BUTTON) , "recipes/redstone/pale_oak_door" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_DOOR) , "recipes/redstone/pale_oak_fence_gate" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_FENCE_GATE) , "recipes/redstone/pale_oak_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_PRESSURE_PLATE) , "recipes/redstone/pale_oak_trapdoor" => Some (& Self :: RECIPES_REDSTONE_PALE_OAK_TRAPDOOR) , "recipes/redstone/piston" => Some (& Self :: RECIPES_REDSTONE_PISTON) , "recipes/redstone/polished_blackstone_button" => Some (& Self :: RECIPES_REDSTONE_POLISHED_BLACKSTONE_BUTTON) , "recipes/redstone/polished_blackstone_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_POLISHED_BLACKSTONE_PRESSURE_PLATE) , "recipes/redstone/redstone" => Some (& Self :: RECIPES_REDSTONE_REDSTONE) , "recipes/redstone/redstone_block" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_BLOCK) , "recipes/redstone/redstone_from_blasting_deepslate_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_DEEPSLATE_REDSTONE_ORE) , "recipes/redstone/redstone_from_blasting_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_REDSTONE_ORE) , "recipes/redstone/redstone_from_smelting_deepslate_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_DEEPSLATE_REDSTONE_ORE) , "recipes/redstone/redstone_from_smelting_redstone_ore" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_REDSTONE_ORE) , "recipes/redstone/redstone_lamp" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_LAMP) , "recipes/redstone/redstone_torch" => Some (& Self :: RECIPES_REDSTONE_REDSTONE_TORCH) , "recipes/redstone/repeater" => Some (& Self :: RECIPES_REDSTONE_REPEATER) , "recipes/redstone/slime_block" => Some (& Self :: RECIPES_REDSTONE_SLIME_BLOCK) , "recipes/redstone/spruce_button" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_BUTTON) , "recipes/redstone/spruce_door" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_DOOR) , "recipes/redstone/spruce_fence_gate" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_FENCE_GATE) , "recipes/redstone/spruce_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_PRESSURE_PLATE) , "recipes/redstone/spruce_trapdoor" => Some (& Self :: RECIPES_REDSTONE_SPRUCE_TRAPDOOR) , "recipes/redstone/sticky_piston" => Some (& Self :: RECIPES_REDSTONE_STICKY_PISTON) , "recipes/redstone/stone_button" => Some (& Self :: RECIPES_REDSTONE_STONE_BUTTON) , "recipes/redstone/stone_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_STONE_PRESSURE_PLATE) , "recipes/redstone/target" => Some (& Self :: RECIPES_REDSTONE_TARGET) , "recipes/redstone/tnt" => Some (& Self :: RECIPES_REDSTONE_TNT) , "recipes/redstone/trapped_chest" => Some (& Self :: RECIPES_REDSTONE_TRAPPED_CHEST) , "recipes/redstone/tripwire_hook" => Some (& Self :: RECIPES_REDSTONE_TRIPWIRE_HOOK) , "recipes/redstone/warped_button" => Some (& Self :: RECIPES_REDSTONE_WARPED_BUTTON) , "recipes/redstone/warped_door" => Some (& Self :: RECIPES_REDSTONE_WARPED_DOOR) , "recipes/redstone/warped_fence_gate" => Some (& Self :: RECIPES_REDSTONE_WARPED_FENCE_GATE) , "recipes/redstone/warped_pressure_plate" => Some (& Self :: RECIPES_REDSTONE_WARPED_PRESSURE_PLATE) , "recipes/redstone/warped_trapdoor" => Some (& Self :: RECIPES_REDSTONE_WARPED_TRAPDOOR) , "recipes/redstone/waxed_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_BULB) , "recipes/redstone/waxed_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_exposed_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB) , "recipes/redstone/waxed_exposed_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_exposed_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_exposed_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_oxidized_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB) , "recipes/redstone/waxed_oxidized_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_oxidized_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_oxidized_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_weathered_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB) , "recipes/redstone/waxed_weathered_copper_bulb_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB_FROM_HONEYCOMB) , "recipes/redstone/waxed_weathered_copper_door_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_DOOR_FROM_HONEYCOMB) , "recipes/redstone/waxed_weathered_copper_trapdoor_from_honeycomb" => Some (& Self :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_TRAPDOOR_FROM_HONEYCOMB) , "recipes/redstone/weathered_copper_bulb" => Some (& Self :: RECIPES_REDSTONE_WEATHERED_COPPER_BULB) , "recipes/root" => Some (& Self :: RECIPES_ROOT) , "recipes/tools/black_bundle" => Some (& Self :: RECIPES_TOOLS_BLACK_BUNDLE) , "recipes/tools/blue_bundle" => Some (& Self :: RECIPES_TOOLS_BLUE_BUNDLE) , "recipes/tools/brown_bundle" => Some (& Self :: RECIPES_TOOLS_BROWN_BUNDLE) , "recipes/tools/brush" => Some (& Self :: RECIPES_TOOLS_BRUSH) , "recipes/tools/bundle" => Some (& Self :: RECIPES_TOOLS_BUNDLE) , "recipes/tools/clock" => Some (& Self :: RECIPES_TOOLS_CLOCK) , "recipes/tools/compass" => Some (& Self :: RECIPES_TOOLS_COMPASS) , "recipes/tools/copper_axe" => Some (& Self :: RECIPES_TOOLS_COPPER_AXE) , "recipes/tools/copper_hoe" => Some (& Self :: RECIPES_TOOLS_COPPER_HOE) , "recipes/tools/copper_pickaxe" => Some (& Self :: RECIPES_TOOLS_COPPER_PICKAXE) , "recipes/tools/copper_shovel" => Some (& Self :: RECIPES_TOOLS_COPPER_SHOVEL) , "recipes/tools/cyan_bundle" => Some (& Self :: RECIPES_TOOLS_CYAN_BUNDLE) , "recipes/tools/diamond_axe" => Some (& Self :: RECIPES_TOOLS_DIAMOND_AXE) , "recipes/tools/diamond_hoe" => Some (& Self :: RECIPES_TOOLS_DIAMOND_HOE) , "recipes/tools/diamond_pickaxe" => Some (& Self :: RECIPES_TOOLS_DIAMOND_PICKAXE) , "recipes/tools/diamond_shovel" => Some (& Self :: RECIPES_TOOLS_DIAMOND_SHOVEL) , "recipes/tools/fishing_rod" => Some (& Self :: RECIPES_TOOLS_FISHING_ROD) , "recipes/tools/flint_and_steel" => Some (& Self :: RECIPES_TOOLS_FLINT_AND_STEEL) , "recipes/tools/golden_axe" => Some (& Self :: RECIPES_TOOLS_GOLDEN_AXE) , "recipes/tools/golden_hoe" => Some (& Self :: RECIPES_TOOLS_GOLDEN_HOE) , "recipes/tools/golden_pickaxe" => Some (& Self :: RECIPES_TOOLS_GOLDEN_PICKAXE) , "recipes/tools/golden_shovel" => Some (& Self :: RECIPES_TOOLS_GOLDEN_SHOVEL) , "recipes/tools/gray_bundle" => Some (& Self :: RECIPES_TOOLS_GRAY_BUNDLE) , "recipes/tools/green_bundle" => Some (& Self :: RECIPES_TOOLS_GREEN_BUNDLE) , "recipes/tools/iron_axe" => Some (& Self :: RECIPES_TOOLS_IRON_AXE) , "recipes/tools/iron_hoe" => Some (& Self :: RECIPES_TOOLS_IRON_HOE) , "recipes/tools/iron_pickaxe" => Some (& Self :: RECIPES_TOOLS_IRON_PICKAXE) , "recipes/tools/iron_shovel" => Some (& Self :: RECIPES_TOOLS_IRON_SHOVEL) , "recipes/tools/lead" => Some (& Self :: RECIPES_TOOLS_LEAD) , "recipes/tools/light_blue_bundle" => Some (& Self :: RECIPES_TOOLS_LIGHT_BLUE_BUNDLE) , "recipes/tools/light_gray_bundle" => Some (& Self :: RECIPES_TOOLS_LIGHT_GRAY_BUNDLE) , "recipes/tools/lime_bundle" => Some (& Self :: RECIPES_TOOLS_LIME_BUNDLE) , "recipes/tools/magenta_bundle" => Some (& Self :: RECIPES_TOOLS_MAGENTA_BUNDLE) , "recipes/tools/netherite_axe_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_AXE_SMITHING) , "recipes/tools/netherite_hoe_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_HOE_SMITHING) , "recipes/tools/netherite_pickaxe_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_PICKAXE_SMITHING) , "recipes/tools/netherite_shovel_smithing" => Some (& Self :: RECIPES_TOOLS_NETHERITE_SHOVEL_SMITHING) , "recipes/tools/orange_bundle" => Some (& Self :: RECIPES_TOOLS_ORANGE_BUNDLE) , "recipes/tools/pink_bundle" => Some (& Self :: RECIPES_TOOLS_PINK_BUNDLE) , "recipes/tools/purple_bundle" => Some (& Self :: RECIPES_TOOLS_PURPLE_BUNDLE) , "recipes/tools/recovery_compass" => Some (& Self :: RECIPES_TOOLS_RECOVERY_COMPASS) , "recipes/tools/red_bundle" => Some (& Self :: RECIPES_TOOLS_RED_BUNDLE) , "recipes/tools/shears" => Some (& Self :: RECIPES_TOOLS_SHEARS) , "recipes/tools/spyglass" => Some (& Self :: RECIPES_TOOLS_SPYGLASS) , "recipes/tools/stone_axe" => Some (& Self :: RECIPES_TOOLS_STONE_AXE) , "recipes/tools/stone_hoe" => Some (& Self :: RECIPES_TOOLS_STONE_HOE) , "recipes/tools/stone_pickaxe" => Some (& Self :: RECIPES_TOOLS_STONE_PICKAXE) , "recipes/tools/stone_shovel" => Some (& Self :: RECIPES_TOOLS_STONE_SHOVEL) , "recipes/tools/white_bundle" => Some (& Self :: RECIPES_TOOLS_WHITE_BUNDLE) , "recipes/tools/wooden_axe" => Some (& Self :: RECIPES_TOOLS_WOODEN_AXE) , "recipes/tools/wooden_hoe" => Some (& Self :: RECIPES_TOOLS_WOODEN_HOE) , "recipes/tools/wooden_pickaxe" => Some (& Self :: RECIPES_TOOLS_WOODEN_PICKAXE) , "recipes/tools/wooden_shovel" => Some (& Self :: RECIPES_TOOLS_WOODEN_SHOVEL) , "recipes/tools/yellow_bundle" => Some (& Self :: RECIPES_TOOLS_YELLOW_BUNDLE) , "recipes/transportation/acacia_boat" => Some (& Self :: RECIPES_TRANSPORTATION_ACACIA_BOAT) , "recipes/transportation/acacia_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_ACACIA_CHEST_BOAT) , "recipes/transportation/activator_rail" => Some (& Self :: RECIPES_TRANSPORTATION_ACTIVATOR_RAIL) , "recipes/transportation/bamboo_chest_raft" => Some (& Self :: RECIPES_TRANSPORTATION_BAMBOO_CHEST_RAFT) , "recipes/transportation/bamboo_raft" => Some (& Self :: RECIPES_TRANSPORTATION_BAMBOO_RAFT) , "recipes/transportation/birch_boat" => Some (& Self :: RECIPES_TRANSPORTATION_BIRCH_BOAT) , "recipes/transportation/birch_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_BIRCH_CHEST_BOAT) , "recipes/transportation/carrot_on_a_stick" => Some (& Self :: RECIPES_TRANSPORTATION_CARROT_ON_A_STICK) , "recipes/transportation/cherry_boat" => Some (& Self :: RECIPES_TRANSPORTATION_CHERRY_BOAT) , "recipes/transportation/cherry_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_CHERRY_CHEST_BOAT) , "recipes/transportation/chest_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_CHEST_MINECART) , "recipes/transportation/dark_oak_boat" => Some (& Self :: RECIPES_TRANSPORTATION_DARK_OAK_BOAT) , "recipes/transportation/dark_oak_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_DARK_OAK_CHEST_BOAT) , "recipes/transportation/detector_rail" => Some (& Self :: RECIPES_TRANSPORTATION_DETECTOR_RAIL) , "recipes/transportation/furnace_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_FURNACE_MINECART) , "recipes/transportation/hopper_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_HOPPER_MINECART) , "recipes/transportation/jungle_boat" => Some (& Self :: RECIPES_TRANSPORTATION_JUNGLE_BOAT) , "recipes/transportation/jungle_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_JUNGLE_CHEST_BOAT) , "recipes/transportation/mangrove_boat" => Some (& Self :: RECIPES_TRANSPORTATION_MANGROVE_BOAT) , "recipes/transportation/mangrove_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_MANGROVE_CHEST_BOAT) , "recipes/transportation/minecart" => Some (& Self :: RECIPES_TRANSPORTATION_MINECART) , "recipes/transportation/oak_boat" => Some (& Self :: RECIPES_TRANSPORTATION_OAK_BOAT) , "recipes/transportation/oak_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_OAK_CHEST_BOAT) , "recipes/transportation/pale_oak_boat" => Some (& Self :: RECIPES_TRANSPORTATION_PALE_OAK_BOAT) , "recipes/transportation/pale_oak_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_PALE_OAK_CHEST_BOAT) , "recipes/transportation/powered_rail" => Some (& Self :: RECIPES_TRANSPORTATION_POWERED_RAIL) , "recipes/transportation/rail" => Some (& Self :: RECIPES_TRANSPORTATION_RAIL) , "recipes/transportation/spruce_boat" => Some (& Self :: RECIPES_TRANSPORTATION_SPRUCE_BOAT) , "recipes/transportation/spruce_chest_boat" => Some (& Self :: RECIPES_TRANSPORTATION_SPRUCE_CHEST_BOAT) , "recipes/transportation/tnt_minecart" => Some (& Self :: RECIPES_TRANSPORTATION_TNT_MINECART) , "recipes/transportation/warped_fungus_on_a_stick" => Some (& Self :: RECIPES_TRANSPORTATION_WARPED_FUNGUS_ON_A_STICK) , "story/cure_zombie_villager" => Some (& Self :: STORY_CURE_ZOMBIE_VILLAGER) , "story/deflect_arrow" => Some (& Self :: STORY_DEFLECT_ARROW) , "story/enchant_item" => Some (& Self :: STORY_ENCHANT_ITEM) , "story/enter_the_end" => Some (& Self :: STORY_ENTER_THE_END) , "story/enter_the_nether" => Some (& Self :: STORY_ENTER_THE_NETHER) , "story/follow_ender_eye" => Some (& Self :: STORY_FOLLOW_ENDER_EYE) , "story/form_obsidian" => Some (& Self :: STORY_FORM_OBSIDIAN) , "story/iron_tools" => Some (& Self :: STORY_IRON_TOOLS) , "story/lava_bucket" => Some (& Self :: STORY_LAVA_BUCKET) , "story/mine_diamond" => Some (& Self :: STORY_MINE_DIAMOND) , "story/mine_stone" => Some (& Self :: STORY_MINE_STONE) , "story/obtain_armor" => Some (& Self :: STORY_OBTAIN_ARMOR) , "story/root" => Some (& Self :: STORY_ROOT) , "story/shiny_gear" => Some (& Self :: STORY_SHINY_GEAR) , "story/smelt_iron" => Some (& Self :: STORY_SMELT_IRON) , "story/upgrade_tools" => Some (& Self :: STORY_UPGRADE_TOOLS) , _ => None }
    }
}

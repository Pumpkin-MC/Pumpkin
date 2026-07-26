pub static ADVANCEMENT_TREE: LazyLock<AdvancementTree> = LazyLock::new(|| {
    let mut nodes = BTreeMap::new();
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/adventuring_time"),
        108usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/arbalistic"),
        1646usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/avoid_vibration"),
        109usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/blowback"),
        1647usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/brush_armadillo"),
        110usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/bullseye"),
        1648usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "adventure/craft_decorated_pot_using_only_sherds",
        ),
        111usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/crafters_crafting_crafters"),
        112usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/fall_from_world_height"),
        113usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/heart_transplanter"),
        114usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/hero_of_the_village"),
        115usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/honey_block_slide"),
        116usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/kill_a_mob"),
        117usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/kill_all_mobs"),
        118usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/kill_mob_near_sculk_catalyst"),
        119usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/lighten_up"),
        1649usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/lightning_rod_with_villager_no_fire"),
        120usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/minecraft_trials_edition"),
        121usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/ol_betsy"),
        122usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/overoverkill"),
        123usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/play_jukebox_in_meadows"),
        124usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/read_power_of_chiseled_bookshelf"),
        125usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/revaulting"),
        1650usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/root"),
        0usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/salvage_sherd"),
        1usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/shoot_arrow"),
        126usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/sleep_in_bed"),
        2usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/sniper_duel"),
        127usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/spear_many_mobs"),
        128usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/spyglass_at_dragon"),
        1651usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/spyglass_at_ghast"),
        129usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/spyglass_at_parrot"),
        3usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/summon_iron_golem"),
        130usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/throw_trident"),
        131usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/totem_of_undying"),
        132usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/trade"),
        4usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/trade_at_world_height"),
        5usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "adventure/trim_with_all_exclusive_armor_patterns",
        ),
        133usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/trim_with_any_armor_pattern"),
        6usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/two_birds_one_arrow"),
        134usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/under_lock_and_key"),
        135usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/use_lodestone"),
        7usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/very_very_frightening"),
        136usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/voluntary_exile"),
        8usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "adventure/walk_on_powder_snow_with_leather_boots",
        ),
        9usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/who_needs_rockets"),
        137usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "adventure/whos_the_pillager_now"),
        138usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/dragon_breath"),
        1652usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/dragon_egg"),
        1653usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/elytra"),
        1673usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/enter_end_gateway"),
        1654usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/find_end_city"),
        1655usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/kill_dragon"),
        139usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/levitate"),
        1656usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "end/respawn_dragon"),
        140usize,
    );
    nodes.insert(Identifier::from_static("minecraft", "end/root"), 10usize);
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/allay_deliver_cake_to_note_block"),
        1657usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/allay_deliver_item_to_player"),
        141usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/axolotl_in_a_bucket"),
        1658usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/balanced_diet"),
        1659usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/bred_all_animals"),
        1660usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/breed_an_animal"),
        142usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/complete_catalogue"),
        143usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/feed_snifflet"),
        1661usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/fishy_business"),
        144usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/froglights"),
        1662usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/kill_axolotl_target"),
        1663usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/leash_all_frog_variants"),
        145usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/make_a_sign_glow"),
        146usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/obtain_netherite_hoe"),
        1664usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/obtain_sniffer_egg"),
        147usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/place_dried_ghast_in_water"),
        148usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/plant_any_sniffer_seed"),
        1665usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/plant_seed"),
        149usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/remove_wolf_armor"),
        150usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/repair_wolf_armor"),
        151usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/ride_a_boat_with_a_goat"),
        152usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/root"),
        11usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/safely_harvest_honey"),
        12usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/silk_touch_nest"),
        13usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/tactical_fishing"),
        153usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/tadpole_in_a_bucket"),
        14usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/tame_an_animal"),
        15usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/uh_oh"),
        16usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/wax_off"),
        154usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/wax_on"),
        17usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "husbandry/whole_pack"),
        18usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/all_effects"),
        1680usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/all_potions"),
        1674usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/brew_potion"),
        1666usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/charge_respawn_anchor"),
        1667usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/create_beacon"),
        1668usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/create_full_beacon"),
        1669usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/distract_piglin"),
        155usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/explore_nether"),
        1670usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/fast_travel"),
        156usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/find_bastion"),
        157usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/find_fortress"),
        158usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/get_wither_skull"),
        159usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/loot_bastion"),
        160usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/netherite_armor"),
        1671usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/obtain_ancient_debris"),
        161usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/obtain_blaze_rod"),
        162usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/obtain_crying_obsidian"),
        163usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/return_to_sender"),
        164usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/ride_strider"),
        165usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/ride_strider_in_overworld_lava"),
        166usize,
    );
    nodes.insert(Identifier::from_static("minecraft", "nether/root"), 19usize);
    nodes.insert(
        Identifier::from_static("minecraft", "nether/summon_wither"),
        167usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "nether/uneasy_alliance"),
        168usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/blaze_powder"),
        169usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/brewing_stand"),
        170usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/cauldron"),
        171usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/fermented_spider_eye"),
        172usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/glass_bottle"),
        173usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/glistering_melon_slice"),
        174usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/golden_carrot"),
        175usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/brewing/magma_cream"),
        176usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/acacia_planks"),
        177usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/acacia_slab"),
        178usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/acacia_stairs"),
        179usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/acacia_wood"),
        180usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/amethyst_block"),
        181usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/andesite"),
        182usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/andesite_slab"),
        183usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/andesite_slab_from_andesite_stonecutting",
        ),
        184usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/andesite_stairs"),
        185usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/andesite_stairs_from_andesite_stonecutting",
        ),
        186usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bamboo_block"),
        187usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bamboo_mosaic_slab"),
        188usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bamboo_mosaic_stairs"),
        189usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bamboo_planks"),
        190usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bamboo_slab"),
        191usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bamboo_stairs"),
        192usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/birch_planks"),
        193usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/birch_slab"),
        194usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/birch_stairs"),
        195usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/birch_wood"),
        196usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/black_concrete_powder"),
        197usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/black_stained_glass"),
        198usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/black_terracotta"),
        199usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/blackstone_slab"),
        200usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/blackstone_slab_from_blackstone_stonecutting",
        ),
        201usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/blackstone_stairs"),
        202usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/blackstone_stairs_from_blackstone_stonecutting",
        ),
        203usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/blue_concrete_powder"),
        204usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/blue_ice"),
        205usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/blue_stained_glass"),
        206usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/blue_terracotta"),
        207usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bone_block"),
        208usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bookshelf"),
        209usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/brick_slab"),
        210usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/brick_slab_from_bricks_stonecutting",
        ),
        211usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/brick_stairs"),
        212usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/brick_stairs_from_bricks_stonecutting",
        ),
        213usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/bricks"),
        214usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/brown_concrete_powder"),
        215usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/brown_stained_glass"),
        216usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/brown_terracotta"),
        217usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cherry_planks"),
        218usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cherry_slab"),
        219usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cherry_stairs"),
        220usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cherry_wood"),
        221usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_bookshelf"),
        222usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_cinnabar"),
        223usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_cinnabar_from_cinnabar_stonecutting",
        ),
        224usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_copper"),
        225usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_copper_from_copper_block_stonecutting",
        ),
        226usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_copper_from_cut_copper_stonecutting",
        ),
        227usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_deepslate"),
        228usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_deepslate_from_cobbled_deepslate_stonecutting",
        ),
        229usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_deepslate_from_deepslate_stonecutting",
        ),
        230usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_nether_bricks",
        ),
        231usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_nether_bricks_from_nether_bricks_stonecutting",
        ),
        232usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_polished_blackstone",
        ),
        233usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_polished_blackstone_from_blackstone_stonecutting",
        ),
        234usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/chiseled_polished_blackstone_from_polished_blackstone_stonecutting") , 235usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_quartz_block"),
        236usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_quartz_block_from_quartz_block_stonecutting",
        ),
        237usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_red_sandstone",
        ),
        238usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_red_sandstone_from_red_sandstone_stonecutting",
        ),
        239usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_resin_bricks"),
        240usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_resin_bricks_from_resin_bricks_stonecutting",
        ),
        241usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_sandstone"),
        242usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_sandstone_from_sandstone_stonecutting",
        ),
        243usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_stone_bricks"),
        244usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_stone_bricks_from_stone_bricks_stonecutting",
        ),
        245usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_stone_bricks_from_stone_stonecutting",
        ),
        246usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_sulfur"),
        247usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_sulfur_from_sulfur_stonecutting",
        ),
        248usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_tuff"),
        249usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/chiseled_tuff_bricks"),
        250usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_tuff_bricks_from_polished_tuff_stonecutting",
        ),
        251usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_bricks_stonecutting",
        ),
        252usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_tuff_bricks_from_tuff_stonecutting",
        ),
        253usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/chiseled_tuff_from_tuff_stonecutting",
        ),
        254usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cinnabar_brick_slab"),
        255usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_brick_slab_from_cinnabar_bricks_stonecutting",
        ),
        256usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_brick_slab_from_cinnabar_stonecutting",
        ),
        257usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_brick_slab_from_polished_cinnabar_stonecutting",
        ),
        258usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cinnabar_brick_stairs"),
        259usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_brick_stairs_from_cinnabar_bricks_stonecutting",
        ),
        260usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_brick_stairs_from_cinnabar_stonecutting",
        ),
        261usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_brick_stairs_from_polished_cinnabar_stonecutting",
        ),
        262usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cinnabar_bricks"),
        263usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_bricks_from_cinnabar_stonecutting",
        ),
        264usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_bricks_from_polished_cinnabar_stonecutting",
        ),
        265usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cinnabar_slab"),
        266usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_slab_from_cinnabar_stonecutting",
        ),
        267usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cinnabar_stairs"),
        268usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cinnabar_stairs_from_cinnabar_stonecutting",
        ),
        269usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/clay"),
        270usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/coal_block"),
        271usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/coarse_dirt"),
        272usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_from_deepslate_stonecutting",
        ),
        273usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_slab",
        ),
        274usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_slab_from_cobbled_deepslate_stonecutting",
        ),
        275usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_slab_from_deepslate_stonecutting",
        ),
        276usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_stairs",
        ),
        277usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_stairs_from_cobbled_deepslate_stonecutting",
        ),
        278usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobbled_deepslate_stairs_from_deepslate_stonecutting",
        ),
        279usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobblestone_from_stone_stonecutting",
        ),
        280usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cobblestone_slab"),
        281usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobblestone_slab_from_cobblestone_stonecutting",
        ),
        282usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobblestone_slab_from_stone_stonecutting",
        ),
        283usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cobblestone_stairs"),
        284usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobblestone_stairs_from_cobblestone_stonecutting",
        ),
        285usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cobblestone_stairs_from_stone_stonecutting",
        ),
        286usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/copper_block"),
        287usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/copper_grate"),
        288usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/copper_grate_from_copper_block_stonecutting",
        ),
        289usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cracked_deepslate_bricks",
        ),
        290usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cracked_deepslate_tiles",
        ),
        291usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cracked_nether_bricks"),
        292usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cracked_polished_blackstone_bricks",
        ),
        293usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cracked_stone_bricks"),
        294usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/crimson_hyphae"),
        295usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/crimson_planks"),
        296usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/crimson_slab"),
        297usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/crimson_stairs"),
        298usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cut_copper"),
        299usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_copper_from_copper_block_stonecutting",
        ),
        300usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cut_copper_slab"),
        301usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_copper_slab_from_copper_block_stonecutting",
        ),
        302usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_copper_slab_from_cut_copper_stonecutting",
        ),
        303usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cut_copper_stairs"),
        304usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_copper_stairs_from_copper_block_stonecutting",
        ),
        305usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_copper_stairs_from_cut_copper_stonecutting",
        ),
        306usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cut_red_sandstone"),
        307usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_red_sandstone_from_red_sandstone_stonecutting",
        ),
        308usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_red_sandstone_slab",
        ),
        309usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_red_sandstone_slab_from_cut_red_sandstone_stonecutting",
        ),
        310usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_red_sandstone_slab_from_red_sandstone_stonecutting",
        ),
        311usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cut_sandstone"),
        312usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_sandstone_from_sandstone_stonecutting",
        ),
        313usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cut_sandstone_slab"),
        314usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_sandstone_slab_from_cut_sandstone_stonecutting",
        ),
        315usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/cut_sandstone_slab_from_sandstone_stonecutting",
        ),
        316usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cyan_concrete_powder"),
        317usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cyan_stained_glass"),
        318usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/cyan_terracotta"),
        319usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dark_oak_planks"),
        320usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dark_oak_slab"),
        321usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dark_oak_stairs"),
        322usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dark_oak_wood"),
        323usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dark_prismarine"),
        324usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dark_prismarine_slab"),
        325usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/dark_prismarine_slab_from_dark_prismarine_stonecutting",
        ),
        326usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/dark_prismarine_stairs",
        ),
        327usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/dark_prismarine_stairs_from_dark_prismarine_stonecutting",
        ),
        328usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/deepslate"),
        329usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/deepslate_brick_slab"),
        330usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_slab_from_cobbled_deepslate_stonecutting",
        ),
        331usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_slab_from_deepslate_bricks_stonecutting",
        ),
        332usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_slab_from_deepslate_stonecutting",
        ),
        333usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_slab_from_polished_deepslate_stonecutting",
        ),
        334usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_stairs",
        ),
        335usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_stairs_from_cobbled_deepslate_stonecutting",
        ),
        336usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_stairs_from_deepslate_bricks_stonecutting",
        ),
        337usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_stairs_from_deepslate_stonecutting",
        ),
        338usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_brick_stairs_from_polished_deepslate_stonecutting",
        ),
        339usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/deepslate_bricks"),
        340usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_bricks_from_cobbled_deepslate_stonecutting",
        ),
        341usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_bricks_from_deepslate_stonecutting",
        ),
        342usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_bricks_from_polished_deepslate_stonecutting",
        ),
        343usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/deepslate_tile_slab"),
        344usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_slab_from_cobbled_deepslate_stonecutting",
        ),
        345usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_slab_from_deepslate_bricks_stonecutting",
        ),
        346usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_slab_from_deepslate_stonecutting",
        ),
        347usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_slab_from_deepslate_tiles_stonecutting",
        ),
        348usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_slab_from_polished_deepslate_stonecutting",
        ),
        349usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/deepslate_tile_stairs"),
        350usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_stairs_from_cobbled_deepslate_stonecutting",
        ),
        351usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_bricks_stonecutting",
        ),
        352usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_stonecutting",
        ),
        353usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_stairs_from_deepslate_tiles_stonecutting",
        ),
        354usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tile_stairs_from_polished_deepslate_stonecutting",
        ),
        355usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/deepslate_tiles"),
        356usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tiles_from_cobbled_deepslate_stonecutting",
        ),
        357usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tiles_from_deepslate_bricks_stonecutting",
        ),
        358usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tiles_from_deepslate_stonecutting",
        ),
        359usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/deepslate_tiles_from_polished_deepslate_stonecutting",
        ),
        360usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/diamond_block"),
        361usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/diorite"),
        362usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/diorite_slab"),
        363usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/diorite_slab_from_diorite_stonecutting",
        ),
        364usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/diorite_stairs"),
        365usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/diorite_stairs_from_diorite_stonecutting",
        ),
        366usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dried_ghast"),
        367usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dried_kelp_block"),
        368usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dripstone_block"),
        369usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_black_wool"),
        370usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_blue_wool"),
        371usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_brown_wool"),
        372usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_cyan_wool"),
        373usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_gray_wool"),
        374usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_green_wool"),
        375usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_light_blue_wool"),
        376usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_light_gray_wool"),
        377usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_lime_wool"),
        378usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_magenta_wool"),
        379usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_orange_wool"),
        380usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_pink_wool"),
        381usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_purple_wool"),
        382usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_red_wool"),
        383usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_white_wool"),
        384usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/dye_yellow_wool"),
        385usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/emerald_block"),
        386usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/end_stone_brick_slab"),
        387usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/end_stone_brick_slab_from_end_stone_bricks_stonecutting",
        ),
        388usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/end_stone_brick_slab_from_end_stone_stonecutting",
        ),
        389usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/end_stone_brick_stairs",
        ),
        390usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_bricks_stonecutting",
        ),
        391usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/end_stone_brick_stairs_from_end_stone_stonecutting",
        ),
        392usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/end_stone_bricks"),
        393usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/end_stone_bricks_from_end_stone_stonecutting",
        ),
        394usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_chiseled_copper",
        ),
        395usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_chiseled_copper_from_exposed_copper_stonecutting",
        ),
        396usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_chiseled_copper_from_exposed_cut_copper_stonecutting",
        ),
        397usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/exposed_copper_grate"),
        398usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_copper_grate_from_exposed_copper_stonecutting",
        ),
        399usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/exposed_cut_copper"),
        400usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_cut_copper_from_exposed_copper_stonecutting",
        ),
        401usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_cut_copper_slab",
        ),
        402usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_copper_stonecutting",
        ),
        403usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_cut_copper_slab_from_exposed_cut_copper_stonecutting",
        ),
        404usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_cut_copper_stairs",
        ),
        405usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_copper_stonecutting",
        ),
        406usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/exposed_cut_copper_stairs_from_exposed_cut_copper_stonecutting") , 407usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/glass"),
        408usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/glowstone"),
        409usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/gold_block"),
        410usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/granite"),
        411usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/granite_slab"),
        412usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/granite_slab_from_granite_stonecutting",
        ),
        413usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/granite_stairs"),
        414usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/granite_stairs_from_granite_stonecutting",
        ),
        415usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/gray_concrete_powder"),
        416usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/gray_stained_glass"),
        417usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/gray_terracotta"),
        418usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/green_concrete_powder"),
        419usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/green_stained_glass"),
        420usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/green_terracotta"),
        421usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/hay_block"),
        422usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/iron_block"),
        423usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/jack_o_lantern"),
        424usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/jungle_planks"),
        425usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/jungle_slab"),
        426usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/jungle_stairs"),
        427usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/jungle_wood"),
        428usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/lapis_block"),
        429usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/light_blue_concrete_powder",
        ),
        430usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/light_blue_stained_glass",
        ),
        431usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/light_blue_terracotta"),
        432usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/light_gray_concrete_powder",
        ),
        433usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/light_gray_stained_glass",
        ),
        434usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/light_gray_terracotta"),
        435usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/lime_concrete_powder"),
        436usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/lime_stained_glass"),
        437usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/lime_terracotta"),
        438usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/magenta_concrete_powder",
        ),
        439usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/magenta_stained_glass"),
        440usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/magenta_terracotta"),
        441usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/magma_block"),
        442usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mangrove_planks"),
        443usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mangrove_slab"),
        444usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mangrove_stairs"),
        445usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mangrove_wood"),
        446usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/melon"),
        447usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_cobblestone_from_moss_block",
        ),
        448usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_cobblestone_from_vine",
        ),
        449usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_cobblestone_slab",
        ),
        450usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_cobblestone_slab_from_mossy_cobblestone_stonecutting",
        ),
        451usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_cobblestone_stairs",
        ),
        452usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_cobblestone_stairs_from_mossy_cobblestone_stonecutting",
        ),
        453usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_stone_brick_slab",
        ),
        454usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_stone_brick_slab_from_mossy_stone_bricks_stonecutting",
        ),
        455usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_stone_brick_stairs",
        ),
        456usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_stone_brick_stairs_from_mossy_stone_bricks_stonecutting",
        ),
        457usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_stone_bricks_from_moss_block",
        ),
        458usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mossy_stone_bricks_from_vine",
        ),
        459usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mud_brick_slab"),
        460usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mud_brick_slab_from_mud_bricks_stonecutting",
        ),
        461usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mud_brick_stairs"),
        462usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/mud_brick_stairs_from_mud_bricks_stonecutting",
        ),
        463usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/mud_bricks"),
        464usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/muddy_mangrove_roots"),
        465usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/nether_brick_slab"),
        466usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/nether_brick_slab_from_nether_bricks_stonecutting",
        ),
        467usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/nether_brick_stairs"),
        468usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/nether_brick_stairs_from_nether_bricks_stonecutting",
        ),
        469usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/nether_bricks"),
        470usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/nether_wart_block"),
        471usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/netherite_block"),
        472usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/oak_planks"),
        473usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/oak_slab"),
        474usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/oak_stairs"),
        475usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/oak_wood"),
        476usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/orange_concrete_powder",
        ),
        477usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/orange_stained_glass"),
        478usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/orange_terracotta"),
        479usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_chiseled_copper",
        ),
        480usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_copper_stonecutting",
        ),
        481usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/oxidized_chiseled_copper_from_oxidized_cut_copper_stonecutting") , 482usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/oxidized_copper_grate"),
        483usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_copper_grate_from_oxidized_copper_stonecutting",
        ),
        484usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/oxidized_cut_copper"),
        485usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_cut_copper_from_oxidized_copper_stonecutting",
        ),
        486usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_cut_copper_slab",
        ),
        487usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_copper_stonecutting",
        ),
        488usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/oxidized_cut_copper_slab_from_oxidized_cut_copper_stonecutting") , 489usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_cut_copper_stairs",
        ),
        490usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_copper_stonecutting",
        ),
        491usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/oxidized_cut_copper_stairs_from_oxidized_cut_copper_stonecutting") , 492usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/packed_ice"),
        493usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/packed_mud"),
        494usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pale_oak_planks"),
        495usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pale_oak_slab"),
        496usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pale_oak_stairs"),
        497usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pale_oak_wood"),
        498usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pink_concrete_powder"),
        499usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pink_stained_glass"),
        500usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/pink_terracotta"),
        501usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_andesite"),
        502usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_from_andesite_stonecutting",
        ),
        503usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_slab",
        ),
        504usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_slab_from_andesite_stonecutting",
        ),
        505usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_slab_from_polished_andesite_stonecutting",
        ),
        506usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_stairs",
        ),
        507usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_stairs_from_andesite_stonecutting",
        ),
        508usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_andesite_stairs_from_polished_andesite_stonecutting",
        ),
        509usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_basalt"),
        510usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_basalt_from_basalt_stonecutting",
        ),
        511usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_blackstone"),
        512usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_brick_slab",
        ),
        513usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_brick_slab_from_blackstone_stonecutting",
        ),
        514usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_bricks_stonecutting") , 515usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_brick_slab_from_polished_blackstone_stonecutting") , 516usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_brick_stairs",
        ),
        517usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_brick_stairs_from_blackstone_stonecutting",
        ),
        518usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_bricks_stonecutting") , 519usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_brick_stairs_from_polished_blackstone_stonecutting") , 520usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_bricks",
        ),
        521usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_bricks_from_blackstone_stonecutting",
        ),
        522usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_bricks_from_polished_blackstone_stonecutting") , 523usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_from_blackstone_stonecutting",
        ),
        524usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_slab",
        ),
        525usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_slab_from_blackstone_stonecutting",
        ),
        526usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_slab_from_polished_blackstone_stonecutting") , 527usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_stairs",
        ),
        528usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_blackstone_stairs_from_blackstone_stonecutting",
        ),
        529usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_blackstone_stairs_from_polished_blackstone_stonecutting") , 530usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_cinnabar"),
        531usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_from_cinnabar_stonecutting",
        ),
        532usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_slab",
        ),
        533usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_slab_from_cinnabar_stonecutting",
        ),
        534usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_slab_from_polished_cinnabar_stonecutting",
        ),
        535usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_stairs",
        ),
        536usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_stairs_from_cinnabar_stonecutting",
        ),
        537usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_cinnabar_stairs_from_polished_cinnabar_stonecutting",
        ),
        538usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_deepslate"),
        539usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_from_cobbled_deepslate_stonecutting",
        ),
        540usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_from_deepslate_stonecutting",
        ),
        541usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_slab",
        ),
        542usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_slab_from_cobbled_deepslate_stonecutting",
        ),
        543usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_slab_from_deepslate_stonecutting",
        ),
        544usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_slab_from_polished_deepslate_stonecutting",
        ),
        545usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_stairs",
        ),
        546usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_stairs_from_cobbled_deepslate_stonecutting",
        ),
        547usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_deepslate_stairs_from_deepslate_stonecutting",
        ),
        548usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/polished_deepslate_stairs_from_polished_deepslate_stonecutting") , 549usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_diorite"),
        550usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_diorite_from_diorite_stonecutting",
        ),
        551usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_diorite_slab"),
        552usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_diorite_slab_from_diorite_stonecutting",
        ),
        553usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_diorite_slab_from_polished_diorite_stonecutting",
        ),
        554usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_diorite_stairs",
        ),
        555usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_diorite_stairs_from_diorite_stonecutting",
        ),
        556usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_diorite_stairs_from_polished_diorite_stonecutting",
        ),
        557usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_granite"),
        558usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_granite_from_granite_stonecutting",
        ),
        559usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_granite_slab"),
        560usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_granite_slab_from_granite_stonecutting",
        ),
        561usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_granite_slab_from_polished_granite_stonecutting",
        ),
        562usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_granite_stairs",
        ),
        563usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_granite_stairs_from_granite_stonecutting",
        ),
        564usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_granite_stairs_from_polished_granite_stonecutting",
        ),
        565usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_sulfur"),
        566usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_sulfur_from_sulfur_stonecutting",
        ),
        567usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_sulfur_slab"),
        568usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_sulfur_slab_from_polished_sulfur_stonecutting",
        ),
        569usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_sulfur_slab_from_sulfur_stonecutting",
        ),
        570usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_sulfur_stairs",
        ),
        571usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_sulfur_stairs_from_polished_sulfur_stonecutting",
        ),
        572usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_sulfur_stairs_from_sulfur_stonecutting",
        ),
        573usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_tuff"),
        574usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_tuff_from_tuff_stonecutting",
        ),
        575usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_tuff_slab"),
        576usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_tuff_slab_from_polished_tuff_stonecutting",
        ),
        577usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_tuff_slab_from_tuff_stonecutting",
        ),
        578usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/polished_tuff_stairs"),
        579usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_tuff_stairs_from_polished_tuff_stonecutting",
        ),
        580usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/polished_tuff_stairs_from_tuff_stonecutting",
        ),
        581usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/potent_sulfur"),
        582usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/prismarine"),
        583usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/prismarine_brick_slab"),
        584usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/prismarine_brick_slab_from_prismarine_bricks_stonecutting",
        ),
        585usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/prismarine_brick_stairs",
        ),
        586usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/prismarine_brick_stairs_from_prismarine_bricks_stonecutting",
        ),
        587usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/prismarine_bricks"),
        588usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/prismarine_slab"),
        589usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/prismarine_slab_from_prismarine_stonecutting",
        ),
        590usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/prismarine_stairs"),
        591usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/prismarine_stairs_from_prismarine_stonecutting",
        ),
        592usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/purple_concrete_powder",
        ),
        593usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/purple_stained_glass"),
        594usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/purple_terracotta"),
        595usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/purpur_block"),
        596usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/purpur_pillar"),
        597usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/purpur_pillar_from_purpur_block_stonecutting",
        ),
        598usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/purpur_slab"),
        599usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/purpur_slab_from_purpur_block_stonecutting",
        ),
        600usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/purpur_stairs"),
        601usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/purpur_stairs_from_purpur_block_stonecutting",
        ),
        602usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/quartz_block"),
        603usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/quartz_bricks"),
        604usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/quartz_bricks_from_quartz_block_stonecutting",
        ),
        605usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/quartz_pillar"),
        606usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/quartz_pillar_from_quartz_block_stonecutting",
        ),
        607usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/quartz_slab"),
        608usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/quartz_slab_from_quartz_block_stonecutting",
        ),
        609usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/quartz_stairs"),
        610usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/quartz_stairs_from_quartz_block_stonecutting",
        ),
        611usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/raw_copper_block"),
        612usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/raw_gold_block"),
        613usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/raw_iron_block"),
        614usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_concrete_powder"),
        615usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_nether_brick_slab"),
        616usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/red_nether_brick_slab_from_red_nether_bricks_stonecutting",
        ),
        617usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/red_nether_brick_stairs",
        ),
        618usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/red_nether_brick_stairs_from_red_nether_bricks_stonecutting",
        ),
        619usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_nether_bricks"),
        620usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_sandstone"),
        621usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_sandstone_slab"),
        622usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/red_sandstone_slab_from_red_sandstone_stonecutting",
        ),
        623usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_sandstone_stairs"),
        624usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/red_sandstone_stairs_from_red_sandstone_stonecutting",
        ),
        625usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_stained_glass"),
        626usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/red_terracotta"),
        627usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/resin_block"),
        628usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/resin_brick_slab"),
        629usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/resin_brick_slab_from_resin_bricks_stonecutting",
        ),
        630usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/resin_brick_stairs"),
        631usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/resin_brick_stairs_from_resin_bricks_stonecutting",
        ),
        632usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/resin_bricks"),
        633usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sandstone"),
        634usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sandstone_slab"),
        635usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sandstone_slab_from_sandstone_stonecutting",
        ),
        636usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sandstone_stairs"),
        637usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sandstone_stairs_from_sandstone_stonecutting",
        ),
        638usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sea_lantern"),
        639usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_basalt"),
        640usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_quartz"),
        641usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_quartz_slab"),
        642usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_quartz_slab_from_smooth_quartz_stonecutting",
        ),
        643usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_quartz_stairs"),
        644usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_quartz_stairs_from_smooth_quartz_stonecutting",
        ),
        645usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_red_sandstone"),
        646usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_red_sandstone_slab",
        ),
        647usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/smooth_red_sandstone_slab_from_smooth_red_sandstone_stonecutting") , 648usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_red_sandstone_stairs",
        ),
        649usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/smooth_red_sandstone_stairs_from_smooth_red_sandstone_stonecutting") , 650usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_sandstone"),
        651usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_sandstone_slab"),
        652usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_sandstone_slab_from_smooth_sandstone_stonecutting",
        ),
        653usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_sandstone_stairs",
        ),
        654usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_sandstone_stairs_from_smooth_sandstone_stonecutting",
        ),
        655usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_stone"),
        656usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/smooth_stone_slab"),
        657usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/smooth_stone_slab_from_smooth_stone_stonecutting",
        ),
        658usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/snow_block"),
        659usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sponge"),
        660usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/spruce_planks"),
        661usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/spruce_slab"),
        662usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/spruce_stairs"),
        663usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/spruce_wood"),
        664usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stone"),
        665usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stone_brick_slab"),
        666usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_brick_slab_from_stone_bricks_stonecutting",
        ),
        667usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_brick_slab_from_stone_stonecutting",
        ),
        668usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stone_brick_stairs"),
        669usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_brick_stairs_from_stone_bricks_stonecutting",
        ),
        670usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_brick_stairs_from_stone_stonecutting",
        ),
        671usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stone_bricks"),
        672usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_bricks_from_stone_stonecutting",
        ),
        673usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stone_slab"),
        674usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_slab_from_stone_stonecutting",
        ),
        675usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stone_stairs"),
        676usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stone_stairs_from_stone_stonecutting",
        ),
        677usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stripped_acacia_wood"),
        678usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stripped_birch_wood"),
        679usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stripped_cherry_wood"),
        680usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stripped_crimson_hyphae",
        ),
        681usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stripped_dark_oak_wood",
        ),
        682usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stripped_jungle_wood"),
        683usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stripped_mangrove_wood",
        ),
        684usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stripped_oak_wood"),
        685usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stripped_pale_oak_wood",
        ),
        686usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/stripped_spruce_wood"),
        687usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/stripped_warped_hyphae",
        ),
        688usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sulfur_brick_slab"),
        689usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_brick_slab_from_polished_sulfur_stonecutting",
        ),
        690usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_brick_slab_from_sulfur_bricks_stonecutting",
        ),
        691usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_brick_slab_from_sulfur_stonecutting",
        ),
        692usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sulfur_brick_stairs"),
        693usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_brick_stairs_from_polished_sulfur_stonecutting",
        ),
        694usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_brick_stairs_from_sulfur_bricks_stonecutting",
        ),
        695usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_brick_stairs_from_sulfur_stonecutting",
        ),
        696usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sulfur_bricks"),
        697usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_bricks_from_polished_sulfur_stonecutting",
        ),
        698usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_bricks_from_sulfur_stonecutting",
        ),
        699usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_from_sulfur_spikes",
        ),
        700usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sulfur_slab"),
        701usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_slab_from_sulfur_stonecutting",
        ),
        702usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/sulfur_stairs"),
        703usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/sulfur_stairs_from_sulfur_stonecutting",
        ),
        704usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/terracotta"),
        705usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/tinted_glass"),
        706usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/tuff_brick_slab"),
        707usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_brick_slab_from_polished_tuff_stonecutting",
        ),
        708usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_brick_slab_from_tuff_bricks_stonecutting",
        ),
        709usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_brick_slab_from_tuff_stonecutting",
        ),
        710usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/tuff_brick_stairs"),
        711usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_brick_stairs_from_polished_tuff_stonecutting",
        ),
        712usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_brick_stairs_from_tuff_bricks_stonecutting",
        ),
        713usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_brick_stairs_from_tuff_stonecutting",
        ),
        714usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/tuff_bricks"),
        715usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_bricks_from_polished_tuff_stonecutting",
        ),
        716usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_bricks_from_tuff_stonecutting",
        ),
        717usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/tuff_slab"),
        718usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_slab_from_tuff_stonecutting",
        ),
        719usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/tuff_stairs"),
        720usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/tuff_stairs_from_tuff_stonecutting",
        ),
        721usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/warped_hyphae"),
        722usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/warped_planks"),
        723usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/warped_slab"),
        724usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/warped_stairs"),
        725usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/waxed_chiseled_copper"),
        726usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_chiseled_copper_from_honeycomb",
        ),
        727usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_chiseled_copper_from_waxed_copper_block_stonecutting",
        ),
        728usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_chiseled_copper_from_waxed_cut_copper_stonecutting",
        ),
        729usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_bars_from_honeycomb",
        ),
        730usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_block_from_honeycomb",
        ),
        731usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_chain_from_honeycomb",
        ),
        732usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_chest_from_honeycomb",
        ),
        733usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_golem_statue_from_honeycomb",
        ),
        734usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/waxed_copper_grate"),
        735usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_grate_from_honeycomb",
        ),
        736usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_grate_from_waxed_copper_block_stonecutting",
        ),
        737usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_copper_lantern_from_honeycomb",
        ),
        738usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/waxed_cut_copper"),
        739usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_from_honeycomb",
        ),
        740usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_from_waxed_copper_block_stonecutting",
        ),
        741usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/waxed_cut_copper_slab"),
        742usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_slab_from_honeycomb",
        ),
        743usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_copper_block_stonecutting",
        ),
        744usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_slab_from_waxed_cut_copper_stonecutting",
        ),
        745usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_stairs",
        ),
        746usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_stairs_from_honeycomb",
        ),
        747usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting",
        ),
        748usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting",
        ),
        749usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_chiseled_copper",
        ),
        750usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_chiseled_copper_from_honeycomb",
        ),
        751usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_copper_stonecutting") , 752usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_chiseled_copper_from_waxed_exposed_cut_copper_stonecutting") , 753usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_bars_from_honeycomb",
        ),
        754usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_chain_from_honeycomb",
        ),
        755usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_chest_from_honeycomb",
        ),
        756usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_from_honeycomb",
        ),
        757usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_golem_statue_from_honeycomb",
        ),
        758usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_grate",
        ),
        759usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_grate_from_honeycomb",
        ),
        760usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_copper_grate_from_waxed_exposed_copper_stonecutting") , 761usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_copper_lantern_from_honeycomb",
        ),
        762usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_cut_copper",
        ),
        763usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_cut_copper_from_honeycomb",
        ),
        764usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting") , 765usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_cut_copper_slab",
        ),
        766usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_honeycomb",
        ),
        767usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_copper_stonecutting") , 768usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_cut_copper_slab_from_waxed_exposed_cut_copper_stonecutting") , 769usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_cut_copper_stairs",
        ),
        770usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_honeycomb",
        ),
        771usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_copper_stonecutting") , 772usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_exposed_cut_copper_stairs_from_waxed_exposed_cut_copper_stonecutting") , 773usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_exposed_lightning_rod_from_honeycomb",
        ),
        774usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_lightning_rod_from_honeycomb",
        ),
        775usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_chiseled_copper",
        ),
        776usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_honeycomb",
        ),
        777usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_copper_stonecutting") , 778usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_chiseled_copper_from_waxed_oxidized_cut_copper_stonecutting") , 779usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_bars_from_honeycomb",
        ),
        780usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_chain_from_honeycomb",
        ),
        781usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_chest_from_honeycomb",
        ),
        782usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_from_honeycomb",
        ),
        783usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_golem_statue_from_honeycomb",
        ),
        784usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_grate",
        ),
        785usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_grate_from_honeycomb",
        ),
        786usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_copper_grate_from_waxed_oxidized_copper_stonecutting") , 787usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_copper_lantern_from_honeycomb",
        ),
        788usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_cut_copper",
        ),
        789usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_cut_copper_from_honeycomb",
        ),
        790usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_cut_copper_from_waxed_oxidized_copper_stonecutting") , 791usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_cut_copper_slab",
        ),
        792usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_honeycomb",
        ),
        793usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_copper_stonecutting") , 794usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_cut_copper_slab_from_waxed_oxidized_cut_copper_stonecutting") , 795usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_cut_copper_stairs",
        ),
        796usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_honeycomb",
        ),
        797usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_copper_stonecutting") , 798usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_cut_copper_stonecutting") , 799usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_oxidized_lightning_rod_from_honeycomb",
        ),
        800usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_chiseled_copper",
        ),
        801usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_chiseled_copper_from_honeycomb",
        ),
        802usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_copper_stonecutting") , 803usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_chiseled_copper_from_waxed_weathered_cut_copper_stonecutting") , 804usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_bars_from_honeycomb",
        ),
        805usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_chain_from_honeycomb",
        ),
        806usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_chest_from_honeycomb",
        ),
        807usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_from_honeycomb",
        ),
        808usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_golem_statue_from_honeycomb",
        ),
        809usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_grate",
        ),
        810usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_grate_from_honeycomb",
        ),
        811usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_copper_grate_from_waxed_weathered_copper_stonecutting") , 812usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_copper_lantern_from_honeycomb",
        ),
        813usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_cut_copper",
        ),
        814usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_cut_copper_from_honeycomb",
        ),
        815usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_cut_copper_from_waxed_weathered_copper_stonecutting") , 816usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_cut_copper_slab",
        ),
        817usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_honeycomb",
        ),
        818usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_copper_stonecutting") , 819usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_cut_copper_slab_from_waxed_weathered_cut_copper_stonecutting") , 820usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_cut_copper_stairs",
        ),
        821usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_honeycomb",
        ),
        822usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_copper_stonecutting") , 823usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/waxed_weathered_cut_copper_stairs_from_waxed_weathered_cut_copper_stonecutting") , 824usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/waxed_weathered_lightning_rod_from_honeycomb",
        ),
        825usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_chiseled_copper",
        ),
        826usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_chiseled_copper_from_weathered_copper_stonecutting",
        ),
        827usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/weathered_chiseled_copper_from_weathered_cut_copper_stonecutting") , 828usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_copper_grate",
        ),
        829usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_copper_grate_from_weathered_copper_stonecutting",
        ),
        830usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/weathered_cut_copper"),
        831usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_cut_copper_from_weathered_copper_stonecutting",
        ),
        832usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_cut_copper_slab",
        ),
        833usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_copper_stonecutting",
        ),
        834usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/weathered_cut_copper_slab_from_weathered_cut_copper_stonecutting") , 835usize) ;
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/weathered_cut_copper_stairs",
        ),
        836usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_copper_stonecutting") , 837usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/building_blocks/weathered_cut_copper_stairs_from_weathered_cut_copper_stonecutting") , 838usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/white_concrete_powder"),
        839usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/white_stained_glass"),
        840usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/white_terracotta"),
        841usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/white_wool_from_string",
        ),
        842usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/building_blocks/yellow_concrete_powder",
        ),
        843usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/yellow_stained_glass"),
        844usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/building_blocks/yellow_terracotta"),
        845usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/arrow"),
        846usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/black_harness"),
        847usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/blue_harness"),
        848usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/bow"),
        849usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/brown_harness"),
        850usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/copper_boots"),
        851usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/copper_chestplate"),
        852usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/copper_helmet"),
        853usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/copper_leggings"),
        854usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/copper_spear"),
        855usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/copper_sword"),
        856usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/crossbow"),
        857usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/cyan_harness"),
        858usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/diamond_boots"),
        859usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/diamond_chestplate"),
        860usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/diamond_helmet"),
        861usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/diamond_leggings"),
        862usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/diamond_spear"),
        863usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/diamond_sword"),
        864usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_black_harness"),
        865usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_blue_harness"),
        866usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_brown_harness"),
        867usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_cyan_harness"),
        868usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_gray_harness"),
        869usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_green_harness"),
        870usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_light_blue_harness"),
        871usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_light_gray_harness"),
        872usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_lime_harness"),
        873usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_magenta_harness"),
        874usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_orange_harness"),
        875usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_pink_harness"),
        876usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_purple_harness"),
        877usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_red_harness"),
        878usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_white_harness"),
        879usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/dye_yellow_harness"),
        880usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/golden_boots"),
        881usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/golden_chestplate"),
        882usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/golden_helmet"),
        883usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/golden_leggings"),
        884usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/golden_spear"),
        885usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/golden_sword"),
        886usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/gray_harness"),
        887usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/green_harness"),
        888usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/iron_boots"),
        889usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/iron_chestplate"),
        890usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/iron_helmet"),
        891usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/iron_leggings"),
        892usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/iron_spear"),
        893usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/iron_sword"),
        894usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/leather_boots"),
        895usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/leather_chestplate"),
        896usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/leather_helmet"),
        897usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/leather_leggings"),
        898usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/light_blue_harness"),
        899usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/light_gray_harness"),
        900usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/lime_harness"),
        901usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/mace"),
        902usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/magenta_harness"),
        903usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_boots_smithing"),
        904usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_chestplate_smithing"),
        905usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_helmet_smithing"),
        906usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_horse_armor_smithing"),
        907usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_leggings_smithing"),
        908usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/combat/netherite_nautilus_armor_smithing",
        ),
        909usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_spear_smithing"),
        910usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/netherite_sword_smithing"),
        911usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/orange_harness"),
        912usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/pink_harness"),
        913usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/purple_harness"),
        914usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/red_harness"),
        915usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/saddle"),
        916usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/shield"),
        917usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/spectral_arrow"),
        918usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/stone_spear"),
        919usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/stone_sword"),
        920usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/turtle_helmet"),
        921usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/white_harness"),
        922usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/wolf_armor"),
        923usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/wooden_spear"),
        924usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/wooden_sword"),
        925usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/combat/yellow_harness"),
        926usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/acacia_fence"),
        927usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/acacia_hanging_sign"),
        928usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/acacia_shelf"),
        929usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/acacia_sign"),
        930usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/andesite_wall"),
        931usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/andesite_wall_from_andesite_stonecutting",
        ),
        932usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/anvil"),
        933usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/armor_stand"),
        934usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/bamboo_fence"),
        935usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/bamboo_hanging_sign"),
        936usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/bamboo_mosaic"),
        937usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/bamboo_shelf"),
        938usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/bamboo_sign"),
        939usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/barrel"),
        940usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/beehive"),
        941usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/birch_fence"),
        942usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/birch_hanging_sign"),
        943usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/birch_shelf"),
        944usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/birch_sign"),
        945usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_banner"),
        946usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_bed"),
        947usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_candle"),
        948usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_carpet"),
        949usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_glazed_terracotta"),
        950usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_shulker_box"),
        951usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/black_stained_glass_pane"),
        952usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/black_stained_glass_pane_from_glass_pane",
        ),
        953usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blackstone_wall"),
        954usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/blackstone_wall_from_blackstone_stonecutting",
        ),
        955usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blast_furnace"),
        956usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_banner"),
        957usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_bed"),
        958usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_candle"),
        959usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_carpet"),
        960usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_glazed_terracotta"),
        961usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_shulker_box"),
        962usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/blue_stained_glass_pane"),
        963usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/blue_stained_glass_pane_from_glass_pane",
        ),
        964usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brick_wall"),
        965usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/brick_wall_from_bricks_stonecutting",
        ),
        966usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_banner"),
        967usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_bed"),
        968usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_candle"),
        969usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_carpet"),
        970usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_glazed_terracotta"),
        971usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_shulker_box"),
        972usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/brown_stained_glass_pane"),
        973usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/brown_stained_glass_pane_from_glass_pane",
        ),
        974usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/campfire"),
        975usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/candle"),
        976usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cartography_table"),
        977usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cherry_fence"),
        978usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cherry_hanging_sign"),
        979usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cherry_shelf"),
        980usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cherry_sign"),
        981usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/chest"),
        982usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cinnabar_brick_wall"),
        983usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cinnabar_brick_wall_from_cinnabar_bricks_stonecutting",
        ),
        984usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cinnabar_brick_wall_from_cinnabar_stonecutting",
        ),
        985usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cinnabar_brick_wall_from_polished_cinnabar_stonecutting",
        ),
        986usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cinnabar_wall"),
        987usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cinnabar_wall_from_cinnabar_stonecutting",
        ),
        988usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cobbled_deepslate_wall"),
        989usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cobbled_deepslate_wall_from_cobbled_deepslate_stonecutting",
        ),
        990usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cobbled_deepslate_wall_from_deepslate_stonecutting",
        ),
        991usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cobblestone_wall"),
        992usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cobblestone_wall_from_cobblestone_stonecutting",
        ),
        993usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cobblestone_wall_from_stone_stonecutting",
        ),
        994usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/composter"),
        995usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/copper_bars"),
        996usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/copper_chain"),
        997usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/copper_chest"),
        998usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/copper_lantern"),
        999usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/copper_torch"),
        1000usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/crafting_table"),
        1001usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/crimson_fence"),
        1002usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/crimson_hanging_sign"),
        1003usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/crimson_shelf"),
        1004usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/crimson_sign"),
        1005usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_banner"),
        1006usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_bed"),
        1007usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_candle"),
        1008usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_carpet"),
        1009usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_glazed_terracotta"),
        1010usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_shulker_box"),
        1011usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/cyan_stained_glass_pane"),
        1012usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/cyan_stained_glass_pane_from_glass_pane",
        ),
        1013usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dark_oak_fence"),
        1014usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dark_oak_hanging_sign"),
        1015usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dark_oak_shelf"),
        1016usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dark_oak_sign"),
        1017usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/decorated_pot_simple"),
        1018usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/deepslate_brick_wall"),
        1019usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_brick_wall_from_cobbled_deepslate_stonecutting",
        ),
        1020usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_brick_wall_from_deepslate_bricks_stonecutting",
        ),
        1021usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_brick_wall_from_deepslate_stonecutting",
        ),
        1022usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_brick_wall_from_polished_deepslate_stonecutting",
        ),
        1023usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/deepslate_tile_wall"),
        1024usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_tile_wall_from_cobbled_deepslate_stonecutting",
        ),
        1025usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_tile_wall_from_deepslate_bricks_stonecutting",
        ),
        1026usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_tile_wall_from_deepslate_stonecutting",
        ),
        1027usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_tile_wall_from_deepslate_tiles_stonecutting",
        ),
        1028usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/deepslate_tile_wall_from_polished_deepslate_stonecutting",
        ),
        1029usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/diorite_wall"),
        1030usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/diorite_wall_from_diorite_stonecutting",
        ),
        1031usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_black_bed"),
        1032usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_black_carpet"),
        1033usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_blue_bed"),
        1034usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_blue_carpet"),
        1035usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_brown_bed"),
        1036usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_brown_carpet"),
        1037usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_cyan_bed"),
        1038usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_cyan_carpet"),
        1039usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_gray_bed"),
        1040usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_gray_carpet"),
        1041usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_green_bed"),
        1042usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_green_carpet"),
        1043usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_light_blue_bed"),
        1044usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_light_blue_carpet"),
        1045usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_light_gray_bed"),
        1046usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_light_gray_carpet"),
        1047usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_lime_bed"),
        1048usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_lime_carpet"),
        1049usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_magenta_bed"),
        1050usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_magenta_carpet"),
        1051usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_orange_bed"),
        1052usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_orange_carpet"),
        1053usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_pink_bed"),
        1054usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_pink_carpet"),
        1055usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_purple_bed"),
        1056usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_purple_carpet"),
        1057usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_red_bed"),
        1058usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_red_carpet"),
        1059usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_white_bed"),
        1060usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_white_carpet"),
        1061usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_yellow_bed"),
        1062usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/dye_yellow_carpet"),
        1063usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/enchanting_table"),
        1064usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/end_crystal"),
        1065usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/end_rod"),
        1066usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/end_stone_brick_wall"),
        1067usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/end_stone_brick_wall_from_end_stone_bricks_stonecutting",
        ),
        1068usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/end_stone_brick_wall_from_end_stone_stonecutting",
        ),
        1069usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/ender_chest"),
        1070usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/fletching_table"),
        1071usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/flower_pot"),
        1072usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/furnace"),
        1073usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/glass_pane"),
        1074usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/glow_item_frame"),
        1075usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/golden_dandelion"),
        1076usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/granite_wall"),
        1077usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/granite_wall_from_granite_stonecutting",
        ),
        1078usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_banner"),
        1079usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_bed"),
        1080usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_candle"),
        1081usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_carpet"),
        1082usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_glazed_terracotta"),
        1083usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_shulker_box"),
        1084usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/gray_stained_glass_pane"),
        1085usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/gray_stained_glass_pane_from_glass_pane",
        ),
        1086usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_banner"),
        1087usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_bed"),
        1088usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_candle"),
        1089usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_carpet"),
        1090usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_glazed_terracotta"),
        1091usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_shulker_box"),
        1092usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/green_stained_glass_pane"),
        1093usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/green_stained_glass_pane_from_glass_pane",
        ),
        1094usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/grindstone"),
        1095usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/honeycomb_block"),
        1096usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/iron_bars"),
        1097usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/iron_chain"),
        1098usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/item_frame"),
        1099usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/jukebox"),
        1100usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/jungle_fence"),
        1101usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/jungle_hanging_sign"),
        1102usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/jungle_shelf"),
        1103usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/jungle_sign"),
        1104usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/ladder"),
        1105usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lantern"),
        1106usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_blue_banner"),
        1107usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_blue_bed"),
        1108usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_blue_candle"),
        1109usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_blue_carpet"),
        1110usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/light_blue_glazed_terracotta",
        ),
        1111usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_blue_shulker_box"),
        1112usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/light_blue_stained_glass_pane",
        ),
        1113usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/light_blue_stained_glass_pane_from_glass_pane",
        ),
        1114usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_gray_banner"),
        1115usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_gray_bed"),
        1116usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_gray_candle"),
        1117usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_gray_carpet"),
        1118usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/light_gray_glazed_terracotta",
        ),
        1119usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/light_gray_shulker_box"),
        1120usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/light_gray_stained_glass_pane",
        ),
        1121usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/light_gray_stained_glass_pane_from_glass_pane",
        ),
        1122usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_banner"),
        1123usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_bed"),
        1124usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_candle"),
        1125usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_carpet"),
        1126usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_glazed_terracotta"),
        1127usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_shulker_box"),
        1128usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lime_stained_glass_pane"),
        1129usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/lime_stained_glass_pane_from_glass_pane",
        ),
        1130usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/lodestone"),
        1131usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/loom"),
        1132usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/magenta_banner"),
        1133usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/magenta_bed"),
        1134usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/magenta_candle"),
        1135usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/magenta_carpet"),
        1136usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/magenta_glazed_terracotta"),
        1137usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/magenta_shulker_box"),
        1138usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/magenta_stained_glass_pane",
        ),
        1139usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/magenta_stained_glass_pane_from_glass_pane",
        ),
        1140usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mangrove_fence"),
        1141usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mangrove_hanging_sign"),
        1142usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mangrove_shelf"),
        1143usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mangrove_sign"),
        1144usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/moss_carpet"),
        1145usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mossy_cobblestone_wall"),
        1146usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/mossy_cobblestone_wall_from_mossy_cobblestone_stonecutting",
        ),
        1147usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mossy_stone_brick_wall"),
        1148usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/mossy_stone_brick_wall_from_mossy_stone_bricks_stonecutting",
        ),
        1149usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/mud_brick_wall"),
        1150usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/mud_brick_wall_from_mud_bricks_stonecutting",
        ),
        1151usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/nether_brick_fence"),
        1152usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/nether_brick_wall"),
        1153usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/nether_brick_wall_from_nether_bricks_stonecutting",
        ),
        1154usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/oak_fence"),
        1155usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/oak_hanging_sign"),
        1156usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/oak_shelf"),
        1157usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/oak_sign"),
        1158usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_banner"),
        1159usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_bed"),
        1160usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_candle"),
        1161usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_carpet"),
        1162usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_glazed_terracotta"),
        1163usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_shulker_box"),
        1164usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/orange_stained_glass_pane"),
        1165usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/orange_stained_glass_pane_from_glass_pane",
        ),
        1166usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/painting"),
        1167usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pale_moss_carpet"),
        1168usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pale_oak_fence"),
        1169usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pale_oak_hanging_sign"),
        1170usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pale_oak_shelf"),
        1171usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pale_oak_sign"),
        1172usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_banner"),
        1173usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_bed"),
        1174usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_candle"),
        1175usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_carpet"),
        1176usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_glazed_terracotta"),
        1177usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_shulker_box"),
        1178usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/pink_stained_glass_pane"),
        1179usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/pink_stained_glass_pane_from_glass_pane",
        ),
        1180usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_blackstone_brick_wall",
        ),
        1181usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_blackstone_brick_wall_from_blackstone_stonecutting",
        ),
        1182usize,
    );
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_bricks_stonecutting") , 1183usize) ;
    nodes . insert (Identifier :: from_static ("minecraft" , "recipes/decorations/polished_blackstone_brick_wall_from_polished_blackstone_stonecutting") , 1184usize) ;
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/polished_blackstone_wall"),
        1185usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_blackstone_wall_from_blackstone_stonecutting",
        ),
        1186usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_blackstone_wall_from_polished_blackstone_stonecutting",
        ),
        1187usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/polished_cinnabar_wall"),
        1188usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_cinnabar_wall_from_cinnabar_stonecutting",
        ),
        1189usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_cinnabar_wall_from_polished_cinnabar_stonecutting",
        ),
        1190usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/polished_deepslate_wall"),
        1191usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_deepslate_wall_from_cobbled_deepslate_stonecutting",
        ),
        1192usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_deepslate_wall_from_deepslate_stonecutting",
        ),
        1193usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_deepslate_wall_from_polished_deepslate_stonecutting",
        ),
        1194usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/polished_sulfur_wall"),
        1195usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_sulfur_wall_from_polished_sulfur_stonecutting",
        ),
        1196usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_sulfur_wall_from_sulfur_stonecutting",
        ),
        1197usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/polished_tuff_wall"),
        1198usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_tuff_wall_from_polished_tuff_stonecutting",
        ),
        1199usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/polished_tuff_wall_from_tuff_stonecutting",
        ),
        1200usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/prismarine_wall"),
        1201usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/prismarine_wall_from_prismarine_stonecutting",
        ),
        1202usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_banner"),
        1203usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_bed"),
        1204usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_candle"),
        1205usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_carpet"),
        1206usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_glazed_terracotta"),
        1207usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_shulker_box"),
        1208usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/purple_stained_glass_pane"),
        1209usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/purple_stained_glass_pane_from_glass_pane",
        ),
        1210usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_banner"),
        1211usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_bed"),
        1212usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_candle"),
        1213usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_carpet"),
        1214usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_glazed_terracotta"),
        1215usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_nether_brick_wall"),
        1216usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/red_nether_brick_wall_from_red_nether_bricks_stonecutting",
        ),
        1217usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_sandstone_wall"),
        1218usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/red_sandstone_wall_from_red_sandstone_stonecutting",
        ),
        1219usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_shulker_box"),
        1220usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/red_stained_glass_pane"),
        1221usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/red_stained_glass_pane_from_glass_pane",
        ),
        1222usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/resin_brick_wall"),
        1223usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/resin_brick_wall_from_resin_bricks_stonecutting",
        ),
        1224usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/respawn_anchor"),
        1225usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/sandstone_wall"),
        1226usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/sandstone_wall_from_sandstone_stonecutting",
        ),
        1227usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/scaffolding"),
        1228usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/shulker_box"),
        1229usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/smithing_table"),
        1230usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/smoker"),
        1231usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/snow"),
        1232usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/soul_campfire"),
        1233usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/soul_lantern"),
        1234usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/soul_torch"),
        1235usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/spruce_fence"),
        1236usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/spruce_hanging_sign"),
        1237usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/spruce_shelf"),
        1238usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/spruce_sign"),
        1239usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/stone_brick_wall"),
        1240usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/stone_brick_wall_from_stone_bricks_stonecutting",
        ),
        1241usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/stone_brick_wall_from_stone_stonecutting",
        ),
        1242usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/stonecutter"),
        1243usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/sulfur_brick_wall"),
        1244usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/sulfur_brick_wall_from_polished_sulfur_stonecutting",
        ),
        1245usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/sulfur_brick_wall_from_sulfur_bricks_stonecutting",
        ),
        1246usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/sulfur_brick_wall_from_sulfur_stonecutting",
        ),
        1247usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/sulfur_wall"),
        1248usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/sulfur_wall_from_sulfur_stonecutting",
        ),
        1249usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/torch"),
        1250usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/tuff_brick_wall"),
        1251usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/tuff_brick_wall_from_polished_tuff_stonecutting",
        ),
        1252usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/tuff_brick_wall_from_tuff_bricks_stonecutting",
        ),
        1253usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/tuff_brick_wall_from_tuff_stonecutting",
        ),
        1254usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/tuff_wall"),
        1255usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/tuff_wall_from_tuff_stonecutting",
        ),
        1256usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/warped_fence"),
        1257usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/warped_hanging_sign"),
        1258usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/warped_shelf"),
        1259usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/warped_sign"),
        1260usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_banner"),
        1261usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_bed"),
        1262usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_candle"),
        1263usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_carpet"),
        1264usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_glazed_terracotta"),
        1265usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_shulker_box"),
        1266usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/white_stained_glass_pane"),
        1267usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/white_stained_glass_pane_from_glass_pane",
        ),
        1268usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_banner"),
        1269usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_bed"),
        1270usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_candle"),
        1271usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_carpet"),
        1272usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_glazed_terracotta"),
        1273usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_shulker_box"),
        1274usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/decorations/yellow_stained_glass_pane"),
        1275usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/decorations/yellow_stained_glass_pane_from_glass_pane",
        ),
        1276usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/baked_potato"),
        1277usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/baked_potato_from_campfire_cooking",
        ),
        1278usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/baked_potato_from_smoking"),
        1279usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/beetroot_soup"),
        1280usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/bread"),
        1281usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cake"),
        1282usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_beef"),
        1283usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/cooked_beef_from_campfire_cooking",
        ),
        1284usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_beef_from_smoking"),
        1285usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_chicken"),
        1286usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/cooked_chicken_from_campfire_cooking",
        ),
        1287usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_chicken_from_smoking"),
        1288usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_cod"),
        1289usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_cod_from_campfire_cooking"),
        1290usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_cod_from_smoking"),
        1291usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_mutton"),
        1292usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/cooked_mutton_from_campfire_cooking",
        ),
        1293usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_mutton_from_smoking"),
        1294usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_porkchop"),
        1295usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/cooked_porkchop_from_campfire_cooking",
        ),
        1296usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_porkchop_from_smoking"),
        1297usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_rabbit"),
        1298usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/cooked_rabbit_from_campfire_cooking",
        ),
        1299usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_rabbit_from_smoking"),
        1300usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_salmon"),
        1301usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/cooked_salmon_from_campfire_cooking",
        ),
        1302usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cooked_salmon_from_smoking"),
        1303usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/cookie"),
        1304usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/dried_kelp"),
        1305usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/dried_kelp_from_campfire_cooking"),
        1306usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/dried_kelp_from_smelting"),
        1307usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/dried_kelp_from_smoking"),
        1308usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/golden_apple"),
        1309usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/honey_bottle"),
        1310usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/mushroom_stew"),
        1311usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/pumpkin_pie"),
        1312usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/rabbit_stew_from_brown_mushroom"),
        1313usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/rabbit_stew_from_red_mushroom"),
        1314usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_allium"),
        1315usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_azure_bluet"),
        1316usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_blue_orchid"),
        1317usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/suspicious_stew_from_closed_eyeblossom",
        ),
        1318usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_cornflower"),
        1319usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_dandelion"),
        1320usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/suspicious_stew_from_golden_dandelion",
        ),
        1321usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/suspicious_stew_from_lily_of_the_valley",
        ),
        1322usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/suspicious_stew_from_open_eyeblossom",
        ),
        1323usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/food/suspicious_stew_from_orange_tulip",
        ),
        1324usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_oxeye_daisy"),
        1325usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_pink_tulip"),
        1326usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_poppy"),
        1327usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_red_tulip"),
        1328usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_torchflower"),
        1329usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_white_tulip"),
        1330usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/food/suspicious_stew_from_wither_rose"),
        1331usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/beacon"),
        1332usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/black_dye"),
        1333usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/black_dye_from_wither_rose"),
        1334usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/blue_dye"),
        1335usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/blue_dye_from_cornflower"),
        1336usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/bolt_armor_trim_smithing_template",
        ),
        1337usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/bolt_armor_trim_smithing_template_smithing_trim",
        ),
        1338usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/bone_meal"),
        1339usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/bone_meal_from_bone_block"),
        1340usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/book"),
        1341usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/bordure_indented_banner_pattern"),
        1342usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/bowl"),
        1343usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/brick"),
        1344usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/brown_dye"),
        1345usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/bucket"),
        1346usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/charcoal"),
        1347usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/coal"),
        1348usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/coal_from_blasting_coal_ore"),
        1349usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/coal_from_blasting_deepslate_coal_ore",
        ),
        1350usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/coal_from_smelting_coal_ore"),
        1351usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/coal_from_smelting_deepslate_coal_ore",
        ),
        1352usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/coast_armor_trim_smithing_template",
        ),
        1353usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/coast_armor_trim_smithing_template_smithing_trim",
        ),
        1354usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/conduit"),
        1355usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/copper_ingot"),
        1356usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_blasting_copper_ore",
        ),
        1357usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_blasting_deepslate_copper_ore",
        ),
        1358usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_blasting_raw_copper",
        ),
        1359usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/copper_ingot_from_nuggets"),
        1360usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_smelting_copper_ore",
        ),
        1361usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_smelting_deepslate_copper_ore",
        ),
        1362usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_smelting_raw_copper",
        ),
        1363usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/copper_ingot_from_waxed_copper_block",
        ),
        1364usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/copper_nugget"),
        1365usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/copper_nugget_from_blasting"),
        1366usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/copper_nugget_from_smelting"),
        1367usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/creaking_heart"),
        1368usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/creeper_banner_pattern"),
        1369usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/cyan_dye"),
        1370usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/cyan_dye_from_pitcher_plant"),
        1371usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/diamond"),
        1372usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/diamond_from_blasting_deepslate_diamond_ore",
        ),
        1373usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/diamond_from_blasting_diamond_ore",
        ),
        1374usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/diamond_from_smelting_deepslate_diamond_ore",
        ),
        1375usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/diamond_from_smelting_diamond_ore",
        ),
        1376usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/dune_armor_trim_smithing_template",
        ),
        1377usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/dune_armor_trim_smithing_template_smithing_trim",
        ),
        1378usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/emerald"),
        1379usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/emerald_from_blasting_deepslate_emerald_ore",
        ),
        1380usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/emerald_from_blasting_emerald_ore",
        ),
        1381usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/emerald_from_smelting_deepslate_emerald_ore",
        ),
        1382usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/emerald_from_smelting_emerald_ore",
        ),
        1383usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/ender_eye"),
        1384usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/eye_armor_trim_smithing_template"),
        1385usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/eye_armor_trim_smithing_template_smithing_trim",
        ),
        1386usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/field_masoned_banner_pattern"),
        1387usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/fire_charge"),
        1388usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/firework_rocket_simple"),
        1389usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/flow_armor_trim_smithing_template",
        ),
        1390usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/flow_armor_trim_smithing_template_smithing_trim",
        ),
        1391usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/flower_banner_pattern"),
        1392usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_blasting_deepslate_gold_ore",
        ),
        1393usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_blasting_gold_ore",
        ),
        1394usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_blasting_nether_gold_ore",
        ),
        1395usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_blasting_raw_gold",
        ),
        1396usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gold_ingot_from_gold_block"),
        1397usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gold_ingot_from_nuggets"),
        1398usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_smelting_deepslate_gold_ore",
        ),
        1399usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_smelting_gold_ore",
        ),
        1400usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_smelting_nether_gold_ore",
        ),
        1401usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/gold_ingot_from_smelting_raw_gold",
        ),
        1402usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gold_nugget"),
        1403usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gold_nugget_from_blasting"),
        1404usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gold_nugget_from_smelting"),
        1405usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gray_dye"),
        1406usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/gray_dye_from_closed_eyeblossom"),
        1407usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/green_dye"),
        1408usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/host_armor_trim_smithing_template",
        ),
        1409usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/host_armor_trim_smithing_template_smithing_trim",
        ),
        1410usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/iron_ingot_from_blasting_deepslate_iron_ore",
        ),
        1411usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/iron_ingot_from_blasting_iron_ore",
        ),
        1412usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/iron_ingot_from_blasting_raw_iron",
        ),
        1413usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/iron_ingot_from_iron_block"),
        1414usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/iron_ingot_from_nuggets"),
        1415usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/iron_ingot_from_smelting_deepslate_iron_ore",
        ),
        1416usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/iron_ingot_from_smelting_iron_ore",
        ),
        1417usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/iron_ingot_from_smelting_raw_iron",
        ),
        1418usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/iron_nugget"),
        1419usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/iron_nugget_from_blasting"),
        1420usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/iron_nugget_from_smelting"),
        1421usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/lapis_lazuli"),
        1422usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/lapis_lazuli_from_blasting_deepslate_lapis_ore",
        ),
        1423usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/lapis_lazuli_from_blasting_lapis_ore",
        ),
        1424usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/lapis_lazuli_from_smelting_deepslate_lapis_ore",
        ),
        1425usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/lapis_lazuli_from_smelting_lapis_ore",
        ),
        1426usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leaf_litter"),
        1427usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather"),
        1428usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather_boots_dyed"),
        1429usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather_chestplate_dyed"),
        1430usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather_helmet_dyed"),
        1431usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather_horse_armor"),
        1432usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather_horse_armor_dyed"),
        1433usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/leather_leggings_dyed"),
        1434usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/light_blue_dye_from_blue_orchid"),
        1435usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/light_blue_dye_from_blue_white_dye",
        ),
        1436usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/light_gray_dye_from_azure_bluet"),
        1437usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/light_gray_dye_from_black_white_dye",
        ),
        1438usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/light_gray_dye_from_gray_white_dye",
        ),
        1439usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/light_gray_dye_from_oxeye_daisy"),
        1440usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/light_gray_dye_from_white_tulip"),
        1441usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/lime_dye"),
        1442usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/lime_dye_from_smelting"),
        1443usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/magenta_dye_from_allium"),
        1444usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/magenta_dye_from_blue_red_pink"),
        1445usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/magenta_dye_from_blue_red_white_dye",
        ),
        1446usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/magenta_dye_from_lilac"),
        1447usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/magenta_dye_from_purple_and_pink"),
        1448usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/map"),
        1449usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/map_cloning"),
        1450usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/melon_seeds"),
        1451usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/mojang_banner_pattern"),
        1452usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/music_disc_5"),
        1453usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/nether_brick"),
        1454usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/netherite_ingot"),
        1455usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/netherite_ingot_from_netherite_block",
        ),
        1456usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/netherite_scrap"),
        1457usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/netherite_scrap_from_blasting"),
        1458usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/netherite_upgrade_smithing_template",
        ),
        1459usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/orange_dye_from_open_eyeblossom"),
        1460usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/orange_dye_from_orange_tulip"),
        1461usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/orange_dye_from_red_yellow"),
        1462usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/orange_dye_from_torchflower"),
        1463usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/paper"),
        1464usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/pink_dye_from_cactus_flower"),
        1465usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/pink_dye_from_peony"),
        1466usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/pink_dye_from_pink_petals"),
        1467usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/pink_dye_from_pink_tulip"),
        1468usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/pink_dye_from_red_white_dye"),
        1469usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/popped_chorus_fruit"),
        1470usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/pumpkin_seeds"),
        1471usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/purple_dye"),
        1472usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/quartz"),
        1473usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/quartz_from_blasting"),
        1474usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/raiser_armor_trim_smithing_template",
        ),
        1475usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/raiser_armor_trim_smithing_template_smithing_trim",
        ),
        1476usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/raw_copper"),
        1477usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/raw_gold"),
        1478usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/raw_iron"),
        1479usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/red_dye_from_beetroot"),
        1480usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/red_dye_from_poppy"),
        1481usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/red_dye_from_rose_bush"),
        1482usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/red_dye_from_tulip"),
        1483usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/resin_brick"),
        1484usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/resin_clump"),
        1485usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/rib_armor_trim_smithing_template"),
        1486usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/rib_armor_trim_smithing_template_smithing_trim",
        ),
        1487usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/sentry_armor_trim_smithing_template",
        ),
        1488usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/sentry_armor_trim_smithing_template_smithing_trim",
        ),
        1489usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/shaper_armor_trim_smithing_template",
        ),
        1490usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/shaper_armor_trim_smithing_template_smithing_trim",
        ),
        1491usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/silence_armor_trim_smithing_template",
        ),
        1492usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/silence_armor_trim_smithing_template_smithing_trim",
        ),
        1493usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/skull_banner_pattern"),
        1494usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/slime_ball"),
        1495usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/snout_armor_trim_smithing_template",
        ),
        1496usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/snout_armor_trim_smithing_template_smithing_trim",
        ),
        1497usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/spire_armor_trim_smithing_template",
        ),
        1498usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/spire_armor_trim_smithing_template_smithing_trim",
        ),
        1499usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/stick"),
        1500usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/stick_from_bamboo_item"),
        1501usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/sugar_from_honey_bottle"),
        1502usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/sugar_from_sugar_cane"),
        1503usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/tide_armor_trim_smithing_template",
        ),
        1504usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/tide_armor_trim_smithing_template_smithing_trim",
        ),
        1505usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/tipped_arrow"),
        1506usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/vex_armor_trim_smithing_template"),
        1507usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/vex_armor_trim_smithing_template_smithing_trim",
        ),
        1508usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/ward_armor_trim_smithing_template",
        ),
        1509usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/ward_armor_trim_smithing_template_smithing_trim",
        ),
        1510usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/wayfinder_armor_trim_smithing_template",
        ),
        1511usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/wayfinder_armor_trim_smithing_template_smithing_trim",
        ),
        1512usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/wheat"),
        1513usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/white_dye"),
        1514usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/white_dye_from_lily_of_the_valley",
        ),
        1515usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/wild_armor_trim_smithing_template",
        ),
        1516usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/misc/wild_armor_trim_smithing_template_smithing_trim",
        ),
        1517usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/wind_charge"),
        1518usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/wolf_armor_dyed"),
        1519usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/writable_book"),
        1520usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/yellow_dye_from_dandelion"),
        1521usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/yellow_dye_from_golden_dandelion"),
        1522usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/yellow_dye_from_sunflower"),
        1523usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/misc/yellow_dye_from_wildflowers"),
        1524usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/acacia_button"),
        1525usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/acacia_door"),
        1526usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/acacia_fence_gate"),
        1527usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/acacia_pressure_plate"),
        1528usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/acacia_trapdoor"),
        1529usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/bamboo_button"),
        1530usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/bamboo_door"),
        1531usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/bamboo_fence_gate"),
        1532usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/bamboo_pressure_plate"),
        1533usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/bamboo_trapdoor"),
        1534usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/birch_button"),
        1535usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/birch_door"),
        1536usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/birch_fence_gate"),
        1537usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/birch_pressure_plate"),
        1538usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/birch_trapdoor"),
        1539usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/calibrated_sculk_sensor"),
        1540usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/cherry_button"),
        1541usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/cherry_door"),
        1542usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/cherry_fence_gate"),
        1543usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/cherry_pressure_plate"),
        1544usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/cherry_trapdoor"),
        1545usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/comparator"),
        1546usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/copper_bulb"),
        1547usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/copper_door"),
        1548usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/copper_trapdoor"),
        1549usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/crafter"),
        1550usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/crimson_button"),
        1551usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/crimson_door"),
        1552usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/crimson_fence_gate"),
        1553usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/crimson_pressure_plate"),
        1554usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/crimson_trapdoor"),
        1555usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dark_oak_button"),
        1556usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dark_oak_door"),
        1557usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dark_oak_fence_gate"),
        1558usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dark_oak_pressure_plate"),
        1559usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dark_oak_trapdoor"),
        1560usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/daylight_detector"),
        1561usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dispenser"),
        1562usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/dropper"),
        1563usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/exposed_copper_bulb"),
        1564usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/heavy_weighted_pressure_plate",
        ),
        1565usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/honey_block"),
        1566usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/hopper"),
        1567usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/iron_door"),
        1568usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/iron_trapdoor"),
        1569usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/jungle_button"),
        1570usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/jungle_door"),
        1571usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/jungle_fence_gate"),
        1572usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/jungle_pressure_plate"),
        1573usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/jungle_trapdoor"),
        1574usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/lectern"),
        1575usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/lever"),
        1576usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/light_weighted_pressure_plate",
        ),
        1577usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/lightning_rod"),
        1578usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/mangrove_button"),
        1579usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/mangrove_door"),
        1580usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/mangrove_fence_gate"),
        1581usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/mangrove_pressure_plate"),
        1582usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/mangrove_trapdoor"),
        1583usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/note_block"),
        1584usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/oak_button"),
        1585usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/oak_door"),
        1586usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/oak_fence_gate"),
        1587usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/oak_pressure_plate"),
        1588usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/oak_trapdoor"),
        1589usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/observer"),
        1590usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/oxidized_copper_bulb"),
        1591usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/pale_oak_button"),
        1592usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/pale_oak_door"),
        1593usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/pale_oak_fence_gate"),
        1594usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/pale_oak_pressure_plate"),
        1595usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/pale_oak_trapdoor"),
        1596usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/piston"),
        1597usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/polished_blackstone_button"),
        1598usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/polished_blackstone_pressure_plate",
        ),
        1599usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/redstone"),
        1600usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/redstone_block"),
        1601usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/redstone_from_blasting_deepslate_redstone_ore",
        ),
        1602usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/redstone_from_blasting_redstone_ore",
        ),
        1603usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/redstone_from_smelting_deepslate_redstone_ore",
        ),
        1604usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/redstone_from_smelting_redstone_ore",
        ),
        1605usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/redstone_lamp"),
        1606usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/redstone_torch"),
        1607usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/repeater"),
        1608usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/slime_block"),
        1609usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/spruce_button"),
        1610usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/spruce_door"),
        1611usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/spruce_fence_gate"),
        1612usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/spruce_pressure_plate"),
        1613usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/spruce_trapdoor"),
        1614usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/sticky_piston"),
        1615usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/stone_button"),
        1616usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/stone_pressure_plate"),
        1617usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/target"),
        1618usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/tnt"),
        1619usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/trapped_chest"),
        1620usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/tripwire_hook"),
        1621usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/warped_button"),
        1622usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/warped_door"),
        1623usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/warped_fence_gate"),
        1624usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/warped_pressure_plate"),
        1625usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/warped_trapdoor"),
        1626usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/waxed_copper_bulb"),
        1627usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_copper_bulb_from_honeycomb",
        ),
        1628usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_copper_door_from_honeycomb",
        ),
        1629usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_copper_trapdoor_from_honeycomb",
        ),
        1630usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/waxed_exposed_copper_bulb"),
        1631usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_exposed_copper_bulb_from_honeycomb",
        ),
        1632usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_exposed_copper_door_from_honeycomb",
        ),
        1633usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_exposed_copper_trapdoor_from_honeycomb",
        ),
        1634usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/waxed_oxidized_copper_bulb"),
        1635usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_oxidized_copper_bulb_from_honeycomb",
        ),
        1636usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_oxidized_copper_door_from_honeycomb",
        ),
        1637usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_oxidized_copper_trapdoor_from_honeycomb",
        ),
        1638usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/waxed_weathered_copper_bulb"),
        1639usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_weathered_copper_bulb_from_honeycomb",
        ),
        1640usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_weathered_copper_door_from_honeycomb",
        ),
        1641usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/redstone/waxed_weathered_copper_trapdoor_from_honeycomb",
        ),
        1642usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/redstone/weathered_copper_bulb"),
        1643usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/root"),
        20usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/black_bundle"),
        21usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/blue_bundle"),
        22usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/brown_bundle"),
        23usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/brush"),
        24usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/bundle"),
        25usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/clock"),
        26usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/compass"),
        27usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/copper_axe"),
        28usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/copper_hoe"),
        29usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/copper_pickaxe"),
        30usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/copper_shovel"),
        31usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/cyan_bundle"),
        32usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/diamond_axe"),
        33usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/diamond_hoe"),
        34usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/diamond_pickaxe"),
        35usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/diamond_shovel"),
        36usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/fishing_rod"),
        37usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/flint_and_steel"),
        38usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/golden_axe"),
        39usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/golden_hoe"),
        40usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/golden_pickaxe"),
        41usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/golden_shovel"),
        42usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/gray_bundle"),
        43usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/green_bundle"),
        44usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/iron_axe"),
        45usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/iron_hoe"),
        46usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/iron_pickaxe"),
        47usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/iron_shovel"),
        48usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/lead"),
        49usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/light_blue_bundle"),
        50usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/light_gray_bundle"),
        51usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/lime_bundle"),
        52usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/magenta_bundle"),
        53usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/name_tag"),
        54usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/netherite_axe_smithing"),
        55usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/netherite_hoe_smithing"),
        56usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/netherite_pickaxe_smithing"),
        57usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/netherite_shovel_smithing"),
        58usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/orange_bundle"),
        59usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/pink_bundle"),
        60usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/purple_bundle"),
        61usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/recovery_compass"),
        62usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/red_bundle"),
        63usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/shears"),
        64usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/spyglass"),
        65usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/stone_axe"),
        66usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/stone_hoe"),
        67usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/stone_pickaxe"),
        68usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/stone_shovel"),
        69usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/white_bundle"),
        70usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/wooden_axe"),
        71usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/wooden_hoe"),
        72usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/wooden_pickaxe"),
        73usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/wooden_shovel"),
        74usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/tools/yellow_bundle"),
        75usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/acacia_boat"),
        76usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/acacia_chest_boat"),
        77usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/activator_rail"),
        78usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/bamboo_chest_raft"),
        79usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/bamboo_raft"),
        80usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/birch_boat"),
        81usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/birch_chest_boat"),
        82usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/carrot_on_a_stick"),
        83usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/cherry_boat"),
        84usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/cherry_chest_boat"),
        85usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/chest_minecart"),
        86usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/dark_oak_boat"),
        87usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/dark_oak_chest_boat"),
        88usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/detector_rail"),
        89usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/furnace_minecart"),
        90usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/hopper_minecart"),
        91usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/jungle_boat"),
        92usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/jungle_chest_boat"),
        93usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/mangrove_boat"),
        94usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/mangrove_chest_boat"),
        95usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/minecart"),
        96usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/oak_boat"),
        97usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/oak_chest_boat"),
        98usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/pale_oak_boat"),
        99usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/pale_oak_chest_boat"),
        100usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/powered_rail"),
        101usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/rail"),
        102usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/spruce_boat"),
        103usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/spruce_chest_boat"),
        104usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "recipes/transportation/tnt_minecart"),
        105usize,
    );
    nodes.insert(
        Identifier::from_static(
            "minecraft",
            "recipes/transportation/warped_fungus_on_a_stick",
        ),
        106usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/cure_zombie_villager"),
        1686usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/deflect_arrow"),
        1681usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/enchant_item"),
        1682usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/enter_the_end"),
        1687usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/enter_the_nether"),
        1684usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/follow_ender_eye"),
        1685usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/form_obsidian"),
        1683usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/iron_tools"),
        1675usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/lava_bucket"),
        1676usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/mine_diamond"),
        1677usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/mine_stone"),
        1644usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/obtain_armor"),
        1678usize,
    );
    nodes.insert(Identifier::from_static("minecraft", "story/root"), 107usize);
    nodes.insert(
        Identifier::from_static("minecraft", "story/shiny_gear"),
        1679usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/smelt_iron"),
        1672usize,
    );
    nodes.insert(
        Identifier::from_static("minecraft", "story/upgrade_tools"),
        1645usize,
    );
    let nodes_vector = vec ! [AdvancementNode { parent : None , children : vec ! [1usize , 2usize , 3usize , 4usize , 6usize , 7usize , 8usize , 109usize , 110usize , 112usize , 113usize , 114usize , 116usize , 117usize , 120usize , 121usize , 122usize , 125usize] , value : Advancement :: ADVENTURE_ROOT , } , AdvancementNode { parent : Some (0usize) , children : vec ! [111usize] , value : Advancement :: ADVENTURE_SALVAGE_SHERD , } , AdvancementNode { parent : Some (0usize) , children : vec ! [9usize , 108usize , 124usize] , value : Advancement :: ADVENTURE_SLEEP_IN_BED , } , AdvancementNode { parent : Some (0usize) , children : vec ! [129usize] , value : Advancement :: ADVENTURE_SPYGLASS_AT_PARROT , } , AdvancementNode { parent : Some (0usize) , children : vec ! [5usize , 130usize] , value : Advancement :: ADVENTURE_TRADE , } , AdvancementNode { parent : Some (4usize) , children : vec ! [] , value : Advancement :: ADVENTURE_TRADE_AT_WORLD_HEIGHT , } , AdvancementNode { parent : Some (0usize) , children : vec ! [133usize] , value : Advancement :: ADVENTURE_TRIM_WITH_ANY_ARMOR_PATTERN , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_USE_LODESTONE , } , AdvancementNode { parent : Some (0usize) , children : vec ! [115usize] , value : Advancement :: ADVENTURE_VOLUNTARY_EXILE , } , AdvancementNode { parent : Some (2usize) , children : vec ! [] , value : Advancement :: ADVENTURE_WALK_ON_POWDER_SNOW_WITH_LEATHER_BOOTS , } , AdvancementNode { parent : None , children : vec ! [139usize] , value : Advancement :: END_ROOT , } , AdvancementNode { parent : None , children : vec ! [12usize , 13usize , 14usize , 15usize , 16usize , 141usize , 142usize , 144usize , 146usize , 147usize , 148usize , 149usize , 152usize] , value : Advancement :: HUSBANDRY_ROOT , } , AdvancementNode { parent : Some (11usize) , children : vec ! [17usize] , value : Advancement :: HUSBANDRY_SAFELY_HARVEST_HONEY , } , AdvancementNode { parent : Some (11usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_SILK_TOUCH_NEST , } , AdvancementNode { parent : Some (11usize) , children : vec ! [145usize] , value : Advancement :: HUSBANDRY_TADPOLE_IN_A_BUCKET , } , AdvancementNode { parent : Some (11usize) , children : vec ! [18usize , 143usize , 150usize , 151usize] , value : Advancement :: HUSBANDRY_TAME_AN_ANIMAL , } , AdvancementNode { parent : Some (11usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_UH_OH , } , AdvancementNode { parent : Some (12usize) , children : vec ! [154usize] , value : Advancement :: HUSBANDRY_WAX_ON , } , AdvancementNode { parent : Some (15usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_WHOLE_PACK , } , AdvancementNode { parent : None , children : vec ! [155usize , 156usize , 157usize , 158usize , 161usize , 163usize , 164usize , 165usize] , value : Advancement :: NETHER_ROOT , } , AdvancementNode { parent : None , children : vec ! [21usize , 22usize , 23usize , 24usize , 25usize , 26usize , 27usize , 28usize , 29usize , 30usize , 31usize , 32usize , 33usize , 34usize , 35usize , 36usize , 37usize , 38usize , 39usize , 40usize , 41usize , 42usize , 43usize , 44usize , 45usize , 46usize , 47usize , 48usize , 49usize , 50usize , 51usize , 52usize , 53usize , 54usize , 55usize , 56usize , 57usize , 58usize , 59usize , 60usize , 61usize , 62usize , 63usize , 64usize , 65usize , 66usize , 67usize , 68usize , 69usize , 70usize , 71usize , 72usize , 73usize , 74usize , 75usize , 76usize , 77usize , 78usize , 79usize , 80usize , 81usize , 82usize , 83usize , 84usize , 85usize , 86usize , 87usize , 88usize , 89usize , 90usize , 91usize , 92usize , 93usize , 94usize , 95usize , 96usize , 97usize , 98usize , 99usize , 100usize , 101usize , 102usize , 103usize , 104usize , 105usize , 106usize , 169usize , 170usize , 171usize , 172usize , 173usize , 174usize , 175usize , 176usize , 177usize , 178usize , 179usize , 180usize , 181usize , 182usize , 183usize , 184usize , 185usize , 186usize , 187usize , 188usize , 189usize , 190usize , 191usize , 192usize , 193usize , 194usize , 195usize , 196usize , 197usize , 198usize , 199usize , 200usize , 201usize , 202usize , 203usize , 204usize , 205usize , 206usize , 207usize , 208usize , 209usize , 210usize , 211usize , 212usize , 213usize , 214usize , 215usize , 216usize , 217usize , 218usize , 219usize , 220usize , 221usize , 222usize , 223usize , 224usize , 225usize , 226usize , 227usize , 228usize , 229usize , 230usize , 231usize , 232usize , 233usize , 234usize , 235usize , 236usize , 237usize , 238usize , 239usize , 240usize , 241usize , 242usize , 243usize , 244usize , 245usize , 246usize , 247usize , 248usize , 249usize , 250usize , 251usize , 252usize , 253usize , 254usize , 255usize , 256usize , 257usize , 258usize , 259usize , 260usize , 261usize , 262usize , 263usize , 264usize , 265usize , 266usize , 267usize , 268usize , 269usize , 270usize , 271usize , 272usize , 273usize , 274usize , 275usize , 276usize , 277usize , 278usize , 279usize , 280usize , 281usize , 282usize , 283usize , 284usize , 285usize , 286usize , 287usize , 288usize , 289usize , 290usize , 291usize , 292usize , 293usize , 294usize , 295usize , 296usize , 297usize , 298usize , 299usize , 300usize , 301usize , 302usize , 303usize , 304usize , 305usize , 306usize , 307usize , 308usize , 309usize , 310usize , 311usize , 312usize , 313usize , 314usize , 315usize , 316usize , 317usize , 318usize , 319usize , 320usize , 321usize , 322usize , 323usize , 324usize , 325usize , 326usize , 327usize , 328usize , 329usize , 330usize , 331usize , 332usize , 333usize , 334usize , 335usize , 336usize , 337usize , 338usize , 339usize , 340usize , 341usize , 342usize , 343usize , 344usize , 345usize , 346usize , 347usize , 348usize , 349usize , 350usize , 351usize , 352usize , 353usize , 354usize , 355usize , 356usize , 357usize , 358usize , 359usize , 360usize , 361usize , 362usize , 363usize , 364usize , 365usize , 366usize , 367usize , 368usize , 369usize , 370usize , 371usize , 372usize , 373usize , 374usize , 375usize , 376usize , 377usize , 378usize , 379usize , 380usize , 381usize , 382usize , 383usize , 384usize , 385usize , 386usize , 387usize , 388usize , 389usize , 390usize , 391usize , 392usize , 393usize , 394usize , 395usize , 396usize , 397usize , 398usize , 399usize , 400usize , 401usize , 402usize , 403usize , 404usize , 405usize , 406usize , 407usize , 408usize , 409usize , 410usize , 411usize , 412usize , 413usize , 414usize , 415usize , 416usize , 417usize , 418usize , 419usize , 420usize , 421usize , 422usize , 423usize , 424usize , 425usize , 426usize , 427usize , 428usize , 429usize , 430usize , 431usize , 432usize , 433usize , 434usize , 435usize , 436usize , 437usize , 438usize , 439usize , 440usize , 441usize , 442usize , 443usize , 444usize , 445usize , 446usize , 447usize , 448usize , 449usize , 450usize , 451usize , 452usize , 453usize , 454usize , 455usize , 456usize , 457usize , 458usize , 459usize , 460usize , 461usize , 462usize , 463usize , 464usize , 465usize , 466usize , 467usize , 468usize , 469usize , 470usize , 471usize , 472usize , 473usize , 474usize , 475usize , 476usize , 477usize , 478usize , 479usize , 480usize , 481usize , 482usize , 483usize , 484usize , 485usize , 486usize , 487usize , 488usize , 489usize , 490usize , 491usize , 492usize , 493usize , 494usize , 495usize , 496usize , 497usize , 498usize , 499usize , 500usize , 501usize , 502usize , 503usize , 504usize , 505usize , 506usize , 507usize , 508usize , 509usize , 510usize , 511usize , 512usize , 513usize , 514usize , 515usize , 516usize , 517usize , 518usize , 519usize , 520usize , 521usize , 522usize , 523usize , 524usize , 525usize , 526usize , 527usize , 528usize , 529usize , 530usize , 531usize , 532usize , 533usize , 534usize , 535usize , 536usize , 537usize , 538usize , 539usize , 540usize , 541usize , 542usize , 543usize , 544usize , 545usize , 546usize , 547usize , 548usize , 549usize , 550usize , 551usize , 552usize , 553usize , 554usize , 555usize , 556usize , 557usize , 558usize , 559usize , 560usize , 561usize , 562usize , 563usize , 564usize , 565usize , 566usize , 567usize , 568usize , 569usize , 570usize , 571usize , 572usize , 573usize , 574usize , 575usize , 576usize , 577usize , 578usize , 579usize , 580usize , 581usize , 582usize , 583usize , 584usize , 585usize , 586usize , 587usize , 588usize , 589usize , 590usize , 591usize , 592usize , 593usize , 594usize , 595usize , 596usize , 597usize , 598usize , 599usize , 600usize , 601usize , 602usize , 603usize , 604usize , 605usize , 606usize , 607usize , 608usize , 609usize , 610usize , 611usize , 612usize , 613usize , 614usize , 615usize , 616usize , 617usize , 618usize , 619usize , 620usize , 621usize , 622usize , 623usize , 624usize , 625usize , 626usize , 627usize , 628usize , 629usize , 630usize , 631usize , 632usize , 633usize , 634usize , 635usize , 636usize , 637usize , 638usize , 639usize , 640usize , 641usize , 642usize , 643usize , 644usize , 645usize , 646usize , 647usize , 648usize , 649usize , 650usize , 651usize , 652usize , 653usize , 654usize , 655usize , 656usize , 657usize , 658usize , 659usize , 660usize , 661usize , 662usize , 663usize , 664usize , 665usize , 666usize , 667usize , 668usize , 669usize , 670usize , 671usize , 672usize , 673usize , 674usize , 675usize , 676usize , 677usize , 678usize , 679usize , 680usize , 681usize , 682usize , 683usize , 684usize , 685usize , 686usize , 687usize , 688usize , 689usize , 690usize , 691usize , 692usize , 693usize , 694usize , 695usize , 696usize , 697usize , 698usize , 699usize , 700usize , 701usize , 702usize , 703usize , 704usize , 705usize , 706usize , 707usize , 708usize , 709usize , 710usize , 711usize , 712usize , 713usize , 714usize , 715usize , 716usize , 717usize , 718usize , 719usize , 720usize , 721usize , 722usize , 723usize , 724usize , 725usize , 726usize , 727usize , 728usize , 729usize , 730usize , 731usize , 732usize , 733usize , 734usize , 735usize , 736usize , 737usize , 738usize , 739usize , 740usize , 741usize , 742usize , 743usize , 744usize , 745usize , 746usize , 747usize , 748usize , 749usize , 750usize , 751usize , 752usize , 753usize , 754usize , 755usize , 756usize , 757usize , 758usize , 759usize , 760usize , 761usize , 762usize , 763usize , 764usize , 765usize , 766usize , 767usize , 768usize , 769usize , 770usize , 771usize , 772usize , 773usize , 774usize , 775usize , 776usize , 777usize , 778usize , 779usize , 780usize , 781usize , 782usize , 783usize , 784usize , 785usize , 786usize , 787usize , 788usize , 789usize , 790usize , 791usize , 792usize , 793usize , 794usize , 795usize , 796usize , 797usize , 798usize , 799usize , 800usize , 801usize , 802usize , 803usize , 804usize , 805usize , 806usize , 807usize , 808usize , 809usize , 810usize , 811usize , 812usize , 813usize , 814usize , 815usize , 816usize , 817usize , 818usize , 819usize , 820usize , 821usize , 822usize , 823usize , 824usize , 825usize , 826usize , 827usize , 828usize , 829usize , 830usize , 831usize , 832usize , 833usize , 834usize , 835usize , 836usize , 837usize , 838usize , 839usize , 840usize , 841usize , 842usize , 843usize , 844usize , 845usize , 846usize , 847usize , 848usize , 849usize , 850usize , 851usize , 852usize , 853usize , 854usize , 855usize , 856usize , 857usize , 858usize , 859usize , 860usize , 861usize , 862usize , 863usize , 864usize , 865usize , 866usize , 867usize , 868usize , 869usize , 870usize , 871usize , 872usize , 873usize , 874usize , 875usize , 876usize , 877usize , 878usize , 879usize , 880usize , 881usize , 882usize , 883usize , 884usize , 885usize , 886usize , 887usize , 888usize , 889usize , 890usize , 891usize , 892usize , 893usize , 894usize , 895usize , 896usize , 897usize , 898usize , 899usize , 900usize , 901usize , 902usize , 903usize , 904usize , 905usize , 906usize , 907usize , 908usize , 909usize , 910usize , 911usize , 912usize , 913usize , 914usize , 915usize , 916usize , 917usize , 918usize , 919usize , 920usize , 921usize , 922usize , 923usize , 924usize , 925usize , 926usize , 927usize , 928usize , 929usize , 930usize , 931usize , 932usize , 933usize , 934usize , 935usize , 936usize , 937usize , 938usize , 939usize , 940usize , 941usize , 942usize , 943usize , 944usize , 945usize , 946usize , 947usize , 948usize , 949usize , 950usize , 951usize , 952usize , 953usize , 954usize , 955usize , 956usize , 957usize , 958usize , 959usize , 960usize , 961usize , 962usize , 963usize , 964usize , 965usize , 966usize , 967usize , 968usize , 969usize , 970usize , 971usize , 972usize , 973usize , 974usize , 975usize , 976usize , 977usize , 978usize , 979usize , 980usize , 981usize , 982usize , 983usize , 984usize , 985usize , 986usize , 987usize , 988usize , 989usize , 990usize , 991usize , 992usize , 993usize , 994usize , 995usize , 996usize , 997usize , 998usize , 999usize , 1000usize , 1001usize , 1002usize , 1003usize , 1004usize , 1005usize , 1006usize , 1007usize , 1008usize , 1009usize , 1010usize , 1011usize , 1012usize , 1013usize , 1014usize , 1015usize , 1016usize , 1017usize , 1018usize , 1019usize , 1020usize , 1021usize , 1022usize , 1023usize , 1024usize , 1025usize , 1026usize , 1027usize , 1028usize , 1029usize , 1030usize , 1031usize , 1032usize , 1033usize , 1034usize , 1035usize , 1036usize , 1037usize , 1038usize , 1039usize , 1040usize , 1041usize , 1042usize , 1043usize , 1044usize , 1045usize , 1046usize , 1047usize , 1048usize , 1049usize , 1050usize , 1051usize , 1052usize , 1053usize , 1054usize , 1055usize , 1056usize , 1057usize , 1058usize , 1059usize , 1060usize , 1061usize , 1062usize , 1063usize , 1064usize , 1065usize , 1066usize , 1067usize , 1068usize , 1069usize , 1070usize , 1071usize , 1072usize , 1073usize , 1074usize , 1075usize , 1076usize , 1077usize , 1078usize , 1079usize , 1080usize , 1081usize , 1082usize , 1083usize , 1084usize , 1085usize , 1086usize , 1087usize , 1088usize , 1089usize , 1090usize , 1091usize , 1092usize , 1093usize , 1094usize , 1095usize , 1096usize , 1097usize , 1098usize , 1099usize , 1100usize , 1101usize , 1102usize , 1103usize , 1104usize , 1105usize , 1106usize , 1107usize , 1108usize , 1109usize , 1110usize , 1111usize , 1112usize , 1113usize , 1114usize , 1115usize , 1116usize , 1117usize , 1118usize , 1119usize , 1120usize , 1121usize , 1122usize , 1123usize , 1124usize , 1125usize , 1126usize , 1127usize , 1128usize , 1129usize , 1130usize , 1131usize , 1132usize , 1133usize , 1134usize , 1135usize , 1136usize , 1137usize , 1138usize , 1139usize , 1140usize , 1141usize , 1142usize , 1143usize , 1144usize , 1145usize , 1146usize , 1147usize , 1148usize , 1149usize , 1150usize , 1151usize , 1152usize , 1153usize , 1154usize , 1155usize , 1156usize , 1157usize , 1158usize , 1159usize , 1160usize , 1161usize , 1162usize , 1163usize , 1164usize , 1165usize , 1166usize , 1167usize , 1168usize , 1169usize , 1170usize , 1171usize , 1172usize , 1173usize , 1174usize , 1175usize , 1176usize , 1177usize , 1178usize , 1179usize , 1180usize , 1181usize , 1182usize , 1183usize , 1184usize , 1185usize , 1186usize , 1187usize , 1188usize , 1189usize , 1190usize , 1191usize , 1192usize , 1193usize , 1194usize , 1195usize , 1196usize , 1197usize , 1198usize , 1199usize , 1200usize , 1201usize , 1202usize , 1203usize , 1204usize , 1205usize , 1206usize , 1207usize , 1208usize , 1209usize , 1210usize , 1211usize , 1212usize , 1213usize , 1214usize , 1215usize , 1216usize , 1217usize , 1218usize , 1219usize , 1220usize , 1221usize , 1222usize , 1223usize , 1224usize , 1225usize , 1226usize , 1227usize , 1228usize , 1229usize , 1230usize , 1231usize , 1232usize , 1233usize , 1234usize , 1235usize , 1236usize , 1237usize , 1238usize , 1239usize , 1240usize , 1241usize , 1242usize , 1243usize , 1244usize , 1245usize , 1246usize , 1247usize , 1248usize , 1249usize , 1250usize , 1251usize , 1252usize , 1253usize , 1254usize , 1255usize , 1256usize , 1257usize , 1258usize , 1259usize , 1260usize , 1261usize , 1262usize , 1263usize , 1264usize , 1265usize , 1266usize , 1267usize , 1268usize , 1269usize , 1270usize , 1271usize , 1272usize , 1273usize , 1274usize , 1275usize , 1276usize , 1277usize , 1278usize , 1279usize , 1280usize , 1281usize , 1282usize , 1283usize , 1284usize , 1285usize , 1286usize , 1287usize , 1288usize , 1289usize , 1290usize , 1291usize , 1292usize , 1293usize , 1294usize , 1295usize , 1296usize , 1297usize , 1298usize , 1299usize , 1300usize , 1301usize , 1302usize , 1303usize , 1304usize , 1305usize , 1306usize , 1307usize , 1308usize , 1309usize , 1310usize , 1311usize , 1312usize , 1313usize , 1314usize , 1315usize , 1316usize , 1317usize , 1318usize , 1319usize , 1320usize , 1321usize , 1322usize , 1323usize , 1324usize , 1325usize , 1326usize , 1327usize , 1328usize , 1329usize , 1330usize , 1331usize , 1332usize , 1333usize , 1334usize , 1335usize , 1336usize , 1337usize , 1338usize , 1339usize , 1340usize , 1341usize , 1342usize , 1343usize , 1344usize , 1345usize , 1346usize , 1347usize , 1348usize , 1349usize , 1350usize , 1351usize , 1352usize , 1353usize , 1354usize , 1355usize , 1356usize , 1357usize , 1358usize , 1359usize , 1360usize , 1361usize , 1362usize , 1363usize , 1364usize , 1365usize , 1366usize , 1367usize , 1368usize , 1369usize , 1370usize , 1371usize , 1372usize , 1373usize , 1374usize , 1375usize , 1376usize , 1377usize , 1378usize , 1379usize , 1380usize , 1381usize , 1382usize , 1383usize , 1384usize , 1385usize , 1386usize , 1387usize , 1388usize , 1389usize , 1390usize , 1391usize , 1392usize , 1393usize , 1394usize , 1395usize , 1396usize , 1397usize , 1398usize , 1399usize , 1400usize , 1401usize , 1402usize , 1403usize , 1404usize , 1405usize , 1406usize , 1407usize , 1408usize , 1409usize , 1410usize , 1411usize , 1412usize , 1413usize , 1414usize , 1415usize , 1416usize , 1417usize , 1418usize , 1419usize , 1420usize , 1421usize , 1422usize , 1423usize , 1424usize , 1425usize , 1426usize , 1427usize , 1428usize , 1429usize , 1430usize , 1431usize , 1432usize , 1433usize , 1434usize , 1435usize , 1436usize , 1437usize , 1438usize , 1439usize , 1440usize , 1441usize , 1442usize , 1443usize , 1444usize , 1445usize , 1446usize , 1447usize , 1448usize , 1449usize , 1450usize , 1451usize , 1452usize , 1453usize , 1454usize , 1455usize , 1456usize , 1457usize , 1458usize , 1459usize , 1460usize , 1461usize , 1462usize , 1463usize , 1464usize , 1465usize , 1466usize , 1467usize , 1468usize , 1469usize , 1470usize , 1471usize , 1472usize , 1473usize , 1474usize , 1475usize , 1476usize , 1477usize , 1478usize , 1479usize , 1480usize , 1481usize , 1482usize , 1483usize , 1484usize , 1485usize , 1486usize , 1487usize , 1488usize , 1489usize , 1490usize , 1491usize , 1492usize , 1493usize , 1494usize , 1495usize , 1496usize , 1497usize , 1498usize , 1499usize , 1500usize , 1501usize , 1502usize , 1503usize , 1504usize , 1505usize , 1506usize , 1507usize , 1508usize , 1509usize , 1510usize , 1511usize , 1512usize , 1513usize , 1514usize , 1515usize , 1516usize , 1517usize , 1518usize , 1519usize , 1520usize , 1521usize , 1522usize , 1523usize , 1524usize , 1525usize , 1526usize , 1527usize , 1528usize , 1529usize , 1530usize , 1531usize , 1532usize , 1533usize , 1534usize , 1535usize , 1536usize , 1537usize , 1538usize , 1539usize , 1540usize , 1541usize , 1542usize , 1543usize , 1544usize , 1545usize , 1546usize , 1547usize , 1548usize , 1549usize , 1550usize , 1551usize , 1552usize , 1553usize , 1554usize , 1555usize , 1556usize , 1557usize , 1558usize , 1559usize , 1560usize , 1561usize , 1562usize , 1563usize , 1564usize , 1565usize , 1566usize , 1567usize , 1568usize , 1569usize , 1570usize , 1571usize , 1572usize , 1573usize , 1574usize , 1575usize , 1576usize , 1577usize , 1578usize , 1579usize , 1580usize , 1581usize , 1582usize , 1583usize , 1584usize , 1585usize , 1586usize , 1587usize , 1588usize , 1589usize , 1590usize , 1591usize , 1592usize , 1593usize , 1594usize , 1595usize , 1596usize , 1597usize , 1598usize , 1599usize , 1600usize , 1601usize , 1602usize , 1603usize , 1604usize , 1605usize , 1606usize , 1607usize , 1608usize , 1609usize , 1610usize , 1611usize , 1612usize , 1613usize , 1614usize , 1615usize , 1616usize , 1617usize , 1618usize , 1619usize , 1620usize , 1621usize , 1622usize , 1623usize , 1624usize , 1625usize , 1626usize , 1627usize , 1628usize , 1629usize , 1630usize , 1631usize , 1632usize , 1633usize , 1634usize , 1635usize , 1636usize , 1637usize , 1638usize , 1639usize , 1640usize , 1641usize , 1642usize , 1643usize] , value : Advancement :: RECIPES_ROOT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_BLACK_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_BLUE_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_BROWN_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_BRUSH , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_CLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_COMPASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_COPPER_AXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_COPPER_HOE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_COPPER_PICKAXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_COPPER_SHOVEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_CYAN_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_DIAMOND_AXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_DIAMOND_HOE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_DIAMOND_PICKAXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_DIAMOND_SHOVEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_FISHING_ROD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_FLINT_AND_STEEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_GOLDEN_AXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_GOLDEN_HOE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_GOLDEN_PICKAXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_GOLDEN_SHOVEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_GRAY_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_GREEN_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_IRON_AXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_IRON_HOE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_IRON_PICKAXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_IRON_SHOVEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_LEAD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_LIGHT_BLUE_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_LIGHT_GRAY_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_LIME_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_MAGENTA_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_NAME_TAG , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_NETHERITE_AXE_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_NETHERITE_HOE_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_NETHERITE_PICKAXE_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_NETHERITE_SHOVEL_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_ORANGE_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_PINK_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_PURPLE_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_RECOVERY_COMPASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_RED_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_SHEARS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_SPYGLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_STONE_AXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_STONE_HOE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_STONE_PICKAXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_STONE_SHOVEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_WHITE_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_WOODEN_AXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_WOODEN_HOE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_WOODEN_PICKAXE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_WOODEN_SHOVEL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TOOLS_YELLOW_BUNDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_ACACIA_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_ACACIA_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_ACTIVATOR_RAIL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_BAMBOO_CHEST_RAFT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_BAMBOO_RAFT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_BIRCH_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_BIRCH_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_CARROT_ON_A_STICK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_CHERRY_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_CHERRY_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_CHEST_MINECART , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_DARK_OAK_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_DARK_OAK_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_DETECTOR_RAIL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_FURNACE_MINECART , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_HOPPER_MINECART , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_JUNGLE_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_JUNGLE_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_MANGROVE_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_MANGROVE_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_MINECART , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_OAK_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_OAK_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_PALE_OAK_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_PALE_OAK_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_POWERED_RAIL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_RAIL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_SPRUCE_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_SPRUCE_CHEST_BOAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_TNT_MINECART , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_TRANSPORTATION_WARPED_FUNGUS_ON_A_STICK , } , AdvancementNode { parent : None , children : vec ! [1644usize] , value : Advancement :: STORY_ROOT , } , AdvancementNode { parent : Some (2usize) , children : vec ! [] , value : Advancement :: ADVENTURE_ADVENTURING_TIME , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_AVOID_VIBRATION , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_BRUSH_ARMADILLO , } , AdvancementNode { parent : Some (1usize) , children : vec ! [] , value : Advancement :: ADVENTURE_CRAFT_DECORATED_POT_USING_ONLY_SHERDS , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_CRAFTERS_CRAFTING_CRAFTERS , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_FALL_FROM_WORLD_HEIGHT , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_HEART_TRANSPLANTER , } , AdvancementNode { parent : Some (8usize) , children : vec ! [] , value : Advancement :: ADVENTURE_HERO_OF_THE_VILLAGE , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_HONEY_BLOCK_SLIDE , } , AdvancementNode { parent : Some (0usize) , children : vec ! [118usize , 119usize , 126usize , 128usize , 131usize , 132usize] , value : Advancement :: ADVENTURE_KILL_A_MOB , } , AdvancementNode { parent : Some (117usize) , children : vec ! [] , value : Advancement :: ADVENTURE_KILL_ALL_MOBS , } , AdvancementNode { parent : Some (117usize) , children : vec ! [] , value : Advancement :: ADVENTURE_KILL_MOB_NEAR_SCULK_CATALYST , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_LIGHTNING_ROD_WITH_VILLAGER_NO_FIRE , } , AdvancementNode { parent : Some (0usize) , children : vec ! [123usize , 135usize , 137usize , 1647usize , 1649usize] , value : Advancement :: ADVENTURE_MINECRAFT_TRIALS_EDITION , } , AdvancementNode { parent : Some (0usize) , children : vec ! [134usize , 138usize , 1646usize] , value : Advancement :: ADVENTURE_OL_BETSY , } , AdvancementNode { parent : Some (121usize) , children : vec ! [] , value : Advancement :: ADVENTURE_OVEROVERKILL , } , AdvancementNode { parent : Some (2usize) , children : vec ! [] , value : Advancement :: ADVENTURE_PLAY_JUKEBOX_IN_MEADOWS , } , AdvancementNode { parent : Some (0usize) , children : vec ! [] , value : Advancement :: ADVENTURE_READ_POWER_OF_CHISELED_BOOKSHELF , } , AdvancementNode { parent : Some (117usize) , children : vec ! [127usize , 1648usize] , value : Advancement :: ADVENTURE_SHOOT_ARROW , } , AdvancementNode { parent : Some (126usize) , children : vec ! [] , value : Advancement :: ADVENTURE_SNIPER_DUEL , } , AdvancementNode { parent : Some (117usize) , children : vec ! [] , value : Advancement :: ADVENTURE_SPEAR_MANY_MOBS , } , AdvancementNode { parent : Some (3usize) , children : vec ! [1651usize] , value : Advancement :: ADVENTURE_SPYGLASS_AT_GHAST , } , AdvancementNode { parent : Some (4usize) , children : vec ! [] , value : Advancement :: ADVENTURE_SUMMON_IRON_GOLEM , } , AdvancementNode { parent : Some (117usize) , children : vec ! [136usize] , value : Advancement :: ADVENTURE_THROW_TRIDENT , } , AdvancementNode { parent : Some (117usize) , children : vec ! [] , value : Advancement :: ADVENTURE_TOTEM_OF_UNDYING , } , AdvancementNode { parent : Some (6usize) , children : vec ! [] , value : Advancement :: ADVENTURE_TRIM_WITH_ALL_EXCLUSIVE_ARMOR_PATTERNS , } , AdvancementNode { parent : Some (122usize) , children : vec ! [] , value : Advancement :: ADVENTURE_TWO_BIRDS_ONE_ARROW , } , AdvancementNode { parent : Some (121usize) , children : vec ! [1650usize] , value : Advancement :: ADVENTURE_UNDER_LOCK_AND_KEY , } , AdvancementNode { parent : Some (131usize) , children : vec ! [] , value : Advancement :: ADVENTURE_VERY_VERY_FRIGHTENING , } , AdvancementNode { parent : Some (121usize) , children : vec ! [] , value : Advancement :: ADVENTURE_WHO_NEEDS_ROCKETS , } , AdvancementNode { parent : Some (122usize) , children : vec ! [] , value : Advancement :: ADVENTURE_WHOS_THE_PILLAGER_NOW , } , AdvancementNode { parent : Some (10usize) , children : vec ! [140usize , 1652usize , 1653usize , 1654usize] , value : Advancement :: END_KILL_DRAGON , } , AdvancementNode { parent : Some (139usize) , children : vec ! [] , value : Advancement :: END_RESPAWN_DRAGON , } , AdvancementNode { parent : Some (11usize) , children : vec ! [1657usize] , value : Advancement :: HUSBANDRY_ALLAY_DELIVER_ITEM_TO_PLAYER , } , AdvancementNode { parent : Some (11usize) , children : vec ! [1660usize] , value : Advancement :: HUSBANDRY_BREED_AN_ANIMAL , } , AdvancementNode { parent : Some (15usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_COMPLETE_CATALOGUE , } , AdvancementNode { parent : Some (11usize) , children : vec ! [153usize] , value : Advancement :: HUSBANDRY_FISHY_BUSINESS , } , AdvancementNode { parent : Some (14usize) , children : vec ! [1662usize] , value : Advancement :: HUSBANDRY_LEASH_ALL_FROG_VARIANTS , } , AdvancementNode { parent : Some (11usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_MAKE_A_SIGN_GLOW , } , AdvancementNode { parent : Some (11usize) , children : vec ! [1661usize] , value : Advancement :: HUSBANDRY_OBTAIN_SNIFFER_EGG , } , AdvancementNode { parent : Some (11usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_PLACE_DRIED_GHAST_IN_WATER , } , AdvancementNode { parent : Some (11usize) , children : vec ! [1659usize , 1664usize] , value : Advancement :: HUSBANDRY_PLANT_SEED , } , AdvancementNode { parent : Some (15usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_REMOVE_WOLF_ARMOR , } , AdvancementNode { parent : Some (15usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_REPAIR_WOLF_ARMOR , } , AdvancementNode { parent : Some (11usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT , } , AdvancementNode { parent : Some (144usize) , children : vec ! [1658usize] , value : Advancement :: HUSBANDRY_TACTICAL_FISHING , } , AdvancementNode { parent : Some (17usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_WAX_OFF , } , AdvancementNode { parent : Some (19usize) , children : vec ! [] , value : Advancement :: NETHER_DISTRACT_PIGLIN , } , AdvancementNode { parent : Some (19usize) , children : vec ! [] , value : Advancement :: NETHER_FAST_TRAVEL , } , AdvancementNode { parent : Some (19usize) , children : vec ! [160usize] , value : Advancement :: NETHER_FIND_BASTION , } , AdvancementNode { parent : Some (19usize) , children : vec ! [159usize , 162usize] , value : Advancement :: NETHER_FIND_FORTRESS , } , AdvancementNode { parent : Some (158usize) , children : vec ! [167usize] , value : Advancement :: NETHER_GET_WITHER_SKULL , } , AdvancementNode { parent : Some (157usize) , children : vec ! [] , value : Advancement :: NETHER_LOOT_BASTION , } , AdvancementNode { parent : Some (19usize) , children : vec ! [1671usize] , value : Advancement :: NETHER_OBTAIN_ANCIENT_DEBRIS , } , AdvancementNode { parent : Some (158usize) , children : vec ! [1666usize] , value : Advancement :: NETHER_OBTAIN_BLAZE_ROD , } , AdvancementNode { parent : Some (19usize) , children : vec ! [1667usize] , value : Advancement :: NETHER_OBTAIN_CRYING_OBSIDIAN , } , AdvancementNode { parent : Some (19usize) , children : vec ! [168usize] , value : Advancement :: NETHER_RETURN_TO_SENDER , } , AdvancementNode { parent : Some (19usize) , children : vec ! [166usize , 1670usize] , value : Advancement :: NETHER_RIDE_STRIDER , } , AdvancementNode { parent : Some (165usize) , children : vec ! [] , value : Advancement :: NETHER_RIDE_STRIDER_IN_OVERWORLD_LAVA , } , AdvancementNode { parent : Some (159usize) , children : vec ! [1668usize] , value : Advancement :: NETHER_SUMMON_WITHER , } , AdvancementNode { parent : Some (164usize) , children : vec ! [] , value : Advancement :: NETHER_UNEASY_ALLIANCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_BLAZE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_BREWING_STAND , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_CAULDRON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_FERMENTED_SPIDER_EYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_GLASS_BOTTLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_GLISTERING_MELON_SLICE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_GOLDEN_CARROT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BREWING_MAGMA_CREAM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ACACIA_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ACACIA_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ACACIA_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ACACIA_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_AMETHYST_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ANDESITE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BAMBOO_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BAMBOO_MOSAIC_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BAMBOO_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BAMBOO_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BAMBOO_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BIRCH_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BIRCH_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BIRCH_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BIRCH_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACK_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACK_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACK_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLUE_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLUE_ICE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLUE_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BLUE_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BONE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BOOKSHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BRICK_SLAB_FROM_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BRICK_STAIRS_FROM_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BROWN_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BROWN_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_BROWN_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHERRY_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHERRY_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHERRY_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHERRY_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_BOOKSHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_CINNABAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_CINNABAR_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_COPPER_FROM_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_DEEPSLATE_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_NETHER_BRICKS_FROM_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_POLISHED_BLACKSTONE_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_QUARTZ_BLOCK_FROM_QUARTZ_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_RESIN_BRICKS_FROM_RESIN_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_SANDSTONE_FROM_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_FROM_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_STONE_BRICKS_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_SULFUR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_SULFUR_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_BRICKS_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CHISELED_TUFF_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_SLAB_FROM_CINNABAR_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_SLAB_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_SLAB_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_STAIRS_FROM_CINNABAR_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_STAIRS_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICK_STAIRS_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICKS_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_BRICKS_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_SLAB_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CINNABAR_STAIRS_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CLAY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COAL_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COARSE_DIRT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_SLAB_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLED_DEEPSLATE_STAIRS_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB_FROM_COBBLESTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_SLAB_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS_FROM_COBBLESTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COBBLESTONE_STAIRS_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COPPER_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_COPPER_GRATE_FROM_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRACKED_DEEPSLATE_TILES , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRACKED_NETHER_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRACKED_POLISHED_BLACKSTONE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRACKED_STONE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRIMSON_HYPHAE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRIMSON_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRIMSON_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CRIMSON_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_FROM_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_SLAB_FROM_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_COPPER_STAIRS_FROM_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_FROM_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_CUT_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_FROM_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_CUT_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CUT_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CYAN_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CYAN_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_CYAN_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_OAK_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_OAK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_OAK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_OAK_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_SLAB_FROM_DARK_PRISMARINE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DARK_PRISMARINE_STAIRS_FROM_DARK_PRISMARINE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICK_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_BRICKS_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_DEEPSLATE_TILES_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_DEEPSLATE_TILES_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DEEPSLATE_TILES_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DIAMOND_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DIORITE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DIORITE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DIORITE_SLAB_FROM_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DRIED_GHAST , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DRIED_KELP_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DRIPSTONE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_BLACK_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_BLUE_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_BROWN_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_CYAN_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_GRAY_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_GREEN_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_LIGHT_BLUE_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_LIGHT_GRAY_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_LIME_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_MAGENTA_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_ORANGE_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_PINK_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_PURPLE_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_RED_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_WHITE_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_DYE_YELLOW_WOOL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EMERALD_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_SLAB_FROM_END_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICK_STAIRS_FROM_END_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_END_STONE_BRICKS_FROM_END_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CHISELED_COPPER_FROM_EXPOSED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_COPPER_GRATE_FROM_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_FROM_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_SLAB_FROM_EXPOSED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_EXPOSED_CUT_COPPER_STAIRS_FROM_EXPOSED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GLOWSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GOLD_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRANITE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRANITE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRANITE_SLAB_FROM_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRAY_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRAY_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GRAY_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GREEN_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GREEN_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_GREEN_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_HAY_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_IRON_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_JACK_O_LANTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_JUNGLE_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_JUNGLE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_JUNGLE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_JUNGLE_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LAPIS_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIGHT_BLUE_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIGHT_GRAY_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIME_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIME_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_LIME_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MAGENTA_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MAGENTA_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MAGENTA_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MAGMA_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MANGROVE_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MANGROVE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MANGROVE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MANGROVE_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MELON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_MOSS_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_FROM_VINE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_SLAB_FROM_MOSSY_COBBLESTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_COBBLESTONE_STAIRS_FROM_MOSSY_COBBLESTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_SLAB_FROM_MOSSY_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICK_STAIRS_FROM_MOSSY_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_MOSS_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MOSSY_STONE_BRICKS_FROM_VINE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_SLAB_FROM_MUD_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MUD_BRICK_STAIRS_FROM_MUD_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MUD_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_MUDDY_MANGROVE_ROOTS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_SLAB_FROM_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHER_BRICK_STAIRS_FROM_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHER_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHER_WART_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_NETHERITE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OAK_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OAK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OAK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OAK_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ORANGE_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ORANGE_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_ORANGE_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CHISELED_COPPER_FROM_OXIDIZED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_COPPER_GRATE_FROM_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_FROM_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_SLAB_FROM_OXIDIZED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_OXIDIZED_CUT_COPPER_STAIRS_FROM_OXIDIZED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PACKED_ICE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PACKED_MUD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PALE_OAK_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PALE_OAK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PALE_OAK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PALE_OAK_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PINK_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PINK_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PINK_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_FROM_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_SLAB_FROM_POLISHED_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_ANDESITE_STAIRS_FROM_POLISHED_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BASALT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BASALT_FROM_BASALT_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICK_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_BRICKS_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_SLAB_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_BLACKSTONE_STAIRS_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_SLAB_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_SLAB_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_STAIRS_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_CINNABAR_STAIRS_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_SLAB_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DEEPSLATE_STAIRS_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_FROM_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_SLAB_FROM_POLISHED_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_DIORITE_STAIRS_FROM_POLISHED_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_FROM_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_SLAB_FROM_POLISHED_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_GRANITE_STAIRS_FROM_POLISHED_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_SLAB_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_SLAB_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_STAIRS_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_SULFUR_STAIRS_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_SLAB_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POLISHED_TUFF_STAIRS_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_POTENT_SULFUR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_SLAB_FROM_PRISMARINE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICK_STAIRS_FROM_PRISMARINE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_SLAB_FROM_PRISMARINE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PRISMARINE_STAIRS_FROM_PRISMARINE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPLE_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPLE_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPLE_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_PILLAR_FROM_PURPUR_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_SLAB_FROM_PURPUR_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_PURPUR_STAIRS_FROM_PURPUR_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_BRICKS_FROM_QUARTZ_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_PILLAR_FROM_QUARTZ_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_SLAB_FROM_QUARTZ_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_QUARTZ_STAIRS_FROM_QUARTZ_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RAW_COPPER_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RAW_GOLD_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RAW_IRON_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_SLAB_FROM_RED_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICK_STAIRS_FROM_RED_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_NETHER_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_SLAB_FROM_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_SANDSTONE_STAIRS_FROM_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RESIN_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_SLAB_FROM_RESIN_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RESIN_BRICK_STAIRS_FROM_RESIN_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_RESIN_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SANDSTONE_SLAB_FROM_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SANDSTONE_STAIRS_FROM_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SEA_LANTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_BASALT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_SLAB_FROM_SMOOTH_QUARTZ_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_QUARTZ_STAIRS_FROM_SMOOTH_QUARTZ_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_SLAB_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_RED_SANDSTONE_STAIRS_FROM_SMOOTH_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_SLAB_FROM_SMOOTH_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_SANDSTONE_STAIRS_FROM_SMOOTH_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SMOOTH_STONE_SLAB_FROM_SMOOTH_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SNOW_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SPONGE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SPRUCE_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SPRUCE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SPRUCE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SPRUCE_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_SLAB_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICK_STAIRS_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_BRICKS_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_SLAB_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STONE_STAIRS_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_ACACIA_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_BIRCH_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_CHERRY_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_CRIMSON_HYPHAE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_DARK_OAK_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_JUNGLE_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_MANGROVE_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_OAK_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_PALE_OAK_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_SPRUCE_WOOD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_STRIPPED_WARPED_HYPHAE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_SLAB_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_SLAB_FROM_SULFUR_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_SLAB_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_STAIRS_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_STAIRS_FROM_SULFUR_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICK_STAIRS_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICKS_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_BRICKS_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_FROM_SULFUR_SPIKES , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_SLAB_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_SULFUR_STAIRS_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TINTED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_SLAB_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICK_STAIRS_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_BRICKS_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_SLAB_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_TUFF_STAIRS_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WARPED_HYPHAE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WARPED_PLANKS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WARPED_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WARPED_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CHISELED_COPPER_FROM_WAXED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BARS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_BLOCK_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHAIN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_CHEST_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_GRATE_FROM_WAXED_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_COPPER_LANTERN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_FROM_WAXED_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_SLAB_FROM_WAXED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_COPPER_BLOCK_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_CUT_COPPER_STAIRS_FROM_WAXED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CHISELED_COPPER_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_BARS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHAIN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_CHEST_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_GRATE_FROM_WAXED_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_COPPER_LANTERN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_FROM_WAXED_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_SLAB_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_CUT_COPPER_STAIRS_FROM_WAXED_EXPOSED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_EXPOSED_LIGHTNING_ROD_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_LIGHTNING_ROD_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CHISELED_COPPER_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_BARS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHAIN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_CHEST_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_GRATE_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_COPPER_LANTERN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_SLAB_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_CUT_COPPER_STAIRS_FROM_WAXED_OXIDIZED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_OXIDIZED_LIGHTNING_ROD_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CHISELED_COPPER_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_BARS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHAIN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_CHEST_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GOLEM_STATUE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_GRATE_FROM_WAXED_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_COPPER_LANTERN_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_FROM_WAXED_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_SLAB_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_CUT_COPPER_STAIRS_FROM_WAXED_WEATHERED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WAXED_WEATHERED_LIGHTNING_ROD_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CHISELED_COPPER_FROM_WEATHERED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_COPPER_GRATE_FROM_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_FROM_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_SLAB_FROM_WEATHERED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WEATHERED_CUT_COPPER_STAIRS_FROM_WEATHERED_CUT_COPPER_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WHITE_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WHITE_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WHITE_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_WHITE_WOOL_FROM_STRING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_YELLOW_CONCRETE_POWDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_YELLOW_STAINED_GLASS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_BUILDING_BLOCKS_YELLOW_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_ARROW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_BLACK_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_BLUE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_BOW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_BROWN_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_COPPER_BOOTS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_COPPER_CHESTPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_COPPER_HELMET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_COPPER_LEGGINGS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_COPPER_SPEAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_COPPER_SWORD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_CROSSBOW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_CYAN_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DIAMOND_BOOTS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DIAMOND_CHESTPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DIAMOND_HELMET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DIAMOND_LEGGINGS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DIAMOND_SPEAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DIAMOND_SWORD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_BLACK_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_BLUE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_BROWN_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_CYAN_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_GRAY_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_GREEN_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_LIGHT_BLUE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_LIGHT_GRAY_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_LIME_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_MAGENTA_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_ORANGE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_PINK_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_PURPLE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_RED_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_WHITE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_DYE_YELLOW_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GOLDEN_BOOTS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GOLDEN_CHESTPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GOLDEN_HELMET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GOLDEN_LEGGINGS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GOLDEN_SPEAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GOLDEN_SWORD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GRAY_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_GREEN_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_IRON_BOOTS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_IRON_CHESTPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_IRON_HELMET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_IRON_LEGGINGS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_IRON_SPEAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_IRON_SWORD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LEATHER_BOOTS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LEATHER_CHESTPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LEATHER_HELMET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LEATHER_LEGGINGS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LIGHT_BLUE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LIGHT_GRAY_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_LIME_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_MACE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_MAGENTA_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_BOOTS_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_CHESTPLATE_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_HELMET_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_HORSE_ARMOR_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_LEGGINGS_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_NAUTILUS_ARMOR_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_SPEAR_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_NETHERITE_SWORD_SMITHING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_ORANGE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_PINK_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_PURPLE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_RED_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_SADDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_SHIELD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_SPECTRAL_ARROW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_STONE_SPEAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_STONE_SWORD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_TURTLE_HELMET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_WHITE_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_WOLF_ARMOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_WOODEN_SPEAR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_WOODEN_SWORD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_COMBAT_YELLOW_HARNESS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ACACIA_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ACACIA_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ACACIA_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ACACIA_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ANDESITE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ANDESITE_WALL_FROM_ANDESITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ANVIL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ARMOR_STAND , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BAMBOO_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BAMBOO_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BAMBOO_MOSAIC , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BAMBOO_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BAMBOO_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BARREL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BEEHIVE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BIRCH_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BIRCH_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BIRCH_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BIRCH_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACK_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACKSTONE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLAST_FURNACE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BRICK_WALL_FROM_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_BROWN_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CAMPFIRE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CARTOGRAPHY_TABLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CHERRY_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CHERRY_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CHERRY_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CHERRY_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CHEST , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CINNABAR_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CINNABAR_BRICK_WALL_FROM_CINNABAR_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CINNABAR_BRICK_WALL_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CINNABAR_BRICK_WALL_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CINNABAR_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CINNABAR_WALL_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COBBLED_DEEPSLATE_WALL_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COBBLESTONE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COBBLESTONE_WALL_FROM_COBBLESTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COBBLESTONE_WALL_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COMPOSTER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COPPER_BARS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COPPER_CHAIN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COPPER_CHEST , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COPPER_LANTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_COPPER_TORCH , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CRAFTING_TABLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CRIMSON_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CRIMSON_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CRIMSON_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CRIMSON_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_CYAN_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DARK_OAK_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DARK_OAK_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DARK_OAK_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DARK_OAK_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DECORATED_POT_SIMPLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_BRICK_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_DEEPSLATE_TILES_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DEEPSLATE_TILE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DIORITE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DIORITE_WALL_FROM_DIORITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_BLACK_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_BLACK_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_BLUE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_BLUE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_BROWN_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_BROWN_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_CYAN_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_CYAN_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_GRAY_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_GRAY_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_GREEN_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_GREEN_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_LIGHT_BLUE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_LIGHT_BLUE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_LIGHT_GRAY_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_LIGHT_GRAY_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_LIME_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_LIME_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_MAGENTA_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_MAGENTA_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_ORANGE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_ORANGE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_PINK_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_PINK_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_PURPLE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_PURPLE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_RED_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_RED_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_WHITE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_WHITE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_YELLOW_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_DYE_YELLOW_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ENCHANTING_TABLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_END_CRYSTAL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_END_ROD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_END_STONE_BRICK_WALL_FROM_END_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ENDER_CHEST , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_FLETCHING_TABLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_FLOWER_POT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_FURNACE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GLOW_ITEM_FRAME , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GOLDEN_DANDELION , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRANITE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRANITE_WALL_FROM_GRANITE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GREEN_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_GRINDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_HONEYCOMB_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_IRON_BARS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_IRON_CHAIN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ITEM_FRAME , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_JUKEBOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_JUNGLE_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_JUNGLE_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_JUNGLE_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_JUNGLE_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LADDER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LANTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_BLUE_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIGHT_GRAY_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LIME_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LODESTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_LOOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MAGENTA_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MANGROVE_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MANGROVE_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MANGROVE_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MANGROVE_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MOSS_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MOSSY_COBBLESTONE_WALL_FROM_MOSSY_COBBLESTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MOSSY_STONE_BRICK_WALL_FROM_MOSSY_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MUD_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_MUD_BRICK_WALL_FROM_MUD_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_NETHER_BRICK_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_NETHER_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_NETHER_BRICK_WALL_FROM_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_OAK_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_OAK_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_OAK_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_OAK_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_ORANGE_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PAINTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PALE_MOSS_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PALE_OAK_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PALE_OAK_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PALE_OAK_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PALE_OAK_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PINK_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_BRICK_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_BLACKSTONE_WALL_FROM_POLISHED_BLACKSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_CINNABAR_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_CINNABAR_WALL_FROM_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_CINNABAR_WALL_FROM_POLISHED_CINNABAR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_COBBLED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_DEEPSLATE_WALL_FROM_POLISHED_DEEPSLATE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_SULFUR_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_SULFUR_WALL_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_SULFUR_WALL_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_POLISHED_TUFF_WALL_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PRISMARINE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PRISMARINE_WALL_FROM_PRISMARINE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_PURPLE_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_NETHER_BRICK_WALL_FROM_RED_NETHER_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_SANDSTONE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_SANDSTONE_WALL_FROM_RED_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RED_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RESIN_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RESIN_BRICK_WALL_FROM_RESIN_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_RESPAWN_ANCHOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SANDSTONE_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SANDSTONE_WALL_FROM_SANDSTONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SCAFFOLDING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SMITHING_TABLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SMOKER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SNOW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SOUL_CAMPFIRE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SOUL_LANTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SOUL_TORCH , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SPRUCE_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SPRUCE_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SPRUCE_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SPRUCE_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_STONE_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_STONE_BRICK_WALL_FROM_STONE_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_STONE_BRICK_WALL_FROM_STONE_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_STONECUTTER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SULFUR_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SULFUR_BRICK_WALL_FROM_POLISHED_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SULFUR_BRICK_WALL_FROM_SULFUR_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SULFUR_BRICK_WALL_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SULFUR_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_SULFUR_WALL_FROM_SULFUR_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TORCH , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TUFF_BRICK_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_POLISHED_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_BRICKS_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TUFF_BRICK_WALL_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TUFF_WALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_TUFF_WALL_FROM_TUFF_STONECUTTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WARPED_FENCE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WARPED_HANGING_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WARPED_SHELF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WARPED_SIGN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_WHITE_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_BANNER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_BED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_CANDLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_CARPET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_GLAZED_TERRACOTTA , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_SHULKER_BOX , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_DECORATIONS_YELLOW_STAINED_GLASS_PANE_FROM_GLASS_PANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_BAKED_POTATO , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_BAKED_POTATO_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_BAKED_POTATO_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_BEETROOT_SOUP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_BREAD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_CAKE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_BEEF , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_BEEF_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_BEEF_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_CHICKEN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_CHICKEN_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_CHICKEN_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_COD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_COD_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_COD_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_MUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_MUTTON_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_MUTTON_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_PORKCHOP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_PORKCHOP_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_PORKCHOP_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_RABBIT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_RABBIT_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_RABBIT_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_SALMON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_SALMON_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKED_SALMON_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_COOKIE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_DRIED_KELP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_DRIED_KELP_FROM_CAMPFIRE_COOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_DRIED_KELP_FROM_SMELTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_DRIED_KELP_FROM_SMOKING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_GOLDEN_APPLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_HONEY_BOTTLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_MUSHROOM_STEW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_PUMPKIN_PIE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_RABBIT_STEW_FROM_BROWN_MUSHROOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_RABBIT_STEW_FROM_RED_MUSHROOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ALLIUM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_AZURE_BLUET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_BLUE_ORCHID , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CLOSED_EYEBLOSSOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_CORNFLOWER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_DANDELION , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_GOLDEN_DANDELION , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_LILY_OF_THE_VALLEY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OPEN_EYEBLOSSOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_ORANGE_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_OXEYE_DAISY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_PINK_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_POPPY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_RED_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_TORCHFLOWER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WHITE_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_FOOD_SUSPICIOUS_STEW_FROM_WITHER_ROSE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BEACON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BLACK_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BLACK_DYE_FROM_WITHER_ROSE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BLUE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BLUE_DYE_FROM_CORNFLOWER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BOLT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BONE_MEAL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BONE_MEAL_FROM_BONE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BOOK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BORDURE_INDENTED_BANNER_PATTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BOWL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BRICK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BROWN_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_BUCKET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_CHARCOAL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAL_FROM_BLASTING_COAL_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAL_FROM_BLASTING_DEEPSLATE_COAL_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAL_FROM_SMELTING_COAL_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAL_FROM_SMELTING_DEEPSLATE_COAL_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COAST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_CONDUIT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_COPPER_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_DEEPSLATE_COPPER_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_BLASTING_RAW_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_NUGGETS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_COPPER_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_DEEPSLATE_COPPER_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_SMELTING_RAW_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_INGOT_FROM_WAXED_COPPER_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_NUGGET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_NUGGET_FROM_BLASTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_COPPER_NUGGET_FROM_SMELTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_CREAKING_HEART , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_CREEPER_BANNER_PATTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_CYAN_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_CYAN_DYE_FROM_PITCHER_PLANT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DIAMOND , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DIAMOND_FROM_BLASTING_DEEPSLATE_DIAMOND_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DIAMOND_FROM_BLASTING_DIAMOND_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DIAMOND_FROM_SMELTING_DEEPSLATE_DIAMOND_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DIAMOND_FROM_SMELTING_DIAMOND_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_DUNE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EMERALD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EMERALD_FROM_BLASTING_DEEPSLATE_EMERALD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EMERALD_FROM_BLASTING_EMERALD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EMERALD_FROM_SMELTING_DEEPSLATE_EMERALD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EMERALD_FROM_SMELTING_EMERALD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_ENDER_EYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_EYE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_FIELD_MASONED_BANNER_PATTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_FIRE_CHARGE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_FIREWORK_ROCKET_SIMPLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_FLOW_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_FLOWER_BANNER_PATTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_DEEPSLATE_GOLD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_GOLD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_NETHER_GOLD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_BLASTING_RAW_GOLD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_GOLD_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_NUGGETS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_DEEPSLATE_GOLD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_GOLD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_NETHER_GOLD_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_INGOT_FROM_SMELTING_RAW_GOLD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_NUGGET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_NUGGET_FROM_BLASTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GOLD_NUGGET_FROM_SMELTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GRAY_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GRAY_DYE_FROM_CLOSED_EYEBLOSSOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_GREEN_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_HOST_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_DEEPSLATE_IRON_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_IRON_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_BLASTING_RAW_IRON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_IRON_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_NUGGETS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_DEEPSLATE_IRON_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_IRON_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_INGOT_FROM_SMELTING_RAW_IRON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_NUGGET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_NUGGET_FROM_BLASTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_IRON_NUGGET_FROM_SMELTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LAPIS_LAZULI , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_DEEPSLATE_LAPIS_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LAPIS_LAZULI_FROM_BLASTING_LAPIS_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_DEEPSLATE_LAPIS_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LAPIS_LAZULI_FROM_SMELTING_LAPIS_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEAF_LITTER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER_BOOTS_DYED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER_CHESTPLATE_DYED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER_HELMET_DYED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER_HORSE_ARMOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER_HORSE_ARMOR_DYED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LEATHER_LEGGINGS_DYED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_ORCHID , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_BLUE_DYE_FROM_BLUE_WHITE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_AZURE_BLUET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_BLACK_WHITE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_GRAY_WHITE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_OXEYE_DAISY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIGHT_GRAY_DYE_FROM_WHITE_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIME_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_LIME_DYE_FROM_SMELTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAGENTA_DYE_FROM_ALLIUM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_PINK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAGENTA_DYE_FROM_BLUE_RED_WHITE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAGENTA_DYE_FROM_LILAC , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAGENTA_DYE_FROM_PURPLE_AND_PINK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MAP_CLONING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MELON_SEEDS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MOJANG_BANNER_PATTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_MUSIC_DISC_5 , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_NETHER_BRICK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_NETHERITE_INGOT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_NETHERITE_INGOT_FROM_NETHERITE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_NETHERITE_SCRAP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_NETHERITE_SCRAP_FROM_BLASTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_NETHERITE_UPGRADE_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_ORANGE_DYE_FROM_OPEN_EYEBLOSSOM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_ORANGE_DYE_FROM_ORANGE_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_ORANGE_DYE_FROM_RED_YELLOW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_ORANGE_DYE_FROM_TORCHFLOWER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PAPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PINK_DYE_FROM_CACTUS_FLOWER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PINK_DYE_FROM_PEONY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PINK_DYE_FROM_PINK_PETALS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PINK_DYE_FROM_PINK_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PINK_DYE_FROM_RED_WHITE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_POPPED_CHORUS_FRUIT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PUMPKIN_SEEDS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_PURPLE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_QUARTZ , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_QUARTZ_FROM_BLASTING , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RAISER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RAW_COPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RAW_GOLD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RAW_IRON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RED_DYE_FROM_BEETROOT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RED_DYE_FROM_POPPY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RED_DYE_FROM_ROSE_BUSH , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RED_DYE_FROM_TULIP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RESIN_BRICK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RESIN_CLUMP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_RIB_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SKULL_BANNER_PATTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SLIME_BALL , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_STICK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_STICK_FROM_BAMBOO_ITEM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SUGAR_FROM_HONEY_BOTTLE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_SUGAR_FROM_SUGAR_CANE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_TIDE_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_TIPPED_ARROW , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_VEX_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WARD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WHEAT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WHITE_DYE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WHITE_DYE_FROM_LILY_OF_THE_VALLEY , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WILD_ARMOR_TRIM_SMITHING_TEMPLATE_SMITHING_TRIM , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WIND_CHARGE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WOLF_ARMOR_DYED , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_WRITABLE_BOOK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_YELLOW_DYE_FROM_DANDELION , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_YELLOW_DYE_FROM_GOLDEN_DANDELION , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_YELLOW_DYE_FROM_SUNFLOWER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_MISC_YELLOW_DYE_FROM_WILDFLOWERS , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_ACACIA_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_ACACIA_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_ACACIA_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_ACACIA_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_ACACIA_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BAMBOO_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BAMBOO_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BAMBOO_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BAMBOO_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BAMBOO_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BIRCH_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BIRCH_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BIRCH_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BIRCH_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_BIRCH_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CALIBRATED_SCULK_SENSOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CHERRY_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CHERRY_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CHERRY_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CHERRY_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CHERRY_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_COMPARATOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_COPPER_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_COPPER_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CRAFTER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CRIMSON_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CRIMSON_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CRIMSON_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CRIMSON_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_CRIMSON_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DARK_OAK_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DARK_OAK_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DARK_OAK_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DARK_OAK_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DARK_OAK_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DAYLIGHT_DETECTOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DISPENSER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_DROPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_EXPOSED_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_HEAVY_WEIGHTED_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_HONEY_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_HOPPER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_IRON_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_IRON_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_JUNGLE_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_JUNGLE_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_JUNGLE_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_JUNGLE_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_JUNGLE_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_LECTERN , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_LEVER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_LIGHT_WEIGHTED_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_LIGHTNING_ROD , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_MANGROVE_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_MANGROVE_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_MANGROVE_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_MANGROVE_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_MANGROVE_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_NOTE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OAK_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OAK_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OAK_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OAK_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OAK_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OBSERVER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_OXIDIZED_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_PALE_OAK_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_PALE_OAK_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_PALE_OAK_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_PALE_OAK_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_PALE_OAK_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_PISTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_POLISHED_BLACKSTONE_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_POLISHED_BLACKSTONE_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_DEEPSLATE_REDSTONE_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_FROM_BLASTING_REDSTONE_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_DEEPSLATE_REDSTONE_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_FROM_SMELTING_REDSTONE_ORE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_LAMP , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REDSTONE_TORCH , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_REPEATER , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_SLIME_BLOCK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_SPRUCE_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_SPRUCE_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_SPRUCE_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_SPRUCE_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_SPRUCE_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_STICKY_PISTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_STONE_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_STONE_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_TARGET , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_TNT , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_TRAPPED_CHEST , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_TRIPWIRE_HOOK , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WARPED_BUTTON , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WARPED_DOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WARPED_FENCE_GATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WARPED_PRESSURE_PLATE , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WARPED_TRAPDOOR , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_COPPER_BULB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_COPPER_DOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_COPPER_TRAPDOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_BULB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_DOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_EXPOSED_COPPER_TRAPDOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_BULB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_DOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_OXIDIZED_COPPER_TRAPDOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_BULB_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_DOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WAXED_WEATHERED_COPPER_TRAPDOOR_FROM_HONEYCOMB , } , AdvancementNode { parent : Some (20usize) , children : vec ! [] , value : Advancement :: RECIPES_REDSTONE_WEATHERED_COPPER_BULB , } , AdvancementNode { parent : Some (107usize) , children : vec ! [1645usize] , value : Advancement :: STORY_MINE_STONE , } , AdvancementNode { parent : Some (1644usize) , children : vec ! [1672usize] , value : Advancement :: STORY_UPGRADE_TOOLS , } , AdvancementNode { parent : Some (122usize) , children : vec ! [] , value : Advancement :: ADVENTURE_ARBALISTIC , } , AdvancementNode { parent : Some (121usize) , children : vec ! [] , value : Advancement :: ADVENTURE_BLOWBACK , } , AdvancementNode { parent : Some (126usize) , children : vec ! [] , value : Advancement :: ADVENTURE_BULLSEYE , } , AdvancementNode { parent : Some (121usize) , children : vec ! [] , value : Advancement :: ADVENTURE_LIGHTEN_UP , } , AdvancementNode { parent : Some (135usize) , children : vec ! [] , value : Advancement :: ADVENTURE_REVAULTING , } , AdvancementNode { parent : Some (129usize) , children : vec ! [] , value : Advancement :: ADVENTURE_SPYGLASS_AT_DRAGON , } , AdvancementNode { parent : Some (139usize) , children : vec ! [] , value : Advancement :: END_DRAGON_BREATH , } , AdvancementNode { parent : Some (139usize) , children : vec ! [] , value : Advancement :: END_DRAGON_EGG , } , AdvancementNode { parent : Some (139usize) , children : vec ! [1655usize] , value : Advancement :: END_ENTER_END_GATEWAY , } , AdvancementNode { parent : Some (1654usize) , children : vec ! [1656usize , 1673usize] , value : Advancement :: END_FIND_END_CITY , } , AdvancementNode { parent : Some (1655usize) , children : vec ! [] , value : Advancement :: END_LEVITATE , } , AdvancementNode { parent : Some (141usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_ALLAY_DELIVER_CAKE_TO_NOTE_BLOCK , } , AdvancementNode { parent : Some (153usize) , children : vec ! [1663usize] , value : Advancement :: HUSBANDRY_AXOLOTL_IN_A_BUCKET , } , AdvancementNode { parent : Some (149usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_BALANCED_DIET , } , AdvancementNode { parent : Some (142usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_BRED_ALL_ANIMALS , } , AdvancementNode { parent : Some (147usize) , children : vec ! [1665usize] , value : Advancement :: HUSBANDRY_FEED_SNIFFLET , } , AdvancementNode { parent : Some (145usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_FROGLIGHTS , } , AdvancementNode { parent : Some (1658usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_KILL_AXOLOTL_TARGET , } , AdvancementNode { parent : Some (149usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_OBTAIN_NETHERITE_HOE , } , AdvancementNode { parent : Some (1661usize) , children : vec ! [] , value : Advancement :: HUSBANDRY_PLANT_ANY_SNIFFER_SEED , } , AdvancementNode { parent : Some (162usize) , children : vec ! [1674usize] , value : Advancement :: NETHER_BREW_POTION , } , AdvancementNode { parent : Some (163usize) , children : vec ! [] , value : Advancement :: NETHER_CHARGE_RESPAWN_ANCHOR , } , AdvancementNode { parent : Some (167usize) , children : vec ! [1669usize] , value : Advancement :: NETHER_CREATE_BEACON , } , AdvancementNode { parent : Some (1668usize) , children : vec ! [] , value : Advancement :: NETHER_CREATE_FULL_BEACON , } , AdvancementNode { parent : Some (165usize) , children : vec ! [] , value : Advancement :: NETHER_EXPLORE_NETHER , } , AdvancementNode { parent : Some (161usize) , children : vec ! [] , value : Advancement :: NETHER_NETHERITE_ARMOR , } , AdvancementNode { parent : Some (1645usize) , children : vec ! [1675usize , 1676usize , 1678usize] , value : Advancement :: STORY_SMELT_IRON , } , AdvancementNode { parent : Some (1655usize) , children : vec ! [] , value : Advancement :: END_ELYTRA , } , AdvancementNode { parent : Some (1666usize) , children : vec ! [1680usize] , value : Advancement :: NETHER_ALL_POTIONS , } , AdvancementNode { parent : Some (1672usize) , children : vec ! [1677usize] , value : Advancement :: STORY_IRON_TOOLS , } , AdvancementNode { parent : Some (1672usize) , children : vec ! [1683usize] , value : Advancement :: STORY_LAVA_BUCKET , } , AdvancementNode { parent : Some (1675usize) , children : vec ! [1679usize , 1682usize] , value : Advancement :: STORY_MINE_DIAMOND , } , AdvancementNode { parent : Some (1672usize) , children : vec ! [1681usize] , value : Advancement :: STORY_OBTAIN_ARMOR , } , AdvancementNode { parent : Some (1677usize) , children : vec ! [] , value : Advancement :: STORY_SHINY_GEAR , } , AdvancementNode { parent : Some (1674usize) , children : vec ! [] , value : Advancement :: NETHER_ALL_EFFECTS , } , AdvancementNode { parent : Some (1678usize) , children : vec ! [] , value : Advancement :: STORY_DEFLECT_ARROW , } , AdvancementNode { parent : Some (1677usize) , children : vec ! [] , value : Advancement :: STORY_ENCHANT_ITEM , } , AdvancementNode { parent : Some (1676usize) , children : vec ! [1684usize] , value : Advancement :: STORY_FORM_OBSIDIAN , } , AdvancementNode { parent : Some (1683usize) , children : vec ! [1685usize , 1686usize] , value : Advancement :: STORY_ENTER_THE_NETHER , } , AdvancementNode { parent : Some (1684usize) , children : vec ! [1687usize] , value : Advancement :: STORY_FOLLOW_ENDER_EYE , } , AdvancementNode { parent : Some (1684usize) , children : vec ! [] , value : Advancement :: STORY_CURE_ZOMBIE_VILLAGER , } , AdvancementNode { parent : Some (1685usize) , children : vec ! [] , value : Advancement :: STORY_ENTER_THE_END , }] ;
    let roots = vec![0usize, 10usize, 11usize, 19usize, 20usize, 107usize];
    let tasks = vec![
        1usize, 2usize, 3usize, 4usize, 5usize, 6usize, 7usize, 8usize, 9usize, 12usize, 13usize,
        14usize, 15usize, 16usize, 17usize, 18usize, 21usize, 22usize, 23usize, 24usize, 25usize,
        26usize, 27usize, 28usize, 29usize, 30usize, 31usize, 32usize, 33usize, 34usize, 35usize,
        36usize, 37usize, 38usize, 39usize, 40usize, 41usize, 42usize, 43usize, 44usize, 45usize,
        46usize, 47usize, 48usize, 49usize, 50usize, 51usize, 52usize, 53usize, 54usize, 55usize,
        56usize, 57usize, 58usize, 59usize, 60usize, 61usize, 62usize, 63usize, 64usize, 65usize,
        66usize, 67usize, 68usize, 69usize, 70usize, 71usize, 72usize, 73usize, 74usize, 75usize,
        76usize, 77usize, 78usize, 79usize, 80usize, 81usize, 82usize, 83usize, 84usize, 85usize,
        86usize, 87usize, 88usize, 89usize, 90usize, 91usize, 92usize, 93usize, 94usize, 95usize,
        96usize, 97usize, 98usize, 99usize, 100usize, 101usize, 102usize, 103usize, 104usize,
        105usize, 106usize, 108usize, 109usize, 110usize, 111usize, 112usize, 113usize, 114usize,
        115usize, 116usize, 117usize, 118usize, 119usize, 120usize, 121usize, 122usize, 123usize,
        124usize, 125usize, 126usize, 127usize, 128usize, 129usize, 130usize, 131usize, 132usize,
        133usize, 134usize, 135usize, 136usize, 137usize, 138usize, 139usize, 140usize, 141usize,
        142usize, 143usize, 144usize, 145usize, 146usize, 147usize, 148usize, 149usize, 150usize,
        151usize, 152usize, 153usize, 154usize, 155usize, 156usize, 157usize, 158usize, 159usize,
        160usize, 161usize, 162usize, 163usize, 164usize, 165usize, 166usize, 167usize, 168usize,
        169usize, 170usize, 171usize, 172usize, 173usize, 174usize, 175usize, 176usize, 177usize,
        178usize, 179usize, 180usize, 181usize, 182usize, 183usize, 184usize, 185usize, 186usize,
        187usize, 188usize, 189usize, 190usize, 191usize, 192usize, 193usize, 194usize, 195usize,
        196usize, 197usize, 198usize, 199usize, 200usize, 201usize, 202usize, 203usize, 204usize,
        205usize, 206usize, 207usize, 208usize, 209usize, 210usize, 211usize, 212usize, 213usize,
        214usize, 215usize, 216usize, 217usize, 218usize, 219usize, 220usize, 221usize, 222usize,
        223usize, 224usize, 225usize, 226usize, 227usize, 228usize, 229usize, 230usize, 231usize,
        232usize, 233usize, 234usize, 235usize, 236usize, 237usize, 238usize, 239usize, 240usize,
        241usize, 242usize, 243usize, 244usize, 245usize, 246usize, 247usize, 248usize, 249usize,
        250usize, 251usize, 252usize, 253usize, 254usize, 255usize, 256usize, 257usize, 258usize,
        259usize, 260usize, 261usize, 262usize, 263usize, 264usize, 265usize, 266usize, 267usize,
        268usize, 269usize, 270usize, 271usize, 272usize, 273usize, 274usize, 275usize, 276usize,
        277usize, 278usize, 279usize, 280usize, 281usize, 282usize, 283usize, 284usize, 285usize,
        286usize, 287usize, 288usize, 289usize, 290usize, 291usize, 292usize, 293usize, 294usize,
        295usize, 296usize, 297usize, 298usize, 299usize, 300usize, 301usize, 302usize, 303usize,
        304usize, 305usize, 306usize, 307usize, 308usize, 309usize, 310usize, 311usize, 312usize,
        313usize, 314usize, 315usize, 316usize, 317usize, 318usize, 319usize, 320usize, 321usize,
        322usize, 323usize, 324usize, 325usize, 326usize, 327usize, 328usize, 329usize, 330usize,
        331usize, 332usize, 333usize, 334usize, 335usize, 336usize, 337usize, 338usize, 339usize,
        340usize, 341usize, 342usize, 343usize, 344usize, 345usize, 346usize, 347usize, 348usize,
        349usize, 350usize, 351usize, 352usize, 353usize, 354usize, 355usize, 356usize, 357usize,
        358usize, 359usize, 360usize, 361usize, 362usize, 363usize, 364usize, 365usize, 366usize,
        367usize, 368usize, 369usize, 370usize, 371usize, 372usize, 373usize, 374usize, 375usize,
        376usize, 377usize, 378usize, 379usize, 380usize, 381usize, 382usize, 383usize, 384usize,
        385usize, 386usize, 387usize, 388usize, 389usize, 390usize, 391usize, 392usize, 393usize,
        394usize, 395usize, 396usize, 397usize, 398usize, 399usize, 400usize, 401usize, 402usize,
        403usize, 404usize, 405usize, 406usize, 407usize, 408usize, 409usize, 410usize, 411usize,
        412usize, 413usize, 414usize, 415usize, 416usize, 417usize, 418usize, 419usize, 420usize,
        421usize, 422usize, 423usize, 424usize, 425usize, 426usize, 427usize, 428usize, 429usize,
        430usize, 431usize, 432usize, 433usize, 434usize, 435usize, 436usize, 437usize, 438usize,
        439usize, 440usize, 441usize, 442usize, 443usize, 444usize, 445usize, 446usize, 447usize,
        448usize, 449usize, 450usize, 451usize, 452usize, 453usize, 454usize, 455usize, 456usize,
        457usize, 458usize, 459usize, 460usize, 461usize, 462usize, 463usize, 464usize, 465usize,
        466usize, 467usize, 468usize, 469usize, 470usize, 471usize, 472usize, 473usize, 474usize,
        475usize, 476usize, 477usize, 478usize, 479usize, 480usize, 481usize, 482usize, 483usize,
        484usize, 485usize, 486usize, 487usize, 488usize, 489usize, 490usize, 491usize, 492usize,
        493usize, 494usize, 495usize, 496usize, 497usize, 498usize, 499usize, 500usize, 501usize,
        502usize, 503usize, 504usize, 505usize, 506usize, 507usize, 508usize, 509usize, 510usize,
        511usize, 512usize, 513usize, 514usize, 515usize, 516usize, 517usize, 518usize, 519usize,
        520usize, 521usize, 522usize, 523usize, 524usize, 525usize, 526usize, 527usize, 528usize,
        529usize, 530usize, 531usize, 532usize, 533usize, 534usize, 535usize, 536usize, 537usize,
        538usize, 539usize, 540usize, 541usize, 542usize, 543usize, 544usize, 545usize, 546usize,
        547usize, 548usize, 549usize, 550usize, 551usize, 552usize, 553usize, 554usize, 555usize,
        556usize, 557usize, 558usize, 559usize, 560usize, 561usize, 562usize, 563usize, 564usize,
        565usize, 566usize, 567usize, 568usize, 569usize, 570usize, 571usize, 572usize, 573usize,
        574usize, 575usize, 576usize, 577usize, 578usize, 579usize, 580usize, 581usize, 582usize,
        583usize, 584usize, 585usize, 586usize, 587usize, 588usize, 589usize, 590usize, 591usize,
        592usize, 593usize, 594usize, 595usize, 596usize, 597usize, 598usize, 599usize, 600usize,
        601usize, 602usize, 603usize, 604usize, 605usize, 606usize, 607usize, 608usize, 609usize,
        610usize, 611usize, 612usize, 613usize, 614usize, 615usize, 616usize, 617usize, 618usize,
        619usize, 620usize, 621usize, 622usize, 623usize, 624usize, 625usize, 626usize, 627usize,
        628usize, 629usize, 630usize, 631usize, 632usize, 633usize, 634usize, 635usize, 636usize,
        637usize, 638usize, 639usize, 640usize, 641usize, 642usize, 643usize, 644usize, 645usize,
        646usize, 647usize, 648usize, 649usize, 650usize, 651usize, 652usize, 653usize, 654usize,
        655usize, 656usize, 657usize, 658usize, 659usize, 660usize, 661usize, 662usize, 663usize,
        664usize, 665usize, 666usize, 667usize, 668usize, 669usize, 670usize, 671usize, 672usize,
        673usize, 674usize, 675usize, 676usize, 677usize, 678usize, 679usize, 680usize, 681usize,
        682usize, 683usize, 684usize, 685usize, 686usize, 687usize, 688usize, 689usize, 690usize,
        691usize, 692usize, 693usize, 694usize, 695usize, 696usize, 697usize, 698usize, 699usize,
        700usize, 701usize, 702usize, 703usize, 704usize, 705usize, 706usize, 707usize, 708usize,
        709usize, 710usize, 711usize, 712usize, 713usize, 714usize, 715usize, 716usize, 717usize,
        718usize, 719usize, 720usize, 721usize, 722usize, 723usize, 724usize, 725usize, 726usize,
        727usize, 728usize, 729usize, 730usize, 731usize, 732usize, 733usize, 734usize, 735usize,
        736usize, 737usize, 738usize, 739usize, 740usize, 741usize, 742usize, 743usize, 744usize,
        745usize, 746usize, 747usize, 748usize, 749usize, 750usize, 751usize, 752usize, 753usize,
        754usize, 755usize, 756usize, 757usize, 758usize, 759usize, 760usize, 761usize, 762usize,
        763usize, 764usize, 765usize, 766usize, 767usize, 768usize, 769usize, 770usize, 771usize,
        772usize, 773usize, 774usize, 775usize, 776usize, 777usize, 778usize, 779usize, 780usize,
        781usize, 782usize, 783usize, 784usize, 785usize, 786usize, 787usize, 788usize, 789usize,
        790usize, 791usize, 792usize, 793usize, 794usize, 795usize, 796usize, 797usize, 798usize,
        799usize, 800usize, 801usize, 802usize, 803usize, 804usize, 805usize, 806usize, 807usize,
        808usize, 809usize, 810usize, 811usize, 812usize, 813usize, 814usize, 815usize, 816usize,
        817usize, 818usize, 819usize, 820usize, 821usize, 822usize, 823usize, 824usize, 825usize,
        826usize, 827usize, 828usize, 829usize, 830usize, 831usize, 832usize, 833usize, 834usize,
        835usize, 836usize, 837usize, 838usize, 839usize, 840usize, 841usize, 842usize, 843usize,
        844usize, 845usize, 846usize, 847usize, 848usize, 849usize, 850usize, 851usize, 852usize,
        853usize, 854usize, 855usize, 856usize, 857usize, 858usize, 859usize, 860usize, 861usize,
        862usize, 863usize, 864usize, 865usize, 866usize, 867usize, 868usize, 869usize, 870usize,
        871usize, 872usize, 873usize, 874usize, 875usize, 876usize, 877usize, 878usize, 879usize,
        880usize, 881usize, 882usize, 883usize, 884usize, 885usize, 886usize, 887usize, 888usize,
        889usize, 890usize, 891usize, 892usize, 893usize, 894usize, 895usize, 896usize, 897usize,
        898usize, 899usize, 900usize, 901usize, 902usize, 903usize, 904usize, 905usize, 906usize,
        907usize, 908usize, 909usize, 910usize, 911usize, 912usize, 913usize, 914usize, 915usize,
        916usize, 917usize, 918usize, 919usize, 920usize, 921usize, 922usize, 923usize, 924usize,
        925usize, 926usize, 927usize, 928usize, 929usize, 930usize, 931usize, 932usize, 933usize,
        934usize, 935usize, 936usize, 937usize, 938usize, 939usize, 940usize, 941usize, 942usize,
        943usize, 944usize, 945usize, 946usize, 947usize, 948usize, 949usize, 950usize, 951usize,
        952usize, 953usize, 954usize, 955usize, 956usize, 957usize, 958usize, 959usize, 960usize,
        961usize, 962usize, 963usize, 964usize, 965usize, 966usize, 967usize, 968usize, 969usize,
        970usize, 971usize, 972usize, 973usize, 974usize, 975usize, 976usize, 977usize, 978usize,
        979usize, 980usize, 981usize, 982usize, 983usize, 984usize, 985usize, 986usize, 987usize,
        988usize, 989usize, 990usize, 991usize, 992usize, 993usize, 994usize, 995usize, 996usize,
        997usize, 998usize, 999usize, 1000usize, 1001usize, 1002usize, 1003usize, 1004usize,
        1005usize, 1006usize, 1007usize, 1008usize, 1009usize, 1010usize, 1011usize, 1012usize,
        1013usize, 1014usize, 1015usize, 1016usize, 1017usize, 1018usize, 1019usize, 1020usize,
        1021usize, 1022usize, 1023usize, 1024usize, 1025usize, 1026usize, 1027usize, 1028usize,
        1029usize, 1030usize, 1031usize, 1032usize, 1033usize, 1034usize, 1035usize, 1036usize,
        1037usize, 1038usize, 1039usize, 1040usize, 1041usize, 1042usize, 1043usize, 1044usize,
        1045usize, 1046usize, 1047usize, 1048usize, 1049usize, 1050usize, 1051usize, 1052usize,
        1053usize, 1054usize, 1055usize, 1056usize, 1057usize, 1058usize, 1059usize, 1060usize,
        1061usize, 1062usize, 1063usize, 1064usize, 1065usize, 1066usize, 1067usize, 1068usize,
        1069usize, 1070usize, 1071usize, 1072usize, 1073usize, 1074usize, 1075usize, 1076usize,
        1077usize, 1078usize, 1079usize, 1080usize, 1081usize, 1082usize, 1083usize, 1084usize,
        1085usize, 1086usize, 1087usize, 1088usize, 1089usize, 1090usize, 1091usize, 1092usize,
        1093usize, 1094usize, 1095usize, 1096usize, 1097usize, 1098usize, 1099usize, 1100usize,
        1101usize, 1102usize, 1103usize, 1104usize, 1105usize, 1106usize, 1107usize, 1108usize,
        1109usize, 1110usize, 1111usize, 1112usize, 1113usize, 1114usize, 1115usize, 1116usize,
        1117usize, 1118usize, 1119usize, 1120usize, 1121usize, 1122usize, 1123usize, 1124usize,
        1125usize, 1126usize, 1127usize, 1128usize, 1129usize, 1130usize, 1131usize, 1132usize,
        1133usize, 1134usize, 1135usize, 1136usize, 1137usize, 1138usize, 1139usize, 1140usize,
        1141usize, 1142usize, 1143usize, 1144usize, 1145usize, 1146usize, 1147usize, 1148usize,
        1149usize, 1150usize, 1151usize, 1152usize, 1153usize, 1154usize, 1155usize, 1156usize,
        1157usize, 1158usize, 1159usize, 1160usize, 1161usize, 1162usize, 1163usize, 1164usize,
        1165usize, 1166usize, 1167usize, 1168usize, 1169usize, 1170usize, 1171usize, 1172usize,
        1173usize, 1174usize, 1175usize, 1176usize, 1177usize, 1178usize, 1179usize, 1180usize,
        1181usize, 1182usize, 1183usize, 1184usize, 1185usize, 1186usize, 1187usize, 1188usize,
        1189usize, 1190usize, 1191usize, 1192usize, 1193usize, 1194usize, 1195usize, 1196usize,
        1197usize, 1198usize, 1199usize, 1200usize, 1201usize, 1202usize, 1203usize, 1204usize,
        1205usize, 1206usize, 1207usize, 1208usize, 1209usize, 1210usize, 1211usize, 1212usize,
        1213usize, 1214usize, 1215usize, 1216usize, 1217usize, 1218usize, 1219usize, 1220usize,
        1221usize, 1222usize, 1223usize, 1224usize, 1225usize, 1226usize, 1227usize, 1228usize,
        1229usize, 1230usize, 1231usize, 1232usize, 1233usize, 1234usize, 1235usize, 1236usize,
        1237usize, 1238usize, 1239usize, 1240usize, 1241usize, 1242usize, 1243usize, 1244usize,
        1245usize, 1246usize, 1247usize, 1248usize, 1249usize, 1250usize, 1251usize, 1252usize,
        1253usize, 1254usize, 1255usize, 1256usize, 1257usize, 1258usize, 1259usize, 1260usize,
        1261usize, 1262usize, 1263usize, 1264usize, 1265usize, 1266usize, 1267usize, 1268usize,
        1269usize, 1270usize, 1271usize, 1272usize, 1273usize, 1274usize, 1275usize, 1276usize,
        1277usize, 1278usize, 1279usize, 1280usize, 1281usize, 1282usize, 1283usize, 1284usize,
        1285usize, 1286usize, 1287usize, 1288usize, 1289usize, 1290usize, 1291usize, 1292usize,
        1293usize, 1294usize, 1295usize, 1296usize, 1297usize, 1298usize, 1299usize, 1300usize,
        1301usize, 1302usize, 1303usize, 1304usize, 1305usize, 1306usize, 1307usize, 1308usize,
        1309usize, 1310usize, 1311usize, 1312usize, 1313usize, 1314usize, 1315usize, 1316usize,
        1317usize, 1318usize, 1319usize, 1320usize, 1321usize, 1322usize, 1323usize, 1324usize,
        1325usize, 1326usize, 1327usize, 1328usize, 1329usize, 1330usize, 1331usize, 1332usize,
        1333usize, 1334usize, 1335usize, 1336usize, 1337usize, 1338usize, 1339usize, 1340usize,
        1341usize, 1342usize, 1343usize, 1344usize, 1345usize, 1346usize, 1347usize, 1348usize,
        1349usize, 1350usize, 1351usize, 1352usize, 1353usize, 1354usize, 1355usize, 1356usize,
        1357usize, 1358usize, 1359usize, 1360usize, 1361usize, 1362usize, 1363usize, 1364usize,
        1365usize, 1366usize, 1367usize, 1368usize, 1369usize, 1370usize, 1371usize, 1372usize,
        1373usize, 1374usize, 1375usize, 1376usize, 1377usize, 1378usize, 1379usize, 1380usize,
        1381usize, 1382usize, 1383usize, 1384usize, 1385usize, 1386usize, 1387usize, 1388usize,
        1389usize, 1390usize, 1391usize, 1392usize, 1393usize, 1394usize, 1395usize, 1396usize,
        1397usize, 1398usize, 1399usize, 1400usize, 1401usize, 1402usize, 1403usize, 1404usize,
        1405usize, 1406usize, 1407usize, 1408usize, 1409usize, 1410usize, 1411usize, 1412usize,
        1413usize, 1414usize, 1415usize, 1416usize, 1417usize, 1418usize, 1419usize, 1420usize,
        1421usize, 1422usize, 1423usize, 1424usize, 1425usize, 1426usize, 1427usize, 1428usize,
        1429usize, 1430usize, 1431usize, 1432usize, 1433usize, 1434usize, 1435usize, 1436usize,
        1437usize, 1438usize, 1439usize, 1440usize, 1441usize, 1442usize, 1443usize, 1444usize,
        1445usize, 1446usize, 1447usize, 1448usize, 1449usize, 1450usize, 1451usize, 1452usize,
        1453usize, 1454usize, 1455usize, 1456usize, 1457usize, 1458usize, 1459usize, 1460usize,
        1461usize, 1462usize, 1463usize, 1464usize, 1465usize, 1466usize, 1467usize, 1468usize,
        1469usize, 1470usize, 1471usize, 1472usize, 1473usize, 1474usize, 1475usize, 1476usize,
        1477usize, 1478usize, 1479usize, 1480usize, 1481usize, 1482usize, 1483usize, 1484usize,
        1485usize, 1486usize, 1487usize, 1488usize, 1489usize, 1490usize, 1491usize, 1492usize,
        1493usize, 1494usize, 1495usize, 1496usize, 1497usize, 1498usize, 1499usize, 1500usize,
        1501usize, 1502usize, 1503usize, 1504usize, 1505usize, 1506usize, 1507usize, 1508usize,
        1509usize, 1510usize, 1511usize, 1512usize, 1513usize, 1514usize, 1515usize, 1516usize,
        1517usize, 1518usize, 1519usize, 1520usize, 1521usize, 1522usize, 1523usize, 1524usize,
        1525usize, 1526usize, 1527usize, 1528usize, 1529usize, 1530usize, 1531usize, 1532usize,
        1533usize, 1534usize, 1535usize, 1536usize, 1537usize, 1538usize, 1539usize, 1540usize,
        1541usize, 1542usize, 1543usize, 1544usize, 1545usize, 1546usize, 1547usize, 1548usize,
        1549usize, 1550usize, 1551usize, 1552usize, 1553usize, 1554usize, 1555usize, 1556usize,
        1557usize, 1558usize, 1559usize, 1560usize, 1561usize, 1562usize, 1563usize, 1564usize,
        1565usize, 1566usize, 1567usize, 1568usize, 1569usize, 1570usize, 1571usize, 1572usize,
        1573usize, 1574usize, 1575usize, 1576usize, 1577usize, 1578usize, 1579usize, 1580usize,
        1581usize, 1582usize, 1583usize, 1584usize, 1585usize, 1586usize, 1587usize, 1588usize,
        1589usize, 1590usize, 1591usize, 1592usize, 1593usize, 1594usize, 1595usize, 1596usize,
        1597usize, 1598usize, 1599usize, 1600usize, 1601usize, 1602usize, 1603usize, 1604usize,
        1605usize, 1606usize, 1607usize, 1608usize, 1609usize, 1610usize, 1611usize, 1612usize,
        1613usize, 1614usize, 1615usize, 1616usize, 1617usize, 1618usize, 1619usize, 1620usize,
        1621usize, 1622usize, 1623usize, 1624usize, 1625usize, 1626usize, 1627usize, 1628usize,
        1629usize, 1630usize, 1631usize, 1632usize, 1633usize, 1634usize, 1635usize, 1636usize,
        1637usize, 1638usize, 1639usize, 1640usize, 1641usize, 1642usize, 1643usize, 1644usize,
        1645usize, 1646usize, 1647usize, 1648usize, 1649usize, 1650usize, 1651usize, 1652usize,
        1653usize, 1654usize, 1655usize, 1656usize, 1657usize, 1658usize, 1659usize, 1660usize,
        1661usize, 1662usize, 1663usize, 1664usize, 1665usize, 1666usize, 1667usize, 1668usize,
        1669usize, 1670usize, 1671usize, 1672usize, 1673usize, 1674usize, 1675usize, 1676usize,
        1677usize, 1678usize, 1679usize, 1680usize, 1681usize, 1682usize, 1683usize, 1684usize,
        1685usize, 1686usize, 1687usize,
    ];
    AdvancementTree {
        nodes,
        nodes_vector,
        roots,
        tasks,
    }
});

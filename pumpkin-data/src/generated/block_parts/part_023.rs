impl BlockId {
    pub const AIR: Self = BlockId::new(0u16).unwrap();
    pub const STONE: Self = BlockId::new(1u16).unwrap();
    pub const GRANITE: Self = BlockId::new(2u16).unwrap();
    pub const POLISHED_GRANITE: Self = BlockId::new(3u16).unwrap();
    pub const DIORITE: Self = BlockId::new(4u16).unwrap();
    pub const POLISHED_DIORITE: Self = BlockId::new(5u16).unwrap();
    pub const ANDESITE: Self = BlockId::new(6u16).unwrap();
    pub const POLISHED_ANDESITE: Self = BlockId::new(7u16).unwrap();
    pub const GRASS_BLOCK: Self = BlockId::new(8u16).unwrap();
    pub const DIRT: Self = BlockId::new(9u16).unwrap();
    pub const COARSE_DIRT: Self = BlockId::new(10u16).unwrap();
    pub const PODZOL: Self = BlockId::new(11u16).unwrap();
    pub const COBBLESTONE: Self = BlockId::new(12u16).unwrap();
    pub const OAK_PLANKS: Self = BlockId::new(13u16).unwrap();
    pub const SPRUCE_PLANKS: Self = BlockId::new(14u16).unwrap();
    pub const BIRCH_PLANKS: Self = BlockId::new(15u16).unwrap();
    pub const JUNGLE_PLANKS: Self = BlockId::new(16u16).unwrap();
    pub const ACACIA_PLANKS: Self = BlockId::new(17u16).unwrap();
    pub const CHERRY_PLANKS: Self = BlockId::new(18u16).unwrap();
    pub const DARK_OAK_PLANKS: Self = BlockId::new(19u16).unwrap();
    pub const PALE_OAK_WOOD: Self = BlockId::new(20u16).unwrap();
    pub const PALE_OAK_PLANKS: Self = BlockId::new(21u16).unwrap();
    pub const MANGROVE_PLANKS: Self = BlockId::new(22u16).unwrap();
    pub const BAMBOO_PLANKS: Self = BlockId::new(23u16).unwrap();
    pub const BAMBOO_MOSAIC: Self = BlockId::new(24u16).unwrap();
    pub const OAK_SAPLING: Self = BlockId::new(25u16).unwrap();
    pub const SPRUCE_SAPLING: Self = BlockId::new(26u16).unwrap();
    pub const BIRCH_SAPLING: Self = BlockId::new(27u16).unwrap();
    pub const JUNGLE_SAPLING: Self = BlockId::new(28u16).unwrap();
    pub const ACACIA_SAPLING: Self = BlockId::new(29u16).unwrap();
    pub const CHERRY_SAPLING: Self = BlockId::new(30u16).unwrap();
    pub const DARK_OAK_SAPLING: Self = BlockId::new(31u16).unwrap();
    pub const PALE_OAK_SAPLING: Self = BlockId::new(32u16).unwrap();
    pub const MANGROVE_PROPAGULE: Self = BlockId::new(33u16).unwrap();
    pub const BEDROCK: Self = BlockId::new(34u16).unwrap();
    pub const WATER: Self = BlockId::new(35u16).unwrap();
    pub const LAVA: Self = BlockId::new(36u16).unwrap();
    pub const SAND: Self = BlockId::new(37u16).unwrap();
    pub const SUSPICIOUS_SAND: Self = BlockId::new(38u16).unwrap();
    pub const RED_SAND: Self = BlockId::new(39u16).unwrap();
    pub const GRAVEL: Self = BlockId::new(40u16).unwrap();
    pub const SUSPICIOUS_GRAVEL: Self = BlockId::new(41u16).unwrap();
    pub const GOLD_ORE: Self = BlockId::new(42u16).unwrap();
    pub const DEEPSLATE_GOLD_ORE: Self = BlockId::new(43u16).unwrap();
    pub const IRON_ORE: Self = BlockId::new(44u16).unwrap();
    pub const DEEPSLATE_IRON_ORE: Self = BlockId::new(45u16).unwrap();
    pub const COAL_ORE: Self = BlockId::new(46u16).unwrap();
    pub const DEEPSLATE_COAL_ORE: Self = BlockId::new(47u16).unwrap();
    pub const NETHER_GOLD_ORE: Self = BlockId::new(48u16).unwrap();
    pub const OAK_LOG: Self = BlockId::new(49u16).unwrap();
    pub const SPRUCE_LOG: Self = BlockId::new(50u16).unwrap();
    pub const BIRCH_LOG: Self = BlockId::new(51u16).unwrap();
    pub const JUNGLE_LOG: Self = BlockId::new(52u16).unwrap();
    pub const ACACIA_LOG: Self = BlockId::new(53u16).unwrap();
    pub const CHERRY_LOG: Self = BlockId::new(54u16).unwrap();
    pub const DARK_OAK_LOG: Self = BlockId::new(55u16).unwrap();
    pub const PALE_OAK_LOG: Self = BlockId::new(56u16).unwrap();
    pub const MANGROVE_LOG: Self = BlockId::new(57u16).unwrap();
    pub const MANGROVE_ROOTS: Self = BlockId::new(58u16).unwrap();
    pub const MUDDY_MANGROVE_ROOTS: Self = BlockId::new(59u16).unwrap();
    pub const BAMBOO_BLOCK: Self = BlockId::new(60u16).unwrap();
    pub const STRIPPED_SPRUCE_LOG: Self = BlockId::new(61u16).unwrap();
    pub const STRIPPED_BIRCH_LOG: Self = BlockId::new(62u16).unwrap();
    pub const STRIPPED_JUNGLE_LOG: Self = BlockId::new(63u16).unwrap();
    pub const STRIPPED_ACACIA_LOG: Self = BlockId::new(64u16).unwrap();
    pub const STRIPPED_CHERRY_LOG: Self = BlockId::new(65u16).unwrap();
    pub const STRIPPED_DARK_OAK_LOG: Self = BlockId::new(66u16).unwrap();
    pub const STRIPPED_PALE_OAK_LOG: Self = BlockId::new(67u16).unwrap();
    pub const STRIPPED_OAK_LOG: Self = BlockId::new(68u16).unwrap();
    pub const STRIPPED_MANGROVE_LOG: Self = BlockId::new(69u16).unwrap();
    pub const STRIPPED_BAMBOO_BLOCK: Self = BlockId::new(70u16).unwrap();
    pub const OAK_WOOD: Self = BlockId::new(71u16).unwrap();
    pub const SPRUCE_WOOD: Self = BlockId::new(72u16).unwrap();
    pub const BIRCH_WOOD: Self = BlockId::new(73u16).unwrap();
    pub const JUNGLE_WOOD: Self = BlockId::new(74u16).unwrap();
    pub const ACACIA_WOOD: Self = BlockId::new(75u16).unwrap();
    pub const CHERRY_WOOD: Self = BlockId::new(76u16).unwrap();
    pub const DARK_OAK_WOOD: Self = BlockId::new(77u16).unwrap();
    pub const MANGROVE_WOOD: Self = BlockId::new(78u16).unwrap();
    pub const STRIPPED_OAK_WOOD: Self = BlockId::new(79u16).unwrap();
    pub const STRIPPED_SPRUCE_WOOD: Self = BlockId::new(80u16).unwrap();
    pub const STRIPPED_BIRCH_WOOD: Self = BlockId::new(81u16).unwrap();
    pub const STRIPPED_JUNGLE_WOOD: Self = BlockId::new(82u16).unwrap();
    pub const STRIPPED_ACACIA_WOOD: Self = BlockId::new(83u16).unwrap();
    pub const STRIPPED_CHERRY_WOOD: Self = BlockId::new(84u16).unwrap();
    pub const STRIPPED_DARK_OAK_WOOD: Self = BlockId::new(85u16).unwrap();
    pub const STRIPPED_PALE_OAK_WOOD: Self = BlockId::new(86u16).unwrap();
    pub const STRIPPED_MANGROVE_WOOD: Self = BlockId::new(87u16).unwrap();
    pub const OAK_LEAVES: Self = BlockId::new(88u16).unwrap();
    pub const SPRUCE_LEAVES: Self = BlockId::new(89u16).unwrap();
    pub const BIRCH_LEAVES: Self = BlockId::new(90u16).unwrap();
    pub const JUNGLE_LEAVES: Self = BlockId::new(91u16).unwrap();
    pub const ACACIA_LEAVES: Self = BlockId::new(92u16).unwrap();
    pub const CHERRY_LEAVES: Self = BlockId::new(93u16).unwrap();
    pub const DARK_OAK_LEAVES: Self = BlockId::new(94u16).unwrap();
    pub const PALE_OAK_LEAVES: Self = BlockId::new(95u16).unwrap();
    pub const MANGROVE_LEAVES: Self = BlockId::new(96u16).unwrap();
    pub const AZALEA_LEAVES: Self = BlockId::new(97u16).unwrap();
    pub const FLOWERING_AZALEA_LEAVES: Self = BlockId::new(98u16).unwrap();
    pub const SPONGE: Self = BlockId::new(99u16).unwrap();
    pub const WET_SPONGE: Self = BlockId::new(100u16).unwrap();
    pub const GLASS: Self = BlockId::new(101u16).unwrap();
    pub const LAPIS_ORE: Self = BlockId::new(102u16).unwrap();
    pub const DEEPSLATE_LAPIS_ORE: Self = BlockId::new(103u16).unwrap();
    pub const LAPIS_BLOCK: Self = BlockId::new(104u16).unwrap();
    pub const DISPENSER: Self = BlockId::new(105u16).unwrap();
    pub const SANDSTONE: Self = BlockId::new(106u16).unwrap();
    pub const CHISELED_SANDSTONE: Self = BlockId::new(107u16).unwrap();
    pub const CUT_SANDSTONE: Self = BlockId::new(108u16).unwrap();
    pub const NOTE_BLOCK: Self = BlockId::new(109u16).unwrap();
    pub const WHITE_BED: Self = BlockId::new(110u16).unwrap();
    pub const ORANGE_BED: Self = BlockId::new(111u16).unwrap();
    pub const MAGENTA_BED: Self = BlockId::new(112u16).unwrap();
    pub const LIGHT_BLUE_BED: Self = BlockId::new(113u16).unwrap();
    pub const YELLOW_BED: Self = BlockId::new(114u16).unwrap();
    pub const LIME_BED: Self = BlockId::new(115u16).unwrap();
    pub const PINK_BED: Self = BlockId::new(116u16).unwrap();
    pub const GRAY_BED: Self = BlockId::new(117u16).unwrap();
    pub const LIGHT_GRAY_BED: Self = BlockId::new(118u16).unwrap();
    pub const CYAN_BED: Self = BlockId::new(119u16).unwrap();
    pub const PURPLE_BED: Self = BlockId::new(120u16).unwrap();
    pub const BLUE_BED: Self = BlockId::new(121u16).unwrap();
    pub const BROWN_BED: Self = BlockId::new(122u16).unwrap();
    pub const GREEN_BED: Self = BlockId::new(123u16).unwrap();
    pub const RED_BED: Self = BlockId::new(124u16).unwrap();
    pub const BLACK_BED: Self = BlockId::new(125u16).unwrap();
    pub const POWERED_RAIL: Self = BlockId::new(126u16).unwrap();
    pub const DETECTOR_RAIL: Self = BlockId::new(127u16).unwrap();
    pub const STICKY_PISTON: Self = BlockId::new(128u16).unwrap();
    pub const COBWEB: Self = BlockId::new(129u16).unwrap();
    pub const SHORT_GRASS: Self = BlockId::new(130u16).unwrap();
    pub const FERN: Self = BlockId::new(131u16).unwrap();
    pub const DEAD_BUSH: Self = BlockId::new(132u16).unwrap();
    pub const BUSH: Self = BlockId::new(133u16).unwrap();
    pub const SHORT_DRY_GRASS: Self = BlockId::new(134u16).unwrap();
    pub const TALL_DRY_GRASS: Self = BlockId::new(135u16).unwrap();
    pub const SEAGRASS: Self = BlockId::new(136u16).unwrap();
    pub const TALL_SEAGRASS: Self = BlockId::new(137u16).unwrap();
    pub const PISTON: Self = BlockId::new(138u16).unwrap();
    pub const PISTON_HEAD: Self = BlockId::new(139u16).unwrap();
    pub const WHITE_WOOL: Self = BlockId::new(140u16).unwrap();
    pub const ORANGE_WOOL: Self = BlockId::new(141u16).unwrap();
    pub const MAGENTA_WOOL: Self = BlockId::new(142u16).unwrap();
    pub const LIGHT_BLUE_WOOL: Self = BlockId::new(143u16).unwrap();
    pub const YELLOW_WOOL: Self = BlockId::new(144u16).unwrap();
    pub const LIME_WOOL: Self = BlockId::new(145u16).unwrap();
    pub const PINK_WOOL: Self = BlockId::new(146u16).unwrap();
    pub const GRAY_WOOL: Self = BlockId::new(147u16).unwrap();
    pub const LIGHT_GRAY_WOOL: Self = BlockId::new(148u16).unwrap();
    pub const CYAN_WOOL: Self = BlockId::new(149u16).unwrap();
    pub const PURPLE_WOOL: Self = BlockId::new(150u16).unwrap();
    pub const BLUE_WOOL: Self = BlockId::new(151u16).unwrap();
    pub const BROWN_WOOL: Self = BlockId::new(152u16).unwrap();
    pub const GREEN_WOOL: Self = BlockId::new(153u16).unwrap();
    pub const RED_WOOL: Self = BlockId::new(154u16).unwrap();
    pub const BLACK_WOOL: Self = BlockId::new(155u16).unwrap();
    pub const MOVING_PISTON: Self = BlockId::new(156u16).unwrap();
    pub const DANDELION: Self = BlockId::new(157u16).unwrap();
    pub const GOLDEN_DANDELION: Self = BlockId::new(158u16).unwrap();
    pub const TORCHFLOWER: Self = BlockId::new(159u16).unwrap();
    pub const POPPY: Self = BlockId::new(160u16).unwrap();
    pub const BLUE_ORCHID: Self = BlockId::new(161u16).unwrap();
    pub const ALLIUM: Self = BlockId::new(162u16).unwrap();
    pub const AZURE_BLUET: Self = BlockId::new(163u16).unwrap();
    pub const RED_TULIP: Self = BlockId::new(164u16).unwrap();
    pub const ORANGE_TULIP: Self = BlockId::new(165u16).unwrap();
    pub const WHITE_TULIP: Self = BlockId::new(166u16).unwrap();
    pub const PINK_TULIP: Self = BlockId::new(167u16).unwrap();
    pub const OXEYE_DAISY: Self = BlockId::new(168u16).unwrap();
    pub const CORNFLOWER: Self = BlockId::new(169u16).unwrap();
    pub const WITHER_ROSE: Self = BlockId::new(170u16).unwrap();
    pub const LILY_OF_THE_VALLEY: Self = BlockId::new(171u16).unwrap();
    pub const BROWN_MUSHROOM: Self = BlockId::new(172u16).unwrap();
    pub const RED_MUSHROOM: Self = BlockId::new(173u16).unwrap();
    pub const GOLD_BLOCK: Self = BlockId::new(174u16).unwrap();
    pub const IRON_BLOCK: Self = BlockId::new(175u16).unwrap();
    pub const BRICKS: Self = BlockId::new(176u16).unwrap();
    pub const TNT: Self = BlockId::new(177u16).unwrap();
    pub const BOOKSHELF: Self = BlockId::new(178u16).unwrap();
    pub const CHISELED_BOOKSHELF: Self = BlockId::new(179u16).unwrap();
    pub const ACACIA_SHELF: Self = BlockId::new(180u16).unwrap();
    pub const BAMBOO_SHELF: Self = BlockId::new(181u16).unwrap();
    pub const BIRCH_SHELF: Self = BlockId::new(182u16).unwrap();
    pub const CHERRY_SHELF: Self = BlockId::new(183u16).unwrap();
    pub const CRIMSON_SHELF: Self = BlockId::new(184u16).unwrap();
    pub const DARK_OAK_SHELF: Self = BlockId::new(185u16).unwrap();
    pub const JUNGLE_SHELF: Self = BlockId::new(186u16).unwrap();
    pub const MANGROVE_SHELF: Self = BlockId::new(187u16).unwrap();
    pub const OAK_SHELF: Self = BlockId::new(188u16).unwrap();
    pub const PALE_OAK_SHELF: Self = BlockId::new(189u16).unwrap();
    pub const SPRUCE_SHELF: Self = BlockId::new(190u16).unwrap();
    pub const WARPED_SHELF: Self = BlockId::new(191u16).unwrap();
    pub const MOSSY_COBBLESTONE: Self = BlockId::new(192u16).unwrap();
    pub const OBSIDIAN: Self = BlockId::new(193u16).unwrap();
    pub const TORCH: Self = BlockId::new(194u16).unwrap();
    pub const WALL_TORCH: Self = BlockId::new(195u16).unwrap();
    pub const FIRE: Self = BlockId::new(196u16).unwrap();
    pub const SOUL_FIRE: Self = BlockId::new(197u16).unwrap();
    pub const SPAWNER: Self = BlockId::new(198u16).unwrap();
    pub const CREAKING_HEART: Self = BlockId::new(199u16).unwrap();
    pub const OAK_STAIRS: Self = BlockId::new(200u16).unwrap();
    pub const CHEST: Self = BlockId::new(201u16).unwrap();
    pub const REDSTONE_WIRE: Self = BlockId::new(202u16).unwrap();
    pub const DIAMOND_ORE: Self = BlockId::new(203u16).unwrap();
    pub const DEEPSLATE_DIAMOND_ORE: Self = BlockId::new(204u16).unwrap();
    pub const DIAMOND_BLOCK: Self = BlockId::new(205u16).unwrap();
    pub const CRAFTING_TABLE: Self = BlockId::new(206u16).unwrap();
    pub const WHEAT: Self = BlockId::new(207u16).unwrap();
    pub const FARMLAND: Self = BlockId::new(208u16).unwrap();
    pub const FURNACE: Self = BlockId::new(209u16).unwrap();
    pub const OAK_SIGN: Self = BlockId::new(210u16).unwrap();
    pub const SPRUCE_SIGN: Self = BlockId::new(211u16).unwrap();
    pub const BIRCH_SIGN: Self = BlockId::new(212u16).unwrap();
    pub const ACACIA_SIGN: Self = BlockId::new(213u16).unwrap();
    pub const CHERRY_SIGN: Self = BlockId::new(214u16).unwrap();
    pub const JUNGLE_SIGN: Self = BlockId::new(215u16).unwrap();
    pub const DARK_OAK_SIGN: Self = BlockId::new(216u16).unwrap();
    pub const PALE_OAK_SIGN: Self = BlockId::new(217u16).unwrap();
    pub const MANGROVE_SIGN: Self = BlockId::new(218u16).unwrap();
    pub const BAMBOO_SIGN: Self = BlockId::new(219u16).unwrap();
    pub const OAK_DOOR: Self = BlockId::new(220u16).unwrap();
    pub const LADDER: Self = BlockId::new(221u16).unwrap();
    pub const RAIL: Self = BlockId::new(222u16).unwrap();
    pub const COBBLESTONE_STAIRS: Self = BlockId::new(223u16).unwrap();
    pub const OAK_WALL_SIGN: Self = BlockId::new(224u16).unwrap();
    pub const SPRUCE_WALL_SIGN: Self = BlockId::new(225u16).unwrap();
    pub const BIRCH_WALL_SIGN: Self = BlockId::new(226u16).unwrap();
    pub const ACACIA_WALL_SIGN: Self = BlockId::new(227u16).unwrap();
    pub const CHERRY_WALL_SIGN: Self = BlockId::new(228u16).unwrap();
    pub const JUNGLE_WALL_SIGN: Self = BlockId::new(229u16).unwrap();
    pub const DARK_OAK_WALL_SIGN: Self = BlockId::new(230u16).unwrap();
    pub const PALE_OAK_WALL_SIGN: Self = BlockId::new(231u16).unwrap();
    pub const MANGROVE_WALL_SIGN: Self = BlockId::new(232u16).unwrap();
    pub const BAMBOO_WALL_SIGN: Self = BlockId::new(233u16).unwrap();
    pub const OAK_HANGING_SIGN: Self = BlockId::new(234u16).unwrap();
    pub const SPRUCE_HANGING_SIGN: Self = BlockId::new(235u16).unwrap();
    pub const BIRCH_HANGING_SIGN: Self = BlockId::new(236u16).unwrap();
    pub const ACACIA_HANGING_SIGN: Self = BlockId::new(237u16).unwrap();
    pub const CHERRY_HANGING_SIGN: Self = BlockId::new(238u16).unwrap();
    pub const JUNGLE_HANGING_SIGN: Self = BlockId::new(239u16).unwrap();
    pub const DARK_OAK_HANGING_SIGN: Self = BlockId::new(240u16).unwrap();
    pub const PALE_OAK_HANGING_SIGN: Self = BlockId::new(241u16).unwrap();
    pub const CRIMSON_HANGING_SIGN: Self = BlockId::new(242u16).unwrap();
    pub const WARPED_HANGING_SIGN: Self = BlockId::new(243u16).unwrap();
    pub const MANGROVE_HANGING_SIGN: Self = BlockId::new(244u16).unwrap();
    pub const BAMBOO_HANGING_SIGN: Self = BlockId::new(245u16).unwrap();
    pub const OAK_WALL_HANGING_SIGN: Self = BlockId::new(246u16).unwrap();
    pub const SPRUCE_WALL_HANGING_SIGN: Self = BlockId::new(247u16).unwrap();
    pub const BIRCH_WALL_HANGING_SIGN: Self = BlockId::new(248u16).unwrap();
    pub const ACACIA_WALL_HANGING_SIGN: Self = BlockId::new(249u16).unwrap();
    pub const CHERRY_WALL_HANGING_SIGN: Self = BlockId::new(250u16).unwrap();
    pub const JUNGLE_WALL_HANGING_SIGN: Self = BlockId::new(251u16).unwrap();
    pub const DARK_OAK_WALL_HANGING_SIGN: Self = BlockId::new(252u16).unwrap();
    pub const PALE_OAK_WALL_HANGING_SIGN: Self = BlockId::new(253u16).unwrap();
    pub const MANGROVE_WALL_HANGING_SIGN: Self = BlockId::new(254u16).unwrap();
    pub const CRIMSON_WALL_HANGING_SIGN: Self = BlockId::new(255u16).unwrap();
    pub const WARPED_WALL_HANGING_SIGN: Self = BlockId::new(256u16).unwrap();
    pub const BAMBOO_WALL_HANGING_SIGN: Self = BlockId::new(257u16).unwrap();
    pub const LEVER: Self = BlockId::new(258u16).unwrap();
    pub const STONE_PRESSURE_PLATE: Self = BlockId::new(259u16).unwrap();
    pub const IRON_DOOR: Self = BlockId::new(260u16).unwrap();
    pub const OAK_PRESSURE_PLATE: Self = BlockId::new(261u16).unwrap();
    pub const SPRUCE_PRESSURE_PLATE: Self = BlockId::new(262u16).unwrap();
    pub const BIRCH_PRESSURE_PLATE: Self = BlockId::new(263u16).unwrap();
    pub const JUNGLE_PRESSURE_PLATE: Self = BlockId::new(264u16).unwrap();
    pub const ACACIA_PRESSURE_PLATE: Self = BlockId::new(265u16).unwrap();
    pub const CHERRY_PRESSURE_PLATE: Self = BlockId::new(266u16).unwrap();
    pub const DARK_OAK_PRESSURE_PLATE: Self = BlockId::new(267u16).unwrap();
    pub const PALE_OAK_PRESSURE_PLATE: Self = BlockId::new(268u16).unwrap();
    pub const MANGROVE_PRESSURE_PLATE: Self = BlockId::new(269u16).unwrap();
    pub const BAMBOO_PRESSURE_PLATE: Self = BlockId::new(270u16).unwrap();
    pub const REDSTONE_ORE: Self = BlockId::new(271u16).unwrap();
    pub const DEEPSLATE_REDSTONE_ORE: Self = BlockId::new(272u16).unwrap();
    pub const REDSTONE_TORCH: Self = BlockId::new(273u16).unwrap();
    pub const REDSTONE_WALL_TORCH: Self = BlockId::new(274u16).unwrap();
    pub const STONE_BUTTON: Self = BlockId::new(275u16).unwrap();
    pub const SNOW: Self = BlockId::new(276u16).unwrap();
    pub const ICE: Self = BlockId::new(277u16).unwrap();
    pub const SNOW_BLOCK: Self = BlockId::new(278u16).unwrap();
    pub const CACTUS: Self = BlockId::new(279u16).unwrap();
    pub const CACTUS_FLOWER: Self = BlockId::new(280u16).unwrap();
    pub const CLAY: Self = BlockId::new(281u16).unwrap();
    pub const SUGAR_CANE: Self = BlockId::new(282u16).unwrap();
    pub const JUKEBOX: Self = BlockId::new(283u16).unwrap();
    pub const OAK_FENCE: Self = BlockId::new(284u16).unwrap();
    pub const NETHERRACK: Self = BlockId::new(285u16).unwrap();
    pub const SOUL_SAND: Self = BlockId::new(286u16).unwrap();
    pub const SOUL_SOIL: Self = BlockId::new(287u16).unwrap();
    pub const BASALT: Self = BlockId::new(288u16).unwrap();
    pub const POLISHED_BASALT: Self = BlockId::new(289u16).unwrap();
    pub const SOUL_TORCH: Self = BlockId::new(290u16).unwrap();
    pub const SOUL_WALL_TORCH: Self = BlockId::new(291u16).unwrap();
    pub const COPPER_TORCH: Self = BlockId::new(292u16).unwrap();
    pub const COPPER_WALL_TORCH: Self = BlockId::new(293u16).unwrap();
    pub const GLOWSTONE: Self = BlockId::new(294u16).unwrap();
    pub const NETHER_PORTAL: Self = BlockId::new(295u16).unwrap();
    pub const CARVED_PUMPKIN: Self = BlockId::new(296u16).unwrap();
    pub const JACK_O_LANTERN: Self = BlockId::new(297u16).unwrap();
    pub const CAKE: Self = BlockId::new(298u16).unwrap();
    pub const REPEATER: Self = BlockId::new(299u16).unwrap();
    pub const WHITE_STAINED_GLASS: Self = BlockId::new(300u16).unwrap();
    pub const ORANGE_STAINED_GLASS: Self = BlockId::new(301u16).unwrap();
    pub const MAGENTA_STAINED_GLASS: Self = BlockId::new(302u16).unwrap();
    pub const LIGHT_BLUE_STAINED_GLASS: Self = BlockId::new(303u16).unwrap();
    pub const YELLOW_STAINED_GLASS: Self = BlockId::new(304u16).unwrap();
    pub const LIME_STAINED_GLASS: Self = BlockId::new(305u16).unwrap();
    pub const PINK_STAINED_GLASS: Self = BlockId::new(306u16).unwrap();
    pub const GRAY_STAINED_GLASS: Self = BlockId::new(307u16).unwrap();
    pub const LIGHT_GRAY_STAINED_GLASS: Self = BlockId::new(308u16).unwrap();
    pub const CYAN_STAINED_GLASS: Self = BlockId::new(309u16).unwrap();
    pub const PURPLE_STAINED_GLASS: Self = BlockId::new(310u16).unwrap();
    pub const BLUE_STAINED_GLASS: Self = BlockId::new(311u16).unwrap();
    pub const BROWN_STAINED_GLASS: Self = BlockId::new(312u16).unwrap();
    pub const GREEN_STAINED_GLASS: Self = BlockId::new(313u16).unwrap();
    pub const RED_STAINED_GLASS: Self = BlockId::new(314u16).unwrap();
    pub const BLACK_STAINED_GLASS: Self = BlockId::new(315u16).unwrap();
    pub const OAK_TRAPDOOR: Self = BlockId::new(316u16).unwrap();
    pub const SPRUCE_TRAPDOOR: Self = BlockId::new(317u16).unwrap();
    pub const BIRCH_TRAPDOOR: Self = BlockId::new(318u16).unwrap();
    pub const JUNGLE_TRAPDOOR: Self = BlockId::new(319u16).unwrap();
    pub const ACACIA_TRAPDOOR: Self = BlockId::new(320u16).unwrap();
    pub const CHERRY_TRAPDOOR: Self = BlockId::new(321u16).unwrap();
    pub const DARK_OAK_TRAPDOOR: Self = BlockId::new(322u16).unwrap();
    pub const PALE_OAK_TRAPDOOR: Self = BlockId::new(323u16).unwrap();
    pub const MANGROVE_TRAPDOOR: Self = BlockId::new(324u16).unwrap();
    pub const BAMBOO_TRAPDOOR: Self = BlockId::new(325u16).unwrap();
    pub const STONE_BRICKS: Self = BlockId::new(326u16).unwrap();
    pub const MOSSY_STONE_BRICKS: Self = BlockId::new(327u16).unwrap();
    pub const CRACKED_STONE_BRICKS: Self = BlockId::new(328u16).unwrap();
    pub const CHISELED_STONE_BRICKS: Self = BlockId::new(329u16).unwrap();
    pub const PACKED_MUD: Self = BlockId::new(330u16).unwrap();
    pub const MUD_BRICKS: Self = BlockId::new(331u16).unwrap();
    pub const INFESTED_STONE: Self = BlockId::new(332u16).unwrap();
    pub const INFESTED_COBBLESTONE: Self = BlockId::new(333u16).unwrap();
    pub const INFESTED_STONE_BRICKS: Self = BlockId::new(334u16).unwrap();
    pub const INFESTED_MOSSY_STONE_BRICKS: Self = BlockId::new(335u16).unwrap();
    pub const INFESTED_CRACKED_STONE_BRICKS: Self = BlockId::new(336u16).unwrap();
    pub const INFESTED_CHISELED_STONE_BRICKS: Self = BlockId::new(337u16).unwrap();
    pub const BROWN_MUSHROOM_BLOCK: Self = BlockId::new(338u16).unwrap();
    pub const RED_MUSHROOM_BLOCK: Self = BlockId::new(339u16).unwrap();
    pub const MUSHROOM_STEM: Self = BlockId::new(340u16).unwrap();
    pub const IRON_BARS: Self = BlockId::new(341u16).unwrap();
    pub const COPPER_BARS: Self = BlockId::new(342u16).unwrap();
    pub const EXPOSED_COPPER_BARS: Self = BlockId::new(343u16).unwrap();
    pub const WEATHERED_COPPER_BARS: Self = BlockId::new(344u16).unwrap();
    pub const OXIDIZED_COPPER_BARS: Self = BlockId::new(345u16).unwrap();
    pub const WAXED_COPPER_BARS: Self = BlockId::new(346u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_BARS: Self = BlockId::new(347u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_BARS: Self = BlockId::new(348u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_BARS: Self = BlockId::new(349u16).unwrap();
    pub const IRON_CHAIN: Self = BlockId::new(350u16).unwrap();
    pub const COPPER_CHAIN: Self = BlockId::new(351u16).unwrap();
    pub const EXPOSED_COPPER_CHAIN: Self = BlockId::new(352u16).unwrap();
    pub const WEATHERED_COPPER_CHAIN: Self = BlockId::new(353u16).unwrap();
    pub const OXIDIZED_COPPER_CHAIN: Self = BlockId::new(354u16).unwrap();
    pub const WAXED_COPPER_CHAIN: Self = BlockId::new(355u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_CHAIN: Self = BlockId::new(356u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_CHAIN: Self = BlockId::new(357u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_CHAIN: Self = BlockId::new(358u16).unwrap();
    pub const GLASS_PANE: Self = BlockId::new(359u16).unwrap();
    pub const PUMPKIN: Self = BlockId::new(360u16).unwrap();
    pub const MELON: Self = BlockId::new(361u16).unwrap();
    pub const ATTACHED_PUMPKIN_STEM: Self = BlockId::new(362u16).unwrap();
    pub const ATTACHED_MELON_STEM: Self = BlockId::new(363u16).unwrap();
    pub const PUMPKIN_STEM: Self = BlockId::new(364u16).unwrap();
    pub const MELON_STEM: Self = BlockId::new(365u16).unwrap();
    pub const VINE: Self = BlockId::new(366u16).unwrap();
    pub const GLOW_LICHEN: Self = BlockId::new(367u16).unwrap();
    pub const RESIN_CLUMP: Self = BlockId::new(368u16).unwrap();
    pub const OAK_FENCE_GATE: Self = BlockId::new(369u16).unwrap();
    pub const BRICK_STAIRS: Self = BlockId::new(370u16).unwrap();
    pub const STONE_BRICK_STAIRS: Self = BlockId::new(371u16).unwrap();
    pub const MUD_BRICK_STAIRS: Self = BlockId::new(372u16).unwrap();
    pub const MYCELIUM: Self = BlockId::new(373u16).unwrap();
    pub const LILY_PAD: Self = BlockId::new(374u16).unwrap();
    pub const RESIN_BLOCK: Self = BlockId::new(375u16).unwrap();
    pub const RESIN_BRICKS: Self = BlockId::new(376u16).unwrap();
    pub const RESIN_BRICK_STAIRS: Self = BlockId::new(377u16).unwrap();
    pub const RESIN_BRICK_SLAB: Self = BlockId::new(378u16).unwrap();
    pub const RESIN_BRICK_WALL: Self = BlockId::new(379u16).unwrap();
    pub const CHISELED_RESIN_BRICKS: Self = BlockId::new(380u16).unwrap();
    pub const NETHER_BRICKS: Self = BlockId::new(381u16).unwrap();
    pub const NETHER_BRICK_FENCE: Self = BlockId::new(382u16).unwrap();
    pub const NETHER_BRICK_STAIRS: Self = BlockId::new(383u16).unwrap();
    pub const NETHER_WART: Self = BlockId::new(384u16).unwrap();
    pub const ENCHANTING_TABLE: Self = BlockId::new(385u16).unwrap();
    pub const BREWING_STAND: Self = BlockId::new(386u16).unwrap();
    pub const CAULDRON: Self = BlockId::new(387u16).unwrap();
    pub const WATER_CAULDRON: Self = BlockId::new(388u16).unwrap();
    pub const LAVA_CAULDRON: Self = BlockId::new(389u16).unwrap();
    pub const POWDER_SNOW_CAULDRON: Self = BlockId::new(390u16).unwrap();
    pub const END_PORTAL: Self = BlockId::new(391u16).unwrap();
    pub const END_PORTAL_FRAME: Self = BlockId::new(392u16).unwrap();
    pub const END_STONE: Self = BlockId::new(393u16).unwrap();
    pub const DRAGON_EGG: Self = BlockId::new(394u16).unwrap();
    pub const REDSTONE_LAMP: Self = BlockId::new(395u16).unwrap();
    pub const COCOA: Self = BlockId::new(396u16).unwrap();
    pub const SANDSTONE_STAIRS: Self = BlockId::new(397u16).unwrap();
    pub const EMERALD_ORE: Self = BlockId::new(398u16).unwrap();
    pub const DEEPSLATE_EMERALD_ORE: Self = BlockId::new(399u16).unwrap();
    pub const ENDER_CHEST: Self = BlockId::new(400u16).unwrap();
    pub const TRIPWIRE_HOOK: Self = BlockId::new(401u16).unwrap();
    pub const TRIPWIRE: Self = BlockId::new(402u16).unwrap();
    pub const EMERALD_BLOCK: Self = BlockId::new(403u16).unwrap();
    pub const SPRUCE_STAIRS: Self = BlockId::new(404u16).unwrap();
    pub const BIRCH_STAIRS: Self = BlockId::new(405u16).unwrap();
    pub const JUNGLE_STAIRS: Self = BlockId::new(406u16).unwrap();
    pub const COMMAND_BLOCK: Self = BlockId::new(407u16).unwrap();
    pub const BEACON: Self = BlockId::new(408u16).unwrap();
    pub const COBBLESTONE_WALL: Self = BlockId::new(409u16).unwrap();
    pub const MOSSY_COBBLESTONE_WALL: Self = BlockId::new(410u16).unwrap();
    pub const FLOWER_POT: Self = BlockId::new(411u16).unwrap();
    pub const POTTED_TORCHFLOWER: Self = BlockId::new(412u16).unwrap();
    pub const POTTED_OAK_SAPLING: Self = BlockId::new(413u16).unwrap();
    pub const POTTED_SPRUCE_SAPLING: Self = BlockId::new(414u16).unwrap();
    pub const POTTED_BIRCH_SAPLING: Self = BlockId::new(415u16).unwrap();
    pub const POTTED_JUNGLE_SAPLING: Self = BlockId::new(416u16).unwrap();
    pub const POTTED_ACACIA_SAPLING: Self = BlockId::new(417u16).unwrap();
    pub const POTTED_CHERRY_SAPLING: Self = BlockId::new(418u16).unwrap();
    pub const POTTED_DARK_OAK_SAPLING: Self = BlockId::new(419u16).unwrap();
    pub const POTTED_PALE_OAK_SAPLING: Self = BlockId::new(420u16).unwrap();
    pub const POTTED_MANGROVE_PROPAGULE: Self = BlockId::new(421u16).unwrap();
    pub const POTTED_FERN: Self = BlockId::new(422u16).unwrap();
    pub const POTTED_DANDELION: Self = BlockId::new(423u16).unwrap();
    pub const POTTED_GOLDEN_DANDELION: Self = BlockId::new(424u16).unwrap();
    pub const POTTED_POPPY: Self = BlockId::new(425u16).unwrap();
    pub const POTTED_BLUE_ORCHID: Self = BlockId::new(426u16).unwrap();
    pub const POTTED_ALLIUM: Self = BlockId::new(427u16).unwrap();
    pub const POTTED_AZURE_BLUET: Self = BlockId::new(428u16).unwrap();
    pub const POTTED_RED_TULIP: Self = BlockId::new(429u16).unwrap();
    pub const POTTED_ORANGE_TULIP: Self = BlockId::new(430u16).unwrap();
    pub const POTTED_WHITE_TULIP: Self = BlockId::new(431u16).unwrap();
    pub const POTTED_PINK_TULIP: Self = BlockId::new(432u16).unwrap();
    pub const POTTED_OXEYE_DAISY: Self = BlockId::new(433u16).unwrap();
    pub const POTTED_CORNFLOWER: Self = BlockId::new(434u16).unwrap();
    pub const POTTED_LILY_OF_THE_VALLEY: Self = BlockId::new(435u16).unwrap();
    pub const POTTED_WITHER_ROSE: Self = BlockId::new(436u16).unwrap();
    pub const POTTED_RED_MUSHROOM: Self = BlockId::new(437u16).unwrap();
    pub const POTTED_BROWN_MUSHROOM: Self = BlockId::new(438u16).unwrap();
    pub const POTTED_DEAD_BUSH: Self = BlockId::new(439u16).unwrap();
    pub const POTTED_CACTUS: Self = BlockId::new(440u16).unwrap();
    pub const CARROTS: Self = BlockId::new(441u16).unwrap();
    pub const POTATOES: Self = BlockId::new(442u16).unwrap();
    pub const OAK_BUTTON: Self = BlockId::new(443u16).unwrap();
    pub const SPRUCE_BUTTON: Self = BlockId::new(444u16).unwrap();
    pub const BIRCH_BUTTON: Self = BlockId::new(445u16).unwrap();
    pub const JUNGLE_BUTTON: Self = BlockId::new(446u16).unwrap();
    pub const ACACIA_BUTTON: Self = BlockId::new(447u16).unwrap();
    pub const CHERRY_BUTTON: Self = BlockId::new(448u16).unwrap();
    pub const DARK_OAK_BUTTON: Self = BlockId::new(449u16).unwrap();
    pub const PALE_OAK_BUTTON: Self = BlockId::new(450u16).unwrap();
    pub const MANGROVE_BUTTON: Self = BlockId::new(451u16).unwrap();
    pub const BAMBOO_BUTTON: Self = BlockId::new(452u16).unwrap();
    pub const SKELETON_SKULL: Self = BlockId::new(453u16).unwrap();
    pub const SKELETON_WALL_SKULL: Self = BlockId::new(454u16).unwrap();
    pub const WITHER_SKELETON_SKULL: Self = BlockId::new(455u16).unwrap();
    pub const WITHER_SKELETON_WALL_SKULL: Self = BlockId::new(456u16).unwrap();
    pub const ZOMBIE_HEAD: Self = BlockId::new(457u16).unwrap();
    pub const ZOMBIE_WALL_HEAD: Self = BlockId::new(458u16).unwrap();
    pub const PLAYER_HEAD: Self = BlockId::new(459u16).unwrap();
    pub const PLAYER_WALL_HEAD: Self = BlockId::new(460u16).unwrap();
    pub const CREEPER_HEAD: Self = BlockId::new(461u16).unwrap();
    pub const CREEPER_WALL_HEAD: Self = BlockId::new(462u16).unwrap();
    pub const DRAGON_HEAD: Self = BlockId::new(463u16).unwrap();
    pub const DRAGON_WALL_HEAD: Self = BlockId::new(464u16).unwrap();
    pub const PIGLIN_HEAD: Self = BlockId::new(465u16).unwrap();
    pub const PIGLIN_WALL_HEAD: Self = BlockId::new(466u16).unwrap();
    pub const ANVIL: Self = BlockId::new(467u16).unwrap();
    pub const CHIPPED_ANVIL: Self = BlockId::new(468u16).unwrap();
    pub const DAMAGED_ANVIL: Self = BlockId::new(469u16).unwrap();
    pub const TRAPPED_CHEST: Self = BlockId::new(470u16).unwrap();
    pub const LIGHT_WEIGHTED_PRESSURE_PLATE: Self = BlockId::new(471u16).unwrap();
    pub const HEAVY_WEIGHTED_PRESSURE_PLATE: Self = BlockId::new(472u16).unwrap();
    pub const COMPARATOR: Self = BlockId::new(473u16).unwrap();
    pub const DAYLIGHT_DETECTOR: Self = BlockId::new(474u16).unwrap();
    pub const REDSTONE_BLOCK: Self = BlockId::new(475u16).unwrap();
    pub const NETHER_QUARTZ_ORE: Self = BlockId::new(476u16).unwrap();
    pub const HOPPER: Self = BlockId::new(477u16).unwrap();
    pub const QUARTZ_BLOCK: Self = BlockId::new(478u16).unwrap();
    pub const CHISELED_QUARTZ_BLOCK: Self = BlockId::new(479u16).unwrap();
    pub const QUARTZ_PILLAR: Self = BlockId::new(480u16).unwrap();
    pub const QUARTZ_STAIRS: Self = BlockId::new(481u16).unwrap();
    pub const ACTIVATOR_RAIL: Self = BlockId::new(482u16).unwrap();
    pub const DROPPER: Self = BlockId::new(483u16).unwrap();
    pub const WHITE_TERRACOTTA: Self = BlockId::new(484u16).unwrap();
    pub const ORANGE_TERRACOTTA: Self = BlockId::new(485u16).unwrap();
    pub const MAGENTA_TERRACOTTA: Self = BlockId::new(486u16).unwrap();
    pub const LIGHT_BLUE_TERRACOTTA: Self = BlockId::new(487u16).unwrap();
    pub const YELLOW_TERRACOTTA: Self = BlockId::new(488u16).unwrap();
    pub const LIME_TERRACOTTA: Self = BlockId::new(489u16).unwrap();
    pub const PINK_TERRACOTTA: Self = BlockId::new(490u16).unwrap();
    pub const GRAY_TERRACOTTA: Self = BlockId::new(491u16).unwrap();
    pub const LIGHT_GRAY_TERRACOTTA: Self = BlockId::new(492u16).unwrap();
    pub const CYAN_TERRACOTTA: Self = BlockId::new(493u16).unwrap();
    pub const PURPLE_TERRACOTTA: Self = BlockId::new(494u16).unwrap();
    pub const BLUE_TERRACOTTA: Self = BlockId::new(495u16).unwrap();
    pub const BROWN_TERRACOTTA: Self = BlockId::new(496u16).unwrap();
    pub const GREEN_TERRACOTTA: Self = BlockId::new(497u16).unwrap();
    pub const RED_TERRACOTTA: Self = BlockId::new(498u16).unwrap();
    pub const BLACK_TERRACOTTA: Self = BlockId::new(499u16).unwrap();
    pub const WHITE_STAINED_GLASS_PANE: Self = BlockId::new(500u16).unwrap();
    pub const ORANGE_STAINED_GLASS_PANE: Self = BlockId::new(501u16).unwrap();
    pub const MAGENTA_STAINED_GLASS_PANE: Self = BlockId::new(502u16).unwrap();
    pub const LIGHT_BLUE_STAINED_GLASS_PANE: Self = BlockId::new(503u16).unwrap();
    pub const YELLOW_STAINED_GLASS_PANE: Self = BlockId::new(504u16).unwrap();
    pub const LIME_STAINED_GLASS_PANE: Self = BlockId::new(505u16).unwrap();
    pub const PINK_STAINED_GLASS_PANE: Self = BlockId::new(506u16).unwrap();
    pub const GRAY_STAINED_GLASS_PANE: Self = BlockId::new(507u16).unwrap();
    pub const LIGHT_GRAY_STAINED_GLASS_PANE: Self = BlockId::new(508u16).unwrap();
    pub const CYAN_STAINED_GLASS_PANE: Self = BlockId::new(509u16).unwrap();
    pub const PURPLE_STAINED_GLASS_PANE: Self = BlockId::new(510u16).unwrap();
    pub const BLUE_STAINED_GLASS_PANE: Self = BlockId::new(511u16).unwrap();
    pub const BROWN_STAINED_GLASS_PANE: Self = BlockId::new(512u16).unwrap();
    pub const GREEN_STAINED_GLASS_PANE: Self = BlockId::new(513u16).unwrap();
    pub const RED_STAINED_GLASS_PANE: Self = BlockId::new(514u16).unwrap();
    pub const BLACK_STAINED_GLASS_PANE: Self = BlockId::new(515u16).unwrap();
    pub const ACACIA_STAIRS: Self = BlockId::new(516u16).unwrap();
    pub const CHERRY_STAIRS: Self = BlockId::new(517u16).unwrap();
    pub const DARK_OAK_STAIRS: Self = BlockId::new(518u16).unwrap();
    pub const PALE_OAK_STAIRS: Self = BlockId::new(519u16).unwrap();
    pub const MANGROVE_STAIRS: Self = BlockId::new(520u16).unwrap();
    pub const BAMBOO_STAIRS: Self = BlockId::new(521u16).unwrap();
    pub const BAMBOO_MOSAIC_STAIRS: Self = BlockId::new(522u16).unwrap();
    pub const SLIME_BLOCK: Self = BlockId::new(523u16).unwrap();
    pub const BARRIER: Self = BlockId::new(524u16).unwrap();
    pub const LIGHT: Self = BlockId::new(525u16).unwrap();
    pub const IRON_TRAPDOOR: Self = BlockId::new(526u16).unwrap();
    pub const PRISMARINE: Self = BlockId::new(527u16).unwrap();
    pub const PRISMARINE_BRICKS: Self = BlockId::new(528u16).unwrap();
    pub const DARK_PRISMARINE: Self = BlockId::new(529u16).unwrap();
    pub const PRISMARINE_STAIRS: Self = BlockId::new(530u16).unwrap();
    pub const PRISMARINE_BRICK_STAIRS: Self = BlockId::new(531u16).unwrap();
    pub const DARK_PRISMARINE_STAIRS: Self = BlockId::new(532u16).unwrap();
    pub const PRISMARINE_SLAB: Self = BlockId::new(533u16).unwrap();
    pub const PRISMARINE_BRICK_SLAB: Self = BlockId::new(534u16).unwrap();
    pub const DARK_PRISMARINE_SLAB: Self = BlockId::new(535u16).unwrap();
    pub const SEA_LANTERN: Self = BlockId::new(536u16).unwrap();
    pub const HAY_BLOCK: Self = BlockId::new(537u16).unwrap();
    pub const WHITE_CARPET: Self = BlockId::new(538u16).unwrap();
    pub const ORANGE_CARPET: Self = BlockId::new(539u16).unwrap();
    pub const MAGENTA_CARPET: Self = BlockId::new(540u16).unwrap();
    pub const LIGHT_BLUE_CARPET: Self = BlockId::new(541u16).unwrap();
    pub const YELLOW_CARPET: Self = BlockId::new(542u16).unwrap();
    pub const LIME_CARPET: Self = BlockId::new(543u16).unwrap();
    pub const PINK_CARPET: Self = BlockId::new(544u16).unwrap();
    pub const GRAY_CARPET: Self = BlockId::new(545u16).unwrap();
    pub const LIGHT_GRAY_CARPET: Self = BlockId::new(546u16).unwrap();
    pub const CYAN_CARPET: Self = BlockId::new(547u16).unwrap();
    pub const PURPLE_CARPET: Self = BlockId::new(548u16).unwrap();
    pub const BLUE_CARPET: Self = BlockId::new(549u16).unwrap();
    pub const BROWN_CARPET: Self = BlockId::new(550u16).unwrap();
    pub const GREEN_CARPET: Self = BlockId::new(551u16).unwrap();
    pub const RED_CARPET: Self = BlockId::new(552u16).unwrap();
    pub const BLACK_CARPET: Self = BlockId::new(553u16).unwrap();
    pub const TERRACOTTA: Self = BlockId::new(554u16).unwrap();
    pub const COAL_BLOCK: Self = BlockId::new(555u16).unwrap();
    pub const PACKED_ICE: Self = BlockId::new(556u16).unwrap();
    pub const SUNFLOWER: Self = BlockId::new(557u16).unwrap();
    pub const LILAC: Self = BlockId::new(558u16).unwrap();
    pub const ROSE_BUSH: Self = BlockId::new(559u16).unwrap();
    pub const PEONY: Self = BlockId::new(560u16).unwrap();
    pub const TALL_GRASS: Self = BlockId::new(561u16).unwrap();
    pub const LARGE_FERN: Self = BlockId::new(562u16).unwrap();
    pub const WHITE_BANNER: Self = BlockId::new(563u16).unwrap();
    pub const ORANGE_BANNER: Self = BlockId::new(564u16).unwrap();
    pub const MAGENTA_BANNER: Self = BlockId::new(565u16).unwrap();
    pub const LIGHT_BLUE_BANNER: Self = BlockId::new(566u16).unwrap();
    pub const YELLOW_BANNER: Self = BlockId::new(567u16).unwrap();
    pub const LIME_BANNER: Self = BlockId::new(568u16).unwrap();
    pub const PINK_BANNER: Self = BlockId::new(569u16).unwrap();
    pub const GRAY_BANNER: Self = BlockId::new(570u16).unwrap();
    pub const LIGHT_GRAY_BANNER: Self = BlockId::new(571u16).unwrap();
    pub const CYAN_BANNER: Self = BlockId::new(572u16).unwrap();
    pub const PURPLE_BANNER: Self = BlockId::new(573u16).unwrap();
    pub const BLUE_BANNER: Self = BlockId::new(574u16).unwrap();
    pub const BROWN_BANNER: Self = BlockId::new(575u16).unwrap();
    pub const GREEN_BANNER: Self = BlockId::new(576u16).unwrap();
    pub const RED_BANNER: Self = BlockId::new(577u16).unwrap();
    pub const BLACK_BANNER: Self = BlockId::new(578u16).unwrap();
    pub const WHITE_WALL_BANNER: Self = BlockId::new(579u16).unwrap();
    pub const ORANGE_WALL_BANNER: Self = BlockId::new(580u16).unwrap();
    pub const MAGENTA_WALL_BANNER: Self = BlockId::new(581u16).unwrap();
    pub const LIGHT_BLUE_WALL_BANNER: Self = BlockId::new(582u16).unwrap();
    pub const YELLOW_WALL_BANNER: Self = BlockId::new(583u16).unwrap();
    pub const LIME_WALL_BANNER: Self = BlockId::new(584u16).unwrap();
    pub const PINK_WALL_BANNER: Self = BlockId::new(585u16).unwrap();
    pub const GRAY_WALL_BANNER: Self = BlockId::new(586u16).unwrap();
    pub const LIGHT_GRAY_WALL_BANNER: Self = BlockId::new(587u16).unwrap();
    pub const CYAN_WALL_BANNER: Self = BlockId::new(588u16).unwrap();
    pub const PURPLE_WALL_BANNER: Self = BlockId::new(589u16).unwrap();
    pub const BLUE_WALL_BANNER: Self = BlockId::new(590u16).unwrap();
    pub const BROWN_WALL_BANNER: Self = BlockId::new(591u16).unwrap();
    pub const GREEN_WALL_BANNER: Self = BlockId::new(592u16).unwrap();
    pub const RED_WALL_BANNER: Self = BlockId::new(593u16).unwrap();
    pub const BLACK_WALL_BANNER: Self = BlockId::new(594u16).unwrap();
    pub const RED_SANDSTONE: Self = BlockId::new(595u16).unwrap();
    pub const CHISELED_RED_SANDSTONE: Self = BlockId::new(596u16).unwrap();
    pub const CUT_RED_SANDSTONE: Self = BlockId::new(597u16).unwrap();
    pub const RED_SANDSTONE_STAIRS: Self = BlockId::new(598u16).unwrap();
    pub const OAK_SLAB: Self = BlockId::new(599u16).unwrap();
    pub const SPRUCE_SLAB: Self = BlockId::new(600u16).unwrap();
    pub const BIRCH_SLAB: Self = BlockId::new(601u16).unwrap();
    pub const JUNGLE_SLAB: Self = BlockId::new(602u16).unwrap();
    pub const ACACIA_SLAB: Self = BlockId::new(603u16).unwrap();
    pub const CHERRY_SLAB: Self = BlockId::new(604u16).unwrap();
    pub const DARK_OAK_SLAB: Self = BlockId::new(605u16).unwrap();
    pub const PALE_OAK_SLAB: Self = BlockId::new(606u16).unwrap();
    pub const MANGROVE_SLAB: Self = BlockId::new(607u16).unwrap();
    pub const BAMBOO_SLAB: Self = BlockId::new(608u16).unwrap();
    pub const BAMBOO_MOSAIC_SLAB: Self = BlockId::new(609u16).unwrap();
    pub const STONE_SLAB: Self = BlockId::new(610u16).unwrap();
    pub const SMOOTH_STONE_SLAB: Self = BlockId::new(611u16).unwrap();
    pub const SANDSTONE_SLAB: Self = BlockId::new(612u16).unwrap();
    pub const CUT_SANDSTONE_SLAB: Self = BlockId::new(613u16).unwrap();
    pub const PETRIFIED_OAK_SLAB: Self = BlockId::new(614u16).unwrap();
    pub const COBBLESTONE_SLAB: Self = BlockId::new(615u16).unwrap();
    pub const BRICK_SLAB: Self = BlockId::new(616u16).unwrap();
    pub const STONE_BRICK_SLAB: Self = BlockId::new(617u16).unwrap();
    pub const MUD_BRICK_SLAB: Self = BlockId::new(618u16).unwrap();
    pub const NETHER_BRICK_SLAB: Self = BlockId::new(619u16).unwrap();
    pub const QUARTZ_SLAB: Self = BlockId::new(620u16).unwrap();
    pub const RED_SANDSTONE_SLAB: Self = BlockId::new(621u16).unwrap();
    pub const CUT_RED_SANDSTONE_SLAB: Self = BlockId::new(622u16).unwrap();
    pub const PURPUR_SLAB: Self = BlockId::new(623u16).unwrap();
    pub const SMOOTH_STONE: Self = BlockId::new(624u16).unwrap();
    pub const SMOOTH_SANDSTONE: Self = BlockId::new(625u16).unwrap();
    pub const SMOOTH_QUARTZ: Self = BlockId::new(626u16).unwrap();
    pub const SMOOTH_RED_SANDSTONE: Self = BlockId::new(627u16).unwrap();
    pub const SPRUCE_FENCE_GATE: Self = BlockId::new(628u16).unwrap();
    pub const BIRCH_FENCE_GATE: Self = BlockId::new(629u16).unwrap();
    pub const JUNGLE_FENCE_GATE: Self = BlockId::new(630u16).unwrap();
    pub const ACACIA_FENCE_GATE: Self = BlockId::new(631u16).unwrap();
    pub const CHERRY_FENCE_GATE: Self = BlockId::new(632u16).unwrap();
    pub const DARK_OAK_FENCE_GATE: Self = BlockId::new(633u16).unwrap();
    pub const PALE_OAK_FENCE_GATE: Self = BlockId::new(634u16).unwrap();
    pub const MANGROVE_FENCE_GATE: Self = BlockId::new(635u16).unwrap();
    pub const BAMBOO_FENCE_GATE: Self = BlockId::new(636u16).unwrap();
    pub const SPRUCE_FENCE: Self = BlockId::new(637u16).unwrap();
    pub const BIRCH_FENCE: Self = BlockId::new(638u16).unwrap();
    pub const JUNGLE_FENCE: Self = BlockId::new(639u16).unwrap();
    pub const ACACIA_FENCE: Self = BlockId::new(640u16).unwrap();
    pub const CHERRY_FENCE: Self = BlockId::new(641u16).unwrap();
    pub const DARK_OAK_FENCE: Self = BlockId::new(642u16).unwrap();
    pub const PALE_OAK_FENCE: Self = BlockId::new(643u16).unwrap();
    pub const MANGROVE_FENCE: Self = BlockId::new(644u16).unwrap();
    pub const BAMBOO_FENCE: Self = BlockId::new(645u16).unwrap();
    pub const SPRUCE_DOOR: Self = BlockId::new(646u16).unwrap();
    pub const BIRCH_DOOR: Self = BlockId::new(647u16).unwrap();
    pub const JUNGLE_DOOR: Self = BlockId::new(648u16).unwrap();
    pub const ACACIA_DOOR: Self = BlockId::new(649u16).unwrap();
    pub const CHERRY_DOOR: Self = BlockId::new(650u16).unwrap();
    pub const DARK_OAK_DOOR: Self = BlockId::new(651u16).unwrap();
    pub const PALE_OAK_DOOR: Self = BlockId::new(652u16).unwrap();
    pub const MANGROVE_DOOR: Self = BlockId::new(653u16).unwrap();
    pub const BAMBOO_DOOR: Self = BlockId::new(654u16).unwrap();
    pub const END_ROD: Self = BlockId::new(655u16).unwrap();
    pub const CHORUS_PLANT: Self = BlockId::new(656u16).unwrap();
    pub const CHORUS_FLOWER: Self = BlockId::new(657u16).unwrap();
    pub const PURPUR_BLOCK: Self = BlockId::new(658u16).unwrap();
    pub const PURPUR_PILLAR: Self = BlockId::new(659u16).unwrap();
    pub const PURPUR_STAIRS: Self = BlockId::new(660u16).unwrap();
    pub const END_STONE_BRICKS: Self = BlockId::new(661u16).unwrap();
    pub const TORCHFLOWER_CROP: Self = BlockId::new(662u16).unwrap();
    pub const PITCHER_CROP: Self = BlockId::new(663u16).unwrap();
    pub const PITCHER_PLANT: Self = BlockId::new(664u16).unwrap();
    pub const BEETROOTS: Self = BlockId::new(665u16).unwrap();
    pub const DIRT_PATH: Self = BlockId::new(666u16).unwrap();
    pub const END_GATEWAY: Self = BlockId::new(667u16).unwrap();
    pub const REPEATING_COMMAND_BLOCK: Self = BlockId::new(668u16).unwrap();
    pub const CHAIN_COMMAND_BLOCK: Self = BlockId::new(669u16).unwrap();
    pub const FROSTED_ICE: Self = BlockId::new(670u16).unwrap();
    pub const MAGMA_BLOCK: Self = BlockId::new(671u16).unwrap();
    pub const NETHER_WART_BLOCK: Self = BlockId::new(672u16).unwrap();
    pub const RED_NETHER_BRICKS: Self = BlockId::new(673u16).unwrap();
    pub const BONE_BLOCK: Self = BlockId::new(674u16).unwrap();
    pub const STRUCTURE_VOID: Self = BlockId::new(675u16).unwrap();
    pub const OBSERVER: Self = BlockId::new(676u16).unwrap();
    pub const SHULKER_BOX: Self = BlockId::new(677u16).unwrap();
    pub const WHITE_SHULKER_BOX: Self = BlockId::new(678u16).unwrap();
    pub const ORANGE_SHULKER_BOX: Self = BlockId::new(679u16).unwrap();
    pub const MAGENTA_SHULKER_BOX: Self = BlockId::new(680u16).unwrap();
    pub const LIGHT_BLUE_SHULKER_BOX: Self = BlockId::new(681u16).unwrap();
    pub const YELLOW_SHULKER_BOX: Self = BlockId::new(682u16).unwrap();
    pub const LIME_SHULKER_BOX: Self = BlockId::new(683u16).unwrap();
    pub const PINK_SHULKER_BOX: Self = BlockId::new(684u16).unwrap();
    pub const GRAY_SHULKER_BOX: Self = BlockId::new(685u16).unwrap();
    pub const LIGHT_GRAY_SHULKER_BOX: Self = BlockId::new(686u16).unwrap();
    pub const CYAN_SHULKER_BOX: Self = BlockId::new(687u16).unwrap();
    pub const PURPLE_SHULKER_BOX: Self = BlockId::new(688u16).unwrap();
    pub const BLUE_SHULKER_BOX: Self = BlockId::new(689u16).unwrap();
    pub const BROWN_SHULKER_BOX: Self = BlockId::new(690u16).unwrap();
    pub const GREEN_SHULKER_BOX: Self = BlockId::new(691u16).unwrap();
    pub const RED_SHULKER_BOX: Self = BlockId::new(692u16).unwrap();
    pub const BLACK_SHULKER_BOX: Self = BlockId::new(693u16).unwrap();
    pub const WHITE_GLAZED_TERRACOTTA: Self = BlockId::new(694u16).unwrap();
    pub const ORANGE_GLAZED_TERRACOTTA: Self = BlockId::new(695u16).unwrap();
    pub const MAGENTA_GLAZED_TERRACOTTA: Self = BlockId::new(696u16).unwrap();
    pub const LIGHT_BLUE_GLAZED_TERRACOTTA: Self = BlockId::new(697u16).unwrap();
    pub const YELLOW_GLAZED_TERRACOTTA: Self = BlockId::new(698u16).unwrap();
    pub const LIME_GLAZED_TERRACOTTA: Self = BlockId::new(699u16).unwrap();
    pub const PINK_GLAZED_TERRACOTTA: Self = BlockId::new(700u16).unwrap();
    pub const GRAY_GLAZED_TERRACOTTA: Self = BlockId::new(701u16).unwrap();
    pub const LIGHT_GRAY_GLAZED_TERRACOTTA: Self = BlockId::new(702u16).unwrap();
    pub const CYAN_GLAZED_TERRACOTTA: Self = BlockId::new(703u16).unwrap();
    pub const PURPLE_GLAZED_TERRACOTTA: Self = BlockId::new(704u16).unwrap();
    pub const BLUE_GLAZED_TERRACOTTA: Self = BlockId::new(705u16).unwrap();
    pub const BROWN_GLAZED_TERRACOTTA: Self = BlockId::new(706u16).unwrap();
    pub const GREEN_GLAZED_TERRACOTTA: Self = BlockId::new(707u16).unwrap();
    pub const RED_GLAZED_TERRACOTTA: Self = BlockId::new(708u16).unwrap();
    pub const BLACK_GLAZED_TERRACOTTA: Self = BlockId::new(709u16).unwrap();
    pub const WHITE_CONCRETE: Self = BlockId::new(710u16).unwrap();
    pub const ORANGE_CONCRETE: Self = BlockId::new(711u16).unwrap();
    pub const MAGENTA_CONCRETE: Self = BlockId::new(712u16).unwrap();
    pub const LIGHT_BLUE_CONCRETE: Self = BlockId::new(713u16).unwrap();
    pub const YELLOW_CONCRETE: Self = BlockId::new(714u16).unwrap();
    pub const LIME_CONCRETE: Self = BlockId::new(715u16).unwrap();
    pub const PINK_CONCRETE: Self = BlockId::new(716u16).unwrap();
    pub const GRAY_CONCRETE: Self = BlockId::new(717u16).unwrap();
    pub const LIGHT_GRAY_CONCRETE: Self = BlockId::new(718u16).unwrap();
    pub const CYAN_CONCRETE: Self = BlockId::new(719u16).unwrap();
    pub const PURPLE_CONCRETE: Self = BlockId::new(720u16).unwrap();
    pub const BLUE_CONCRETE: Self = BlockId::new(721u16).unwrap();
    pub const BROWN_CONCRETE: Self = BlockId::new(722u16).unwrap();
    pub const GREEN_CONCRETE: Self = BlockId::new(723u16).unwrap();
    pub const RED_CONCRETE: Self = BlockId::new(724u16).unwrap();
    pub const BLACK_CONCRETE: Self = BlockId::new(725u16).unwrap();
    pub const WHITE_CONCRETE_POWDER: Self = BlockId::new(726u16).unwrap();
    pub const ORANGE_CONCRETE_POWDER: Self = BlockId::new(727u16).unwrap();
    pub const MAGENTA_CONCRETE_POWDER: Self = BlockId::new(728u16).unwrap();
    pub const LIGHT_BLUE_CONCRETE_POWDER: Self = BlockId::new(729u16).unwrap();
    pub const YELLOW_CONCRETE_POWDER: Self = BlockId::new(730u16).unwrap();
    pub const LIME_CONCRETE_POWDER: Self = BlockId::new(731u16).unwrap();
    pub const PINK_CONCRETE_POWDER: Self = BlockId::new(732u16).unwrap();
    pub const GRAY_CONCRETE_POWDER: Self = BlockId::new(733u16).unwrap();
    pub const LIGHT_GRAY_CONCRETE_POWDER: Self = BlockId::new(734u16).unwrap();
    pub const CYAN_CONCRETE_POWDER: Self = BlockId::new(735u16).unwrap();
    pub const PURPLE_CONCRETE_POWDER: Self = BlockId::new(736u16).unwrap();
    pub const BLUE_CONCRETE_POWDER: Self = BlockId::new(737u16).unwrap();
    pub const BROWN_CONCRETE_POWDER: Self = BlockId::new(738u16).unwrap();
    pub const GREEN_CONCRETE_POWDER: Self = BlockId::new(739u16).unwrap();
    pub const RED_CONCRETE_POWDER: Self = BlockId::new(740u16).unwrap();
    pub const BLACK_CONCRETE_POWDER: Self = BlockId::new(741u16).unwrap();
    pub const KELP: Self = BlockId::new(742u16).unwrap();
    pub const KELP_PLANT: Self = BlockId::new(743u16).unwrap();
    pub const DRIED_KELP_BLOCK: Self = BlockId::new(744u16).unwrap();
    pub const TURTLE_EGG: Self = BlockId::new(745u16).unwrap();
    pub const SNIFFER_EGG: Self = BlockId::new(746u16).unwrap();
    pub const DRIED_GHAST: Self = BlockId::new(747u16).unwrap();
    pub const DEAD_TUBE_CORAL_BLOCK: Self = BlockId::new(748u16).unwrap();
    pub const DEAD_BRAIN_CORAL_BLOCK: Self = BlockId::new(749u16).unwrap();
    pub const DEAD_BUBBLE_CORAL_BLOCK: Self = BlockId::new(750u16).unwrap();
    pub const DEAD_FIRE_CORAL_BLOCK: Self = BlockId::new(751u16).unwrap();
    pub const DEAD_HORN_CORAL_BLOCK: Self = BlockId::new(752u16).unwrap();
    pub const TUBE_CORAL_BLOCK: Self = BlockId::new(753u16).unwrap();
    pub const BRAIN_CORAL_BLOCK: Self = BlockId::new(754u16).unwrap();
    pub const BUBBLE_CORAL_BLOCK: Self = BlockId::new(755u16).unwrap();
    pub const FIRE_CORAL_BLOCK: Self = BlockId::new(756u16).unwrap();
    pub const HORN_CORAL_BLOCK: Self = BlockId::new(757u16).unwrap();
    pub const DEAD_TUBE_CORAL: Self = BlockId::new(758u16).unwrap();
    pub const DEAD_BRAIN_CORAL: Self = BlockId::new(759u16).unwrap();
    pub const DEAD_BUBBLE_CORAL: Self = BlockId::new(760u16).unwrap();
    pub const DEAD_FIRE_CORAL: Self = BlockId::new(761u16).unwrap();
    pub const DEAD_HORN_CORAL: Self = BlockId::new(762u16).unwrap();
    pub const TUBE_CORAL: Self = BlockId::new(763u16).unwrap();
    pub const BRAIN_CORAL: Self = BlockId::new(764u16).unwrap();
    pub const BUBBLE_CORAL: Self = BlockId::new(765u16).unwrap();
    pub const FIRE_CORAL: Self = BlockId::new(766u16).unwrap();
    pub const HORN_CORAL: Self = BlockId::new(767u16).unwrap();
    pub const DEAD_TUBE_CORAL_FAN: Self = BlockId::new(768u16).unwrap();
    pub const DEAD_BRAIN_CORAL_FAN: Self = BlockId::new(769u16).unwrap();
    pub const DEAD_BUBBLE_CORAL_FAN: Self = BlockId::new(770u16).unwrap();
    pub const DEAD_FIRE_CORAL_FAN: Self = BlockId::new(771u16).unwrap();
    pub const DEAD_HORN_CORAL_FAN: Self = BlockId::new(772u16).unwrap();
    pub const TUBE_CORAL_FAN: Self = BlockId::new(773u16).unwrap();
    pub const BRAIN_CORAL_FAN: Self = BlockId::new(774u16).unwrap();
    pub const BUBBLE_CORAL_FAN: Self = BlockId::new(775u16).unwrap();
    pub const FIRE_CORAL_FAN: Self = BlockId::new(776u16).unwrap();
    pub const HORN_CORAL_FAN: Self = BlockId::new(777u16).unwrap();
    pub const DEAD_TUBE_CORAL_WALL_FAN: Self = BlockId::new(778u16).unwrap();
    pub const DEAD_BRAIN_CORAL_WALL_FAN: Self = BlockId::new(779u16).unwrap();
    pub const DEAD_BUBBLE_CORAL_WALL_FAN: Self = BlockId::new(780u16).unwrap();
    pub const DEAD_FIRE_CORAL_WALL_FAN: Self = BlockId::new(781u16).unwrap();
    pub const DEAD_HORN_CORAL_WALL_FAN: Self = BlockId::new(782u16).unwrap();
    pub const TUBE_CORAL_WALL_FAN: Self = BlockId::new(783u16).unwrap();
    pub const BRAIN_CORAL_WALL_FAN: Self = BlockId::new(784u16).unwrap();
    pub const BUBBLE_CORAL_WALL_FAN: Self = BlockId::new(785u16).unwrap();
    pub const FIRE_CORAL_WALL_FAN: Self = BlockId::new(786u16).unwrap();
    pub const HORN_CORAL_WALL_FAN: Self = BlockId::new(787u16).unwrap();
    pub const SEA_PICKLE: Self = BlockId::new(788u16).unwrap();
    pub const BLUE_ICE: Self = BlockId::new(789u16).unwrap();
    pub const CONDUIT: Self = BlockId::new(790u16).unwrap();
    pub const BAMBOO_SAPLING: Self = BlockId::new(791u16).unwrap();
    pub const BAMBOO: Self = BlockId::new(792u16).unwrap();
    pub const POTTED_BAMBOO: Self = BlockId::new(793u16).unwrap();
    pub const VOID_AIR: Self = BlockId::new(794u16).unwrap();
    pub const CAVE_AIR: Self = BlockId::new(795u16).unwrap();
    pub const BUBBLE_COLUMN: Self = BlockId::new(796u16).unwrap();
    pub const POLISHED_GRANITE_STAIRS: Self = BlockId::new(797u16).unwrap();
    pub const SMOOTH_RED_SANDSTONE_STAIRS: Self = BlockId::new(798u16).unwrap();
    pub const MOSSY_STONE_BRICK_STAIRS: Self = BlockId::new(799u16).unwrap();
    pub const POLISHED_DIORITE_STAIRS: Self = BlockId::new(800u16).unwrap();
    pub const MOSSY_COBBLESTONE_STAIRS: Self = BlockId::new(801u16).unwrap();
    pub const END_STONE_BRICK_STAIRS: Self = BlockId::new(802u16).unwrap();
    pub const STONE_STAIRS: Self = BlockId::new(803u16).unwrap();
    pub const SMOOTH_SANDSTONE_STAIRS: Self = BlockId::new(804u16).unwrap();
    pub const SMOOTH_QUARTZ_STAIRS: Self = BlockId::new(805u16).unwrap();
    pub const GRANITE_STAIRS: Self = BlockId::new(806u16).unwrap();
    pub const ANDESITE_STAIRS: Self = BlockId::new(807u16).unwrap();
    pub const RED_NETHER_BRICK_STAIRS: Self = BlockId::new(808u16).unwrap();
    pub const POLISHED_ANDESITE_STAIRS: Self = BlockId::new(809u16).unwrap();
    pub const DIORITE_STAIRS: Self = BlockId::new(810u16).unwrap();
    pub const POLISHED_GRANITE_SLAB: Self = BlockId::new(811u16).unwrap();
    pub const SMOOTH_RED_SANDSTONE_SLAB: Self = BlockId::new(812u16).unwrap();
    pub const MOSSY_STONE_BRICK_SLAB: Self = BlockId::new(813u16).unwrap();
    pub const POLISHED_DIORITE_SLAB: Self = BlockId::new(814u16).unwrap();
    pub const MOSSY_COBBLESTONE_SLAB: Self = BlockId::new(815u16).unwrap();
    pub const END_STONE_BRICK_SLAB: Self = BlockId::new(816u16).unwrap();
    pub const SMOOTH_SANDSTONE_SLAB: Self = BlockId::new(817u16).unwrap();
    pub const SMOOTH_QUARTZ_SLAB: Self = BlockId::new(818u16).unwrap();
    pub const GRANITE_SLAB: Self = BlockId::new(819u16).unwrap();
    pub const ANDESITE_SLAB: Self = BlockId::new(820u16).unwrap();
    pub const RED_NETHER_BRICK_SLAB: Self = BlockId::new(821u16).unwrap();
    pub const POLISHED_ANDESITE_SLAB: Self = BlockId::new(822u16).unwrap();
    pub const DIORITE_SLAB: Self = BlockId::new(823u16).unwrap();
    pub const BRICK_WALL: Self = BlockId::new(824u16).unwrap();
    pub const PRISMARINE_WALL: Self = BlockId::new(825u16).unwrap();
    pub const RED_SANDSTONE_WALL: Self = BlockId::new(826u16).unwrap();
    pub const MOSSY_STONE_BRICK_WALL: Self = BlockId::new(827u16).unwrap();
    pub const GRANITE_WALL: Self = BlockId::new(828u16).unwrap();
    pub const STONE_BRICK_WALL: Self = BlockId::new(829u16).unwrap();
    pub const MUD_BRICK_WALL: Self = BlockId::new(830u16).unwrap();
    pub const NETHER_BRICK_WALL: Self = BlockId::new(831u16).unwrap();
    pub const ANDESITE_WALL: Self = BlockId::new(832u16).unwrap();
    pub const RED_NETHER_BRICK_WALL: Self = BlockId::new(833u16).unwrap();
    pub const SANDSTONE_WALL: Self = BlockId::new(834u16).unwrap();
    pub const END_STONE_BRICK_WALL: Self = BlockId::new(835u16).unwrap();
    pub const DIORITE_WALL: Self = BlockId::new(836u16).unwrap();
    pub const SCAFFOLDING: Self = BlockId::new(837u16).unwrap();
    pub const LOOM: Self = BlockId::new(838u16).unwrap();
    pub const BARREL: Self = BlockId::new(839u16).unwrap();
    pub const SMOKER: Self = BlockId::new(840u16).unwrap();
    pub const BLAST_FURNACE: Self = BlockId::new(841u16).unwrap();
    pub const CARTOGRAPHY_TABLE: Self = BlockId::new(842u16).unwrap();
    pub const FLETCHING_TABLE: Self = BlockId::new(843u16).unwrap();
    pub const GRINDSTONE: Self = BlockId::new(844u16).unwrap();
    pub const LECTERN: Self = BlockId::new(845u16).unwrap();
    pub const SMITHING_TABLE: Self = BlockId::new(846u16).unwrap();
    pub const STONECUTTER: Self = BlockId::new(847u16).unwrap();
    pub const BELL: Self = BlockId::new(848u16).unwrap();
    pub const LANTERN: Self = BlockId::new(849u16).unwrap();
    pub const SOUL_LANTERN: Self = BlockId::new(850u16).unwrap();
    pub const COPPER_LANTERN: Self = BlockId::new(851u16).unwrap();
    pub const EXPOSED_COPPER_LANTERN: Self = BlockId::new(852u16).unwrap();
    pub const WEATHERED_COPPER_LANTERN: Self = BlockId::new(853u16).unwrap();
    pub const OXIDIZED_COPPER_LANTERN: Self = BlockId::new(854u16).unwrap();
    pub const WAXED_COPPER_LANTERN: Self = BlockId::new(855u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_LANTERN: Self = BlockId::new(856u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_LANTERN: Self = BlockId::new(857u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_LANTERN: Self = BlockId::new(858u16).unwrap();
    pub const CAMPFIRE: Self = BlockId::new(859u16).unwrap();
    pub const SOUL_CAMPFIRE: Self = BlockId::new(860u16).unwrap();
    pub const SWEET_BERRY_BUSH: Self = BlockId::new(861u16).unwrap();
    pub const WARPED_STEM: Self = BlockId::new(862u16).unwrap();
    pub const STRIPPED_WARPED_STEM: Self = BlockId::new(863u16).unwrap();
    pub const WARPED_HYPHAE: Self = BlockId::new(864u16).unwrap();
    pub const STRIPPED_WARPED_HYPHAE: Self = BlockId::new(865u16).unwrap();
    pub const WARPED_NYLIUM: Self = BlockId::new(866u16).unwrap();
    pub const WARPED_FUNGUS: Self = BlockId::new(867u16).unwrap();
    pub const WARPED_WART_BLOCK: Self = BlockId::new(868u16).unwrap();
    pub const WARPED_ROOTS: Self = BlockId::new(869u16).unwrap();
    pub const NETHER_SPROUTS: Self = BlockId::new(870u16).unwrap();
    pub const CRIMSON_STEM: Self = BlockId::new(871u16).unwrap();
    pub const STRIPPED_CRIMSON_STEM: Self = BlockId::new(872u16).unwrap();
    pub const CRIMSON_HYPHAE: Self = BlockId::new(873u16).unwrap();
    pub const STRIPPED_CRIMSON_HYPHAE: Self = BlockId::new(874u16).unwrap();
    pub const CRIMSON_NYLIUM: Self = BlockId::new(875u16).unwrap();
    pub const CRIMSON_FUNGUS: Self = BlockId::new(876u16).unwrap();
    pub const SHROOMLIGHT: Self = BlockId::new(877u16).unwrap();
    pub const WEEPING_VINES: Self = BlockId::new(878u16).unwrap();
    pub const WEEPING_VINES_PLANT: Self = BlockId::new(879u16).unwrap();
    pub const TWISTING_VINES: Self = BlockId::new(880u16).unwrap();
    pub const TWISTING_VINES_PLANT: Self = BlockId::new(881u16).unwrap();
    pub const CRIMSON_ROOTS: Self = BlockId::new(882u16).unwrap();
    pub const CRIMSON_PLANKS: Self = BlockId::new(883u16).unwrap();
    pub const WARPED_PLANKS: Self = BlockId::new(884u16).unwrap();
    pub const CRIMSON_SLAB: Self = BlockId::new(885u16).unwrap();
    pub const WARPED_SLAB: Self = BlockId::new(886u16).unwrap();
    pub const CRIMSON_PRESSURE_PLATE: Self = BlockId::new(887u16).unwrap();
    pub const WARPED_PRESSURE_PLATE: Self = BlockId::new(888u16).unwrap();
    pub const CRIMSON_FENCE: Self = BlockId::new(889u16).unwrap();
    pub const WARPED_FENCE: Self = BlockId::new(890u16).unwrap();
    pub const CRIMSON_TRAPDOOR: Self = BlockId::new(891u16).unwrap();
    pub const WARPED_TRAPDOOR: Self = BlockId::new(892u16).unwrap();
    pub const CRIMSON_FENCE_GATE: Self = BlockId::new(893u16).unwrap();
    pub const WARPED_FENCE_GATE: Self = BlockId::new(894u16).unwrap();
    pub const CRIMSON_STAIRS: Self = BlockId::new(895u16).unwrap();
    pub const WARPED_STAIRS: Self = BlockId::new(896u16).unwrap();
    pub const CRIMSON_BUTTON: Self = BlockId::new(897u16).unwrap();
    pub const WARPED_BUTTON: Self = BlockId::new(898u16).unwrap();
    pub const CRIMSON_DOOR: Self = BlockId::new(899u16).unwrap();
    pub const WARPED_DOOR: Self = BlockId::new(900u16).unwrap();
    pub const CRIMSON_SIGN: Self = BlockId::new(901u16).unwrap();
    pub const WARPED_SIGN: Self = BlockId::new(902u16).unwrap();
    pub const CRIMSON_WALL_SIGN: Self = BlockId::new(903u16).unwrap();
    pub const WARPED_WALL_SIGN: Self = BlockId::new(904u16).unwrap();
    pub const STRUCTURE_BLOCK: Self = BlockId::new(905u16).unwrap();
    pub const JIGSAW: Self = BlockId::new(906u16).unwrap();
    pub const TEST_BLOCK: Self = BlockId::new(907u16).unwrap();
    pub const TEST_INSTANCE_BLOCK: Self = BlockId::new(908u16).unwrap();
    pub const COMPOSTER: Self = BlockId::new(909u16).unwrap();
    pub const TARGET: Self = BlockId::new(910u16).unwrap();
    pub const BEE_NEST: Self = BlockId::new(911u16).unwrap();
    pub const BEEHIVE: Self = BlockId::new(912u16).unwrap();
    pub const HONEY_BLOCK: Self = BlockId::new(913u16).unwrap();
    pub const HONEYCOMB_BLOCK: Self = BlockId::new(914u16).unwrap();
    pub const NETHERITE_BLOCK: Self = BlockId::new(915u16).unwrap();
    pub const ANCIENT_DEBRIS: Self = BlockId::new(916u16).unwrap();
    pub const CRYING_OBSIDIAN: Self = BlockId::new(917u16).unwrap();
    pub const RESPAWN_ANCHOR: Self = BlockId::new(918u16).unwrap();
    pub const POTTED_CRIMSON_FUNGUS: Self = BlockId::new(919u16).unwrap();
    pub const POTTED_WARPED_FUNGUS: Self = BlockId::new(920u16).unwrap();
    pub const POTTED_CRIMSON_ROOTS: Self = BlockId::new(921u16).unwrap();
    pub const POTTED_WARPED_ROOTS: Self = BlockId::new(922u16).unwrap();
    pub const LODESTONE: Self = BlockId::new(923u16).unwrap();
    pub const BLACKSTONE: Self = BlockId::new(924u16).unwrap();
    pub const BLACKSTONE_STAIRS: Self = BlockId::new(925u16).unwrap();
    pub const BLACKSTONE_WALL: Self = BlockId::new(926u16).unwrap();
    pub const BLACKSTONE_SLAB: Self = BlockId::new(927u16).unwrap();
    pub const POLISHED_BLACKSTONE: Self = BlockId::new(928u16).unwrap();
    pub const POLISHED_BLACKSTONE_BRICKS: Self = BlockId::new(929u16).unwrap();
    pub const CRACKED_POLISHED_BLACKSTONE_BRICKS: Self = BlockId::new(930u16).unwrap();
    pub const CHISELED_POLISHED_BLACKSTONE: Self = BlockId::new(931u16).unwrap();
    pub const POLISHED_BLACKSTONE_BRICK_SLAB: Self = BlockId::new(932u16).unwrap();
    pub const POLISHED_BLACKSTONE_BRICK_STAIRS: Self = BlockId::new(933u16).unwrap();
    pub const POLISHED_BLACKSTONE_BRICK_WALL: Self = BlockId::new(934u16).unwrap();
    pub const GILDED_BLACKSTONE: Self = BlockId::new(935u16).unwrap();
    pub const POLISHED_BLACKSTONE_STAIRS: Self = BlockId::new(936u16).unwrap();
    pub const POLISHED_BLACKSTONE_SLAB: Self = BlockId::new(937u16).unwrap();
    pub const POLISHED_BLACKSTONE_PRESSURE_PLATE: Self = BlockId::new(938u16).unwrap();
    pub const POLISHED_BLACKSTONE_BUTTON: Self = BlockId::new(939u16).unwrap();
    pub const POLISHED_BLACKSTONE_WALL: Self = BlockId::new(940u16).unwrap();
    pub const CHISELED_NETHER_BRICKS: Self = BlockId::new(941u16).unwrap();
    pub const CRACKED_NETHER_BRICKS: Self = BlockId::new(942u16).unwrap();
    pub const QUARTZ_BRICKS: Self = BlockId::new(943u16).unwrap();
    pub const CANDLE: Self = BlockId::new(944u16).unwrap();
    pub const WHITE_CANDLE: Self = BlockId::new(945u16).unwrap();
    pub const ORANGE_CANDLE: Self = BlockId::new(946u16).unwrap();
    pub const MAGENTA_CANDLE: Self = BlockId::new(947u16).unwrap();
    pub const LIGHT_BLUE_CANDLE: Self = BlockId::new(948u16).unwrap();
    pub const YELLOW_CANDLE: Self = BlockId::new(949u16).unwrap();
    pub const LIME_CANDLE: Self = BlockId::new(950u16).unwrap();
    pub const PINK_CANDLE: Self = BlockId::new(951u16).unwrap();
    pub const GRAY_CANDLE: Self = BlockId::new(952u16).unwrap();
    pub const LIGHT_GRAY_CANDLE: Self = BlockId::new(953u16).unwrap();
    pub const CYAN_CANDLE: Self = BlockId::new(954u16).unwrap();
    pub const PURPLE_CANDLE: Self = BlockId::new(955u16).unwrap();
    pub const BLUE_CANDLE: Self = BlockId::new(956u16).unwrap();
    pub const BROWN_CANDLE: Self = BlockId::new(957u16).unwrap();
    pub const GREEN_CANDLE: Self = BlockId::new(958u16).unwrap();
    pub const RED_CANDLE: Self = BlockId::new(959u16).unwrap();
    pub const BLACK_CANDLE: Self = BlockId::new(960u16).unwrap();
    pub const CANDLE_CAKE: Self = BlockId::new(961u16).unwrap();
    pub const WHITE_CANDLE_CAKE: Self = BlockId::new(962u16).unwrap();
    pub const ORANGE_CANDLE_CAKE: Self = BlockId::new(963u16).unwrap();
    pub const MAGENTA_CANDLE_CAKE: Self = BlockId::new(964u16).unwrap();
    pub const LIGHT_BLUE_CANDLE_CAKE: Self = BlockId::new(965u16).unwrap();
    pub const YELLOW_CANDLE_CAKE: Self = BlockId::new(966u16).unwrap();
    pub const LIME_CANDLE_CAKE: Self = BlockId::new(967u16).unwrap();
    pub const PINK_CANDLE_CAKE: Self = BlockId::new(968u16).unwrap();
    pub const GRAY_CANDLE_CAKE: Self = BlockId::new(969u16).unwrap();
    pub const LIGHT_GRAY_CANDLE_CAKE: Self = BlockId::new(970u16).unwrap();
    pub const CYAN_CANDLE_CAKE: Self = BlockId::new(971u16).unwrap();
    pub const PURPLE_CANDLE_CAKE: Self = BlockId::new(972u16).unwrap();
    pub const BLUE_CANDLE_CAKE: Self = BlockId::new(973u16).unwrap();
    pub const BROWN_CANDLE_CAKE: Self = BlockId::new(974u16).unwrap();
    pub const GREEN_CANDLE_CAKE: Self = BlockId::new(975u16).unwrap();
    pub const RED_CANDLE_CAKE: Self = BlockId::new(976u16).unwrap();
    pub const BLACK_CANDLE_CAKE: Self = BlockId::new(977u16).unwrap();
    pub const AMETHYST_BLOCK: Self = BlockId::new(978u16).unwrap();
    pub const BUDDING_AMETHYST: Self = BlockId::new(979u16).unwrap();
    pub const AMETHYST_CLUSTER: Self = BlockId::new(980u16).unwrap();
    pub const LARGE_AMETHYST_BUD: Self = BlockId::new(981u16).unwrap();
    pub const MEDIUM_AMETHYST_BUD: Self = BlockId::new(982u16).unwrap();
    pub const SMALL_AMETHYST_BUD: Self = BlockId::new(983u16).unwrap();
    pub const TUFF: Self = BlockId::new(984u16).unwrap();
    pub const TUFF_SLAB: Self = BlockId::new(985u16).unwrap();
    pub const TUFF_STAIRS: Self = BlockId::new(986u16).unwrap();
    pub const TUFF_WALL: Self = BlockId::new(987u16).unwrap();
    pub const POLISHED_TUFF: Self = BlockId::new(988u16).unwrap();
    pub const POLISHED_TUFF_SLAB: Self = BlockId::new(989u16).unwrap();
    pub const POLISHED_TUFF_STAIRS: Self = BlockId::new(990u16).unwrap();
    pub const POLISHED_TUFF_WALL: Self = BlockId::new(991u16).unwrap();
    pub const CHISELED_TUFF: Self = BlockId::new(992u16).unwrap();
    pub const TUFF_BRICKS: Self = BlockId::new(993u16).unwrap();
    pub const TUFF_BRICK_SLAB: Self = BlockId::new(994u16).unwrap();
    pub const TUFF_BRICK_STAIRS: Self = BlockId::new(995u16).unwrap();
    pub const TUFF_BRICK_WALL: Self = BlockId::new(996u16).unwrap();
    pub const CHISELED_TUFF_BRICKS: Self = BlockId::new(997u16).unwrap();
    pub const SULFUR: Self = BlockId::new(998u16).unwrap();
    pub const POTENT_SULFUR: Self = BlockId::new(999u16).unwrap();
    pub const SULFUR_SLAB: Self = BlockId::new(1000u16).unwrap();
    pub const SULFUR_STAIRS: Self = BlockId::new(1001u16).unwrap();
    pub const SULFUR_WALL: Self = BlockId::new(1002u16).unwrap();
    pub const POLISHED_SULFUR: Self = BlockId::new(1003u16).unwrap();
    pub const POLISHED_SULFUR_SLAB: Self = BlockId::new(1004u16).unwrap();
    pub const POLISHED_SULFUR_STAIRS: Self = BlockId::new(1005u16).unwrap();
    pub const POLISHED_SULFUR_WALL: Self = BlockId::new(1006u16).unwrap();
    pub const SULFUR_BRICKS: Self = BlockId::new(1007u16).unwrap();
    pub const SULFUR_BRICK_SLAB: Self = BlockId::new(1008u16).unwrap();
    pub const SULFUR_BRICK_STAIRS: Self = BlockId::new(1009u16).unwrap();
    pub const SULFUR_BRICK_WALL: Self = BlockId::new(1010u16).unwrap();
    pub const CHISELED_SULFUR: Self = BlockId::new(1011u16).unwrap();
    pub const CINNABAR: Self = BlockId::new(1012u16).unwrap();
    pub const CINNABAR_SLAB: Self = BlockId::new(1013u16).unwrap();
    pub const CINNABAR_STAIRS: Self = BlockId::new(1014u16).unwrap();
    pub const CINNABAR_WALL: Self = BlockId::new(1015u16).unwrap();
    pub const POLISHED_CINNABAR: Self = BlockId::new(1016u16).unwrap();
    pub const POLISHED_CINNABAR_SLAB: Self = BlockId::new(1017u16).unwrap();
    pub const POLISHED_CINNABAR_STAIRS: Self = BlockId::new(1018u16).unwrap();
    pub const POLISHED_CINNABAR_WALL: Self = BlockId::new(1019u16).unwrap();
    pub const CINNABAR_BRICKS: Self = BlockId::new(1020u16).unwrap();
    pub const CINNABAR_BRICK_SLAB: Self = BlockId::new(1021u16).unwrap();
    pub const CINNABAR_BRICK_STAIRS: Self = BlockId::new(1022u16).unwrap();
    pub const CINNABAR_BRICK_WALL: Self = BlockId::new(1023u16).unwrap();
    pub const CHISELED_CINNABAR: Self = BlockId::new(1024u16).unwrap();
    pub const CALCITE: Self = BlockId::new(1025u16).unwrap();
    pub const TINTED_GLASS: Self = BlockId::new(1026u16).unwrap();
    pub const POWDER_SNOW: Self = BlockId::new(1027u16).unwrap();
    pub const SCULK_SENSOR: Self = BlockId::new(1028u16).unwrap();
    pub const CALIBRATED_SCULK_SENSOR: Self = BlockId::new(1029u16).unwrap();
    pub const SCULK: Self = BlockId::new(1030u16).unwrap();
    pub const SCULK_VEIN: Self = BlockId::new(1031u16).unwrap();
    pub const SCULK_CATALYST: Self = BlockId::new(1032u16).unwrap();
    pub const SCULK_SHRIEKER: Self = BlockId::new(1033u16).unwrap();
    pub const COPPER_BLOCK: Self = BlockId::new(1034u16).unwrap();
    pub const EXPOSED_COPPER: Self = BlockId::new(1035u16).unwrap();
    pub const WEATHERED_COPPER: Self = BlockId::new(1036u16).unwrap();
    pub const OXIDIZED_COPPER: Self = BlockId::new(1037u16).unwrap();
    pub const WAXED_COPPER_BLOCK: Self = BlockId::new(1038u16).unwrap();
    pub const WAXED_EXPOSED_COPPER: Self = BlockId::new(1039u16).unwrap();
    pub const WAXED_WEATHERED_COPPER: Self = BlockId::new(1040u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER: Self = BlockId::new(1041u16).unwrap();
    pub const COPPER_ORE: Self = BlockId::new(1042u16).unwrap();
    pub const DEEPSLATE_COPPER_ORE: Self = BlockId::new(1043u16).unwrap();
    pub const CUT_COPPER: Self = BlockId::new(1044u16).unwrap();
    pub const EXPOSED_CUT_COPPER: Self = BlockId::new(1045u16).unwrap();
    pub const WEATHERED_CUT_COPPER: Self = BlockId::new(1046u16).unwrap();
    pub const OXIDIZED_CUT_COPPER: Self = BlockId::new(1047u16).unwrap();
    pub const WAXED_CUT_COPPER: Self = BlockId::new(1048u16).unwrap();
    pub const WAXED_EXPOSED_CUT_COPPER: Self = BlockId::new(1049u16).unwrap();
    pub const WAXED_WEATHERED_CUT_COPPER: Self = BlockId::new(1050u16).unwrap();
    pub const WAXED_OXIDIZED_CUT_COPPER: Self = BlockId::new(1051u16).unwrap();
    pub const CHISELED_COPPER: Self = BlockId::new(1052u16).unwrap();
    pub const EXPOSED_CHISELED_COPPER: Self = BlockId::new(1053u16).unwrap();
    pub const WEATHERED_CHISELED_COPPER: Self = BlockId::new(1054u16).unwrap();
    pub const OXIDIZED_CHISELED_COPPER: Self = BlockId::new(1055u16).unwrap();
    pub const WAXED_CHISELED_COPPER: Self = BlockId::new(1056u16).unwrap();
    pub const WAXED_EXPOSED_CHISELED_COPPER: Self = BlockId::new(1057u16).unwrap();
    pub const WAXED_WEATHERED_CHISELED_COPPER: Self = BlockId::new(1058u16).unwrap();
    pub const WAXED_OXIDIZED_CHISELED_COPPER: Self = BlockId::new(1059u16).unwrap();
    pub const CUT_COPPER_STAIRS: Self = BlockId::new(1060u16).unwrap();
    pub const EXPOSED_CUT_COPPER_STAIRS: Self = BlockId::new(1061u16).unwrap();
    pub const WEATHERED_CUT_COPPER_STAIRS: Self = BlockId::new(1062u16).unwrap();
    pub const OXIDIZED_CUT_COPPER_STAIRS: Self = BlockId::new(1063u16).unwrap();
    pub const WAXED_CUT_COPPER_STAIRS: Self = BlockId::new(1064u16).unwrap();
    pub const WAXED_EXPOSED_CUT_COPPER_STAIRS: Self = BlockId::new(1065u16).unwrap();
    pub const WAXED_WEATHERED_CUT_COPPER_STAIRS: Self = BlockId::new(1066u16).unwrap();
    pub const WAXED_OXIDIZED_CUT_COPPER_STAIRS: Self = BlockId::new(1067u16).unwrap();
    pub const CUT_COPPER_SLAB: Self = BlockId::new(1068u16).unwrap();
    pub const EXPOSED_CUT_COPPER_SLAB: Self = BlockId::new(1069u16).unwrap();
    pub const WEATHERED_CUT_COPPER_SLAB: Self = BlockId::new(1070u16).unwrap();
    pub const OXIDIZED_CUT_COPPER_SLAB: Self = BlockId::new(1071u16).unwrap();
    pub const WAXED_CUT_COPPER_SLAB: Self = BlockId::new(1072u16).unwrap();
    pub const WAXED_EXPOSED_CUT_COPPER_SLAB: Self = BlockId::new(1073u16).unwrap();
    pub const WAXED_WEATHERED_CUT_COPPER_SLAB: Self = BlockId::new(1074u16).unwrap();
    pub const WAXED_OXIDIZED_CUT_COPPER_SLAB: Self = BlockId::new(1075u16).unwrap();
    pub const COPPER_DOOR: Self = BlockId::new(1076u16).unwrap();
    pub const EXPOSED_COPPER_DOOR: Self = BlockId::new(1077u16).unwrap();
    pub const WEATHERED_COPPER_DOOR: Self = BlockId::new(1078u16).unwrap();
    pub const OXIDIZED_COPPER_DOOR: Self = BlockId::new(1079u16).unwrap();
    pub const WAXED_COPPER_DOOR: Self = BlockId::new(1080u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_DOOR: Self = BlockId::new(1081u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_DOOR: Self = BlockId::new(1082u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_DOOR: Self = BlockId::new(1083u16).unwrap();
    pub const COPPER_TRAPDOOR: Self = BlockId::new(1084u16).unwrap();
    pub const EXPOSED_COPPER_TRAPDOOR: Self = BlockId::new(1085u16).unwrap();
    pub const WEATHERED_COPPER_TRAPDOOR: Self = BlockId::new(1086u16).unwrap();
    pub const OXIDIZED_COPPER_TRAPDOOR: Self = BlockId::new(1087u16).unwrap();
    pub const WAXED_COPPER_TRAPDOOR: Self = BlockId::new(1088u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_TRAPDOOR: Self = BlockId::new(1089u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_TRAPDOOR: Self = BlockId::new(1090u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_TRAPDOOR: Self = BlockId::new(1091u16).unwrap();
    pub const COPPER_GRATE: Self = BlockId::new(1092u16).unwrap();
    pub const EXPOSED_COPPER_GRATE: Self = BlockId::new(1093u16).unwrap();
    pub const WEATHERED_COPPER_GRATE: Self = BlockId::new(1094u16).unwrap();
    pub const OXIDIZED_COPPER_GRATE: Self = BlockId::new(1095u16).unwrap();
    pub const WAXED_COPPER_GRATE: Self = BlockId::new(1096u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_GRATE: Self = BlockId::new(1097u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_GRATE: Self = BlockId::new(1098u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_GRATE: Self = BlockId::new(1099u16).unwrap();
    pub const COPPER_BULB: Self = BlockId::new(1100u16).unwrap();
    pub const EXPOSED_COPPER_BULB: Self = BlockId::new(1101u16).unwrap();
    pub const WEATHERED_COPPER_BULB: Self = BlockId::new(1102u16).unwrap();
    pub const OXIDIZED_COPPER_BULB: Self = BlockId::new(1103u16).unwrap();
    pub const WAXED_COPPER_BULB: Self = BlockId::new(1104u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_BULB: Self = BlockId::new(1105u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_BULB: Self = BlockId::new(1106u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_BULB: Self = BlockId::new(1107u16).unwrap();
    pub const COPPER_CHEST: Self = BlockId::new(1108u16).unwrap();
    pub const EXPOSED_COPPER_CHEST: Self = BlockId::new(1109u16).unwrap();
    pub const WEATHERED_COPPER_CHEST: Self = BlockId::new(1110u16).unwrap();
    pub const OXIDIZED_COPPER_CHEST: Self = BlockId::new(1111u16).unwrap();
    pub const WAXED_COPPER_CHEST: Self = BlockId::new(1112u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_CHEST: Self = BlockId::new(1113u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_CHEST: Self = BlockId::new(1114u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_CHEST: Self = BlockId::new(1115u16).unwrap();
    pub const COPPER_GOLEM_STATUE: Self = BlockId::new(1116u16).unwrap();
    pub const EXPOSED_COPPER_GOLEM_STATUE: Self = BlockId::new(1117u16).unwrap();
    pub const WEATHERED_COPPER_GOLEM_STATUE: Self = BlockId::new(1118u16).unwrap();
    pub const OXIDIZED_COPPER_GOLEM_STATUE: Self = BlockId::new(1119u16).unwrap();
    pub const WAXED_COPPER_GOLEM_STATUE: Self = BlockId::new(1120u16).unwrap();
    pub const WAXED_EXPOSED_COPPER_GOLEM_STATUE: Self = BlockId::new(1121u16).unwrap();
    pub const WAXED_WEATHERED_COPPER_GOLEM_STATUE: Self = BlockId::new(1122u16).unwrap();
    pub const WAXED_OXIDIZED_COPPER_GOLEM_STATUE: Self = BlockId::new(1123u16).unwrap();
    pub const LIGHTNING_ROD: Self = BlockId::new(1124u16).unwrap();
    pub const EXPOSED_LIGHTNING_ROD: Self = BlockId::new(1125u16).unwrap();
    pub const WEATHERED_LIGHTNING_ROD: Self = BlockId::new(1126u16).unwrap();
    pub const OXIDIZED_LIGHTNING_ROD: Self = BlockId::new(1127u16).unwrap();
    pub const WAXED_LIGHTNING_ROD: Self = BlockId::new(1128u16).unwrap();
    pub const WAXED_EXPOSED_LIGHTNING_ROD: Self = BlockId::new(1129u16).unwrap();
    pub const WAXED_WEATHERED_LIGHTNING_ROD: Self = BlockId::new(1130u16).unwrap();
    pub const WAXED_OXIDIZED_LIGHTNING_ROD: Self = BlockId::new(1131u16).unwrap();
    pub const DRIPSTONE_BLOCK: Self = BlockId::new(1132u16).unwrap();
    pub const POINTED_DRIPSTONE: Self = BlockId::new(1133u16).unwrap();
    pub const SULFUR_SPIKE: Self = BlockId::new(1134u16).unwrap();
    pub const CAVE_VINES: Self = BlockId::new(1135u16).unwrap();
    pub const CAVE_VINES_PLANT: Self = BlockId::new(1136u16).unwrap();
    pub const SPORE_BLOSSOM: Self = BlockId::new(1137u16).unwrap();
    pub const AZALEA: Self = BlockId::new(1138u16).unwrap();
    pub const FLOWERING_AZALEA: Self = BlockId::new(1139u16).unwrap();
    pub const MOSS_CARPET: Self = BlockId::new(1140u16).unwrap();
    pub const PINK_PETALS: Self = BlockId::new(1141u16).unwrap();
    pub const WILDFLOWERS: Self = BlockId::new(1142u16).unwrap();
    pub const LEAF_LITTER: Self = BlockId::new(1143u16).unwrap();
    pub const MOSS_BLOCK: Self = BlockId::new(1144u16).unwrap();
    pub const BIG_DRIPLEAF: Self = BlockId::new(1145u16).unwrap();
    pub const BIG_DRIPLEAF_STEM: Self = BlockId::new(1146u16).unwrap();
    pub const SMALL_DRIPLEAF: Self = BlockId::new(1147u16).unwrap();
    pub const HANGING_ROOTS: Self = BlockId::new(1148u16).unwrap();
    pub const ROOTED_DIRT: Self = BlockId::new(1149u16).unwrap();
    pub const MUD: Self = BlockId::new(1150u16).unwrap();
    pub const DEEPSLATE: Self = BlockId::new(1151u16).unwrap();
    pub const COBBLED_DEEPSLATE: Self = BlockId::new(1152u16).unwrap();
    pub const COBBLED_DEEPSLATE_STAIRS: Self = BlockId::new(1153u16).unwrap();
    pub const COBBLED_DEEPSLATE_SLAB: Self = BlockId::new(1154u16).unwrap();
    pub const COBBLED_DEEPSLATE_WALL: Self = BlockId::new(1155u16).unwrap();
    pub const POLISHED_DEEPSLATE: Self = BlockId::new(1156u16).unwrap();
    pub const POLISHED_DEEPSLATE_STAIRS: Self = BlockId::new(1157u16).unwrap();
    pub const POLISHED_DEEPSLATE_SLAB: Self = BlockId::new(1158u16).unwrap();
    pub const POLISHED_DEEPSLATE_WALL: Self = BlockId::new(1159u16).unwrap();
    pub const DEEPSLATE_TILES: Self = BlockId::new(1160u16).unwrap();
    pub const DEEPSLATE_TILE_STAIRS: Self = BlockId::new(1161u16).unwrap();
    pub const DEEPSLATE_TILE_SLAB: Self = BlockId::new(1162u16).unwrap();
    pub const DEEPSLATE_TILE_WALL: Self = BlockId::new(1163u16).unwrap();
    pub const DEEPSLATE_BRICKS: Self = BlockId::new(1164u16).unwrap();
    pub const DEEPSLATE_BRICK_STAIRS: Self = BlockId::new(1165u16).unwrap();
    pub const DEEPSLATE_BRICK_SLAB: Self = BlockId::new(1166u16).unwrap();
    pub const DEEPSLATE_BRICK_WALL: Self = BlockId::new(1167u16).unwrap();
    pub const CHISELED_DEEPSLATE: Self = BlockId::new(1168u16).unwrap();
    pub const CRACKED_DEEPSLATE_BRICKS: Self = BlockId::new(1169u16).unwrap();
    pub const CRACKED_DEEPSLATE_TILES: Self = BlockId::new(1170u16).unwrap();
    pub const INFESTED_DEEPSLATE: Self = BlockId::new(1171u16).unwrap();
    pub const SMOOTH_BASALT: Self = BlockId::new(1172u16).unwrap();
    pub const RAW_IRON_BLOCK: Self = BlockId::new(1173u16).unwrap();
    pub const RAW_COPPER_BLOCK: Self = BlockId::new(1174u16).unwrap();
    pub const RAW_GOLD_BLOCK: Self = BlockId::new(1175u16).unwrap();
    pub const POTTED_AZALEA_BUSH: Self = BlockId::new(1176u16).unwrap();
    pub const POTTED_FLOWERING_AZALEA_BUSH: Self = BlockId::new(1177u16).unwrap();
    pub const OCHRE_FROGLIGHT: Self = BlockId::new(1178u16).unwrap();
    pub const VERDANT_FROGLIGHT: Self = BlockId::new(1179u16).unwrap();
    pub const PEARLESCENT_FROGLIGHT: Self = BlockId::new(1180u16).unwrap();
    pub const FROGSPAWN: Self = BlockId::new(1181u16).unwrap();
    pub const REINFORCED_DEEPSLATE: Self = BlockId::new(1182u16).unwrap();
    pub const DECORATED_POT: Self = BlockId::new(1183u16).unwrap();
    pub const CRAFTER: Self = BlockId::new(1184u16).unwrap();
    pub const TRIAL_SPAWNER: Self = BlockId::new(1185u16).unwrap();
    pub const VAULT: Self = BlockId::new(1186u16).unwrap();
    pub const HEAVY_CORE: Self = BlockId::new(1187u16).unwrap();
    pub const PALE_MOSS_BLOCK: Self = BlockId::new(1188u16).unwrap();
    pub const PALE_MOSS_CARPET: Self = BlockId::new(1189u16).unwrap();
    pub const PALE_HANGING_MOSS: Self = BlockId::new(1190u16).unwrap();
    pub const OPEN_EYEBLOSSOM: Self = BlockId::new(1191u16).unwrap();
    pub const CLOSED_EYEBLOSSOM: Self = BlockId::new(1192u16).unwrap();
    pub const POTTED_OPEN_EYEBLOSSOM: Self = BlockId::new(1193u16).unwrap();
    pub const POTTED_CLOSED_EYEBLOSSOM: Self = BlockId::new(1194u16).unwrap();
    pub const FIREFLY_BUSH: Self = BlockId::new(1195u16).unwrap();
    pub(crate) const BLOCK_COUNT: u16 = mappings::TYPE_FROM_RAW_ID.len() as u16;
    #[doc = r" Get a [`BlockId`] from a [`BlockStateId`]"]
    #[inline]
    #[must_use]
    pub const fn from_state_id(id: BlockStateId) -> BlockId {
        unsafe { std::hint::assert_unchecked(id.as_u16() < BlockStateId::STATE_COUNT) }
        mappings::BLOCK_ID_FROM_STATE_ID[id.as_u16() as usize]
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachFace {
    Floor,
    Wall,
    Ceiling,
}
impl EnumVariants for AttachFace {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Floor => 0u16,
            Self::Wall => 1u16,
            Self::Ceiling => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Floor,
            1u16 => Self::Wall,
            2u16 => Self::Ceiling,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Wall => "wall",
            Self::Ceiling => "ceiling",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "floor" => Self::Floor,
            "wall" => Self::Wall,
            "ceiling" => Self::Ceiling,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}
impl EnumVariants for Axis {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::X => 0u16,
            Self::Y => 1u16,
            Self::Z => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::X,
            1u16 => Self::Y,
            2u16 => Self::Z,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "x" => Self::X,
            "y" => Self::Y,
            "z" => Self::Z,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BambooLeaves {
    None,
    Small,
    Large,
}
impl EnumVariants for BambooLeaves {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::None => 0u16,
            Self::Small => 1u16,
            Self::Large => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::None,
            1u16 => Self::Small,
            2u16 => Self::Large,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Small => "small",
            Self::Large => "large",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "small" => Self::Small,
            "large" => Self::Large,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BedPart {
    Head,
    Foot,
}
impl EnumVariants for BedPart {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Head => 0u16,
            Self::Foot => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Head,
            1u16 => Self::Foot,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Head => "head",
            Self::Foot => "foot",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "head" => Self::Head,
            "foot" => Self::Foot,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BellAttachment {
    Floor,
    Ceiling,
    SingleWall,
    DoubleWall,
}
impl EnumVariants for BellAttachment {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Floor => 0u16,
            Self::Ceiling => 1u16,
            Self::SingleWall => 2u16,
            Self::DoubleWall => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Floor,
            1u16 => Self::Ceiling,
            2u16 => Self::SingleWall,
            3u16 => Self::DoubleWall,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Ceiling => "ceiling",
            Self::SingleWall => "single_wall",
            Self::DoubleWall => "double_wall",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "floor" => Self::Floor,
            "ceiling" => Self::Ceiling,
            "single_wall" => Self::SingleWall,
            "double_wall" => Self::DoubleWall,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChestType {
    Single,
    Left,
    Right,
}
impl EnumVariants for ChestType {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Single => 0u16,
            Self::Left => 1u16,
            Self::Right => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Single,
            1u16 => Self::Left,
            2u16 => Self::Right,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "single" => Self::Single,
            "left" => Self::Left,
            "right" => Self::Right,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopperGolemPose {
    Standing,
    Sitting,
    Running,
    Star,
}
impl EnumVariants for CopperGolemPose {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Standing => 0u16,
            Self::Sitting => 1u16,
            Self::Running => 2u16,
            Self::Star => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Standing,
            1u16 => Self::Sitting,
            2u16 => Self::Running,
            3u16 => Self::Star,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Standing => "standing",
            Self::Sitting => "sitting",
            Self::Running => "running",
            Self::Star => "star",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "standing" => Self::Standing,
            "sitting" => Self::Sitting,
            "running" => Self::Running,
            "star" => Self::Star,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreakingHeartState {
    Uprooted,
    Dormant,
    Awake,
}
impl EnumVariants for CreakingHeartState {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Uprooted => 0u16,
            Self::Dormant => 1u16,
            Self::Awake => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Uprooted,
            1u16 => Self::Dormant,
            2u16 => Self::Awake,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Uprooted => "uprooted",
            Self::Dormant => "dormant",
            Self::Awake => "awake",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "uprooted" => Self::Uprooted,
            "dormant" => Self::Dormant,
            "awake" => Self::Awake,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoorHinge {
    Left,
    Right,
}
impl EnumVariants for DoorHinge {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Left => 0u16,
            Self::Right => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Left,
            1u16 => Self::Right,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "left" => Self::Left,
            "right" => Self::Right,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoubleBlockHalf {
    Upper,
    Lower,
}
impl EnumVariants for DoubleBlockHalf {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Upper => 0u16,
            Self::Lower => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Upper,
            1u16 => Self::Lower,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Upper => "upper",
            Self::Lower => "lower",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "upper" => Self::Upper,
            "lower" => Self::Lower,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EastRedstone {
    Up,
    Side,
    None,
}
impl EnumVariants for EastRedstone {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Up => 0u16,
            Self::Side => 1u16,
            Self::None => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Up,
            1u16 => Self::Side,
            2u16 => Self::None,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Side => "side",
            Self::None => "none",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "up" => Self::Up,
            "side" => Self::Side,
            "none" => Self::None,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EastWall {
    None,
    Low,
    Tall,
}
impl EnumVariants for EastWall {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::None => 0u16,
            Self::Low => 1u16,
            Self::Tall => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::None,
            1u16 => Self::Low,
            2u16 => Self::Tall,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Tall => "tall",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "low" => Self::Low,
            "tall" => Self::Tall,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Facing {
    North,
    East,
    South,
    West,
    Up,
    Down,
}
impl EnumVariants for Facing {
    fn variant_count() -> u16 {
        6u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::North => 0u16,
            Self::East => 1u16,
            Self::South => 2u16,
            Self::West => 3u16,
            Self::Up => 4u16,
            Self::Down => 5u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::North,
            1u16 => Self::East,
            2u16 => Self::South,
            3u16 => Self::West,
            4u16 => Self::Up,
            5u16 => Self::Down,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "north" => Self::North,
            "east" => Self::East,
            "south" => Self::South,
            "west" => Self::West,
            "up" => Self::Up,
            "down" => Self::Down,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FacingHopper {
    Down,
    North,
    South,
    West,
    East,
}
impl EnumVariants for FacingHopper {
    fn variant_count() -> u16 {
        5u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Down => 0u16,
            Self::North => 1u16,
            Self::South => 2u16,
            Self::West => 3u16,
            Self::East => 4u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Down,
            1u16 => Self::North,
            2u16 => Self::South,
            3u16 => Self::West,
            4u16 => Self::East,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "down" => Self::Down,
            "north" => Self::North,
            "south" => Self::South,
            "west" => Self::West,
            "east" => Self::East,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Half {
    Top,
    Bottom,
}
impl EnumVariants for Half {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Top => 0u16,
            Self::Bottom => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Top,
            1u16 => Self::Bottom,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalAxis {
    X,
    Z,
}
impl EnumVariants for HorizontalAxis {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::X => 0u16,
            Self::Z => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::X,
            1u16 => Self::Z,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Z => "z",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "x" => Self::X,
            "z" => Self::Z,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalFacing {
    North,
    South,
    West,
    East,
}
impl EnumVariants for HorizontalFacing {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::North => 0u16,
            Self::South => 1u16,
            Self::West => 2u16,
            Self::East => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::North,
            1u16 => Self::South,
            2u16 => Self::West,
            3u16 => Self::East,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "north" => Self::North,
            "south" => Self::South,
            "west" => Self::West,
            "east" => Self::East,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeComparator {
    Compare,
    Subtract,
}
impl EnumVariants for ModeComparator {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Compare => 0u16,
            Self::Subtract => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Compare,
            1u16 => Self::Subtract,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Compare => "compare",
            Self::Subtract => "subtract",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "compare" => Self::Compare,
            "subtract" => Self::Subtract,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NorthRedstone {
    Up,
    Side,
    None,
}
impl EnumVariants for NorthRedstone {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Up => 0u16,
            Self::Side => 1u16,
            Self::None => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Up,
            1u16 => Self::Side,
            2u16 => Self::None,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Side => "side",
            Self::None => "none",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "up" => Self::Up,
            "side" => Self::Side,
            "none" => Self::None,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NorthWall {
    None,
    Low,
    Tall,
}
impl EnumVariants for NorthWall {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::None => 0u16,
            Self::Low => 1u16,
            Self::Tall => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::None,
            1u16 => Self::Low,
            2u16 => Self::Tall,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Tall => "tall",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "low" => Self::Low,
            "tall" => Self::Tall,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoteblockInstrument {
    Harp,
    Basedrum,
    Snare,
    Hat,
    Bass,
    Flute,
    Bell,
    Guitar,
    Chime,
    Xylophone,
    IronXylophone,
    CowBell,
    Didgeridoo,
    Bit,
    Banjo,
    Pling,
    Trumpet,
    TrumpetExposed,
    TrumpetOxidized,
    TrumpetWeathered,
    Zombie,
    Skeleton,
    Creeper,
    Dragon,
    WitherSkeleton,
    Piglin,
    CustomHead,
}
impl EnumVariants for NoteblockInstrument {
    fn variant_count() -> u16 {
        27u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Harp => 0u16,
            Self::Basedrum => 1u16,
            Self::Snare => 2u16,
            Self::Hat => 3u16,
            Self::Bass => 4u16,
            Self::Flute => 5u16,
            Self::Bell => 6u16,
            Self::Guitar => 7u16,
            Self::Chime => 8u16,
            Self::Xylophone => 9u16,
            Self::IronXylophone => 10u16,
            Self::CowBell => 11u16,
            Self::Didgeridoo => 12u16,
            Self::Bit => 13u16,
            Self::Banjo => 14u16,
            Self::Pling => 15u16,
            Self::Trumpet => 16u16,
            Self::TrumpetExposed => 17u16,
            Self::TrumpetOxidized => 18u16,
            Self::TrumpetWeathered => 19u16,
            Self::Zombie => 20u16,
            Self::Skeleton => 21u16,
            Self::Creeper => 22u16,
            Self::Dragon => 23u16,
            Self::WitherSkeleton => 24u16,
            Self::Piglin => 25u16,
            Self::CustomHead => 26u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Harp,
            1u16 => Self::Basedrum,
            2u16 => Self::Snare,
            3u16 => Self::Hat,
            4u16 => Self::Bass,
            5u16 => Self::Flute,
            6u16 => Self::Bell,
            7u16 => Self::Guitar,
            8u16 => Self::Chime,
            9u16 => Self::Xylophone,
            10u16 => Self::IronXylophone,
            11u16 => Self::CowBell,
            12u16 => Self::Didgeridoo,
            13u16 => Self::Bit,
            14u16 => Self::Banjo,
            15u16 => Self::Pling,
            16u16 => Self::Trumpet,
            17u16 => Self::TrumpetExposed,
            18u16 => Self::TrumpetOxidized,
            19u16 => Self::TrumpetWeathered,
            20u16 => Self::Zombie,
            21u16 => Self::Skeleton,
            22u16 => Self::Creeper,
            23u16 => Self::Dragon,
            24u16 => Self::WitherSkeleton,
            25u16 => Self::Piglin,
            26u16 => Self::CustomHead,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Harp => "harp",
            Self::Basedrum => "basedrum",
            Self::Snare => "snare",
            Self::Hat => "hat",
            Self::Bass => "bass",
            Self::Flute => "flute",
            Self::Bell => "bell",
            Self::Guitar => "guitar",
            Self::Chime => "chime",
            Self::Xylophone => "xylophone",
            Self::IronXylophone => "iron_xylophone",
            Self::CowBell => "cow_bell",
            Self::Didgeridoo => "didgeridoo",
            Self::Bit => "bit",
            Self::Banjo => "banjo",
            Self::Pling => "pling",
            Self::Trumpet => "trumpet",
            Self::TrumpetExposed => "trumpet_exposed",
            Self::TrumpetOxidized => "trumpet_oxidized",
            Self::TrumpetWeathered => "trumpet_weathered",
            Self::Zombie => "zombie",
            Self::Skeleton => "skeleton",
            Self::Creeper => "creeper",
            Self::Dragon => "dragon",
            Self::WitherSkeleton => "wither_skeleton",
            Self::Piglin => "piglin",
            Self::CustomHead => "custom_head",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "harp" => Self::Harp,
            "basedrum" => Self::Basedrum,
            "snare" => Self::Snare,
            "hat" => Self::Hat,
            "bass" => Self::Bass,
            "flute" => Self::Flute,
            "bell" => Self::Bell,
            "guitar" => Self::Guitar,
            "chime" => Self::Chime,
            "xylophone" => Self::Xylophone,
            "iron_xylophone" => Self::IronXylophone,
            "cow_bell" => Self::CowBell,
            "didgeridoo" => Self::Didgeridoo,
            "bit" => Self::Bit,
            "banjo" => Self::Banjo,
            "pling" => Self::Pling,
            "trumpet" => Self::Trumpet,
            "trumpet_exposed" => Self::TrumpetExposed,
            "trumpet_oxidized" => Self::TrumpetOxidized,
            "trumpet_weathered" => Self::TrumpetWeathered,
            "zombie" => Self::Zombie,
            "skeleton" => Self::Skeleton,
            "creeper" => Self::Creeper,
            "dragon" => Self::Dragon,
            "wither_skeleton" => Self::WitherSkeleton,
            "piglin" => Self::Piglin,
            "custom_head" => Self::CustomHead,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    DownEast,
    DownNorth,
    DownSouth,
    DownWest,
    UpEast,
    UpNorth,
    UpSouth,
    UpWest,
    WestUp,
    EastUp,
    NorthUp,
    SouthUp,
}
impl EnumVariants for Orientation {
    fn variant_count() -> u16 {
        12u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::DownEast => 0u16,
            Self::DownNorth => 1u16,
            Self::DownSouth => 2u16,
            Self::DownWest => 3u16,
            Self::UpEast => 4u16,
            Self::UpNorth => 5u16,
            Self::UpSouth => 6u16,
            Self::UpWest => 7u16,
            Self::WestUp => 8u16,
            Self::EastUp => 9u16,
            Self::NorthUp => 10u16,
            Self::SouthUp => 11u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::DownEast,
            1u16 => Self::DownNorth,
            2u16 => Self::DownSouth,
            3u16 => Self::DownWest,
            4u16 => Self::UpEast,
            5u16 => Self::UpNorth,
            6u16 => Self::UpSouth,
            7u16 => Self::UpWest,
            8u16 => Self::WestUp,
            9u16 => Self::EastUp,
            10u16 => Self::NorthUp,
            11u16 => Self::SouthUp,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::DownEast => "down_east",
            Self::DownNorth => "down_north",
            Self::DownSouth => "down_south",
            Self::DownWest => "down_west",
            Self::UpEast => "up_east",
            Self::UpNorth => "up_north",
            Self::UpSouth => "up_south",
            Self::UpWest => "up_west",
            Self::WestUp => "west_up",
            Self::EastUp => "east_up",
            Self::NorthUp => "north_up",
            Self::SouthUp => "south_up",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "down_east" => Self::DownEast,
            "down_north" => Self::DownNorth,
            "down_south" => Self::DownSouth,
            "down_west" => Self::DownWest,
            "up_east" => Self::UpEast,
            "up_north" => Self::UpNorth,
            "up_south" => Self::UpSouth,
            "up_west" => Self::UpWest,
            "west_up" => Self::WestUp,
            "east_up" => Self::EastUp,
            "north_up" => Self::NorthUp,
            "south_up" => Self::SouthUp,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PistonType {
    Normal,
    Sticky,
}
impl EnumVariants for PistonType {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Normal => 0u16,
            Self::Sticky => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Normal,
            1u16 => Self::Sticky,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Sticky => "sticky",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "normal" => Self::Normal,
            "sticky" => Self::Sticky,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PotentSulfurState {
    Dry,
    Wet,
    Dormant,
    Erupting,
    Continuous,
}
impl EnumVariants for PotentSulfurState {
    fn variant_count() -> u16 {
        5u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Dry => 0u16,
            Self::Wet => 1u16,
            Self::Dormant => 2u16,
            Self::Erupting => 3u16,
            Self::Continuous => 4u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Dry,
            1u16 => Self::Wet,
            2u16 => Self::Dormant,
            3u16 => Self::Erupting,
            4u16 => Self::Continuous,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Dry => "dry",
            Self::Wet => "wet",
            Self::Dormant => "dormant",
            Self::Erupting => "erupting",
            Self::Continuous => "continuous",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "dry" => Self::Dry,
            "wet" => Self::Wet,
            "dormant" => Self::Dormant,
            "erupting" => Self::Erupting,
            "continuous" => Self::Continuous,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}
impl EnumVariants for RailShape {
    fn variant_count() -> u16 {
        10u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::NorthSouth => 0u16,
            Self::EastWest => 1u16,
            Self::AscendingEast => 2u16,
            Self::AscendingWest => 3u16,
            Self::AscendingNorth => 4u16,
            Self::AscendingSouth => 5u16,
            Self::SouthEast => 6u16,
            Self::SouthWest => 7u16,
            Self::NorthWest => 8u16,
            Self::NorthEast => 9u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::NorthSouth,
            1u16 => Self::EastWest,
            2u16 => Self::AscendingEast,
            3u16 => Self::AscendingWest,
            4u16 => Self::AscendingNorth,
            5u16 => Self::AscendingSouth,
            6u16 => Self::SouthEast,
            7u16 => Self::SouthWest,
            8u16 => Self::NorthWest,
            9u16 => Self::NorthEast,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::NorthSouth => "north_south",
            Self::EastWest => "east_west",
            Self::AscendingEast => "ascending_east",
            Self::AscendingWest => "ascending_west",
            Self::AscendingNorth => "ascending_north",
            Self::AscendingSouth => "ascending_south",
            Self::SouthEast => "south_east",
            Self::SouthWest => "south_west",
            Self::NorthWest => "north_west",
            Self::NorthEast => "north_east",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "north_south" => Self::NorthSouth,
            "east_west" => Self::EastWest,
            "ascending_east" => Self::AscendingEast,
            "ascending_west" => Self::AscendingWest,
            "ascending_north" => Self::AscendingNorth,
            "ascending_south" => Self::AscendingSouth,
            "south_east" => Self::SouthEast,
            "south_west" => Self::SouthWest,
            "north_west" => Self::NorthWest,
            "north_east" => Self::NorthEast,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RailShapeStraight {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
}
impl EnumVariants for RailShapeStraight {
    fn variant_count() -> u16 {
        6u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::NorthSouth => 0u16,
            Self::EastWest => 1u16,
            Self::AscendingEast => 2u16,
            Self::AscendingWest => 3u16,
            Self::AscendingNorth => 4u16,
            Self::AscendingSouth => 5u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::NorthSouth,
            1u16 => Self::EastWest,
            2u16 => Self::AscendingEast,
            3u16 => Self::AscendingWest,
            4u16 => Self::AscendingNorth,
            5u16 => Self::AscendingSouth,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::NorthSouth => "north_south",
            Self::EastWest => "east_west",
            Self::AscendingEast => "ascending_east",
            Self::AscendingWest => "ascending_west",
            Self::AscendingNorth => "ascending_north",
            Self::AscendingSouth => "ascending_south",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "north_south" => Self::NorthSouth,
            "east_west" => Self::EastWest,
            "ascending_east" => Self::AscendingEast,
            "ascending_west" => Self::AscendingWest,
            "ascending_north" => Self::AscendingNorth,
            "ascending_south" => Self::AscendingSouth,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SculkSensorPhase {
    Inactive,
    Active,
    Cooldown,
}
impl EnumVariants for SculkSensorPhase {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Inactive => 0u16,
            Self::Active => 1u16,
            Self::Cooldown => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Inactive,
            1u16 => Self::Active,
            2u16 => Self::Cooldown,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Cooldown => "cooldown",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "inactive" => Self::Inactive,
            "active" => Self::Active,
            "cooldown" => Self::Cooldown,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SideChainPart {
    Unconnected,
    Right,
    Center,
    Left,
}
impl EnumVariants for SideChainPart {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Unconnected => 0u16,
            Self::Right => 1u16,
            Self::Center => 2u16,
            Self::Left => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Unconnected,
            1u16 => Self::Right,
            2u16 => Self::Center,
            3u16 => Self::Left,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Unconnected => "unconnected",
            Self::Right => "right",
            Self::Center => "center",
            Self::Left => "left",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "unconnected" => Self::Unconnected,
            "right" => Self::Right,
            "center" => Self::Center,
            "left" => Self::Left,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlabType {
    Top,
    Bottom,
    Double,
}
impl EnumVariants for SlabType {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Top => 0u16,
            Self::Bottom => 1u16,
            Self::Double => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Top,
            1u16 => Self::Bottom,
            2u16 => Self::Double,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Double => "double",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            "double" => Self::Double,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SouthRedstone {
    Up,
    Side,
    None,
}
impl EnumVariants for SouthRedstone {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Up => 0u16,
            Self::Side => 1u16,
            Self::None => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Up,
            1u16 => Self::Side,
            2u16 => Self::None,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Side => "side",
            Self::None => "none",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "up" => Self::Up,
            "side" => Self::Side,
            "none" => Self::None,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SouthWall {
    None,
    Low,
    Tall,
}
impl EnumVariants for SouthWall {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::None => 0u16,
            Self::Low => 1u16,
            Self::Tall => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::None,
            1u16 => Self::Low,
            2u16 => Self::Tall,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Tall => "tall",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "low" => Self::Low,
            "tall" => Self::Tall,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeleothemThickness {
    TipMerge,
    Tip,
    Frustum,
    Middle,
    Base,
}
impl EnumVariants for SpeleothemThickness {
    fn variant_count() -> u16 {
        5u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::TipMerge => 0u16,
            Self::Tip => 1u16,
            Self::Frustum => 2u16,
            Self::Middle => 3u16,
            Self::Base => 4u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::TipMerge,
            1u16 => Self::Tip,
            2u16 => Self::Frustum,
            3u16 => Self::Middle,
            4u16 => Self::Base,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::TipMerge => "tip_merge",
            Self::Tip => "tip",
            Self::Frustum => "frustum",
            Self::Middle => "middle",
            Self::Base => "base",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "tip_merge" => Self::TipMerge,
            "tip" => Self::Tip,
            "frustum" => Self::Frustum,
            "middle" => Self::Middle,
            "base" => Self::Base,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StairsShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}
impl EnumVariants for StairsShape {
    fn variant_count() -> u16 {
        5u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Straight => 0u16,
            Self::InnerLeft => 1u16,
            Self::InnerRight => 2u16,
            Self::OuterLeft => 3u16,
            Self::OuterRight => 4u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Straight,
            1u16 => Self::InnerLeft,
            2u16 => Self::InnerRight,
            3u16 => Self::OuterLeft,
            4u16 => Self::OuterRight,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Straight => "straight",
            Self::InnerLeft => "inner_left",
            Self::InnerRight => "inner_right",
            Self::OuterLeft => "outer_left",
            Self::OuterRight => "outer_right",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "straight" => Self::Straight,
            "inner_left" => Self::InnerLeft,
            "inner_right" => Self::InnerRight,
            "outer_left" => Self::OuterLeft,
            "outer_right" => Self::OuterRight,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureblockMode {
    Save,
    Load,
    Corner,
    Data,
}
impl EnumVariants for StructureblockMode {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Save => 0u16,
            Self::Load => 1u16,
            Self::Corner => 2u16,
            Self::Data => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Save,
            1u16 => Self::Load,
            2u16 => Self::Corner,
            3u16 => Self::Data,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Save => "save",
            Self::Load => "load",
            Self::Corner => "corner",
            Self::Data => "data",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "save" => Self::Save,
            "load" => Self::Load,
            "corner" => Self::Corner,
            "data" => Self::Data,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}
impl EnumVariants for TestBlockMode {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Start => 0u16,
            Self::Log => 1u16,
            Self::Fail => 2u16,
            Self::Accept => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Start,
            1u16 => Self::Log,
            2u16 => Self::Fail,
            3u16 => Self::Accept,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "start" => Self::Start,
            "log" => Self::Log,
            "fail" => Self::Fail,
            "accept" => Self::Accept,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tilt {
    None,
    Unstable,
    Partial,
    Full,
}
impl EnumVariants for Tilt {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::None => 0u16,
            Self::Unstable => 1u16,
            Self::Partial => 2u16,
            Self::Full => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::None,
            1u16 => Self::Unstable,
            2u16 => Self::Partial,
            3u16 => Self::Full,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Unstable => "unstable",
            Self::Partial => "partial",
            Self::Full => "full",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "unstable" => Self::Unstable,
            "partial" => Self::Partial,
            "full" => Self::Full,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrialSpawnerState {
    Inactive,
    WaitingForPlayers,
    Active,
    WaitingForRewardEjection,
    EjectingReward,
    Cooldown,
}
impl EnumVariants for TrialSpawnerState {
    fn variant_count() -> u16 {
        6u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Inactive => 0u16,
            Self::WaitingForPlayers => 1u16,
            Self::Active => 2u16,
            Self::WaitingForRewardEjection => 3u16,
            Self::EjectingReward => 4u16,
            Self::Cooldown => 5u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Inactive,
            1u16 => Self::WaitingForPlayers,
            2u16 => Self::Active,
            3u16 => Self::WaitingForRewardEjection,
            4u16 => Self::EjectingReward,
            5u16 => Self::Cooldown,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::WaitingForPlayers => "waiting_for_players",
            Self::Active => "active",
            Self::WaitingForRewardEjection => "waiting_for_reward_ejection",
            Self::EjectingReward => "ejecting_reward",
            Self::Cooldown => "cooldown",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "inactive" => Self::Inactive,
            "waiting_for_players" => Self::WaitingForPlayers,
            "active" => Self::Active,
            "waiting_for_reward_ejection" => Self::WaitingForRewardEjection,
            "ejecting_reward" => Self::EjectingReward,
            "cooldown" => Self::Cooldown,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultState {
    Inactive,
    Active,
    Unlocking,
    Ejecting,
}
impl EnumVariants for VaultState {
    fn variant_count() -> u16 {
        4u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Inactive => 0u16,
            Self::Active => 1u16,
            Self::Unlocking => 2u16,
            Self::Ejecting => 3u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Inactive,
            1u16 => Self::Active,
            2u16 => Self::Unlocking,
            3u16 => Self::Ejecting,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Unlocking => "unlocking",
            Self::Ejecting => "ejecting",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "inactive" => Self::Inactive,
            "active" => Self::Active,
            "unlocking" => Self::Unlocking,
            "ejecting" => Self::Ejecting,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalDirection {
    Up,
    Down,
}
impl EnumVariants for VerticalDirection {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Up => 0u16,
            Self::Down => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Up,
            1u16 => Self::Down,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "up" => Self::Up,
            "down" => Self::Down,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WestRedstone {
    Up,
    Side,
    None,
}
impl EnumVariants for WestRedstone {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::Up => 0u16,
            Self::Side => 1u16,
            Self::None => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::Up,
            1u16 => Self::Side,
            2u16 => Self::None,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Side => "side",
            Self::None => "none",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "up" => Self::Up,
            "side" => Self::Side,
            "none" => Self::None,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WestWall {
    None,
    Low,
    Tall,
}
impl EnumVariants for WestWall {
    fn variant_count() -> u16 {
        3u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::None => 0u16,
            Self::Low => 1u16,
            Self::Tall => 2u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::None,
            1u16 => Self::Low,
            2u16 => Self::Tall,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Tall => "tall",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "low" => Self::Low,
            "tall" => Self::Tall,
            _ => panic!("Invalid value: {value}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakFenceGateLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#in_wall: bool,
    pub r#open: bool,
    pub r#powered: bool,
}
impl BlockProperties for OakFenceGateLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (!self.r#open as u16, 2),
            (!self.r#in_wall as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#open: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#in_wall: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            369u16
                | 628u16
                | 629u16
                | 630u16
                | 631u16
                | 632u16
                | 633u16
                | 634u16
                | 635u16
                | 636u16
                | 893u16
                | 894u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakFenceGateLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakFenceGateLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakFenceGateLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("in_wall", if self.r#in_wall { "true" } else { "false" }),
            ("open", if self.r#open { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            369u16
                | 628u16
                | 629u16
                | 630u16
                | 631u16
                | 632u16
                | 633u16
                | 634u16
                | 635u16
                | 636u16
                | 893u16
                | 894u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakFenceGateLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "in_wall" => block_props.r#in_wall = matches!(*value, "true"),
                "open" => block_props.r#open = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CakeLikeProperties {
    pub r#bites: u8,
}
impl BlockProperties for CakeLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#bites as u16, 7u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#bites: {
                let value = (index % 7u16) as u8;
                index /= 7u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 298u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CakeLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CakeLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CakeLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "bites",
            match self.r#bites {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 298u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CakeLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "bites" {
                block_props.r#bites = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JukeboxLikeProperties {
    pub r#has_record: bool,
}
impl BlockProperties for JukeboxLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#has_record as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#has_record: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 283u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "JukeboxLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "JukeboxLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "JukeboxLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "has_record",
            if self.r#has_record { "true" } else { "false" },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 283u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "JukeboxLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "has_record" {
                block_props.r#has_record = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetherPortalLikeProperties {
    pub r#axis: HorizontalAxis,
}
impl BlockProperties for NetherPortalLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#axis.to_index(), HorizontalAxis::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#axis: {
                let value = index % HorizontalAxis::variant_count();
                index /= HorizontalAxis::variant_count();
                HorizontalAxis::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 295u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NetherPortalLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "NetherPortalLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NetherPortalLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("axis", self.r#axis.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 295u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NetherPortalLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "axis" {
                block_props.r#axis = HorizontalAxis::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaleMossCarpetLikeProperties {
    pub r#bottom: bool,
    pub r#east: EastWall,
    pub r#north: NorthWall,
    pub r#south: SouthWall,
    pub r#west: WestWall,
}
impl BlockProperties for PaleMossCarpetLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#west.to_index(), WestWall::variant_count()),
            (self.r#south.to_index(), SouthWall::variant_count()),
            (self.r#north.to_index(), NorthWall::variant_count()),
            (self.r#east.to_index(), EastWall::variant_count()),
            (!self.r#bottom as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % WestWall::variant_count();
                index /= WestWall::variant_count();
                WestWall::from_index(value)
            },
            r#south: {
                let value = index % SouthWall::variant_count();
                index /= SouthWall::variant_count();
                SouthWall::from_index(value)
            },
            r#north: {
                let value = index % NorthWall::variant_count();
                index /= NorthWall::variant_count();
                NorthWall::from_index(value)
            },
            r#east: {
                let value = index % EastWall::variant_count();
                index /= EastWall::variant_count();
                EastWall::from_index(value)
            },
            r#bottom: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1189u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleMossCarpetLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PaleMossCarpetLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleMossCarpetLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("bottom", if self.r#bottom { "true" } else { "false" }),
            ("east", self.r#east.to_value()),
            ("north", self.r#north.to_value()),
            ("south", self.r#south.to_value()),
            ("west", self.r#west.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1189u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleMossCarpetLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "bottom" => block_props.r#bottom = matches!(*value, "true"),
                "east" => block_props.r#east = EastWall::from_value(value),
                "north" => block_props.r#north = NorthWall::from_value(value),
                "south" => block_props.r#south = SouthWall::from_value(value),
                "west" => block_props.r#west = WestWall::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResinBrickWallLikeProperties {
    pub r#east: EastWall,
    pub r#north: NorthWall,
    pub r#south: SouthWall,
    pub r#up: bool,
    pub r#waterlogged: bool,
    pub r#west: WestWall,
}
impl BlockProperties for ResinBrickWallLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#west.to_index(), WestWall::variant_count()),
            (!self.r#waterlogged as u16, 2),
            (!self.r#up as u16, 2),
            (self.r#south.to_index(), SouthWall::variant_count()),
            (self.r#north.to_index(), NorthWall::variant_count()),
            (self.r#east.to_index(), EastWall::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % WestWall::variant_count();
                index /= WestWall::variant_count();
                WestWall::from_index(value)
            },
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#up: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % SouthWall::variant_count();
                index /= SouthWall::variant_count();
                SouthWall::from_index(value)
            },
            r#north: {
                let value = index % NorthWall::variant_count();
                index /= NorthWall::variant_count();
                NorthWall::from_index(value)
            },
            r#east: {
                let value = index % EastWall::variant_count();
                index /= EastWall::variant_count();
                EastWall::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            379u16
                | 409u16
                | 410u16
                | 824u16
                | 825u16
                | 826u16
                | 827u16
                | 828u16
                | 829u16
                | 830u16
                | 831u16
                | 832u16
                | 833u16
                | 834u16
                | 835u16
                | 836u16
                | 926u16
                | 934u16
                | 940u16
                | 987u16
                | 991u16
                | 996u16
                | 1002u16
                | 1006u16
                | 1010u16
                | 1015u16
                | 1019u16
                | 1023u16
                | 1155u16
                | 1159u16
                | 1163u16
                | 1167u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ResinBrickWallLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ResinBrickWallLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ResinBrickWallLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("east", self.r#east.to_value()),
            ("north", self.r#north.to_value()),
            ("south", self.r#south.to_value()),
            ("up", if self.r#up { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
            ("west", self.r#west.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            379u16
                | 409u16
                | 410u16
                | 824u16
                | 825u16
                | 826u16
                | 827u16
                | 828u16
                | 829u16
                | 830u16
                | 831u16
                | 832u16
                | 833u16
                | 834u16
                | 835u16
                | 836u16
                | 926u16
                | 934u16
                | 940u16
                | 987u16
                | 991u16
                | 996u16
                | 1002u16
                | 1006u16
                | 1010u16
                | 1015u16
                | 1019u16
                | 1023u16
                | 1155u16
                | 1159u16
                | 1163u16
                | 1167u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ResinBrickWallLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "east" => block_props.r#east = EastWall::from_value(value),
                "north" => block_props.r#north = NorthWall::from_value(value),
                "south" => block_props.r#south = SouthWall::from_value(value),
                "up" => block_props.r#up = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                "west" => block_props.r#west = WestWall::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriedGhastLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#hydration: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for DriedGhastLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#hydration as u16, 4u16),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#hydration: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 747u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DriedGhastLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "DriedGhastLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DriedGhastLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "hydration",
                match self.r#hydration {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 747u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DriedGhastLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "hydration" => {
                    block_props.r#hydration = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        _ => 0u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MovingPistonLikeProperties {
    pub r#facing: Facing,
    pub r#type: PistonType,
}
impl BlockProperties for MovingPistonLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#type.to_index(), PistonType::variant_count()),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#type: {
                let value = index % PistonType::variant_count();
                index /= PistonType::variant_count();
                PistonType::from_index(value)
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 156u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MovingPistonLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "MovingPistonLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MovingPistonLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("type", self.r#type.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 156u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MovingPistonLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "type" => block_props.r#type = PistonType::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PistonHeadLikeProperties {
    pub r#facing: Facing,
    pub r#short: bool,
    pub r#type: PistonType,
}
impl BlockProperties for PistonHeadLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#type.to_index(), PistonType::variant_count()),
            (!self.r#short as u16, 2),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#type: {
                let value = index % PistonType::variant_count();
                index /= PistonType::variant_count();
                PistonType::from_index(value)
            },
            r#short: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 139u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PistonHeadLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PistonHeadLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PistonHeadLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("short", if self.r#short { "true" } else { "false" }),
            ("type", self.r#type.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 139u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PistonHeadLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "short" => block_props.r#short = matches!(*value, "true"),
                "type" => block_props.r#type = PistonType::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalibratedSculkSensorLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#power: u8,
    pub r#sculk_sensor_phase: SculkSensorPhase,
    pub r#waterlogged: bool,
}
impl BlockProperties for CalibratedSculkSensorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (
                self.r#sculk_sensor_phase.to_index(),
                SculkSensorPhase::variant_count(),
            ),
            (self.r#power as u16, 16u16),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#sculk_sensor_phase: {
                let value = index % SculkSensorPhase::variant_count();
                index /= SculkSensorPhase::variant_count();
                SculkSensorPhase::from_index(value)
            },
            r#power: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1029u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CalibratedSculkSensorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CalibratedSculkSensorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CalibratedSculkSensorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "power",
                match self.r#power {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            ("sculk_sensor_phase", self.r#sculk_sensor_phase.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1029u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CalibratedSculkSensorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "power" => {
                    block_props.r#power = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "sculk_sensor_phase" => {
                    block_props.r#sculk_sensor_phase = SculkSensorPhase::from_value(value)
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SculkSensorLikeProperties {
    pub r#power: u8,
    pub r#sculk_sensor_phase: SculkSensorPhase,
    pub r#waterlogged: bool,
}
impl BlockProperties for SculkSensorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (
                self.r#sculk_sensor_phase.to_index(),
                SculkSensorPhase::variant_count(),
            ),
            (self.r#power as u16, 16u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#sculk_sensor_phase: {
                let value = index % SculkSensorPhase::variant_count();
                index /= SculkSensorPhase::variant_count();
                SculkSensorPhase::from_index(value)
            },
            r#power: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1028u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkSensorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SculkSensorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkSensorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "power",
                match self.r#power {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            ("sculk_sensor_phase", self.r#sculk_sensor_phase.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1028u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkSensorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "power" => {
                    block_props.r#power = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "sculk_sensor_phase" => {
                    block_props.r#sculk_sensor_phase = SculkSensorPhase::from_value(value)
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StickyPistonLikeProperties {
    pub r#extended: bool,
    pub r#facing: Facing,
}
impl BlockProperties for StickyPistonLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#facing.to_index(), Facing::variant_count()),
            (!self.r#extended as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
            r#extended: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 128u16 | 138u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StickyPistonLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "StickyPistonLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StickyPistonLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("extended", if self.r#extended { "true" } else { "false" }),
            ("facing", self.r#facing.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 128u16 | 138u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StickyPistonLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "extended" => block_props.r#extended = matches!(*value, "true"),
                "facing" => block_props.r#facing = Facing::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedstoneWireLikeProperties {
    pub r#east: EastRedstone,
    pub r#north: NorthRedstone,
    pub r#power: u8,
    pub r#south: SouthRedstone,
    pub r#west: WestRedstone,
}
impl BlockProperties for RedstoneWireLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#west.to_index(), WestRedstone::variant_count()),
            (self.r#south.to_index(), SouthRedstone::variant_count()),
            (self.r#power as u16, 16u16),
            (self.r#north.to_index(), NorthRedstone::variant_count()),
            (self.r#east.to_index(), EastRedstone::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % WestRedstone::variant_count();
                index /= WestRedstone::variant_count();
                WestRedstone::from_index(value)
            },
            r#south: {
                let value = index % SouthRedstone::variant_count();
                index /= SouthRedstone::variant_count();
                SouthRedstone::from_index(value)
            },
            r#power: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
            r#north: {
                let value = index % NorthRedstone::variant_count();
                index /= NorthRedstone::variant_count();
                NorthRedstone::from_index(value)
            },
            r#east: {
                let value = index % EastRedstone::variant_count();
                index /= EastRedstone::variant_count();
                EastRedstone::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 202u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RedstoneWireLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "RedstoneWireLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RedstoneWireLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("east", self.r#east.to_value()),
            ("north", self.r#north.to_value()),
            (
                "power",
                match self.r#power {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            ("south", self.r#south.to_value()),
            ("west", self.r#west.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 202u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RedstoneWireLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "east" => block_props.r#east = EastRedstone::from_value(value),
                "north" => block_props.r#north = NorthRedstone::from_value(value),
                "power" => {
                    block_props.r#power = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "south" => block_props.r#south = SouthRedstone::from_value(value),
                "west" => block_props.r#west = WestRedstone::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChiseledBookshelfLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#slot_0_occupied: bool,
    pub r#slot_1_occupied: bool,
    pub r#slot_2_occupied: bool,
    pub r#slot_3_occupied: bool,
    pub r#slot_4_occupied: bool,
    pub r#slot_5_occupied: bool,
}
impl BlockProperties for ChiseledBookshelfLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#slot_5_occupied as u16, 2),
            (!self.r#slot_4_occupied as u16, 2),
            (!self.r#slot_3_occupied as u16, 2),
            (!self.r#slot_2_occupied as u16, 2),
            (!self.r#slot_1_occupied as u16, 2),
            (!self.r#slot_0_occupied as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#slot_5_occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#slot_4_occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#slot_3_occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#slot_2_occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#slot_1_occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#slot_0_occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 179u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChiseledBookshelfLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ChiseledBookshelfLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChiseledBookshelfLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "slot_0_occupied",
                if self.r#slot_0_occupied {
                    "true"
                } else {
                    "false"
                },
            ),
            (
                "slot_1_occupied",
                if self.r#slot_1_occupied {
                    "true"
                } else {
                    "false"
                },
            ),
            (
                "slot_2_occupied",
                if self.r#slot_2_occupied {
                    "true"
                } else {
                    "false"
                },
            ),
            (
                "slot_3_occupied",
                if self.r#slot_3_occupied {
                    "true"
                } else {
                    "false"
                },
            ),
            (
                "slot_4_occupied",
                if self.r#slot_4_occupied {
                    "true"
                } else {
                    "false"
                },
            ),
            (
                "slot_5_occupied",
                if self.r#slot_5_occupied {
                    "true"
                } else {
                    "false"
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 179u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChiseledBookshelfLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "slot_0_occupied" => block_props.r#slot_0_occupied = matches!(*value, "true"),
                "slot_1_occupied" => block_props.r#slot_1_occupied = matches!(*value, "true"),
                "slot_2_occupied" => block_props.r#slot_2_occupied = matches!(*value, "true"),
                "slot_3_occupied" => block_props.r#slot_3_occupied = matches!(*value, "true"),
                "slot_4_occupied" => block_props.r#slot_4_occupied = matches!(*value, "true"),
                "slot_5_occupied" => block_props.r#slot_5_occupied = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructureBlockLikeProperties {
    pub r#mode: StructureblockMode,
}
impl BlockProperties for StructureBlockLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#mode.to_index(), StructureblockMode::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#mode: {
                let value = index % StructureblockMode::variant_count();
                index /= StructureblockMode::variant_count();
                StructureblockMode::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 905u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StructureBlockLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "StructureBlockLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StructureBlockLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("mode", self.r#mode.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 905u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StructureBlockLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "mode" {
                block_props.r#mode = StructureblockMode::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChorusFlowerLikeProperties {
    pub r#age: u8,
}
impl BlockProperties for ChorusFlowerLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#age as u16, 6u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#age: {
                let value = (index % 6u16) as u8;
                index /= 6u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 657u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChorusFlowerLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ChorusFlowerLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChorusFlowerLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "age",
            match self.r#age {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 657u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChorusFlowerLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "age" {
                block_props.r#age = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CactusLikeProperties {
    pub r#age: u8,
}
impl BlockProperties for CactusLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#age as u16, 16u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#age: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 279u16 | 282u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CactusLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CactusLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CactusLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "age",
            match self.r#age {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                9u8 => "9",
                10u8 => "10",
                11u8 => "11",
                12u8 => "12",
                13u8 => "13",
                14u8 => "14",
                15u8 => "15",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 279u16 | 282u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CactusLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "age" {
                block_props.r#age = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    "9" => 9u8,
                    "10" => 10u8,
                    "11" => 11u8,
                    "12" => 12u8,
                    "13" => 13u8,
                    "14" => 14u8,
                    "15" => 15u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FireLikeProperties {
    pub r#age: u8,
    pub r#east: bool,
    pub r#north: bool,
    pub r#south: bool,
    pub r#up: bool,
    pub r#west: bool,
}
impl BlockProperties for FireLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#west as u16, 2),
            (!self.r#up as u16, 2),
            (!self.r#south as u16, 2),
            (!self.r#north as u16, 2),
            (!self.r#east as u16, 2),
            (self.r#age as u16, 16u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#up: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#north: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#east: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#age: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 196u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FireLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "FireLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FireLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "age",
                match self.r#age {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            ("east", if self.r#east { "true" } else { "false" }),
            ("north", if self.r#north { "true" } else { "false" }),
            ("south", if self.r#south { "true" } else { "false" }),
            ("up", if self.r#up { "true" } else { "false" }),
            ("west", if self.r#west { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 196u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FireLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "age" => {
                    block_props.r#age = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "east" => block_props.r#east = matches!(*value, "true"),
                "north" => block_props.r#north = matches!(*value, "true"),
                "south" => block_props.r#south = matches!(*value, "true"),
                "up" => block_props.r#up = matches!(*value, "true"),
                "west" => block_props.r#west = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HopperLikeProperties {
    pub r#enabled: bool,
    pub r#facing: FacingHopper,
}
impl BlockProperties for HopperLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#facing.to_index(), FacingHopper::variant_count()),
            (!self.r#enabled as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % FacingHopper::variant_count();
                index /= FacingHopper::variant_count();
                FacingHopper::from_index(value)
            },
            r#enabled: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 477u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "HopperLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "HopperLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "HopperLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("enabled", if self.r#enabled { "true" } else { "false" }),
            ("facing", self.r#facing.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 477u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "HopperLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "enabled" => block_props.r#enabled = matches!(*value, "true"),
                "facing" => block_props.r#facing = FacingHopper::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrialSpawnerLikeProperties {
    pub r#ominous: bool,
    pub r#trial_spawner_state: TrialSpawnerState,
}
impl BlockProperties for TrialSpawnerLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (
                self.r#trial_spawner_state.to_index(),
                TrialSpawnerState::variant_count(),
            ),
            (!self.r#ominous as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#trial_spawner_state: {
                let value = index % TrialSpawnerState::variant_count();
                index /= TrialSpawnerState::variant_count();
                TrialSpawnerState::from_index(value)
            },
            r#ominous: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1185u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TrialSpawnerLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TrialSpawnerLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TrialSpawnerLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("ominous", if self.r#ominous { "true" } else { "false" }),
            ("trial_spawner_state", self.r#trial_spawner_state.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1185u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TrialSpawnerLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "ominous" => block_props.r#ominous = matches!(*value, "true"),
                "trial_spawner_state" => {
                    block_props.r#trial_spawner_state = TrialSpawnerState::from_value(value)
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScaffoldingLikeProperties {
    pub r#bottom: bool,
    pub r#distance: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for ScaffoldingLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#distance as u16, 8u16),
            (!self.r#bottom as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#distance: {
                let value = (index % 8u16) as u8;
                index /= 8u16;
                value
            },
            r#bottom: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 837u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ScaffoldingLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ScaffoldingLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ScaffoldingLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("bottom", if self.r#bottom { "true" } else { "false" }),
            (
                "distance",
                match self.r#distance {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 837u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ScaffoldingLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "bottom" => block_props.r#bottom = matches!(*value, "true"),
                "distance" => {
                    block_props.r#distance = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        _ => 0u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComparatorLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#mode: ModeComparator,
    pub r#powered: bool,
}
impl BlockProperties for ComparatorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#mode.to_index(), ModeComparator::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#mode: {
                let value = index % ModeComparator::variant_count();
                index /= ModeComparator::variant_count();
                ModeComparator::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 473u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ComparatorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ComparatorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ComparatorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("mode", self.r#mode.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 473u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ComparatorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "mode" => block_props.r#mode = ModeComparator::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WallTorchLikeProperties {
    pub r#facing: HorizontalFacing,
}
impl BlockProperties for WallTorchLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#facing.to_index(), HorizontalFacing::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            195u16
                | 291u16
                | 293u16
                | 296u16
                | 297u16
                | 362u16
                | 363u16
                | 467u16
                | 468u16
                | 469u16
                | 579u16
                | 580u16
                | 581u16
                | 582u16
                | 583u16
                | 584u16
                | 585u16
                | 586u16
                | 587u16
                | 588u16
                | 589u16
                | 590u16
                | 591u16
                | 592u16
                | 593u16
                | 594u16
                | 694u16
                | 695u16
                | 696u16
                | 697u16
                | 698u16
                | 699u16
                | 700u16
                | 701u16
                | 702u16
                | 703u16
                | 704u16
                | 705u16
                | 706u16
                | 707u16
                | 708u16
                | 709u16
                | 838u16
                | 847u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WallTorchLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "WallTorchLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WallTorchLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("facing", self.r#facing.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            195u16
                | 291u16
                | 293u16
                | 296u16
                | 297u16
                | 362u16
                | 363u16
                | 467u16
                | 468u16
                | 469u16
                | 579u16
                | 580u16
                | 581u16
                | 582u16
                | 583u16
                | 584u16
                | 585u16
                | 586u16
                | 587u16
                | 588u16
                | 589u16
                | 590u16
                | 591u16
                | 592u16
                | 593u16
                | 594u16
                | 694u16
                | 695u16
                | 696u16
                | 697u16
                | 698u16
                | 699u16
                | 700u16
                | 701u16
                | 702u16
                | 703u16
                | 704u16
                | 705u16
                | 706u16
                | 707u16
                | 708u16
                | 709u16
                | 838u16
                | 847u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WallTorchLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "facing" {
                block_props.r#facing = HorizontalFacing::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VaultLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#ominous: bool,
    pub r#vault_state: VaultState,
}
impl BlockProperties for VaultLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#vault_state.to_index(), VaultState::variant_count()),
            (!self.r#ominous as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#vault_state: {
                let value = index % VaultState::variant_count();
                index /= VaultState::variant_count();
                VaultState::from_index(value)
            },
            r#ominous: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1186u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "VaultLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "VaultLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "VaultLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("ominous", if self.r#ominous { "true" } else { "false" }),
            ("vault_state", self.r#vault_state.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1186u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "VaultLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "ominous" => block_props.r#ominous = matches!(*value, "true"),
                "vault_state" => block_props.r#vault_state = VaultState::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrindstoneLikeProperties {
    pub r#face: AttachFace,
    pub r#facing: HorizontalFacing,
}
impl BlockProperties for GrindstoneLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (self.r#face.to_index(), AttachFace::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#face: {
                let value = index % AttachFace::variant_count();
                index /= AttachFace::variant_count();
                AttachFace::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 844u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GrindstoneLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "GrindstoneLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GrindstoneLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("face", self.r#face.to_value()),
            ("facing", self.r#facing.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 844u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GrindstoneLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "face" => block_props.r#face = AttachFace::from_value(value),
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeverLikeProperties {
    pub r#face: AttachFace,
    pub r#facing: HorizontalFacing,
    pub r#powered: bool,
}
impl BlockProperties for LeverLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (self.r#face.to_index(), AttachFace::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#face: {
                let value = index % AttachFace::variant_count();
                index /= AttachFace::variant_count();
                AttachFace::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            258u16
                | 275u16
                | 443u16
                | 444u16
                | 445u16
                | 446u16
                | 447u16
                | 448u16
                | 449u16
                | 450u16
                | 451u16
                | 452u16
                | 897u16
                | 898u16
                | 939u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LeverLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LeverLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LeverLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("face", self.r#face.to_value()),
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            258u16
                | 275u16
                | 443u16
                | 444u16
                | 445u16
                | 446u16
                | 447u16
                | 448u16
                | 449u16
                | 450u16
                | 451u16
                | 452u16
                | 897u16
                | 898u16
                | 939u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LeverLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "face" => block_props.r#face = AttachFace::from_value(value),
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepeaterLikeProperties {
    pub r#delay: u8,
    pub r#facing: HorizontalFacing,
    pub r#locked: bool,
    pub r#powered: bool,
}
impl BlockProperties for RepeaterLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (!self.r#locked as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            ((self.r#delay - 1u8) as u16, 4u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#locked: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#delay: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 299u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RepeaterLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "RepeaterLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RepeaterLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "delay",
                match self.r#delay {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
            ("facing", self.r#facing.to_value()),
            ("locked", if self.r#locked { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 299u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RepeaterLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "delay" => {
                    block_props.r#delay = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 1u8,
                    }
                }
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "locked" => block_props.r#locked = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakDoorLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#half: DoubleBlockHalf,
    pub r#hinge: DoorHinge,
    pub r#open: bool,
    pub r#powered: bool,
}
impl BlockProperties for OakDoorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (!self.r#open as u16, 2),
            (self.r#hinge.to_index(), DoorHinge::variant_count()),
            (self.r#half.to_index(), DoubleBlockHalf::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#open: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#hinge: {
                let value = index % DoorHinge::variant_count();
                index /= DoorHinge::variant_count();
                DoorHinge::from_index(value)
            },
            r#half: {
                let value = index % DoubleBlockHalf::variant_count();
                index /= DoubleBlockHalf::variant_count();
                DoubleBlockHalf::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            220u16
                | 260u16
                | 646u16
                | 647u16
                | 648u16
                | 649u16
                | 650u16
                | 651u16
                | 652u16
                | 653u16
                | 654u16
                | 899u16
                | 900u16
                | 1076u16
                | 1077u16
                | 1078u16
                | 1079u16
                | 1080u16
                | 1081u16
                | 1082u16
                | 1083u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakDoorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakDoorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakDoorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("half", self.r#half.to_value()),
            ("hinge", self.r#hinge.to_value()),
            ("open", if self.r#open { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            220u16
                | 260u16
                | 646u16
                | 647u16
                | 648u16
                | 649u16
                | 650u16
                | 651u16
                | 652u16
                | 653u16
                | 654u16
                | 899u16
                | 900u16
                | 1076u16
                | 1077u16
                | 1078u16
                | 1079u16
                | 1080u16
                | 1081u16
                | 1082u16
                | 1083u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakDoorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "half" => block_props.r#half = DoubleBlockHalf::from_value(value),
                "hinge" => block_props.r#hinge = DoorHinge::from_value(value),
                "open" => block_props.r#open = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakStairsLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#half: Half,
    pub r#shape: StairsShape,
    pub r#waterlogged: bool,
}
impl BlockProperties for OakStairsLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#shape.to_index(), StairsShape::variant_count()),
            (self.r#half.to_index(), Half::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#shape: {
                let value = index % StairsShape::variant_count();
                index /= StairsShape::variant_count();
                StairsShape::from_index(value)
            },
            r#half: {
                let value = index % Half::variant_count();
                index /= Half::variant_count();
                Half::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            200u16
                | 223u16
                | 370u16
                | 371u16
                | 372u16
                | 377u16
                | 383u16
                | 397u16
                | 404u16
                | 405u16
                | 406u16
                | 481u16
                | 516u16
                | 517u16
                | 518u16
                | 519u16
                | 520u16
                | 521u16
                | 522u16
                | 530u16
                | 531u16
                | 532u16
                | 598u16
                | 660u16
                | 797u16
                | 798u16
                | 799u16
                | 800u16
                | 801u16
                | 802u16
                | 803u16
                | 804u16
                | 805u16
                | 806u16
                | 807u16
                | 808u16
                | 809u16
                | 810u16
                | 895u16
                | 896u16
                | 925u16
                | 933u16
                | 936u16
                | 986u16
                | 990u16
                | 995u16
                | 1001u16
                | 1005u16
                | 1009u16
                | 1014u16
                | 1018u16
                | 1022u16
                | 1060u16
                | 1061u16
                | 1062u16
                | 1063u16
                | 1064u16
                | 1065u16
                | 1066u16
                | 1067u16
                | 1153u16
                | 1157u16
                | 1161u16
                | 1165u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakStairsLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakStairsLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakStairsLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("half", self.r#half.to_value()),
            ("shape", self.r#shape.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            200u16
                | 223u16
                | 370u16
                | 371u16
                | 372u16
                | 377u16
                | 383u16
                | 397u16
                | 404u16
                | 405u16
                | 406u16
                | 481u16
                | 516u16
                | 517u16
                | 518u16
                | 519u16
                | 520u16
                | 521u16
                | 522u16
                | 530u16
                | 531u16
                | 532u16
                | 598u16
                | 660u16
                | 797u16
                | 798u16
                | 799u16
                | 800u16
                | 801u16
                | 802u16
                | 803u16
                | 804u16
                | 805u16
                | 806u16
                | 807u16
                | 808u16
                | 809u16
                | 810u16
                | 895u16
                | 896u16
                | 925u16
                | 933u16
                | 936u16
                | 986u16
                | 990u16
                | 995u16
                | 1001u16
                | 1005u16
                | 1009u16
                | 1014u16
                | 1018u16
                | 1022u16
                | 1060u16
                | 1061u16
                | 1062u16
                | 1063u16
                | 1064u16
                | 1065u16
                | 1066u16
                | 1067u16
                | 1153u16
                | 1157u16
                | 1161u16
                | 1165u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakStairsLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "half" => block_props.r#half = Half::from_value(value),
                "shape" => block_props.r#shape = StairsShape::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SmallDripleafLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#half: DoubleBlockHalf,
    pub r#waterlogged: bool,
}
impl BlockProperties for SmallDripleafLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#half.to_index(), DoubleBlockHalf::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#half: {
                let value = index % DoubleBlockHalf::variant_count();
                index /= DoubleBlockHalf::variant_count();
                DoubleBlockHalf::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1147u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SmallDripleafLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SmallDripleafLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SmallDripleafLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("half", self.r#half.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1147u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SmallDripleafLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "half" => block_props.r#half = DoubleBlockHalf::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkeletonWallSkullLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#powered: bool,
}
impl BlockProperties for SkeletonWallSkullLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            454u16 | 456u16 | 458u16 | 460u16 | 462u16 | 464u16 | 466u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SkeletonWallSkullLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SkeletonWallSkullLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SkeletonWallSkullLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            454u16 | 456u16 | 458u16 | 460u16 | 462u16 | 464u16 | 466u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SkeletonWallSkullLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakTrapdoorLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#half: Half,
    pub r#open: bool,
    pub r#powered: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for OakTrapdoorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (!self.r#powered as u16, 2),
            (!self.r#open as u16, 2),
            (self.r#half.to_index(), Half::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#open: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#half: {
                let value = index % Half::variant_count();
                index /= Half::variant_count();
                Half::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            316u16
                | 317u16
                | 318u16
                | 319u16
                | 320u16
                | 321u16
                | 322u16
                | 323u16
                | 324u16
                | 325u16
                | 526u16
                | 891u16
                | 892u16
                | 1084u16
                | 1085u16
                | 1086u16
                | 1087u16
                | 1088u16
                | 1089u16
                | 1090u16
                | 1091u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakTrapdoorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakTrapdoorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakTrapdoorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("half", self.r#half.to_value()),
            ("open", if self.r#open { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            316u16
                | 317u16
                | 318u16
                | 319u16
                | 320u16
                | 321u16
                | 322u16
                | 323u16
                | 324u16
                | 325u16
                | 526u16
                | 891u16
                | 892u16
                | 1084u16
                | 1085u16
                | 1086u16
                | 1087u16
                | 1088u16
                | 1089u16
                | 1090u16
                | 1091u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakTrapdoorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "half" => block_props.r#half = Half::from_value(value),
                "open" => block_props.r#open = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LecternLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#has_book: bool,
    pub r#powered: bool,
}
impl BlockProperties for LecternLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (!self.r#has_book as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#has_book: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 845u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LecternLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LecternLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LecternLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("has_book", if self.r#has_book { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 845u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LecternLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "has_book" => block_props.r#has_book = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BellLikeProperties {
    pub r#attachment: BellAttachment,
    pub r#facing: HorizontalFacing,
    pub r#powered: bool,
}
impl BlockProperties for BellLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (
                self.r#attachment.to_index(),
                BellAttachment::variant_count(),
            ),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#attachment: {
                let value = index % BellAttachment::variant_count();
                index /= BellAttachment::variant_count();
                BellAttachment::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 848u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BellLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BellLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BellLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("attachment", self.r#attachment.to_value()),
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 848u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BellLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "attachment" => block_props.r#attachment = BellAttachment::from_value(value),
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TripwireHookLikeProperties {
    pub r#attached: bool,
    pub r#facing: HorizontalFacing,
    pub r#powered: bool,
}
impl BlockProperties for TripwireHookLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (!self.r#attached as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#attached: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 401u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TripwireHookLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TripwireHookLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TripwireHookLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("attached", if self.r#attached { "true" } else { "false" }),
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 401u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TripwireHookLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "attached" => block_props.r#attached = matches!(*value, "true"),
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcaciaShelfLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#powered: bool,
    pub r#side_chain: SideChainPart,
    pub r#waterlogged: bool,
}
impl BlockProperties for AcaciaShelfLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#side_chain.to_index(), SideChainPart::variant_count()),
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#side_chain: {
                let value = index % SideChainPart::variant_count();
                index /= SideChainPart::variant_count();
                SideChainPart::from_index(value)
            },
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            180u16
                | 181u16
                | 182u16
                | 183u16
                | 184u16
                | 185u16
                | 186u16
                | 187u16
                | 188u16
                | 189u16
                | 190u16
                | 191u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "AcaciaShelfLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "AcaciaShelfLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "AcaciaShelfLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
            ("side_chain", self.r#side_chain.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            180u16
                | 181u16
                | 182u16
                | 183u16
                | 184u16
                | 185u16
                | 186u16
                | 187u16
                | 188u16
                | 189u16
                | 190u16
                | 191u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "AcaciaShelfLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                "side_chain" => block_props.r#side_chain = SideChainPart::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndPortalFrameLikeProperties {
    pub r#eye: bool,
    pub r#facing: HorizontalFacing,
}
impl BlockProperties for EndPortalFrameLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (!self.r#eye as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#eye: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 392u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "EndPortalFrameLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "EndPortalFrameLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "EndPortalFrameLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("eye", if self.r#eye { "true" } else { "false" }),
            ("facing", self.r#facing.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 392u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "EndPortalFrameLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "eye" => block_props.r#eye = matches!(*value, "true"),
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FurnaceLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#lit: bool,
}
impl BlockProperties for FurnaceLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#lit as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#lit: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 209u16 | 274u16 | 840u16 | 841u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FurnaceLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "FurnaceLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FurnaceLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("lit", if self.r#lit { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 209u16 | 274u16 | 840u16 | 841u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FurnaceLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "lit" => block_props.r#lit = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CampfireLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#lit: bool,
    pub r#signal_fire: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for CampfireLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (!self.r#signal_fire as u16, 2),
            (!self.r#lit as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#signal_fire: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#lit: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 859u16 | 860u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CampfireLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CampfireLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CampfireLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("lit", if self.r#lit { "true" } else { "false" }),
            (
                "signal_fire",
                if self.r#signal_fire { "true" } else { "false" },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 859u16 | 860u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CampfireLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "lit" => block_props.r#lit = matches!(*value, "true"),
                "signal_fire" => block_props.r#signal_fire = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopperGolemStatueLikeProperties {
    pub r#copper_golem_pose: CopperGolemPose,
    pub r#facing: HorizontalFacing,
    pub r#waterlogged: bool,
}
impl BlockProperties for CopperGolemStatueLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (
                self.r#copper_golem_pose.to_index(),
                CopperGolemPose::variant_count(),
            ),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#copper_golem_pose: {
                let value = index % CopperGolemPose::variant_count();
                index /= CopperGolemPose::variant_count();
                CopperGolemPose::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            1116u16 | 1117u16 | 1118u16 | 1119u16 | 1120u16 | 1121u16 | 1122u16 | 1123u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CopperGolemStatueLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CopperGolemStatueLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CopperGolemStatueLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("copper_golem_pose", self.r#copper_golem_pose.to_value()),
            ("facing", self.r#facing.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            1116u16 | 1117u16 | 1118u16 | 1119u16 | 1120u16 | 1121u16 | 1122u16 | 1123u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CopperGolemStatueLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "copper_golem_pose" => {
                    block_props.r#copper_golem_pose = CopperGolemPose::from_value(value)
                }
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WhiteBedLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#occupied: bool,
    pub r#part: BedPart,
}
impl BlockProperties for WhiteBedLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#part.to_index(), BedPart::variant_count()),
            (!self.r#occupied as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#part: {
                let value = index % BedPart::variant_count();
                index /= BedPart::variant_count();
                BedPart::from_index(value)
            },
            r#occupied: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            110u16
                | 111u16
                | 112u16
                | 113u16
                | 114u16
                | 115u16
                | 116u16
                | 117u16
                | 118u16
                | 119u16
                | 120u16
                | 121u16
                | 122u16
                | 123u16
                | 124u16
                | 125u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WhiteBedLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "WhiteBedLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WhiteBedLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("occupied", if self.r#occupied { "true" } else { "false" }),
            ("part", self.r#part.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            110u16
                | 111u16
                | 112u16
                | 113u16
                | 114u16
                | 115u16
                | 116u16
                | 117u16
                | 118u16
                | 119u16
                | 120u16
                | 121u16
                | 122u16
                | 123u16
                | 124u16
                | 125u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WhiteBedLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "occupied" => block_props.r#occupied = matches!(*value, "true"),
                "part" => block_props.r#part = BedPart::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeeNestLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#honey_level: u8,
}
impl BlockProperties for BeeNestLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#honey_level as u16, 6u16),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#honey_level: {
                let value = (index % 6u16) as u8;
                index /= 6u16;
                value
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 911u16 | 912u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BeeNestLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BeeNestLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BeeNestLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "honey_level",
                match self.r#honey_level {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 911u16 | 912u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BeeNestLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "honey_level" => {
                    block_props.r#honey_level = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        _ => 0u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecoratedPotLikeProperties {
    pub r#cracked: bool,
    pub r#facing: HorizontalFacing,
    pub r#waterlogged: bool,
}
impl BlockProperties for DecoratedPotLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (!self.r#cracked as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#cracked: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1183u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DecoratedPotLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "DecoratedPotLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DecoratedPotLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("cracked", if self.r#cracked { "true" } else { "false" }),
            ("facing", self.r#facing.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1183u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DecoratedPotLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "cracked" => block_props.r#cracked = matches!(*value, "true"),
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LadderLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#waterlogged: bool,
}
impl BlockProperties for LadderLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            221u16
                | 224u16
                | 225u16
                | 226u16
                | 227u16
                | 228u16
                | 229u16
                | 230u16
                | 231u16
                | 232u16
                | 233u16
                | 246u16
                | 247u16
                | 248u16
                | 249u16
                | 250u16
                | 251u16
                | 252u16
                | 253u16
                | 254u16
                | 255u16
                | 256u16
                | 257u16
                | 400u16
                | 778u16
                | 779u16
                | 780u16
                | 781u16
                | 782u16
                | 783u16
                | 784u16
                | 785u16
                | 786u16
                | 787u16
                | 903u16
                | 904u16
                | 1146u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LadderLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LadderLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LadderLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            221u16
                | 224u16
                | 225u16
                | 226u16
                | 227u16
                | 228u16
                | 229u16
                | 230u16
                | 231u16
                | 232u16
                | 233u16
                | 246u16
                | 247u16
                | 248u16
                | 249u16
                | 250u16
                | 251u16
                | 252u16
                | 253u16
                | 254u16
                | 255u16
                | 256u16
                | 257u16
                | 400u16
                | 778u16
                | 779u16
                | 780u16
                | 781u16
                | 782u16
                | 783u16
                | 784u16
                | 785u16
                | 786u16
                | 787u16
                | 903u16
                | 904u16
                | 1146u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LadderLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BigDripleafLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#tilt: Tilt,
    pub r#waterlogged: bool,
}
impl BlockProperties for BigDripleafLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#tilt.to_index(), Tilt::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#tilt: {
                let value = index % Tilt::variant_count();
                index /= Tilt::variant_count();
                Tilt::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1145u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BigDripleafLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BigDripleafLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BigDripleafLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("tilt", self.r#tilt.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1145u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BigDripleafLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "tilt" => block_props.r#tilt = Tilt::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChestLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#type: ChestType,
    pub r#waterlogged: bool,
}
impl BlockProperties for ChestLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#type.to_index(), ChestType::variant_count()),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#type: {
                let value = index % ChestType::variant_count();
                index /= ChestType::variant_count();
                ChestType::from_index(value)
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            201u16
                | 470u16
                | 1108u16
                | 1109u16
                | 1110u16
                | 1111u16
                | 1112u16
                | 1113u16
                | 1114u16
                | 1115u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChestLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ChestLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChestLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("type", self.r#type.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            201u16
                | 470u16
                | 1108u16
                | 1109u16
                | 1110u16
                | 1111u16
                | 1112u16
                | 1113u16
                | 1114u16
                | 1115u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ChestLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "type" => block_props.r#type = ChestType::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PinkPetalsLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#flower_amount: u8,
}
impl BlockProperties for PinkPetalsLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            ((self.r#flower_amount - 1u8) as u16, 4u16),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#flower_amount: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value + 1u8
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1141u16 | 1142u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PinkPetalsLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PinkPetalsLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PinkPetalsLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "flower_amount",
                match self.r#flower_amount {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1141u16 | 1142u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PinkPetalsLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "flower_amount" => {
                    block_props.r#flower_amount = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 1u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeafLitterLikeProperties {
    pub r#facing: HorizontalFacing,
    pub r#segment_amount: u8,
}
impl BlockProperties for LeafLitterLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            ((self.r#segment_amount - 1u8) as u16, 4u16),
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#segment_amount: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value + 1u8
            },
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1143u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LeafLitterLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LeafLitterLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LeafLitterLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "segment_amount",
                match self.r#segment_amount {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1143u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LeafLitterLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                "segment_amount" => {
                    block_props.r#segment_amount = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 1u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CocoaLikeProperties {
    pub r#age: u8,
    pub r#facing: HorizontalFacing,
}
impl BlockProperties for CocoaLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#facing.to_index(), HorizontalFacing::variant_count()),
            (self.r#age as u16, 3u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % HorizontalFacing::variant_count();
                index /= HorizontalFacing::variant_count();
                HorizontalFacing::from_index(value)
            },
            r#age: {
                let value = (index % 3u16) as u8;
                index /= 3u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 396u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CocoaLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CocoaLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CocoaLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "age",
                match self.r#age {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    _ => unreachable!(),
                },
            ),
            ("facing", self.r#facing.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 396u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CocoaLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "age" => {
                    block_props.r#age = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        _ => 0u8,
                    }
                }
                "facing" => block_props.r#facing = HorizontalFacing::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SculkShriekerLikeProperties {
    pub r#can_summon: bool,
    pub r#shrieking: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for SculkShriekerLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (!self.r#shrieking as u16, 2),
            (!self.r#can_summon as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#shrieking: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#can_summon: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1033u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkShriekerLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SculkShriekerLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkShriekerLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "can_summon",
                if self.r#can_summon { "true" } else { "false" },
            ),
            ("shrieking", if self.r#shrieking { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1033u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkShriekerLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "can_summon" => block_props.r#can_summon = matches!(*value, "true"),
                "shrieking" => block_props.r#shrieking = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PotentSulfurLikeProperties {
    pub r#potent_sulfur_state: PotentSulfurState,
}
impl BlockProperties for PotentSulfurLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(
            self.r#potent_sulfur_state.to_index(),
            PotentSulfurState::variant_count(),
        )]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#potent_sulfur_state: {
                let value = index % PotentSulfurState::variant_count();
                index /= PotentSulfurState::variant_count();
                PotentSulfurState::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 999u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PotentSulfurLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PotentSulfurLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PotentSulfurLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("potent_sulfur_state", self.r#potent_sulfur_state.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 999u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PotentSulfurLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "potent_sulfur_state" {
                block_props.r#potent_sulfur_state = PotentSulfurState::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BambooLikeProperties {
    pub r#age: u8,
    pub r#leaves: BambooLeaves,
    pub r#stage: u8,
}
impl BlockProperties for BambooLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#stage as u16, 2u16),
            (self.r#leaves.to_index(), BambooLeaves::variant_count()),
            (self.r#age as u16, 2u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#stage: {
                let value = (index % 2u16) as u8;
                index /= 2u16;
                value
            },
            r#leaves: {
                let value = index % BambooLeaves::variant_count();
                index /= BambooLeaves::variant_count();
                BambooLeaves::from_index(value)
            },
            r#age: {
                let value = (index % 2u16) as u8;
                index /= 2u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 792u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BambooLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BambooLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BambooLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "age",
                match self.r#age {
                    0u8 => "0",
                    1u8 => "1",
                    _ => unreachable!(),
                },
            ),
            ("leaves", self.r#leaves.to_value()),
            (
                "stage",
                match self.r#stage {
                    0u8 => "0",
                    1u8 => "1",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 792u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BambooLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "age" => {
                    block_props.r#age = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        _ => 0u8,
                    }
                }
                "leaves" => block_props.r#leaves = BambooLeaves::from_value(value),
                "stage" => {
                    block_props.r#stage = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        _ => 0u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RespawnAnchorLikeProperties {
    pub r#charges: u8,
}
impl BlockProperties for RespawnAnchorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#charges as u16, 5u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#charges: {
                let value = (index % 5u16) as u8;
                index /= 5u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 918u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RespawnAnchorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "RespawnAnchorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RespawnAnchorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "charges",
            match self.r#charges {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 918u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RespawnAnchorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "charges" {
                block_props.r#charges = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndRodLikeProperties {
    pub r#facing: Facing,
}
impl BlockProperties for EndRodLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#facing.to_index(), Facing::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            655u16
                | 677u16
                | 678u16
                | 679u16
                | 680u16
                | 681u16
                | 682u16
                | 683u16
                | 684u16
                | 685u16
                | 686u16
                | 687u16
                | 688u16
                | 689u16
                | 690u16
                | 691u16
                | 692u16
                | 693u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "EndRodLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "EndRodLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "EndRodLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("facing", self.r#facing.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            655u16
                | 677u16
                | 678u16
                | 679u16
                | 680u16
                | 681u16
                | 682u16
                | 683u16
                | 684u16
                | 685u16
                | 686u16
                | 687u16
                | 688u16
                | 689u16
                | 690u16
                | 691u16
                | 692u16
                | 693u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "EndRodLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "facing" {
                block_props.r#facing = Facing::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispenserLikeProperties {
    pub r#facing: Facing,
    pub r#triggered: bool,
}
impl BlockProperties for DispenserLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#triggered as u16, 2),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#triggered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 105u16 | 483u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DispenserLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "DispenserLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DispenserLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("triggered", if self.r#triggered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 105u16 | 483u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DispenserLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "triggered" => block_props.r#triggered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObserverLikeProperties {
    pub r#facing: Facing,
    pub r#powered: bool,
}
impl BlockProperties for ObserverLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 676u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ObserverLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ObserverLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ObserverLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 676u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ObserverLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightningRodLikeProperties {
    pub r#facing: Facing,
    pub r#powered: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for LightningRodLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (!self.r#powered as u16, 2),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            1124u16 | 1125u16 | 1126u16 | 1127u16 | 1128u16 | 1129u16 | 1130u16 | 1131u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightningRodLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LightningRodLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightningRodLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("powered", if self.r#powered { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            1124u16 | 1125u16 | 1126u16 | 1127u16 | 1128u16 | 1129u16 | 1130u16 | 1131u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightningRodLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BarrelLikeProperties {
    pub r#facing: Facing,
    pub r#open: bool,
}
impl BlockProperties for BarrelLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#open as u16, 2),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#open: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 839u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BarrelLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BarrelLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BarrelLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            ("open", if self.r#open { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 839u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BarrelLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "open" => block_props.r#open = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AmethystClusterLikeProperties {
    pub r#facing: Facing,
    pub r#waterlogged: bool,
}
impl BlockProperties for AmethystClusterLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#facing.to_index(), Facing::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 980u16 | 981u16 | 982u16 | 983u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "AmethystClusterLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "AmethystClusterLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "AmethystClusterLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("facing", self.r#facing.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 980u16 | 981u16 | 982u16 | 983u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "AmethystClusterLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "facing" => block_props.r#facing = Facing::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandBlockLikeProperties {
    pub r#conditional: bool,
    pub r#facing: Facing,
}
impl BlockProperties for CommandBlockLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#facing.to_index(), Facing::variant_count()),
            (!self.r#conditional as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#facing: {
                let value = index % Facing::variant_count();
                index /= Facing::variant_count();
                Facing::from_index(value)
            },
            r#conditional: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 407u16 | 668u16 | 669u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CommandBlockLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CommandBlockLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CommandBlockLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "conditional",
                if self.r#conditional { "true" } else { "false" },
            ),
            ("facing", self.r#facing.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 407u16 | 668u16 | 669u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CommandBlockLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "conditional" => block_props.r#conditional = matches!(*value, "true"),
                "facing" => block_props.r#facing = Facing::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoteBlockLikeProperties {
    pub r#instrument: NoteblockInstrument,
    pub r#note: u8,
    pub r#powered: bool,
}
impl BlockProperties for NoteBlockLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#powered as u16, 2),
            (self.r#note as u16, 25u16),
            (
                self.r#instrument.to_index(),
                NoteblockInstrument::variant_count(),
            ),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#note: {
                let value = (index % 25u16) as u8;
                index /= 25u16;
                value
            },
            r#instrument: {
                let value = index % NoteblockInstrument::variant_count();
                index /= NoteblockInstrument::variant_count();
                NoteblockInstrument::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 109u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NoteBlockLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "NoteBlockLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NoteBlockLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("instrument", self.r#instrument.to_value()),
            (
                "note",
                match self.r#note {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    16u8 => "16",
                    17u8 => "17",
                    18u8 => "18",
                    19u8 => "19",
                    20u8 => "20",
                    21u8 => "21",
                    22u8 => "22",
                    23u8 => "23",
                    24u8 => "24",
                    _ => unreachable!(),
                },
            ),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 109u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NoteBlockLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "instrument" => block_props.r#instrument = NoteblockInstrument::from_value(value),
                "note" => {
                    block_props.r#note = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        "16" => 16u8,
                        "17" => 17u8,
                        "18" => 18u8,
                        "19" => 19u8,
                        "20" => 20u8,
                        "21" => 21u8,
                        "22" => 22u8,
                        "23" => 23u8,
                        "24" => 24u8,
                        _ => 0u8,
                    }
                }
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrewingStandLikeProperties {
    pub r#has_bottle_0: bool,
    pub r#has_bottle_1: bool,
    pub r#has_bottle_2: bool,
}
impl BlockProperties for BrewingStandLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#has_bottle_2 as u16, 2),
            (!self.r#has_bottle_1 as u16, 2),
            (!self.r#has_bottle_0 as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#has_bottle_2: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#has_bottle_1: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#has_bottle_0: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 386u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BrewingStandLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BrewingStandLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BrewingStandLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "has_bottle_0",
                if self.r#has_bottle_0 { "true" } else { "false" },
            ),
            (
                "has_bottle_1",
                if self.r#has_bottle_1 { "true" } else { "false" },
            ),
            (
                "has_bottle_2",
                if self.r#has_bottle_2 { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 386u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BrewingStandLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "has_bottle_0" => block_props.r#has_bottle_0 = matches!(*value, "true"),
                "has_bottle_1" => block_props.r#has_bottle_1 = matches!(*value, "true"),
                "has_bottle_2" => block_props.r#has_bottle_2 = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnowLikeProperties {
    pub r#layers: u8,
}
impl BlockProperties for SnowLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [((self.r#layers - 1u8) as u16, 8u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#layers: {
                let value = (index % 8u16) as u8;
                index /= 8u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 276u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SnowLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SnowLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SnowLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "layers",
            match self.r#layers {
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 276u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SnowLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "layers" {
                block_props.r#layers = match *value {
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    _ => 1u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TallSeagrassLikeProperties {
    pub r#half: DoubleBlockHalf,
}
impl BlockProperties for TallSeagrassLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#half.to_index(), DoubleBlockHalf::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#half: {
                let value = index % DoubleBlockHalf::variant_count();
                index /= DoubleBlockHalf::variant_count();
                DoubleBlockHalf::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            137u16 | 557u16 | 558u16 | 559u16 | 560u16 | 561u16 | 562u16 | 664u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TallSeagrassLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TallSeagrassLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TallSeagrassLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("half", self.r#half.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            137u16 | 557u16 | 558u16 | 559u16 | 560u16 | 561u16 | 562u16 | 664u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TallSeagrassLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "half" {
                block_props.r#half = DoubleBlockHalf::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PitcherCropLikeProperties {
    pub r#age: u8,
    pub r#half: DoubleBlockHalf,
}
impl BlockProperties for PitcherCropLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#half.to_index(), DoubleBlockHalf::variant_count()),
            (self.r#age as u16, 5u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#half: {
                let value = index % DoubleBlockHalf::variant_count();
                index /= DoubleBlockHalf::variant_count();
                DoubleBlockHalf::from_index(value)
            },
            r#age: {
                let value = (index % 5u16) as u8;
                index /= 5u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 663u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PitcherCropLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PitcherCropLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PitcherCropLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "age",
                match self.r#age {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
            ("half", self.r#half.to_value()),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 663u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PitcherCropLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "age" => {
                    block_props.r#age = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 0u8,
                    }
                }
                "half" => block_props.r#half = DoubleBlockHalf::from_value(value),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrafterLikeProperties {
    pub r#crafting: bool,
    pub r#orientation: Orientation,
    pub r#triggered: bool,
}
impl BlockProperties for CrafterLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#triggered as u16, 2),
            (self.r#orientation.to_index(), Orientation::variant_count()),
            (!self.r#crafting as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#triggered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#orientation: {
                let value = index % Orientation::variant_count();
                index /= Orientation::variant_count();
                Orientation::from_index(value)
            },
            r#crafting: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1184u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CrafterLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CrafterLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CrafterLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("crafting", if self.r#crafting { "true" } else { "false" }),
            ("orientation", self.r#orientation.to_value()),
            ("triggered", if self.r#triggered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1184u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CrafterLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "crafting" => block_props.r#crafting = matches!(*value, "true"),
                "orientation" => block_props.r#orientation = Orientation::from_value(value),
                "triggered" => block_props.r#triggered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FarmlandLikeProperties {
    pub r#moisture: u8,
}
impl BlockProperties for FarmlandLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#moisture as u16, 8u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#moisture: {
                let value = (index % 8u16) as u8;
                index /= 8u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 208u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FarmlandLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "FarmlandLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FarmlandLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "moisture",
            match self.r#moisture {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 208u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "FarmlandLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "moisture" {
                block_props.r#moisture = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuspiciousSandLikeProperties {
    pub r#dusted: u8,
}
impl BlockProperties for SuspiciousSandLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#dusted as u16, 4u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#dusted: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 38u16 | 41u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SuspiciousSandLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SuspiciousSandLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SuspiciousSandLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "dusted",
            match self.r#dusted {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 38u16 | 41u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SuspiciousSandLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "dusted" {
                block_props.r#dusted = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResinBrickSlabLikeProperties {
    pub r#type: SlabType,
    pub r#waterlogged: bool,
}
impl BlockProperties for ResinBrickSlabLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#type.to_index(), SlabType::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#type: {
                let value = index % SlabType::variant_count();
                index /= SlabType::variant_count();
                SlabType::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            378u16
                | 533u16
                | 534u16
                | 535u16
                | 599u16
                | 600u16
                | 601u16
                | 602u16
                | 603u16
                | 604u16
                | 605u16
                | 606u16
                | 607u16
                | 608u16
                | 609u16
                | 610u16
                | 611u16
                | 612u16
                | 613u16
                | 614u16
                | 615u16
                | 616u16
                | 617u16
                | 618u16
                | 619u16
                | 620u16
                | 621u16
                | 622u16
                | 623u16
                | 811u16
                | 812u16
                | 813u16
                | 814u16
                | 815u16
                | 816u16
                | 817u16
                | 818u16
                | 819u16
                | 820u16
                | 821u16
                | 822u16
                | 823u16
                | 885u16
                | 886u16
                | 927u16
                | 932u16
                | 937u16
                | 985u16
                | 989u16
                | 994u16
                | 1000u16
                | 1004u16
                | 1008u16
                | 1013u16
                | 1017u16
                | 1021u16
                | 1068u16
                | 1069u16
                | 1070u16
                | 1071u16
                | 1072u16
                | 1073u16
                | 1074u16
                | 1075u16
                | 1154u16
                | 1158u16
                | 1162u16
                | 1166u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ResinBrickSlabLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ResinBrickSlabLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ResinBrickSlabLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("type", self.r#type.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            378u16
                | 533u16
                | 534u16
                | 535u16
                | 599u16
                | 600u16
                | 601u16
                | 602u16
                | 603u16
                | 604u16
                | 605u16
                | 606u16
                | 607u16
                | 608u16
                | 609u16
                | 610u16
                | 611u16
                | 612u16
                | 613u16
                | 614u16
                | 615u16
                | 616u16
                | 617u16
                | 618u16
                | 619u16
                | 620u16
                | 621u16
                | 622u16
                | 623u16
                | 811u16
                | 812u16
                | 813u16
                | 814u16
                | 815u16
                | 816u16
                | 817u16
                | 818u16
                | 819u16
                | 820u16
                | 821u16
                | 822u16
                | 823u16
                | 885u16
                | 886u16
                | 927u16
                | 932u16
                | 937u16
                | 985u16
                | 989u16
                | 994u16
                | 1000u16
                | 1004u16
                | 1008u16
                | 1013u16
                | 1017u16
                | 1021u16
                | 1068u16
                | 1069u16
                | 1070u16
                | 1071u16
                | 1072u16
                | 1073u16
                | 1074u16
                | 1075u16
                | 1154u16
                | 1158u16
                | 1162u16
                | 1166u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ResinBrickSlabLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "type" => block_props.r#type = SlabType::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StonePressurePlateLikeProperties {
    pub r#powered: bool,
}
impl BlockProperties for StonePressurePlateLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#powered as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            259u16
                | 261u16
                | 262u16
                | 263u16
                | 264u16
                | 265u16
                | 266u16
                | 267u16
                | 268u16
                | 269u16
                | 270u16
                | 887u16
                | 888u16
                | 938u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StonePressurePlateLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "StonePressurePlateLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StonePressurePlateLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("powered", if self.r#powered { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            259u16
                | 261u16
                | 262u16
                | 263u16
                | 264u16
                | 265u16
                | 266u16
                | 267u16
                | 268u16
                | 269u16
                | 270u16
                | 887u16
                | 888u16
                | 938u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "StonePressurePlateLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "powered" {
                block_props.r#powered = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopperBulbLikeProperties {
    pub r#lit: bool,
    pub r#powered: bool,
}
impl BlockProperties for CopperBulbLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#powered as u16, 2), (!self.r#lit as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#lit: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            1100u16 | 1101u16 | 1102u16 | 1103u16 | 1104u16 | 1105u16 | 1106u16 | 1107u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CopperBulbLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CopperBulbLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CopperBulbLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("lit", if self.r#lit { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            1100u16 | 1101u16 | 1102u16 | 1103u16 | 1104u16 | 1105u16 | 1106u16 | 1107u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CopperBulbLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "lit" => block_props.r#lit = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TripwireLikeProperties {
    pub r#attached: bool,
    pub r#disarmed: bool,
    pub r#east: bool,
    pub r#north: bool,
    pub r#powered: bool,
    pub r#south: bool,
    pub r#west: bool,
}
impl BlockProperties for TripwireLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#west as u16, 2),
            (!self.r#south as u16, 2),
            (!self.r#powered as u16, 2),
            (!self.r#north as u16, 2),
            (!self.r#east as u16, 2),
            (!self.r#disarmed as u16, 2),
            (!self.r#attached as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#north: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#east: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#disarmed: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#attached: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 402u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TripwireLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TripwireLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TripwireLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("attached", if self.r#attached { "true" } else { "false" }),
            ("disarmed", if self.r#disarmed { "true" } else { "false" }),
            ("east", if self.r#east { "true" } else { "false" }),
            ("north", if self.r#north { "true" } else { "false" }),
            ("powered", if self.r#powered { "true" } else { "false" }),
            ("south", if self.r#south { "true" } else { "false" }),
            ("west", if self.r#west { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 402u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TripwireLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "attached" => block_props.r#attached = matches!(*value, "true"),
                "disarmed" => block_props.r#disarmed = matches!(*value, "true"),
                "east" => block_props.r#east = matches!(*value, "true"),
                "north" => block_props.r#north = matches!(*value, "true"),
                "powered" => block_props.r#powered = matches!(*value, "true"),
                "south" => block_props.r#south = matches!(*value, "true"),
                "west" => block_props.r#west = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoweredRailLikeProperties {
    pub r#powered: bool,
    pub r#shape: RailShapeStraight,
    pub r#waterlogged: bool,
}
impl BlockProperties for PoweredRailLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#shape.to_index(), RailShapeStraight::variant_count()),
            (!self.r#powered as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#shape: {
                let value = index % RailShapeStraight::variant_count();
                index /= RailShapeStraight::variant_count();
                RailShapeStraight::from_index(value)
            },
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 126u16 | 127u16 | 482u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PoweredRailLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PoweredRailLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PoweredRailLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("powered", if self.r#powered { "true" } else { "false" }),
            ("shape", self.r#shape.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 126u16 | 127u16 | 482u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PoweredRailLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "powered" => block_props.r#powered = matches!(*value, "true"),
                "shape" => block_props.r#shape = RailShapeStraight::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkeletonSkullLikeProperties {
    pub r#powered: bool,
    pub r#rotation: u8,
}
impl BlockProperties for SkeletonSkullLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#rotation as u16, 16u16), (!self.r#powered as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#rotation: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
            r#powered: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            453u16 | 455u16 | 457u16 | 459u16 | 461u16 | 463u16 | 465u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SkeletonSkullLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SkeletonSkullLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SkeletonSkullLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("powered", if self.r#powered { "true" } else { "false" }),
            (
                "rotation",
                match self.r#rotation {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            453u16 | 455u16 | 457u16 | 459u16 | 461u16 | 463u16 | 465u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SkeletonSkullLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "powered" => block_props.r#powered = matches!(*value, "true"),
                "rotation" => {
                    block_props.r#rotation = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TestBlockLikeProperties {
    pub r#mode: TestBlockMode,
}
impl BlockProperties for TestBlockLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#mode.to_index(), TestBlockMode::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#mode: {
                let value = index % TestBlockMode::variant_count();
                index /= TestBlockMode::variant_count();
                TestBlockMode::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 907u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TestBlockLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TestBlockLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TestBlockLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("mode", self.r#mode.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 907u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TestBlockLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "mode" {
                block_props.r#mode = TestBlockMode::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComposterLikeProperties {
    pub r#level: u8,
}
impl BlockProperties for ComposterLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#level as u16, 9u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#level: {
                let value = (index % 9u16) as u8;
                index /= 9u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 909u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ComposterLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "ComposterLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ComposterLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "level",
            match self.r#level {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 909u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "ComposterLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "level" {
                block_props.r#level = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaveVinesPlantLikeProperties {
    pub r#berries: bool,
}
impl BlockProperties for CaveVinesPlantLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#berries as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#berries: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1136u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CaveVinesPlantLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CaveVinesPlantLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CaveVinesPlantLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("berries", if self.r#berries { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1136u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CaveVinesPlantLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "berries" {
                block_props.r#berries = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaveVinesLikeProperties {
    pub r#age: u8,
    pub r#berries: bool,
}
impl BlockProperties for CaveVinesLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#berries as u16, 2), (self.r#age as u16, 26u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#berries: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#age: {
                let value = (index % 26u16) as u8;
                index /= 26u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1135u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CaveVinesLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CaveVinesLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CaveVinesLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "age",
                match self.r#age {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    16u8 => "16",
                    17u8 => "17",
                    18u8 => "18",
                    19u8 => "19",
                    20u8 => "20",
                    21u8 => "21",
                    22u8 => "22",
                    23u8 => "23",
                    24u8 => "24",
                    25u8 => "25",
                    _ => unreachable!(),
                },
            ),
            ("berries", if self.r#berries { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1135u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CaveVinesLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "age" => {
                    block_props.r#age = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        "16" => 16u8,
                        "17" => 17u8,
                        "18" => 18u8,
                        "19" => 19u8,
                        "20" => 20u8,
                        "21" => 21u8,
                        "22" => 22u8,
                        "23" => 23u8,
                        "24" => 24u8,
                        "25" => 25u8,
                        _ => 0u8,
                    }
                }
                "berries" => block_props.r#berries = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TntLikeProperties {
    pub r#unstable: bool,
}
impl BlockProperties for TntLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#unstable as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#unstable: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 177u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TntLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TntLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TntLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("unstable", if self.r#unstable { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 177u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TntLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "unstable" {
                block_props.r#unstable = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrownMushroomBlockLikeProperties {
    pub r#down: bool,
    pub r#east: bool,
    pub r#north: bool,
    pub r#south: bool,
    pub r#up: bool,
    pub r#west: bool,
}
impl BlockProperties for BrownMushroomBlockLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#west as u16, 2),
            (!self.r#up as u16, 2),
            (!self.r#south as u16, 2),
            (!self.r#north as u16, 2),
            (!self.r#east as u16, 2),
            (!self.r#down as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#up: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#north: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#east: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#down: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 338u16 | 339u16 | 340u16 | 656u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BrownMushroomBlockLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BrownMushroomBlockLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BrownMushroomBlockLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("down", if self.r#down { "true" } else { "false" }),
            ("east", if self.r#east { "true" } else { "false" }),
            ("north", if self.r#north { "true" } else { "false" }),
            ("south", if self.r#south { "true" } else { "false" }),
            ("up", if self.r#up { "true" } else { "false" }),
            ("west", if self.r#west { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 338u16 | 339u16 | 340u16 | 656u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BrownMushroomBlockLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "down" => block_props.r#down = matches!(*value, "true"),
                "east" => block_props.r#east = matches!(*value, "true"),
                "north" => block_props.r#north = matches!(*value, "true"),
                "south" => block_props.r#south = matches!(*value, "true"),
                "up" => block_props.r#up = matches!(*value, "true"),
                "west" => block_props.r#west = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlowLichenLikeProperties {
    pub r#down: bool,
    pub r#east: bool,
    pub r#north: bool,
    pub r#south: bool,
    pub r#up: bool,
    pub r#waterlogged: bool,
    pub r#west: bool,
}
impl BlockProperties for GlowLichenLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#west as u16, 2),
            (!self.r#waterlogged as u16, 2),
            (!self.r#up as u16, 2),
            (!self.r#south as u16, 2),
            (!self.r#north as u16, 2),
            (!self.r#east as u16, 2),
            (!self.r#down as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#up: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#north: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#east: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#down: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 367u16 | 368u16 | 1031u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GlowLichenLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "GlowLichenLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GlowLichenLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("down", if self.r#down { "true" } else { "false" }),
            ("east", if self.r#east { "true" } else { "false" }),
            ("north", if self.r#north { "true" } else { "false" }),
            ("south", if self.r#south { "true" } else { "false" }),
            ("up", if self.r#up { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
            ("west", if self.r#west { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 367u16 | 368u16 | 1031u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GlowLichenLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "down" => block_props.r#down = matches!(*value, "true"),
                "east" => block_props.r#east = matches!(*value, "true"),
                "north" => block_props.r#north = matches!(*value, "true"),
                "south" => block_props.r#south = matches!(*value, "true"),
                "up" => block_props.r#up = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                "west" => block_props.r#west = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VineLikeProperties {
    pub r#east: bool,
    pub r#north: bool,
    pub r#south: bool,
    pub r#up: bool,
    pub r#west: bool,
}
impl BlockProperties for VineLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#west as u16, 2),
            (!self.r#up as u16, 2),
            (!self.r#south as u16, 2),
            (!self.r#north as u16, 2),
            (!self.r#east as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#up: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#north: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#east: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 366u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "VineLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "VineLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "VineLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("east", if self.r#east { "true" } else { "false" }),
            ("north", if self.r#north { "true" } else { "false" }),
            ("south", if self.r#south { "true" } else { "false" }),
            ("up", if self.r#up { "true" } else { "false" }),
            ("west", if self.r#west { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 366u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "VineLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "east" => block_props.r#east = matches!(*value, "true"),
                "north" => block_props.r#north = matches!(*value, "true"),
                "south" => block_props.r#south = matches!(*value, "true"),
                "up" => block_props.r#up = matches!(*value, "true"),
                "west" => block_props.r#west = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedstoneOreLikeProperties {
    pub r#lit: bool,
}
impl BlockProperties for RedstoneOreLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#lit as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#lit: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            271u16
                | 272u16
                | 273u16
                | 395u16
                | 961u16
                | 962u16
                | 963u16
                | 964u16
                | 965u16
                | 966u16
                | 967u16
                | 968u16
                | 969u16
                | 970u16
                | 971u16
                | 972u16
                | 973u16
                | 974u16
                | 975u16
                | 976u16
                | 977u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RedstoneOreLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "RedstoneOreLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RedstoneOreLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("lit", if self.r#lit { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            271u16
                | 272u16
                | 273u16
                | 395u16
                | 961u16
                | 962u16
                | 963u16
                | 964u16
                | 965u16
                | 966u16
                | 967u16
                | 968u16
                | 969u16
                | 970u16
                | 971u16
                | 972u16
                | 973u16
                | 974u16
                | 975u16
                | 976u16
                | 977u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RedstoneOreLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "lit" {
                block_props.r#lit = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandleLikeProperties {
    pub r#candles: u8,
    pub r#lit: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for CandleLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (!self.r#lit as u16, 2),
            ((self.r#candles - 1u8) as u16, 4u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#lit: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#candles: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            944u16
                | 945u16
                | 946u16
                | 947u16
                | 948u16
                | 949u16
                | 950u16
                | 951u16
                | 952u16
                | 953u16
                | 954u16
                | 955u16
                | 956u16
                | 957u16
                | 958u16
                | 959u16
                | 960u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CandleLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CandleLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CandleLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "candles",
                match self.r#candles {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
            ("lit", if self.r#lit { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            944u16
                | 945u16
                | 946u16
                | 947u16
                | 948u16
                | 949u16
                | 950u16
                | 951u16
                | 952u16
                | 953u16
                | 954u16
                | 955u16
                | 956u16
                | 957u16
                | 958u16
                | 959u16
                | 960u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CandleLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "candles" => {
                    block_props.r#candles = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 1u8,
                    }
                }
                "lit" => block_props.r#lit = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaleHangingMossLikeProperties {
    pub r#tip: bool,
}
impl BlockProperties for PaleHangingMossLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#tip as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#tip: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1190u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleHangingMossLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PaleHangingMossLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleHangingMossLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("tip", if self.r#tip { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1190u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleHangingMossLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "tip" {
                block_props.r#tip = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BubbleColumnLikeProperties {
    pub r#drag: bool,
}
impl BlockProperties for BubbleColumnLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#drag as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#drag: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 796u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BubbleColumnLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "BubbleColumnLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BubbleColumnLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("drag", if self.r#drag { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 796u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "BubbleColumnLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "drag" {
                block_props.r#drag = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakFenceLikeProperties {
    pub r#east: bool,
    pub r#north: bool,
    pub r#south: bool,
    pub r#waterlogged: bool,
    pub r#west: bool,
}
impl BlockProperties for OakFenceLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#west as u16, 2),
            (!self.r#waterlogged as u16, 2),
            (!self.r#south as u16, 2),
            (!self.r#north as u16, 2),
            (!self.r#east as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#west: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#south: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#north: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#east: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            284u16
                | 341u16
                | 342u16
                | 343u16
                | 344u16
                | 345u16
                | 346u16
                | 347u16
                | 348u16
                | 349u16
                | 359u16
                | 382u16
                | 500u16
                | 501u16
                | 502u16
                | 503u16
                | 504u16
                | 505u16
                | 506u16
                | 507u16
                | 508u16
                | 509u16
                | 510u16
                | 511u16
                | 512u16
                | 513u16
                | 514u16
                | 515u16
                | 637u16
                | 638u16
                | 639u16
                | 640u16
                | 641u16
                | 642u16
                | 643u16
                | 644u16
                | 645u16
                | 889u16
                | 890u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakFenceLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakFenceLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakFenceLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("east", if self.r#east { "true" } else { "false" }),
            ("north", if self.r#north { "true" } else { "false" }),
            ("south", if self.r#south { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
            ("west", if self.r#west { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            284u16
                | 341u16
                | 342u16
                | 343u16
                | 344u16
                | 345u16
                | 346u16
                | 347u16
                | 348u16
                | 349u16
                | 359u16
                | 382u16
                | 500u16
                | 501u16
                | 502u16
                | 503u16
                | 504u16
                | 505u16
                | 506u16
                | 507u16
                | 508u16
                | 509u16
                | 510u16
                | 511u16
                | 512u16
                | 513u16
                | 514u16
                | 515u16
                | 637u16
                | 638u16
                | 639u16
                | 640u16
                | 641u16
                | 642u16
                | 643u16
                | 644u16
                | 645u16
                | 889u16
                | 890u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakFenceLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "east" => block_props.r#east = matches!(*value, "true"),
                "north" => block_props.r#north = matches!(*value, "true"),
                "south" => block_props.r#south = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                "west" => block_props.r#west = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SculkCatalystLikeProperties {
    pub r#bloom: bool,
}
impl BlockProperties for SculkCatalystLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#bloom as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#bloom: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1032u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkCatalystLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SculkCatalystLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkCatalystLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("bloom", if self.r#bloom { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1032u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SculkCatalystLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "bloom" {
                block_props.r#bloom = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrassBlockLikeProperties {
    pub r#snowy: bool,
}
impl BlockProperties for GrassBlockLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#snowy as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#snowy: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 8u16 | 11u16 | 373u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GrassBlockLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "GrassBlockLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GrassBlockLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("snowy", if self.r#snowy { "true" } else { "false" })]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 8u16 | 11u16 | 373u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "GrassBlockLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "snowy" {
                block_props.r#snowy = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakLeavesLikeProperties {
    pub r#distance: u8,
    pub r#persistent: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for OakLeavesLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (!self.r#persistent as u16, 2),
            ((self.r#distance - 1u8) as u16, 7u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#persistent: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#distance: {
                let value = (index % 7u16) as u8;
                index /= 7u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            88u16 | 89u16 | 90u16 | 91u16 | 92u16 | 93u16 | 94u16 | 95u16 | 96u16 | 97u16 | 98u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakLeavesLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakLeavesLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakLeavesLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "distance",
                match self.r#distance {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    _ => unreachable!(),
                },
            ),
            (
                "persistent",
                if self.r#persistent { "true" } else { "false" },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            88u16 | 89u16 | 90u16 | 91u16 | 92u16 | 93u16 | 94u16 | 95u16 | 96u16 | 97u16 | 98u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakLeavesLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "distance" => {
                    block_props.r#distance = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        _ => 1u8,
                    }
                }
                "persistent" => block_props.r#persistent = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnifferEggLikeProperties {
    pub r#hatch: u8,
}
impl BlockProperties for SnifferEggLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#hatch as u16, 3u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#hatch: {
                let value = (index % 3u16) as u8;
                index /= 3u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 746u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SnifferEggLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SnifferEggLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SnifferEggLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "hatch",
            match self.r#hatch {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 746u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SnifferEggLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "hatch" {
                block_props.r#hatch = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TurtleEggLikeProperties {
    pub r#eggs: u8,
    pub r#hatch: u8,
}
impl BlockProperties for TurtleEggLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (self.r#hatch as u16, 3u16),
            ((self.r#eggs - 1u8) as u16, 4u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#hatch: {
                let value = (index % 3u16) as u8;
                index /= 3u16;
                value
            },
            r#eggs: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 745u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TurtleEggLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TurtleEggLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TurtleEggLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "eggs",
                match self.r#eggs {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
            (
                "hatch",
                match self.r#hatch {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 745u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TurtleEggLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "eggs" => {
                    block_props.r#eggs = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 1u8,
                    }
                }
                "hatch" => {
                    block_props.r#hatch = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        _ => 0u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakHangingSignLikeProperties {
    pub r#attached: bool,
    pub r#rotation: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for OakHangingSignLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#rotation as u16, 16u16),
            (!self.r#attached as u16, 2),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#rotation: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
            r#attached: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            234u16
                | 235u16
                | 236u16
                | 237u16
                | 238u16
                | 239u16
                | 240u16
                | 241u16
                | 242u16
                | 243u16
                | 244u16
                | 245u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakHangingSignLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakHangingSignLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakHangingSignLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("attached", if self.r#attached { "true" } else { "false" }),
            (
                "rotation",
                match self.r#rotation {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            234u16
                | 235u16
                | 236u16
                | 237u16
                | 238u16
                | 239u16
                | 240u16
                | 241u16
                | 242u16
                | 243u16
                | 244u16
                | 245u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakHangingSignLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "attached" => block_props.r#attached = matches!(*value, "true"),
                "rotation" => {
                    block_props.r#rotation = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DaylightDetectorLikeProperties {
    pub r#inverted: bool,
    pub r#power: u8,
}
impl BlockProperties for DaylightDetectorLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#power as u16, 16u16), (!self.r#inverted as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#power: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
            r#inverted: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 474u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DaylightDetectorLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "DaylightDetectorLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DaylightDetectorLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("inverted", if self.r#inverted { "true" } else { "false" }),
            (
                "power",
                match self.r#power {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 474u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "DaylightDetectorLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "inverted" => block_props.r#inverted = matches!(*value, "true"),
                "power" => {
                    block_props.r#power = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterCauldronLikeProperties {
    pub r#level: u8,
}
impl BlockProperties for WaterCauldronLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [((self.r#level - 1u8) as u16, 3u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#level: {
                let value = (index % 3u16) as u8;
                index /= 3u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 388u16 | 390u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WaterCauldronLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "WaterCauldronLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WaterCauldronLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "level",
            match self.r#level {
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 388u16 | 390u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WaterCauldronLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "level" {
                block_props.r#level = match *value {
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    _ => 1u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MangrovePropaguleLikeProperties {
    pub r#age: u8,
    pub r#hanging: bool,
    pub r#stage: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for MangrovePropaguleLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#stage as u16, 2u16),
            (!self.r#hanging as u16, 2),
            (self.r#age as u16, 5u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#stage: {
                let value = (index % 2u16) as u8;
                index /= 2u16;
                value
            },
            r#hanging: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#age: {
                let value = (index % 5u16) as u8;
                index /= 5u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 33u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MangrovePropaguleLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "MangrovePropaguleLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MangrovePropaguleLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "age",
                match self.r#age {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
            ("hanging", if self.r#hanging { "true" } else { "false" }),
            (
                "stage",
                match self.r#stage {
                    0u8 => "0",
                    1u8 => "1",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 33u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MangrovePropaguleLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "age" => {
                    block_props.r#age = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 0u8,
                    }
                }
                "hanging" => block_props.r#hanging = matches!(*value, "true"),
                "stage" => {
                    block_props.r#stage = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        _ => 0u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LanternLikeProperties {
    pub r#hanging: bool,
    pub r#waterlogged: bool,
}
impl BlockProperties for LanternLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#waterlogged as u16, 2), (!self.r#hanging as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#hanging: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            849u16 | 850u16 | 851u16 | 852u16 | 853u16 | 854u16 | 855u16 | 856u16 | 857u16 | 858u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LanternLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LanternLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LanternLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("hanging", if self.r#hanging { "true" } else { "false" }),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            849u16 | 850u16 | 851u16 | 852u16 | 853u16 | 854u16 | 855u16 | 856u16 | 857u16 | 858u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LanternLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "hanging" => block_props.r#hanging = matches!(*value, "true"),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakSaplingLikeProperties {
    pub r#stage: u8,
}
impl BlockProperties for OakSaplingLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#stage as u16, 2u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#stage: {
                let value = (index % 2u16) as u8;
                index /= 2u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            25u16 | 26u16 | 27u16 | 28u16 | 29u16 | 30u16 | 31u16 | 32u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakSaplingLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakSaplingLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakSaplingLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "stage",
            match self.r#stage {
                0u8 => "0",
                1u8 => "1",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            25u16 | 26u16 | 27u16 | 28u16 | 29u16 | 30u16 | 31u16 | 32u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakSaplingLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "stage" {
                block_props.r#stage = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WheatLikeProperties {
    pub r#age: u8,
}
impl BlockProperties for WheatLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#age as u16, 8u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#age: {
                let value = (index % 8u16) as u8;
                index /= 8u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            207u16 | 364u16 | 365u16 | 441u16 | 442u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WheatLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "WheatLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WheatLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "age",
            match self.r#age {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            207u16 | 364u16 | 365u16 | 441u16 | 442u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WheatLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "age" {
                block_props.r#age = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JigsawLikeProperties {
    pub r#orientation: Orientation,
}
impl BlockProperties for JigsawLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#orientation.to_index(), Orientation::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#orientation: {
                let value = index % Orientation::variant_count();
                index /= Orientation::variant_count();
                Orientation::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 906u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "JigsawLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "JigsawLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "JigsawLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("orientation", self.r#orientation.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 906u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "JigsawLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "orientation" {
                block_props.r#orientation = Orientation::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointedDripstoneLikeProperties {
    pub r#thickness: SpeleothemThickness,
    pub r#vertical_direction: VerticalDirection,
    pub r#waterlogged: bool,
}
impl BlockProperties for PointedDripstoneLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (
                self.r#vertical_direction.to_index(),
                VerticalDirection::variant_count(),
            ),
            (
                self.r#thickness.to_index(),
                SpeleothemThickness::variant_count(),
            ),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#vertical_direction: {
                let value = index % VerticalDirection::variant_count();
                index /= VerticalDirection::variant_count();
                VerticalDirection::from_index(value)
            },
            r#thickness: {
                let value = index % SpeleothemThickness::variant_count();
                index /= SpeleothemThickness::variant_count();
                SpeleothemThickness::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 1133u16 | 1134u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PointedDripstoneLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PointedDripstoneLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PointedDripstoneLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("thickness", self.r#thickness.to_value()),
            ("vertical_direction", self.r#vertical_direction.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 1133u16 | 1134u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PointedDripstoneLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "thickness" => block_props.r#thickness = SpeleothemThickness::from_value(value),
                "vertical_direction" => {
                    block_props.r#vertical_direction = VerticalDirection::from_value(value)
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeaPickleLikeProperties {
    pub r#pickles: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for SeaPickleLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            ((self.r#pickles - 1u8) as u16, 4u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#pickles: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value + 1u8
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 788u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SeaPickleLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "SeaPickleLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SeaPickleLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "pickles",
                match self.r#pickles {
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 788u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "SeaPickleLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "pickles" => {
                    block_props.r#pickles = match *value {
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        _ => 1u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MangroveRootsLikeProperties {
    pub r#waterlogged: bool,
}
impl BlockProperties for MangroveRootsLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(!self.r#waterlogged as u16, 2)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            58u16
                | 524u16
                | 758u16
                | 759u16
                | 760u16
                | 761u16
                | 762u16
                | 763u16
                | 764u16
                | 765u16
                | 766u16
                | 767u16
                | 768u16
                | 769u16
                | 770u16
                | 771u16
                | 772u16
                | 773u16
                | 774u16
                | 775u16
                | 776u16
                | 777u16
                | 790u16
                | 1092u16
                | 1093u16
                | 1094u16
                | 1095u16
                | 1096u16
                | 1097u16
                | 1098u16
                | 1099u16
                | 1148u16
                | 1187u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MangroveRootsLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "MangroveRootsLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MangroveRootsLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "waterlogged",
            if self.r#waterlogged { "true" } else { "false" },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            58u16
                | 524u16
                | 758u16
                | 759u16
                | 760u16
                | 761u16
                | 762u16
                | 763u16
                | 764u16
                | 765u16
                | 766u16
                | 767u16
                | 768u16
                | 769u16
                | 770u16
                | 771u16
                | 772u16
                | 773u16
                | 774u16
                | 775u16
                | 776u16
                | 777u16
                | 790u16
                | 1092u16
                | 1093u16
                | 1094u16
                | 1095u16
                | 1096u16
                | 1097u16
                | 1098u16
                | 1099u16
                | 1148u16
                | 1187u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "MangroveRootsLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "waterlogged" {
                block_props.r#waterlogged = matches!(*value, "true")
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RailLikeProperties {
    pub r#shape: RailShape,
    pub r#waterlogged: bool,
}
impl BlockProperties for RailLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#shape.to_index(), RailShape::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#shape: {
                let value = index % RailShape::variant_count();
                index /= RailShape::variant_count();
                RailShape::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 222u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RailLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "RailLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RailLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("shape", self.r#shape.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 222u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "RailLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "shape" => block_props.r#shape = RailShape::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IronChainLikeProperties {
    pub r#axis: Axis,
    pub r#waterlogged: bool,
}
impl BlockProperties for IronChainLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#axis.to_index(), Axis::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#axis: {
                let value = index % Axis::variant_count();
                index /= Axis::variant_count();
                Axis::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            350u16 | 351u16 | 352u16 | 353u16 | 354u16 | 355u16 | 356u16 | 357u16 | 358u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "IronChainLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "IronChainLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "IronChainLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("axis", self.r#axis.to_value()),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            350u16 | 351u16 | 352u16 | 353u16 | 354u16 | 355u16 | 356u16 | 357u16 | 358u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "IronChainLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "axis" => block_props.r#axis = Axis::from_value(value),
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OakSignLikeProperties {
    pub r#rotation: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for OakSignLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#rotation as u16, 16u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#rotation: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            210u16
                | 211u16
                | 212u16
                | 213u16
                | 214u16
                | 215u16
                | 216u16
                | 217u16
                | 218u16
                | 219u16
                | 901u16
                | 902u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakSignLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "OakSignLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakSignLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "rotation",
                match self.r#rotation {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            210u16
                | 211u16
                | 212u16
                | 213u16
                | 214u16
                | 215u16
                | 216u16
                | 217u16
                | 218u16
                | 219u16
                | 901u16
                | 902u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "OakSignLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "rotation" => {
                    block_props.r#rotation = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightLikeProperties {
    pub r#level: u8,
    pub r#waterlogged: bool,
}
impl BlockProperties for LightLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#waterlogged as u16, 2),
            (self.r#level as u16, 16u16),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#waterlogged: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#level: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 525u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LightLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "level",
                match self.r#level {
                    0u8 => "0",
                    1u8 => "1",
                    2u8 => "2",
                    3u8 => "3",
                    4u8 => "4",
                    5u8 => "5",
                    6u8 => "6",
                    7u8 => "7",
                    8u8 => "8",
                    9u8 => "9",
                    10u8 => "10",
                    11u8 => "11",
                    12u8 => "12",
                    13u8 => "13",
                    14u8 => "14",
                    15u8 => "15",
                    _ => unreachable!(),
                },
            ),
            (
                "waterlogged",
                if self.r#waterlogged { "true" } else { "false" },
            ),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 525u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "level" => {
                    block_props.r#level = match *value {
                        "0" => 0u8,
                        "1" => 1u8,
                        "2" => 2u8,
                        "3" => 3u8,
                        "4" => 4u8,
                        "5" => 5u8,
                        "6" => 6u8,
                        "7" => 7u8,
                        "8" => 8u8,
                        "9" => 9u8,
                        "10" => 10u8,
                        "11" => 11u8,
                        "12" => 12u8,
                        "13" => 13u8,
                        "14" => 14u8,
                        "15" => 15u8,
                        _ => 0u8,
                    }
                }
                "waterlogged" => block_props.r#waterlogged = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaleOakWoodLikeProperties {
    pub r#axis: Axis,
}
impl BlockProperties for PaleOakWoodLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#axis.to_index(), Axis::variant_count())]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#axis: {
                let value = index % Axis::variant_count();
                index /= Axis::variant_count();
                Axis::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            20u16
                | 49u16
                | 50u16
                | 51u16
                | 52u16
                | 53u16
                | 54u16
                | 55u16
                | 56u16
                | 57u16
                | 59u16
                | 60u16
                | 61u16
                | 62u16
                | 63u16
                | 64u16
                | 65u16
                | 66u16
                | 67u16
                | 68u16
                | 69u16
                | 70u16
                | 71u16
                | 72u16
                | 73u16
                | 74u16
                | 75u16
                | 76u16
                | 77u16
                | 78u16
                | 79u16
                | 80u16
                | 81u16
                | 82u16
                | 83u16
                | 84u16
                | 85u16
                | 86u16
                | 87u16
                | 288u16
                | 289u16
                | 480u16
                | 537u16
                | 659u16
                | 674u16
                | 862u16
                | 863u16
                | 864u16
                | 865u16
                | 871u16
                | 872u16
                | 873u16
                | 874u16
                | 1151u16
                | 1171u16
                | 1178u16
                | 1179u16
                | 1180u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleOakWoodLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "PaleOakWoodLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleOakWoodLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![("axis", self.r#axis.to_value())]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            20u16
                | 49u16
                | 50u16
                | 51u16
                | 52u16
                | 53u16
                | 54u16
                | 55u16
                | 56u16
                | 57u16
                | 59u16
                | 60u16
                | 61u16
                | 62u16
                | 63u16
                | 64u16
                | 65u16
                | 66u16
                | 67u16
                | 68u16
                | 69u16
                | 70u16
                | 71u16
                | 72u16
                | 73u16
                | 74u16
                | 75u16
                | 76u16
                | 77u16
                | 78u16
                | 79u16
                | 80u16
                | 81u16
                | 82u16
                | 83u16
                | 84u16
                | 85u16
                | 86u16
                | 87u16
                | 288u16
                | 289u16
                | 480u16
                | 537u16
                | 659u16
                | 674u16
                | 862u16
                | 863u16
                | 864u16
                | 865u16
                | 871u16
                | 872u16
                | 873u16
                | 874u16
                | 1151u16
                | 1171u16
                | 1178u16
                | 1179u16
                | 1180u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "PaleOakWoodLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "axis" {
                block_props.r#axis = Axis::from_value(value)
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CreakingHeartLikeProperties {
    pub r#axis: Axis,
    pub r#creaking_heart_state: CreakingHeartState,
    pub r#natural: bool,
}
impl BlockProperties for CreakingHeartLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [
            (!self.r#natural as u16, 2),
            (
                self.r#creaking_heart_state.to_index(),
                CreakingHeartState::variant_count(),
            ),
            (self.r#axis.to_index(), Axis::variant_count()),
        ]
        .iter()
        .fold((0, 1), |(curr, mul), &(val, count)| {
            (curr + val * mul, mul * count)
        });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#natural: {
                let value = index % 2;
                index /= 2;
                value == 0
            },
            r#creaking_heart_state: {
                let value = index % CreakingHeartState::variant_count();
                index /= CreakingHeartState::variant_count();
                CreakingHeartState::from_index(value)
            },
            r#axis: {
                let value = index % Axis::variant_count();
                index /= Axis::variant_count();
                Axis::from_index(value)
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 199u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CreakingHeartLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "CreakingHeartLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CreakingHeartLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("axis", self.r#axis.to_value()),
            (
                "creaking_heart_state",
                self.r#creaking_heart_state.to_value(),
            ),
            ("natural", if self.r#natural { "true" } else { "false" }),
        ]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 199u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "CreakingHeartLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            match *key {
                "axis" => block_props.r#axis = Axis::from_value(value),
                "creaking_heart_state" => {
                    block_props.r#creaking_heart_state = CreakingHeartState::from_value(value)
                }
                "natural" => block_props.r#natural = matches!(*value, "true"),
                _ => {}
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WhiteBannerLikeProperties {
    pub r#rotation: u8,
}
impl BlockProperties for WhiteBannerLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#rotation as u16, 16u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#rotation: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(
            block_id.as_u16(),
            563u16
                | 564u16
                | 565u16
                | 566u16
                | 567u16
                | 568u16
                | 569u16
                | 570u16
                | 571u16
                | 572u16
                | 573u16
                | 574u16
                | 575u16
                | 576u16
                | 577u16
                | 578u16
        )
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WhiteBannerLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "WhiteBannerLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WhiteBannerLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "rotation",
            match self.r#rotation {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                9u8 => "9",
                10u8 => "10",
                11u8 => "11",
                12u8 => "12",
                13u8 => "13",
                14u8 => "14",
                15u8 => "15",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(
            block.id.as_u16(),
            563u16
                | 564u16
                | 565u16
                | 566u16
                | 567u16
                | 568u16
                | 569u16
                | 570u16
                | 571u16
                | 572u16
                | 573u16
                | 574u16
                | 575u16
                | 576u16
                | 577u16
                | 578u16
        ) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WhiteBannerLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "rotation" {
                block_props.r#rotation = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    "9" => 9u8,
                    "10" => 10u8,
                    "11" => 11u8,
                    "12" => 12u8,
                    "13" => 13u8,
                    "14" => 14u8,
                    "15" => 15u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterLikeProperties {
    pub r#level: u8,
}
impl BlockProperties for WaterLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#level as u16, 16u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#level: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 35u16 | 36u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WaterLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "WaterLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WaterLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "level",
            match self.r#level {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                9u8 => "9",
                10u8 => "10",
                11u8 => "11",
                12u8 => "12",
                13u8 => "13",
                14u8 => "14",
                15u8 => "15",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 35u16 | 36u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "WaterLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "level" {
                block_props.r#level = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    "9" => 9u8,
                    "10" => 10u8,
                    "11" => 11u8,
                    "12" => 12u8,
                    "13" => 13u8,
                    "14" => 14u8,
                    "15" => 15u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KelpLikeProperties {
    pub r#age: u8,
}
impl BlockProperties for KelpLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#age as u16, 26u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#age: {
                let value = (index % 26u16) as u8;
                index /= 26u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 742u16 | 878u16 | 880u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "KelpLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "KelpLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "KelpLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "age",
            match self.r#age {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                9u8 => "9",
                10u8 => "10",
                11u8 => "11",
                12u8 => "12",
                13u8 => "13",
                14u8 => "14",
                15u8 => "15",
                16u8 => "16",
                17u8 => "17",
                18u8 => "18",
                19u8 => "19",
                20u8 => "20",
                21u8 => "21",
                22u8 => "22",
                23u8 => "23",
                24u8 => "24",
                25u8 => "25",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 742u16 | 878u16 | 880u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "KelpLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "age" {
                block_props.r#age = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    "9" => 9u8,
                    "10" => 10u8,
                    "11" => 11u8,
                    "12" => 12u8,
                    "13" => 13u8,
                    "14" => 14u8,
                    "15" => 15u8,
                    "16" => 16u8,
                    "17" => 17u8,
                    "18" => 18u8,
                    "19" => 19u8,
                    "20" => 20u8,
                    "21" => 21u8,
                    "22" => 22u8,
                    "23" => 23u8,
                    "24" => 24u8,
                    "25" => 25u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightWeightedPressurePlateLikeProperties {
    pub r#power: u8,
}
impl BlockProperties for LightWeightedPressurePlateLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#power as u16, 16u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#power: {
                let value = (index % 16u16) as u8;
                index /= 16u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 471u16 | 472u16 | 910u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightWeightedPressurePlateLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "LightWeightedPressurePlateLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightWeightedPressurePlateLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "power",
            match self.r#power {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                4u8 => "4",
                5u8 => "5",
                6u8 => "6",
                7u8 => "7",
                8u8 => "8",
                9u8 => "9",
                10u8 => "10",
                11u8 => "11",
                12u8 => "12",
                13u8 => "13",
                14u8 => "14",
                15u8 => "15",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 471u16 | 472u16 | 910u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "LightWeightedPressurePlateLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "power" {
                block_props.r#power = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    "4" => 4u8,
                    "5" => 5u8,
                    "6" => 6u8,
                    "7" => 7u8,
                    "8" => 8u8,
                    "9" => 9u8,
                    "10" => 10u8,
                    "11" => 11u8,
                    "12" => 12u8,
                    "13" => 13u8,
                    "14" => 14u8,
                    "15" => 15u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TorchflowerCropLikeProperties {
    pub r#age: u8,
}
impl BlockProperties for TorchflowerCropLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#age as u16, 2u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#age: {
                let value = (index % 2u16) as u8;
                index /= 2u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 662u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TorchflowerCropLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "TorchflowerCropLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TorchflowerCropLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "age",
            match self.r#age {
                0u8 => "0",
                1u8 => "1",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 662u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "TorchflowerCropLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "age" {
                block_props.r#age = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetherWartLikeProperties {
    pub r#age: u8,
}
impl BlockProperties for NetherWartLikeProperties {
    fn to_index(&self) -> u16 {
        let (index, _) = [(self.r#age as u16, 4u16)]
            .iter()
            .fold((0, 1), |(curr, mul), &(val, count)| {
                (curr + val * mul, mul * count)
            });
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#age: {
                let value = (index % 4u16) as u8;
                index /= 4u16;
                value
            },
        }
    }
    #[inline]
    #[allow(clippy::manual_range_patterns)]
    fn handles_block_id(block_id: BlockId) -> bool
    where
        Self: Sized,
    {
        matches!(block_id.as_u16(), 384u16 | 665u16 | 670u16 | 861u16)
    }
    fn to_state_id(&self, block: &Block) -> BlockStateId {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NetherWartLikeProperties"
            );
        }
        block.states[self.to_index() as usize].id
    }
    fn from_state_id(id: BlockStateId, block: &Block) -> Self {
        debug_assert!(
            Self::handles_block_id(block.id),
            "{} is not a valid block for {}",
            block.name,
            "NetherWartLikeProperties"
        );
        let min_id = block.states[0].id.as_u16();
        let max_id = block.states.last().map(|s| s.id.as_u16()).unwrap_or(min_id);
        if (min_id..=max_id).contains(&id.as_u16()) {
            Self::from_index(id.as_u16() - min_id)
        } else {
            #[cfg(debug_assertions)]
            panic!("State ID {} does not exist for {}", id, block.name);
            #[cfg(not(debug_assertions))]
            Self::from_index(0)
        }
    }
    fn default(block: &Block) -> Self {
        if !Self::handles_block_id(block.id) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NetherWartLikeProperties"
            );
        }
        Self::from_state_id(block.default_state.id, block)
    }
    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        vec![(
            "age",
            match self.r#age {
                0u8 => "0",
                1u8 => "1",
                2u8 => "2",
                3u8 => "3",
                _ => unreachable!(),
            },
        )]
    }
    #[allow(clippy::manual_range_patterns)]
    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        #[cfg(debug_assertions)]
        if !matches!(block.id.as_u16(), 384u16 | 665u16 | 670u16 | 861u16) {
            panic!(
                "{} is not a valid block for {}",
                block.name, "NetherWartLikeProperties"
            );
        }
        let mut block_props = Self::default(block);
        for (key, value) in props {
            if *key == "age" {
                block_props.r#age = match *value {
                    "0" => 0u8,
                    "1" => 1u8,
                    "2" => 2u8,
                    "3" => 3u8,
                    _ => 0u8,
                }
            }
        }
        block_props
    }
}
impl Facing {
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}
impl HorizontalFacing {
    #[must_use]
    pub fn all() -> [Self; 4] {
        [Self::North, Self::South, Self::West, Self::East]
    }
    #[must_use]
    pub fn to_offset(&self) -> Vector3<i32> {
        match self {
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
        .into()
    }
    #[must_use]
    pub const fn to_axis(&self) -> HorizontalAxis {
        match self {
            Self::North | Self::South => HorizontalAxis::Z,
            Self::West | Self::East => HorizontalAxis::X,
        }
    }
    #[must_use]
    pub const fn to_facing(&self) -> HorizontalFacing {
        match self {
            Self::North => HorizontalFacing::North,
            Self::South => HorizontalFacing::South,
            Self::West => HorizontalFacing::West,
            Self::East => HorizontalFacing::East,
        }
    }
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
    #[must_use]
    pub const fn rotate_clockwise(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::East => Self::South,
        }
    }
    #[must_use]
    pub const fn rotate_counter_clockwise(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::South => Self::East,
            Self::West => Self::South,
            Self::East => Self::North,
        }
    }
}
impl RailShape {
    #[must_use]
    pub const fn is_ascending(&self) -> bool {
        matches!(
            self,
            Self::AscendingEast | Self::AscendingWest | Self::AscendingNorth | Self::AscendingSouth
        )
    }
}
impl RailShapeStraight {
    #[must_use]
    pub const fn is_ascending(&self) -> bool {
        matches!(
            self,
            Self::AscendingEast | Self::AscendingWest | Self::AscendingNorth | Self::AscendingSouth
        )
    }
}

use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use std::fs;

use crate::placed_feature::{
    value_to_block_direction, value_to_block_predicate, value_to_block_state_codec,
    value_to_height_provider, value_to_int_provider, value_to_y_offset,
};

pub fn build() -> TokenStream {
    let json_content = fs::read_to_string("../assets/configured_features.json")
        .expect("Failed to read configured_features.json");
    let json: Value =
        serde_json::from_str(&json_content).expect("Failed to parse configured_features.json");

    let entries: Vec<TokenStream> = json
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, value)| {
            let cf = value_to_configured_feature(value);
            quote! {
                map.insert(#name.to_string(), #cf);
            }
        })
        .collect();

    quote! {
        #[allow(clippy::all, unused_imports, dead_code)]
        fn build_configured_features() -> std::collections::HashMap<String, ConfiguredFeature> {
            use crate::generation::block_predicate::{
                AllOfBlockPredicate, AnyOfBlockPredicate, BlockPredicate,
                HasSturdyFacePredicate, InsideWorldBoundsBlockPredicate,
                MatchingBlockTagPredicate, MatchingBlocksBlockPredicate, MatchingBlocksWrapper,
                MatchingFluidsBlockPredicate, NotBlockPredicate, OffsetBlocksBlockPredicate,
                ReplaceableBlockPredicate, SolidBlockPredicate, WouldSurviveBlockPredicate,
            };
            use crate::generation::height_provider::{
                HeightProvider, TrapezoidHeightProvider, UniformHeightProvider,
                VeryBiasedToBottomHeightProvider,
            };
            use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
            use pumpkin_util::math::int_provider::{
                BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider,
                ConstantIntProvider, IntProvider, NormalIntProvider, UniformIntProvider,
                WeightedEntry, WeightedListIntProvider,
            };
            use crate::block::BlockStateCodec;
            use crate::generation::block_state_provider::{
                BlockStateProvider, DualNoiseBlockStateProvider, NoiseBlockStateProvider,
                NoiseBlockStateProviderBase, NoiseThresholdBlockStateProvider, PillarBlockStateProvider,
                RandomizedIntBlockStateProvider, SimpleStateProvider, WeightedBlockStateProvider,
            };
            use pumpkin_util::math::pool::Weighted;
            use pumpkin_util::DoublePerlinNoiseParametersCodec;
            use crate::generation::rule::{
                RuleTest,
                block_match::BlockMatchRuleTest,
                block_state_match::BlockStateMatchRuleTest,
                tag_match::TagMatchRuleTest,
                random_block_match::RandomBlockMatchRuleTest,
                random_block_state_match::RandomBlockStateMatchRuleTest,
            };
            use crate::generation::feature::placed_features::{
                Feature, PlacedFeature, PlacementModifier,
                BiomePlacementModifier, BlockFilterPlacementModifier,
                CountOnEveryLayerPlacementModifier, CountPlacementModifier,
                EnvironmentScanPlacementModifier, HeightRangePlacementModifier,
                HeightmapPlacementModifier, NoiseBasedCountPlacementModifier,
                NoiseThresholdCountPlacementModifier, RandomOffsetPlacementModifier,
                RarityFilterPlacementModifier, SquarePlacementModifier,
                SurfaceThresholdFilterPlacementModifier, SurfaceWaterDepthFilterPlacementModifier,
                PlacedFeatureWrapper,
            };
            use crate::generation::feature::features::{
                bamboo::BambooFeature,
                block_column::{BlockColumnFeature, Layer},
                end_spike::{EndSpikeFeature, Spike},
                fallen_tree::FallenTreeFeature,
                nether_forest_vegetation::NetherForestVegetationFeature,
                netherrack_replace_blobs::ReplaceBlobsFeature,
                ore::{OreFeature, OreTarget},
                random_boolean_selector::RandomBooleanFeature,
                random_patch::RandomPatchFeature,
                random_selector::{RandomFeature, RandomFeatureEntry},
                sea_pickle::SeaPickleFeature,
                seagrass::SeagrassFeature,
                simple_block::SimpleBlockFeature,
                simple_random_selector::SimpleRandomFeature,
                spring_feature::{BlockWrapper, SpringFeatureFeature},
                geode::GeodeFeature,
                tree::TreeFeature,
                tree::trunk::{TrunkPlacer, TrunkType,
                    bending::BendingTrunkPlacer,
                    cherry::CherryTrunkPlacer,
                    dark_oak::DarkOakTrunkPlacer,
                    fancy::FancyTrunkPlacer,
                    forking::ForkingTrunkPlacer,
                    giant::GiantTrunkPlacer,
                    mega_jungle::MegaJungleTrunkPlacer,
                    straight::StraightTrunkPlacer,
                    upwards_branching::UpwardsBranchingTrunkPlacer,
                },
                tree::foliage::{FoliagePlacer, FoliageType,
                    acacia::AcaciaFoliagePlacer,
                    blob::BlobFoliagePlacer,
                    bush::BushFoliagePlacer,
                    cherry::CherryFoliagePlacer,
                    dark_oak::DarkOakFoliagePlacer,
                    fancy::LargeOakFoliagePlacer,
                    jungle::JungleFoliagePlacer,
                    mega_pine::MegaPineFoliagePlacer,
                    pine::PineFoliagePlacer,
                    random_spread::RandomSpreadFoliagePlacer,
                    spruce::SpruceFoliagePlacer,
                },
                tree::decorator::{
                    TreeDecorator,
                    alter_ground::AlterGroundTreeDecorator,
                    attached_to_leaves::AttachedToLeavesTreeDecorator,
                    attached_to_logs::AttachedToLogsTreeDecorator,
                    beehive::BeehiveTreeDecorator,
                    cocoa::CocoaTreeDecorator,
                    creaking_heart::CreakingHeartTreeDecorator,
                    leave_vine::LeavesVineTreeDecorator,
                    pale_moss::PaleMossTreeDecorator,
                    place_on_ground::PlaceOnGroundTreeDecorator,
                    trunk_vine::TrunkVineTreeDecorator,
                },
            };
            use crate::generation::feature::size::{FeatureSize, FeatureSizeType, ThreeLayersFeatureSize, TwoLayersFeatureSize};
            use pumpkin_data::{Block, BlockDirection};
            use pumpkin_util::math::vector3::Vector3;
            use pumpkin_util::HeightMap;
            use crate::generation::feature::features::drip_stone::small::SmallDripstoneFeature;
            let mut map = std::collections::HashMap::new();
            #(#entries)*
            map
        }
    }
}

pub fn value_to_configured_feature(v: &Value) -> TokenStream {
    let type_str = v["type"].as_str().unwrap_or("");
    let config = &v["config"];
    match type_str {
        "minecraft:no_op" => quote! { ConfiguredFeature::NoOp },
        "minecraft:bamboo" => {
            let prob = config["probability"].as_f64().unwrap_or(0.0) as f32;
            quote! { ConfiguredFeature::Bamboo(BambooFeature { probability: #prob }) }
        }
        "minecraft:seagrass" => {
            let prob = config["probability"].as_f64().unwrap_or(0.0) as f32;
            quote! { ConfiguredFeature::Seagrass(SeagrassFeature { probability: #prob }) }
        }
        "minecraft:sea_pickle" => {
            let count = value_to_int_provider(&config["count"]);
            quote! { ConfiguredFeature::SeaPickle(SeaPickleFeature { count: #count }) }
        }
        "minecraft:nether_forest_vegetation" => {
            let provider = value_to_block_state_provider(&config["state_provider"]);
            let w = config["spread_width"].as_i64().unwrap_or(8) as i32;
            let h = config["spread_height"].as_i64().unwrap_or(4) as i32;
            quote! {
                ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
                    state_provider: #provider,
                    spread_width: #w,
                    spread_height: #h,
                })
            }
        }
        "minecraft:netherrack_replace_blobs" => {
            let target = value_to_block_state_codec(&config["target"]);
            let state = value_to_block_state_codec(&config["state"]);
            let radius = value_to_int_provider(&config["radius"]);
            quote! {
                ConfiguredFeature::NetherrackReplaceBlobs(ReplaceBlobsFeature {
                    target: #target,
                    state: #state,
                    radius: #radius,
                })
            }
        }
        "minecraft:simple_block" => {
            let to_place = value_to_block_state_provider(&config["to_place"]);
            let schedule = if config["schedule_tick"].is_boolean() {
                let b = config["schedule_tick"].as_bool().unwrap_or(false);
                quote! { Some(#b) }
            } else {
                quote! { None }
            };
            quote! {
                ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
                    to_place: #to_place,
                    schedule_tick: #schedule,
                })
            }
        }
        "minecraft:ore" | "minecraft:scattered_ore" => {
            let size = config["size"].as_i64().unwrap_or(0) as i32;
            let discard = config["discard_chance_on_air_exposure"].as_f64().unwrap_or(0.0) as f32;
            let targets: Vec<TokenStream> = config["targets"]
                .as_array()
                .map(|arr| arr.iter().map(|t| {
                    let rule = value_to_rule_test(&t["target"]);
                    let state = value_to_block_state_codec(&t["state"]);
                    quote! { OreTarget { target: #rule, state: #state } }
                }).collect())
                .unwrap_or_default();
            if type_str == "minecraft:scattered_ore" {
                quote! {
                    ConfiguredFeature::ScatteredOre(crate::generation::feature::features::scattered_ore::ScatteredOreFeature {
                        // TODO
                    })
                }
            } else {
                quote! {
                    ConfiguredFeature::Ore(OreFeature {
                        size: #size,
                        discard_chance_on_air_exposure: #discard,
                        targets: vec![#(#targets),*],
                    })
                }
            }
        }
        "minecraft:spring_feature" => {
            let state = value_to_block_state_codec(&config["state"]);
            let req = config["requires_block_below"].as_bool().unwrap_or(true);
            let rock = config["rock_count"].as_i64().unwrap_or(4) as i32;
            let hole = config["hole_count"].as_i64().unwrap_or(1) as i32;
            let valid = value_to_block_wrapper(&config["valid_blocks"]);
            quote! {
                ConfiguredFeature::SpringFeature(SpringFeatureFeature {
                    state: #state,
                    requires_block_below: #req,
                    rock_count: #rock,
                    hole_count: #hole,
                    valid_blocks: #valid,
                })
            }
        }
        "minecraft:block_column" => {
            let layers: Vec<TokenStream> = config["layers"]
                .as_array()
                .map(|arr| arr.iter().map(|l| {
                    let h = value_to_int_provider(&l["height"]);
                    let p = value_to_block_state_provider(&l["provider"]);
                    quote! { Layer { height: #h, provider: #p } }
                }).collect())
                .unwrap_or_default();
            let dir = value_to_block_direction(config["direction"].as_str().unwrap_or("up"));
            let allowed = value_to_block_predicate(&config["allowed_placement"]);
            let tip = config["prioritize_tip"].as_bool().unwrap_or(false);
            quote! {
                ConfiguredFeature::BlockColumn(BlockColumnFeature {
                    layers: vec![#(#layers),*],
                    direction: #dir,
                    allowed_placement: #allowed,
                    prioritize_tip: #tip,
                })
            }
        }
        "minecraft:fallen_tree" => {
            let trunk = value_to_block_state_provider(&config["trunk_provider"]);
            quote! {
                ConfiguredFeature::FallenTree(FallenTreeFeature {
                    trunk_provider: #trunk,
                })
            }
        }
        "minecraft:random_patch" | "minecraft:flower" | "minecraft:no_bonemeal_flower" => {
            let tries = config["tries"].as_u64().unwrap_or(128) as u8;
            let xz = config["xz_spread"].as_u64().unwrap_or(7) as u8;
            let y = config["y_spread"].as_u64().unwrap_or(3) as u8;
            let feature = value_to_inline_placed_feature(&config["feature"]);
            let variant = match type_str {
                "minecraft:flower" => quote! { ConfiguredFeature::Flower },
                "minecraft:no_bonemeal_flower" => quote! { ConfiguredFeature::NoBonemealFlower },
                _ => quote! { ConfiguredFeature::RandomPatch },
            };
            quote! {
                #variant(RandomPatchFeature {
                    tries: #tries,
                    xz_spread: #xz,
                    y_spread: #y,
                    feature: Box::new(#feature),
                })
            }
        }
        "minecraft:random_selector" => {
            let entries: Vec<TokenStream> = config["features"]
                .as_array()
                .map(|arr| arr.iter().map(|e| {
                    let feat = value_to_placed_feature_wrapper(&e["feature"]);
                    let chance = e["chance"].as_f64().unwrap_or(0.1) as f32;
                    quote! { RandomFeatureEntry { feature: #feat, chance: #chance } }
                }).collect())
                .unwrap_or_default();
            let default = value_to_placed_feature_wrapper(&config["default"]);
            quote! {
                ConfiguredFeature::RandomSelector(RandomFeature {
                    features: vec![#(#entries),*],
                    default: Box::new(#default),
                })
            }
        }
        "minecraft:simple_random_selector" => {
            let features: Vec<TokenStream> = config["features"]
                .as_array()
                .map(|arr| arr.iter().map(|f| value_to_inline_placed_feature(f)).collect())
                .unwrap_or_default();
            quote! {
                ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
                    features: vec![#(#features),*],
                })
            }
        }
        "minecraft:random_boolean_selector" => {
            let ft = value_to_placed_feature_wrapper(&config["feature_true"]);
            let ff = value_to_placed_feature_wrapper(&config["feature_false"]);
            quote! {
                ConfiguredFeature::RandomBooleanSelector(RandomBooleanFeature {
                    feature_true: Box::new(#ft),
                    feature_false: Box::new(#ff),
                })
            }
        }
        "minecraft:end_spike" => {
            let inv = config["crystal_invulnerable"].as_bool().unwrap_or(false);
            let spikes: Vec<TokenStream> = config["spikes"]
                .as_array()
                .map(|arr| arr.iter().map(|s| {
                    let cx = s["centerX"].as_i64().unwrap_or(0) as i32;
                    let cz = s["centerZ"].as_i64().unwrap_or(0) as i32;
                    let r = s["radius"].as_i64().unwrap_or(0) as i32;
                    let h = s["height"].as_i64().unwrap_or(0) as i32;
                    let g = s["guarded"].as_bool().unwrap_or(false);
                    quote! {
                        Spike { center_x: #cx, center_z: #cz, radius: #r, height: #h, guarded: #g }
                    }
                }).collect())
                .unwrap_or_default();
            quote! {
                ConfiguredFeature::EndSpike(EndSpikeFeature {
                    crystal_invulnerable: #inv,
                    spikes: vec![#(#spikes),*],
                })
            }
        }
        "minecraft:tree" => {
            let tree = value_to_tree_feature(config);
            quote! { ConfiguredFeature::Tree(Box::new(#tree)) }
        }
        "minecraft:pointed_dripstone" => {
            let taller = config["chance_of_taller_dripstone"].as_f64().unwrap_or(0.2) as f32;
            let dir_spread = config["chance_of_directional_spread"].as_f64().unwrap_or(0.7) as f32;
            let r2 = config["chance_of_spread_radius2"].as_f64().unwrap_or(0.5) as f32;
            let r3 = config["chance_of_spread_radius3"].as_f64().unwrap_or(0.5) as f32;
            quote! {
                ConfiguredFeature::PointedDripstone(SmallDripstoneFeature {
                    chance_of_taller_dripstone: #taller,
                    chance_of_directional_spread: #dir_spread,
                    chance_of_spread_radius2: #r2,
                    chance_of_spread_radius3: #r3,
                })
            }
        }
        "minecraft:geode" => {
            let blocks = &config["blocks"];
            let filling_provider = value_to_block_state_provider(&blocks["filling_provider"]);
            let inner_layer_provider = value_to_block_state_provider(&blocks["inner_layer_provider"]);
            let alternate_inner_layer_provider =
                value_to_block_state_provider(&blocks["alternate_inner_layer_provider"]);
            let middle_layer_provider = value_to_block_state_provider(&blocks["middle_layer_provider"]);
            let outer_layer_provider = value_to_block_state_provider(&blocks["outer_layer_provider"]);
            let inner_placements: Vec<TokenStream> = blocks["inner_placements"]
                .as_array()
                .map(|arr| arr.iter().map(|s| value_to_block_state_codec(s)).collect())
                .unwrap_or_default();
            let cannot_replace = value_to_block_wrapper(&blocks["cannot_replace"]);
            let invalid_blocks = value_to_block_wrapper(&blocks["invalid_blocks"]);

            let layers = &config["layers"];
            let filling = layers["filling"].as_f64().unwrap_or(1.7);
            let inner_layer = layers["inner_layer"].as_f64().unwrap_or(2.2);
            let middle_layer = layers["middle_layer"].as_f64().unwrap_or(3.2);
            let outer_layer = layers["outer_layer"].as_f64().unwrap_or(4.2);

            let crack = &config["crack"];
            let generate_crack_chance = crack["generate_crack_chance"].as_f64().unwrap_or(1.0);
            let base_crack_size = crack["base_crack_size"].as_f64().unwrap_or(2.0);
            let crack_point_offset = crack["crack_point_offset"].as_i64().unwrap_or(2) as i32;

            let use_potential_placements_chance =
                config["use_potential_placements_chance"].as_f64().unwrap_or(0.35);
            let use_alternate_layer0_chance =
                config["use_alternate_layer0_chance"].as_f64().unwrap_or(0.0);
            let placements_require_layer0_alternate =
                config["placements_require_layer0_alternate"].as_bool().unwrap_or(true);
            let outer_wall_distance = value_to_int_provider(&config["outer_wall_distance"]);
            let distribution_points = value_to_int_provider(&config["distribution_points"]);
            let point_offset = value_to_int_provider(&config["point_offset"]);
            let min_gen_offset = config["min_gen_offset"].as_i64().unwrap_or(-16) as i32;
            let max_gen_offset = config["max_gen_offset"].as_i64().unwrap_or(16) as i32;
            let noise_multiplier = config["noise_multiplier"].as_f64().unwrap_or(0.05);
            let invalid_blocks_threshold =
                config["invalid_blocks_threshold"].as_i64().unwrap_or(0) as i32;

            quote! {
                ConfiguredFeature::Geode(Box::new(GeodeFeature {
                    filling_provider: #filling_provider,
                    inner_layer_provider: #inner_layer_provider,
                    alternate_inner_layer_provider: #alternate_inner_layer_provider,
                    middle_layer_provider: #middle_layer_provider,
                    outer_layer_provider: #outer_layer_provider,
                    inner_placements: vec![#(#inner_placements),*],
                    cannot_replace: #cannot_replace,
                    invalid_blocks: #invalid_blocks,
                    filling: #filling,
                    inner_layer: #inner_layer,
                    middle_layer: #middle_layer,
                    outer_layer: #outer_layer,
                    generate_crack_chance: #generate_crack_chance,
                    base_crack_size: #base_crack_size,
                    crack_point_offset: #crack_point_offset,
                    use_potential_placements_chance: #use_potential_placements_chance,
                    use_alternate_layer0_chance: #use_alternate_layer0_chance,
                    placements_require_layer0_alternate: #placements_require_layer0_alternate,
                    outer_wall_distance: #outer_wall_distance,
                    distribution_points: #distribution_points,
                    point_offset: #point_offset,
                    min_gen_offset: #min_gen_offset,
                    max_gen_offset: #max_gen_offset,
                    noise_multiplier: #noise_multiplier,
                    invalid_blocks_threshold: #invalid_blocks_threshold,
                }))
            }
        },
        // All TODO/empty features
        "minecraft:fossil" => quote! { ConfiguredFeature::Fossil(crate::generation::feature::features::fossil::FossilFeature {}) },
        "minecraft:lake" => quote! { ConfiguredFeature::Lake(crate::generation::feature::features::lake::LakeFeature {}) },
        "minecraft:disk" => quote! { ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {}) },
        "minecraft:huge_brown_mushroom" => quote! { ConfiguredFeature::HugeBrownMushroom(crate::generation::feature::features::huge_brown_mushroom::HugeBrownMushroomFeature {}) },
        "minecraft:huge_red_mushroom" => quote! { ConfiguredFeature::HugeRedMushroom(crate::generation::feature::features::huge_red_mushroom::HugeRedMushroomFeature {}) },
        "minecraft:ice_spike" => quote! { ConfiguredFeature::IceSpike(crate::generation::feature::features::ice_spike::IceSpikeFeature {}) },
        "minecraft:glowstone_blob" => quote! { ConfiguredFeature::GlowstoneBlob(crate::generation::feature::features::glowstone_blob::GlowstoneBlobFeature {}) },
        "minecraft:freeze_top_layer" => quote! { ConfiguredFeature::FreezeTopLayer(crate::generation::feature::features::freeze_top_layer::FreezeTopLayerFeature {}) },
        "minecraft:vines" => quote! { ConfiguredFeature::Vines(crate::generation::feature::features::vines::VinesFeature) },
        "minecraft:vegetation_patch" => quote! { ConfiguredFeature::VegetationPatch(crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {}) },
        "minecraft:waterlogged_vegetation_patch" => quote! { ConfiguredFeature::WaterloggedVegetationPatch(crate::generation::feature::features::waterlogged_vegetation_patch::WaterloggedVegetationPatchFeature {}) },
        "minecraft:root_system" => quote! { ConfiguredFeature::RootSystem(crate::generation::feature::features::root_system::RootSystemFeature {}) },
        "minecraft:multiface_growth" => quote! { ConfiguredFeature::MultifaceGrowth(crate::generation::feature::features::multiface_growth::MultifaceGrowthFeature {}) },
        "minecraft:underwater_magma" => quote! { ConfiguredFeature::UnderwaterMagma(crate::generation::feature::features::underwater_magma::UnderwaterMagmaFeature {}) },
        "minecraft:monster_room" => quote! { ConfiguredFeature::MonsterRoom(crate::generation::feature::features::monster_room::DungeonFeature {}) },
        "minecraft:blue_ice" => quote! { ConfiguredFeature::BlueIce(crate::generation::feature::features::blue_ice::BlueIceFeature {}) },
        "minecraft:iceberg" => quote! { ConfiguredFeature::Iceberg(crate::generation::feature::features::iceberg::IcebergFeature {}) },
        "minecraft:forest_rock" => quote! { ConfiguredFeature::ForestRock(crate::generation::feature::features::forest_rock::ForestRockFeature {}) },
        "minecraft:end_platform" => quote! { ConfiguredFeature::EndPlatform(crate::generation::feature::features::end_platform::EndPlatformFeature) },
        "minecraft:end_island" => quote! { ConfiguredFeature::EndIsland(crate::generation::feature::features::end_island::EndIslandFeature {}) },
        "minecraft:end_gateway" => quote! { ConfiguredFeature::EndGateway(crate::generation::feature::features::end_gateway::EndGatewayFeature {}) },
        "minecraft:kelp" => quote! { ConfiguredFeature::Kelp(crate::generation::feature::features::kelp::KelpFeature {}) },
        "minecraft:coral_tree" => quote! { ConfiguredFeature::CoralTree(crate::generation::feature::features::coral::coral_tree::CoralTreeFeature) },
        "minecraft:coral_mushroom" => quote! { ConfiguredFeature::CoralMushroom(crate::generation::feature::features::coral::coral_mushroom::CoralMushroomFeature) },
        "minecraft:coral_claw" => quote! { ConfiguredFeature::CoralClaw(crate::generation::feature::features::coral::coral_claw::CoralClawFeature) },
        "minecraft:huge_fungus" => quote! { ConfiguredFeature::HugeFungus(crate::generation::feature::features::huge_fungus::HugeFungusFeature {}) },
        "minecraft:weeping_vines" => quote! { ConfiguredFeature::WeepingVines(crate::generation::feature::features::weeping_vines::WeepingVinesFeature {}) },
        "minecraft:twisting_vines" => quote! { ConfiguredFeature::TwistingVines(crate::generation::feature::features::twisting_vines::TwistingVinesFeature {}) },
        "minecraft:basalt_columns" => quote! { ConfiguredFeature::BasaltColumns(crate::generation::feature::features::basalt_columns::BasaltColumnsFeature {}) },
        "minecraft:delta_feature" => quote! { ConfiguredFeature::DeltaFeature(crate::generation::feature::features::delta_feature::DeltaFeatureFeature {}) },
        "minecraft:fill_layer" => quote! { ConfiguredFeature::FillLayer(crate::generation::feature::features::fill_layer::FillLayerFeature {}) },
        "minecraft:bonus_chest" => quote! { ConfiguredFeature::BonusChest(crate::generation::feature::features::bonus_chest::BonusChestFeature {}) },
        "minecraft:basalt_pillar" => quote! { ConfiguredFeature::BasaltPillar(crate::generation::feature::features::basalt_pillar::BasaltPillarFeature {}) },
        "minecraft:dripstone_cluster" => quote! { ConfiguredFeature::DripstoneCluster(crate::generation::feature::features::drip_stone::cluster::DripstoneClusterFeature {}) },
        "minecraft:large_dripstone" => quote! { ConfiguredFeature::LargeDripstone(crate::generation::feature::features::drip_stone::large::LargeDripstoneFeature {}) },
        "minecraft:sculk_patch" => quote! { ConfiguredFeature::SculkPatch(crate::generation::feature::features::sculk_patch::SculkPatchFeature {}) },
        "minecraft:block_pile" => quote! { ConfiguredFeature::BlockPile(crate::generation::feature::features::block_pile::BlockPileFeature {}) },
        "minecraft:chorus_plant" => quote! { ConfiguredFeature::ChorusPlant(crate::generation::feature::features::chorus_plant::ChorusPlantFeature {}) },
        "minecraft:replace_single_block" => quote! { ConfiguredFeature::ReplaceSingleBlock(crate::generation::feature::features::replace_single_block::ReplaceSingleBlockFeature {}) },
        "minecraft:void_start_platform" => quote! { ConfiguredFeature::VoidStartPlatform(crate::generation::feature::features::void_start_platform::VoidStartPlatformFeature {}) },
        "minecraft:desert_well" => quote! { ConfiguredFeature::DesertWell(crate::generation::feature::features::desert_well::DesertWellFeature) },
        other => {
            let msg = format!("unknown configured feature type: {other}");
            quote! { compile_error!(#msg) }
        }
    }
}

fn value_to_block_state_provider(v: &Value) -> TokenStream {
    let type_str = v["type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:simple_state_provider" => {
            let state = value_to_block_state_codec(&v["state"]);
            quote! { BlockStateProvider::Simple(SimpleStateProvider { state: #state }) }
        }
        "minecraft:weighted_state_provider" => {
            let entries: Vec<TokenStream> = v["entries"]
                .as_array()
                .map(|arr| arr.iter().map(|e| {
                    let data = value_to_block_state_codec(&e["data"]);
                    let weight = e["weight"].as_i64().unwrap_or(1) as i32;
                    quote! { Weighted { data: #data, weight: #weight } }
                }).collect())
                .unwrap_or_default();
            quote! {
                BlockStateProvider::Weighted(WeightedBlockStateProvider {
                    entries: vec![#(#entries),*],
                })
            }
        }
        "minecraft:rotated_block_provider" => {
            let state = value_to_block_state_codec(&v["state"]);
            quote! { BlockStateProvider::Pillar(PillarBlockStateProvider { state: #state }) }
        }
        "minecraft:noise_provider" => {
            let base = value_to_noise_base(v);
            let states: Vec<TokenStream> = v["states"]
                .as_array()
                .map(|arr| arr.iter().map(|s| value_to_block_state_codec(s)).collect())
                .unwrap_or_default();
            quote! {
                BlockStateProvider::NoiseProvider(NoiseBlockStateProvider {
                    base: #base,
                    states: vec![#(#states),*],
                })
            }
        }
        "minecraft:dual_noise_provider" => {
            let base_provider = value_to_noise_base(v);
            let states: Vec<TokenStream> = v["states"]
                .as_array()
                .map(|arr| arr.iter().map(|s| value_to_block_state_codec(s)).collect())
                .unwrap_or_default();
            let base_noise_provider = quote! {
                NoiseBlockStateProvider { base: #base_provider, states: vec![#(#states),*] }
            };
            let v0 = v["variety"][0].as_u64().unwrap_or(2) as u32;
            let v1 = v["variety"][1].as_u64().unwrap_or(4) as u32;
            let slow_noise = value_to_dpnp(&v["slow_noise"]);
            let slow_scale = v["slow_scale"].as_f64().unwrap_or(1.0) as f64;
            quote! {
                BlockStateProvider::DualNoise(DualNoiseBlockStateProvider {
                    base: #base_noise_provider,
                    variety: [#v0, #v1],
                    slow_noise: #slow_noise,
                    slow_scale: #slow_scale,
                })
            }
        }
        "minecraft:noise_threshold_provider" => {
            let base = value_to_noise_base(v);
            let threshold = v["threshold"].as_f64().unwrap_or(0.0) as f32;
            let high_chance = v["high_chance"].as_f64().unwrap_or(0.0) as f32;
            let default = value_to_block_state_codec(&v["default_state"]);
            let low: Vec<TokenStream> = v["low_states"]
                .as_array()
                .map(|a| a.iter().map(|s| value_to_block_state_codec(s)).collect())
                .unwrap_or_default();
            let high: Vec<TokenStream> = v["high_states"]
                .as_array()
                .map(|a| a.iter().map(|s| value_to_block_state_codec(s)).collect())
                .unwrap_or_default();
            quote! {
                BlockStateProvider::NoiseThreshold(NoiseThresholdBlockStateProvider {
                    base: #base,
                    threshold: #threshold,
                    high_chance: #high_chance,
                    default_state: #default,
                    low_states: vec![#(#low),*],
                    high_states: vec![#(#high),*],
                })
            }
        }
        "minecraft:randomized_int_state_provider" => {
            let src = value_to_block_state_provider(&v["source"]);
            let prop = v["property"].as_str().unwrap_or("");
            let vals = value_to_int_provider(&v["values"]);
            quote! {
                BlockStateProvider::RandomizedInt(RandomizedIntBlockStateProvider {
                    source: Box::new(#src),
                    property: #prop.to_string(),
                    values: #vals,
                })
            }
        }
        _ => {
            // Default to air
            quote! {
                BlockStateProvider::Simple(SimpleStateProvider {
                    state: BlockStateCodec { name: &pumpkin_data::Block::AIR, properties: None },
                })
            }
        }
    }
}

fn value_to_noise_base(v: &Value) -> TokenStream {
    let seed = v["seed"].as_i64().unwrap_or(0);
    let noise = value_to_dpnp(&v["noise"]);
    let scale = v["scale"].as_f64().unwrap_or(1.0) as f32;
    quote! {
        NoiseBlockStateProviderBase { seed: #seed, noise: #noise, scale: #scale }
    }
}

fn value_to_dpnp(v: &Value) -> TokenStream {
    let first_octave = v["firstOctave"].as_i64().unwrap_or(-7) as i32;
    let amplitudes: Vec<f64> = v["amplitudes"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
        .unwrap_or_default();
    quote! {
        DoublePerlinNoiseParametersCodec {
            first_octave: #first_octave,
            amplitudes: vec![#(#amplitudes),*],
        }
    }
}

fn value_to_rule_test(v: &Value) -> TokenStream {
    let type_str = v["predicate_type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:always_true" | "" => quote! { RuleTest::AlwaysTrue },
        "minecraft:block_match" => {
            let block = v["block"].as_str().unwrap_or("minecraft:stone");
            quote! { RuleTest::BlockMatch(BlockMatchRuleTest { block: #block.to_string() }) }
        }
        "minecraft:blockstate_match" => {
            let state = value_to_block_state_codec(&v["block_state"]);
            quote! { RuleTest::BlockStateMatch(BlockStateMatchRuleTest { block_state: #state }) }
        }
        "minecraft:tag_match" => {
            let tag = v["tag"].as_str().unwrap_or("");
            quote! { RuleTest::TagMatch(TagMatchRuleTest { tag: #tag.to_string() }) }
        }
        "minecraft:random_block_match" => {
            let block = v["block"].as_str().unwrap_or("minecraft:stone");
            let prob = v["probability"].as_f64().unwrap_or(0.5) as f32;
            quote! { RuleTest::RandomBlockMatch(RandomBlockMatchRuleTest { block: #block.to_string(), probability: #prob }) }
        }
        "minecraft:random_blockstate_match" => {
            let state = value_to_block_state_codec(&v["block_state"]);
            let prob = v["probability"].as_f64().unwrap_or(0.5) as f32;
            quote! { RuleTest::RandomBlockStateMatch(RandomBlockStateMatchRuleTest { block_state: #state, probability: #prob }) }
        }
        other => {
            let msg = format!("unknown rule test: {other}");
            quote! { compile_error!(#msg) }
        }
    }
}

fn value_to_block_wrapper(v: &Value) -> TokenStream {
    match v {
        Value::String(s) => quote! { BlockWrapper::Single(#s.to_string()) },
        Value::Array(arr) => {
            let items: Vec<TokenStream> = arr
                .iter()
                .filter_map(|s| s.as_str().map(|s| quote! { #s.to_string() }))
                .collect();
            quote! { BlockWrapper::Multi(vec![#(#items),*]) }
        }
        _ => quote! { BlockWrapper::Single(String::new()) },
    }
}

fn value_to_tree_feature(config: &Value) -> TokenStream {
    let dirt = value_to_block_state_provider(&config["dirt_provider"]);
    let trunk = value_to_block_state_provider(&config["trunk_provider"]);
    let trunk_placer = value_to_trunk_placer(&config["trunk_placer"]);
    let foliage = value_to_block_state_provider(&config["foliage_provider"]);
    let foliage_placer = value_to_foliage_placer(&config["foliage_placer"]);
    let min_size = value_to_feature_size(&config["minimum_size"]);
    let ignore_vines = config["ignore_vines"].as_bool().unwrap_or(true);
    let force_dirt = config["force_dirt"].as_bool().unwrap_or(false);
    let decorators: Vec<TokenStream> = config["decorators"]
        .as_array()
        .map(|arr| arr.iter().map(value_to_tree_decorator).collect())
        .unwrap_or_default();
    quote! {
        TreeFeature {
            dirt_provider: #dirt,
            trunk_provider: #trunk,
            trunk_placer: #trunk_placer,
            foliage_provider: #foliage,
            foliage_placer: #foliage_placer,
            minimum_size: #min_size,
            ignore_vines: #ignore_vines,
            force_dirt: #force_dirt,
            decorators: vec![#(#decorators),*],
        }
    }
}

fn value_to_trunk_placer(v: &Value) -> TokenStream {
    let base = v["base_height"].as_u64().unwrap_or(5) as u8;
    let rand_a = v["height_rand_a"].as_u64().unwrap_or(0) as u8;
    let rand_b = v["height_rand_b"].as_u64().unwrap_or(0) as u8;
    let type_str = v["type"].as_str().unwrap_or("");
    let trunk_type = match type_str {
        "minecraft:straight_trunk_placer" => quote! { TrunkType::Straight(StraightTrunkPlacer) },
        "minecraft:forking_trunk_placer" => quote! { TrunkType::Forking(ForkingTrunkPlacer) },
        "minecraft:giant_trunk_placer" => quote! { TrunkType::Giant(GiantTrunkPlacer) },
        "minecraft:mega_jungle_trunk_placer" => {
            quote! { TrunkType::MegaJungle(MegaJungleTrunkPlacer) }
        }
        "minecraft:dark_oak_trunk_placer" => quote! { TrunkType::DarkOak(DarkOakTrunkPlacer) },
        "minecraft:fancy_trunk_placer" => quote! { TrunkType::Fancy(FancyTrunkPlacer) },
        "minecraft:bending_trunk_placer" => {
            let min_h = v["min_height_for_leaves"].as_u64().unwrap_or(1) as u32;
            let bend = value_to_int_provider(&v["bend_length"]);
            quote! {
                TrunkType::Bending(BendingTrunkPlacer {
                    min_height_for_leaves: #min_h,
                    bend_length: #bend,
                })
            }
        }
        "minecraft:upwards_branching_trunk_placer" => {
            quote! { TrunkType::UpwardsBranching(UpwardsBranchingTrunkPlacer {}) }
        }
        "minecraft:cherry_trunk_placer" => {
            quote! { TrunkType::Cherry(CherryTrunkPlacer {}) }
        }
        _ => quote! { TrunkType::Straight(StraightTrunkPlacer) },
    };
    quote! {
        TrunkPlacer {
            base_height: #base,
            height_rand_a: #rand_a,
            height_rand_b: #rand_b,
            r#type: #trunk_type,
        }
    }
}

fn value_to_foliage_placer(v: &Value) -> TokenStream {
    let radius = value_to_int_provider(&v["radius"]);
    let offset = value_to_int_provider(&v["offset"]);
    let type_str = v["type"].as_str().unwrap_or("");
    let foliage_type = match type_str {
        "minecraft:blob_foliage_placer" => {
            let h = v["height"].as_i64().unwrap_or(3) as i32;
            quote! { FoliageType::Blob(BlobFoliagePlacer { height: #h }) }
        }
        "minecraft:spruce_foliage_placer" => {
            let th = value_to_int_provider(&v["trunk_height"]);
            quote! { FoliageType::Spruce(SpruceFoliagePlacer { trunk_height: #th }) }
        }
        "minecraft:pine_foliage_placer" => {
            let h = value_to_int_provider(&v["height"]);
            quote! { FoliageType::Pine(PineFoliagePlacer { height: #h }) }
        }
        "minecraft:acacia_foliage_placer" => {
            quote! { FoliageType::Acacia(AcaciaFoliagePlacer) }
        }
        "minecraft:bush_foliage_placer" => {
            let h = v["height"].as_i64().unwrap_or(2) as i32;
            quote! { FoliageType::Bush(BushFoliagePlacer { height: #h }) }
        }
        "minecraft:fancy_foliage_placer" => {
            let h = v["height"].as_i64().unwrap_or(4) as i32;
            quote! { FoliageType::Fancy(LargeOakFoliagePlacer { height: #h }) }
        }
        "minecraft:jungle_foliage_placer" => {
            let h = v["height"].as_i64().unwrap_or(2) as i32;
            quote! { FoliageType::Jungle(JungleFoliagePlacer { height: #h }) }
        }
        "minecraft:mega_pine_foliage_placer" => {
            let ch = value_to_int_provider(&v["crown_height"]);
            quote! { FoliageType::MegaPine(MegaPineFoliagePlacer { crown_height: #ch }) }
        }
        "minecraft:dark_oak_foliage_placer" => {
            quote! { FoliageType::DarkOak(DarkOakFoliagePlacer) }
        }
        "minecraft:random_spread_foliage_placer" => {
            let fh = value_to_int_provider(&v["foliage_height"]);
            let lpa = v["leaf_placement_attempts"].as_i64().unwrap_or(128) as i32;
            quote! {
                FoliageType::RandomSpread(RandomSpreadFoliagePlacer {
                    foliage_height: #fh,
                    leaf_placement_attempts: #lpa,
                })
            }
        }
        "minecraft:cherry_foliage_placer" => {
            let h = value_to_int_provider(&v["height"]);
            let wblh = v["wide_bottom_layer_hole_chance"].as_f64().unwrap_or(0.0) as f32;
            let ch = v["corner_hole_chance"].as_f64().unwrap_or(0.0) as f32;
            let hlc = v["hanging_leaves_chance"].as_f64().unwrap_or(0.0) as f32;
            let hlec = v["hanging_leaves_extension_chance"].as_f64().unwrap_or(0.0) as f32;
            quote! {
                FoliageType::Cherry(CherryFoliagePlacer {
                    height: #h,
                    wide_bottom_layer_hole_chance: #wblh,
                    corner_hole_chance: #ch,
                    hanging_leaves_chance: #hlc,
                    hanging_leaves_extension_chance: #hlec,
                })
            }
        }
        _ => {
            let h = v["height"].as_i64().unwrap_or(3) as i32;
            quote! { FoliageType::Blob(BlobFoliagePlacer { height: #h }) }
        }
    };
    quote! {
        FoliagePlacer { radius: #radius, offset: #offset, r#type: #foliage_type }
    }
}

fn value_to_feature_size(v: &Value) -> TokenStream {
    let min_clipped = if v["min_clipped_height"].is_number() {
        let val = v["min_clipped_height"].as_u64().unwrap_or(0) as u8;
        quote! { Some(#val) }
    } else {
        quote! { None }
    };
    let type_str = v["type"].as_str().unwrap_or("");
    let size_type = match type_str {
        "minecraft:two_layers_feature_size" => {
            let limit = v["limit"].as_u64().unwrap_or(1) as u8;
            let lower = v["lower_size"].as_u64().unwrap_or(0) as u8;
            let upper = v["upper_size"].as_u64().unwrap_or(1) as u8;
            quote! {
                FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: #limit,
                    lower_size: #lower,
                    upper_size: #upper,
                })
            }
        }
        "minecraft:three_layers_feature_size" => {
            let limit = v["limit"].as_u64().unwrap_or(1) as u8;
            let upper_limit = v["upper_limit"].as_u64().unwrap_or(1) as u8;
            let lower = v["lower_size"].as_u64().unwrap_or(0) as u8;
            let middle = v["middle_size"].as_u64().unwrap_or(1) as u8;
            let upper = v["upper_size"].as_u64().unwrap_or(1) as u8;
            quote! {
                FeatureSizeType::ThreeLayersFeatureSize(ThreeLayersFeatureSize {
                    limit: #limit,
                    upper_limit: #upper_limit,
                    lower_size: #lower,
                    middle_size: #middle,
                    upper_size: #upper,
                })
            }
        }
        _ => {
            quote! {
                FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1,
                    lower_size: 0,
                    upper_size: 1,
                })
            }
        }
    };
    quote! {
        FeatureSize { min_clipped_height: #min_clipped, r#type: #size_type }
    }
}

fn value_to_tree_decorator(v: &Value) -> TokenStream {
    let type_str = v["type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:trunk_vine" => quote! { TreeDecorator::TrunkVine(TrunkVineTreeDecorator) },
        "minecraft:leave_vine" => quote! { TreeDecorator::LeaveVine(LeavesVineTreeDecorator {}) },
        "minecraft:cocoa" => quote! { TreeDecorator::Cocoa(CocoaTreeDecorator {}) },
        "minecraft:beehive" => {
            let prob = v["probability"].as_f64().unwrap_or(0.0) as f32;
            quote! { TreeDecorator::Beehive(BeehiveTreeDecorator { probability: #prob }) }
        }
        "minecraft:alter_ground" => {
            quote! { TreeDecorator::AlterGround(AlterGroundTreeDecorator {}) }
        }
        "minecraft:attached_to_logs" => {
            let prob = v["probability"].as_f64().unwrap_or(0.0) as f32;
            let bp = value_to_block_state_provider(&v["block_provider"]);
            let dirs: Vec<TokenStream> = v["directions"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|d| d.as_str().map(value_to_block_direction)).collect())
                .unwrap_or_default();
            quote! {
                TreeDecorator::AttachedToLogs(AttachedToLogsTreeDecorator {
                    probability: #prob,
                    block_provider: #bp,
                    directions: vec![#(#dirs),*],
                })
            }
        }
        "minecraft:attached_to_leaves" => {
            quote! { TreeDecorator::AttachedToLeaves(AttachedToLeavesTreeDecorator {}) }
        }
        "minecraft:place_on_ground" => {
            let tries = v["tries"].as_i64().unwrap_or(1) as i32;
            let radius = v["radius"].as_i64().unwrap_or(1) as i32;
            let height = v["height"].as_i64().unwrap_or(1) as i32;
            let bsp = value_to_block_state_provider(&v["block_state_provider"]);
            quote! {
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: #tries,
                    radius: #radius,
                    height: #height,
                    block_state_provider: #bsp,
                })
            }
        }
        "minecraft:creaking_heart" => {
            quote! { TreeDecorator::CreakingHeart(CreakingHeartTreeDecorator {}) }
        }
        "minecraft:pale_moss" => {
            quote! { TreeDecorator::PaleMoss(PaleMossTreeDecorator {}) }
        }
        other => {
            let msg = format!("unknown tree decorator: {other}");
            quote! { compile_error!(#msg) }
        }
    }
}

fn value_to_inline_placed_feature(v: &Value) -> TokenStream {
    let feature = value_to_feature_ref(&v["feature"]);
    let placement_arr = v["placement"].as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    let placement: Vec<TokenStream> = placement_arr
        .iter()
        .map(|p| value_to_placement_modifier_cf(p))
        .collect();
    quote! {
        PlacedFeature {
            feature: #feature,
            placement: vec![#(#placement),*],
        }
    }
}

fn value_to_placed_feature_wrapper(v: &Value) -> TokenStream {
    match v {
        Value::String(s) => {
            let name = s.strip_prefix("minecraft:").unwrap_or(s);
            quote! { PlacedFeatureWrapper::Named(#name.to_string()) }
        }
        Value::Object(_) => {
            // It might be a PlacedFeature object
            let pf = value_to_inline_placed_feature(v);
            quote! { PlacedFeatureWrapper::Direct(#pf) }
        }
        _ => quote! { PlacedFeatureWrapper::Named(String::new()) },
    }
}

fn value_to_feature_ref(v: &Value) -> TokenStream {
    match v {
        Value::String(s) => {
            let name = s.strip_prefix("minecraft:").unwrap_or(s);
            quote! { Feature::Named(#name.to_string()) }
        }
        Value::Object(_) => {
            let cf = value_to_configured_feature(v);
            quote! { Feature::Inlined(Box::new(#cf)) }
        }
        _ => quote! { Feature::Named(String::new()) },
    }
}

// Placement modifier in context of configured_features (same as placed_feature but re-imported)
fn value_to_placement_modifier_cf(v: &Value) -> TokenStream {
    let type_str = v["type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:biome" => quote! { PlacementModifier::Biome(BiomePlacementModifier) },
        "minecraft:in_square" => quote! { PlacementModifier::InSquare(SquarePlacementModifier) },
        "minecraft:fixed_placement" => quote! { PlacementModifier::FixedPlacement },
        "minecraft:heightmap" => {
            let hm = crate::placed_feature::value_to_height_map(v["heightmap"].as_str().unwrap_or("MOTION_BLOCKING"));
            quote! { PlacementModifier::Heightmap(HeightmapPlacementModifier { heightmap: #hm }) }
        }
        "minecraft:height_range" => {
            let h = value_to_height_provider(&v["height"]);
            quote! { PlacementModifier::HeightRange(HeightRangePlacementModifier { height: #h }) }
        }
        "minecraft:count" => {
            let c = value_to_int_provider(&v["count"]);
            quote! { PlacementModifier::Count(CountPlacementModifier { count: #c }) }
        }
        "minecraft:count_on_every_layer" => {
            let c = value_to_int_provider(&v["count"]);
            quote! { PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier { count: #c }) }
        }
        "minecraft:rarity_filter" => {
            let ch = v["chance"].as_u64().unwrap_or(1) as u32;
            quote! { PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: #ch }) }
        }
        "minecraft:block_predicate_filter" => {
            let pred = value_to_block_predicate(&v["predicate"]);
            quote! { PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier { predicate: #pred }) }
        }
        "minecraft:surface_relative_threshold_filter" => {
            let hm = crate::placed_feature::value_to_height_map(v["heightmap"].as_str().unwrap_or("MOTION_BLOCKING"));
            let mn = if v["min_inclusive"].is_number() { let x = v["min_inclusive"].as_i64().unwrap() as i32; quote!{Some(#x)} } else { quote!{None} };
            let mx = if v["max_inclusive"].is_number() { let x = v["max_inclusive"].as_i64().unwrap() as i32; quote!{Some(#x)} } else { quote!{None} };
            quote! { PlacementModifier::SurfaceRelativeThresholdFilter(SurfaceThresholdFilterPlacementModifier { heightmap: #hm, min_inclusive: #mn, max_inclusive: #mx }) }
        }
        "minecraft:surface_water_depth_filter" => {
            let d = v["max_water_depth"].as_i64().unwrap_or(0) as i32;
            quote! { PlacementModifier::SurfaceWaterDepthFilter(SurfaceWaterDepthFilterPlacementModifier { max_water_depth: #d }) }
        }
        "minecraft:random_offset" => {
            let xz = value_to_int_provider(&v["xz_spread"]);
            let y = value_to_int_provider(&v["y_spread"]);
            quote! { PlacementModifier::RandomOffset(RandomOffsetPlacementModifier { xz_spread: #xz, y_spread: #y }) }
        }
        "minecraft:noise_based_count" => {
            let r = v["noise_to_count_ratio"].as_i64().unwrap_or(0) as i32;
            let f = v["noise_factor"].as_f64().unwrap_or(1.0);
            let o = v["noise_offset"].as_f64().unwrap_or(0.0);
            quote! { PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier { noise_to_count_ratio: #r, noise_factor: #f, noise_offset: #o }) }
        }
        "minecraft:noise_threshold_count" => {
            let l = v["noise_level"].as_f64().unwrap_or(0.0);
            let b = v["below_noise"].as_i64().unwrap_or(0) as i32;
            let a = v["above_noise"].as_i64().unwrap_or(0) as i32;
            quote! { PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier { noise_level: #l, below_noise: #b, above_noise: #a }) }
        }
        "minecraft:environment_scan" => {
            let dir = value_to_block_direction(v["direction_of_search"].as_str().unwrap_or("down"));
            let tc = value_to_block_predicate(&v["target_condition"]);
            let asc = if v["allowed_search_condition"].is_object() {
                let p = value_to_block_predicate(&v["allowed_search_condition"]);
                quote! { Some(#p) }
            } else { quote! { None } };
            let steps = v["max_steps"].as_i64().unwrap_or(1) as i32;
            quote! { PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier { direction_of_search: #dir, target_condition: #tc, allowed_search_condition: #asc, max_steps: #steps }) }
        }
        _ => quote! { PlacementModifier::Biome(BiomePlacementModifier) },
    }
}

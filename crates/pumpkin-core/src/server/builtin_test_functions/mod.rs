mod allay_interaction;
mod block_tuning_and_invert;
mod bubble_columns;
mod buttons_levers_plates_variants;
mod commands_and_entities_extended;
mod commands_and_gamemode;
mod comparators_extended;
mod composter_and_cauldron;
mod containers_and_commands;
mod containers_and_hoppers_extended;
mod dispenser_utilities;
mod dispensers_and_crafters;
mod dispensers_and_droppers_extended;
mod doors_and_gates;
mod doors_trapdoors_gates_variants;
mod entities_and_ai;
mod fluids_and_sponge;
mod furnace_smelting;
mod golems;
mod lamps_bulbs_and_noteblocks_extended;
mod loot_tables;
mod note_and_comparators;
mod pistons;
mod pistons_extended;
mod plants_and_support_blocks;
mod polar_bear;
mod rails;
mod rails_and_fluids_extended;
mod redstone_gates;
mod redstone_inputs;
mod redstone_lamps_and_bulbs;
mod redstone_logic_circuits;
mod redstone_wire_and_power;
mod scaffolding;
mod sheep_shearing;
mod simulated_player;
mod target_and_projectiles;
mod tripwire;
mod walls_stairs_and_shapes;
mod world_interaction;

use std::sync::Once;

use pumpkin_data::Block;
use pumpkin_gametest::{GameTestContext, GameTestResult, register_test_function};
use pumpkin_util::math::position::BlockPos;

static REGISTER_BUILTIN_TEST_FUNCTIONS: Once = Once::new();

#[expect(
    clippy::too_many_lines,
    reason = "registers all built-in test functions in one call_once table"
)]
pub fn ensure_builtin_test_functions_registered() {
    REGISTER_BUILTIN_TEST_FUNCTIONS.call_once(|| {
        register_test_function("pumpkin:allay_interaction", allay_interaction::test_allay_interaction);
        register_test_function("pumpkin:polar_bear_cub_defense_and_target_lifecycle", polar_bear::test_polar_bear_cub_defense_and_target_lifecycle);
        register_test_function("pumpkin:shearing_tool_loot_and_item_events", sheep_shearing::test_shearing_tool_loot_and_item_events);
        register_test_function("pumpkin:block_tuning_and_invert", block_tuning_and_invert::test_block_tuning_and_invert);
        register_test_function("pumpkin:commands_gamemode_give_setblock", commands_and_gamemode::test_commands_gamemode_give_setblock);
        register_test_function("pumpkin:loot_entity_context_and_nested_tables", loot_tables::test_loot_entity_context_and_nested_tables);
        register_test_function("pumpkin:simulated_player_lifecycle", simulated_player::test_simulated_player_lifecycle);
        register_test_function("pumpkin:simulated_player_actions", simulated_player::test_simulated_player_actions);
        register_test_function("pumpkin:world_block_use_and_break", world_interaction::test_world_block_use_and_break);
        register_test_function("pumpkin:redstone_torch_inverter", redstone_gates::test_redstone_torch_inverter);
        register_test_function("pumpkin:redstone_repeater_delay", redstone_gates::test_redstone_repeater_delay);
        register_test_function("pumpkin:redstone_comparator_modes", redstone_gates::test_redstone_comparator_modes);
        register_test_function("pumpkin:composter_and_cauldron_comparators", composter_and_cauldron::test_composter_and_cauldron_comparators);
        register_test_function("pumpkin:end_portal_frame_and_respawn_anchor_comparators", note_and_comparators::test_end_portal_frame_and_respawn_anchor_comparators);
        register_test_function("pumpkin:note_block_instrument_and_power", note_and_comparators::test_note_block_instrument_and_power);
        register_test_function("pumpkin:redstone_observer_pulse", redstone_inputs::test_redstone_observer_pulse);
        register_test_function("pumpkin:stone_button_pulse", redstone_inputs::test_stone_button_pulse);
        register_test_function("pumpkin:lever_toggles_power", redstone_inputs::test_lever_toggles_power);
        register_test_function("pumpkin:pressure_plate_entity_activation", redstone_inputs::test_pressure_plate_entity_activation);
        register_test_function("pumpkin:target_block_projectile_hit", target_and_projectiles::test_target_block_projectile_hit);
        register_test_function("pumpkin:copper_bulb_toggles_and_comparator", redstone_lamps_and_bulbs::test_copper_bulb_toggles_and_comparator);
        register_test_function("pumpkin:redstone_lamp_delayed_off", redstone_lamps_and_bulbs::test_redstone_lamp_delayed_off);
        register_test_function("pumpkin:piston_and_sticky_piston", pistons::test_piston_and_sticky_piston);
        register_test_function("pumpkin:dispenser_and_dropper", dispensers_and_crafters::test_dispenser_and_dropper);
        register_test_function("pumpkin:dispenser_honeycomb_beehive_and_shulker", dispenser_utilities::test_dispenser_honeycomb_beehive_and_shulker);
        register_test_function("pumpkin:crafter_crafts_into_chest", dispensers_and_crafters::test_crafter_crafts_into_chest);
        register_test_function("pumpkin:doors_trapdoors_fence_gates_redstone", doors_and_gates::test_doors_trapdoors_fence_gates_redstone);
        register_test_function("pumpkin:water_lava_interactions", fluids_and_sponge::test_water_lava_interactions);
        register_test_function("pumpkin:sponge_absorbs_water", fluids_and_sponge::test_sponge_absorbs_water);
        register_test_function("pumpkin:bubble_column_upward_and_downward", bubble_columns::test_bubble_column_upward_and_downward);
        register_test_function("pumpkin:falling_sand_lands", containers_and_commands::test_falling_sand_lands);
        register_test_function("pumpkin:scaffolding_distance_and_collapse", scaffolding::test_scaffolding_distance_and_collapse);
        register_test_function("pumpkin:command_block_chain_execution", containers_and_commands::test_command_block_chain_execution);
        register_test_function("pumpkin:hopper_transfers_to_chest", containers_and_commands::test_hopper_transfers_to_chest);
        register_test_function("pumpkin:tnt_redstone_ignites", containers_and_commands::test_tnt_redstone_ignites);
        register_test_function("pumpkin:furnace_ignites_and_smelts", furnace_smelting::test_furnace_ignites_and_smelts);
        register_test_function("pumpkin:powered_rail_propagation", rails::test_powered_rail_propagation);
        register_test_function("pumpkin:activator_rail_propagation", rails::test_activator_rail_propagation);
        register_test_function("pumpkin:detector_rail_minecart", rails::test_detector_rail_minecart);
        register_test_function("pumpkin:tripwire_entity_collision", tripwire::test_tripwire_entity_collision);
        register_test_function("pumpkin:snow_golem_pattern_spawns", golems::test_snow_golem_pattern_spawns);
        register_test_function("pumpkin:iron_golem_pattern_spawns", golems::test_iron_golem_pattern_spawns);
        register_test_function("pumpkin:wither_pattern_spawns", golems::test_wither_pattern_spawns);
        register_test_function("pumpkin:item_entities_merge", entities_and_ai::test_item_entities_merge);
        register_test_function("pumpkin:zombie_pathfinds_to_villager", entities_and_ai::test_zombie_pathfinds_to_villager);

        // redstone_wire_and_power (14)
        register_test_function("pumpkin:redstone_wire_attenuation_1_to_4", redstone_wire_and_power::test_redstone_wire_attenuation_1_to_4);
        register_test_function("pumpkin:redstone_wire_attenuation_5_to_8", redstone_wire_and_power::test_redstone_wire_attenuation_5_to_8);
        register_test_function("pumpkin:redstone_wire_attenuation_9_to_12", redstone_wire_and_power::test_redstone_wire_attenuation_9_to_12);
        register_test_function("pumpkin:redstone_wire_attenuation_13_to_15", redstone_wire_and_power::test_redstone_wire_attenuation_13_to_15);
        register_test_function("pumpkin:redstone_wire_corner_ne_and_sw", redstone_wire_and_power::test_redstone_wire_corner_north_east);
        register_test_function("pumpkin:redstone_wire_t_junction", redstone_wire_and_power::test_redstone_wire_t_junction_split);
        register_test_function("pumpkin:redstone_wire_cross_4_way", redstone_wire_and_power::test_redstone_wire_cross_junction_four_way);
        register_test_function("pumpkin:redstone_wire_step_up_climb", redstone_wire_and_power::test_redstone_wire_step_up_climb);
        register_test_function("pumpkin:redstone_wire_step_down_drop", redstone_wire_and_power::test_redstone_wire_step_down_drop);
        register_test_function("pumpkin:redstone_wire_dual_sources_max", redstone_wire_and_power::test_redstone_wire_dual_sources_max_power);
        register_test_function("pumpkin:redstone_wire_source_removal_deenergizes", redstone_wire_and_power::test_redstone_wire_source_removal_deenergizes);
        register_test_function("pumpkin:redstone_wire_repeater_reboosts_to_15", redstone_wire_and_power::test_redstone_wire_repeater_reboosts_signal);
        register_test_function("pumpkin:redstone_wire_strong_power_through_stone", redstone_wire_and_power::test_redstone_wire_strong_power_through_stone);
        register_test_function("pumpkin:redstone_block_powers_all_horizontal_wires", redstone_wire_and_power::test_redstone_wire_corner_south_west);

        // redstone_logic_circuits (12)
        register_test_function("pumpkin:redstone_double_inverter_buffer", redstone_logic_circuits::test_redstone_double_inverter_buffer);
        register_test_function("pumpkin:redstone_wall_torch_inverter", redstone_logic_circuits::test_redstone_wall_torch_inverter);
        register_test_function("pumpkin:redstone_or_gate", redstone_logic_circuits::test_redstone_or_gate_two_inputs);
        register_test_function("pumpkin:redstone_nor_gate", redstone_logic_circuits::test_redstone_nor_gate_two_inputs);
        register_test_function("pumpkin:redstone_and_gate_three_torches", redstone_logic_circuits::test_redstone_and_gate_three_torches);
        register_test_function("pumpkin:redstone_nand_gate_two_torches", redstone_logic_circuits::test_redstone_nand_gate_two_torches);
        register_test_function("pumpkin:redstone_rs_nor_latch", redstone_logic_circuits::test_redstone_rs_nor_latch_memory);
        register_test_function("pumpkin:redstone_repeater_delay_2_ticks", redstone_logic_circuits::test_redstone_repeater_delay_2_ticks);
        register_test_function("pumpkin:redstone_repeater_delay_3_ticks", redstone_logic_circuits::test_redstone_repeater_delay_3_ticks);
        register_test_function("pumpkin:redstone_repeater_delay_4_ticks", redstone_logic_circuits::test_redstone_repeater_delay_4_ticks);
        register_test_function("pumpkin:redstone_repeater_locking_holds_off", redstone_logic_circuits::test_redstone_repeater_locking_holds_off);
        register_test_function("pumpkin:redstone_repeater_locking_holds_on", redstone_logic_circuits::test_redstone_repeater_locking_holds_on);

        // comparators_extended (14)
        register_test_function("pumpkin:comparator_composter_levels_1_to_4", comparators_extended::test_comparator_composter_levels_1_to_4);
        register_test_function("pumpkin:comparator_composter_levels_5_to_8", comparators_extended::test_comparator_composter_levels_5_to_8);
        register_test_function("pumpkin:comparator_water_cauldron_levels_1_to_3", comparators_extended::test_comparator_water_cauldron_levels_1_2_3);
        register_test_function("pumpkin:comparator_powder_snow_cauldron_levels_1_to_3", comparators_extended::test_comparator_powder_snow_cauldron_levels_1_2_3);
        register_test_function("pumpkin:comparator_respawn_anchor_charges_0_to_4", comparators_extended::test_comparator_respawn_anchor_charges_0_1_3_4);
        register_test_function("pumpkin:comparator_cake_bites_0_2_4_6", comparators_extended::test_comparator_cake_bites_0_2_4_6);
        register_test_function("pumpkin:comparator_candle_cake_and_lit_candle_cake", comparators_extended::test_comparator_candle_cake_full_signal);
        register_test_function("pumpkin:comparator_chest_single_item_signal_1", comparators_extended::test_comparator_chest_single_item_signal_1);
        register_test_function("pumpkin:comparator_chest_nonstackable_items_signal", comparators_extended::test_comparator_chest_nine_swords_signal_5);
        register_test_function("pumpkin:comparator_hopper_partial_and_full", comparators_extended::test_comparator_hopper_fill_levels_1_and_5);
        register_test_function("pumpkin:comparator_dispenser_and_dropper_signal", comparators_extended::test_comparator_dispenser_and_dropper_three_swords);
        register_test_function("pumpkin:comparator_furnace_slots_signal", comparators_extended::test_comparator_furnace_slot_signal);
        register_test_function("pumpkin:comparator_copper_bulb_oxidized_variants", comparators_extended::test_comparator_copper_bulbs_oxidation_variants);
        register_test_function("pumpkin:comparator_subtract_clamps_to_zero", comparators_extended::test_comparator_subtract_mode_clamps_to_zero);

        // buttons_levers_plates_variants (12)
        register_test_function("pumpkin:oak_button_30_tick_pulse", buttons_levers_plates_variants::test_oak_button_30_tick_pulse);
        register_test_function("pumpkin:polished_blackstone_button_20_tick_pulse", buttons_levers_plates_variants::test_polished_blackstone_button_20_tick_pulse);
        register_test_function("pumpkin:button_floor_and_ceiling_power", buttons_levers_plates_variants::test_stone_button_floor_mount_strong_power);
        register_test_function("pumpkin:button_wall_strong_powers_behind_block", buttons_levers_plates_variants::test_stone_button_wall_mount_strong_power);
        register_test_function("pumpkin:button_breaks_when_support_removed", buttons_levers_plates_variants::test_button_breaks_when_support_removed);
        register_test_function("pumpkin:lever_wall_and_ceiling_orientations", buttons_levers_plates_variants::test_lever_ceiling_mount_strong_power);
        register_test_function("pumpkin:lever_strong_powers_supporting_block", buttons_levers_plates_variants::test_lever_wall_mount_strong_power);
        register_test_function("pumpkin:lever_breaks_when_support_removed", buttons_levers_plates_variants::test_lever_breaks_when_support_removed);
        register_test_function("pumpkin:stone_pressure_plate_ignores_items_activates_on_mob", buttons_levers_plates_variants::test_stone_button_ceiling_mount_strong_power);
        register_test_function("pumpkin:wooden_pressure_plate_activates_on_item_and_clears", buttons_levers_plates_variants::test_wooden_pressure_plate_item_activation);
        register_test_function("pumpkin:light_weighted_pressure_plate_counts_entities", buttons_levers_plates_variants::test_weighted_pressure_plate_light_scaling);
        register_test_function("pumpkin:heavy_weighted_pressure_plate_signal_1", buttons_levers_plates_variants::test_weighted_pressure_plate_heavy_scaling);

        // doors_trapdoors_gates_variants (12)
        register_test_function("pumpkin:oak_and_spruce_doors_redstone", doors_trapdoors_gates_variants::test_oak_and_spruce_doors_redstone);
        register_test_function("pumpkin:birch_and_jungle_doors_redstone", doors_trapdoors_gates_variants::test_birch_and_jungle_doors_redstone);
        register_test_function("pumpkin:acacia_dark_oak_mangrove_cherry_doors_redstone", doors_trapdoors_gates_variants::test_acacia_dark_oak_mangrove_cherry_doors);
        register_test_function("pumpkin:bamboo_crimson_warped_doors_redstone", doors_trapdoors_gates_variants::test_bamboo_crimson_warped_doors);
        register_test_function("pumpkin:copper_doors_oxidation_variants_redstone", doors_trapdoors_gates_variants::test_copper_doors_oxidation_variants);
        register_test_function("pumpkin:door_breaking_lower_removes_upper", doors_trapdoors_gates_variants::test_door_breaks_when_lower_half_removed);
        register_test_function("pumpkin:trapdoor_wooden_variants_redstone", doors_trapdoors_gates_variants::test_wooden_trapdoors_variants_redstone);
        register_test_function("pumpkin:copper_trapdoors_oxidation_variants_redstone", doors_trapdoors_gates_variants::test_copper_trapdoors_oxidation_variants);
        register_test_function("pumpkin:fence_gate_wooden_variants_redstone", doors_trapdoors_gates_variants::test_fence_gates_wood_variants_redstone);
        register_test_function("pumpkin:fence_gate_in_wall_neighbor_update", doors_trapdoors_gates_variants::test_fence_gate_in_wall_neighbor_update);
        register_test_function("pumpkin:fence_horizontal_connections", doors_trapdoors_gates_variants::test_fence_horizontal_connections_update);
        register_test_function("pumpkin:iron_bars_and_glass_pane_connections", doors_trapdoors_gates_variants::test_iron_bars_and_glass_pane_connections);

        // pistons_extended (12)
        register_test_function("pumpkin:piston_pushes_upward_and_retracts", pistons_extended::test_piston_pushes_upward_and_retracts);
        register_test_function("pumpkin:piston_pushes_downward_and_retracts", pistons_extended::test_piston_pushes_downward_and_retracts);
        register_test_function("pumpkin:piston_pushes_north_and_south", pistons_extended::test_piston_pushes_north_and_south);
        register_test_function("pumpkin:piston_pushes_west_and_east", pistons_extended::test_piston_pushes_west_and_east);
        register_test_function("pumpkin:sticky_piston_pulls_upward_and_downward", pistons_extended::test_sticky_piston_pulls_upward_and_downward);
        register_test_function("pumpkin:piston_pushes_multi_block_row", pistons_extended::test_piston_pushes_multi_block_row);
        register_test_function("pumpkin:piston_blocked_by_obsidian", pistons_extended::test_piston_blocked_by_obsidian);
        register_test_function("pumpkin:piston_blocked_by_bedrock", pistons_extended::test_piston_blocked_by_bedrock);
        register_test_function("pumpkin:piston_destroys_torch_and_moss_carpet", pistons_extended::test_piston_destroys_fragile_torch_and_moss);
        register_test_function("pumpkin:sticky_piston_leaves_obsidian_behind", pistons_extended::test_sticky_piston_leaves_obsidian_unmoved);
        register_test_function("pumpkin:sticky_piston_redstone_block_powers_lamp", pistons_extended::test_piston_pushes_redstone_block_powers_lamp);
        register_test_function("pumpkin:piston_one_stone_leaves_pushed_block", pistons_extended::test_sticky_piston_pulls_redstone_block_cuts_power);

        // dispensers_and_droppers_extended (12)
        register_test_function("pumpkin:dispenser_lava_bucket_places_and_picks_up", dispensers_and_droppers_extended::test_dispenser_lava_bucket_place_and_pickup);
        register_test_function("pumpkin:dispenser_powder_snow_bucket_places_and_picks_up", dispensers_and_droppers_extended::test_dispenser_powder_snow_bucket_place_and_pickup);
        register_test_function("pumpkin:dispenser_shears_harvest_full_beehive", dispensers_and_droppers_extended::test_dispenser_shears_full_beehive_drops_honeycomb);
        register_test_function("pumpkin:dispenser_glass_bottle_fills_from_water", dispensers_and_droppers_extended::test_dispenser_glass_bottle_fills_water_from_source);
        register_test_function("pumpkin:dispenser_flint_and_steel_primes_tnt", dispensers_and_droppers_extended::test_dispenser_flint_and_steel_primes_tnt);
        register_test_function("pumpkin:dispenser_flint_and_steel_lights_campfire", dispensers_and_droppers_extended::test_dispenser_flint_and_steel_lights_campfire);
        register_test_function("pumpkin:dispenser_armor_stand_spawns_entity", dispensers_and_droppers_extended::test_dispenser_armor_stand_spawns_entity);
        register_test_function("pumpkin:dispenser_tnt_spawns_primed_tnt", dispensers_and_droppers_extended::test_dispenser_tnt_spawns_primed_tnt);
        register_test_function("pumpkin:dispenser_minecart_places_on_rail", dispensers_and_droppers_extended::test_dispenser_minecart_places_on_rail);
        register_test_function("pumpkin:dispenser_boat_places_on_water", dispensers_and_droppers_extended::test_dispenser_boat_places_on_water);
        register_test_function("pumpkin:dropper_drops_bucket_as_item_entity", dispensers_and_droppers_extended::test_dropper_drops_item_entity_into_air);
        register_test_function("pumpkin:dropper_transfers_into_hopper_and_barrel", dispensers_and_droppers_extended::test_dropper_transfers_into_hopper_and_barrel);

        // containers_and_hoppers_extended (12)
        register_test_function("pumpkin:hopper_pulls_item_entity_from_above", containers_and_hoppers_extended::test_hopper_pulls_item_entity_from_above);
        register_test_function("pumpkin:hopper_pulls_from_chest_above", containers_and_hoppers_extended::test_hopper_pulls_from_chest_above);
        register_test_function("pumpkin:hopper_pushes_sideways_into_barrel", containers_and_hoppers_extended::test_hopper_pushes_sideways_into_barrel);
        register_test_function("pumpkin:hopper_chain_three_hoppers_to_chest", containers_and_hoppers_extended::test_hopper_chain_three_hoppers_to_chest);
        register_test_function("pumpkin:hopper_locked_by_redstone_stops_transfer", containers_and_hoppers_extended::test_hopper_locked_by_redstone_stops_transfer);
        register_test_function("pumpkin:hopper_unlocks_when_redstone_removed", containers_and_hoppers_extended::test_hopper_unlocked_resumes_transfer);
        register_test_function("pumpkin:hopper_feeds_furnace_top_and_side", containers_and_hoppers_extended::test_hopper_feeds_furnace_top_and_side);
        register_test_function("pumpkin:blast_furnace_smelts_raw_iron", containers_and_hoppers_extended::test_blast_furnace_smelts_raw_iron);
        register_test_function("pumpkin:smoker_cooks_raw_beef", containers_and_hoppers_extended::test_smoker_cooks_raw_beef);
        register_test_function("pumpkin:crafter_crafts_iron_ingot_from_nuggets", containers_and_hoppers_extended::test_crafter_crafts_iron_ingot_from_nuggets);
        register_test_function("pumpkin:crafter_crafts_sticks_from_two_planks", containers_and_hoppers_extended::test_crafter_disabled_slots_skip_items);
        register_test_function("pumpkin:barrel_and_trapped_chest_store_items", containers_and_hoppers_extended::test_barrel_and_trapped_chest_inventories);

        // walls_stairs_and_shapes (12)
        register_test_function("pumpkin:wall_straight_line_has_no_center_post", walls_stairs_and_shapes::test_cobblestone_wall_connects_north_south_straight);
        register_test_function("pumpkin:wall_corner_raises_center_post", walls_stairs_and_shapes::test_cobblestone_wall_corner_raises_center_post);
        register_test_function("pumpkin:wall_under_full_block_turns_tall", walls_stairs_and_shapes::test_cobblestone_wall_tall_side_under_full_block);
        register_test_function("pumpkin:wall_connects_to_iron_bars_and_glass_pane", walls_stairs_and_shapes::test_wall_connects_to_iron_bars_and_glass_pane);
        register_test_function("pumpkin:wall_connects_to_fence_gate", walls_stairs_and_shapes::test_wall_connects_to_fence_gate);
        register_test_function("pumpkin:stairs_inner_left_and_inner_right_shapes", walls_stairs_and_shapes::test_stairs_inner_left_and_right_corner_shapes);
        register_test_function("pumpkin:stairs_outer_left_and_outer_right_shapes", walls_stairs_and_shapes::test_stairs_outer_left_and_right_corner_shapes);
        register_test_function("pumpkin:stairs_revert_to_straight_when_neighbor_removed", walls_stairs_and_shapes::test_stairs_revert_to_straight_when_neighbor_removed);
        register_test_function("pumpkin:dirt_path_turns_to_dirt_under_solid_block", walls_stairs_and_shapes::test_dirt_path_turns_to_dirt_under_solid_block);
        register_test_function("pumpkin:farmland_turns_to_dirt_under_solid_block", walls_stairs_and_shapes::test_farmland_turns_to_dirt_under_solid_block);
        register_test_function("pumpkin:campfire_signal_fire_over_hay_block", walls_stairs_and_shapes::test_campfire_signal_fire_over_hay_block);
        register_test_function("pumpkin:campfire_waterlogged_extinguishes_lit", walls_stairs_and_shapes::test_campfire_waterlogged_extinguishes_lit);

        // plants_and_support_blocks (12)
        register_test_function("pumpkin:cactus_breaks_when_solid_block_placed_adjacent", plants_and_support_blocks::test_cactus_breaks_when_solid_block_placed_adjacent);
        register_test_function("pumpkin:cactus_breaks_when_sand_below_removed", plants_and_support_blocks::test_cactus_breaks_when_sand_support_removed);
        register_test_function("pumpkin:sugar_cane_breaks_when_water_removed", plants_and_support_blocks::test_sugar_cane_breaks_when_water_removed);
        register_test_function("pumpkin:sugar_cane_tall_column_cascades_break", plants_and_support_blocks::test_sugar_cane_column_cascades_break);
        register_test_function("pumpkin:torch_and_soul_torch_break_without_support", plants_and_support_blocks::test_torch_and_soul_torch_break_without_support);
        register_test_function("pumpkin:wall_torch_breaks_when_wall_removed", plants_and_support_blocks::test_wall_torch_breaks_when_wall_removed);
        register_test_function("pumpkin:standing_lantern_breaks_when_floor_removed", plants_and_support_blocks::test_lantern_standing_breaks_when_floor_removed);
        register_test_function("pumpkin:hanging_lantern_breaks_when_ceiling_removed", plants_and_support_blocks::test_lantern_hanging_breaks_when_ceiling_removed);
        register_test_function("pumpkin:lantern_on_fence_gate_breaks_when_gate_opens", plants_and_support_blocks::test_lantern_on_fence_gate_breaks_when_gate_opens);
        register_test_function("pumpkin:cake_and_candle_cake_break_without_support", plants_and_support_blocks::test_cake_and_candle_cake_break_when_floor_removed);
        register_test_function("pumpkin:carpet_and_moss_carpet_break_without_support", plants_and_support_blocks::test_carpet_and_moss_carpet_break_when_floor_removed);
        register_test_function("pumpkin:pressure_plate_and_redstone_wire_break_without_support", plants_and_support_blocks::test_pressure_plate_breaks_when_floor_removed);

        // lamps_bulbs_and_noteblocks_extended (12)
        register_test_function("pumpkin:copper_bulb_variants_toggle_and_remember_lit", lamps_bulbs_and_noteblocks_extended::test_copper_bulb_oxidation_variants_toggle);
        register_test_function("pumpkin:waxed_copper_bulb_variants_toggle_and_remember_lit", lamps_bulbs_and_noteblocks_extended::test_waxed_copper_bulb_variants_toggle);
        register_test_function("pumpkin:redstone_lamp_stays_lit_while_powered", lamps_bulbs_and_noteblocks_extended::test_redstone_lamp_stays_lit_while_powered);
        register_test_function("pumpkin:note_block_instruments_rock_sand_glass_wood", lamps_bulbs_and_noteblocks_extended::test_note_block_instruments_basedrum_snare_hat_bass);
        register_test_function("pumpkin:note_block_instruments_clay_gold_wool_ice", lamps_bulbs_and_noteblocks_extended::test_note_block_instruments_flute_bell_guitar_chime);
        register_test_function("pumpkin:note_block_instruments_bone_iron_soul_pumpkin", lamps_bulbs_and_noteblocks_extended::test_note_block_instruments_xylophone_iron_cowbell_didgeridoo);
        register_test_function("pumpkin:note_block_instruments_emerald_hay_glowstone_default", lamps_bulbs_and_noteblocks_extended::test_note_block_instruments_bit_banjo_pling_harp);
        register_test_function("pumpkin:note_block_mob_head_above_sets_instrument", lamps_bulbs_and_noteblocks_extended::test_note_block_top_mob_head_instruments);
        register_test_function("pumpkin:daylight_detector_normal_and_inverted_signal", lamps_bulbs_and_noteblocks_extended::test_daylight_detector_signal_calculation);
        register_test_function("pumpkin:observer_detects_redstone_lamp_state_change", lamps_bulbs_and_noteblocks_extended::test_observer_detects_repeater_and_lamp_changes);
        register_test_function("pumpkin:observer_chain_propagates_pulse", lamps_bulbs_and_noteblocks_extended::test_observer_chain_propagates_pulse);
        register_test_function("pumpkin:target_block_egg_hit_emits_pulse_and_clears", lamps_bulbs_and_noteblocks_extended::test_target_block_arrow_hit_20_tick_pulse);

        // rails_and_fluids_extended (12)
        register_test_function("pumpkin:powered_rail_deenergizes_beyond_8_blocks", rails_and_fluids_extended::test_powered_rail_deenergizes_beyond_8_blocks);
        register_test_function("pumpkin:activator_rail_deenergizes_beyond_8_blocks", rails_and_fluids_extended::test_activator_rail_deenergizes_beyond_8_blocks);
        register_test_function("pumpkin:powered_rail_bidirectional_boost_from_center", rails_and_fluids_extended::test_powered_rail_bidirectional_boost_from_center);
        register_test_function("pumpkin:detector_rail_powers_adjacent_powered_rail", rails_and_fluids_extended::test_detector_rail_powers_adjacent_powered_rail);
        register_test_function("pumpkin:detector_rail_activates_on_chest_minecart", rails_and_fluids_extended::test_detector_rail_comparator_reads_chest_minecart);
        register_test_function("pumpkin:rail_breaks_when_supporting_block_removed", rails_and_fluids_extended::test_rail_breaks_when_support_block_removed);
        register_test_function("pumpkin:water_flows_horizontally_to_neighbors", rails_and_fluids_extended::test_water_flows_horizontally_and_decreases_level);
        register_test_function("pumpkin:water_flows_downward_into_hole", rails_and_fluids_extended::test_water_falls_downward_into_basin);
        register_test_function("pumpkin:lava_source_turned_to_obsidian_by_water", rails_and_fluids_extended::test_lava_meets_flowing_water_forms_cobblestone);
        register_test_function("pumpkin:water_flow_breaks_redstone_torch", rails_and_fluids_extended::test_water_breaks_torch_and_carpet_in_path);
        register_test_function("pumpkin:sponge_absorbs_multi_block_water_pool", rails_and_fluids_extended::test_water_meets_lava_source_forms_obsidian);
        register_test_function("pumpkin:sponge_does_not_absorb_lava", rails_and_fluids_extended::test_sponge_ignores_lava_and_stays_dry);

        // commands_and_entities_extended (11)
        register_test_function("pumpkin:impulse_command_block_setblock_on_power", commands_and_entities_extended::test_command_block_impulse_setblock);
        register_test_function("pumpkin:chain_command_block_three_stage_pipeline", commands_and_entities_extended::test_command_block_chain_three_stages);
        register_test_function("pumpkin:conditional_chain_command_block_skips_on_failure", commands_and_entities_extended::test_command_block_unconditional_vs_conditional);
        register_test_function("pumpkin:auto_command_block_executes_without_redstone", commands_and_entities_extended::test_command_block_auto_executes_on_placement);
        register_test_function("pumpkin:falling_gravel_and_red_sand_land", commands_and_entities_extended::test_falling_gravel_and_red_sand_land);
        register_test_function("pumpkin:falling_concrete_powder_and_anvil_land", commands_and_entities_extended::test_falling_concrete_powder_solidifies_in_water);
        register_test_function("pumpkin:tnt_primed_by_redstone_torch", commands_and_entities_extended::test_tnt_primed_by_redstone_torch_and_wire);
        register_test_function("pumpkin:snow_golem_horizontal_pattern_spawns", commands_and_entities_extended::test_snow_golem_horizontal_pattern_spawns);
        register_test_function("pumpkin:iron_golem_pattern_clears_all_four_blocks", commands_and_entities_extended::test_iron_golem_clears_all_four_pattern_blocks);
        register_test_function("pumpkin:different_item_entities_do_not_merge", commands_and_entities_extended::test_item_entities_different_types_do_not_merge);
        register_test_function("pumpkin:item_entities_stack_up_to_combined_count", commands_and_entities_extended::test_falling_anvil_lands_and_preserves_block);
    });
}

pub(super) async fn build_stone_floor(
    test: &GameTestContext,
    width: i32,
    depth: i32,
) -> GameTestResult<()> {
    for x in 0..width {
        for z in 0..depth {
            test.set_block_default(&BlockPos::new(x, 0, z), &Block::STONE)
                .await?;
        }
    }
    Ok(())
}

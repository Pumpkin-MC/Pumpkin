use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rustc_hash::FxHashSet;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::player::Player;

use super::position_tracker::PositionTracker;
use super::value::{DamageSourceMemory, SpearStatus};
use super::walk_target::WalkTarget;
use super::{
    GlobalPos, MemoryCodec, MemoryModuleId, MemoryModuleType, NearestVisibleLivingEntities, codec,
};

macro_rules! memory_module_types {
    ($($id:literal $konst:ident : $ty:ty = $name:literal, $codec:expr;)*) => {
        $(pub const $konst: MemoryModuleType<$ty> = MemoryModuleType::new($id);)*

        pub const MEMORY_TYPE_COUNT: usize = [$($id),*].len();

        static NAMES: [&str; MEMORY_TYPE_COUNT] = [$($name),*];

        static CODECS: [Option<&'static MemoryCodec>; MEMORY_TYPE_COUNT] = [$($codec),*];

        #[cfg(test)]
        static DECLARED_IDS: [u8; MEMORY_TYPE_COUNT] = [$($id),*];

        #[must_use]
        pub fn from_name(name: &str) -> Option<MemoryModuleId> {
            match name {
                $($name => Some(MemoryModuleId::new($id)),)*
                _ => None,
            }
        }
    };
}

memory_module_types! {
    0 DUMMY: () = "minecraft:dummy", None;
    1 HOME: GlobalPos = "minecraft:home", Some(&codec::GLOBAL_POS);
    2 JOB_SITE: GlobalPos = "minecraft:job_site", Some(&codec::GLOBAL_POS);
    3 POTENTIAL_JOB_SITE: GlobalPos = "minecraft:potential_job_site", Some(&codec::GLOBAL_POS);
    4 MEETING_POINT: GlobalPos = "minecraft:meeting_point", Some(&codec::GLOBAL_POS);
    5 SECONDARY_JOB_SITE: Vec<GlobalPos> = "minecraft:secondary_job_site", None;
    6 NEAREST_LIVING_ENTITIES: Vec<Arc<dyn EntityBase>> = "minecraft:mobs", None;
    7 NEAREST_VISIBLE_LIVING_ENTITIES: NearestVisibleLivingEntities = "minecraft:visible_mobs", None;
    8 VISIBLE_VILLAGER_BABIES: Vec<Arc<dyn EntityBase>> = "minecraft:visible_villager_babies", None;
    9 NEAREST_PLAYERS: Vec<Arc<Player>> = "minecraft:nearest_players", None;
    10 NEAREST_VISIBLE_PLAYER: Arc<Player> = "minecraft:nearest_visible_player", None;
    11 NEAREST_VISIBLE_ATTACKABLE_PLAYER: Arc<Player> = "minecraft:nearest_visible_targetable_player", None;
    12 NEAREST_VISIBLE_ATTACKABLE_PLAYERS: Vec<Arc<Player>> = "minecraft:nearest_visible_targetable_players", None;
    13 WALK_TARGET: WalkTarget = "minecraft:walk_target", None;
    14 LOOK_TARGET: Arc<dyn PositionTracker> = "minecraft:look_target", None;
    15 ATTACK_TARGET: Arc<dyn EntityBase> = "minecraft:attack_target", None;
    16 ATTACK_COOLING_DOWN: bool = "minecraft:attack_cooling_down", None;
    17 INTERACTION_TARGET: Arc<dyn EntityBase> = "minecraft:interaction_target", None;
    18 BREED_TARGET: Arc<dyn EntityBase> = "minecraft:breed_target", None;
    19 RIDE_TARGET: Arc<dyn EntityBase> = "minecraft:ride_target", None;
    20 PATH: Path = "minecraft:path", None;
    21 DOORS_TO_CLOSE: FxHashSet<GlobalPos> = "minecraft:doors_to_close", None;
    22 NEAREST_BED: BlockPos = "minecraft:nearest_bed", None;
    23 HURT_BY: DamageSourceMemory = "minecraft:hurt_by", None;
    24 HURT_BY_ENTITY: Arc<dyn EntityBase> = "minecraft:hurt_by_entity", None;
    25 AVOID_TARGET: Arc<dyn EntityBase> = "minecraft:avoid_target", None;
    26 NEAREST_HOSTILE: Arc<dyn EntityBase> = "minecraft:nearest_hostile", None;
    27 NEAREST_ATTACKABLE: Arc<dyn EntityBase> = "minecraft:nearest_attackable", None;
    28 HIDING_PLACE: GlobalPos = "minecraft:hiding_place", None;
    29 HEARD_BELL_TIME: i64 = "minecraft:heard_bell_time", None;
    30 CANT_REACH_WALK_TARGET_SINCE: i64 = "minecraft:cant_reach_walk_target_since", None;
    31 GOLEM_DETECTED_RECENTLY: bool = "minecraft:golem_detected_recently", Some(&codec::BOOL);
    32 DANGER_DETECTED_RECENTLY: bool = "minecraft:danger_detected_recently", Some(&codec::BOOL);
    33 LAST_SLEPT: i64 = "minecraft:last_slept", Some(&codec::LONG);
    34 LAST_WOKEN: i64 = "minecraft:last_woken", Some(&codec::LONG);
    35 LAST_WORKED_AT_POI: i64 = "minecraft:last_worked_at_poi", Some(&codec::LONG);
    36 NEAREST_VISIBLE_ADULT: Arc<dyn EntityBase> = "minecraft:nearest_visible_adult", None;
    37 NEAREST_VISIBLE_WANTED_ITEM: Arc<dyn EntityBase> = "minecraft:nearest_visible_wanted_item", None;
    38 NEAREST_VISIBLE_NEMESIS: Arc<dyn EntityBase> = "minecraft:nearest_visible_nemesis", None;
    39 PLAY_DEAD_TICKS: i32 = "minecraft:play_dead_ticks", Some(&codec::INT);
    40 TEMPTING_PLAYER: Arc<Player> = "minecraft:tempting_player", None;
    41 TEMPTATION_COOLDOWN_TICKS: i32 = "minecraft:temptation_cooldown_ticks", Some(&codec::INT);
    42 GAZE_COOLDOWN_TICKS: i32 = "minecraft:gaze_cooldown_ticks", Some(&codec::INT);
    43 IS_TEMPTED: bool = "minecraft:is_tempted", Some(&codec::BOOL);
    44 LONG_JUMP_COOLDOWN_TICKS: i32 = "minecraft:long_jump_cooling_down", Some(&codec::INT);
    45 LONG_JUMP_MID_JUMP: bool = "minecraft:long_jump_mid_jump", None;
    46 HAS_HUNTING_COOLDOWN: bool = "minecraft:has_hunting_cooldown", Some(&codec::BOOL);
    47 RAM_COOLDOWN_TICKS: i32 = "minecraft:ram_cooldown_ticks", Some(&codec::INT);
    48 RAM_TARGET: Vector3<f64> = "minecraft:ram_target", None;
    49 IS_IN_WATER: () = "minecraft:is_in_water", Some(&codec::UNIT);
    50 IS_PREGNANT: () = "minecraft:is_pregnant", Some(&codec::UNIT);
    51 IS_PANICKING: bool = "minecraft:is_panicking", Some(&codec::BOOL);
    52 UNREACHABLE_TONGUE_TARGETS: Vec<Uuid> = "minecraft:unreachable_tongue_targets", None;
    53 VISITED_BLOCK_POSITIONS: FxHashSet<GlobalPos> = "minecraft:visited_block_positions", Some(&codec::GLOBAL_POS_SET);
    54 UNREACHABLE_TRANSPORT_BLOCK_POSITIONS: FxHashSet<GlobalPos> = "minecraft:unreachable_transport_block_positions", Some(&codec::GLOBAL_POS_SET);
    55 TRANSPORT_ITEMS_COOLDOWN_TICKS: i32 = "minecraft:transport_items_cooldown_ticks", None;
    56 CHARGE_COOLDOWN_TICKS: i32 = "minecraft:charge_cooldown_ticks", Some(&codec::INT);
    57 ATTACK_TARGET_COOLDOWN: i32 = "minecraft:attack_target_cooldown", Some(&codec::INT);
    58 SPEAR_FLEEING_TIME: i32 = "minecraft:spear_fleeing_time", None;
    59 SPEAR_FLEEING_POSITION: Vector3<f64> = "minecraft:spear_fleeing_position", None;
    60 SPEAR_CHARGE_POSITION: Vector3<f64> = "minecraft:spear_charge_position", None;
    61 SPEAR_ENGAGE_TIME: i32 = "minecraft:spear_engage_time", None;
    62 SPEAR_STATUS: SpearStatus = "minecraft:spear_status", None;
    63 ANGRY_AT: Uuid = "minecraft:angry_at", Some(&codec::UUID);
    64 UNIVERSAL_ANGER: bool = "minecraft:universal_anger", Some(&codec::BOOL);
    65 ADMIRING_ITEM: bool = "minecraft:admiring_item", Some(&codec::BOOL);
    66 TIME_TRYING_TO_REACH_ADMIRE_ITEM: i32 = "minecraft:time_trying_to_reach_admire_item", None;
    67 DISABLE_WALK_TO_ADMIRE_ITEM: bool = "minecraft:disable_walk_to_admire_item", None;
    68 ADMIRING_DISABLED: bool = "minecraft:admiring_disabled", Some(&codec::BOOL);
    69 HUNTED_RECENTLY: bool = "minecraft:hunted_recently", Some(&codec::BOOL);
    70 CELEBRATE_LOCATION: BlockPos = "minecraft:celebrate_location", None;
    71 DANCING: bool = "minecraft:dancing", None;
    72 NEAREST_VISIBLE_HUNTABLE_HOGLIN: Arc<dyn EntityBase> = "minecraft:nearest_visible_huntable_hoglin", None;
    73 NEAREST_VISIBLE_BABY_HOGLIN: Arc<dyn EntityBase> = "minecraft:nearest_visible_baby_hoglin", None;
    74 NEAREST_TARGETABLE_PLAYER_NOT_WEARING_GOLD: Arc<Player> = "minecraft:nearest_targetable_player_not_wearing_gold", None;
    75 NEARBY_ADULT_PIGLINS: Vec<Arc<dyn EntityBase>> = "minecraft:nearby_adult_piglins", None;
    76 NEAREST_VISIBLE_ADULT_PIGLINS: Vec<Arc<dyn EntityBase>> = "minecraft:nearest_visible_adult_piglins", None;
    77 NEAREST_VISIBLE_ADULT_HOGLINS: Vec<Arc<dyn EntityBase>> = "minecraft:nearest_visible_adult_hoglins", None;
    78 NEAREST_VISIBLE_ADULT_PIGLIN: Arc<dyn EntityBase> = "minecraft:nearest_visible_adult_piglin", None;
    79 NEAREST_VISIBLE_ZOMBIFIED: Arc<dyn EntityBase> = "minecraft:nearest_visible_zombified", None;
    80 VISIBLE_ADULT_PIGLIN_COUNT: i32 = "minecraft:visible_adult_piglin_count", None;
    81 VISIBLE_ADULT_HOGLIN_COUNT: i32 = "minecraft:visible_adult_hoglin_count", None;
    82 NEAREST_PLAYER_HOLDING_WANTED_ITEM: Arc<Player> = "minecraft:nearest_player_holding_wanted_item", None;
    83 ATE_RECENTLY: bool = "minecraft:ate_recently", None;
    84 NEAREST_REPELLENT: BlockPos = "minecraft:nearest_repellent", None;
    85 PACIFIED: bool = "minecraft:pacified", None;
    86 ROAR_TARGET: Arc<dyn EntityBase> = "minecraft:roar_target", None;
    87 DISTURBANCE_LOCATION: BlockPos = "minecraft:disturbance_location", None;
    88 RECENT_PROJECTILE: () = "minecraft:recent_projectile", Some(&codec::UNIT);
    89 IS_SNIFFING: () = "minecraft:is_sniffing", Some(&codec::UNIT);
    90 IS_EMERGING: () = "minecraft:is_emerging", Some(&codec::UNIT);
    91 ROAR_SOUND_DELAY: () = "minecraft:roar_sound_delay", Some(&codec::UNIT);
    92 DIG_COOLDOWN: () = "minecraft:dig_cooldown", Some(&codec::UNIT);
    93 ROAR_SOUND_COOLDOWN: () = "minecraft:roar_sound_cooldown", Some(&codec::UNIT);
    94 SNIFF_COOLDOWN: () = "minecraft:sniff_cooldown", Some(&codec::UNIT);
    95 TOUCH_COOLDOWN: () = "minecraft:touch_cooldown", Some(&codec::UNIT);
    96 VIBRATION_COOLDOWN: () = "minecraft:vibration_cooldown", Some(&codec::UNIT);
    97 SONIC_BOOM_COOLDOWN: () = "minecraft:sonic_boom_cooldown", Some(&codec::UNIT);
    98 SONIC_BOOM_SOUND_COOLDOWN: () = "minecraft:sonic_boom_sound_cooldown", Some(&codec::UNIT);
    99 SONIC_BOOM_SOUND_DELAY: () = "minecraft:sonic_boom_sound_delay", Some(&codec::UNIT);
    100 LIKED_PLAYER: Uuid = "minecraft:liked_player", Some(&codec::UUID);
    101 LIKED_NOTEBLOCK_POSITION: GlobalPos = "minecraft:liked_noteblock", Some(&codec::GLOBAL_POS);
    102 LIKED_NOTEBLOCK_COOLDOWN_TICKS: i32 = "minecraft:liked_noteblock_cooldown_ticks", Some(&codec::INT);
    103 ITEM_PICKUP_COOLDOWN_TICKS: i32 = "minecraft:item_pickup_cooldown_ticks", Some(&codec::INT);
    104 SNIFFER_EXPLORED_POSITIONS: Vec<GlobalPos> = "minecraft:sniffer_explored_positions", Some(&codec::GLOBAL_POS_LIST);
    105 SNIFFER_SNIFFING_TARGET: BlockPos = "minecraft:sniffer_sniffing_target", None;
    106 SNIFFER_DIGGING: bool = "minecraft:sniffer_digging", None;
    107 SNIFFER_HAPPY: bool = "minecraft:sniffer_happy", None;
    108 BREEZE_JUMP_COOLDOWN: () = "minecraft:breeze_jump_cooldown", Some(&codec::UNIT);
    109 BREEZE_SHOOT: () = "minecraft:breeze_shoot", Some(&codec::UNIT);
    110 BREEZE_SHOOT_CHARGING: () = "minecraft:breeze_shoot_charging", Some(&codec::UNIT);
    111 BREEZE_SHOOT_RECOVERING: () = "minecraft:breeze_shoot_recover", Some(&codec::UNIT);
    112 BREEZE_SHOOT_COOLDOWN: () = "minecraft:breeze_shoot_cooldown", Some(&codec::UNIT);
    113 BREEZE_JUMP_INHALING: () = "minecraft:breeze_jump_inhaling", Some(&codec::UNIT);
    114 BREEZE_JUMP_TARGET: BlockPos = "minecraft:breeze_jump_target", Some(&codec::BLOCK_POS);
    115 BREEZE_LEAVING_WATER: () = "minecraft:breeze_leaving_water", Some(&codec::UNIT);
}

#[must_use]
pub fn name_of(id: MemoryModuleId) -> &'static str {
    NAMES
        .get(id.index())
        .copied()
        .unwrap_or("minecraft:unknown")
}

#[must_use]
pub fn codec_of(id: MemoryModuleId) -> Option<&'static MemoryCodec> {
    CODECS.get(id.index()).copied().flatten()
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashSet;

    use super::*;

    #[test]
    fn registry_has_every_vanilla_type_in_declaration_order() {
        assert_eq!(MEMORY_TYPE_COUNT, 116);
        for (index, declared) in DECLARED_IDS.iter().enumerate() {
            assert_eq!(usize::from(*declared), index);
        }
    }

    #[test]
    fn registry_names_are_unique_and_resolve_back_to_their_id() {
        let unique: FxHashSet<&str> = NAMES.iter().copied().collect();
        assert_eq!(unique.len(), MEMORY_TYPE_COUNT);
        for index in 0..MEMORY_TYPE_COUNT {
            let id = MemoryModuleId::new(index as u8);
            assert_eq!(from_name(name_of(id)), Some(id));
        }
    }

    #[test]
    fn registry_constants_match_their_table_entry() {
        assert_eq!(DUMMY.id(), MemoryModuleId::new(0));
        assert_eq!(name_of(ADMIRING_ITEM.id()), "minecraft:admiring_item");
        assert_eq!(
            name_of(BREEZE_LEAVING_WATER.id()),
            "minecraft:breeze_leaving_water"
        );
        assert!(ADMIRING_ITEM.can_serialize());
        assert!(!DANCING.can_serialize());
    }

    #[test]
    fn persistent_type_count_matches_vanilla() {
        let persistent = CODECS.iter().filter(|codec| codec.is_some()).count();
        assert_eq!(persistent, 53);
    }
}

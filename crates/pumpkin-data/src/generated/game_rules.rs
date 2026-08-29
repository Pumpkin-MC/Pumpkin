/* This file is generated. Do not edit manually. */
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameRule {
    AdvanceTime,
    AdvanceWeather,
    AllowEnteringNetherUsingPortals,
    BlockDrops,
    BlockExplosionDropDecay,
    CommandBlockOutput,
    CommandBlocksWork,
    DrowningDamage,
    ElytraMovementCheck,
    EnderPearlsVanishOnDeath,
    EntityDrops,
    FallDamage,
    FireDamage,
    FireSpreadRadiusAroundPlayer,
    ForgiveDeadPlayers,
    FreezeDamage,
    GlobalSoundEvents,
    ImmediateRespawn,
    KeepInventory,
    LavaSourceConversion,
    LimitedCrafting,
    LocatorBar,
    LogAdminCommands,
    MaxBlockModifications,
    MaxCommandForks,
    MaxCommandSequenceLength,
    MaxEntityCramming,
    MaxMinecartSpeed,
    MaxSnowAccumulationHeight,
    MobDrops,
    MobExplosionDropDecay,
    MobGriefing,
    NaturalHealthRegeneration,
    PlayerMovementCheck,
    PlayersNetherPortalCreativeDelay,
    PlayersNetherPortalDefaultDelay,
    PlayersSleepingPercentage,
    ProjectilesCanBreakBlocks,
    Pvp,
    Raids,
    RandomTickSpeed,
    ReducedDebugInfo,
    RespawnRadius,
    SendCommandFeedback,
    ShowAdvancementMessages,
    ShowDeathMessages,
    SpawnMobs,
    SpawnMonsters,
    SpawnPatrols,
    SpawnPhantoms,
    SpawnWanderingTraders,
    SpawnWardens,
    SpawnerBlocksWork,
    SpectatorsGenerateChunks,
    SpreadVines,
    TntExplodes,
    TntExplosionDropDecay,
    UniversalAnger,
    WaterSourceConversion,
}
impl GameRule {
    pub const fn all() -> &'static [Self] {
        &[
            Self::AdvanceTime,
            Self::AdvanceWeather,
            Self::AllowEnteringNetherUsingPortals,
            Self::BlockDrops,
            Self::BlockExplosionDropDecay,
            Self::CommandBlockOutput,
            Self::CommandBlocksWork,
            Self::DrowningDamage,
            Self::ElytraMovementCheck,
            Self::EnderPearlsVanishOnDeath,
            Self::EntityDrops,
            Self::FallDamage,
            Self::FireDamage,
            Self::FireSpreadRadiusAroundPlayer,
            Self::ForgiveDeadPlayers,
            Self::FreezeDamage,
            Self::GlobalSoundEvents,
            Self::ImmediateRespawn,
            Self::KeepInventory,
            Self::LavaSourceConversion,
            Self::LimitedCrafting,
            Self::LocatorBar,
            Self::LogAdminCommands,
            Self::MaxBlockModifications,
            Self::MaxCommandForks,
            Self::MaxCommandSequenceLength,
            Self::MaxEntityCramming,
            Self::MaxMinecartSpeed,
            Self::MaxSnowAccumulationHeight,
            Self::MobDrops,
            Self::MobExplosionDropDecay,
            Self::MobGriefing,
            Self::NaturalHealthRegeneration,
            Self::PlayerMovementCheck,
            Self::PlayersNetherPortalCreativeDelay,
            Self::PlayersNetherPortalDefaultDelay,
            Self::PlayersSleepingPercentage,
            Self::ProjectilesCanBreakBlocks,
            Self::Pvp,
            Self::Raids,
            Self::RandomTickSpeed,
            Self::ReducedDebugInfo,
            Self::RespawnRadius,
            Self::SendCommandFeedback,
            Self::ShowAdvancementMessages,
            Self::ShowDeathMessages,
            Self::SpawnMobs,
            Self::SpawnMonsters,
            Self::SpawnPatrols,
            Self::SpawnPhantoms,
            Self::SpawnWanderingTraders,
            Self::SpawnWardens,
            Self::SpawnerBlocksWork,
            Self::SpectatorsGenerateChunks,
            Self::SpreadVines,
            Self::TntExplodes,
            Self::TntExplosionDropDecay,
            Self::UniversalAnger,
            Self::WaterSourceConversion,
        ]
    }
}
impl fmt::Display for GameRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AdvanceTime => write!(f, "advance_time"),
            Self::AdvanceWeather => write!(f, "advance_weather"),
            Self::AllowEnteringNetherUsingPortals => {
                write!(f, "allow_entering_nether_using_portals")
            }
            Self::BlockDrops => write!(f, "block_drops"),
            Self::BlockExplosionDropDecay => write!(f, "block_explosion_drop_decay"),
            Self::CommandBlockOutput => write!(f, "command_block_output"),
            Self::CommandBlocksWork => write!(f, "command_blocks_work"),
            Self::DrowningDamage => write!(f, "drowning_damage"),
            Self::ElytraMovementCheck => write!(f, "elytra_movement_check"),
            Self::EnderPearlsVanishOnDeath => write!(f, "ender_pearls_vanish_on_death"),
            Self::EntityDrops => write!(f, "entity_drops"),
            Self::FallDamage => write!(f, "fall_damage"),
            Self::FireDamage => write!(f, "fire_damage"),
            Self::FireSpreadRadiusAroundPlayer => write!(f, "fire_spread_radius_around_player"),
            Self::ForgiveDeadPlayers => write!(f, "forgive_dead_players"),
            Self::FreezeDamage => write!(f, "freeze_damage"),
            Self::GlobalSoundEvents => write!(f, "global_sound_events"),
            Self::ImmediateRespawn => write!(f, "immediate_respawn"),
            Self::KeepInventory => write!(f, "keep_inventory"),
            Self::LavaSourceConversion => write!(f, "lava_source_conversion"),
            Self::LimitedCrafting => write!(f, "limited_crafting"),
            Self::LocatorBar => write!(f, "locator_bar"),
            Self::LogAdminCommands => write!(f, "log_admin_commands"),
            Self::MaxBlockModifications => write!(f, "max_block_modifications"),
            Self::MaxCommandForks => write!(f, "max_command_forks"),
            Self::MaxCommandSequenceLength => write!(f, "max_command_sequence_length"),
            Self::MaxEntityCramming => write!(f, "max_entity_cramming"),
            Self::MaxMinecartSpeed => write!(f, "max_minecart_speed"),
            Self::MaxSnowAccumulationHeight => write!(f, "max_snow_accumulation_height"),
            Self::MobDrops => write!(f, "mob_drops"),
            Self::MobExplosionDropDecay => write!(f, "mob_explosion_drop_decay"),
            Self::MobGriefing => write!(f, "mob_griefing"),
            Self::NaturalHealthRegeneration => write!(f, "natural_health_regeneration"),
            Self::PlayerMovementCheck => write!(f, "player_movement_check"),
            Self::PlayersNetherPortalCreativeDelay => {
                write!(f, "players_nether_portal_creative_delay")
            }
            Self::PlayersNetherPortalDefaultDelay => {
                write!(f, "players_nether_portal_default_delay")
            }
            Self::PlayersSleepingPercentage => write!(f, "players_sleeping_percentage"),
            Self::ProjectilesCanBreakBlocks => write!(f, "projectiles_can_break_blocks"),
            Self::Pvp => write!(f, "pvp"),
            Self::Raids => write!(f, "raids"),
            Self::RandomTickSpeed => write!(f, "random_tick_speed"),
            Self::ReducedDebugInfo => write!(f, "reduced_debug_info"),
            Self::RespawnRadius => write!(f, "respawn_radius"),
            Self::SendCommandFeedback => write!(f, "send_command_feedback"),
            Self::ShowAdvancementMessages => write!(f, "show_advancement_messages"),
            Self::ShowDeathMessages => write!(f, "show_death_messages"),
            Self::SpawnMobs => write!(f, "spawn_mobs"),
            Self::SpawnMonsters => write!(f, "spawn_monsters"),
            Self::SpawnPatrols => write!(f, "spawn_patrols"),
            Self::SpawnPhantoms => write!(f, "spawn_phantoms"),
            Self::SpawnWanderingTraders => write!(f, "spawn_wandering_traders"),
            Self::SpawnWardens => write!(f, "spawn_wardens"),
            Self::SpawnerBlocksWork => write!(f, "spawner_blocks_work"),
            Self::SpectatorsGenerateChunks => write!(f, "spectators_generate_chunks"),
            Self::SpreadVines => write!(f, "spread_vines"),
            Self::TntExplodes => write!(f, "tnt_explodes"),
            Self::TntExplosionDropDecay => write!(f, "tnt_explosion_drop_decay"),
            Self::UniversalAnger => write!(f, "universal_anger"),
            Self::WaterSourceConversion => write!(f, "water_source_conversion"),
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct GameRuleRegistry {
    #[serde(rename = "advance_time")]
    #[serde(default = "default_advance_time")]
    #[serde(with = "as_string")]
    pub advance_time: bool,
    #[serde(rename = "advance_weather")]
    #[serde(default = "default_advance_weather")]
    #[serde(with = "as_string")]
    pub advance_weather: bool,
    #[serde(rename = "allow_entering_nether_using_portals")]
    #[serde(default = "default_allow_entering_nether_using_portals")]
    #[serde(with = "as_string")]
    pub allow_entering_nether_using_portals: bool,
    #[serde(rename = "block_drops")]
    #[serde(default = "default_block_drops")]
    #[serde(with = "as_string")]
    pub block_drops: bool,
    #[serde(rename = "block_explosion_drop_decay")]
    #[serde(default = "default_block_explosion_drop_decay")]
    #[serde(with = "as_string")]
    pub block_explosion_drop_decay: bool,
    #[serde(rename = "command_block_output")]
    #[serde(default = "default_command_block_output")]
    #[serde(with = "as_string")]
    pub command_block_output: bool,
    #[serde(rename = "command_blocks_work")]
    #[serde(default = "default_command_blocks_work")]
    #[serde(with = "as_string")]
    pub command_blocks_work: bool,
    #[serde(rename = "drowning_damage")]
    #[serde(default = "default_drowning_damage")]
    #[serde(with = "as_string")]
    pub drowning_damage: bool,
    #[serde(rename = "elytra_movement_check")]
    #[serde(default = "default_elytra_movement_check")]
    #[serde(with = "as_string")]
    pub elytra_movement_check: bool,
    #[serde(rename = "ender_pearls_vanish_on_death")]
    #[serde(default = "default_ender_pearls_vanish_on_death")]
    #[serde(with = "as_string")]
    pub ender_pearls_vanish_on_death: bool,
    #[serde(rename = "entity_drops")]
    #[serde(default = "default_entity_drops")]
    #[serde(with = "as_string")]
    pub entity_drops: bool,
    #[serde(rename = "fall_damage")]
    #[serde(default = "default_fall_damage")]
    #[serde(with = "as_string")]
    pub fall_damage: bool,
    #[serde(rename = "fire_damage")]
    #[serde(default = "default_fire_damage")]
    #[serde(with = "as_string")]
    pub fire_damage: bool,
    #[serde(rename = "fire_spread_radius_around_player")]
    #[serde(default = "default_fire_spread_radius_around_player")]
    #[serde(with = "as_string")]
    pub fire_spread_radius_around_player: i64,
    #[serde(rename = "forgive_dead_players")]
    #[serde(default = "default_forgive_dead_players")]
    #[serde(with = "as_string")]
    pub forgive_dead_players: bool,
    #[serde(rename = "freeze_damage")]
    #[serde(default = "default_freeze_damage")]
    #[serde(with = "as_string")]
    pub freeze_damage: bool,
    #[serde(rename = "global_sound_events")]
    #[serde(default = "default_global_sound_events")]
    #[serde(with = "as_string")]
    pub global_sound_events: bool,
    #[serde(rename = "immediate_respawn")]
    #[serde(default = "default_immediate_respawn")]
    #[serde(with = "as_string")]
    pub immediate_respawn: bool,
    #[serde(rename = "keep_inventory")]
    #[serde(default = "default_keep_inventory")]
    #[serde(with = "as_string")]
    pub keep_inventory: bool,
    #[serde(rename = "lava_source_conversion")]
    #[serde(default = "default_lava_source_conversion")]
    #[serde(with = "as_string")]
    pub lava_source_conversion: bool,
    #[serde(rename = "limited_crafting")]
    #[serde(default = "default_limited_crafting")]
    #[serde(with = "as_string")]
    pub limited_crafting: bool,
    #[serde(rename = "locator_bar")]
    #[serde(default = "default_locator_bar")]
    #[serde(with = "as_string")]
    pub locator_bar: bool,
    #[serde(rename = "log_admin_commands")]
    #[serde(default = "default_log_admin_commands")]
    #[serde(with = "as_string")]
    pub log_admin_commands: bool,
    #[serde(rename = "max_block_modifications")]
    #[serde(default = "default_max_block_modifications")]
    #[serde(with = "as_string")]
    pub max_block_modifications: i64,
    #[serde(rename = "max_command_forks")]
    #[serde(default = "default_max_command_forks")]
    #[serde(with = "as_string")]
    pub max_command_forks: i64,
    #[serde(rename = "max_command_sequence_length")]
    #[serde(default = "default_max_command_sequence_length")]
    #[serde(with = "as_string")]
    pub max_command_sequence_length: i64,
    #[serde(rename = "max_entity_cramming")]
    #[serde(default = "default_max_entity_cramming")]
    #[serde(with = "as_string")]
    pub max_entity_cramming: i64,
    #[serde(rename = "max_minecart_speed")]
    #[serde(default = "default_max_minecart_speed")]
    #[serde(with = "as_string")]
    pub max_minecart_speed: i64,
    #[serde(rename = "max_snow_accumulation_height")]
    #[serde(default = "default_max_snow_accumulation_height")]
    #[serde(with = "as_string")]
    pub max_snow_accumulation_height: i64,
    #[serde(rename = "mob_drops")]
    #[serde(default = "default_mob_drops")]
    #[serde(with = "as_string")]
    pub mob_drops: bool,
    #[serde(rename = "mob_explosion_drop_decay")]
    #[serde(default = "default_mob_explosion_drop_decay")]
    #[serde(with = "as_string")]
    pub mob_explosion_drop_decay: bool,
    #[serde(rename = "mob_griefing")]
    #[serde(default = "default_mob_griefing")]
    #[serde(with = "as_string")]
    pub mob_griefing: bool,
    #[serde(rename = "natural_health_regeneration")]
    #[serde(default = "default_natural_health_regeneration")]
    #[serde(with = "as_string")]
    pub natural_health_regeneration: bool,
    #[serde(rename = "player_movement_check")]
    #[serde(default = "default_player_movement_check")]
    #[serde(with = "as_string")]
    pub player_movement_check: bool,
    #[serde(rename = "players_nether_portal_creative_delay")]
    #[serde(default = "default_players_nether_portal_creative_delay")]
    #[serde(with = "as_string")]
    pub players_nether_portal_creative_delay: i64,
    #[serde(rename = "players_nether_portal_default_delay")]
    #[serde(default = "default_players_nether_portal_default_delay")]
    #[serde(with = "as_string")]
    pub players_nether_portal_default_delay: i64,
    #[serde(rename = "players_sleeping_percentage")]
    #[serde(default = "default_players_sleeping_percentage")]
    #[serde(with = "as_string")]
    pub players_sleeping_percentage: i64,
    #[serde(rename = "projectiles_can_break_blocks")]
    #[serde(default = "default_projectiles_can_break_blocks")]
    #[serde(with = "as_string")]
    pub projectiles_can_break_blocks: bool,
    #[serde(rename = "pvp")]
    #[serde(default = "default_pvp")]
    #[serde(with = "as_string")]
    pub pvp: bool,
    #[serde(rename = "raids")]
    #[serde(default = "default_raids")]
    #[serde(with = "as_string")]
    pub raids: bool,
    #[serde(rename = "random_tick_speed")]
    #[serde(default = "default_random_tick_speed")]
    #[serde(with = "as_string")]
    pub random_tick_speed: i64,
    #[serde(rename = "reduced_debug_info")]
    #[serde(default = "default_reduced_debug_info")]
    #[serde(with = "as_string")]
    pub reduced_debug_info: bool,
    #[serde(rename = "respawn_radius")]
    #[serde(default = "default_respawn_radius")]
    #[serde(with = "as_string")]
    pub respawn_radius: i64,
    #[serde(rename = "send_command_feedback")]
    #[serde(default = "default_send_command_feedback")]
    #[serde(with = "as_string")]
    pub send_command_feedback: bool,
    #[serde(rename = "show_advancement_messages")]
    #[serde(default = "default_show_advancement_messages")]
    #[serde(with = "as_string")]
    pub show_advancement_messages: bool,
    #[serde(rename = "show_death_messages")]
    #[serde(default = "default_show_death_messages")]
    #[serde(with = "as_string")]
    pub show_death_messages: bool,
    #[serde(rename = "spawn_mobs")]
    #[serde(default = "default_spawn_mobs")]
    #[serde(with = "as_string")]
    pub spawn_mobs: bool,
    #[serde(rename = "spawn_monsters")]
    #[serde(default = "default_spawn_monsters")]
    #[serde(with = "as_string")]
    pub spawn_monsters: bool,
    #[serde(rename = "spawn_patrols")]
    #[serde(default = "default_spawn_patrols")]
    #[serde(with = "as_string")]
    pub spawn_patrols: bool,
    #[serde(rename = "spawn_phantoms")]
    #[serde(default = "default_spawn_phantoms")]
    #[serde(with = "as_string")]
    pub spawn_phantoms: bool,
    #[serde(rename = "spawn_wandering_traders")]
    #[serde(default = "default_spawn_wandering_traders")]
    #[serde(with = "as_string")]
    pub spawn_wandering_traders: bool,
    #[serde(rename = "spawn_wardens")]
    #[serde(default = "default_spawn_wardens")]
    #[serde(with = "as_string")]
    pub spawn_wardens: bool,
    #[serde(rename = "spawner_blocks_work")]
    #[serde(default = "default_spawner_blocks_work")]
    #[serde(with = "as_string")]
    pub spawner_blocks_work: bool,
    #[serde(rename = "spectators_generate_chunks")]
    #[serde(default = "default_spectators_generate_chunks")]
    #[serde(with = "as_string")]
    pub spectators_generate_chunks: bool,
    #[serde(rename = "spread_vines")]
    #[serde(default = "default_spread_vines")]
    #[serde(with = "as_string")]
    pub spread_vines: bool,
    #[serde(rename = "tnt_explodes")]
    #[serde(default = "default_tnt_explodes")]
    #[serde(with = "as_string")]
    pub tnt_explodes: bool,
    #[serde(rename = "tnt_explosion_drop_decay")]
    #[serde(default = "default_tnt_explosion_drop_decay")]
    #[serde(with = "as_string")]
    pub tnt_explosion_drop_decay: bool,
    #[serde(rename = "universal_anger")]
    #[serde(default = "default_universal_anger")]
    #[serde(with = "as_string")]
    pub universal_anger: bool,
    #[serde(rename = "water_source_conversion")]
    #[serde(default = "default_water_source_conversion")]
    #[serde(with = "as_string")]
    pub water_source_conversion: bool,
}
pub enum GameRuleValue<I, B> {
    Int(I),
    Bool(B),
}
impl<I: fmt::Display, B: fmt::Display> fmt::Display for GameRuleValue<I, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
        }
    }
}
impl GameRuleRegistry {
    pub fn get(&self, rule: &GameRule) -> GameRuleValue<&i64, &bool> {
        match rule {
            GameRule::AdvanceTime => GameRuleValue::Bool(&self.advance_time),
            GameRule::AdvanceWeather => GameRuleValue::Bool(&self.advance_weather),
            GameRule::AllowEnteringNetherUsingPortals => {
                GameRuleValue::Bool(&self.allow_entering_nether_using_portals)
            }
            GameRule::BlockDrops => GameRuleValue::Bool(&self.block_drops),
            GameRule::BlockExplosionDropDecay => {
                GameRuleValue::Bool(&self.block_explosion_drop_decay)
            }
            GameRule::CommandBlockOutput => GameRuleValue::Bool(&self.command_block_output),
            GameRule::CommandBlocksWork => GameRuleValue::Bool(&self.command_blocks_work),
            GameRule::DrowningDamage => GameRuleValue::Bool(&self.drowning_damage),
            GameRule::ElytraMovementCheck => GameRuleValue::Bool(&self.elytra_movement_check),
            GameRule::EnderPearlsVanishOnDeath => {
                GameRuleValue::Bool(&self.ender_pearls_vanish_on_death)
            }
            GameRule::EntityDrops => GameRuleValue::Bool(&self.entity_drops),
            GameRule::FallDamage => GameRuleValue::Bool(&self.fall_damage),
            GameRule::FireDamage => GameRuleValue::Bool(&self.fire_damage),
            GameRule::FireSpreadRadiusAroundPlayer => {
                GameRuleValue::Int(&self.fire_spread_radius_around_player)
            }
            GameRule::ForgiveDeadPlayers => GameRuleValue::Bool(&self.forgive_dead_players),
            GameRule::FreezeDamage => GameRuleValue::Bool(&self.freeze_damage),
            GameRule::GlobalSoundEvents => GameRuleValue::Bool(&self.global_sound_events),
            GameRule::ImmediateRespawn => GameRuleValue::Bool(&self.immediate_respawn),
            GameRule::KeepInventory => GameRuleValue::Bool(&self.keep_inventory),
            GameRule::LavaSourceConversion => GameRuleValue::Bool(&self.lava_source_conversion),
            GameRule::LimitedCrafting => GameRuleValue::Bool(&self.limited_crafting),
            GameRule::LocatorBar => GameRuleValue::Bool(&self.locator_bar),
            GameRule::LogAdminCommands => GameRuleValue::Bool(&self.log_admin_commands),
            GameRule::MaxBlockModifications => GameRuleValue::Int(&self.max_block_modifications),
            GameRule::MaxCommandForks => GameRuleValue::Int(&self.max_command_forks),
            GameRule::MaxCommandSequenceLength => {
                GameRuleValue::Int(&self.max_command_sequence_length)
            }
            GameRule::MaxEntityCramming => GameRuleValue::Int(&self.max_entity_cramming),
            GameRule::MaxMinecartSpeed => GameRuleValue::Int(&self.max_minecart_speed),
            GameRule::MaxSnowAccumulationHeight => {
                GameRuleValue::Int(&self.max_snow_accumulation_height)
            }
            GameRule::MobDrops => GameRuleValue::Bool(&self.mob_drops),
            GameRule::MobExplosionDropDecay => GameRuleValue::Bool(&self.mob_explosion_drop_decay),
            GameRule::MobGriefing => GameRuleValue::Bool(&self.mob_griefing),
            GameRule::NaturalHealthRegeneration => {
                GameRuleValue::Bool(&self.natural_health_regeneration)
            }
            GameRule::PlayerMovementCheck => GameRuleValue::Bool(&self.player_movement_check),
            GameRule::PlayersNetherPortalCreativeDelay => {
                GameRuleValue::Int(&self.players_nether_portal_creative_delay)
            }
            GameRule::PlayersNetherPortalDefaultDelay => {
                GameRuleValue::Int(&self.players_nether_portal_default_delay)
            }
            GameRule::PlayersSleepingPercentage => {
                GameRuleValue::Int(&self.players_sleeping_percentage)
            }
            GameRule::ProjectilesCanBreakBlocks => {
                GameRuleValue::Bool(&self.projectiles_can_break_blocks)
            }
            GameRule::Pvp => GameRuleValue::Bool(&self.pvp),
            GameRule::Raids => GameRuleValue::Bool(&self.raids),
            GameRule::RandomTickSpeed => GameRuleValue::Int(&self.random_tick_speed),
            GameRule::ReducedDebugInfo => GameRuleValue::Bool(&self.reduced_debug_info),
            GameRule::RespawnRadius => GameRuleValue::Int(&self.respawn_radius),
            GameRule::SendCommandFeedback => GameRuleValue::Bool(&self.send_command_feedback),
            GameRule::ShowAdvancementMessages => {
                GameRuleValue::Bool(&self.show_advancement_messages)
            }
            GameRule::ShowDeathMessages => GameRuleValue::Bool(&self.show_death_messages),
            GameRule::SpawnMobs => GameRuleValue::Bool(&self.spawn_mobs),
            GameRule::SpawnMonsters => GameRuleValue::Bool(&self.spawn_monsters),
            GameRule::SpawnPatrols => GameRuleValue::Bool(&self.spawn_patrols),
            GameRule::SpawnPhantoms => GameRuleValue::Bool(&self.spawn_phantoms),
            GameRule::SpawnWanderingTraders => GameRuleValue::Bool(&self.spawn_wandering_traders),
            GameRule::SpawnWardens => GameRuleValue::Bool(&self.spawn_wardens),
            GameRule::SpawnerBlocksWork => GameRuleValue::Bool(&self.spawner_blocks_work),
            GameRule::SpectatorsGenerateChunks => {
                GameRuleValue::Bool(&self.spectators_generate_chunks)
            }
            GameRule::SpreadVines => GameRuleValue::Bool(&self.spread_vines),
            GameRule::TntExplodes => GameRuleValue::Bool(&self.tnt_explodes),
            GameRule::TntExplosionDropDecay => GameRuleValue::Bool(&self.tnt_explosion_drop_decay),
            GameRule::UniversalAnger => GameRuleValue::Bool(&self.universal_anger),
            GameRule::WaterSourceConversion => GameRuleValue::Bool(&self.water_source_conversion),
        }
    }
    pub fn get_mut(&mut self, rule: &GameRule) -> GameRuleValue<&mut i64, &mut bool> {
        match rule {
            GameRule::AdvanceTime => GameRuleValue::Bool(&mut self.advance_time),
            GameRule::AdvanceWeather => GameRuleValue::Bool(&mut self.advance_weather),
            GameRule::AllowEnteringNetherUsingPortals => {
                GameRuleValue::Bool(&mut self.allow_entering_nether_using_portals)
            }
            GameRule::BlockDrops => GameRuleValue::Bool(&mut self.block_drops),
            GameRule::BlockExplosionDropDecay => {
                GameRuleValue::Bool(&mut self.block_explosion_drop_decay)
            }
            GameRule::CommandBlockOutput => GameRuleValue::Bool(&mut self.command_block_output),
            GameRule::CommandBlocksWork => GameRuleValue::Bool(&mut self.command_blocks_work),
            GameRule::DrowningDamage => GameRuleValue::Bool(&mut self.drowning_damage),
            GameRule::ElytraMovementCheck => GameRuleValue::Bool(&mut self.elytra_movement_check),
            GameRule::EnderPearlsVanishOnDeath => {
                GameRuleValue::Bool(&mut self.ender_pearls_vanish_on_death)
            }
            GameRule::EntityDrops => GameRuleValue::Bool(&mut self.entity_drops),
            GameRule::FallDamage => GameRuleValue::Bool(&mut self.fall_damage),
            GameRule::FireDamage => GameRuleValue::Bool(&mut self.fire_damage),
            GameRule::FireSpreadRadiusAroundPlayer => {
                GameRuleValue::Int(&mut self.fire_spread_radius_around_player)
            }
            GameRule::ForgiveDeadPlayers => GameRuleValue::Bool(&mut self.forgive_dead_players),
            GameRule::FreezeDamage => GameRuleValue::Bool(&mut self.freeze_damage),
            GameRule::GlobalSoundEvents => GameRuleValue::Bool(&mut self.global_sound_events),
            GameRule::ImmediateRespawn => GameRuleValue::Bool(&mut self.immediate_respawn),
            GameRule::KeepInventory => GameRuleValue::Bool(&mut self.keep_inventory),
            GameRule::LavaSourceConversion => GameRuleValue::Bool(&mut self.lava_source_conversion),
            GameRule::LimitedCrafting => GameRuleValue::Bool(&mut self.limited_crafting),
            GameRule::LocatorBar => GameRuleValue::Bool(&mut self.locator_bar),
            GameRule::LogAdminCommands => GameRuleValue::Bool(&mut self.log_admin_commands),
            GameRule::MaxBlockModifications => {
                GameRuleValue::Int(&mut self.max_block_modifications)
            }
            GameRule::MaxCommandForks => GameRuleValue::Int(&mut self.max_command_forks),
            GameRule::MaxCommandSequenceLength => {
                GameRuleValue::Int(&mut self.max_command_sequence_length)
            }
            GameRule::MaxEntityCramming => GameRuleValue::Int(&mut self.max_entity_cramming),
            GameRule::MaxMinecartSpeed => GameRuleValue::Int(&mut self.max_minecart_speed),
            GameRule::MaxSnowAccumulationHeight => {
                GameRuleValue::Int(&mut self.max_snow_accumulation_height)
            }
            GameRule::MobDrops => GameRuleValue::Bool(&mut self.mob_drops),
            GameRule::MobExplosionDropDecay => {
                GameRuleValue::Bool(&mut self.mob_explosion_drop_decay)
            }
            GameRule::MobGriefing => GameRuleValue::Bool(&mut self.mob_griefing),
            GameRule::NaturalHealthRegeneration => {
                GameRuleValue::Bool(&mut self.natural_health_regeneration)
            }
            GameRule::PlayerMovementCheck => GameRuleValue::Bool(&mut self.player_movement_check),
            GameRule::PlayersNetherPortalCreativeDelay => {
                GameRuleValue::Int(&mut self.players_nether_portal_creative_delay)
            }
            GameRule::PlayersNetherPortalDefaultDelay => {
                GameRuleValue::Int(&mut self.players_nether_portal_default_delay)
            }
            GameRule::PlayersSleepingPercentage => {
                GameRuleValue::Int(&mut self.players_sleeping_percentage)
            }
            GameRule::ProjectilesCanBreakBlocks => {
                GameRuleValue::Bool(&mut self.projectiles_can_break_blocks)
            }
            GameRule::Pvp => GameRuleValue::Bool(&mut self.pvp),
            GameRule::Raids => GameRuleValue::Bool(&mut self.raids),
            GameRule::RandomTickSpeed => GameRuleValue::Int(&mut self.random_tick_speed),
            GameRule::ReducedDebugInfo => GameRuleValue::Bool(&mut self.reduced_debug_info),
            GameRule::RespawnRadius => GameRuleValue::Int(&mut self.respawn_radius),
            GameRule::SendCommandFeedback => GameRuleValue::Bool(&mut self.send_command_feedback),
            GameRule::ShowAdvancementMessages => {
                GameRuleValue::Bool(&mut self.show_advancement_messages)
            }
            GameRule::ShowDeathMessages => GameRuleValue::Bool(&mut self.show_death_messages),
            GameRule::SpawnMobs => GameRuleValue::Bool(&mut self.spawn_mobs),
            GameRule::SpawnMonsters => GameRuleValue::Bool(&mut self.spawn_monsters),
            GameRule::SpawnPatrols => GameRuleValue::Bool(&mut self.spawn_patrols),
            GameRule::SpawnPhantoms => GameRuleValue::Bool(&mut self.spawn_phantoms),
            GameRule::SpawnWanderingTraders => {
                GameRuleValue::Bool(&mut self.spawn_wandering_traders)
            }
            GameRule::SpawnWardens => GameRuleValue::Bool(&mut self.spawn_wardens),
            GameRule::SpawnerBlocksWork => GameRuleValue::Bool(&mut self.spawner_blocks_work),
            GameRule::SpectatorsGenerateChunks => {
                GameRuleValue::Bool(&mut self.spectators_generate_chunks)
            }
            GameRule::SpreadVines => GameRuleValue::Bool(&mut self.spread_vines),
            GameRule::TntExplodes => GameRuleValue::Bool(&mut self.tnt_explodes),
            GameRule::TntExplosionDropDecay => {
                GameRuleValue::Bool(&mut self.tnt_explosion_drop_decay)
            }
            GameRule::UniversalAnger => GameRuleValue::Bool(&mut self.universal_anger),
            GameRule::WaterSourceConversion => {
                GameRuleValue::Bool(&mut self.water_source_conversion)
            }
        }
    }
}
impl Default for GameRuleRegistry {
    fn default() -> Self {
        Self {
            advance_time: true,
            advance_weather: true,
            allow_entering_nether_using_portals: true,
            block_drops: true,
            block_explosion_drop_decay: true,
            command_block_output: true,
            command_blocks_work: true,
            drowning_damage: true,
            elytra_movement_check: true,
            ender_pearls_vanish_on_death: true,
            entity_drops: true,
            fall_damage: true,
            fire_damage: true,
            fire_spread_radius_around_player: 128i64,
            forgive_dead_players: true,
            freeze_damage: true,
            global_sound_events: true,
            immediate_respawn: false,
            keep_inventory: false,
            lava_source_conversion: false,
            limited_crafting: false,
            locator_bar: true,
            log_admin_commands: true,
            max_block_modifications: 32768i64,
            max_command_forks: 65536i64,
            max_command_sequence_length: 65536i64,
            max_entity_cramming: 24i64,
            max_minecart_speed: 8i64,
            max_snow_accumulation_height: 1i64,
            mob_drops: true,
            mob_explosion_drop_decay: true,
            mob_griefing: true,
            natural_health_regeneration: true,
            player_movement_check: true,
            players_nether_portal_creative_delay: 0i64,
            players_nether_portal_default_delay: 80i64,
            players_sleeping_percentage: 100i64,
            projectiles_can_break_blocks: true,
            pvp: true,
            raids: true,
            random_tick_speed: 3i64,
            reduced_debug_info: false,
            respawn_radius: 10i64,
            send_command_feedback: true,
            show_advancement_messages: true,
            show_death_messages: true,
            spawn_mobs: true,
            spawn_monsters: true,
            spawn_patrols: true,
            spawn_phantoms: true,
            spawn_wandering_traders: true,
            spawn_wardens: true,
            spawner_blocks_work: true,
            spectators_generate_chunks: true,
            spread_vines: true,
            tnt_explodes: true,
            tnt_explosion_drop_decay: false,
            universal_anger: false,
            water_source_conversion: true,
        }
    }
}
fn default_advance_time() -> bool {
    GameRuleRegistry::default().advance_time
}
fn default_advance_weather() -> bool {
    GameRuleRegistry::default().advance_weather
}
fn default_allow_entering_nether_using_portals() -> bool {
    GameRuleRegistry::default().allow_entering_nether_using_portals
}
fn default_block_drops() -> bool {
    GameRuleRegistry::default().block_drops
}
fn default_block_explosion_drop_decay() -> bool {
    GameRuleRegistry::default().block_explosion_drop_decay
}
fn default_command_block_output() -> bool {
    GameRuleRegistry::default().command_block_output
}
fn default_command_blocks_work() -> bool {
    GameRuleRegistry::default().command_blocks_work
}
fn default_drowning_damage() -> bool {
    GameRuleRegistry::default().drowning_damage
}
fn default_elytra_movement_check() -> bool {
    GameRuleRegistry::default().elytra_movement_check
}
fn default_ender_pearls_vanish_on_death() -> bool {
    GameRuleRegistry::default().ender_pearls_vanish_on_death
}
fn default_entity_drops() -> bool {
    GameRuleRegistry::default().entity_drops
}
fn default_fall_damage() -> bool {
    GameRuleRegistry::default().fall_damage
}
fn default_fire_damage() -> bool {
    GameRuleRegistry::default().fire_damage
}
fn default_fire_spread_radius_around_player() -> i64 {
    GameRuleRegistry::default().fire_spread_radius_around_player
}
fn default_forgive_dead_players() -> bool {
    GameRuleRegistry::default().forgive_dead_players
}
fn default_freeze_damage() -> bool {
    GameRuleRegistry::default().freeze_damage
}
fn default_global_sound_events() -> bool {
    GameRuleRegistry::default().global_sound_events
}
fn default_immediate_respawn() -> bool {
    GameRuleRegistry::default().immediate_respawn
}
fn default_keep_inventory() -> bool {
    GameRuleRegistry::default().keep_inventory
}
fn default_lava_source_conversion() -> bool {
    GameRuleRegistry::default().lava_source_conversion
}
fn default_limited_crafting() -> bool {
    GameRuleRegistry::default().limited_crafting
}
fn default_locator_bar() -> bool {
    GameRuleRegistry::default().locator_bar
}
fn default_log_admin_commands() -> bool {
    GameRuleRegistry::default().log_admin_commands
}
fn default_max_block_modifications() -> i64 {
    GameRuleRegistry::default().max_block_modifications
}
fn default_max_command_forks() -> i64 {
    GameRuleRegistry::default().max_command_forks
}
fn default_max_command_sequence_length() -> i64 {
    GameRuleRegistry::default().max_command_sequence_length
}
fn default_max_entity_cramming() -> i64 {
    GameRuleRegistry::default().max_entity_cramming
}
fn default_max_minecart_speed() -> i64 {
    GameRuleRegistry::default().max_minecart_speed
}
fn default_max_snow_accumulation_height() -> i64 {
    GameRuleRegistry::default().max_snow_accumulation_height
}
fn default_mob_drops() -> bool {
    GameRuleRegistry::default().mob_drops
}
fn default_mob_explosion_drop_decay() -> bool {
    GameRuleRegistry::default().mob_explosion_drop_decay
}
fn default_mob_griefing() -> bool {
    GameRuleRegistry::default().mob_griefing
}
fn default_natural_health_regeneration() -> bool {
    GameRuleRegistry::default().natural_health_regeneration
}
fn default_player_movement_check() -> bool {
    GameRuleRegistry::default().player_movement_check
}
fn default_players_nether_portal_creative_delay() -> i64 {
    GameRuleRegistry::default().players_nether_portal_creative_delay
}
fn default_players_nether_portal_default_delay() -> i64 {
    GameRuleRegistry::default().players_nether_portal_default_delay
}
fn default_players_sleeping_percentage() -> i64 {
    GameRuleRegistry::default().players_sleeping_percentage
}
fn default_projectiles_can_break_blocks() -> bool {
    GameRuleRegistry::default().projectiles_can_break_blocks
}
fn default_pvp() -> bool {
    GameRuleRegistry::default().pvp
}
fn default_raids() -> bool {
    GameRuleRegistry::default().raids
}
fn default_random_tick_speed() -> i64 {
    GameRuleRegistry::default().random_tick_speed
}
fn default_reduced_debug_info() -> bool {
    GameRuleRegistry::default().reduced_debug_info
}
fn default_respawn_radius() -> i64 {
    GameRuleRegistry::default().respawn_radius
}
fn default_send_command_feedback() -> bool {
    GameRuleRegistry::default().send_command_feedback
}
fn default_show_advancement_messages() -> bool {
    GameRuleRegistry::default().show_advancement_messages
}
fn default_show_death_messages() -> bool {
    GameRuleRegistry::default().show_death_messages
}
fn default_spawn_mobs() -> bool {
    GameRuleRegistry::default().spawn_mobs
}
fn default_spawn_monsters() -> bool {
    GameRuleRegistry::default().spawn_monsters
}
fn default_spawn_patrols() -> bool {
    GameRuleRegistry::default().spawn_patrols
}
fn default_spawn_phantoms() -> bool {
    GameRuleRegistry::default().spawn_phantoms
}
fn default_spawn_wandering_traders() -> bool {
    GameRuleRegistry::default().spawn_wandering_traders
}
fn default_spawn_wardens() -> bool {
    GameRuleRegistry::default().spawn_wardens
}
fn default_spawner_blocks_work() -> bool {
    GameRuleRegistry::default().spawner_blocks_work
}
fn default_spectators_generate_chunks() -> bool {
    GameRuleRegistry::default().spectators_generate_chunks
}
fn default_spread_vines() -> bool {
    GameRuleRegistry::default().spread_vines
}
fn default_tnt_explodes() -> bool {
    GameRuleRegistry::default().tnt_explodes
}
fn default_tnt_explosion_drop_decay() -> bool {
    GameRuleRegistry::default().tnt_explosion_drop_decay
}
fn default_universal_anger() -> bool {
    GameRuleRegistry::default().universal_anger
}
fn default_water_source_conversion() -> bool {
    GameRuleRegistry::default().water_source_conversion
}
mod as_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::{fmt::Display, str::FromStr};
    pub fn serialize<T: Display, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        D: Deserializer<'de>,
        <T as FromStr>::Err: Display,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<T>().map_err(serde::de::Error::custom)
    }
}

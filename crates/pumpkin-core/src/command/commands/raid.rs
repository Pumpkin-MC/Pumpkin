use std::sync::Arc;
use uuid::Uuid;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::DISPATCHER_PARSE_EXCEPTION;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::entity::mob::raider::create_ominous_banner;
use crate::entity::player::Player;
use crate::entity::r#type::from_type;
use crate::world::raid::RaidStatus;

const DESCRIPTION: &str = "Controls or queries village raids.";
const PERMISSION: &str = "minecraft:command.raid";

struct StartExecutor {
    has_omen_lvl: bool,
}

impl CommandExecutor for StartExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let (is_already_started, raid_created) = {
            let mut raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if raids.get_raid_at(&pos).is_some() {
                (true, None)
            } else {
                let omen_lvl = if self.has_omen_lvl {
                    IntegerArgumentType::get(context, "omenlvl")?
                } else {
                    1
                };
                let raid_id = raids.create_or_extend_raid(pos, &world);
                if let Some(id) = raid_id
                    && let Some(raid) = raids.get_mut(id)
                {
                    raid.set_raid_omen_level(omen_lvl);
                }
                (false, raid_id)
            }
        };
        if is_already_started {
            context
                .source
                .send_feedback(TextComponent::text("Raid already started close by"), false);
            return Ok(0);
        }
        if raid_created.is_some() {
            context.source.send_feedback(
                TextComponent::text("Created a raid in your local village"),
                true,
            );
            Ok(1)
        } else {
            context.source.send_feedback(
                TextComponent::text("Failed to create a raid in your local village"),
                false,
            );
            Ok(0)
        }
    }
}

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let stopped = {
            let mut raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(raid) = raids.get_raid_at_mut(&pos) {
                raid.active = false;
                raid.status = RaidStatus::Stopped;
                let players_to_remove: Vec<Arc<Player>> = world
                    .players
                    .load()
                    .iter()
                    .filter(|p| raid.players_in_raid.contains(&p.gameprofile.id))
                    .cloned()
                    .collect();
                let bossbar_uuid = raid.bossbar.uuid;
                raid.players_in_raid.clear();
                Some((players_to_remove, bossbar_uuid))
            } else {
                None
            }
        };
        if let Some((players, bossbar_uuid)) = stopped {
            for p in players {
                p.remove_bossbar(bossbar_uuid);
            }
            context
                .source
                .send_feedback(TextComponent::text("Stopped raid"), true);
            Ok(1)
        } else {
            context
                .source
                .send_feedback(TextComponent::text("No raid here"), false);
            Ok(0)
        }
    }
}

struct CheckExecutor;

impl CommandExecutor for CheckExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let info = {
            let raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            raids.get_raid_at(&pos).map(|raid| {
                let alive = raid.get_total_raiders_alive();
                let living_health = raid.get_health_of_living_raiders(&world);
                format!(
                    "Num groups spawned: {} Raid omen level: {} Num mobs: {} Raid health: {} / {}",
                    raid.get_groups_spawned(),
                    raid.get_raid_omen_level(),
                    alive,
                    living_health,
                    raid.total_health
                )
            })
        };
        info.map_or_else(
            || {
                context
                    .source
                    .send_feedback(TextComponent::text("Found no started raids"), false);
                Ok(0)
            },
            |msg| {
                context
                    .source
                    .send_feedback(TextComponent::text("Found a started raid!"), false);
                context
                    .source
                    .send_feedback(TextComponent::text(msg), false);
                Ok(1)
            },
        )
    }
}

struct SoundExecutor;

impl CommandExecutor for SoundExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();

        let sound_pos = pos.add_raw(5.0, 0.0, 0.0);
        world.play_sound(
            Sound::EventRaidHorn,
            pumpkin_data::sound::SoundCategory::Neutral,
            &sound_pos,
        );
        Ok(1)
    }
}

struct SpawnLeaderExecutor;

impl CommandExecutor for SpawnLeaderExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();

        let raider_uuid = Uuid::new_v4();
        let raider_entity = from_type(&EntityType::PILLAGER, pos, &world, raider_uuid);

        if let Some(mob) = raider_entity.get_mob()
            && let Some(raider) = mob.as_raider()
        {
            raider.set_patrol_leader(true);
            let banner = create_ominous_banner();
            let living = &mob.get_mob_entity().living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            equipment.put(&EquipmentSlot::HEAD, banner.clone());
            drop(equipment);
            living.send_equipment_changes(&[(EquipmentSlot::HEAD, banner)]);
        }

        world.spawn_entity(raider_entity);
        context
            .source
            .send_feedback(TextComponent::text("Spawned a raid captain"), true);
        Ok(1)
    }
}

struct SetOmenExecutor;

impl CommandExecutor for SetOmenExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let level = IntegerArgumentType::get(context, "level")?;

        let res = {
            let mut raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            raids
                .get_raid_at_mut(&pos)
                .map_or(Err("No raid found here"), |raid| {
                    if level > 5 {
                        Err("Sorry, the max raid omen level you can set is 5")
                    } else {
                        let before = raid.get_raid_omen_level();
                        raid.set_raid_omen_level(level);
                        Ok(before)
                    }
                })
        };
        match res {
            Ok(before) => {
                context.source.send_feedback(
                    TextComponent::text(format!(
                        "Changed village's raid omen level from {before} to {level}"
                    )),
                    true,
                );
                Ok(1)
            }
            Err(msg) => {
                context
                    .source
                    .send_feedback(TextComponent::text(msg), false);
                Ok(0)
            }
        }
    }
}

struct GlowExecutor;

impl CommandExecutor for GlowExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.as_player().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(
                "Only players can execute this command",
            ))
        })?;
        let entity = player.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let raiders = {
            let raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            raids
                .get_raid_at(&pos)
                .map(crate::world::raid::Raid::get_all_raiders)
        };
        raiders.map_or_else(
            || {
                context
                    .source
                    .send_feedback(TextComponent::text("No raid found here"), false);
                Ok(0)
            },
            |raider_uuids| {
                let effect = Effect {
                    effect_type: &StatusEffect::GLOWING,
                    duration: 1000,
                    amplifier: 1,
                    ambient: false,
                    show_particles: false,
                    show_icon: true,
                    blend: true,
                };
                for raider_uuid in raider_uuids {
                    if let Some(e) = world.get_entity_by_uuid(raider_uuid)
                        && let Some(living) = e.get_living_entity()
                    {
                        living.add_effect(effect.clone());
                    }
                }
                Ok(1)
            },
        )
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("raid", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("start")
                    .executes(StartExecutor {
                        has_omen_lvl: false,
                    })
                    .then(
                        argument("omenlvl", IntegerArgumentType::with_min(0))
                            .executes(StartExecutor { has_omen_lvl: true }),
                    ),
            )
            .then(literal("stop").executes(StopExecutor))
            .then(literal("check").executes(CheckExecutor))
            .then(literal("sound").executes(SoundExecutor))
            .then(literal("spawnleader").executes(SpawnLeaderExecutor))
            .then(literal("setomen").then(
                argument("level", IntegerArgumentType::with_min(0)).executes(SetOmenExecutor),
            ))
            .then(literal("glow").executes(GlowExecutor)),
    );
}

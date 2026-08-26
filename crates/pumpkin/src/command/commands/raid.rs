use uuid::Uuid;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::text::TextComponent;

use std::sync::Arc;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use crate::entity::mob::raider::create_ominous_banner;
use crate::entity::player::Player;
use crate::entity::r#type::from_type;
use crate::server::Server;
use crate::world::raid::RaidStatus;

const NAMES: [&str; 1] = ["raid"];
const DESCRIPTION: &str = "Controls or queries village raids.";

const ARG_OMEN_LVL: &str = "omenlvl";
const ARG_LEVEL: &str = "level";

struct StartExecutor {
    has_omen_lvl: bool,
}

impl CommandExecutor for StartExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
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
                    BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_OMEN_LVL)
                        .ok()
                        .and_then(Result::ok)
                        .unwrap_or(1)
                } else {
                    1
                };
                let raid_id = raids.create_or_extend_raid(&player, pos, &world);
                if let Some(id) = raid_id
                    && let Some(raid) = raids.get_mut(id)
                {
                    raid.set_raid_omen_level(omen_lvl);
                }
                (false, raid_id)
            }
        };
        if is_already_started {
            sender.send_message(TextComponent::text("Raid already started close by"));
            return Ok(0);
        }
        if raid_created.is_some() {
            sender.send_message(TextComponent::text("Created a raid in your local village"));
            Ok(1)
        } else {
            sender.send_message(TextComponent::text(
                "Failed to create a raid in your local village",
            ));
            Ok(0)
        }
    }
}

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
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
            sender.send_message(TextComponent::text("Stopped raid"));
            Ok(1)
        } else {
            sender.send_message(TextComponent::text("No raid here"));
            Ok(0)
        }
    }
}

struct CheckExecutor;

impl CommandExecutor for CheckExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
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
                sender.send_message(TextComponent::text("Found no started raids"));
                Ok(0)
            },
            |msg| {
                sender.send_message(TextComponent::text("Found a started raid!"));
                sender.send_message(TextComponent::text(msg));
                Ok(1)
            },
        )
    }
}

struct SoundExecutor;

impl CommandExecutor for SoundExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
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
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
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
        sender.send_message(TextComponent::text("Spawned a raid captain"));
        Ok(1)
    }
}

struct SetOmenExecutor;

impl CommandExecutor for SetOmenExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
        let entity = player.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let level = BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_LEVEL)
            .ok()
            .and_then(Result::ok)
            .unwrap_or(1);

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
                sender.send_message(TextComponent::text(format!(
                    "Changed village's raid omen level from {before} to {level}"
                )));
                Ok(1)
            }
            Err(msg) => {
                sender.send_message(TextComponent::text(msg));
                Ok(0)
            }
        }
    }
}

struct GlowExecutor;

impl CommandExecutor for GlowExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
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
                sender.send_message(TextComponent::text("No raid found here"));
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

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("start")
                .execute(StartExecutor {
                    has_omen_lvl: false,
                })
                .then(
                    argument(
                        ARG_OMEN_LVL,
                        BoundedNumArgumentConsumer::<i32>::new().min(0),
                    )
                    .execute(StartExecutor { has_omen_lvl: true }),
                ),
        )
        .then(literal("stop").execute(StopExecutor))
        .then(literal("check").execute(CheckExecutor))
        .then(literal("sound").execute(SoundExecutor))
        .then(literal("spawnleader").execute(SpawnLeaderExecutor))
        .then(
            literal("setomen").then(
                argument(ARG_LEVEL, BoundedNumArgumentConsumer::<i32>::new().min(0))
                    .execute(SetOmenExecutor),
            ),
        )
        .then(literal("glow").execute(GlowExecutor))
}

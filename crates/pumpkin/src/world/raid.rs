use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::entity::EntityBase;
use crate::entity::mob::raider::create_ominous_banner;
use crate::entity::player::Player;
use crate::entity::r#type::from_type;
use crate::world::World;
use crate::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaidStatus {
    Ongoing,
    Victory,
    Loss,
    Stopped,
}

impl RaidStatus {
    #[must_use]
    pub const fn get_serialized_name(self) -> &'static str {
        match self {
            Self::Ongoing => "ongoing",
            Self::Victory => "victory",
            Self::Loss => "loss",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaiderType {
    Vindicator,
    Evoker,
    Pillager,
    Witch,
    Ravager,
}

impl RaiderType {
    pub const VALUES: [Self; 5] = [
        Self::Vindicator,
        Self::Evoker,
        Self::Pillager,
        Self::Witch,
        Self::Ravager,
    ];

    #[must_use]
    pub const fn entity_type(self) -> &'static EntityType {
        match self {
            Self::Vindicator => &EntityType::VINDICATOR,
            Self::Evoker => &EntityType::EVOKER,
            Self::Pillager => &EntityType::PILLAGER,
            Self::Witch => &EntityType::WITCH,
            Self::Ravager => &EntityType::RAVAGER,
        }
    }

    #[must_use]
    pub const fn spawns_per_wave(self) -> &'static [i32; 8] {
        match self {
            Self::Vindicator => &[0, 0, 2, 0, 1, 4, 2, 5],
            Self::Evoker => &[0, 0, 0, 0, 0, 1, 1, 2],
            Self::Pillager => &[0, 4, 3, 3, 4, 4, 4, 2],
            Self::Witch => &[0, 0, 0, 0, 3, 0, 0, 1],
            Self::Ravager => &[0, 0, 0, 1, 0, 1, 0, 2],
        }
    }
}

pub struct Raid {
    pub id: i32,
    pub center: BlockPos,
    pub status: RaidStatus,
    pub active: bool,
    pub started: bool,
    pub ticks_active: u64,
    pub raid_omen_level: i32,
    pub groups_spawned: i32,
    pub num_groups: i32,
    pub raid_cooldown_ticks: i32,
    pub post_raid_ticks: i32,
    pub celebration_ticks: u32,
    pub total_health: f32,
    pub bossbar: Bossbar,
    pub group_to_leader_map: HashMap<i32, Uuid>,
    pub group_raider_map: HashMap<i32, HashSet<Uuid>>,
    pub heroes_of_the_village: HashSet<Uuid>,
    pub players_in_raid: HashSet<Uuid>,
    pub wave_spawn_pos: Option<BlockPos>,
}

impl Raid {
    pub const VALID_RAID_RADIUS: f64 = 96.0;
    pub const VALID_RAID_RADIUS_SQR: f64 = 9216.0;
    pub const RAID_REMOVAL_THRESHOLD_SQR: f64 = 12544.0;
    pub const MAX_RAID_OMEN_LEVEL: i32 = 5;

    #[must_use]
    pub fn new(id: i32, center: BlockPos, difficulty: Difficulty) -> Self {
        let mut bossbar = Bossbar::new(TextComponent::translate("event.minecraft.raid", []));
        bossbar.color = BossbarColor::Red;
        bossbar.division = BossbarDivisions::Notches10;
        bossbar.health = 0.0;

        let num_groups = Self::get_num_groups(difficulty);

        Self {
            id,
            center,
            status: RaidStatus::Ongoing,
            active: true,
            started: false,
            ticks_active: 0,
            raid_omen_level: 1,
            groups_spawned: 0,
            num_groups,
            raid_cooldown_ticks: 300,
            post_raid_ticks: 0,
            celebration_ticks: 0,
            total_health: 0.0,
            bossbar,
            group_to_leader_map: HashMap::new(),
            group_raider_map: HashMap::new(),
            heroes_of_the_village: HashSet::new(),
            players_in_raid: HashSet::new(),
            wave_spawn_pos: None,
        }
    }

    #[must_use]
    pub const fn get_num_groups(difficulty: Difficulty) -> i32 {
        match difficulty {
            Difficulty::Peaceful => 0,
            Difficulty::Easy => 3,
            Difficulty::Normal => 5,
            Difficulty::Hard => 7,
        }
    }

    #[must_use]
    pub const fn is_over(&self) -> bool {
        self.is_victory() || self.is_loss()
    }

    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        matches!(self.status, RaidStatus::Stopped)
    }

    #[must_use]
    pub const fn is_victory(&self) -> bool {
        matches!(self.status, RaidStatus::Victory)
    }

    #[must_use]
    pub const fn is_loss(&self) -> bool {
        matches!(self.status, RaidStatus::Loss)
    }

    #[must_use]
    pub const fn is_started(&self) -> bool {
        self.started
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub const fn get_groups_spawned(&self) -> i32 {
        self.groups_spawned
    }

    #[must_use]
    pub const fn get_raid_omen_level(&self) -> i32 {
        self.raid_omen_level
    }

    pub fn set_raid_omen_level(&mut self, level: i32) {
        self.raid_omen_level = level.clamp(0, Self::MAX_RAID_OMEN_LEVEL);
    }

    #[must_use]
    pub const fn has_more_waves(&self) -> bool {
        if self.has_bonus_wave() {
            !self.has_spawned_bonus_wave()
        } else {
            !self.is_final_wave()
        }
    }

    #[must_use]
    pub const fn is_final_wave(&self) -> bool {
        self.groups_spawned == self.num_groups
    }

    #[must_use]
    pub const fn has_bonus_wave(&self) -> bool {
        self.raid_omen_level > 1
    }

    #[must_use]
    pub const fn has_spawned_bonus_wave(&self) -> bool {
        self.groups_spawned > self.num_groups
    }

    #[must_use]
    pub fn should_spawn_bonus_group(&self) -> bool {
        self.is_final_wave() && self.get_total_raiders_alive() == 0 && self.has_bonus_wave()
    }

    #[must_use]
    pub fn should_spawn_group(&self) -> bool {
        self.raid_cooldown_ticks == 0
            && (self.groups_spawned < self.num_groups || self.should_spawn_bonus_group())
            && self.get_total_raiders_alive() == 0
    }

    #[must_use]
    pub fn get_total_raiders_alive(&self) -> usize {
        self.group_raider_map.values().map(HashSet::len).sum()
    }

    #[must_use]
    pub fn get_all_raiders(&self) -> HashSet<Uuid> {
        let mut set = HashSet::new();
        for wave_set in self.group_raider_map.values() {
            set.extend(wave_set);
        }
        set
    }

    pub async fn stop(&mut self, world: &World) {
        self.active = false;
        self.status = RaidStatus::Stopped;
        self.remove_all_players(world).await;
    }

    pub async fn remove_all_players(&mut self, world: &World) {
        let players = world.players.load();
        for player in players.iter() {
            if self.players_in_raid.contains(&player.gameprofile.id) {
                player.remove_bossbar(self.bossbar.uuid).await;
            }
        }
        self.players_in_raid.clear();
    }

    pub fn absorb_raid_omen(&mut self, _player: &Player) -> bool {
        self.raid_omen_level = (self.raid_omen_level + 1).clamp(1, Self::MAX_RAID_OMEN_LEVEL);
        true
    }

    pub fn add_hero_of_the_village(&mut self, killer_uuid: Uuid) {
        self.heroes_of_the_village.insert(killer_uuid);
    }

    const fn get_default_num_spawns(
        &self,
        raider_type: RaiderType,
        wave: i32,
        is_bonus_wave: bool,
    ) -> i32 {
        let spawns = raider_type.spawns_per_wave();
        if is_bonus_wave {
            spawns[self.num_groups as usize]
        } else {
            spawns[wave as usize]
        }
    }

    fn get_potential_bonus_spawns(
        raider_type: RaiderType,
        wave: i32,
        difficulty: Difficulty,
        is_bonus_wave: bool,
    ) -> i32 {
        let is_easy = difficulty == Difficulty::Easy;
        let is_normal = difficulty == Difficulty::Normal;
        let bonus_spawns = match raider_type {
            RaiderType::Vindicator | RaiderType::Pillager => {
                if is_easy {
                    rand::random::<u32>() % 2
                } else if is_normal {
                    1
                } else {
                    2
                }
            }
            RaiderType::Evoker => 0,
            RaiderType::Witch => u32::from(!(is_easy || wave <= 2 || wave == 4)),
            RaiderType::Ravager => u32::from(!is_easy && is_bonus_wave),
        };
        if bonus_spawns > 0 {
            (rand::random::<u32>() % (bonus_spawns + 1)) as i32
        } else {
            0
        }
    }

    pub async fn spawn_group(&mut self, world: &Arc<World>, pos: BlockPos) {
        let mut leader_set = false;
        let group_number = self.groups_spawned + 1;
        self.total_health = 0.0;
        let is_bonus_group = self.should_spawn_bonus_group();

        let difficulty = world.level_info.load().difficulty;

        for raider_type in RaiderType::VALUES {
            let num_spawns = self.get_default_num_spawns(raider_type, group_number, is_bonus_group)
                + Self::get_potential_bonus_spawns(
                    raider_type,
                    group_number,
                    difficulty,
                    is_bonus_group,
                );
            let mut ravagers_spawned = 0;

            for _ in 0..num_spawns {
                let uuid = Uuid::new_v4();
                let spawn_pos = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y) + 1.0,
                    f64::from(pos.0.z) + 0.5,
                );
                let entity_base = from_type(raider_type.entity_type(), spawn_pos, world, uuid);

                if let Some(mob) = entity_base.get_mob()
                    && let Some(raider) = mob.as_raider()
                {
                    if !leader_set && raider.can_be_leader() {
                        raider.set_patrol_leader(true);
                        let banner = create_ominous_banner();
                        let living = &mob.get_mob_entity().living_entity;
                        let mut equipment = living.entity_equipment.lock().await;
                        equipment.put(&EquipmentSlot::HEAD, banner.clone());
                        drop(equipment);
                        living.send_equipment_changes(&[(EquipmentSlot::HEAD, banner)]);
                        self.group_to_leader_map.insert(group_number, uuid);
                        leader_set = true;
                    }
                    raider.set_wave(group_number);
                    raider.set_can_join_raid(true);
                    raider.apply_raid_buffs(group_number, false);
                }

                self.join_raid(group_number, uuid, &entity_base);
                world.spawn_entity(entity_base.clone()).await;

                if *raider_type.entity_type() == EntityType::RAVAGER {
                    let mut riding_type: Option<&'static EntityType> = None;
                    if group_number == Self::get_num_groups(Difficulty::Normal) {
                        riding_type = Some(&EntityType::PILLAGER);
                    } else if group_number >= Self::get_num_groups(Difficulty::Hard) {
                        if ravagers_spawned == 0 {
                            riding_type = Some(&EntityType::EVOKER);
                        } else {
                            riding_type = Some(&EntityType::VINDICATOR);
                        }
                    }
                    ravagers_spawned += 1;

                    if let Some(riding_type) = riding_type {
                        let rider_uuid = Uuid::new_v4();
                        let rider_base = from_type(riding_type, spawn_pos, world, rider_uuid);
                        if let Some(mob) = rider_base.get_mob()
                            && let Some(raider) = mob.as_raider()
                        {
                            raider.set_wave(group_number);
                            raider.set_can_join_raid(true);
                            raider.apply_raid_buffs(group_number, false);
                        }
                        self.join_raid(group_number, rider_uuid, &rider_base);
                        world.spawn_entity(rider_base).await;
                    }
                }
            }
        }

        self.wave_spawn_pos = None;
        self.groups_spawned += 1;
        self.update_bossbar(world).await;
    }

    pub fn join_raid(&mut self, wave: i32, uuid: Uuid, entity_base: &Arc<dyn EntityBase>) {
        self.group_raider_map.entry(wave).or_default().insert(uuid);

        if let Some(living) = entity_base.get_living_entity() {
            self.total_health += living.health.load();
        }
    }

    #[must_use]
    pub fn find_random_spawn_pos(&self, _world: &World, max_tries: usize) -> Option<BlockPos> {
        let seconds_remaining = self.raid_cooldown_ticks / 20;
        let how_far = 0.22 * (seconds_remaining as f32) - 0.24;
        let start_angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;

        for i in 0..max_tries {
            let angle = start_angle + std::f32::consts::PI * (i as f32) / 8.0;
            let spawn_x = self.center.0.x
                + ((angle.cos() * 32.0 * how_far) as i32)
                + (rand::random::<i32>().rem_euclid(3)) * (how_far as i32);
            let spawn_z = self.center.0.z
                + ((angle.sin() * 32.0 * how_far) as i32)
                + (rand::random::<i32>().rem_euclid(3)) * (how_far as i32);
            let spawn_y = self.center.0.y;

            if (spawn_y - self.center.0.y).abs() <= 96 {
                let spawn_pos = BlockPos(Vector3::new(spawn_x, spawn_y, spawn_z));
                return Some(spawn_pos);
            }
        }
        None
    }

    pub fn play_sound(&self, world: &World, sound_origin: BlockPos) {
        let raid_loc = sound_origin.to_f64();
        let players = world.players.load();
        for player in players.iter() {
            let player_loc = player.get_entity().pos.load();
            let dx = raid_loc.x - player_loc.x;
            let dz = raid_loc.z - player_loc.z;
            let dist = dx.hypot(dz);
            let sound_pos = if dist > 0.001 {
                Vector3::new(
                    player_loc.x + (13.0 / dist) * dx,
                    player_loc.y,
                    player_loc.z + (13.0 / dist) * dz,
                )
            } else {
                player_loc
            };
            if dist <= 64.0 || self.players_in_raid.contains(&player.gameprofile.id) {
                world.play_sound(
                    Sound::EventRaidHorn,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &sound_pos,
                );
            }
        }
    }

    pub async fn update_players(&mut self, world: &World) {
        let center_f64 = self.center.to_f64();
        let players = world.players.load();
        let mut current_nearby = HashSet::new();

        for player in players.iter() {
            let pos = player.get_entity().pos.load();
            let dist_sq = pos.squared_distance_to_vec(&center_f64);
            if dist_sq <= Self::VALID_RAID_RADIUS_SQR && player.living_entity.health.load() > 0.0 {
                current_nearby.insert(player.gameprofile.id);
                if self.players_in_raid.insert(player.gameprofile.id) {
                    player.send_bossbar(&self.bossbar).await;
                }
            }
        }

        let mut to_remove = Vec::new();
        for player_uuid in &self.players_in_raid {
            if !current_nearby.contains(player_uuid) {
                to_remove.push(*player_uuid);
            }
        }

        for player_uuid in to_remove {
            self.players_in_raid.remove(&player_uuid);
            if let Some(player) = players.iter().find(|p| p.gameprofile.id == player_uuid) {
                player.remove_bossbar(self.bossbar.uuid).await;
            }
        }
    }

    pub fn update_raiders(&mut self, world: &World) {
        let center_f64 = self.center.to_f64();

        for raiders in self.group_raider_map.values_mut() {
            let mut wave_dead = Vec::new();
            for &raider_uuid in raiders.iter() {
                let entity = world.get_entity_by_uuid(raider_uuid);
                match entity {
                    Some(e) => {
                        let is_dead = e.get_living_entity().is_none_or(|l| l.health.load() <= 0.0);
                        let pos = e.get_entity().pos.load();
                        let dist_sq = pos.squared_distance_to_vec(&center_f64);
                        if is_dead || dist_sq >= Self::RAID_REMOVAL_THRESHOLD_SQR {
                            wave_dead.push(raider_uuid);
                        }
                    }
                    None => {
                        wave_dead.push(raider_uuid);
                    }
                }
            }

            for uuid in wave_dead {
                raiders.remove(&uuid);
            }
        }
    }

    pub async fn update_bossbar(&mut self, world: &World) {
        let living_health = self.get_health_of_living_raiders(world);
        let progress = if self.total_health > 0.0 {
            (living_health / self.total_health).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.bossbar.health = progress;

        let players = world.players.load();
        for player in players.iter() {
            if self.players_in_raid.contains(&player.gameprofile.id) {
                player
                    .update_bossbar_health(&self.bossbar.uuid, self.bossbar.health)
                    .await;
                player
                    .update_bossbar_title(&self.bossbar.uuid, self.bossbar.title.clone())
                    .await;
            }
        }
    }

    #[must_use]
    pub fn get_health_of_living_raiders(&self, world: &World) -> f32 {
        let mut health = 0.0;
        for raiders in self.group_raider_map.values() {
            for &uuid in raiders {
                if let Some(e) = world.get_entity_by_uuid(uuid)
                    && let Some(living) = e.get_living_entity()
                {
                    health += living.health.load();
                }
            }
        }
        health
    }

    pub async fn tick(&mut self, world: &Arc<World>) {
        if self.is_stopped() {
            return;
        }

        if self.status == RaidStatus::Ongoing {
            if world.level_info.load().difficulty == Difficulty::Peaceful {
                self.stop(world).await;
                return;
            }

            self.ticks_active += 1;
            if self.ticks_active >= 48000 {
                self.stop(world).await;
                return;
            }

            let raiders_alive = self.get_total_raiders_alive();
            if raiders_alive == 0 && self.has_more_waves() {
                if self.raid_cooldown_ticks <= 0 {
                    if self.groups_spawned > 0 {
                        self.raid_cooldown_ticks = 300;
                        self.bossbar.title = TextComponent::translate("event.minecraft.raid", []);
                    }
                } else {
                    if self.wave_spawn_pos.is_none() && self.raid_cooldown_ticks % 5 == 0 {
                        self.wave_spawn_pos = self.find_random_spawn_pos(world, 8);
                    }

                    if self.raid_cooldown_ticks == 300 || self.raid_cooldown_ticks % 20 == 0 {
                        self.update_players(world).await;
                    }

                    self.raid_cooldown_ticks -= 1;
                    self.bossbar.health =
                        ((300 - self.raid_cooldown_ticks) as f32 / 300.0).clamp(0.0, 1.0);
                }
            }

            if self.ticks_active.is_multiple_of(20) {
                self.update_players(world).await;
                self.update_raiders(world);
                let alive = self.get_total_raiders_alive();
                if alive > 0 && alive <= 2 {
                    self.bossbar.title = TextComponent::translate(
                        "event.minecraft.raid.raiders_remaining",
                        [TextComponent::text(alive.to_string())],
                    );
                } else {
                    self.bossbar.title = TextComponent::translate("event.minecraft.raid", []);
                }
                self.update_bossbar(world).await;
            }

            while self.should_spawn_group() {
                let spawn_pos = self
                    .wave_spawn_pos
                    .or_else(|| self.find_random_spawn_pos(world, 20))
                    .unwrap_or(self.center);

                self.started = true;
                self.spawn_group(world, spawn_pos).await;
                self.play_sound(world, spawn_pos);
            }

            let raiders_alive = self.get_total_raiders_alive();
            if self.is_started() && !self.has_more_waves() && raiders_alive == 0 {
                if self.post_raid_ticks < 40 {
                    self.post_raid_ticks += 1;
                } else {
                    self.status = RaidStatus::Victory;
                    let effect = Effect {
                        effect_type: &StatusEffect::HERO_OF_THE_VILLAGE,
                        duration: 48000,
                        amplifier: (self.raid_omen_level - 1).max(0) as u8,
                        ambient: false,
                        show_particles: false,
                        show_icon: true,
                        blend: true,
                    };
                    let players = world.players.load();
                    for player in players.iter() {
                        if self.heroes_of_the_village.contains(&player.gameprofile.id) {
                            player.add_effect(effect.clone()).await;
                        }
                    }
                }
            }
        } else if self.is_over() {
            self.celebration_ticks += 1;
            if self.celebration_ticks >= 600 {
                self.stop(world).await;
                return;
            }

            if self.celebration_ticks.is_multiple_of(20) {
                self.update_players(world).await;
                if self.is_victory() {
                    self.bossbar.health = 0.0;
                    self.bossbar.title =
                        TextComponent::translate("event.minecraft.raid.victory.full", []);
                } else {
                    self.bossbar.title =
                        TextComponent::translate("event.minecraft.raid.defeat.full", []);
                }
                self.update_bossbar(world).await;
            }
        }
    }
}

#[derive(Default)]
pub struct Raids {
    pub raid_map: HashMap<i32, Raid>,
    pub next_id: i32,
    pub tick_counter: u64,
}

impl Raids {
    #[must_use]
    pub fn get(&self, raid_id: i32) -> Option<&Raid> {
        self.raid_map.get(&raid_id)
    }

    pub fn get_mut(&mut self, raid_id: i32) -> Option<&mut Raid> {
        self.raid_map.get_mut(&raid_id)
    }

    #[must_use]
    pub fn get_raid_at(&self, pos: &BlockPos) -> Option<&Raid> {
        let pos_f64 = pos.to_f64();
        self.raid_map.values().find(|r| {
            r.is_active()
                && r.center.to_f64().squared_distance_to_vec(&pos_f64)
                    <= Raid::VALID_RAID_RADIUS_SQR
        })
    }

    pub fn get_raid_at_mut(&mut self, pos: &BlockPos) -> Option<&mut Raid> {
        let pos_f64 = pos.to_f64();
        self.raid_map.values_mut().find(|r| {
            r.is_active()
                && r.center.to_f64().squared_distance_to_vec(&pos_f64)
                    <= Raid::VALID_RAID_RADIUS_SQR
        })
    }

    #[must_use]
    pub fn get_nearby_raid(&self, pos: &BlockPos, max_dist_sqr: f64) -> Option<&Raid> {
        let pos_f64 = pos.to_f64();
        let mut closest = None;
        let mut closest_dist = max_dist_sqr;

        for raid in self.raid_map.values() {
            let dist = raid.center.to_f64().squared_distance_to_vec(&pos_f64);
            if raid.is_active() && dist < closest_dist {
                closest = Some(raid);
                closest_dist = dist;
            }
        }
        closest
    }

    pub fn create_or_extend_raid(
        &mut self,
        player: &Player,
        raid_position: BlockPos,
        world: &Arc<World>,
    ) -> Option<i32> {
        let raid_center_pos = raid_position;

        let existing_id = self.raid_map.iter().find_map(|(&id, r)| {
            let dist = r
                .center
                .to_f64()
                .squared_distance_to_vec(&raid_center_pos.to_f64());
            (r.is_active() && dist <= Raid::VALID_RAID_RADIUS_SQR).then_some(id)
        });

        if let Some(id) = existing_id {
            if let Some(raid) = self.raid_map.get_mut(&id) {
                raid.absorb_raid_omen(player);
            }
            Some(id)
        } else {
            self.next_id += 1;
            let id = self.next_id;
            let mut raid = Raid::new(id, raid_center_pos, world.level_info.load().difficulty);
            raid.absorb_raid_omen(player);
            self.raid_map.insert(id, raid);
            Some(id)
        }
    }

    pub async fn tick(&mut self, world: &Arc<World>) {
        self.tick_counter += 1;

        let mut stopped_ids = Vec::new();
        for (&id, raid) in &mut self.raid_map {
            raid.tick(world).await;
            if raid.is_stopped() {
                stopped_ids.push(id);
            }
        }

        for id in stopped_ids {
            if let Some(mut raid) = self.raid_map.remove(&id) {
                raid.remove_all_players(world).await;
            }
        }
    }
}

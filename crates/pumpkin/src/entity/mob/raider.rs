use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::mob::patrol::{PatrolData, PatrollingMonster};

#[must_use]
pub fn create_ominous_banner() -> ItemStack {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{CustomNameImpl, DataComponentImpl};
    let mut stack = ItemStack::new(1, &Item::WHITE_BANNER);
    stack.patch.push((
        DataComponent::CustomName,
        Some(
            CustomNameImpl {
                name: TextComponent::translate("block.minecraft.ominous_banner", []),
            }
            .to_dyn(),
        ),
    ));
    stack
}

#[must_use]
pub const fn is_ominous_banner(stack: &ItemStack) -> bool {
    stack.item.id == Item::WHITE_BANNER.id
}

pub struct RaiderData {
    pub patrol_data: PatrolData,
    pub wave: AtomicI32,
    pub can_join_raid: AtomicBool,
    pub ticks_outside_raid: AtomicI32,
    pub is_celebrating: AtomicBool,
    pub raid_id: AtomicCell<Option<i32>>,
}

impl Default for RaiderData {
    fn default() -> Self {
        Self {
            patrol_data: PatrolData::default(),
            wave: AtomicI32::new(0),
            can_join_raid: AtomicBool::new(false),
            ticks_outside_raid: AtomicI32::new(0),
            is_celebrating: AtomicBool::new(false),
            raid_id: AtomicCell::new(None),
        }
    }
}

pub trait Raider: PatrollingMonster {
    fn get_raider_data(&self) -> &RaiderData;

    fn can_join_raid(&self) -> bool {
        self.get_raider_data().can_join_raid.load(Ordering::Relaxed)
    }

    fn set_can_join_raid(&self, can_join: bool) {
        self.get_raider_data()
            .can_join_raid
            .store(can_join, Ordering::Relaxed);
    }

    fn get_wave(&self) -> i32 {
        self.get_raider_data().wave.load(Ordering::Relaxed)
    }

    fn set_wave(&self, wave: i32) {
        self.get_raider_data().wave.store(wave, Ordering::Relaxed);
    }

    fn is_celebrating(&self) -> bool {
        self.get_raider_data()
            .is_celebrating
            .load(Ordering::Relaxed)
    }

    fn set_celebrating(&self, celebrating: bool) {
        self.get_raider_data()
            .is_celebrating
            .store(celebrating, Ordering::Relaxed);
        let entity = &self.get_mob_entity().living_entity.entity;
        entity.send_meta_data(
            &[Metadata::new(
                tracked_data::pillager::IS_CELEBRATING,
                celebrating,
            )],
            None,
        );
    }

    fn has_active_raid(&self) -> bool {
        self.get_raider_data().raid_id.load().is_some()
    }

    fn is_captain(&self) -> bool {
        self.is_patrol_leader()
    }

    fn get_celebrate_sound(&self) -> Sound;

    fn apply_raid_buffs(&self, _wave: i32, _is_captain: bool) {}

    fn write_raider_nbt(&self, nbt: &mut NbtCompound) {
        self.write_patrol_nbt(nbt);
        let data = self.get_raider_data();
        nbt.put_int("Wave", data.wave.load(Ordering::Relaxed));
        nbt.put_bool("CanJoinRaid", data.can_join_raid.load(Ordering::Relaxed));
        if let Some(raid_id) = data.raid_id.load() {
            nbt.put_int("RaidId", raid_id);
        }
    }

    fn read_raider_nbt(&self, nbt: &NbtCompound) {
        self.read_patrol_nbt(nbt);
        let data = self.get_raider_data();
        if let Some(wave) = nbt.get_int("Wave") {
            data.wave.store(wave, Ordering::Relaxed);
        }
        if let Some(can_join) = nbt.get_bool("CanJoinRaid") {
            data.can_join_raid.store(can_join, Ordering::Relaxed);
        }
        if let Some(raid_id) = nbt.get_int("RaidId") {
            data.raid_id.store(Some(raid_id));
        }
    }
}

/// Goal for raiders holding ground during patrols when spotting distant targets, alerting nearby raiders.
pub struct HoldGroundAttackGoal {
    hostile_radius_sqr: f64,
}

impl HoldGroundAttackGoal {
    #[must_use]
    pub fn new(hostile_radius: f32) -> Self {
        Self {
            hostile_radius_sqr: f64::from(hostile_radius * hostile_radius),
        }
    }
}

impl Goal for HoldGroundAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };
        if raider.has_active_raid() || !raider.is_patrolling() {
            return false;
        }
        let target = mob.get_mob_entity().get_target().clone();
        target.is_some()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target().clone();
        target.is_some()
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();

        let target = mob.get_mob_entity().get_target().clone();
        if let Some(target) = target {
            let entity = mob.get_entity();
            let world = entity.world.load();
            let bb = entity.bounding_box.load().expand(8.0, 8.0, 8.0);
            let nearby = world.get_entities_at_box(&bb);

            for cand in nearby {
                if cand.get_entity().entity_id != entity.entity_id
                    && let Some(cand_mob) = cand.get_mob()
                    && cand_mob.as_raider().is_some()
                {
                    cand_mob.get_mob_entity().set_target(Some(target.clone()));
                }
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let target = mob.get_mob_entity().get_target().clone();
        if let Some(target) = target {
            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            if dist_sq > self.hostile_radius_sqr {
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .look_at_entity_with_range(&target, 30.0, 30.0);
            }
        }
    }
}

/// Goal for raiders to pick up dropped ominous banners and become raid / patrol leaders.
pub struct ObtainRaidLeaderBannerGoal;

impl Goal for ObtainRaidLeaderBannerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };
        if !raider.can_be_leader() || raider.is_patrol_leader() {
            return false;
        }
        // Check if dropped banner nearby
        let entity = mob.get_entity();
        let world = entity.world.load();
        let bb = entity.bounding_box.load().expand(16.0, 4.0, 16.0);
        let nearby = world.get_entities_at_box(&bb);

        nearby
            .iter()
            .any(|e| *e.get_entity().entity_type == EntityType::ITEM)
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(raider) = mob.as_raider() else {
            return;
        };
        if !raider.can_be_leader() || raider.is_patrol_leader() {
            return;
        }

        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();
        let bb = entity.bounding_box.load().expand(16.0, 4.0, 16.0);
        let nearby = world.get_entities_at_box(&bb);

        for cand in nearby {
            if *cand.get_entity().entity_type == EntityType::ITEM {
                let cand_pos = cand.get_entity().pos.load();
                let dist = pos.squared_distance_to_vec(&cand_pos);
                if dist < 2.0 {
                    raider.set_patrol_leader(true);
                    let banner = create_ominous_banner();
                    let living = &mob.get_mob_entity().living_entity;
                    if let Ok(mut equipment) = living.entity_equipment.try_lock() {
                        equipment.put(&EquipmentSlot::HEAD, banner.clone());
                        drop(equipment);
                        living.send_equipment_changes(&[(EquipmentSlot::HEAD, banner)]);
                    }
                    cand.get_entity().remove();
                    break;
                }
                let mut nav = mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                nav.set_progress(NavigatorGoal {
                    current_progress: pos,
                    destination: cand_pos,
                    speed: 1.15,
                });
                break;
            }
        }
    }
}

/// Goal for raiders celebrating victory.
pub struct RaiderCelebrationGoal;

impl Goal for RaiderCelebrationGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };
        let target = mob.get_mob_entity().get_target().clone();
        target.is_none() && raider.is_celebrating()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };
        let target = mob.get_mob_entity().get_target().clone();
        target.is_none() && raider.is_celebrating()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(raider) = mob.as_raider() {
            raider.set_celebrating(true);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(raider) = mob.as_raider() {
            raider.set_celebrating(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(raider) = mob.as_raider() else {
            return;
        };
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();

        let r: f32 = rand::random();
        if r < 0.02 && !entity.silent.load(Ordering::Relaxed) {
            world.play_sound(
                raider.get_celebrate_sound(),
                pumpkin_data::sound::SoundCategory::Hostile,
                &pos,
            );
        }
    }
}

/// Goal for moving through village homes during active raids.
pub struct RaiderMoveThroughVillageGoal {
    speed_modifier: f64,
}

impl RaiderMoveThroughVillageGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self { speed_modifier }
    }
}

impl Goal for RaiderMoveThroughVillageGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };
        if !raider.has_active_raid() {
            return false;
        }
        let target = mob.get_mob_entity().get_target().clone();
        target.is_none()
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let mut nav = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if nav.is_idle() {
            let dx: f64 = (rand::random::<f64>() - 0.5) * 32.0;
            let dz: f64 = (rand::random::<f64>() - 0.5) * 32.0;
            let dest = Vector3::new(pos.x + dx, pos.y, pos.z + dz);
            nav.set_progress(NavigatorGoal {
                current_progress: pos,
                destination: dest,
                speed: self.speed_modifier,
            });
        }
    }
}

pub use crate::entity::ai::goal::pathfind_to_raid::PathfindToRaidGoal;

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, effect::StatusEffect, tag::Taggable};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::passive::villager::data::{
    GossipType, VillagerData, VillagerProfession, villager_type_at,
};
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture};

/// `BuiltInRegistries.VILLAGER_PROFESSION` (ZombieVillager.java:140-148,
/// `initializeZombieVillagerData`): a zombie villager is assigned a uniformly random
/// profession from the full profession registry, including `None` and `Nitwit`.
const PROFESSIONS: [VillagerProfession; 15] = [
    VillagerProfession::None,
    VillagerProfession::Armorer,
    VillagerProfession::Butcher,
    VillagerProfession::Cartographer,
    VillagerProfession::Cleric,
    VillagerProfession::Farmer,
    VillagerProfession::Fisherman,
    VillagerProfession::Fletcher,
    VillagerProfession::Leatherworker,
    VillagerProfession::Librarian,
    VillagerProfession::Mason,
    VillagerProfession::Nitwit,
    VillagerProfession::Shepherd,
    VillagerProfession::Toolsmith,
    VillagerProfession::Weaponsmith,
];

/// ZombieVillager.java:201 -- `Math.min(difficulty.getId() - 1, 0)`. Peaceful=0 .. Hard=3,
/// so this is never positive; `Effect::amplifier` is unsigned, so peaceful's negative
/// result clamps to 0 the same as every other difficulty.
fn conversion_strength_amplifier(difficulty_id: i32) -> u8 {
    let capped = (difficulty_id - 1).min(0);
    if capped < 0 { 0 } else { capped as u8 }
}

pub struct ZombieVillagerEntity {
    pub mob_entity: Arc<ZombieEntityBase>,
    villager_data: Mutex<VillagerData>,
    converting: AtomicBool,
    /// Ticks remaining until `finish_conversion`; meaningful only while `converting`.
    conversion_time: AtomicI32,
    conversion_starter: Mutex<Option<Uuid>>,
}

impl ZombieVillagerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Vanilla `ZombieVillager#finalizeSpawn` -> `VillagerDataHolder#finalizeVillagerType`
        // picks the type from `VillagerType.byBiome` at the spawn position.
        let villager_type = villager_type_at(&entity);
        let mob_entity = ZombieEntityBase::new(entity);
        let profession = PROFESSIONS[rand::random_range(0..PROFESSIONS.len())];
        let villager_data = VillagerData::new(villager_type, profession, 1);
        let zombie = Self {
            mob_entity,
            villager_data: Mutex::new(villager_data),
            converting: AtomicBool::new(false),
            conversion_time: AtomicI32::new(-1),
            conversion_starter: Mutex::new(None),
        };
        Arc::new(zombie)
    }

    async fn start_converting(&self, starter: Option<Uuid>, ticks: i32) {
        *self.conversion_starter.lock().await = starter;
        self.conversion_time.store(ticks, Ordering::Relaxed);
        self.converting.store(true, Ordering::Relaxed);

        let living = &self.get_mob_entity().living_entity;
        living.remove_effect(&StatusEffect::WEAKNESS).await;
        let difficulty = self.get_entity().world.load().level_info.load().difficulty as i32;
        living
            .add_effect(Effect {
                effect_type: &StatusEffect::STRENGTH,
                duration: ticks,
                amplifier: conversion_strength_amplifier(difficulty),
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;

        self.get_entity()
            .world
            .load()
            .send_entity_status(self.get_entity(), EntityStatus::ZombieConverting);
    }

    /// ZombieVillager.java:279-302 (`getConversionProgress`): base decrement of 1 per
    /// tick; a 1% roll scans an 8x8x8 volume for iron bars / beds (capped at 14 hits),
    /// each hit adding +1 with 30% probability.
    fn conversion_progress(&self) -> i32 {
        let mut amount = 1;
        if rand::random::<f32>() >= 0.01 {
            return amount;
        }

        let world = self.get_entity().world.load();
        let center = self.get_entity().block_pos.load().0;
        let mut hits = 0;
        'scan: for xx in (center.x - 4)..(center.x + 4) {
            for yy in (center.y - 4)..(center.y + 4) {
                for zz in (center.z - 4)..(center.z + 4) {
                    if hits >= 14 {
                        break 'scan;
                    }
                    let (block, _state) = world.get_block_and_state(&BlockPos::new(xx, yy, zz));
                    if block == &Block::IRON_BARS
                        || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS)
                    {
                        hits += 1;
                        if rand::random::<f32>() < 0.3 {
                            amount += 1;
                        }
                    }
                }
            }
        }

        amount
    }

    /// ZombieVillager.java:231-272 (`finishConversion`) via `ConversionType.SINGLE`
    /// (ConversionType.java): spawns the replacement `Villager`, carries over position,
    /// velocity, rotation, ground state, age/baby, invulnerability, custom name and
    /// active effects, adds the post-cure Nausea, then discards the zombie villager.
    ///
    /// Not ported: `dropPreservedEquipment` (no equipment kept -- `keepEquipment` is
    /// `false` in vanilla's own call), gossip/trade-offer carryover (only ever populated
    /// by the villager-to-zombie-villager infection path, out of scope here), and
    /// `refreshBrain`.
    ///
    /// The `ZOMBIE_VILLAGER_CURED` reputation event (`ZombieVillager.java:262`,
    /// `Villager::onReputationEventFrom` at `Villager.java:855-858`) *is* ported below --
    /// `conversion_starter` already tracks the curing player's UUID for the advancement
    /// trigger, so it doubles as the reputation-event source.
    async fn finish_conversion(&self) {
        let old_entity = self.get_entity();
        let world = old_entity.world.load().clone();
        let pos = old_entity.pos.load();

        let new_entity = Entity::new(world.clone(), pos, &EntityType::VILLAGER);
        let villager = VillagerEntity::new(new_entity);

        let data = *self.villager_data.lock().await;
        *villager.villager_data.lock().await = data;

        let new_entity = villager.get_entity();
        new_entity.velocity.store(old_entity.velocity.load());
        new_entity.yaw.store(old_entity.yaw.load());
        new_entity.pitch.store(old_entity.pitch.load());
        new_entity.on_ground.store(
            old_entity.on_ground.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        new_entity
            .age
            .store(old_entity.age.load(Ordering::Relaxed), Ordering::Relaxed);
        new_entity.invulnerable.store(
            old_entity.invulnerable.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        if let Some(name) = &**old_entity.custom_name.load() {
            new_entity.set_custom_name(name.clone());
            new_entity.custom_name_visible.store(
                old_entity.custom_name_visible.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
        }

        let effects: Vec<_> = self
            .get_mob_entity()
            .living_entity
            .active_effects
            .lock()
            .await
            .values()
            .cloned()
            .collect();
        let new_living = &villager.get_mob_entity().living_entity;
        for effect in effects {
            new_living.add_effect(effect).await;
        }
        new_living
            .add_effect(Effect {
                effect_type: &StatusEffect::NAUSEA,
                duration: 200, // ZombieVillager.java:266
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;

        world.spawn_entity(villager.clone()).await;
        world.sync_world_event(
            WorldEvent::SoundZombieConverted, // ZombieVillager.java:268, levelEvent 1027
            self.get_entity().block_pos.load(),
            0,
        );

        // ZombieVillager.java:259-262: both the advancement trigger and the
        // `ZOMBIE_VILLAGER_CURED` reputation event (`Villager::onReputationEventFrom`,
        // `Villager.java:855-858`) only fire `if (player instanceof ServerPlayer)` -- i.e. the
        // curing player must still be online, not merely have started the conversion. Neither
        // fires immediately on the golden-apple click.
        if let Some(starter) = *self.conversion_starter.lock().await
            && let Some(player) = world.get_player_by_uuid(starter)
        {
            player
                .trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::CuredZombieVillager,
                )
                .await;

            let mut gossips = villager.gossips.lock().await;
            gossips.add(starter, GossipType::MajorPositive, 20);
            gossips.add(starter, GossipType::MinorPositive, 25);
        }

        old_entity.remove().await;
    }
}

impl NBTStorage for ZombieVillagerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.write_nbt(nbt).await;

            let data = *self.villager_data.lock().await;
            let mut villager_data_nbt = NbtCompound::new();
            villager_data_nbt.put_int("Type", data.r#type.0);
            villager_data_nbt.put_int("Profession", data.profession.0);
            villager_data_nbt.put_int("Level", data.level.0);
            nbt.put_compound("VillagerData", villager_data_nbt);

            nbt.put_int(
                "ConversionTime",
                if self.converting.load(Ordering::Relaxed) {
                    self.conversion_time.load(Ordering::Relaxed)
                } else {
                    -1
                },
            );
            let conversion_starter = *self.conversion_starter.lock().await;
            if let Some(starter) = conversion_starter {
                let v = starter.as_u128();
                nbt.put(
                    "ConversionPlayer",
                    NbtTag::IntArray(vec![
                        (v >> 96) as i32,
                        ((v >> 64) & 0xFFFF_FFFF) as i32,
                        ((v >> 32) & 0xFFFF_FFFF) as i32,
                        (v & 0xFFFF_FFFF) as i32,
                    ]),
                );
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.read_nbt_non_mut(nbt).await;

            if let Some(villager_data_nbt) = nbt.get_compound("VillagerData") {
                let mut data = self.villager_data.lock().await;
                if let Some(t) = villager_data_nbt.get_int("Type") {
                    data.r#type = VarInt(t);
                }
                if let Some(p) = villager_data_nbt.get_int("Profession") {
                    data.profession = VarInt(p);
                }
                if let Some(l) = villager_data_nbt.get_int("Level") {
                    data.level = VarInt(l);
                }
            }

            let conversion_time = nbt.get_int("ConversionTime").unwrap_or(-1);
            if conversion_time != -1 {
                let starter = nbt.get_int_array("ConversionPlayer").map(|a| {
                    Uuid::from_u128(
                        (a[0] as u128) << 96
                            | (a[1] as u128) << 64
                            | (a[2] as u128) << 32
                            | (a[3] as u128),
                    )
                });
                *self.conversion_starter.lock().await = starter;
                self.conversion_time
                    .store(conversion_time, Ordering::Relaxed);
                self.converting.store(true, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for ZombieVillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity.mob_entity
    }

    /// ZombieVillager.java:150-161 (`tick`): runs every tick while converting and alive.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if !self.converting.load(Ordering::Relaxed)
                || self
                    .get_mob_entity()
                    .living_entity
                    .dead
                    .load(Ordering::Relaxed)
            {
                return;
            }

            let amount = self.conversion_progress();
            let remaining = self.conversion_time.fetch_sub(amount, Ordering::Relaxed) - amount;
            if remaining <= 0 {
                self.finish_conversion().await;
            }
        })
    }

    /// ZombieVillager.java:163-180 (`mobInteract`): a golden apple only does something
    /// while the zombie villager has Weakness; otherwise the click is swallowed
    /// (`InteractionResult.CONSUME`) without eating the apple.
    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.item.id != Item::GOLDEN_APPLE.id {
                return self
                    .get_mob_entity()
                    .mob_interact(player, item_stack, self.can_be_leashed())
                    .await;
            }

            if !self
                .get_mob_entity()
                .living_entity
                .has_effect(&StatusEffect::WEAKNESS)
                .await
            {
                return true;
            }

            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let ticks = rand::random_range(3600..=6000); // ZombieVillager.java:170
            self.start_converting(Some(player.get_entity().entity_uuid), ticks)
                .await;
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PROFESSIONS, conversion_strength_amplifier};

    #[test]
    fn conversion_timer_stays_in_vanilla_range() {
        for _ in 0..5000 {
            let ticks = rand::random_range(3600..=6000);
            assert!((3600..=6000).contains(&ticks));
        }
    }

    #[test]
    fn strength_amplifier_never_exceeds_zero() {
        for difficulty_id in 0..=3 {
            assert_eq!(conversion_strength_amplifier(difficulty_id), 0);
        }
    }

    #[test]
    fn profession_table_matches_full_registry_size() {
        assert_eq!(PROFESSIONS.len(), 15);
    }

    #[test]
    fn conversion_progress_amount_from_hits() {
        // Mirrors ZombieVillager.java:280-298: base 1, +1 per qualifying-block hit.
        fn amount_for(hits: &[bool]) -> i32 {
            1 + i32::try_from(hits.iter().filter(|&&hit| hit).count()).unwrap()
        }
        assert_eq!(amount_for(&[]), 1);
        assert_eq!(amount_for(&[true; 14]), 15);
        assert_eq!(amount_for(&[false, true, false, true]), 3);
    }
}

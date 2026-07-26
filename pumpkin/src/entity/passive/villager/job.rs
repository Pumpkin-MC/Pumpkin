use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::blocks::bed::BedBlock;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BedPart, BlockProperties, WhiteBedLikeProperties as BedProperties,
};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use crate::entity::player::Player;
use crate::entity::{
    EntityBase,
    mob::{Mob, MobEntity},
};

use super::{VillagerData, VillagerEntity, VillagerProfession};

fn block_to_profession(block: &Block) -> Option<VillagerProfession> {
    if block == &Block::COMPOSTER {
        Some(VillagerProfession::Farmer)
    } else if block == &Block::LECTERN {
        Some(VillagerProfession::Librarian)
    } else if block == &Block::BLAST_FURNACE {
        Some(VillagerProfession::Armorer)
    } else if block == &Block::SMOKER {
        Some(VillagerProfession::Butcher)
    } else if block == &Block::CARTOGRAPHY_TABLE {
        Some(VillagerProfession::Cartographer)
    } else if block == &Block::BREWING_STAND {
        Some(VillagerProfession::Cleric)
    } else if block == &Block::BARREL {
        Some(VillagerProfession::Fisherman)
    } else if block == &Block::FLETCHING_TABLE {
        Some(VillagerProfession::Fletcher)
    } else if block == &Block::CAULDRON
        || block == &Block::WATER_CAULDRON
        || block == &Block::LAVA_CAULDRON
        || block == &Block::POWDER_SNOW_CAULDRON
    {
        Some(VillagerProfession::Leatherworker)
    } else if block == &Block::STONECUTTER {
        Some(VillagerProfession::Mason)
    } else if block == &Block::LOOM {
        Some(VillagerProfession::Shepherd)
    } else if block == &Block::SMITHING_TABLE {
        Some(VillagerProfession::Toolsmith)
    } else if block == &Block::GRINDSTONE {
        Some(VillagerProfession::Weaponsmith)
    } else {
        None
    }
}

fn profession_matches_block(profession: VillagerProfession, block: &Block) -> bool {
    match profession {
        VillagerProfession::Farmer => block == &Block::COMPOSTER,
        VillagerProfession::Librarian => block == &Block::LECTERN,
        VillagerProfession::Armorer => block == &Block::BLAST_FURNACE,
        VillagerProfession::Butcher => block == &Block::SMOKER,
        VillagerProfession::Cartographer => block == &Block::CARTOGRAPHY_TABLE,
        VillagerProfession::Cleric => block == &Block::BREWING_STAND,
        VillagerProfession::Fisherman => block == &Block::BARREL,
        VillagerProfession::Fletcher => block == &Block::FLETCHING_TABLE,
        VillagerProfession::Leatherworker => {
            block == &Block::CAULDRON
                || block == &Block::WATER_CAULDRON
                || block == &Block::LAVA_CAULDRON
                || block == &Block::POWDER_SNOW_CAULDRON
        }
        VillagerProfession::Mason => block == &Block::STONECUTTER,
        VillagerProfession::Shepherd => block == &Block::LOOM,
        VillagerProfession::Toolsmith => block == &Block::SMITHING_TABLE,
        VillagerProfession::Weaponsmith => block == &Block::GRINDSTONE,
        _ => false,
    }
}

impl Mob for VillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            if entity.age.load(Ordering::Relaxed) < 0 {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }

            let villager_data = *self.villager_data.lock().await;
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::VILLAGER_DATA,
                    MetaDataType::VILLAGER_DATA,
                    villager_data,
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::VILLAGER_DATA_FINALIZED,
                    MetaDataType::BOOLEAN,
                    true,
                )],
                None,
            );
        })
    }

    fn get_job_site(&self) -> Option<BlockPos> {
        *self.job_site.lock().unwrap()
    }

    fn get_home(&self) -> Option<BlockPos> {
        *self.home_pos.lock().unwrap()
    }

    #[expect(clippy::too_many_lines)]
    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let age = self.get_entity().age.load(Ordering::Relaxed);
            if age % 20 != 0 {
                return;
            }

            let world = self.get_entity().world.load();

            // 1. Bed / Sleeping logic (for all villagers: babies, nitwits, adults)
            let is_sleeping = self.get_entity().pose.load() == EntityPose::Sleeping;

            // Check if current bed is still valid
            if let Some(current_home) = self.get_home_pos() {
                let (block, state) = world.get_block_and_state(&current_home);
                let valid = if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                    let bed_props = BedProperties::from_state_id(state.id, block);
                    bed_props.part == BedPart::Head
                } else {
                    false
                };

                if !valid {
                    *self.home_pos.lock().unwrap() = None;
                    if is_sleeping {
                        // Wake up if bed was broken
                        self.get_entity().set_pose(EntityPose::Standing);
                        self.get_entity().send_meta_data(
                            &[Metadata::new(
                                TrackedData::SLEEPING_POS_ID,
                                MetaDataType::OPTIONAL_BLOCK_POS,
                                None::<BlockPos>,
                            )],
                            None,
                        );
                    }
                }
            }

            // If no bed, search for one
            if self.get_home_pos().is_none() {
                let pos = self.get_entity().block_pos.load();
                let start = BlockPos::new(pos.0.x - 16, pos.0.y - 4, pos.0.z - 16);
                let end = BlockPos::new(pos.0.x + 16, pos.0.y + 4, pos.0.z + 16);

                let aabb = BoundingBox::new(
                    Vector3::new(
                        pos.0.x as f64 - 32.0,
                        pos.0.y as f64 - 16.0,
                        pos.0.z as f64 - 32.0,
                    ),
                    Vector3::new(
                        pos.0.x as f64 + 32.0,
                        pos.0.y as f64 + 16.0,
                        pos.0.z as f64 + 32.0,
                    ),
                );
                let nearby_entities = world.get_all_at_box(&aabb);

                let mut claimed_homes = Vec::new();
                for entity in nearby_entities {
                    if entity.get_entity().entity_id != self.get_entity().entity_id
                        && entity.get_entity().entity_type
                            == &pumpkin_data::entity::EntityType::VILLAGER
                        && let Some(home) = entity.get_home_pos()
                    {
                        claimed_homes.push(home);
                    }
                }

                let mut best_home = None;
                let mut best_dist = f64::MAX;

                for p in BlockPos::iterate(start, end) {
                    let (block, state) = world.get_block_and_state(&p);
                    if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                        let bed_props = BedProperties::from_state_id(state.id, block);
                        let bed_head_pos = if bed_props.part == BedPart::Head {
                            p
                        } else {
                            p.offset(bed_props.facing.to_offset())
                        };

                        if claimed_homes.contains(&bed_head_pos) {
                            continue;
                        }

                        let dist = bed_head_pos
                            .to_f64()
                            .squared_distance_to_vec(&self.get_entity().pos.load());
                        if dist < best_dist {
                            best_dist = dist;
                            best_home = Some(bed_head_pos);
                        }
                    }
                }

                if let Some(home) = best_home {
                    *self.home_pos.lock().unwrap() = Some(home);
                }
            }

            // Handle Sleeping/Waking up based on time
            let is_sleeping = self.get_entity().pose.load() == EntityPose::Sleeping;
            if let Some(home_pos) = self.get_home_pos() {
                let time = world.level_time.lock().await.time_of_day;
                let is_night = (12000..=23000).contains(&time);

                if is_night {
                    if !is_sleeping {
                        // Check distance to bed. If close enough, go to sleep
                        let dist = home_pos
                            .to_f64()
                            .squared_distance_to_vec(&self.get_entity().pos.load());
                        if dist <= 4.0 {
                            // Within 2 blocks (squared distance 4.0)
                            let (block, state) = world.get_block_and_state(&home_pos);
                            if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                                let bed_props = BedProperties::from_state_id(state.id, block);
                                if !bed_props.occupied {
                                    // Make bed occupied
                                    BedBlock::set_occupied(
                                        true, &world, block, &home_pos, state.id,
                                    )
                                    .await;

                                    self.get_entity().set_pose(EntityPose::Sleeping);
                                    self.get_entity().send_meta_data(
                                        &[Metadata::new(
                                            TrackedData::SLEEPING_POS_ID,
                                            MetaDataType::OPTIONAL_BLOCK_POS,
                                            Some(home_pos),
                                        )],
                                        None,
                                    );
                                }
                            }
                        }
                    }
                } else if is_sleeping {
                    // It is day, wake up!
                    let (block, state) = world.get_block_and_state(&home_pos);
                    if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                        let bed_props = BedProperties::from_state_id(state.id, block);
                        if bed_props.occupied {
                            BedBlock::set_occupied(false, &world, block, &home_pos, state.id).await;
                        }
                    }

                    self.get_entity().set_pose(EntityPose::Standing);
                    self.get_entity().send_meta_data(
                        &[Metadata::new(
                            TrackedData::SLEEPING_POS_ID,
                            MetaDataType::OPTIONAL_BLOCK_POS,
                            None::<BlockPos>,
                        )],
                        None,
                    );
                }
            }

            // 2. Job / Profession logic (skip for Nitwits and babies)
            let data = self.villager_data.lock().await;
            let is_adult = self.get_entity().age.load(Ordering::Relaxed) >= 0;
            let xp = self.xp.load(Ordering::Relaxed);
            let profession = data.profession_enum();
            drop(data);

            if profession == VillagerProfession::Nitwit || !is_adult {
                return;
            }

            if let Some(current_site) = self.get_job_site() {
                let (block, _state) = world.get_block_and_state(&current_site);
                let valid = if profession == VillagerProfession::None {
                    block_to_profession(block).is_some()
                } else {
                    profession_matches_block(profession, block)
                };

                if !valid {
                    *self.job_site.lock().unwrap() = None;
                    if xp == 0 && profession != VillagerProfession::None {
                        let r#type = self.villager_data.lock().await.type_enum();
                        self.set_villager_data(VillagerData::new(
                            r#type,
                            VillagerProfession::None,
                            1,
                        ))
                        .await;
                        self.offers.lock().await.clear();
                    }
                }
            }

            if self.get_job_site().is_none() {
                let pos = self.get_entity().block_pos.load();
                let start = BlockPos::new(pos.0.x - 10, pos.0.y - 4, pos.0.z - 10);
                let end = BlockPos::new(pos.0.x + 10, pos.0.y + 4, pos.0.z + 10);

                let aabb = BoundingBox::new(
                    Vector3::new(
                        pos.0.x as f64 - 32.0,
                        pos.0.y as f64 - 16.0,
                        pos.0.z as f64 - 32.0,
                    ),
                    Vector3::new(
                        pos.0.x as f64 + 32.0,
                        pos.0.y as f64 + 16.0,
                        pos.0.z as f64 + 32.0,
                    ),
                );
                let nearby_entities = world.get_all_at_box(&aabb);

                let mut claimed_sites = Vec::new();
                for entity in nearby_entities {
                    if entity.get_entity().entity_id != self.get_entity().entity_id
                        && entity.get_entity().entity_type
                            == &pumpkin_data::entity::EntityType::VILLAGER
                        && let Some(site) = entity.get_job_site_pos()
                    {
                        claimed_sites.push(site);
                    }
                }

                let mut best_site = None;
                let mut best_dist = f64::MAX;
                let mut best_profession = VillagerProfession::None;

                for p in BlockPos::iterate(start, end) {
                    if claimed_sites.contains(&p) {
                        continue;
                    }

                    let (block, _state) = world.get_block_and_state(&p);
                    if let Some(prof) = block_to_profession(block) {
                        if profession != VillagerProfession::None && prof != profession {
                            continue;
                        }

                        let dist = p
                            .to_f64()
                            .squared_distance_to_vec(&self.get_entity().pos.load());
                        if dist < best_dist {
                            best_dist = dist;
                            best_site = Some(p);
                            best_profession = prof;
                        }
                    }
                }

                if let Some(site) = best_site {
                    *self.job_site.lock().unwrap() = Some(site);
                    if profession == VillagerProfession::None {
                        let r#type = self.villager_data.lock().await.type_enum();
                        self.set_villager_data(VillagerData::new(r#type, best_profession, 1))
                            .await;
                    }
                }
            } else {
                let current_prof = self.villager_data.lock().await.profession_enum();
                if current_prof == VillagerProfession::None
                    && let Some(site) = self.get_job_site()
                {
                    let (block, _state) = world.get_block_and_state(&site);
                    if let Some(prof) = block_to_profession(block) {
                        let r#type = self.villager_data.lock().await.type_enum();
                        self.set_villager_data(VillagerData::new(r#type, prof, 1))
                            .await;
                    }
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        _item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        let player = player.clone();
        Box::pin(async move {
            if self.get_entity().age.load(Ordering::Relaxed) < 0 {
                self.set_unhappy();
                return true;
            }

            let mut offers = self.offers.lock().await;
            if offers.is_empty() {
                let data = self.villager_data.lock().await;
                // Vanilla: only employed villagers (not None / Nitwit) have trades.
                // Unemployed must claim a workstation first.
                if data.profession_enum() != VillagerProfession::None
                    && data.profession_enum() != VillagerProfession::Nitwit
                {
                    let prof = data.profession_enum();
                    let level = data.level.0;
                    drop(data);
                    drop(offers);
                    self.generate_trades(prof, level).await;
                    offers = self.offers.lock().await;
                } else {
                    drop(data);
                }
            }

            if offers.is_empty() {
                self.set_unhappy();
                return true;
            }
            drop(offers);

            player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::TalkedToVillager as i32,
                    1,
                )
                .await;

            self.open_trading_screen(&player).await;

            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workstation_blocks_round_trip_to_their_profession() {
        let workstations = [
            &Block::COMPOSTER,
            &Block::LECTERN,
            &Block::BLAST_FURNACE,
            &Block::SMOKER,
            &Block::CARTOGRAPHY_TABLE,
            &Block::BREWING_STAND,
            &Block::BARREL,
            &Block::FLETCHING_TABLE,
            &Block::CAULDRON,
            &Block::WATER_CAULDRON,
            &Block::LAVA_CAULDRON,
            &Block::POWDER_SNOW_CAULDRON,
            &Block::STONECUTTER,
            &Block::LOOM,
            &Block::SMITHING_TABLE,
            &Block::GRINDSTONE,
        ];
        for block in workstations {
            let profession = block_to_profession(block)
                .unwrap_or_else(|| panic!("{} must map to a profession", block.name));
            assert!(profession_matches_block(profession, block));
        }
    }

    #[test]
    fn specific_workstation_professions_match_vanilla() {
        assert_eq!(
            block_to_profession(&Block::COMPOSTER),
            Some(VillagerProfession::Farmer)
        );
        assert_eq!(
            block_to_profession(&Block::GRINDSTONE),
            Some(VillagerProfession::Weaponsmith)
        );
        assert_eq!(
            block_to_profession(&Block::WATER_CAULDRON),
            Some(VillagerProfession::Leatherworker)
        );
        assert_eq!(block_to_profession(&Block::STONE), None);
    }

    #[test]
    fn unemployed_professions_match_no_workstation() {
        for profession in [VillagerProfession::None, VillagerProfession::Nitwit] {
            assert!(!profession_matches_block(profession, &Block::COMPOSTER));
            assert!(!profession_matches_block(profession, &Block::GRINDSTONE));
        }
    }

    #[test]
    fn professions_reject_other_professions_workstations() {
        assert!(!profession_matches_block(
            VillagerProfession::Farmer,
            &Block::GRINDSTONE,
        ));
        assert!(!profession_matches_block(
            VillagerProfession::Weaponsmith,
            &Block::COMPOSTER,
        ));
    }
}

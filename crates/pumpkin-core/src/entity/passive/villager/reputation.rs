use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use uuid::Uuid;

use super::{GossipType, VillagerEntity};
use crate::entity::EntityBase;
use crate::entity::ai::brain::memory::types;
use crate::entity::ai::brain::{BrainState, VisibilityContext};
use crate::entity::mob::Mob;
use crate::world::World;

use crate::entity::ai::brain::behavior::poi_walk::release_poi::release_poi;

const MAX_GOSSIP_TOPICS: i32 = 10;
const GOSSIP_COOLDOWN: i64 = 1200;
const GOSSIP_DECAY_INTERVAL: i64 = 24000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReputationEventType {
    ZombieVillagerCured,
    Trade,
    VillagerHurt,
    VillagerKilled,
}

impl VillagerEntity {
    pub fn on_reputation_event_from(&self, event: ReputationEventType, source: Uuid) {
        {
            let mut gossips = self
                .gossips
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match event {
                ReputationEventType::ZombieVillagerCured => {
                    gossips.add(source, GossipType::MajorPositive, 20);
                    gossips.add(source, GossipType::MinorPositive, 25);
                }
                ReputationEventType::Trade => gossips.add(source, GossipType::Trading, 2),
                ReputationEventType::VillagerHurt => {
                    gossips.add(source, GossipType::MinorNegative, 25);
                }
                ReputationEventType::VillagerKilled => {
                    gossips.add(source, GossipType::MajorNegative, 25);
                }
            }
        }
        self.refresh_trading_player_prices(Some(source));
    }

    pub fn gossip(
        &self,
        state: &mut BrainState,
        world: &Arc<World>,
        target: &Self,
        timestamp: i64,
    ) {
        let last_gossip_time = self.last_gossip_share_time.load(Relaxed);
        let target_last_gossip_time = target.last_gossip_share_time.load(Relaxed);
        if (timestamp < last_gossip_time || timestamp >= last_gossip_time + GOSSIP_COOLDOWN)
            && (timestamp < target_last_gossip_time
                || timestamp >= target_last_gossip_time + GOSSIP_COOLDOWN)
        {
            let source = target
                .gossips
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let new_gossip = self
                .gossips
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .transfer_from(&source, &mut self.get_random(), MAX_GOSSIP_TOPICS);
            self.last_gossip_share_time.store(timestamp, Relaxed);
            target.last_gossip_share_time.store(timestamp, Relaxed);
            self.spawn_golem_if_needed(state, world, timestamp, 5);
            if new_gossip > 0 {
                self.refresh_trading_player_prices(None);
            }
        }
    }

    pub(super) fn maybe_decay_gossip(&self, game_time: i64) {
        let last_decay = self.last_gossip_decay_time.load(Relaxed);
        if last_decay == 0 {
            self.last_gossip_decay_time.store(game_time, Relaxed);
        } else if game_time >= last_decay + GOSSIP_DECAY_INTERVAL {
            self.gossips
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .decay();
            self.last_gossip_decay_time.store(game_time, Relaxed);
        }
    }

    pub(super) fn tell_witnesses_that_i_was_murdered(&self, murderer: &dyn EntityBase) {
        let world = self.get_entity().world.load_full();
        let follow_range = self
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        let witnesses = self.mob_entity.with_brain(self, |brain| {
            let context = VisibilityContext {
                state: brain,
                world: &world,
                mob: self,
                follow_range,
            };
            brain
                .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                .map(|visible| {
                    visible
                        .find_all(&context, |entity| {
                            entity.get_entity().entity_type == &EntityType::VILLAGER
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        });
        let murderer_uuid = murderer.get_entity().entity_uuid;
        for witness in witnesses {
            if let Some(villager) = witness.cast_any().downcast_ref::<Self>() {
                villager
                    .on_reputation_event_from(ReputationEventType::VillagerKilled, murderer_uuid);
            }
        }
    }

    pub(super) fn release_all_pois(&self) {
        self.mob_entity.with_brain(self, |brain| {
            for memory in [
                types::HOME,
                types::JOB_SITE,
                types::POTENTIAL_JOB_SITE,
                types::MEETING_POINT,
            ] {
                release_poi(brain, self, memory);
            }
        });
    }

    pub(super) fn refresh_trading_player_prices(&self, source: Option<Uuid>) {
        if let Some(player) = self.get_trading_player()
            && source.is_none_or(|uuid| uuid == player.get_entity().entity_uuid)
        {
            self.update_special_prices(&player);
            self.resend_offers_to_trading_player();
        }
    }
}

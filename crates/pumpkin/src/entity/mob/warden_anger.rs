use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use pumpkin_data::{entity::EntityType, sound::Sound};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{EntityBase, RemovalReason};
use crate::world::World;

const CONVERSION_DELAY: i32 = 2;
const MAX_ANGER: i32 = 150;
const DEFAULT_ANGER_DECREASE: i32 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AngerLevel {
    Calm,
    Agitated,
    Angry,
}

impl AngerLevel {
    const SORTED_LEVELS: [Self; 3] = [Self::Angry, Self::Agitated, Self::Calm];

    #[must_use]
    pub const fn minimum_anger(self) -> i32 {
        match self {
            Self::Calm => 0,
            Self::Agitated => 40,
            Self::Angry => 80,
        }
    }

    #[must_use]
    pub const fn ambient_sound(self) -> Sound {
        match self {
            Self::Calm => Sound::EntityWardenAmbient,
            Self::Agitated => Sound::EntityWardenAgitated,
            Self::Angry => Sound::EntityWardenAngry,
        }
    }

    #[must_use]
    pub const fn listening_sound(self) -> Sound {
        match self {
            Self::Calm => Sound::EntityWardenListening,
            Self::Agitated | Self::Angry => Sound::EntityWardenListeningAngry,
        }
    }

    #[must_use]
    pub fn by_anger(anger: i32) -> Self {
        Self::SORTED_LEVELS
            .into_iter()
            .find(|level| anger >= level.minimum_anger())
            .unwrap_or(Self::Calm)
    }

    #[must_use]
    pub fn is_angry(self) -> bool {
        self == Self::Angry
    }
}

pub struct AngerManagement {
    conversion_delay: i32,
    highest_anger: i32,
    suspects: Vec<Arc<dyn EntityBase>>,
    anger_by_suspect: HashMap<i32, i32>,
    anger_by_uuid: HashMap<Uuid, i32>,
}

impl Default for AngerManagement {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl AngerManagement {
    #[must_use]
    pub fn new(anger_by_uuid: HashMap<Uuid, i32>) -> Self {
        Self {
            conversion_delay: rand::rng().random_range(0..=CONVERSION_DELAY),
            highest_anger: 0,
            suspects: Vec::new(),
            anger_by_suspect: HashMap::new(),
            anger_by_uuid,
        }
    }

    pub fn tick(&mut self, world: &World, valid_entity: impl Fn(&dyn EntityBase) -> bool) {
        self.conversion_delay -= 1;
        if self.conversion_delay <= 0 {
            self.convert_from_uuids(world);
            self.conversion_delay = CONVERSION_DELAY;
        }

        self.anger_by_uuid.retain(|_, anger| {
            if *anger <= DEFAULT_ANGER_DECREASE {
                false
            } else {
                *anger -= DEFAULT_ANGER_DECREASE;
                true
            }
        });

        let anger_by_suspect = &mut self.anger_by_suspect;
        let anger_by_uuid = &mut self.anger_by_uuid;
        self.suspects.retain(|suspect| {
            let entity = suspect.get_entity();
            let anger = anger_by_suspect
                .get(&entity.entity_id)
                .copied()
                .unwrap_or(0);
            let removal_reason = entity.removal_reason.load();
            if anger > DEFAULT_ANGER_DECREASE
                && valid_entity(suspect.as_ref())
                && removal_reason.is_none()
            {
                anger_by_suspect.insert(entity.entity_id, anger - DEFAULT_ANGER_DECREASE);
                return true;
            }

            anger_by_suspect.remove(&entity.entity_id);
            if anger > DEFAULT_ANGER_DECREASE
                && matches!(
                    removal_reason,
                    Some(
                        RemovalReason::ChangedDimension
                            | RemovalReason::UnloadedToChunk
                            | RemovalReason::UnloadedWithPlayer
                    )
                )
            {
                anger_by_uuid.insert(entity.entity_uuid, anger - DEFAULT_ANGER_DECREASE);
            }
            false
        });

        self.sort_and_update_highest_anger();
    }

    fn anger_of(&self, entity: &dyn EntityBase) -> i32 {
        self.anger_by_suspect
            .get(&entity.get_entity().entity_id)
            .copied()
            .unwrap_or(0)
    }

    fn sort_and_update_highest_anger(&mut self) {
        let anger_by_suspect = &self.anger_by_suspect;
        let anger_of = |entity: &Arc<dyn EntityBase>| {
            anger_by_suspect
                .get(&entity.get_entity().entity_id)
                .copied()
                .unwrap_or(0)
        };
        self.suspects.sort_by(|first, second| {
            let first_anger = anger_of(first);
            let second_anger = anger_of(second);
            let first_angry = AngerLevel::by_anger(first_anger).is_angry();
            let second_angry = AngerLevel::by_anger(second_anger).is_angry();
            if first_angry != second_angry {
                return if first_angry {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
            let first_player = first.get_entity().entity_type.id == EntityType::PLAYER.id;
            let second_player = second.get_entity().entity_type.id == EntityType::PLAYER.id;
            if first_player != second_player {
                return if first_player {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
            second_anger.cmp(&first_anger)
        });
        self.highest_anger = self.suspects.iter().map(anger_of).max().unwrap_or(0);
    }

    fn convert_from_uuids(&mut self, world: &World) {
        let suspects = &mut self.suspects;
        let anger_by_suspect = &mut self.anger_by_suspect;
        self.anger_by_uuid.retain(|uuid, anger| {
            let entity = world
                .get_player_by_uuid(*uuid)
                .map(|player| player as Arc<dyn EntityBase>)
                .or_else(|| world.get_entity_by_uuid(*uuid));
            let Some(entity) = entity else {
                return true;
            };
            anger_by_suspect.insert(entity.get_entity().entity_id, *anger);
            suspects.push(entity);
            false
        });
    }

    pub fn increase_anger(&mut self, entity: &Arc<dyn EntityBase>, increment: i32) -> i32 {
        let entity_id = entity.get_entity().entity_id;
        let new_suspect = !self.anger_by_suspect.contains_key(&entity_id);
        let mut current_anger = (self.anger_of(entity.as_ref()) + increment).min(MAX_ANGER);
        if new_suspect {
            current_anger += self
                .anger_by_uuid
                .remove(&entity.get_entity().entity_uuid)
                .unwrap_or(0);
            self.suspects.push(entity.clone());
        }
        self.anger_by_suspect.insert(entity_id, current_anger);

        self.sort_and_update_highest_anger();
        current_anger
    }

    pub fn clear_anger(&mut self, entity: &dyn EntityBase) {
        let entity_id = entity.get_entity().entity_id;
        self.anger_by_suspect.remove(&entity_id);
        self.suspects
            .retain(|suspect| suspect.get_entity().entity_id != entity_id);
        self.sort_and_update_highest_anger();
    }

    #[must_use]
    pub fn get_active_anger(&self, current_target: Option<&dyn EntityBase>) -> i32 {
        current_target.map_or(self.highest_anger, |target| self.anger_of(target))
    }

    pub fn get_active_entity(
        &self,
        filter: impl Fn(&dyn EntityBase) -> bool,
    ) -> Option<Arc<dyn EntityBase>> {
        self.suspects
            .iter()
            .find(|suspect| filter(suspect.as_ref()))
            .filter(|suspect| suspect.get_living_entity().is_some())
            .cloned()
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let mut anger_by_uuid = HashMap::new();
        if let Some(suspects) = nbt.get_list("suspects") {
            for suspect in suspects {
                let NbtTag::Compound(suspect) = suspect else {
                    continue;
                };
                let Some(uuid) = suspect.get_uuid("uuid") else {
                    continue;
                };
                let Some(anger) = suspect.get_int("anger").filter(|anger| *anger >= 0) else {
                    continue;
                };
                anger_by_uuid.insert(uuid, anger);
            }
        }
        Self::new(anger_by_uuid)
    }

    #[must_use]
    pub fn to_nbt(&self) -> NbtCompound {
        let pairs = self
            .suspects
            .iter()
            .map(|suspect| {
                (
                    suspect.get_entity().entity_uuid,
                    self.anger_of(suspect.as_ref()),
                )
            })
            .chain(
                self.anger_by_uuid
                    .iter()
                    .map(|(uuid, anger)| (*uuid, *anger)),
            );
        let suspects = pairs
            .map(|(uuid, anger)| {
                let mut suspect = NbtCompound::new();
                suspect.put_uuid("uuid", uuid);
                suspect.put_int("anger", anger);
                NbtTag::Compound(suspect)
            })
            .collect();
        let mut nbt = NbtCompound::new();
        nbt.put("suspects", NbtTag::List(suspects));
        nbt
    }
}

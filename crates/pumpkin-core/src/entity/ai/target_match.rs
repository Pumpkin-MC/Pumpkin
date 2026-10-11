use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Tag, Taggable};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::world::World;

/// Which entity types a target or threat search accepts.
#[derive(Clone, Copy, Debug)]
pub enum TargetMatch {
    /// Every type. The goal's predicate does the filtering.
    Any,
    Type(&'static EntityType),
    AnyOf(&'static [&'static EntityType]),
    Tag(&'static Tag),
    /// Every type outside the tag. e.g. Wither: `wither_friends`.
    AllExcept(&'static Tag),
}

impl TargetMatch {
    #[must_use]
    pub fn matches(self, entity_type: &EntityType) -> bool {
        match self {
            Self::Any => true,
            Self::Type(t) => t == entity_type,
            Self::AnyOf(types) => types.contains(&entity_type),
            Self::Tag(tag) => entity_type.has_tag(tag),
            Self::AllExcept(tag) => !entity_type.has_tag(tag),
        }
    }

    /// Only players can match -> the search can use the player list.
    #[must_use]
    pub fn is_player(self) -> bool {
        matches!(self, Self::Type(t) if t == &EntityType::PLAYER)
    }

    /// Nearest matching entity within `radius` that passes `accept`.
    /// Players live in their own list, so it is searched whenever they can match.
    pub fn find_nearest(
        self,
        world: &World,
        pos: Vector3<f64>,
        radius: f64,
        accept: impl Fn(&dyn EntityBase) -> bool,
    ) -> Option<Arc<dyn EntityBase>> {
        let player = self
            .matches(&EntityType::PLAYER)
            .then(|| world.get_nearest_player(pos, radius, |player| accept(player.as_ref())))
            .flatten()
            .map(|player| player as Arc<dyn EntityBase>);
        if self.is_player() {
            return player;
        }
        let entity = world.get_nearest_entity(pos, radius, None, |entity| {
            self.matches(entity.get_entity().entity_type) && accept(entity.as_ref())
        });
        match (player, entity) {
            (Some(player), Some(entity)) => {
                let dist = |e: &Arc<dyn EntityBase>| {
                    e.get_entity().pos.load().squared_distance_to_vec(&pos)
                };
                Some(if dist(&player) <= dist(&entity) {
                    player
                } else {
                    entity
                })
            }
            (player, entity) => player.or(entity),
        }
    }
}

/// Mob families vanilla goals refer to by class. A tag is used only where it has
/// exactly the class's members: `skeletons` adds the skeleton horse, `zombies` adds horses and zoglin.
impl TargetMatch {
    pub const UNDEAD: Self = Self::Tag(&tag::EntityType::MINECRAFT_UNDEAD);
    /// `AbstractIllager`.
    pub const ILLAGER: Self = Self::Tag(&tag::EntityType::MINECRAFT_ILLAGER);
    pub const CAT_LIKE: Self = Self::AnyOf(&[&EntityType::CAT, &EntityType::OCELOT]);
    /// `AbstractSchoolingFish`: the ones that swim in schools.
    pub const FISH: Self = Self::AnyOf(&[
        &EntityType::COD,
        &EntityType::SALMON,
        &EntityType::TROPICAL_FISH,
    ]);
    pub const GUARDIAN_LIKE: Self =
        Self::AnyOf(&[&EntityType::GUARDIAN, &EntityType::ELDER_GUARDIAN]);
    pub const SQUID_LIKE: Self = Self::AnyOf(&[&EntityType::SQUID, &EntityType::GLOW_SQUID]);
    /// `AbstractVillager`.
    pub const VILLAGER_LIKE: Self =
        Self::AnyOf(&[&EntityType::VILLAGER, &EntityType::WANDERING_TRADER]);
    /// `Zombie`.
    pub const ZOMBIE_LIKE: Self = Self::AnyOf(&[
        &EntityType::ZOMBIE,
        &EntityType::HUSK,
        &EntityType::DROWNED,
        &EntityType::ZOMBIE_VILLAGER,
        &EntityType::ZOMBIFIED_PIGLIN,
    ]);
    /// `AbstractSkeleton`.
    pub const SKELETON_LIKE: Self = Self::AnyOf(&[
        &EntityType::SKELETON,
        &EntityType::STRAY,
        &EntityType::WITHER_SKELETON,
        &EntityType::BOGGED,
        &EntityType::PARCHED,
    ]);
    /// `AbstractPiglin`.
    pub const PIGLIN_LIKE: Self = Self::AnyOf(&[&EntityType::PIGLIN, &EntityType::PIGLIN_BRUTE]);
    /// A piglin's nemesis: wither skeleton or wither, whichever is closest.
    pub const WITHER_LIKE: Self = Self::AnyOf(&[&EntityType::WITHER_SKELETON, &EntityType::WITHER]);
    /// `Llama`.
    pub const LLAMA_LIKE: Self = Self::AnyOf(&[&EntityType::LLAMA, &EntityType::TRADER_LLAMA]);
}

impl From<&'static EntityType> for TargetMatch {
    fn from(entity_type: &'static EntityType) -> Self {
        Self::Type(entity_type)
    }
}

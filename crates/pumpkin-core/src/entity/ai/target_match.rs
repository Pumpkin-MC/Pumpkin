use std::sync::Arc;

use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_data::tag::{self, Tag, Taggable};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{Entity, EntityBase};
use crate::world::{World, entity_islands};

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
    /// Vanilla `Enemy`: every monster but the undead mounts and the sulfur cube.
    Enemy,
    /// Vanilla `Monster.class`: an [`Self::Enemy`] that is not a slime, ghast, phantom,
    /// shulker, hoglin or the dragon.
    Monster,
}

/// Monster category, but not vanilla `Enemy`.
const NOT_ENEMY: &[&EntityType] = &[
    &EntityType::CAMEL_HUSK,
    &EntityType::ZOMBIE_HORSE,
    &EntityType::ZOMBIE_NAUTILUS,
    &EntityType::SULFUR_CUBE,
];

/// Vanilla `Enemy` outside the `Monster` class.
const ENEMY_NOT_MONSTER: &[&EntityType] = &[
    &EntityType::ENDER_DRAGON,
    &EntityType::GHAST,
    &EntityType::HOGLIN,
    &EntityType::MAGMA_CUBE,
    &EntityType::PHANTOM,
    &EntityType::SHULKER,
    &EntityType::SLIME,
];

fn is_enemy(entity_type: &EntityType) -> bool {
    entity_type.category == &MobCategory::MONSTER && !NOT_ENEMY.contains(&entity_type)
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
            Self::Enemy => is_enemy(entity_type),
            Self::Monster => is_enemy(entity_type) && !ENEMY_NOT_MONSTER.contains(&entity_type),
        }
    }

    /// Only players can match -> the search can use the player list.
    #[must_use]
    pub fn is_player(self) -> bool {
        matches!(self, Self::Type(t) if t == &EntityType::PLAYER)
    }

    /// Nearest matching entity in `area` that passes `accept`.
    /// Players live in their own list, so it is searched whenever they can match.
    pub fn find_nearest(
        self,
        world: &World,
        area: &SearchArea,
        accept: impl Fn(&dyn EntityBase) -> bool,
    ) -> Option<Arc<dyn EntityBase>> {
        let dist = |entity: &dyn EntityBase| {
            entity
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&area.origin)
        };
        let player = self
            .matches(&EntityType::PLAYER)
            .then(|| {
                let players = if area.players_anywhere {
                    world.players.load_full().to_vec()
                } else {
                    world.get_players_at_box(&area.area)
                };
                players
                    .into_iter()
                    .filter(|player| {
                        let accepted = accept(player.as_ref());
                        if accepted && area.players_anywhere {
                            entity_islands::check_reached(player.entity_id());
                        }
                        accepted
                    })
                    .min_by(|a, b| dist(a.as_ref()).total_cmp(&dist(b.as_ref())))
                    .map(|player| player as Arc<dyn EntityBase>)
            })
            .flatten();
        if self.is_player() {
            return player;
        }
        // The spatial index keeps list order, so ties go to the first like vanilla.
        let entity = world
            .get_entities_at_box(&area.area)
            .into_iter()
            .filter(|entity| {
                self.matches(entity.get_entity().entity_type) && accept(entity.as_ref())
            })
            .min_by(|a, b| dist(a.as_ref()).total_cmp(&dist(b.as_ref())));
        match (player, entity) {
            (Some(player), Some(entity)) => {
                Some(if dist(player.as_ref()) <= dist(entity.as_ref()) {
                    player
                } else {
                    entity
                })
            }
            (player, entity) => player.or(entity),
        }
    }
}

/// Vanilla's candidate box before the targeting test.
#[derive(Clone, Copy)]
pub struct SearchArea {
    /// The nearest candidate to this point wins.
    pub origin: Vector3<f64>,
    pub area: BoundingBox,
    /// Vanilla `getNearestPlayer` has no box: only the targeting test limits players.
    pub players_anywhere: bool,
}

impl SearchArea {
    /// The mob's box grown by `horizontal` and `vertical`, ranked from its eyes.
    #[must_use]
    pub fn from_eyes(entity: &Entity, horizontal: f64, vertical: f64) -> Self {
        let mut area = Self::from_feet(entity, horizontal, vertical);
        area.origin.y += f64::from(entity.entity_dimension.load().eye_height);
        area
    }

    /// The mob's box grown by `horizontal` and `vertical`, ranked from its feet.
    #[must_use]
    pub fn from_feet(entity: &Entity, horizontal: f64, vertical: f64) -> Self {
        Self {
            origin: entity.pos.load(),
            area: entity
                .bounding_box
                .load()
                .expand(horizontal, vertical, horizontal),
            players_anywhere: false,
        }
    }

    /// Players are not limited by the box, vanilla `getNearestPlayer`.
    #[must_use]
    pub const fn players_anywhere(mut self) -> Self {
        self.players_anywhere = true;
        self
    }

    #[must_use]
    pub fn holds(&self, candidate: &dyn EntityBase) -> bool {
        candidate
            .get_entity()
            .bounding_box
            .load()
            .intersects(&self.area)
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

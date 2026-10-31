use pumpkin_data::entity::MobCategory;

/// How a mob treats players by default.
///
/// Classification only, not the vanilla `Enemy` check
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Temperament {
    Passive,
    Neutral,
    Hostile,
}

impl Temperament {
    /// A grudge outranks the category: the neutral monsters are `MONSTER` too.
    #[must_use]
    pub fn classify(is_neutral: bool, category: &MobCategory) -> Self {
        if is_neutral {
            Self::Neutral
        } else if category == &MobCategory::MONSTER {
            Self::Hostile
        } else {
            Self::Passive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Temperament;
    use pumpkin_data::entity::MobCategory;

    #[test]
    fn neutral_monster_is_neutral() {
        assert_eq!(
            Temperament::classify(true, &MobCategory::MONSTER),
            Temperament::Neutral
        );
    }

    #[test]
    fn neutral_creature_is_neutral() {
        assert_eq!(
            Temperament::classify(true, &MobCategory::CREATURE),
            Temperament::Neutral
        );
    }

    #[test]
    fn monster_is_hostile() {
        assert_eq!(
            Temperament::classify(false, &MobCategory::MONSTER),
            Temperament::Hostile
        );
    }

    #[test]
    fn everything_else_is_passive() {
        for category in [
            &MobCategory::CREATURE,
            &MobCategory::AMBIENT,
            &MobCategory::WATER_CREATURE,
            &MobCategory::WATER_AMBIENT,
            &MobCategory::UNDERGROUND_WATER_CREATURE,
            &MobCategory::AXOLOTLS,
            &MobCategory::MISC,
        ] {
            assert_eq!(Temperament::classify(false, category), Temperament::Passive);
        }
    }
}

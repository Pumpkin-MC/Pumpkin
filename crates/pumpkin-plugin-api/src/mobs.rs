//! Specialized typed wrappers for major Minecraft mob entities.
//!
//! Provides typed accessors and ergonomic helpers (`Sheep::from_entity`, `Wolf::from_mob`, etc.)
//! without needing dozens of WIT resources.

use std::ops::Deref;

pub use crate::wit::pumpkin::plugin::uuid::Uuid;
pub use crate::wit::pumpkin::plugin::world::{
    AgeableData, BlockDirection, CatData, CreeperData, DyeColor, EndermanData, Entity, FoxData,
    IronGolemData, LivingEntity, Mob, MobData, SheepData, ShulkerData, SlimeData, VillagerData,
    VillagerProfession, WolfData, ZombieData,
};

/// Trait implemented by all specialized mob wrappers to allow generic downcasting via `.cast::<T>()`.
pub trait MobCast<'a>: Sized {
    /// Attempts to wrap a [`Mob`] reference if the underlying entity data matches.
    fn from_mob(mob: &'a Mob) -> Option<Self>;

    /// Attempts to wrap an [`Entity`] reference if it is an AI mob matching this type.
    fn from_entity(entity: &'a Entity) -> Option<Self> {
        let mob = entity.as_mob()?;
        // Note: as_mob() returns a new Resource handle, so we extract via MobData check.
        Self::from_mob_owned(mob)
    }

    /// Attempts to wrap a [`LivingEntity`] reference if it is an AI mob matching this type.
    fn from_living(living: &'a LivingEntity) -> Option<Self> {
        let mob = living.as_mob()?;
        Self::from_mob_owned(mob)
    }

    #[doc(hidden)]
    fn from_mob_owned(mob: Mob) -> Option<Self>;
}

macro_rules! define_mob_wrapper {
    (
        $(#[$meta:meta])*
        $name:ident, $variant:ident, $data_ty:ident
    ) => {
        $(#[$meta])*
        pub struct $name<'a> {
            mob: &'a Mob,
            _owned: Option<Mob>,
        }

        impl<'a> $name<'a> {
            /// Wraps a borrowed [`Mob`] reference if this mob is of the matching type.
            #[must_use]
            pub fn from_mob(mob: &'a Mob) -> Option<Self> {
                if matches!(mob.get_mob_data(), MobData::$variant(_)) {
                    Some(Self { mob, _owned: None })
                } else {
                    None
                }
            }

            /// Wraps a borrowed [`Entity`] reference if it is an AI mob of the matching type.
            #[must_use]
            pub fn from_entity(entity: &'a Entity) -> Option<Self> {
                let mob = entity.as_mob()?;
                Self::from_mob_owned(mob)
            }

            /// Wraps a borrowed [`LivingEntity`] reference if it is an AI mob of the matching type.
            #[must_use]
            pub fn from_living(living: &'a LivingEntity) -> Option<Self> {
                let mob = living.as_mob()?;
                Self::from_mob_owned(mob)
            }

            fn from_mob_owned(mob: Mob) -> Option<Self> {
                if matches!(mob.get_mob_data(), MobData::$variant(_)) {
                    // Safe reference extension to the owned Mob
                    let mob_ref = unsafe { &*(&mob as *const Mob) };
                    Some(Self {
                        mob: mob_ref,
                        _owned: Some(mob),
                    })
                } else {
                    None
                }
            }

            /// Retrieves the underlying data record for this mob.
            #[must_use]
            pub fn get_data(&self) -> Option<$data_ty> {
                match self.mob.get_mob_data() {
                    MobData::$variant(data) => Some(data),
                    _ => None,
                }
            }

            /// Updates the underlying data record for this mob.
            pub fn set_data(&self, data: $data_ty) -> bool {
                self.mob.set_mob_data(MobData::$variant(data))
            }
        }

        impl<'a> Deref for $name<'a> {
            type Target = Mob;

            fn deref(&self) -> &Self::Target {
                self.mob
            }
        }

        impl<'a> TryFrom<&'a Mob> for $name<'a> {
            type Error = ();

            fn try_from(mob: &'a Mob) -> Result<Self, Self::Error> {
                Self::from_mob(mob).ok_or(())
            }
        }

        impl<'a> TryFrom<&'a Entity> for $name<'a> {
            type Error = ();

            fn try_from(entity: &'a Entity) -> Result<Self, Self::Error> {
                Self::from_entity(entity).ok_or(())
            }
        }

        impl<'a> TryFrom<&'a LivingEntity> for $name<'a> {
            type Error = ();

            fn try_from(living: &'a LivingEntity) -> Result<Self, Self::Error> {
                Self::from_living(living).ok_or(())
            }
        }

        impl<'a> MobCast<'a> for $name<'a> {
            fn from_mob(mob: &'a Mob) -> Option<Self> {
                Self::from_mob(mob)
            }

            fn from_living(living: &'a LivingEntity) -> Option<Self> {
                Self::from_living(living)
            }

            fn from_mob_owned(mob: Mob) -> Option<Self> {
                Self::from_mob_owned(mob)
            }
        }
    };
}

define_mob_wrapper!(
    /// Specialized wrapper for Sheep entities.
    Sheep, Sheep, SheepData
);

impl<'a> Sheep<'a> {
    /// Gets the fleece dye color of this sheep.
    #[must_use]
    pub fn get_color(&self) -> DyeColor {
        self.get_data().map_or(DyeColor::White, |d| d.color)
    }

    /// Sets the fleece dye color of this sheep.
    pub fn set_color(&self, color: DyeColor) {
        if let Some(mut data) = self.get_data() {
            data.color = color;
            self.set_data(data);
        }
    }

    /// Returns whether this sheep has been sheared.
    #[must_use]
    pub fn is_sheared(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_sheared)
    }

    /// Sets whether this sheep is sheared.
    pub fn set_sheared(&self, sheared: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_sheared = sheared;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Wolf entities.
    Wolf, Wolf, WolfData
);

impl<'a> Wolf<'a> {
    /// Returns whether this wolf is tamed.
    #[must_use]
    pub fn is_tamed(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_tamed)
    }

    /// Sets whether this wolf is tamed.
    pub fn set_tamed(&self, tamed: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_tamed = tamed;
            self.set_data(data);
        }
    }

    /// Gets the UUID of the player who owns this wolf, if any.
    #[must_use]
    pub fn get_owner(&self) -> Option<Uuid> {
        self.get_data().and_then(|d| d.owner)
    }

    /// Sets the owner of this wolf by UUID.
    pub fn set_owner(&self, owner: Option<Uuid>) {
        if let Some(mut data) = self.get_data() {
            data.owner = owner;
            self.set_data(data);
        }
    }

    /// Returns whether this wolf is currently in a sitting pose.
    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_sitting)
    }

    /// Orders this wolf to sit or stand.
    pub fn set_sitting(&self, sitting: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_sitting = sitting;
            self.set_data(data);
        }
    }

    /// Gets the collar dye color of this wolf.
    #[must_use]
    pub fn get_collar_color(&self) -> DyeColor {
        self.get_data().map_or(DyeColor::Red, |d| d.collar_color)
    }

    /// Sets the collar dye color of this wolf.
    pub fn set_collar_color(&self, color: DyeColor) {
        if let Some(mut data) = self.get_data() {
            data.collar_color = color;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Cat entities.
    Cat, Cat, CatData
);

impl<'a> Cat<'a> {
    /// Returns whether this cat is tamed.
    #[must_use]
    pub fn is_tamed(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_tamed)
    }

    /// Sets whether this cat is tamed.
    pub fn set_tamed(&self, tamed: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_tamed = tamed;
            self.set_data(data);
        }
    }

    /// Gets the owner UUID of this cat.
    #[must_use]
    pub fn get_owner(&self) -> Option<Uuid> {
        self.get_data().and_then(|d| d.owner)
    }

    /// Sets the owner UUID of this cat.
    pub fn set_owner(&self, owner: Option<Uuid>) {
        if let Some(mut data) = self.get_data() {
            data.owner = owner;
            self.set_data(data);
        }
    }

    /// Returns whether this cat is in a sitting pose.
    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_sitting)
    }

    /// Orders this cat to sit or stand.
    pub fn set_sitting(&self, sitting: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_sitting = sitting;
            self.set_data(data);
        }
    }

    /// Gets the collar dye color of this cat.
    #[must_use]
    pub fn get_collar_color(&self) -> DyeColor {
        self.get_data().map_or(DyeColor::Red, |d| d.collar_color)
    }

    /// Sets the collar dye color of this cat.
    pub fn set_collar_color(&self, color: DyeColor) {
        if let Some(mut data) = self.get_data() {
            data.collar_color = color;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Villager entities.
    Villager, Villager, VillagerData
);

impl<'a> Villager<'a> {
    /// Gets the profession of this villager.
    #[must_use]
    pub fn get_profession(&self) -> VillagerProfession {
        self.get_data()
            .map_or(VillagerProfession::None, |d| d.profession)
    }

    /// Sets the profession of this villager.
    pub fn set_profession(&self, profession: VillagerProfession) {
        if let Some(mut data) = self.get_data() {
            data.profession = profession;
            self.set_data(data);
        }
    }

    /// Gets the trading level of this villager (1-5).
    #[must_use]
    pub fn get_level(&self) -> u8 {
        self.get_data().map_or(1, |d| d.level)
    }

    /// Sets the trading level of this villager (1-5).
    pub fn set_level(&self, level: u8) {
        if let Some(mut data) = self.get_data() {
            data.level = level;
            self.set_data(data);
        }
    }

    /// Gets the trading experience points of this villager.
    #[must_use]
    pub fn get_experience(&self) -> u32 {
        self.get_data().map_or(0, |d| d.experience)
    }

    /// Sets the trading experience points of this villager.
    pub fn set_experience(&self, experience: u32) {
        if let Some(mut data) = self.get_data() {
            data.experience = experience;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Creeper entities.
    Creeper, Creeper, CreeperData
);

impl<'a> Creeper<'a> {
    /// Returns whether this creeper is charged (struck by lightning).
    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_powered)
    }

    /// Sets whether this creeper is powered/charged.
    pub fn set_powered(&self, powered: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_powered = powered;
            self.set_data(data);
        }
    }

    /// Gets the fuse duration of this creeper in ticks.
    #[must_use]
    pub fn get_fuse(&self) -> i32 {
        self.get_data().map_or(30, |d| d.fuse)
    }

    /// Sets the fuse duration of this creeper in ticks.
    pub fn set_fuse(&self, fuse: i32) {
        if let Some(mut data) = self.get_data() {
            data.fuse = fuse;
            self.set_data(data);
        }
    }

    /// Returns whether this creeper has been manually ignited with flint and steel.
    #[must_use]
    pub fn is_ignited(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_ignited)
    }

    /// Sets whether this creeper is ignited.
    pub fn set_ignited(&self, ignited: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_ignited = ignited;
            self.set_data(data);
        }
    }

    /// Gets the explosion radius of this creeper.
    #[must_use]
    pub fn get_explosion_radius(&self) -> u8 {
        self.get_data().map_or(3, |d| d.explosion_radius)
    }

    /// Sets the explosion radius of this creeper.
    pub fn set_explosion_radius(&self, radius: u8) {
        if let Some(mut data) = self.get_data() {
            data.explosion_radius = radius;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Slime and Magma Cube entities.
    Slime, Slime, SlimeData
);

impl<'a> Slime<'a> {
    /// Gets the size scale of this slime.
    #[must_use]
    pub fn get_size(&self) -> i32 {
        self.get_data().map_or(1, |d| d.size)
    }

    /// Sets the size scale of this slime.
    pub fn set_size(&self, size: i32) {
        if let Some(mut data) = self.get_data() {
            data.size = size;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Enderman entities.
    Enderman, Enderman, EndermanData
);

impl<'a> Enderman<'a> {
    /// Gets the numerical block state ID carried by this enderman, if any.
    #[must_use]
    pub fn get_carried_block(&self) -> Option<u16> {
        self.get_data().and_then(|d| d.carried_block_state)
    }

    /// Sets the carried block state ID for this enderman.
    pub fn set_carried_block(&self, block_state: Option<u16>) {
        if let Some(mut data) = self.get_data() {
            data.carried_block_state = block_state;
            self.set_data(data);
        }
    }

    /// Returns whether this enderman is currently screaming (angry).
    #[must_use]
    pub fn is_screaming(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_screaming)
    }

    /// Returns whether this enderman is staring at a player.
    #[must_use]
    pub fn is_staring(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_staring)
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Iron Golem entities.
    IronGolem, IronGolem, IronGolemData
);

impl<'a> IronGolem<'a> {
    /// Returns whether this iron golem was created by a player.
    #[must_use]
    pub fn is_player_created(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_player_created)
    }

    /// Sets whether this iron golem is considered player-created.
    pub fn set_player_created(&self, created: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_player_created = created;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Fox entities.
    Fox, Fox, FoxData
);

impl<'a> Fox<'a> {
    /// Returns whether this fox is sitting.
    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_sitting)
    }

    /// Sets whether this fox is sitting.
    pub fn set_sitting(&self, sitting: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_sitting = sitting;
            self.set_data(data);
        }
    }

    /// Returns whether this fox is sleeping.
    #[must_use]
    pub fn is_sleeping(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_sleeping)
    }

    /// Sets whether this fox is sleeping.
    pub fn set_sleeping(&self, sleeping: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_sleeping = sleeping;
            self.set_data(data);
        }
    }

    /// Returns whether this fox is crouching.
    #[must_use]
    pub fn is_crouching(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_crouching)
    }

    /// Sets whether this fox is crouching.
    pub fn set_crouching(&self, crouching: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_crouching = crouching;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Shulker entities.
    Shulker, Shulker, ShulkerData
);

impl<'a> Shulker<'a> {
    /// Gets the attached block direction of this shulker.
    #[must_use]
    pub fn get_attached_face(&self) -> BlockDirection {
        self.get_data()
            .map_or(BlockDirection::Down, |d| d.attached_face)
    }

    /// Sets the attached block direction of this shulker.
    pub fn set_attached_face(&self, face: BlockDirection) {
        if let Some(mut data) = self.get_data() {
            data.attached_face = face;
            self.set_data(data);
        }
    }

    /// Gets the raw peek amount of this shulker (0-100).
    #[must_use]
    pub fn get_peek_amount(&self) -> u8 {
        self.get_data().map_or(0, |d| d.peek_amount)
    }

    /// Sets the raw peek amount of this shulker (0-100).
    pub fn set_peek_amount(&self, amount: u8) {
        if let Some(mut data) = self.get_data() {
            data.peek_amount = amount;
            self.set_data(data);
        }
    }

    /// Gets the custom dye color of this shulker, if dyed.
    #[must_use]
    pub fn get_color(&self) -> Option<DyeColor> {
        self.get_data().and_then(|d| d.color)
    }

    /// Sets or removes the custom dye color of this shulker.
    pub fn set_color(&self, color: Option<DyeColor>) {
        if let Some(mut data) = self.get_data() {
            data.color = color;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Zombie entities.
    Zombie, Zombie, ZombieData
);

impl<'a> Zombie<'a> {
    /// Returns whether this zombie is a baby.
    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_baby)
    }

    /// Sets whether this zombie is a baby.
    pub fn set_baby(&self, baby: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_baby = baby;
            self.set_data(data);
        }
    }

    /// Returns whether this zombie is capable of breaking doors.
    #[must_use]
    pub fn can_break_doors(&self) -> bool {
        self.get_data().is_some_and(|d| d.can_break_doors)
    }

    /// Sets whether this zombie can break doors.
    pub fn set_can_break_doors(&self, can_break: bool) {
        if let Some(mut data) = self.get_data() {
            data.can_break_doors = can_break;
            self.set_data(data);
        }
    }
}

define_mob_wrapper!(
    /// Specialized wrapper for Ageable animal mobs (cows, pigs, chickens, rabbits, etc.).
    Ageable, Ageable, AgeableData
);

impl<'a> Ageable<'a> {
    /// Returns whether this animal is a baby.
    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.get_data().is_some_and(|d| d.is_baby)
    }

    /// Sets whether this animal is a baby.
    pub fn set_baby(&self, baby: bool) {
        if let Some(mut data) = self.get_data() {
            data.is_baby = baby;
            self.set_data(data);
        }
    }

    /// Gets the age of this animal in ticks (negative for babies).
    #[must_use]
    pub fn get_age(&self) -> i32 {
        self.get_data().map_or(0, |d| d.age)
    }

    /// Sets the age of this animal in ticks.
    pub fn set_age(&self, age: i32) {
        if let Some(mut data) = self.get_data() {
            data.age = age;
            self.set_data(data);
        }
    }
}

/// Extension trait providing generic `.cast::<T>()` downcasting on [`Entity`] and [`Mob`].
pub trait EntityCastExt {
    /// Attempts to cast this entity or mob reference to a specialized mob wrapper type.
    fn cast<'a, T: MobCast<'a>>(&'a self) -> Option<T>;
}

impl EntityCastExt for Mob {
    fn cast<'a, T: MobCast<'a>>(&'a self) -> Option<T> {
        T::from_mob(self)
    }
}

impl EntityCastExt for Entity {
    fn cast<'a, T: MobCast<'a>>(&'a self) -> Option<T> {
        T::from_entity(self)
    }
}

impl EntityCastExt for LivingEntity {
    fn cast<'a, T: MobCast<'a>>(&'a self) -> Option<T> {
        T::from_living(self)
    }
}

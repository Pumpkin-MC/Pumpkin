use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{
    entity::EntityType, item::Item, meta_data_type::MetaDataType, tracked_data::TrackedData,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::{RngExt, rng};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, eat_grass::EatGrassGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity},
    player::Player,
};
use crate::world::World;
use crate::world::game_event::{GameEventContext, emit_game_event};

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;

use super::animal::Animal;
use crate::block::entities::sign::DyeColor;

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

/// Sentinel meaning "no color rolled yet" -- `mob_init_data_tracker` rolls a biome-based color
/// the first time it sees this; NBT restore always writes an in-range color first, so a
/// freshly-loaded sheep never observes this value.
const COLOR_UNSET: u8 = 0xFF;

/// Vanilla `SheepColorSpawnRules.getSheepColor`: biome-tag-gated weighted color table used at
/// natural spawn time. Each configuration is a weighted pick over 5 entries (weights out of
/// 100); the last entry (weight 82) is itself a nested weighted pick, 499:1 in favor of that
/// configuration's dominant color over `PINK`.
fn get_random_sheep_color(world: &World, pos: pumpkin_util::math::position::BlockPos) -> u8 {
    let biome = world.get_biome(&pos);
    let mut r = rng();

    if biome.has_tag(&tag::WorldgenBiome::MINECRAFT_SPAWNS_WARM_VARIANT_FARM_ANIMALS) {
        weighted_sheep_color(
            &mut r,
            DyeColor::Gray,
            DyeColor::LightGray,
            DyeColor::White,
            DyeColor::Black,
            DyeColor::Brown,
        )
    } else if biome.has_tag(&tag::WorldgenBiome::MINECRAFT_SPAWNS_COLD_VARIANT_FARM_ANIMALS) {
        weighted_sheep_color(
            &mut r,
            DyeColor::LightGray,
            DyeColor::Gray,
            DyeColor::White,
            DyeColor::Brown,
            DyeColor::Black,
        )
    } else {
        weighted_sheep_color(
            &mut r,
            DyeColor::Black,
            DyeColor::Gray,
            DyeColor::LightGray,
            DyeColor::Brown,
            DyeColor::White,
        )
    }
}

/// Shared shape of all three `SheepColorSpawnRules` tables: `a`=5, `b`=5, `c`=5, `d`=3, then 82
/// split 499:1 between `dominant` (the 5th positional slot) and `PINK`.
fn weighted_sheep_color(
    rng: &mut impl RngExt,
    a: DyeColor,
    b: DyeColor,
    c: DyeColor,
    d: DyeColor,
    dominant: DyeColor,
) -> u8 {
    let choices: &[(DyeColor, u32)] = &[(a, 5), (b, 5), (c, 5), (d, 3), (dominant, 82)];
    let picked = super::fish_variant::pick_weighted(rng, choices);
    if picked == dominant {
        return super::fish_variant::pick_weighted(rng, &[(dominant, 499), (DyeColor::Pink, 1)])
            as u8;
    }
    picked as u8
}

fn wool_item_for_color(color: u8) -> &'static Item {
    match DyeColor::from(color as i8) {
        DyeColor::Orange => &Item::ORANGE_WOOL,
        DyeColor::Magenta => &Item::MAGENTA_WOOL,
        DyeColor::LightBlue => &Item::LIGHT_BLUE_WOOL,
        DyeColor::Yellow => &Item::YELLOW_WOOL,
        DyeColor::Lime => &Item::LIME_WOOL,
        DyeColor::Pink => &Item::PINK_WOOL,
        DyeColor::Gray => &Item::GRAY_WOOL,
        DyeColor::LightGray => &Item::LIGHT_GRAY_WOOL,
        DyeColor::Cyan => &Item::CYAN_WOOL,
        DyeColor::Purple => &Item::PURPLE_WOOL,
        DyeColor::Blue => &Item::BLUE_WOOL,
        DyeColor::Brown => &Item::BROWN_WOOL,
        DyeColor::Green => &Item::GREEN_WOOL,
        DyeColor::Red => &Item::RED_WOOL,
        DyeColor::Black => &Item::BLACK_WOOL,
        DyeColor::White => &Item::WHITE_WOOL,
    }
}

pub struct SheepEntity {
    pub mob_entity: MobEntity,
    color_and_sheared: AtomicU8,
    ageable_data: AgeableData,
}

impl SheepEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let sheep = Self {
            mob_entity,
            color_and_sheared: AtomicU8::new(COLOR_UNSET),
            ageable_data: AgeableData::default(),
        };
        let mob_arc = Arc::new(sheep);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.1, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(EatGrassGoal::default()));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn get_packed_byte(&self) -> u8 {
        self.color_and_sheared.load(Ordering::Relaxed)
    }

    /// The packed byte with the pre-spawn-init `COLOR_UNSET` sentinel normalized to `0`. Every
    /// read-modify-write on the packed byte must go through this instead of `get_packed_byte`
    /// directly -- otherwise a `set_color`/`set_sheared` call landing before
    /// `mob_init_data_tracker` has rolled a real color (e.g. `create_offspring` coloring a lamb
    /// before it's spawned) would fold `0xFF`'s garbage high bits into the result.
    fn base_byte(&self) -> u8 {
        let byte = self.get_packed_byte();
        if byte == COLOR_UNSET { 0 } else { byte }
    }

    pub fn get_color(&self) -> u8 {
        self.get_packed_byte() & 0x0F
    }

    pub fn is_sheared(&self) -> bool {
        (self.get_packed_byte() & 0x10) != 0
    }

    fn set_packed_and_sync(&self, byte: u8) {
        self.color_and_sheared.store(byte, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::WOOL_ID,
                MetaDataType::BYTE,
                byte as i8,
            )],
            None,
        );
    }

    pub fn set_color(&self, color: u8) {
        let byte = (self.base_byte() & 0xF0) | (color & 0x0F);
        self.set_packed_and_sync(byte);
    }

    pub fn set_sheared(&self, sheared: bool) {
        let byte = if sheared {
            self.base_byte() | 0x10
        } else {
            self.base_byte() & !0x10
        };
        self.set_packed_and_sync(byte);
    }

    /// Vanilla `Sheep.readyForShearing`: `!isSheared() && !isBaby()`.
    pub fn ready_for_shearing(&self) -> bool {
        !self.is_sheared()
            && self
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed)
                >= 0
    }

    /// Vanilla `Sheep.shear`: plays the shear sound, drops 1-3 wool of the sheep's color, and
    /// marks the sheep sheared. No `shear_sheep`-equivalent loot table exists in `pumpkin-data`
    /// today, so the 1-3 count and color->wool mapping are hardcoded here rather than rolled
    /// from a data-driven table.
    pub async fn shear(&self, world: &Arc<World>) {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();

        world.play_sound(Sound::EntitySheepShear, SoundCategory::Players, &pos);

        let wool_item = wool_item_for_color(self.get_color());
        let count = rng().random_range(1u32..=3);
        for _ in 0..count {
            let velocity = Vector3::new(
                (rand::random::<f64>() - 0.5) * 0.1,
                rand::random::<f64>() * 0.05 + 0.2,
                (rand::random::<f64>() - 0.5) * 0.1,
            );
            let item_entity = Arc::new(ItemEntity::new_with_velocity(
                Entity::new(world.clone(), pos, &EntityType::ITEM),
                ItemStack::new(1, wool_item),
                velocity,
                10,
            ));
            world.spawn_entity(item_entity).await;
        }

        self.set_sheared(true);
    }
}

impl NBTStorage for SheepEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            self.write_animal_nbt(nbt);
            nbt.put_bool("Sheared", self.is_sheared());
            nbt.put_byte("Color", self.get_color() as i8);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.read_animal_nbt(nbt);
            let sheared = nbt.get_bool("Sheared").unwrap_or(false);
            let color = nbt.get_byte("Color").unwrap_or(0) as u8;
            let byte = (color & 0x0F) | if sheared { 0x10 } else { 0 };
            self.color_and_sheared.store(byte, Ordering::Relaxed);
        })
    }
}

impl super::animal::Animal for SheepEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl AgeableMob for SheepEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Mob for SheepEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `Sheep.ate`: un-shears the sheep and, if it's a baby, ages it up by 60 seconds
    /// (60 * 20 ticks), same as `Animal.ate` -> `AgeableMob.ageUp(int)`.
    fn on_eating_grass(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            self.set_sheared(false);
            if AgeableMob::can_age_up(self) {
                self.age_up(60, false);
            }
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;

            if self.color_and_sheared.load(Ordering::Relaxed) == COLOR_UNSET {
                let world = entity.world.load();
                let pos = entity.block_pos.load();
                let color = get_random_sheep_color(&world, pos);
                self.set_packed_and_sync(color & 0x0F);
            } else {
                // NBT restore already loaded a valid color; just resend it so the client has
                // the up-to-date tracked value.
                self.set_packed_and_sync(self.get_packed_byte());
            }

            // This override replaces (rather than chains to) `Mob::mob_init_data_tracker`'s
            // default body, which sends `BABY_ID` for age < 0 -- replicate that here so bred
            // lambs (spawned at age -24000) still render baby-sized.
            if entity.age.load(Ordering::Relaxed) < 0 {
                entity.send_meta_data(
                    &[pumpkin_protocol::java::client::play::Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.is_shears() {
                if self.ready_for_shearing() {
                    let world = self.mob_entity.living_entity.entity.world.load();
                    let pos = self.mob_entity.living_entity.entity.pos.load();
                    self.shear(&world).await;
                    emit_game_event(
                        &world,
                        GameEvent::Shear,
                        pos,
                        GameEventContext::of_entity(player.clone()),
                    )
                    .await;
                    let _ = item_stack.damage_item(1);
                }
                // Vanilla returns `InteractionResult.CONSUME` either way (sheared/baby just
                // blocks the default animal-interact path with no drop).
                return true;
            }

            self.animal_interact(player, item_stack, Sound::EntitySheepAmbient)
                .await
        })
    }

    /// Vanilla `Sheep.getBreedOffspring`: colors the lamb via `DyeColor.getMixedColor` of the
    /// two parents.
    fn create_offspring<'a>(
        &'a self,
        mate: &'a dyn EntityBase,
        world: &'a Arc<World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>> {
        Box::pin(async move {
            let entity = self.get_entity();
            let baby = crate::entity::r#type::from_type(
                entity.entity_type,
                entity.pos.load(),
                world,
                uuid::Uuid::new_v4(),
            );

            if let Some(lamb) = baby.cast_any().downcast_ref::<Self>()
                && let Some(mate_sheep) = mate.cast_any().downcast_ref::<Self>()
            {
                let a = DyeColor::from(self.get_color() as i8);
                let b = DyeColor::from(mate_sheep.get_color() as i8);
                let mixed = DyeColor::mixed_color(a, b, &mut rng());
                lamb.set_color(mixed as u8);
            }

            Some(baby)
        })
    }
}

#[cfg(test)]
mod weighted_sheep_color_tests {
    use super::{DyeColor, weighted_sheep_color};
    use rand::rng;

    #[test]
    fn only_produces_colors_from_the_configured_table() {
        let mut r = rng();
        for _ in 0..500 {
            let color = DyeColor::from(weighted_sheep_color(
                &mut r,
                DyeColor::Black,
                DyeColor::Gray,
                DyeColor::LightGray,
                DyeColor::Brown,
                DyeColor::White,
            ) as i8);
            assert!(matches!(
                color,
                DyeColor::Black
                    | DyeColor::Gray
                    | DyeColor::LightGray
                    | DyeColor::Brown
                    | DyeColor::White
                    | DyeColor::Pink
            ));
        }
    }

    #[test]
    fn dominant_color_vastly_outweighs_pink_within_its_slot() {
        // The dominant/pink split is 499:1, so over many samples pink should be rare but not
        // literally unreachable.
        let mut r = rng();
        let mut pink_count = 0;
        let mut dominant_count = 0;
        for _ in 0..20_000 {
            let color = DyeColor::from(weighted_sheep_color(
                &mut r,
                DyeColor::Black,
                DyeColor::Gray,
                DyeColor::LightGray,
                DyeColor::Brown,
                DyeColor::White,
            ) as i8);
            if color == DyeColor::Pink {
                pink_count += 1;
            } else if color == DyeColor::White {
                dominant_count += 1;
            }
        }
        assert!(dominant_count > pink_count * 50);
    }
}

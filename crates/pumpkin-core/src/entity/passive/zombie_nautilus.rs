use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};
use uuid::Uuid;

use pumpkin_data::{
    BlockState,
    data_component::DataComponent,
    data_component_impl::{DataComponentImpl, EquipmentSlot, ZombieNautilusVariantImpl},
    item::Item,
    item_stack::ItemStack,
    sound::Sound,
    tag::{self, Taggable},
    tracked_data,
    zombie_nautilus_variant::ZombieNautilusVariant,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;

use crate::{
    entity::{
        Entity, EntityBase,
        ageable::{AgeableData, AgeableMob},
        ai::{
            goal::tempt::TemptGoal,
            pathfinder::{Navigator, node::PathType},
        },
        custom_sound::CustomSound,
        mob::{
            Mob, MobEntity,
            spawn::{EntitySpawnReason, SpawnGroupData},
        },
        passive::{
            animal::Animal,
            nautilus::{
                interact_abstract_nautilus, nautilus_can_use_slot, nautilus_equip_sound,
                tick_abstract_nautilus,
            },
            tamable::{TamableAnimal, TamableData},
        },
        player::Player,
    },
    world::World,
};

pub struct ZombieNautilusEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub tamable_data: TamableData,
    pub is_dashing: AtomicBool,
    pub dash_cooldown: AtomicI32,
    pub inventory: Mutex<Vec<ItemStack>>,
    pub variant: AtomicU8,
}

impl ZombieNautilusEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *navigator = Navigator::water_bound(false);
        navigator.set_pathfinding_malus(PathType::Water, 0.0);

        let nautilus = Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            tamable_data: TamableData::default(),
            is_dashing: AtomicBool::new(false),
            dash_cooldown: AtomicI32::new(0),
            inventory: Mutex::new(vec![ItemStack::new(0, &Item::AIR); 9]),
            variant: AtomicU8::new(ZombieNautilusVariant::default().id()),
        });

        nautilus.mob_entity.goals_selector.add_goal(
            1,
            Box::new(TemptGoal::with_tag(
                0.9,
                &tag::Item::MINECRAFT_NAUTILUS_FOOD,
                false,
            )),
        );

        nautilus
    }

    #[must_use]
    pub fn is_dashing(&self) -> bool {
        self.is_dashing.load(Ordering::Relaxed)
    }

    pub fn set_dashing(&self, dashing: bool) {
        self.is_dashing.store(dashing, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::zombie_nautilus::DASH, dashing);
    }

    pub fn set_variant(&self, variant: ZombieNautilusVariant) {
        self.set_variant_id(variant.id());
    }

    pub fn set_variant_id(&self, raw_id: u8) {
        self.variant.store(raw_id, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            tracked_data::zombie_nautilus::DATA_VARIANT_ID,
            VarInt(i32::from(raw_id)),
        );
    }

    #[must_use]
    pub fn variant_name(&self) -> String {
        let variant_id = u32::from(self.variant.load(Ordering::Relaxed));
        self.resolve_dynamic_entry_id("minecraft:zombie_nautilus_variant", variant_id, |id| {
            let variant = ZombieNautilusVariant::from_id(id as u8).unwrap_or_default();
            format!("minecraft:{}", variant.to_name())
        })
    }

    #[must_use]
    pub fn get_dash_sound(&self) -> Sound {
        if self.get_entity().is_under_water() {
            Sound::EntityZombieNautilusDash
        } else {
            Sound::EntityZombieNautilusDashLand
        }
    }

    #[must_use]
    pub fn get_dash_ready_sound(&self) -> Sound {
        if self.get_entity().is_under_water() {
            Sound::EntityZombieNautilusDashReady
        } else {
            Sound::EntityZombieNautilusDashReadyLand
        }
    }

    fn is_mob_controlled(&self) -> bool {
        self.mob_entity
            .first_passenger()
            .is_some_and(|passenger| passenger.get_mob().is_some())
    }

    fn is_aggravated(&self) -> bool {
        self.mob_entity.get_target().is_some()
    }
}

impl AgeableMob for ZombieNautilusEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn can_be_a_baby(&self) -> bool {
        false
    }

    fn default_should_spawn_baby(&self) -> bool {
        false
    }
}

impl Animal for ZombieNautilusEntity {
    fn play_eating_sound(&self) {
        self.make_sound(Sound::EntityZombieNautilusEat);
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        if self.is_tame() {
            item_stack.item.has_tag(&tag::Item::MINECRAFT_NAUTILUS_FOOD)
        } else {
            item_stack
                .item
                .has_tag(&tag::Item::MINECRAFT_NAUTILUS_TAMING_ITEMS)
        }
    }

    fn can_mate(&self, _partner: &dyn EntityBase) -> bool {
        false
    }
}

impl TamableAnimal for ZombieNautilusEntity {
    fn get_tamable_data(&self) -> &TamableData {
        &self.tamable_data
    }
}

impl CustomSound for ZombieNautilusEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_under_water() {
            Sound::EntityZombieNautilusHurt
        } else {
            Sound::EntityZombieNautilusHurtLand
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_under_water() {
            Sound::EntityZombieNautilusDeath
        } else {
            Sound::EntityZombieNautilusDeathLand
        })
    }
}

impl Mob for ZombieNautilusEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_tamable(&self) -> Option<&dyn TamableAnimal> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn sun_protection_slot(&self) -> EquipmentSlot {
        EquipmentSlot::BODY
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        world.is_unobstructed(self.get_entity())
    }

    fn can_be_leashed(&self) -> bool {
        !self.is_aggravated() && !self.is_mob_controlled()
    }

    fn requires_custom_persistence(&self) -> bool {
        self.get_entity().has_vehicle() || self.is_tame()
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        true
    }

    fn is_tamed(&self) -> bool {
        self.is_tame()
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.get_owner()
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.saddled_controlling_passenger(None)
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::BODY
            || *slot == EquipmentSlot::SADDLE
            || self.mob_entity.can_pick_up_loot()
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        nautilus_can_use_slot(self, slot)
    }

    fn get_equip_sound(&self, slot: &EquipmentSlot) -> Option<Sound> {
        nautilus_equip_sound(self.get_entity(), slot)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_under_water() {
            Sound::EntityZombieNautilusAmbient
        } else {
            Sound::EntityZombieNautilusAmbientLand
        })
    }

    fn mob_get_swim_sound(&self) -> Option<Sound> {
        Some(Sound::EntityZombieNautilusSwim)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {}

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(tracked_data::zombie_nautilus::DASH, self.is_dashing());
        entity.set_synced_data(
            tracked_data::zombie_nautilus::DATA_VARIANT_ID,
            VarInt(i32::from(self.variant.load(Ordering::Relaxed))),
        );
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        self.set_variant(ZombieNautilusVariant::select_for_biome(biome.registry_id));
        self.finalize_spawn_super(group_data)
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(id) =
            self.resolve_dynamic_raw_id("minecraft:zombie_nautilus_variant", name, |n| {
                ZombieNautilusVariant::from_name(n).map(|v| u32::from(v.id()))
            })
            && let Ok(variant_id) = u8::try_from(id)
        {
            self.set_variant_id(variant_id);
        }
    }

    fn mob_apply_implicit_component(&self, component: DataComponent, value: &str) {
        if component == DataComponent::ZombieNautilusVariant {
            self.mob_set_variant_name(value);
        }
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        (component == DataComponent::ZombieNautilusVariant).then(|| {
            Box::new(ZombieNautilusVariantImpl {
                value: self.variant_name().into(),
            }) as Box<dyn DataComponentImpl>
        })
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_tamable_nbt(nbt);
        nbt.put_int("DashCooldown", self.dash_cooldown.load(Ordering::Relaxed));
        nbt.put_string("variant", self.variant_name());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddled") == Some(true));
        self.read_tamable_nbt(nbt);
        if let Some(dash) = nbt.get_int("DashCooldown") {
            self.dash_cooldown.store(dash, Ordering::Relaxed);
        }
        if let Some(variant_str) = nbt.get_string("variant") {
            self.mob_set_variant_name(variant_str);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        tick_abstract_nautilus(
            &self.mob_entity,
            &self.dash_cooldown,
            self.is_dashing(),
            |dashing| self.set_dashing(dashing),
            || self.get_dash_ready_sound(),
        );
    }

    fn mob_interact_with_hand(
        &self,
        _player: &Arc<Player>,
        _item_stack: &mut ItemStack,
        _hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        self.mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
        None
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        interact_abstract_nautilus(
            self,
            player,
            item_stack,
            self.is_tame(),
            |owner| self.tame(owner),
            || Sound::EntityZombieNautilusEat,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn zombie_nautilus_variant_and_adult_invariants() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let nautilus = ZombieNautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 60.0, 0.0),
            &EntityType::ZOMBIE_NAUTILUS,
        ));

        assert!(nautilus.sun_protection_slot() == EquipmentSlot::BODY);
        assert!(nautilus.burns_in_daylight());
        assert!(!nautilus.can_be_a_baby());

        let mut nbt = NbtCompound::new();
        nbt.put_int("Age", -24000);
        nbt.put_string("variant", "minecraft:warm".to_string());
        EntityBase::read_nbt_non_mut(nautilus.as_ref(), &nbt);

        assert!(!nautilus.is_baby());
        assert_eq!(nautilus.variant_name(), "minecraft:warm");

        let mut written = NbtCompound::new();
        EntityBase::write_nbt(nautilus.as_ref(), &mut written);
        assert_eq!(written.get_string("variant"), Some("minecraft:warm"));
        assert!(written.get("Age").is_none());

        world.level.shutdown().await;
        Ok(())
    }
}

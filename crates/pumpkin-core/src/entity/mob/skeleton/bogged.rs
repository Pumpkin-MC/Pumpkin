use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};
use pumpkin_data::data_component_impl::{PotionContentsImpl, StatusEffectInstance};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
    sheared: AtomicBool,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.set_synced_data(pumpkin_data::tracked_data::bogged::DATA_SHEARED, false);
        let entity = SkeletonEntityBase::new(entity);
        let bogged = Self {
            entity,
            sheared: AtomicBool::new(false),
        };
        Arc::new(bogged)
    }

    pub fn is_sheared(&self) -> bool {
        self.sheared.load(Ordering::Relaxed)
    }

    pub fn set_sheared(&self, sheared: bool) {
        let old = self.sheared.swap(sheared, Ordering::Relaxed);
        if old != sheared {
            self.entity
                .mob_entity
                .living_entity
                .entity
                .set_synced_data(pumpkin_data::tracked_data::bogged::DATA_SHEARED, sheared);
        }
    }
}

impl Mob for BoggedSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedAmbient)
    }

    fn get_min_attack_interval(&self) -> Option<i32> {
        let world = self.get_entity().world.load();
        if world.get_difficulty() == pumpkin_util::Difficulty::Hard {
            Some(50)
        } else {
            Some(70)
        }
    }

    fn get_arrow_projectile(&self) -> Option<ItemStack> {
        // Plain arrow avoids PotionDurationScale reducing the 100-tick poison duration.
        let mut arrow = ItemStack::new(1, &Item::ARROW);

        let poison_effect = StatusEffectInstance {
            effect_id: Cow::Borrowed("minecraft:poison"),
            amplifier: 0,
            duration: 100,
            ambient: false,
            show_particles: true,
            show_icon: true,
        };

        let potion_contents = PotionContentsImpl {
            potion_id: None,
            custom_color: None,
            custom_effects: vec![poison_effect],
            custom_name: None,
        };

        arrow.set_data_component(potion_contents);
        Some(arrow)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("sheared", self.is_sheared());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(sheared) = nbt.get_bool("sheared") {
            self.set_sheared(sheared);
        }
    }

    fn mob_interact(
        &self,
        player: &Arc<crate::entity::player::Player>,
        item_stack: &mut ItemStack,
    ) -> bool {
        let item = item_stack.get_item();

        if item == &Item::SHEARS && !self.is_sheared() {
            let entity = self.get_entity();
            let world = entity.world.load();
            if let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
                && let Some(server) = world.server.upgrade()
            {
                let mut event =
                    crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent {
                        player: player_arc,
                        entity_id: entity.entity_id,
                        hand: 0,
                        cancelled: false,
                    };
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return false;
                }
            }
            let pos = entity.pos.load();

            world.play_sound(Sound::EntityBoggedShear, SoundCategory::Players, &pos);

            for _ in 0..2 {
                let mushroom = if rand::random::<bool>() {
                    &Item::BROWN_MUSHROOM
                } else {
                    &Item::RED_MUSHROOM
                };

                let item_entity = Arc::new(crate::entity::item::ItemEntity::new(
                    Entity::new(world.clone(), pos, &EntityType::ITEM),
                    ItemStack::new(1, mushroom),
                ));

                world.spawn_entity(item_entity);
            }

            self.set_sheared(true);
            player.damage_held_item(1);

            true
        } else {
            self.entity.mob_entity.mob_interact(player, item_stack)
        }
    }
}

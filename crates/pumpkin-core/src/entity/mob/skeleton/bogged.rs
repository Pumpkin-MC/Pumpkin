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
        self.sheared.store(sheared, Ordering::Relaxed);
        // TODO: Update entity visual (remove mushrooms)
    }
}

impl Mob for BoggedSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn get_arrow_projectile(&self) -> Option<ItemStack> {
        // Vanilla Bogged.getArrow calls arrow.addEffect(new MobEffectInstance(POISON, 100))
        // directly on the AbstractArrow, bypassing any PotionDurationScale.
        // Using a plain ARROW (no PotionDurationScale component, so scale=1.0) means the
        // 100-tick duration is preserved unchanged when applied through PotionContents.
        let mut arrow = ItemStack::new(1, &Item::ARROW);

        let poison_effect = StatusEffectInstance {
            effect_id: Cow::Borrowed("minecraft:poison"),
            amplifier: 0,
            duration: 100, // 5 seconds at 20 ticks/s, matching vanilla MobEffectInstance(POISON, 100)
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

        // Handle shearing
        if item == &Item::SHEARS && !self.is_sheared() {
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();

            // Play shearing sound
            world.play_sound(Sound::EntityBoggedShear, SoundCategory::Players, &pos);

            // Drop 2 mushrooms (randomly brown or red)
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

            // Mark as sheared
            self.set_sheared(true);

            // Damage shears
            player.damage_held_item(1);

            true
        } else {
            self.entity.mob_entity.mob_interact(player, item_stack)
        }
    }
}

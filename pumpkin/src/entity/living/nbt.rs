use super::LivingEntity;
use crate::entity::{NBTStorage, NBTStorageInit, NbtFuture};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::potion::Effect;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use std::sync::atomic::Ordering::Relaxed;
use tracing::warn;

impl NBTStorage for LivingEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;
            nbt.put("Health", NbtTag::Float(self.health.load()));
            // Avoid persisting a lethal fall distance when the entity is dead to prevent death loops
            let fall_distance = if self.dead.load(Relaxed) {
                0.0
            } else {
                self.fall_distance.load()
            };
            // Persist current absorption amount
            nbt.put("AbsorptionAmount", NbtTag::Float(self.absorption.load()));
            nbt.put("fall_distance", NbtTag::Float(fall_distance));
            {
                let effects = self.active_effects.lock().await;
                if !effects.is_empty() {
                    // Iterate effects and create Box<[NbtTag]>
                    let mut effects_list = Vec::with_capacity(effects.len());
                    for effect in effects.values() {
                        let mut effect_nbt = pumpkin_nbt::compound::NbtCompound::new();
                        effect.write_nbt(&mut effect_nbt).await;
                        effects_list.push(NbtTag::Compound(effect_nbt));
                    }
                    nbt.put("active_effects", NbtTag::List(effects_list));
                }
            }
            //TODO: write equipment
            // todo more...
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.read_nbt_non_mut(nbt).await;
            self.health.store(nbt.get_float("Health").unwrap_or(0.0));

            // Clamp any persisted absorption to the entity's configured max
            let raw_abs = nbt.get_float("AbsorptionAmount").unwrap_or(0.0);
            let max_abs = self.get_attribute_value(&Attributes::MAX_ABSORPTION) as f32;
            let clamped_abs = raw_abs.max(0.0).min(max_abs);
            self.absorption.store(clamped_abs);

            // Load fall distance, but if this entity is currently marked dead ensure we don't restore
            // a lethal fall distance that would immediately re-kill on spawn.
            let fd = nbt.get_float("fall_distance").unwrap_or(0.0);
            if self.dead.load(Relaxed) {
                self.fall_distance.store(0.0);
            } else {
                self.fall_distance.store(fd);
            }
            {
                let mut active_effects = self.active_effects.lock().await;
                let nbt_effects = nbt.get_list("active_effects");
                if let Some(nbt_effects) = nbt_effects {
                    for effect in nbt_effects {
                        if let NbtTag::Compound(effect_nbt) = effect {
                            let effect = Effect::create_from_nbt(&mut effect_nbt.clone()).await;
                            if effect.is_none() {
                                warn!("Unable to read effect from nbt");
                                continue;
                            }
                            let mut effect = effect.unwrap();
                            effect.blend = true; // TODO: change, is taken from effect give command
                            active_effects.insert(effect.effect_type, effect);
                        }
                    }
                }
            }
        })
        // todo more...
    }
}

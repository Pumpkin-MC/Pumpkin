use crate::codec::var_int::VarInt;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    DamageImpl, DataComponentImpl, EnchantmentsImpl, MaxStackSizeImpl, PotionContentsImpl,
    StatusEffectInstance, UnbreakableImpl, get,
};
use serde::de;
use serde::de::SeqAccess;
use serde::ser::SerializeStruct;
use std::borrow::Cow;

trait DataComponentCodec<Impl: DataComponentImpl> {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error>;
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Impl, A::Error>;
}

impl DataComponentCodec<Self> for MaxStackSizeImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.size))
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let size = u8::try_from(
            seq.next_element::<VarInt>()?
                .ok_or(de::Error::custom("No MaxStackSize VarInt!"))?
                .0,
        )
        .map_err(|_| de::Error::custom("No MaxStackSize VarInt!"))?;
        Ok(Self { size })
    }
}

impl DataComponentCodec<Self> for DamageImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.damage))
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let damage = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No damage VarInt!"))?
            .0;
        Ok(Self { damage })
    }
}

impl DataComponentCodec<Self> for EnchantmentsImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.enchantment.len() as i32))?;
        for (enc, level) in self.enchantment.iter() {
            seq.serialize_field::<VarInt>("", &VarInt::from(enc.id))?;
            seq.serialize_field::<VarInt>("", &VarInt::from(*level))?;
        }
        Ok(())
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No EnchantmentsImpl len VarInt!"))?
            .0 as usize;
        let mut enc = Vec::with_capacity(len);
        for _ in 0..len {
            let id = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No EnchantmentsImpl id VarInt!"))?
                .0 as u8;
            let level = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No EnchantmentsImpl level VarInt!"))?
                .0;
            enc.push((
                Enchantment::from_id(id).ok_or(de::Error::custom(
                    "EnchantmentsImpl Enchantment VarInt Incorrect!",
                ))?,
                level,
            ));
        }
        Ok(Self {
            enchantment: Cow::from(enc),
        })
    }
}

impl DataComponentCodec<Self> for UnbreakableImpl {
    fn serialize<T: SerializeStruct>(&self, _seq: &mut T) -> Result<(), T::Error> {
        Ok(())
    }
    fn deserialize<'a, A: SeqAccess<'a>>(_seq: &mut A) -> Result<Self, A::Error> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for PotionContentsImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        // Potion ID (optional)
        if let Some(potion_id) = self.potion_id {
            seq.serialize_field::<bool>("", &true)?;
            seq.serialize_field::<VarInt>("", &VarInt::from(potion_id))?;
        } else {
            seq.serialize_field::<bool>("", &false)?;
        }

        // Custom color (optional)
        if let Some(color) = self.custom_color {
            seq.serialize_field::<bool>("", &true)?;
            seq.serialize_field::<i32>("", &color)?;
        } else {
            seq.serialize_field::<bool>("", &false)?;
        }

        // Custom effects list
        seq.serialize_field::<VarInt>("", &VarInt::from(self.custom_effects.len() as i32))?;
        for effect in &self.custom_effects {
            seq.serialize_field::<VarInt>("", &VarInt::from(effect.effect_id))?;
            // Effect parameters
            seq.serialize_field::<VarInt>("", &VarInt::from(effect.amplifier))?;
            seq.serialize_field::<VarInt>("", &VarInt::from(effect.duration))?;
            seq.serialize_field::<bool>("", &effect.ambient)?;
            seq.serialize_field::<bool>("", &effect.show_particles)?;
            seq.serialize_field::<bool>("", &effect.show_icon)?;
            // No hidden effect
            seq.serialize_field::<bool>("", &false)?;
        }

        // Custom name (optional)
        if let Some(name) = &self.custom_name {
            seq.serialize_field::<bool>("", &true)?;
            seq.serialize_field::<&str>("", &name.as_str())?;
        } else {
            seq.serialize_field::<bool>("", &false)?;
        }

        Ok(())
    }

    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        // Potion ID (optional)
        let has_potion = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No PotionContents has_potion bool!"))?;
        let potion_id = if has_potion {
            Some(
                seq.next_element::<VarInt>()?
                    .ok_or(de::Error::custom("No PotionContents potion_id VarInt!"))?
                    .0,
            )
        } else {
            None
        };

        // Custom color (optional)
        let has_color = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No PotionContents has_color bool!"))?;
        let custom_color = if has_color {
            Some(
                seq.next_element::<i32>()?
                    .ok_or(de::Error::custom("No PotionContents custom_color i32!"))?,
            )
        } else {
            None
        };

        // Custom effects list
        let effects_len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No PotionContents effects_len VarInt!"))?
            .0 as usize;

        let mut custom_effects = Vec::with_capacity(effects_len);
        for _ in 0..effects_len {
            let effect_id = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No effect_id VarInt!"))?
                .0;

            // Effect parameters
            let amplifier = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No amplifier VarInt!"))?
                .0;
            let duration = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No duration VarInt!"))?
                .0;
            let ambient = seq
                .next_element::<bool>()?
                .ok_or(de::Error::custom("No ambient bool!"))?;
            let show_particles = seq
                .next_element::<bool>()?
                .ok_or(de::Error::custom("No show_particles bool!"))?;
            let show_icon = seq
                .next_element::<bool>()?
                .ok_or(de::Error::custom("No show_icon bool!"))?;

            // Hidden effect (optional, recursive) - we skip it for now
            let has_hidden = seq
                .next_element::<bool>()?
                .ok_or(de::Error::custom("No has_hidden bool!"))?;
            if has_hidden {
                // Skip hidden effect parameters recursively
                skip_effect_parameters(seq)?;
            }

            custom_effects.push(StatusEffectInstance {
                effect_id,
                amplifier,
                duration,
                ambient,
                show_particles,
                show_icon,
            });
        }

        // Custom name (optional)
        let has_name = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No PotionContents has_name bool!"))?;
        let custom_name = if has_name {
            Some(
                seq.next_element::<String>()?
                    .ok_or(de::Error::custom("No PotionContents custom_name String!"))?,
            )
        } else {
            None
        };

        Ok(Self {
            potion_id,
            custom_color,
            custom_effects,
            custom_name,
        })
    }
}

/// Helper to skip hidden effect parameters recursively
fn skip_effect_parameters<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<(), A::Error> {
    // amplifier
    seq.next_element::<VarInt>()?
        .ok_or(de::Error::custom("No hidden amplifier VarInt!"))?;
    // duration
    seq.next_element::<VarInt>()?
        .ok_or(de::Error::custom("No hidden duration VarInt!"))?;
    // ambient
    seq.next_element::<bool>()?
        .ok_or(de::Error::custom("No hidden ambient bool!"))?;
    // show_particles
    seq.next_element::<bool>()?
        .ok_or(de::Error::custom("No hidden show_particles bool!"))?;
    // show_icon
    seq.next_element::<bool>()?
        .ok_or(de::Error::custom("No hidden show_icon bool!"))?;
    // has_hidden (recursive)
    let has_hidden = seq
        .next_element::<bool>()?
        .ok_or(de::Error::custom("No hidden has_hidden bool!"))?;
    if has_hidden {
        skip_effect_parameters(seq)?;
    }
    Ok(())
}

pub fn deserialize<'a, A: SeqAccess<'a>>(
    id: DataComponent,
    seq: &mut A,
) -> Result<Box<dyn DataComponentImpl>, A::Error> {
    match id {
        DataComponent::MaxStackSize => Ok(MaxStackSizeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Enchantments => Ok(EnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Damage => Ok(DamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Unbreakable => Ok(UnbreakableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotionContents => Ok(PotionContentsImpl::deserialize(seq)?.to_dyn()),
        _ => todo!("{} not yet implemented", id.to_name()),
    }
}
pub fn serialize<T: SerializeStruct>(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    seq: &mut T,
) -> Result<(), T::Error> {
    match id {
        DataComponent::MaxStackSize => get::<MaxStackSizeImpl>(value).serialize(seq),
        DataComponent::Enchantments => get::<EnchantmentsImpl>(value).serialize(seq),
        DataComponent::Damage => get::<DamageImpl>(value).serialize(seq),
        DataComponent::Unbreakable => get::<UnbreakableImpl>(value).serialize(seq),
        DataComponent::PotionContents => get::<PotionContentsImpl>(value).serialize(seq),
        _ => todo!("{} not yet implemented", id.to_name()),
    }
}

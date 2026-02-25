use crate::codec::var_int::VarInt;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    CustomNameImpl, DamageImpl, DataComponentImpl, EnchantmentsImpl, FireworkExplosionImpl,
    FireworkExplosionShape, FoodImpl, FireworksImpl, ItemModelImpl, MaxDamageImpl,
    MaxStackSizeImpl, PotionContentsImpl, StatusEffectInstance, UnbreakableImpl,
};
use pumpkin_util::text::TextComponent;
use pumpkin_nbt::tag::NbtTag;
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

impl DataComponentCodec<Self> for MaxDamageImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.max_damage))
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let max_damage = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No max_damage VarInt!"))?
            .0;
        Ok(Self { max_damage })
    }
}

impl DataComponentCodec<Self> for FoodImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.nutrition))?;
        seq.serialize_field::<f32>("", &self.saturation)?;
        seq.serialize_field::<bool>("", &self.can_always_eat)
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let nutrition = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No Food nutrition VarInt!"))?
            .0;
        let saturation = seq
            .next_element::<f32>()?
            .ok_or(de::Error::custom("No Food saturation f32!"))?;
        let can_always_eat = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No Food can_always_eat bool!"))?;
        Ok(Self {
            nutrition,
            saturation,
            can_always_eat,
        })
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
        const MAX_ENCHANTMENTS: usize = 256;

        let len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No EnchantmentsImpl len VarInt!"))?
            .0 as usize;
        if len > MAX_ENCHANTMENTS {
            return Err(de::Error::custom("Too many enchantments"));
        }
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

impl DataComponentCodec<Self> for ItemModelImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<String>("", &self.model)
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let model = seq
            .next_element::<String>()?
            .ok_or(de::Error::custom("No ItemModel string!"))?;
        Ok(Self { model })
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

impl DataComponentCodec<Self> for CustomNameImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<TextComponent>("", &TextComponent::text(self.name.clone()))
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let text = seq
            .next_element::<String>()?
            .ok_or(de::Error::custom("No CustomName string!"))?;
        Ok(Self { name: text })
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
        const MAX_EFFECTS: usize = 128;

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
        if effects_len > MAX_EFFECTS {
            return Err(de::Error::custom("Too many potion effects"));
        }
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

impl DataComponentCodec<Self> for FireworkExplosionImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        // Shape (VarInt enum)
        seq.serialize_field::<VarInt>("", &VarInt::from(self.shape.to_id()))?;
        // Colors list
        seq.serialize_field::<VarInt>("", &VarInt::from(self.colors.len() as i32))?;
        for color in &self.colors {
            seq.serialize_field::<i32>("", color)?;
        }
        // Fade colors list
        seq.serialize_field::<VarInt>("", &VarInt::from(self.fade_colors.len() as i32))?;
        for color in &self.fade_colors {
            seq.serialize_field::<i32>("", color)?;
        }
        // hasTrail
        seq.serialize_field::<bool>("", &self.has_trail)?;
        // hasTwinkle
        seq.serialize_field::<bool>("", &self.has_twinkle)?;
        Ok(())
    }

    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        // Needs a length cap during deserialization to prevent OOM from malicious packets
        // Vanilla doesn't have any limits (Integer.MAX_VALUE is technically a limit but not enforced in practice)
        const MAX_COLORS: usize = 256;
        const MAX_FADE_COLORS: usize = 256;

        // Shape (VarInt enum)
        let shape_id = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom(
                "No FireworkExplosionImpl shape_id VarInt!",
            ))?
            .0;
        let shape = FireworkExplosionShape::from_id(shape_id)
            .ok_or(de::Error::custom("Invalid FireworkExplosionShape id!"))?;

        // Colors list
        let colors_len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom(
                "No FireworkExplosionImpl colors_len VarInt!",
            ))?
            .0 as usize;
        if colors_len > MAX_COLORS {
            return Err(de::Error::custom(format!(
                "FireworkExplosionImpl colors_len {colors_len} exceeds maximum of {MAX_COLORS}"
            )));
        }
        let mut colors = Vec::with_capacity(colors_len);
        for _ in 0..colors_len {
            let color = seq
                .next_element::<i32>()?
                .ok_or(de::Error::custom("No FireworkExplosionImpl color i32!"))?;
            colors.push(color);
        }

        // Fade colors list
        let fade_colors_len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom(
                "No FireworkExplosionImpl fade_colors_len VarInt!",
            ))?
            .0 as usize;
        if fade_colors_len > MAX_FADE_COLORS {
            return Err(de::Error::custom(format!(
                "FireworkExplosionImpl fade_colors_len {fade_colors_len} exceeds maximum of {MAX_FADE_COLORS}"
            )));
        }
        let mut fade_colors = Vec::with_capacity(fade_colors_len);
        for _ in 0..fade_colors_len {
            let color = seq.next_element::<i32>()?.ok_or(de::Error::custom(
                "No FireworkExplosionImpl fade_color i32!",
            ))?;
            fade_colors.push(color);
        }

        // hasTrail
        let has_trail = seq.next_element::<bool>()?.ok_or(de::Error::custom(
            "No FireworkExplosionImpl has_trail bool!",
        ))?;

        // hasTwinkle
        let has_twinkle = seq.next_element::<bool>()?.ok_or(de::Error::custom(
            "No FireworkExplosionImpl has_twinkle bool!",
        ))?;

        Ok(Self::new(
            shape,
            colors,
            fade_colors,
            has_trail,
            has_twinkle,
        ))
    }
}

impl DataComponentCodec<Self> for FireworksImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        // Flight duration (VarInt)
        seq.serialize_field::<VarInt>("", &VarInt::from(self.flight_duration))?;
        // Explosions list
        seq.serialize_field::<VarInt>("", &VarInt::from(self.explosions.len() as i32))?;
        for explosion in &self.explosions {
            explosion.serialize(seq)?;
        }
        Ok(())
    }

    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        // Needs a length cap during deserialization to prevent OOM from malicious packets
        // Vanilla doesn't have any limits
        const MAX_EXPLOSIONS: usize = 256;
        // Vanilla restricts to 0-255 (UNSIGNED_BYTE in data component codec) (do not trust client NBT to limit it)
        const MAX_FLIGHT_DURATION: i32 = 255;

        // Flight duration
        let flight_duration = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom(
                "No FireworksImpl flight_duration VarInt!",
            ))?
            .0;
        if !(0..=MAX_FLIGHT_DURATION).contains(&flight_duration) {
            return Err(de::Error::custom(format!(
                "FireworksImpl flight_duration {flight_duration} is out of bounds (0-{MAX_FLIGHT_DURATION})"
            )));
        }

        // Explosions list
        let explosions_len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No FireworksImpl explosions_len VarInt!"))?
            .0 as usize;
        if explosions_len > MAX_EXPLOSIONS {
            return Err(de::Error::custom(format!(
                "FireworksImpl explosions_len {explosions_len} exceeds maximum of {MAX_EXPLOSIONS}"
            )));
        }
        let mut explosions = Vec::with_capacity(explosions_len);
        for _ in 0..explosions_len {
            // Recursively deserialize each explosion
            let explosion = FireworkExplosionImpl::deserialize(seq)?;
            explosions.push(explosion);
        }

        Ok(Self::new(flight_duration, explosions))
    }
}

pub fn deserialize<'a, A: SeqAccess<'a>>(
    id: DataComponent,
    seq: &mut A,
) -> Result<Box<dyn DataComponentImpl>, A::Error> {
    match id {
        DataComponent::MaxStackSize => Ok(MaxStackSizeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ItemModel => Ok(ItemModelImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CustomName => Ok(CustomNameImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Enchantments => Ok(EnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Damage => Ok(DamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MaxDamage => Ok(MaxDamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Food => Ok(FoodImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Unbreakable => Ok(UnbreakableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotionContents => Ok(PotionContentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::FireworkExplosion => Ok(FireworkExplosionImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Fireworks => Ok(FireworksImpl::deserialize(seq)?.to_dyn()),
        _ => Err(serde::de::Error::custom(format!(
            "data component {} not yet implemented",
            id.to_name()
        ))),
    }
}
pub fn serialize<T: SerializeStruct>(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    seq: &mut T,
) -> Result<(), T::Error> {
    match id {
        DataComponent::MaxStackSize => {
            if let Some(v) = value.as_any().downcast_ref::<MaxStackSizeImpl>() {
                v.serialize(seq)
            } else if let NbtTag::Int(size) = value.write_data() {
                seq.serialize_field::<VarInt>("", &VarInt::from(size))
            } else {
                Err(serde::ser::Error::custom("MaxStackSize: cdylib downcast failed"))
            }
        }
        DataComponent::CustomName => {
            if let Some(v) = value.as_any().downcast_ref::<CustomNameImpl>() {
                v.serialize(seq)
            } else if let NbtTag::String(name) = value.write_data() {
                let text = TextComponent::text(name);
                seq.serialize_field::<TextComponent>("", &text)
            } else {
                Err(serde::ser::Error::custom("CustomName: cdylib downcast failed"))
            }
        }
        DataComponent::ItemModel => {
            if let Some(v) = value.as_any().downcast_ref::<ItemModelImpl>() {
                v.serialize(seq)
            } else if let NbtTag::String(model) = value.write_data() {
                seq.serialize_field::<String>("", &model)
            } else {
                Err(serde::ser::Error::custom("ItemModel: cdylib downcast failed"))
            }
        }
        DataComponent::Enchantments => {
            if let Some(v) = value.as_any().downcast_ref::<EnchantmentsImpl>() {
                v.serialize(seq)
            } else {
                Err(serde::ser::Error::custom("Enchantments: cdylib downcast failed"))
            }
        }
        DataComponent::Damage => {
            if let Some(v) = value.as_any().downcast_ref::<DamageImpl>() {
                v.serialize(seq)
            } else if let NbtTag::Int(damage) = value.write_data() {
                seq.serialize_field::<VarInt>("", &VarInt::from(damage))
            } else {
                Err(serde::ser::Error::custom("Damage: cdylib downcast failed"))
            }
        }
        DataComponent::MaxDamage => {
            if let Some(v) = value.as_any().downcast_ref::<MaxDamageImpl>() {
                v.serialize(seq)
            } else if let NbtTag::Int(max_damage) = value.write_data() {
                seq.serialize_field::<VarInt>("", &VarInt::from(max_damage))
            } else {
                Err(serde::ser::Error::custom("MaxDamage: cdylib downcast failed"))
            }
        }
        DataComponent::Food => {
            if let Some(v) = value.as_any().downcast_ref::<FoodImpl>() {
                v.serialize(seq)
            } else {
                Err(serde::ser::Error::custom("Food: cdylib downcast failed"))
            }
        }
        DataComponent::Unbreakable => {
            if let Some(v) = value.as_any().downcast_ref::<UnbreakableImpl>() {
                v.serialize(seq)
            } else {
                // Unbreakable has no protocol data, just presence
                Ok(())
            }
        }
        DataComponent::PotionContents => {
            if let Some(v) = value.as_any().downcast_ref::<PotionContentsImpl>() {
                v.serialize(seq)
            } else {
                Err(serde::ser::Error::custom("PotionContents: cdylib downcast failed"))
            }
        }
        DataComponent::FireworkExplosion => {
            if let Some(v) = value.as_any().downcast_ref::<FireworkExplosionImpl>() {
                v.serialize(seq)
            } else {
                Err(serde::ser::Error::custom("FireworkExplosion: cdylib downcast failed"))
            }
        }
        DataComponent::Fireworks => {
            if let Some(v) = value.as_any().downcast_ref::<FireworksImpl>() {
                v.serialize(seq)
            } else {
                Err(serde::ser::Error::custom("Fireworks: cdylib downcast failed"))
            }
        }
        _ => Err(serde::ser::Error::custom(format!(
            "data component {} not yet implemented",
            id.to_name()
        ))),
    }
}

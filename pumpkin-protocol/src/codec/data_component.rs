use std::borrow::Cow;

use crate::codec::var_int::VarInt;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    ConsumableImpl, ConsumeAnimation, ConsumeEffect, DamageImpl, DataComponentImpl,
    EnchantmentsImpl, EquipmentSlot, EquippableImpl, FireworkExplosionImpl, FireworkExplosionShape,
    FireworksImpl, IDSet, IDSetContent, IdOr, ItemModelImpl, MaxStackSizeImpl, PotionContentsImpl,
    SoundEvent, StatusEffectInstance, UnbreakableImpl, get,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use serde::de;
use serde::de::SeqAccess;
use serde::ser::SerializeStruct;

const MAX_STATUS_EFFECTS: usize = 128;

#[must_use]
pub fn data_to_proto_sound(id_or: &IdOr<SoundEvent>) -> crate::IdOr<crate::SoundEvent> {
    match id_or {
        IdOr::Id(id) => crate::IdOr::Id(*id as u16),
        IdOr::Value(sound) => crate::IdOr::Value(crate::SoundEvent {
            sound_name: sound.sound_name.clone(),
            range: sound.range,
        }),
    }
}

#[must_use]
pub fn proto_to_data_sound(id_or: &crate::IdOr<crate::SoundEvent>) -> IdOr<SoundEvent> {
    match id_or {
        crate::IdOr::Id(id) => {
            IdOr::Id(Sound::from_name(Sound::NAMES[*id as usize]).expect("Invalid sound id"))
        }
        crate::IdOr::Value(sound) => IdOr::Value(SoundEvent {
            sound_name: sound.sound_name.clone(),
            range: sound.range,
        }),
    }
}

fn deserialize_idset<'a, A: SeqAccess<'a>, T: IDSetContent>(
    seq: &mut A,
) -> Result<IDSet<T>, A::Error> {
    let id_type = seq
        .next_element::<VarInt>()?
        .ok_or(de::Error::custom("No type/len VarInt in IDSet"))?
        .0;

    match id_type.cmp(&0) {
        std::cmp::Ordering::Equal => {
            let tag = seq
                .next_element::<String>()?
                .ok_or(de::Error::custom("No tag name in IDSet"))?;
            Ok(IDSet::Tag(Cow::Owned(tag)))
        }
        std::cmp::Ordering::Greater => {
            let len = id_type - 1;
            let mut content_vec = Vec::with_capacity(len as usize);

            for _ in 0..len {
                let varint_id = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Missing registry id VarInt in IDSet"))?
                    .0;

                let elmt = T::from_id(varint_id as u16)
                    .ok_or(de::Error::custom("Invalid registry id VarInt in IDSet"))?;
                content_vec.push(elmt);
            }
            Ok(IDSet::IDs(Cow::Owned(content_vec)))
        }
        std::cmp::Ordering::Less => {
            Result::Err(de::Error::custom("Negative type/len VarInt in IDSet"))
        }
    }
}

fn serialize_idset<T: SerializeStruct, C: IDSetContent>(
    idset: &IDSet<C>,
    seq: &mut T,
) -> Result<(), T::Error> {
    match idset {
        IDSet::Tag(tag) => {
            seq.serialize_field::<VarInt>("", &VarInt(0))?;
            seq.serialize_field::<str>("", tag)
        }
        IDSet::IDs(elements) => {
            seq.serialize_field::<VarInt>("", &VarInt(elements.len() as i32 + 1))?;
            for elmt in elements.iter() {
                seq.serialize_field::<VarInt>("", &VarInt(elmt.registry_id() as i32))?;
            }
            Ok(())
        }
    }
}

fn deserialize_status_effects<'a, A: SeqAccess<'a>>(
    seq: &mut A,
) -> Result<Vec<StatusEffectInstance>, A::Error> {
    let effects_len = seq
        .next_element::<VarInt>()?
        .ok_or(de::Error::custom("No effects_len VarInt!"))?
        .0 as usize;
    if effects_len > MAX_STATUS_EFFECTS {
        return Err(de::Error::custom("Too many status effects"));
    }
    let mut custom_effects = Vec::with_capacity(effects_len);
    for _ in 0..effects_len {
        let effect_registry_id = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No effect_id VarInt!"))?
            .0;
        let effect_name = StatusEffect::from_id(effect_registry_id as u16)
            .ok_or(de::Error::custom("Invalid effect_id!"))?
            .minecraft_name;
        let effect_id = Cow::Borrowed(effect_name);

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

    Ok(custom_effects)
}

fn serialize_status_effects<T: SerializeStruct>(
    effects: &Vec<StatusEffectInstance>,
    seq: &mut T,
) -> Result<(), T::Error> {
    seq.serialize_field::<VarInt>("", &VarInt(effects.len() as i32))?;

    for effect in effects {
        seq.serialize_field::<VarInt>(
            "",
            &VarInt(
                StatusEffect::from_minecraft_name(&effect.effect_id)
                    .unwrap()
                    .registry_id() as i32,
            ),
        )?;
        // Effect parameters
        seq.serialize_field::<VarInt>("", &VarInt::from(effect.amplifier))?;
        seq.serialize_field::<VarInt>("", &VarInt::from(effect.duration))?;
        seq.serialize_field::<bool>("", &effect.ambient)?;
        seq.serialize_field::<bool>("", &effect.show_particles)?;
        seq.serialize_field::<bool>("", &effect.show_icon)?;
        // No hidden effect for now
        seq.serialize_field::<bool>("", &false)?;
    }
    Ok(())
}

fn deserialize_consume_effect<'a, A: SeqAccess<'a>>(
    seq: &mut A,
) -> Result<ConsumeEffect, A::Error> {
    let effect_type = seq
        .next_element::<VarInt>()?
        .ok_or(de::Error::custom("No type VarInt in ConsumeEffect"))?
        .0;
    match effect_type {
        0 => {
            let probability = seq
                .next_element::<f32>()?
                .ok_or(de::Error::custom("No probability float in ConsumeEffect"))?;
            Ok(ConsumeEffect::ApplyEffects((
                Cow::Owned(deserialize_status_effects(seq)?),
                probability,
            )))
        }
        1 => {
            let idset = deserialize_idset(seq)?;
            Ok(ConsumeEffect::RemoveEffects(idset))
        }
        2 => Ok(ConsumeEffect::ClearAllEffects),
        3 => {
            let diameter = seq
                .next_element::<f32>()?
                .ok_or(de::Error::custom("No diameter float in ConsumeEffect"))?;
            Ok(ConsumeEffect::TeleportRandomly(diameter))
        }
        4 => {
            let proto_sound_event = seq
                .next_element::<crate::IdOr<crate::SoundEvent>>()?
                .ok_or(de::Error::custom(
                    "No sound IdOr<SoundEvent> in ConsumeEffect",
                ))?;
            Ok(ConsumeEffect::PlaySound(proto_to_data_sound(
                &proto_sound_event,
            )))
        }
        _ => Err(de::Error::custom("Invalid effect_type in ConsumeEffect")),
    }
}

fn serialize_consume_effect<T: SerializeStruct>(
    consume_effect: &ConsumeEffect,
    seq: &mut T,
) -> Result<(), T::Error> {
    seq.serialize_field::<VarInt>("", &VarInt(consume_effect.registry_id() as i32))?;
    match consume_effect {
        ConsumeEffect::ApplyEffects((effects, probability)) => {
            serialize_status_effects(&effects.to_vec(), seq)?;
            seq.serialize_field::<f32>("", probability)?;
        }
        ConsumeEffect::RemoveEffects(idset) => serialize_idset(idset, seq)?,
        ConsumeEffect::ClearAllEffects => (),
        ConsumeEffect::TeleportRandomly(diameter) => seq.serialize_field::<f32>("", diameter)?,
        ConsumeEffect::PlaySound(id_or) => {
            seq.serialize_field::<crate::IdOr<crate::SoundEvent>>("", &data_to_proto_sound(id_or))?;
        }
    }
    Ok(())
}

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

impl DataComponentCodec<Self> for UnbreakableImpl {
    fn serialize<T: SerializeStruct>(&self, _seq: &mut T) -> Result<(), T::Error> {
        Ok(())
    }
    fn deserialize<'a, A: SeqAccess<'a>>(_seq: &mut A) -> Result<Self, A::Error> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for ItemModelImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<String>("", &self.id)
    }

    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let id = seq
            .next_element::<String>()?
            .ok_or(de::Error::custom("No ItemModelImpl id string!"))?;
        Ok(Self { id })
    }
}

impl DataComponentCodec<Self> for ConsumableImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<f32>("", &self.consume_seconds)?;
        seq.serialize_field::<VarInt>("", &VarInt(self.animation.clone() as i32))?;
        seq.serialize_field::<crate::IdOr<crate::SoundEvent>>(
            "",
            &data_to_proto_sound(&self.sound_event),
        )?;
        seq.serialize_field::<bool>("", &self.consume_particles)?;
        seq.serialize_field::<VarInt>("", &VarInt(self.effects.len() as i32))?;

        for effect in self.effects.iter() {
            serialize_consume_effect(effect, seq)?;
        }

        Ok(())
    }

    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let consume_seconds = seq.next_element::<f32>()?.ok_or(de::Error::custom(
            "No ConsumableImpl consume_seconds float!",
        ))?;
        let animation_id = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No ConsumableImpl animation VarInt!"))?;

        let animation: ConsumeAnimation = animation_id
            .0
            .try_into()
            .map_err(|()| de::Error::custom("Invalid ConsumableImpl animation id!"))?;
        let proto_sound_event = seq
            .next_element::<crate::IdOr<crate::SoundEvent>>()?
            .ok_or(de::Error::custom(
                "No ConsumableImpl sound_event IdOr<SoundEvent>!",
            ))?;
        let consume_particles = seq.next_element::<bool>()?.ok_or(de::Error::custom(
            "No ConsumableImpl consume_particles bool!",
        ))?;

        let sound_event = proto_to_data_sound(&proto_sound_event);
        let effects_len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No array_len VarInt in ConsumableImpl"))?
            .0;
        let mut effects_vec = Vec::with_capacity(effects_len as usize);

        for _ in 0..effects_len {
            effects_vec.push(deserialize_consume_effect(seq)?);
        }

        let effects: Cow<'static, [ConsumeEffect]> = Cow::Owned(effects_vec);

        Ok(Self {
            consume_seconds,
            animation,
            sound_event,
            consume_particles,
            effects,
        })
    }
}

impl DataComponentCodec<Self> for EquippableImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt(self.slot.get_slot_index()))?;
        seq.serialize_field::<crate::IdOr<crate::SoundEvent>>(
            "",
            &data_to_proto_sound(&self.equip_sound),
        )?;
        seq.serialize_field::<Option<Cow<'static, str>>>("", &self.asset_id)?;
        seq.serialize_field::<Option<Cow<'static, str>>>("", &self.camera_overlay)?;

        seq.serialize_field::<bool>("", &self.allowed_entities.is_some())?;
        if let Some(allowed) = &self.allowed_entities {
            serialize_idset(allowed, seq)?;
        }

        seq.serialize_field::<bool>("", &self.dispensable)?;
        seq.serialize_field::<bool>("", &self.swappable)?;
        seq.serialize_field::<bool>("", &self.damage_on_hurt)?;
        seq.serialize_field::<bool>("", &self.equip_on_interact)?;
        seq.serialize_field::<bool>("", &self.can_be_sheared)?;
        seq.serialize_field::<crate::IdOr<crate::SoundEvent>>(
            "",
            &data_to_proto_sound(&self.shearing_sound),
        )
    }

    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let slot_index = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No EquippableImpl slot VarInt!"))?
            .0;
        let slot = EquipmentSlot::from_slot_index(slot_index).ok_or(de::Error::custom(format!(
            "Invalid equipment slot index {slot_index}"
        )))?;
        let equip_sound = proto_to_data_sound(
            &seq.next_element::<crate::IdOr<crate::SoundEvent>>()?
                .ok_or(de::Error::custom(
                    "No EquippableImpl equip_sound IdOr<SoundEvent>!",
                ))?,
        );
        let asset_id =
            seq.next_element::<Option<Cow<'static, str>>>()?
                .ok_or(de::Error::custom(
                    "No EquippableImpl asset_id optional string!",
                ))?;
        let camera_overlay =
            seq.next_element::<Option<Cow<'static, str>>>()?
                .ok_or(de::Error::custom(
                    "No EquippableImpl camera_overlay optional string!",
                ))?;
        let has_allowed_entities = seq.next_element::<bool>()?.ok_or(de::Error::custom(
            "No EquippableImpl allowed_entities presence bool!",
        ))?;

        let allowed_entities: Option<IDSet<EntityType>> = if has_allowed_entities {
            Some(deserialize_idset(seq)?)
        } else {
            None
        };

        let dispensable = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No EquippableImpl dispensable bool!"))?;
        let swappable = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No EquippableImpl swappable bool!"))?;
        let damage_on_hurt = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No EquippableImpl damage_on_hurt bool!"))?;
        let equip_on_interact = seq.next_element::<bool>()?.ok_or(de::Error::custom(
            "No EquippableImpl equip_on_interact bool!",
        ))?;
        let can_be_sheared = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No EquippableImpl can_be_shared bool!"))?;
        let shearing_sound = proto_to_data_sound(
            &seq.next_element::<crate::IdOr<crate::SoundEvent>>()?
                .ok_or(de::Error::custom(
                    "No EquippableImpl shearing_sound IdOr<SoundEvent>!",
                ))?,
        );

        Ok(Self {
            slot,
            equip_sound,
            asset_id,
            camera_overlay,
            allowed_entities,
            dispensable,
            swappable,
            damage_on_hurt,
            equip_on_interact,
            can_be_sheared,
            shearing_sound,
        })
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
        serialize_status_effects(&self.custom_effects, seq)?;

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
        let potion_id = has_potion
            .then(|| {
                seq.next_element::<VarInt>()?
                    .ok_or(de::Error::custom("No PotionContents potion_id VarInt!"))
                    .map(|value| value.0)
            })
            .transpose()?;

        // Custom color (optional)
        let has_color = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No PotionContents has_color bool!"))?;
        let custom_color = has_color
            .then(|| {
                seq.next_element::<i32>()?
                    .ok_or(de::Error::custom("No PotionContents custom_color i32!"))
            })
            .transpose()?;

        // Custom effects list
        let custom_effects = deserialize_status_effects(seq)?;

        // Custom name (optional)
        let has_name = seq
            .next_element::<bool>()?
            .ok_or(de::Error::custom("No PotionContents has_name bool!"))?;
        let custom_name = has_name
            .then(|| {
                seq.next_element::<String>()?
                    .ok_or(de::Error::custom("No PotionContents custom_name String!"))
            })
            .transpose()?;

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
        DataComponent::Enchantments => Ok(EnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Damage => Ok(DamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Unbreakable => Ok(UnbreakableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotionContents => Ok(PotionContentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::FireworkExplosion => Ok(FireworkExplosionImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Fireworks => Ok(FireworksImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ItemModel => Ok(ItemModelImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Consumable => Ok(ConsumableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Equippable => Ok(EquippableImpl::deserialize(seq)?.to_dyn()),
        _ => Err(serde::de::Error::custom("TODO")),
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
        DataComponent::FireworkExplosion => get::<FireworkExplosionImpl>(value).serialize(seq),
        DataComponent::Fireworks => get::<FireworksImpl>(value).serialize(seq),
        DataComponent::ItemModel => get::<ItemModelImpl>(value).serialize(seq),
        DataComponent::Consumable => get::<ConsumableImpl>(value).serialize(seq),
        DataComponent::Equippable => get::<EquippableImpl>(value).serialize(seq),
        _ => todo!("{} not yet implemented", id.to_name()),
    }
}

use std::borrow::Cow;

use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    BundleContentsImpl, ConsumableImpl, ConsumeEffect, CustomDataImpl, CustomNameImpl, DamageImpl,
    DataComponentImpl, EnchantmentsImpl, EquippableImpl, FireworkExplosionImpl, FireworksImpl,
    IDSet, IDSetContent, IdOr, ItemModelImpl, MapIdImpl, MaxStackSizeImpl,
    OminousBottleAmplifierImpl, PotionContentsImpl, SoundEvent, StatusEffectInstance,
    StoredEnchantmentsImpl, UnbreakableImpl, UseCooldownImpl, get,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::sound::Sound;

mod impls;

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
pub fn proto_to_data_sound(id_or: &crate::IdOr<crate::SoundEvent>) -> Option<IdOr<SoundEvent>> {
    match id_or {
        crate::IdOr::Id(id) => {
            let name = Sound::NAMES.get(*id as usize)?;
            Some(IdOr::Id(Sound::from_name(name)?))
        }
        crate::IdOr::Value(sound) => Some(IdOr::Value(SoundEvent {
            sound_name: sound.sound_name.clone(),
            range: sound.range,
        })),
    }
}

fn deserialize_idset<T: IDSetContent>(
    seq: &mut impl NetworkReadExt,
) -> Result<IDSet<T>, ReadingError> {
    let id_type = seq.get_var_int()?.0;

    match id_type.cmp(&0) {
        std::cmp::Ordering::Equal => {
            let tag = seq.get_str()?;
            Ok(IDSet::Tag(Cow::Owned(tag.into())))
        }
        std::cmp::Ordering::Greater => {
            let len = id_type - 1;
            let mut content_vec = Vec::with_capacity(len as usize);

            for _ in 0..len {
                let varint_id = seq.get_var_int()?.0;

                let elmt = T::from_id(varint_id as u16).ok_or(ReadingError::Message(
                    "Invalid registry id VarInt in IDSet".into(),
                ))?;
                content_vec.push(elmt);
            }
            Ok(IDSet::IDs(Cow::Owned(content_vec)))
        }
        std::cmp::Ordering::Less => Result::Err(ReadingError::Message(
            "Negative type/len VarInt in IDSet".into(),
        )),
    }
}

fn serialize_idset<C: IDSetContent>(
    idset: &IDSet<C>,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    match idset {
        IDSet::Tag(tag) => {
            seq.write_var_int(&VarInt(0))?;
            seq.write_string(tag)
        }
        IDSet::IDs(elements) => {
            seq.write_var_int(&VarInt(elements.len() as i32 + 1))?;
            for elmt in elements.iter() {
                seq.write_var_int(&VarInt(elmt.registry_id() as i32))?;
            }
            Ok(())
        }
    }
}

fn deserialize_status_effects(
    seq: &mut impl NetworkReadExt,
) -> Result<Vec<StatusEffectInstance>, ReadingError> {
    let effects_len = seq.get_var_int()?.0 as usize;
    if effects_len > MAX_STATUS_EFFECTS {
        return Err(ReadingError::Message("Too many status effects".into()));
    }
    let mut custom_effects = Vec::with_capacity(effects_len);
    for _ in 0..effects_len {
        let effect_registry_id = seq.get_var_int()?.0;
        let effect_name = StatusEffect::from_id(effect_registry_id as u16)
            .ok_or(ReadingError::Message("Invalid effect_id!".into()))?
            .minecraft_name;
        let effect_id = Cow::Borrowed(effect_name);

        // Effect parameters
        let amplifier = seq.get_var_int()?.0;
        let duration = seq.get_var_int()?.0;
        let ambient = seq.get_bool()?;
        let show_particles = seq.get_bool()?;
        let show_icon = seq.get_bool()?;

        // Hidden effect (optional, recursive) - we skip it for now
        let has_hidden = seq.get_bool()?;
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

fn serialize_status_effects(
    effects: &Vec<StatusEffectInstance>,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    seq.write_var_int(&VarInt(effects.len() as i32))?;

    for effect in effects {
        let effect_id = StatusEffect::from_minecraft_name(&effect.effect_id)
            .ok_or_else(|| {
                WritingError::Message(format!("Invalid status effect: {}", effect.effect_id))
            })?
            .registry_id();
        seq.write_var_int(&VarInt(effect_id as i32))?;
        // Effect parameters
        seq.write_var_int(&VarInt::from(effect.amplifier))?;
        seq.write_var_int(&VarInt::from(effect.duration))?;
        seq.write_bool(effect.ambient)?;
        seq.write_bool(effect.show_particles)?;
        seq.write_bool(effect.show_icon)?;
        // No hidden effect for now
        seq.write_bool(false)?;
    }
    Ok(())
}

fn deserialize_consume_effect(
    seq: &mut impl NetworkReadExt,
) -> Result<ConsumeEffect, ReadingError> {
    let effect_type = seq.get_var_int()?.0;
    match effect_type {
        0 => {
            let probability = seq.get_f32()?;
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
            let diameter = seq.get_f32()?;
            Ok(ConsumeEffect::TeleportRandomly(diameter))
        }
        4 => {
            // Need to read IdOr<SoundEvent> manually. This depends on how it is serialized.
            // In vanilla, it's either an id (0) or a sound event (1) ... but wait, `crate::IdOr<crate::SoundEvent>` doesn't have a `NetworkReadExt` method.
            // Let's defer this and assume it implements `read` for now or wait, `IdOr` does implement `PacketRead` or something?
            // Actually, we can just use `IdOr::read` if we impl it, but let's change it to:
            let proto_sound_event = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.into();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })
            .map_err(|e| {
                ReadingError::Message(format!("No sound IdOr<SoundEvent> in ConsumeEffect: {e}"))
            })?;
            Ok(ConsumeEffect::PlaySound(
                proto_to_data_sound(&proto_sound_event).ok_or(ReadingError::Message(
                    "Invalid sound in ConsumeEffect".into(),
                ))?,
            ))
        }
        _ => Err(ReadingError::Message(
            "Invalid effect_type in ConsumeEffect".into(),
        )),
    }
}

fn serialize_consume_effect(
    consume_effect: &ConsumeEffect,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    seq.write_var_int(&VarInt(consume_effect.registry_id() as i32))?;
    match consume_effect {
        ConsumeEffect::ApplyEffects((effects, probability)) => {
            serialize_status_effects(&effects.to_vec(), seq)?;
            seq.write_f32(*probability)?;
        }
        ConsumeEffect::RemoveEffects(idset) => serialize_idset(idset, seq)?,
        ConsumeEffect::ClearAllEffects => (),
        ConsumeEffect::TeleportRandomly(diameter) => seq.write_f32(*diameter)?,
        ConsumeEffect::PlaySound(id_or) => {
            crate::IdOr::<crate::SoundEvent>::write(&data_to_proto_sound(id_or), seq, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            })?;
        }
    }
    Ok(())
}

trait DataComponentCodec<Impl: DataComponentImpl> {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError>;
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Impl, ReadingError>;
}

/// Helper to skip hidden effect parameters recursively
fn skip_effect_parameters(seq: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
    // amplifier
    seq.get_var_int()?;
    // duration
    seq.get_var_int()?;
    // ambient
    seq.get_bool()?;
    // show_particles
    seq.get_bool()?;
    // show_icon
    seq.get_bool()?;
    // has_hidden (recursive)
    let has_hidden = seq.get_bool()?;
    if has_hidden {
        skip_effect_parameters(seq)?;
    }
    Ok(())
}

pub fn deserialize(
    id: DataComponent,
    seq: &mut impl NetworkReadExt,
) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    match id {
        DataComponent::MaxStackSize => Ok(MaxStackSizeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CustomData => Err(ReadingError::Message(
            "CustomData raw component decoding is not supported; use the custom-data item-stack API".into(),
        )),
        DataComponent::Enchantments => Ok(EnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Damage => Ok(DamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Unbreakable => Ok(UnbreakableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotionContents => Ok(PotionContentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::FireworkExplosion => Ok(FireworkExplosionImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Fireworks => Ok(FireworksImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ItemModel => Ok(ItemModelImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CustomName => Ok(CustomNameImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Consumable => Ok(ConsumableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Equippable => Ok(EquippableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::StoredEnchantments => Ok(StoredEnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::UseCooldown => Ok(UseCooldownImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MapId => Ok(MapIdImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BundleContents => Ok(BundleContentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::OminousBottleAmplifier => {
            Ok(OminousBottleAmplifierImpl::deserialize(seq)?.to_dyn())
        }
        _ => Err(ReadingError::Message(format!("{id:?} (TODO)"))),
    }
}
pub fn serialize(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    match id {
        DataComponent::MaxStackSize => get::<MaxStackSizeImpl>(value).serialize(seq),
        DataComponent::CustomData => get::<CustomDataImpl>(value).serialize(seq),
        DataComponent::Enchantments => get::<EnchantmentsImpl>(value).serialize(seq),
        DataComponent::Damage => get::<DamageImpl>(value).serialize(seq),
        DataComponent::Unbreakable => get::<UnbreakableImpl>(value).serialize(seq),
        DataComponent::PotionContents => get::<PotionContentsImpl>(value).serialize(seq),
        DataComponent::FireworkExplosion => get::<FireworkExplosionImpl>(value).serialize(seq),
        DataComponent::Fireworks => get::<FireworksImpl>(value).serialize(seq),
        DataComponent::ItemModel => get::<ItemModelImpl>(value).serialize(seq),
        DataComponent::CustomName => get::<CustomNameImpl>(value).serialize(seq),
        DataComponent::Consumable => get::<ConsumableImpl>(value).serialize(seq),
        DataComponent::Equippable => get::<EquippableImpl>(value).serialize(seq),
        DataComponent::StoredEnchantments => get::<StoredEnchantmentsImpl>(value).serialize(seq),
        DataComponent::UseCooldown => get::<UseCooldownImpl>(value).serialize(seq),
        DataComponent::MapId => get::<MapIdImpl>(value).serialize(seq),
        DataComponent::BundleContents => get::<BundleContentsImpl>(value).serialize(seq),
        DataComponent::OminousBottleAmplifier => {
            get::<OminousBottleAmplifierImpl>(value).serialize(seq)
        }
        _ => Err(WritingError::Message(format!(
            "{} not yet implemented",
            id.to_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{
        DamageImpl, DataComponentImpl, FireworkExplosionImpl, FireworkExplosionShape,
        FireworksImpl, IdOr, ItemModelImpl, MapIdImpl, MaxStackSizeImpl,
        OminousBottleAmplifierImpl, SoundEvent, UnbreakableImpl, UseCooldownImpl, get,
    };

    use super::{data_to_proto_sound, deserialize, proto_to_data_sound, serialize};

    fn encode(id: DataComponent, value: &dyn DataComponentImpl) -> Vec<u8> {
        let mut bytes = Vec::new();
        serialize(id, value, &mut bytes).unwrap();
        bytes
    }

    fn decode(id: DataComponent, mut bytes: &[u8]) -> Box<dyn DataComponentImpl> {
        deserialize(id, &mut bytes).unwrap()
    }

    #[test]
    fn damage_codec_roundtrip() {
        let value = DamageImpl { damage: 5 };
        let bytes = encode(DataComponent::Damage, &value);
        assert_eq!(bytes, [0x05]);
        let decoded = decode(DataComponent::Damage, &bytes);
        assert_eq!(get::<DamageImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn max_stack_size_codec_roundtrip() {
        let value = MaxStackSizeImpl { size: 16 };
        let bytes = encode(DataComponent::MaxStackSize, &value);
        assert_eq!(bytes, [0x10]);
        let decoded = decode(DataComponent::MaxStackSize, &bytes);
        assert_eq!(get::<MaxStackSizeImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn unbreakable_codec_roundtrip() {
        let value = UnbreakableImpl;
        let bytes = encode(DataComponent::Unbreakable, &value);
        assert_eq!(bytes, [0u8; 0]);
        let decoded = decode(DataComponent::Unbreakable, &bytes);
        assert_eq!(get::<UnbreakableImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn item_model_codec_roundtrip() {
        let value = ItemModelImpl {
            id: Cow::Borrowed("minecraft:stick"),
        };
        let bytes = encode(DataComponent::ItemModel, &value);
        // VarInt string length prefix followed by the UTF-8 payload.
        assert_eq!(bytes[0], 15);
        assert_eq!(&bytes[1..], b"minecraft:stick");
        let decoded = decode(DataComponent::ItemModel, &bytes);
        assert_eq!(get::<ItemModelImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn map_id_codec_roundtrip() {
        let value = MapIdImpl { id: 7 };
        let bytes = encode(DataComponent::MapId, &value);
        assert_eq!(bytes, [0x07]);
        let decoded = decode(DataComponent::MapId, &bytes);
        assert_eq!(get::<MapIdImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn ominous_bottle_amplifier_codec_roundtrip() {
        let value = OminousBottleAmplifierImpl { amplifier: 3 };
        let bytes = encode(DataComponent::OminousBottleAmplifier, &value);
        assert_eq!(bytes, [0x03]);
        let decoded = decode(DataComponent::OminousBottleAmplifier, &bytes);
        assert_eq!(get::<OminousBottleAmplifierImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn use_cooldown_codec_roundtrip() {
        let value = UseCooldownImpl {
            seconds: 1.5,
            cooldown_group: None,
        };
        let bytes = encode(DataComponent::UseCooldown, &value);
        // 1.5f32 big-endian followed by the "no cooldown group" flag.
        assert_eq!(bytes, [0x3F, 0xC0, 0x00, 0x00, 0x00]);
        let decoded = decode(DataComponent::UseCooldown, &bytes);
        assert_eq!(get::<UseCooldownImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn fireworks_codec_roundtrip() {
        let shape = FireworkExplosionShape::from_id(0).unwrap();
        let explosion =
            FireworkExplosionImpl::new(shape, vec![0x00FF_0000], vec![0x0000_FF00], true, false);
        let value = FireworksImpl::new(2, vec![explosion]);
        let bytes = encode(DataComponent::Fireworks, &value);
        let decoded = decode(DataComponent::Fireworks, &bytes);
        assert_eq!(get::<FireworksImpl>(decoded.as_ref()), &value);
    }

    #[test]
    fn proto_data_sound_id_roundtrip() {
        let _: fn(&IdOr<SoundEvent>) -> crate::IdOr<crate::SoundEvent> = data_to_proto_sound;
        let _: fn(&crate::IdOr<crate::SoundEvent>) -> Option<IdOr<SoundEvent>> =
            proto_to_data_sound;

        let proto: crate::IdOr<crate::SoundEvent> = crate::IdOr::Id(0);
        let data = proto_to_data_sound(&proto).unwrap();
        assert!(matches!(data_to_proto_sound(&data), crate::IdOr::Id(0)));
    }
}

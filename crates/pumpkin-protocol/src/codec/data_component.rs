#![allow(clippy::wildcard_imports)]

use std::borrow::Cow;

use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::*;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::{serializer::NbtWriteHelperJava, tag::NbtTag};
use pumpkin_util::version::JavaMinecraftVersion;

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

pub(crate) trait DataComponentCodec<Impl: DataComponentImpl> {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError>;
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Impl, ReadingError>;
}

impl DataComponentCodec<Self> for MaxStackSizeImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.size))
    }
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let size = u8::try_from(seq.get_var_int()?.0)
            .map_err(|_| ReadingError::Message("No MaxStackSize VarInt!".into()))?;
        Ok(Self { size })
    }
}

impl DataComponentCodec<Self> for DamageImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.damage))
    }
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let damage = seq.get_var_int()?.0;
        Ok(Self { damage })
    }
}

impl DataComponentCodec<Self> for RepairCostImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.cost))
    }
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let cost = seq.get_var_int()?.0;
        Ok(Self { cost })
    }
}

impl DataComponentCodec<Self> for EnchantmentsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.enchantment.len() as i32))?;
        for (enc, level) in self.enchantment.iter() {
            seq.write_var_int(&VarInt::from(enc.id))?;
            seq.write_var_int(&VarInt::from(*level))?;
        }
        Ok(())
    }
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_ENCHANTMENTS: usize = 256;

        let len = seq.get_var_int()?.0 as usize;
        if len > MAX_ENCHANTMENTS {
            return Err(ReadingError::Message("Too many enchantments".into()));
        }
        let mut enc = Vec::with_capacity(len);
        for _ in 0..len {
            let id = seq.get_var_int()?.0 as u8;
            let level = seq.get_var_int()?.0;
            enc.push((
                Enchantment::from_id(id).ok_or(ReadingError::Message(
                    "EnchantmentsImpl Enchantment VarInt Incorrect!".into(),
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
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }
    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for ItemModelImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_string(&self.id)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_str()?;
        Ok(Self {
            id: Cow::Owned(id.into()),
        })
    }
}

impl DataComponentCodec<Self> for CustomNameImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let mut bytes = Vec::new();
        NbtTag::String(self.name.clone().get_text().into_boxed_str())
            .serialize(&mut NbtWriteHelperJava::new(&mut bytes))
            .map_err(|e| WritingError::Message(e.to_string()))?;
        seq.write_slice(&bytes)?;
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let tag = seq.get_nbt_with_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2)?;
        let name = tag.as_ref().map_or_else(
            pumpkin_util::text::TextComponent::empty,
            pumpkin_util::text::TextComponent::from_nbt,
        );
        Ok(Self { name })
    }
}

impl DataComponentCodec<Self> for LoreImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(self.lines.len() as i32))?;
        for line in &self.lines {
            seq.write_slice(
                &line.encode_for_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2),
            )?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        // TODO: Could probably be extracted?
        const MAX_LORE_LINES: i32 = 256;

        let count = seq.get_var_int()?.0;
        if !(0..=MAX_LORE_LINES).contains(&count) {
            return Err(ReadingError::Message(format!(
                "LoreImpl line count {count} is out of bounds (0-{MAX_LORE_LINES})"
            )));
        }

        let mut lines = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let tag =
                seq.get_nbt_with_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2)?;
            let text = tag.as_ref().map_or_else(
                pumpkin_util::text::TextComponent::empty,
                pumpkin_util::text::TextComponent::from_nbt,
            );
            lines.push(text);
        }
        Ok(Self { lines })
    }
}

impl DataComponentCodec<Self> for ItemNameImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let mut name = pumpkin_nbt::compound::NbtCompound::new();
        name.put_string("translate", self.name.to_string());
        let mut bytes = Vec::new();
        NbtTag::Compound(name)
            .serialize(&mut NbtWriteHelperJava::new(&mut bytes))
            .map_err(|error| WritingError::Message(error.to_string()))?;
        seq.write_slice(&bytes)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let name = seq.get_str()?;
        Ok(Self {
            name: Cow::Owned(name.into()),
        })
    }
}

impl DataComponentCodec<Self> for DyedColorImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_i32(self.rgb)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            rgb: seq.get_i32()?,
        })
    }
}

impl DataComponentCodec<Self> for SuspiciousStewEffectsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let effect_count = i32::try_from(self.effects.len())
            .map_err(|_| WritingError::Message("Too many suspicious stew effects".into()))?;
        seq.write_var_int(&VarInt(effect_count))?;
        for effect in self.effects.iter() {
            let id = StatusEffect::from_minecraft_name(&effect.effect)
                .ok_or_else(|| WritingError::Message("Unknown suspicious stew effect".into()))?
                .id;
            seq.write_var_int(&VarInt(i32::from(id)))?;
            seq.write_var_int(&VarInt(effect.duration))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_EFFECTS: i32 = 128;

        let count = seq.get_var_int()?.0;
        if !(0..=MAX_EFFECTS).contains(&count) {
            return Err(ReadingError::Message(
                "Invalid suspicious stew effect count".into(),
            ));
        }

        let mut effects =
            Vec::with_capacity(usize::try_from(count).map_err(|_| {
                ReadingError::Message("Invalid suspicious stew effect count".into())
            })?);
        for _ in 0..count {
            let id = u16::try_from(seq.get_var_int()?.0)
                .map_err(|_| ReadingError::Message("Invalid suspicious stew effect id".into()))?;
            let effect = StatusEffect::from_id(id)
                .ok_or_else(|| ReadingError::Message("Unknown suspicious stew effect id".into()))?;
            let duration = seq.get_var_int()?.0;
            effects.push(SuspiciousStewEffect {
                effect: Cow::Borrowed(effect.minecraft_name),
                duration,
            });
        }
        Ok(Self {
            effects: Cow::Owned(effects),
        })
    }
}

impl DataComponentCodec<Self> for CustomDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let mut bytes = Vec::new();
        NbtTag::Compound(self.data.clone())
            .serialize(&mut NbtWriteHelperJava::new(&mut bytes))
            .map_err(|e| WritingError::Message(e.to_string()))?;
        seq.write_slice(&bytes)?;
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let data = seq
            .get_compound_nbt_with_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2)?
            .unwrap_or_else(pumpkin_nbt::compound::NbtCompound::new);
        Ok(Self { data })
    }
}

impl DataComponentCodec<Self> for ConsumableImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(self.consume_seconds)?;
        seq.write_var_int(&VarInt(self.animation as i32))?;
        crate::IdOr::<crate::SoundEvent>::write(
            &data_to_proto_sound(&self.sound_event),
            seq,
            |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            },
        )?;
        seq.write_bool(self.consume_particles)?;
        seq.write_var_int(&VarInt(self.effects.len() as i32))?;

        for effect in self.effects.iter() {
            serialize_consume_effect(effect, seq)?;
        }

        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let consume_seconds = seq.get_f32()?;
        let animation_id = seq.get_var_int()?;

        let animation: ConsumeAnimation = animation_id
            .0
            .try_into()
            .map_err(|()| ReadingError::Message("Invalid ConsumableImpl animation id!".into()))?;
        let proto_sound_event = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
            let sound_name = r.get_str()?.into();
            let range = r.get_option(NetworkReadExt::get_f32)?;
            Ok(crate::SoundEvent { sound_name, range })
        })?;
        let consume_particles = seq.get_bool()?;

        let sound_event = proto_to_data_sound(&proto_sound_event).ok_or(ReadingError::Message(
            "Invalid sound in ConsumableImpl".into(),
        ))?;
        let effects_len = seq.get_var_int()?.0;
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
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(self.slot.get_slot_index()))?;
        crate::IdOr::<crate::SoundEvent>::write(
            &data_to_proto_sound(&self.equip_sound),
            seq,
            |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            },
        )?;

        seq.write_bool(self.asset_id.is_some())?;
        if let Some(asset) = &self.asset_id {
            seq.write_string(asset)?;
        }

        seq.write_bool(self.camera_overlay.is_some())?;
        if let Some(overlay) = &self.camera_overlay {
            seq.write_string(overlay)?;
        }

        seq.write_bool(self.allowed_entities.is_some())?;
        if let Some(allowed) = &self.allowed_entities {
            serialize_idset(allowed, seq)?;
        }

        seq.write_bool(self.dispensable)?;
        seq.write_bool(self.swappable)?;
        seq.write_bool(self.damage_on_hurt)?;
        seq.write_bool(self.equip_on_interact)?;
        seq.write_bool(self.can_be_sheared)?;
        crate::IdOr::<crate::SoundEvent>::write(
            &data_to_proto_sound(&self.shearing_sound),
            seq,
            |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            },
        )
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let slot_index = seq.get_var_int()?.0;
        let slot = EquipmentSlot::from_slot_index(slot_index).ok_or(ReadingError::Message(
            format!("Invalid equipment slot index {slot_index}"),
        ))?;
        let equip_sound = proto_to_data_sound(&crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
            let sound_name = r.get_str()?.into();
            let range = r.get_option(NetworkReadExt::get_f32)?;
            Ok(crate::SoundEvent { sound_name, range })
        })?)
        .ok_or(ReadingError::Message(
            "Invalid sound in EquippableImpl".into(),
        ))?;

        let asset_id = if seq.get_bool()? {
            Some(Cow::Owned(seq.get_str()?.into()))
        } else {
            None
        };

        let camera_overlay = if seq.get_bool()? {
            Some(Cow::Owned(seq.get_str()?.into()))
        } else {
            None
        };

        let has_allowed_entities = seq.get_bool()?;

        let allowed_entities: Option<IDSet<EntityType>> = if has_allowed_entities {
            Some(deserialize_idset(seq)?)
        } else {
            None
        };

        let dispensable = seq.get_bool()?;
        let swappable = seq.get_bool()?;
        let damage_on_hurt = seq.get_bool()?;
        let equip_on_interact = seq.get_bool()?;
        let can_be_sheared = seq.get_bool()?;
        let shearing_sound =
            proto_to_data_sound(&crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.into();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })?)
            .ok_or(ReadingError::Message(
                "Invalid shearing sound in EquippableImpl".into(),
            ))?;

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
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        // Potion ID (optional)
        if let Some(potion_id) = self.potion_id {
            seq.write_bool(true)?;
            seq.write_var_int(&VarInt::from(potion_id))?;
        } else {
            seq.write_bool(false)?;
        }

        // Custom color (optional)
        if let Some(color) = self.custom_color {
            seq.write_bool(true)?;
            seq.write_i32(color)?;
        } else {
            seq.write_bool(false)?;
        }

        // Custom effects list
        serialize_status_effects(&self.custom_effects, seq)?;

        // Custom name (optional)
        if let Some(name) = &self.custom_name {
            seq.write_bool(true)?;
            seq.write_string(name.as_str())?;
        } else {
            seq.write_bool(false)?;
        }

        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        // Potion ID (optional)
        let has_potion = seq.get_bool()?;
        let potion_id = has_potion
            .then(|| seq.get_var_int().map(|value| value.0))
            .transpose()?;

        // Custom color (optional)
        let has_color = seq.get_bool()?;
        let custom_color = has_color.then(|| seq.get_i32()).transpose()?;

        // Custom effects list
        let custom_effects = deserialize_status_effects(seq)?;

        // Custom name (optional)
        let has_name = seq.get_bool()?;
        let custom_name = has_name
            .then(|| seq.get_str().map(String::from))
            .transpose()?;

        Ok(Self {
            potion_id,
            custom_color,
            custom_effects,
            custom_name,
        })
    }
}

/// Helper to skip hidden effect parameters iteratively with a depth cap
fn skip_effect_parameters(seq: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
    const MAX_EFFECT_DEPTH: usize = 32;
    let mut depth = 0;
    loop {
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
        // has_hidden
        let has_hidden = seq.get_bool()?;
        if !has_hidden {
            break;
        }
        depth += 1;
        if depth > MAX_EFFECT_DEPTH {
            return Err(ReadingError::TooLarge(
                "Potion effect hidden depth exceeded".into(),
            ));
        }
    }
    Ok(())
}

impl DataComponentCodec<Self> for FireworkExplosionImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        // Shape (VarInt enum)
        seq.write_var_int(&VarInt::from(self.shape.to_id()))?;
        // Colors list
        seq.write_var_int(&VarInt::from(self.colors.len() as i32))?;
        for color in &self.colors {
            seq.write_i32(*color)?;
        }
        // Fade colors list
        seq.write_var_int(&VarInt::from(self.fade_colors.len() as i32))?;
        for color in &self.fade_colors {
            seq.write_i32(*color)?;
        }
        // hasTrail
        seq.write_bool(self.has_trail)?;
        // hasTwinkle
        seq.write_bool(self.has_twinkle)?;
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        // Needs a length cap during deserialization to prevent OOM from malicious packets
        // Vanilla doesn't have any limits (Integer.MAX_VALUE is technically a limit but not enforced in practice)
        const MAX_COLORS: usize = 256;
        const MAX_FADE_COLORS: usize = 256;

        // Shape (VarInt enum)
        let shape_id = seq.get_var_int()?.0;
        let shape = FireworkExplosionShape::from_id(shape_id).ok_or(ReadingError::Message(
            "Invalid FireworkExplosionShape id!".into(),
        ))?;

        // Colors list
        let colors_len = seq.get_var_int()?.0 as usize;
        if colors_len > MAX_COLORS {
            return Err(ReadingError::Message(format!(
                "FireworkExplosionImpl colors_len {colors_len} exceeds maximum of {MAX_COLORS}"
            )));
        }
        let mut colors = Vec::with_capacity(colors_len);
        for _ in 0..colors_len {
            let color = seq.get_i32()?;
            colors.push(color);
        }

        // Fade colors list
        let fade_colors_len = seq.get_var_int()?.0 as usize;
        if fade_colors_len > MAX_FADE_COLORS {
            return Err(ReadingError::Message(format!(
                "FireworkExplosionImpl fade_colors_len {fade_colors_len} exceeds maximum of {MAX_FADE_COLORS}"
            )));
        }
        let mut fade_colors = Vec::with_capacity(fade_colors_len);
        for _ in 0..fade_colors_len {
            let color = seq.get_i32()?;
            fade_colors.push(color);
        }

        // hasTrail
        let has_trail = seq.get_bool()?;

        // hasTwinkle
        let has_twinkle = seq.get_bool()?;

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
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        // Flight duration (VarInt)
        seq.write_var_int(&VarInt::from(self.flight_duration))?;
        // Explosions list
        seq.write_var_int(&VarInt::from(self.explosions.len() as i32))?;
        for explosion in &self.explosions {
            explosion.serialize(seq)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        // Needs a length cap during deserialization to prevent OOM from malicious packets
        // Vanilla doesn't have any limits
        const MAX_EXPLOSIONS: usize = 256;
        // Vanilla restricts to 0-255 (UNSIGNED_BYTE in data component codec) (do not trust client NBT to limit it)
        const MAX_FLIGHT_DURATION: i32 = 255;

        // Flight duration
        let flight_duration = seq.get_var_int()?.0;
        if !(0..=MAX_FLIGHT_DURATION).contains(&flight_duration) {
            return Err(ReadingError::Message(format!(
                "FireworksImpl flight_duration {flight_duration} is out of bounds (0-{MAX_FLIGHT_DURATION})"
            )));
        }

        // Explosions list
        let explosions_len = seq.get_var_int()?.0 as usize;
        if explosions_len > MAX_EXPLOSIONS {
            return Err(ReadingError::Message(format!(
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

impl DataComponentCodec<Self> for StoredEnchantmentsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.enchantment.len() as i32))?;
        for (enc, level) in self.enchantment.iter() {
            seq.write_var_int(&VarInt::from(enc.id))?;
            seq.write_var_int(&VarInt::from(*level))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_ENCHANTMENTS: usize = 256;

        let len = seq.get_var_int()?.0 as usize;

        if len > MAX_ENCHANTMENTS {
            return Err(ReadingError::Message("Too many enchantments".into()));
        }

        let mut stored_enchantments = Vec::with_capacity(len);
        for _ in 0..len {
            let id = seq.get_var_int()?.0 as u8;
            let level = seq.get_var_int()?.0;
            stored_enchantments.push((
                Enchantment::from_id(id).ok_or(ReadingError::Message(
                    "StoredEnchantmentsImpl Enchantment VarInt Incorrect!".into(),
                ))?,
                level,
            ));
        }
        Ok(Self {
            enchantment: Cow::from(stored_enchantments),
        })
    }
}

impl DataComponentCodec<Self> for RepairableImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_idset(&self.items, seq)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            items: deserialize_idset(seq)?,
        })
    }
}

impl DataComponentCodec<Self> for SwingAnimationImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.animation_type.to_id()))?;
        seq.write_var_int(&VarInt::from(self.duration))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let type_id = seq.get_var_int()?.0;
        let animation_type = SwingAnimationType::from_id(type_id).ok_or_else(|| {
            ReadingError::Message(format!("Invalid SwingAnimationType id {type_id}"))
        })?;
        let duration = seq.get_var_int()?.0;
        Ok(Self {
            animation_type,
            duration,
        })
    }
}

impl DataComponentCodec<Self> for RarityImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.rarity.to_id()))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let rarity = Rarity::from_id(id)
            .ok_or_else(|| ReadingError::Message(format!("Invalid Rarity id {id}")))?;
        Ok(Self { rarity })
    }
}

#[allow(clippy::too_many_lines)]
pub fn deserialize(
    id: DataComponent,
    seq: &mut impl NetworkReadExt,
) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    match id {
        DataComponent::CustomData => Ok(CustomDataImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MaxStackSize => Ok(MaxStackSizeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MaxDamage => Ok(MaxDamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Damage => Ok(DamageImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Unbreakable => Ok(UnbreakableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::UseEffects => Ok(UseEffectsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CustomName => Ok(CustomNameImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MinimumAttackCharge => {
            Ok(MinimumAttackChargeImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::DamageType => Ok(DamageTypeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ItemName => Ok(ItemNameImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ItemModel => Ok(ItemModelImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Lore => Ok(LoreImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Rarity => Ok(RarityImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Enchantments => Ok(EnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CanPlaceOn => Ok(CanPlaceOnImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CanBreak => Ok(CanBreakImpl::deserialize(seq)?.to_dyn()),
        DataComponent::AttributeModifiers => Ok(AttributeModifiersImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CustomModelData => Ok(CustomModelDataImpl::deserialize(seq)?.to_dyn()),
        DataComponent::TooltipDisplay => Ok(TooltipDisplayImpl::deserialize(seq)?.to_dyn()),
        DataComponent::RepairCost => Ok(RepairCostImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CreativeSlotLock => Ok(CreativeSlotLockImpl::deserialize(seq)?.to_dyn()),
        DataComponent::EnchantmentGlintOverride => {
            Ok(EnchantmentGlintOverrideImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::IntangibleProjectile => {
            Ok(IntangibleProjectileImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::Food => Ok(FoodImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Consumable => Ok(ConsumableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::UseRemainder => Ok(UseRemainderImpl::deserialize(seq)?.to_dyn()),
        DataComponent::UseCooldown => Ok(UseCooldownImpl::deserialize(seq)?.to_dyn()),
        DataComponent::DamageResistant => Ok(DamageResistantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Tool => Ok(ToolImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Weapon => Ok(WeaponImpl::deserialize(seq)?.to_dyn()),
        DataComponent::AttackRange => Ok(AttackRangeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Enchantable => Ok(EnchantableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Equippable => Ok(EquippableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Repairable => Ok(RepairableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Glider => Ok(GliderImpl::deserialize(seq)?.to_dyn()),
        DataComponent::TooltipStyle => Ok(TooltipStyleImpl::deserialize(seq)?.to_dyn()),
        DataComponent::DeathProtection => Ok(DeathProtectionImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BlocksAttacks => Ok(BlocksAttacksImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PiercingWeapon => Ok(PiercingWeaponImpl::deserialize(seq)?.to_dyn()),
        DataComponent::KineticWeapon => Ok(KineticWeaponImpl::deserialize(seq)?.to_dyn()),
        DataComponent::SwingAnimation => Ok(SwingAnimationImpl::deserialize(seq)?.to_dyn()),
        DataComponent::AdditionalTradeCost => {
            Ok(AdditionalTradeCostImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::StoredEnchantments => Ok(StoredEnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Dye => Ok(DyeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::DyedColor => Ok(DyedColorImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MapColor => Ok(MapColorImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MapId => Ok(MapIdImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MapDecorations => Ok(MapDecorationsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MapPostProcessing => Ok(MapPostProcessingImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ChargedProjectiles => Ok(ChargedProjectilesImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BundleContents => Ok(BundleContentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotionContents => Ok(PotionContentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotionDurationScale => {
            Ok(PotionDurationScaleImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::SuspiciousStewEffects => {
            Ok(SuspiciousStewEffectsImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::WritableBookContent => {
            Ok(WritableBookContentImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::WrittenBookContent => Ok(WrittenBookContentImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Trim => Ok(TrimImpl::deserialize(seq)?.to_dyn()),
        DataComponent::DebugStickState => Ok(DebugStickStateImpl::deserialize(seq)?.to_dyn()),
        DataComponent::EntityData => Ok(EntityDataImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BucketEntityData => Ok(BucketEntityDataImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BlockEntityData => Ok(BlockEntityDataImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Instrument => Ok(InstrumentImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ProvidesTrimMaterial => {
            Ok(ProvidesTrimMaterialImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::OminousBottleAmplifier => {
            Ok(OminousBottleAmplifierImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::JukeboxPlayable => Ok(JukeboxPlayableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ProvidesBannerPatterns => {
            Ok(ProvidesBannerPatternsImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::Recipes => Ok(RecipesImpl::deserialize(seq)?.to_dyn()),
        DataComponent::LodestoneTracker => Ok(LodestoneTrackerImpl::deserialize(seq)?.to_dyn()),
        DataComponent::FireworkExplosion => Ok(FireworkExplosionImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Fireworks => Ok(FireworksImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Profile => Ok(ProfileImpl::deserialize(seq)?.to_dyn()),
        DataComponent::NoteBlockSound => Ok(NoteBlockSoundImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BannerPatterns => Ok(BannerPatternsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BaseColor => Ok(BaseColorImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PotDecorations => Ok(PotDecorationsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Container => Ok(ContainerImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BlockState => Ok(BlockStateImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Bees => Ok(BeesImpl::deserialize(seq)?.to_dyn()),
        DataComponent::SulfurCubeContent => Ok(SulfurCubeContentImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Lock => Ok(LockImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ContainerLoot => Ok(ContainerLootImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BreakSound => Ok(BreakSoundImpl::deserialize(seq)?.to_dyn()),
        DataComponent::VillagerVariant => Ok(VillagerVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::WolfVariant => Ok(WolfVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::WolfSoundVariant => Ok(WolfSoundVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::WolfCollar => Ok(WolfCollarImpl::deserialize(seq)?.to_dyn()),
        DataComponent::FoxVariant => Ok(FoxVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::SalmonSize => Ok(SalmonSizeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ParrotVariant => Ok(ParrotVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::TropicalFishPattern => {
            Ok(TropicalFishPatternImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::TropicalFishBaseColor => {
            Ok(TropicalFishBaseColorImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::TropicalFishPatternColor => {
            Ok(TropicalFishPatternColorImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::MooshroomVariant => Ok(MooshroomVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::RabbitVariant => Ok(RabbitVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PigVariant => Ok(PigVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PigSoundVariant => Ok(PigSoundVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CowVariant => Ok(CowVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CowSoundVariant => Ok(CowSoundVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ChickenVariant => Ok(ChickenVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ChickenSoundVariant => {
            Ok(ChickenSoundVariantImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::ZombieNautilusVariant => {
            Ok(ZombieNautilusVariantImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::FrogVariant => Ok(FrogVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::HorseVariant => Ok(HorseVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::PaintingVariant => Ok(PaintingVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::LlamaVariant => Ok(LlamaVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::AxolotlVariant => Ok(AxolotlVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CatVariant => Ok(CatVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CatSoundVariant => Ok(CatSoundVariantImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CatCollar => Ok(CatCollarImpl::deserialize(seq)?.to_dyn()),
        DataComponent::SheepColor => Ok(SheepColorImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ShulkerColor => Ok(ShulkerColorImpl::deserialize(seq)?.to_dyn()),
    }
}

#[allow(clippy::too_many_lines)]
pub fn serialize(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    match id {
        DataComponent::CustomData => get::<CustomDataImpl>(value).serialize(seq),
        DataComponent::MaxStackSize => get::<MaxStackSizeImpl>(value).serialize(seq),
        DataComponent::MaxDamage => get::<MaxDamageImpl>(value).serialize(seq),
        DataComponent::Damage => get::<DamageImpl>(value).serialize(seq),
        DataComponent::Unbreakable => get::<UnbreakableImpl>(value).serialize(seq),
        DataComponent::UseEffects => get::<UseEffectsImpl>(value).serialize(seq),
        DataComponent::CustomName => get::<CustomNameImpl>(value).serialize(seq),
        DataComponent::MinimumAttackCharge => get::<MinimumAttackChargeImpl>(value).serialize(seq),
        DataComponent::DamageType => get::<DamageTypeImpl>(value).serialize(seq),
        DataComponent::ItemName => get::<ItemNameImpl>(value).serialize(seq),
        DataComponent::ItemModel => get::<ItemModelImpl>(value).serialize(seq),
        DataComponent::Lore => get::<LoreImpl>(value).serialize(seq),
        DataComponent::Rarity => get::<RarityImpl>(value).serialize(seq),
        DataComponent::Enchantments => get::<EnchantmentsImpl>(value).serialize(seq),
        DataComponent::CanPlaceOn => get::<CanPlaceOnImpl>(value).serialize(seq),
        DataComponent::CanBreak => get::<CanBreakImpl>(value).serialize(seq),
        DataComponent::AttributeModifiers => get::<AttributeModifiersImpl>(value).serialize(seq),
        DataComponent::CustomModelData => get::<CustomModelDataImpl>(value).serialize(seq),
        DataComponent::TooltipDisplay => get::<TooltipDisplayImpl>(value).serialize(seq),
        DataComponent::RepairCost => get::<RepairCostImpl>(value).serialize(seq),
        DataComponent::CreativeSlotLock => get::<CreativeSlotLockImpl>(value).serialize(seq),
        DataComponent::EnchantmentGlintOverride => {
            get::<EnchantmentGlintOverrideImpl>(value).serialize(seq)
        }
        DataComponent::IntangibleProjectile => {
            get::<IntangibleProjectileImpl>(value).serialize(seq)
        }
        DataComponent::Food => get::<FoodImpl>(value).serialize(seq),
        DataComponent::Consumable => get::<ConsumableImpl>(value).serialize(seq),
        DataComponent::UseRemainder => get::<UseRemainderImpl>(value).serialize(seq),
        DataComponent::UseCooldown => get::<UseCooldownImpl>(value).serialize(seq),
        DataComponent::DamageResistant => get::<DamageResistantImpl>(value).serialize(seq),
        DataComponent::Tool => get::<ToolImpl>(value).serialize(seq),
        DataComponent::Weapon => get::<WeaponImpl>(value).serialize(seq),
        DataComponent::AttackRange => get::<AttackRangeImpl>(value).serialize(seq),
        DataComponent::Enchantable => get::<EnchantableImpl>(value).serialize(seq),
        DataComponent::Equippable => get::<EquippableImpl>(value).serialize(seq),
        DataComponent::Repairable => get::<RepairableImpl>(value).serialize(seq),
        DataComponent::Glider => get::<GliderImpl>(value).serialize(seq),
        DataComponent::TooltipStyle => get::<TooltipStyleImpl>(value).serialize(seq),
        DataComponent::DeathProtection => get::<DeathProtectionImpl>(value).serialize(seq),
        DataComponent::BlocksAttacks => get::<BlocksAttacksImpl>(value).serialize(seq),
        DataComponent::PiercingWeapon => get::<PiercingWeaponImpl>(value).serialize(seq),
        DataComponent::KineticWeapon => get::<KineticWeaponImpl>(value).serialize(seq),
        DataComponent::SwingAnimation => get::<SwingAnimationImpl>(value).serialize(seq),
        DataComponent::AdditionalTradeCost => get::<AdditionalTradeCostImpl>(value).serialize(seq),
        DataComponent::StoredEnchantments => get::<StoredEnchantmentsImpl>(value).serialize(seq),
        DataComponent::Dye => get::<DyeImpl>(value).serialize(seq),
        DataComponent::DyedColor => get::<DyedColorImpl>(value).serialize(seq),
        DataComponent::MapColor => get::<MapColorImpl>(value).serialize(seq),
        DataComponent::MapId => get::<MapIdImpl>(value).serialize(seq),
        DataComponent::MapDecorations => get::<MapDecorationsImpl>(value).serialize(seq),
        DataComponent::MapPostProcessing => get::<MapPostProcessingImpl>(value).serialize(seq),
        DataComponent::ChargedProjectiles => get::<ChargedProjectilesImpl>(value).serialize(seq),
        DataComponent::BundleContents => get::<BundleContentsImpl>(value).serialize(seq),
        DataComponent::PotionContents => get::<PotionContentsImpl>(value).serialize(seq),
        DataComponent::PotionDurationScale => get::<PotionDurationScaleImpl>(value).serialize(seq),
        DataComponent::SuspiciousStewEffects => {
            get::<SuspiciousStewEffectsImpl>(value).serialize(seq)
        }
        DataComponent::WritableBookContent => get::<WritableBookContentImpl>(value).serialize(seq),
        DataComponent::WrittenBookContent => get::<WrittenBookContentImpl>(value).serialize(seq),
        DataComponent::Trim => get::<TrimImpl>(value).serialize(seq),
        DataComponent::DebugStickState => get::<DebugStickStateImpl>(value).serialize(seq),
        DataComponent::EntityData => get::<EntityDataImpl>(value).serialize(seq),
        DataComponent::BucketEntityData => get::<BucketEntityDataImpl>(value).serialize(seq),
        DataComponent::BlockEntityData => get::<BlockEntityDataImpl>(value).serialize(seq),
        DataComponent::Instrument => get::<InstrumentImpl>(value).serialize(seq),
        DataComponent::ProvidesTrimMaterial => {
            get::<ProvidesTrimMaterialImpl>(value).serialize(seq)
        }
        DataComponent::OminousBottleAmplifier => {
            get::<OminousBottleAmplifierImpl>(value).serialize(seq)
        }
        DataComponent::JukeboxPlayable => get::<JukeboxPlayableImpl>(value).serialize(seq),
        DataComponent::ProvidesBannerPatterns => {
            get::<ProvidesBannerPatternsImpl>(value).serialize(seq)
        }
        DataComponent::Recipes => get::<RecipesImpl>(value).serialize(seq),
        DataComponent::LodestoneTracker => get::<LodestoneTrackerImpl>(value).serialize(seq),
        DataComponent::FireworkExplosion => get::<FireworkExplosionImpl>(value).serialize(seq),
        DataComponent::Fireworks => get::<FireworksImpl>(value).serialize(seq),
        DataComponent::Profile => get::<ProfileImpl>(value).serialize(seq),
        DataComponent::NoteBlockSound => get::<NoteBlockSoundImpl>(value).serialize(seq),
        DataComponent::BannerPatterns => get::<BannerPatternsImpl>(value).serialize(seq),
        DataComponent::BaseColor => get::<BaseColorImpl>(value).serialize(seq),
        DataComponent::PotDecorations => get::<PotDecorationsImpl>(value).serialize(seq),
        DataComponent::Container => get::<ContainerImpl>(value).serialize(seq),
        DataComponent::BlockState => get::<BlockStateImpl>(value).serialize(seq),
        DataComponent::Bees => get::<BeesImpl>(value).serialize(seq),
        DataComponent::SulfurCubeContent => get::<SulfurCubeContentImpl>(value).serialize(seq),
        DataComponent::Lock => get::<LockImpl>(value).serialize(seq),
        DataComponent::ContainerLoot => get::<ContainerLootImpl>(value).serialize(seq),
        DataComponent::BreakSound => get::<BreakSoundImpl>(value).serialize(seq),
        DataComponent::VillagerVariant => get::<VillagerVariantImpl>(value).serialize(seq),
        DataComponent::WolfVariant => get::<WolfVariantImpl>(value).serialize(seq),
        DataComponent::WolfSoundVariant => get::<WolfSoundVariantImpl>(value).serialize(seq),
        DataComponent::WolfCollar => get::<WolfCollarImpl>(value).serialize(seq),
        DataComponent::FoxVariant => get::<FoxVariantImpl>(value).serialize(seq),
        DataComponent::SalmonSize => get::<SalmonSizeImpl>(value).serialize(seq),
        DataComponent::ParrotVariant => get::<ParrotVariantImpl>(value).serialize(seq),
        DataComponent::TropicalFishPattern => get::<TropicalFishPatternImpl>(value).serialize(seq),
        DataComponent::TropicalFishBaseColor => {
            get::<TropicalFishBaseColorImpl>(value).serialize(seq)
        }
        DataComponent::TropicalFishPatternColor => {
            get::<TropicalFishPatternColorImpl>(value).serialize(seq)
        }
        DataComponent::MooshroomVariant => get::<MooshroomVariantImpl>(value).serialize(seq),
        DataComponent::RabbitVariant => get::<RabbitVariantImpl>(value).serialize(seq),
        DataComponent::PigVariant => get::<PigVariantImpl>(value).serialize(seq),
        DataComponent::PigSoundVariant => get::<PigSoundVariantImpl>(value).serialize(seq),
        DataComponent::CowVariant => get::<CowVariantImpl>(value).serialize(seq),
        DataComponent::CowSoundVariant => get::<CowSoundVariantImpl>(value).serialize(seq),
        DataComponent::ChickenVariant => get::<ChickenVariantImpl>(value).serialize(seq),
        DataComponent::ChickenSoundVariant => get::<ChickenSoundVariantImpl>(value).serialize(seq),
        DataComponent::ZombieNautilusVariant => {
            get::<ZombieNautilusVariantImpl>(value).serialize(seq)
        }
        DataComponent::FrogVariant => get::<FrogVariantImpl>(value).serialize(seq),
        DataComponent::HorseVariant => get::<HorseVariantImpl>(value).serialize(seq),
        DataComponent::PaintingVariant => get::<PaintingVariantImpl>(value).serialize(seq),
        DataComponent::LlamaVariant => get::<LlamaVariantImpl>(value).serialize(seq),
        DataComponent::AxolotlVariant => get::<AxolotlVariantImpl>(value).serialize(seq),
        DataComponent::CatVariant => get::<CatVariantImpl>(value).serialize(seq),
        DataComponent::CatSoundVariant => get::<CatSoundVariantImpl>(value).serialize(seq),
        DataComponent::CatCollar => get::<CatCollarImpl>(value).serialize(seq),
        DataComponent::SheepColor => get::<SheepColorImpl>(value).serialize(seq),
        DataComponent::ShulkerColor => get::<ShulkerColorImpl>(value).serialize(seq),
    }
}

impl DataComponentCodec<Self> for MapIdImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        Ok(Self { id })
    }
}

impl DataComponentCodec<Self> for UseCooldownImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(self.seconds)?;
        seq.write_bool(self.cooldown_group.is_some())?;
        if let Some(group) = &self.cooldown_group {
            seq.write_string(group)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let seconds = seq.get_f32()?;
        let cooldown_group = if seq.get_bool()? {
            Some(seq.get_str()?.into())
        } else {
            None
        };
        Ok(Self {
            seconds,
            cooldown_group,
        })
    }
}

fn deserialize_item_stack_template(
    seq: &mut impl NetworkReadExt,
) -> Result<pumpkin_data::item_stack::ItemStack, ReadingError> {
    const MAX_COMPONENTS: i32 = 256;

    let item_id = seq.get_var_int()?.0 as u16;

    let count = seq.get_var_int()?.0 as u8;

    let num_to_add = seq.get_var_int()?.0;
    let num_to_remove = seq.get_var_int()?.0;

    if num_to_add < 0 || num_to_remove < 0 {
        return Err(ReadingError::Message("Negative component count".into()));
    }

    let total_components = num_to_add
        .checked_add(num_to_remove)
        .ok_or_else(|| ReadingError::Message("Component count overflow".into()))?;

    if total_components > MAX_COMPONENTS {
        return Err(ReadingError::Message(
            "Too many components in ItemStackTemplate patch".into(),
        ));
    }

    let mut patch = Vec::with_capacity((num_to_add + num_to_remove) as usize);

    for _ in 0..num_to_add {
        let id_val = seq.get_var_int()?.0;
        let id = DataComponent::try_from_id(id_val as u8)
            .ok_or_else(|| ReadingError::Message(format!("Unknown component ID: {id_val}")))?;

        let _byte_len = seq.get_var_int()?;

        let component_impl = deserialize(id, seq)?;
        patch.push((id, Some(component_impl)));
    }

    for _ in 0..num_to_remove {
        let id_val = seq.get_var_int()?.0;
        let id = DataComponent::try_from_id(id_val as u8)
            .ok_or_else(|| ReadingError::Message("Unknown component ID".into()))?;
        patch.push((id, None));
    }

    Ok(pumpkin_data::item_stack::ItemStack::new_with_component(
        count,
        pumpkin_data::item::Item::from_id(item_id).unwrap_or(&pumpkin_data::item::Item::AIR),
        patch,
    ))
}

fn serialize_item_stack_template(
    stack: &pumpkin_data::item_stack::ItemStack,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    seq.write_var_int(&VarInt::from(stack.item.id))?;
    seq.write_var_int(&VarInt::from(stack.item_count))?;

    let mut to_add = 0u8;
    let mut to_remove = 0u8;
    for (_id, data) in &stack.patch {
        if data.is_none() {
            to_remove += 1;
        } else {
            to_add += 1;
        }
    }

    seq.write_var_int(&VarInt::from(to_add))?;
    seq.write_var_int(&VarInt::from(to_remove))?;

    for (id, data) in &stack.patch {
        if let Some(data) = data {
            seq.write_var_int(&VarInt::from(id.to_id()))?;
            serialize(*id, data.as_ref(), seq)?;
        }
    }

    for (id, data) in &stack.patch {
        if data.is_none() {
            seq.write_var_int(&VarInt::from(id.to_id()))?;
        }
    }

    Ok(())
}

impl DataComponentCodec<Self> for BundleContentsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.items.len() as i32))?;
        for item in &self.items {
            serialize_item_stack_template(item, seq)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_BUNDLE_ITEMS: usize = 64;

        let len = seq.get_var_int()?.0 as usize;

        if len > MAX_BUNDLE_ITEMS {
            return Err(ReadingError::Message(
                "Too many items in BundleContents".into(),
            ));
        }

        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(deserialize_item_stack_template(seq)?);
        }
        Ok(Self { items })
    }
}

macro_rules! codec_string_variant {
    ($struct_name:ident) => {
        impl DataComponentCodec<Self> for $struct_name {
            fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
                seq.write_string(&self.value)
            }
            fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
                let value = seq.get_str()?;
                Ok(Self {
                    value: Cow::Owned(value.into()),
                })
            }
        }
    };
}

codec_string_variant!(VillagerVariantImpl);
codec_string_variant!(WolfVariantImpl);
codec_string_variant!(WolfSoundVariantImpl);
codec_string_variant!(WolfCollarImpl);
codec_string_variant!(FoxVariantImpl);
codec_string_variant!(SalmonSizeImpl);
codec_string_variant!(ParrotVariantImpl);
codec_string_variant!(TropicalFishPatternImpl);
codec_string_variant!(TropicalFishBaseColorImpl);
codec_string_variant!(TropicalFishPatternColorImpl);
codec_string_variant!(MooshroomVariantImpl);
codec_string_variant!(RabbitVariantImpl);
codec_string_variant!(PigVariantImpl);
codec_string_variant!(PigSoundVariantImpl);
codec_string_variant!(CowVariantImpl);
codec_string_variant!(CowSoundVariantImpl);
codec_string_variant!(ChickenVariantImpl);
codec_string_variant!(ChickenSoundVariantImpl);
codec_string_variant!(ZombieNautilusVariantImpl);
codec_string_variant!(FrogVariantImpl);
codec_string_variant!(HorseVariantImpl);
codec_string_variant!(PaintingVariantImpl);
codec_string_variant!(LlamaVariantImpl);
codec_string_variant!(AxolotlVariantImpl);
codec_string_variant!(CatVariantImpl);
codec_string_variant!(CatSoundVariantImpl);
codec_string_variant!(CatCollarImpl);
codec_string_variant!(SheepColorImpl);
codec_string_variant!(ShulkerColorImpl);

impl DataComponentCodec<Self> for MaxDamageImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.max_damage))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let max_damage = seq.get_var_int()?.0;
        Ok(Self { max_damage })
    }
}

impl DataComponentCodec<Self> for UseEffectsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_bool(false)?;
        seq.write_bool(true)?;
        seq.write_f32(0.2)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _can_sprint = seq.get_bool()?;
        let _interact_vibrations = seq.get_bool()?;
        let _speed_multiplier = seq.get_f32()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for MinimumAttackChargeImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(self.charge)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let charge = seq.get_f32()?;
        Ok(Self { charge })
    }
}

impl DataComponentCodec<Self> for DamageTypeImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.damage_type.id as i32))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0 as u8;
        let damage_type = pumpkin_data::damage::DamageType::from_id(id)
            .ok_or_else(|| ReadingError::Message(format!("Invalid DamageType id {id}")))?;
        Ok(Self { damage_type })
    }
}

impl DataComponentCodec<Self> for CanPlaceOnImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let count = seq.get_var_int()?.0;
        for _ in 0..count {
            let has_blocks = seq.get_bool()?;
            if has_blocks {
                let id_type = seq.get_var_int()?.0;
                if id_type == 0 {
                    let _ = seq.get_str()?;
                } else if id_type > 0 {
                    for _ in 0..(id_type - 1) {
                        let _ = seq.get_var_int()?;
                    }
                }
            }
            let has_props = seq.get_bool()?;
            if has_props {
                let props_len = seq.get_var_int()?.0;
                for _ in 0..props_len {
                    let _ = seq.get_str()?;
                    let is_exact = seq.get_bool()?;
                    if is_exact {
                        let _ = seq.get_str()?;
                    } else {
                        if seq.get_bool()? {
                            let _ = seq.get_str()?;
                        }
                        if seq.get_bool()? {
                            let _ = seq.get_str()?;
                        }
                    }
                }
            }
            let has_nbt = seq.get_bool()?;
            if has_nbt {
                let _ = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            }
            let exact_len = seq.get_var_int()?.0;
            for _ in 0..exact_len {
                let comp_id = seq.get_var_int()?.0 as u8;
                if let Some(comp) = DataComponent::try_from_id(comp_id) {
                    let _ = deserialize(comp, seq)?;
                }
            }
            let partial_len = seq.get_var_int()?.0;
            for _ in 0..partial_len {
                let _ = seq.get_var_int()?;
            }
        }
        Ok(Self {
            predicate: NbtTag::List(Vec::new()),
        })
    }
}

impl DataComponentCodec<Self> for CanBreakImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let count = seq.get_var_int()?.0;
        for _ in 0..count {
            let has_blocks = seq.get_bool()?;
            if has_blocks {
                let id_type = seq.get_var_int()?.0;
                if id_type == 0 {
                    let _ = seq.get_str()?;
                } else if id_type > 0 {
                    for _ in 0..(id_type - 1) {
                        let _ = seq.get_var_int()?;
                    }
                }
            }
            let has_props = seq.get_bool()?;
            if has_props {
                let props_len = seq.get_var_int()?.0;
                for _ in 0..props_len {
                    let _ = seq.get_str()?;
                    let is_exact = seq.get_bool()?;
                    if is_exact {
                        let _ = seq.get_str()?;
                    } else {
                        if seq.get_bool()? {
                            let _ = seq.get_str()?;
                        }
                        if seq.get_bool()? {
                            let _ = seq.get_str()?;
                        }
                    }
                }
            }
            let has_nbt = seq.get_bool()?;
            if has_nbt {
                let _ = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            }
            let exact_len = seq.get_var_int()?.0;
            for _ in 0..exact_len {
                let comp_id = seq.get_var_int()?.0 as u8;
                if let Some(comp) = DataComponent::try_from_id(comp_id) {
                    let _ = deserialize(comp, seq)?;
                }
            }
            let partial_len = seq.get_var_int()?.0;
            for _ in 0..partial_len {
                let _ = seq.get_var_int()?;
            }
        }
        Ok(Self {
            predicate: NbtTag::List(Vec::new()),
        })
    }
}

impl DataComponentCodec<Self> for AttributeModifiersImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.attribute_modifiers.len() as i32))?;
        for modifier in self.attribute_modifiers.iter() {
            seq.write_var_int(&VarInt::from(modifier.r#type.id as i32))?;
            seq.write_string(modifier.id)?;
            seq.write_f64(modifier.amount)?;
            seq.write_var_int(&VarInt::from(modifier.operation as i32))?;
            let slot_id = match modifier.slot {
                pumpkin_data::enchantment::AttributeModifierSlot::Any => 0,
                pumpkin_data::enchantment::AttributeModifierSlot::MainHand => 1,
                pumpkin_data::enchantment::AttributeModifierSlot::OffHand => 2,
                pumpkin_data::enchantment::AttributeModifierSlot::Hand => 3,
                pumpkin_data::enchantment::AttributeModifierSlot::Feet => 4,
                pumpkin_data::enchantment::AttributeModifierSlot::Legs => 5,
                pumpkin_data::enchantment::AttributeModifierSlot::Chest => 6,
                pumpkin_data::enchantment::AttributeModifierSlot::Head => 7,
                pumpkin_data::enchantment::AttributeModifierSlot::Armor => 8,
                pumpkin_data::enchantment::AttributeModifierSlot::Body => 9,
                pumpkin_data::enchantment::AttributeModifierSlot::Saddle => 10,
            };
            seq.write_var_int(&VarInt(slot_id))?;
            seq.write_var_int(&VarInt(0))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        for _ in 0..len {
            let _attr_id = seq.get_var_int()?;
            let _id = seq.get_str()?;
            let _amount = seq.get_f64()?;
            let _operation = seq.get_var_int()?;
            let _slot = seq.get_var_int()?;
            let display_type = seq.get_var_int()?.0;
            if display_type == 2 {
                let _ = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            }
        }
        Ok(Self {
            attribute_modifiers: Cow::Borrowed(&[]),
        })
    }
}

impl DataComponentCodec<Self> for CustomModelDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.floats.len() as i32))?;
        for f in &self.floats {
            seq.write_f32(*f)?;
        }
        seq.write_var_int(&VarInt::from(self.flags.len() as i32))?;
        for b in &self.flags {
            seq.write_bool(*b)?;
        }
        seq.write_var_int(&VarInt::from(self.strings.len() as i32))?;
        for s in &self.strings {
            seq.write_string(s)?;
        }
        seq.write_var_int(&VarInt::from(self.colors.len() as i32))?;
        for c in &self.colors {
            seq.write_i32(*c)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let floats_len = seq.get_var_int()?.0 as usize;
        let mut floats = Vec::with_capacity(floats_len);
        for _ in 0..floats_len {
            floats.push(seq.get_f32()?);
        }
        let flags_len = seq.get_var_int()?.0 as usize;
        let mut flags = Vec::with_capacity(flags_len);
        for _ in 0..flags_len {
            flags.push(seq.get_bool()?);
        }
        let strings_len = seq.get_var_int()?.0 as usize;
        let mut strings = Vec::with_capacity(strings_len);
        for _ in 0..strings_len {
            strings.push(seq.get_str()?.to_string());
        }
        let colors_len = seq.get_var_int()?.0 as usize;
        let mut colors = Vec::with_capacity(colors_len);
        for _ in 0..colors_len {
            colors.push(seq.get_i32()?);
        }
        Ok(Self {
            floats,
            flags,
            strings,
            colors,
        })
    }
}

impl DataComponentCodec<Self> for TooltipDisplayImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_bool(false)?;
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _hide_tooltip = seq.get_bool()?;
        let len = seq.get_var_int()?.0 as usize;
        for _ in 0..len {
            let _comp_id = seq.get_var_int()?;
        }
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for CreativeSlotLockImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for EnchantmentGlintOverrideImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_bool(true)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_bool()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for IntangibleProjectileImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for FoodImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.nutrition))?;
        seq.write_f32(self.saturation)?;
        seq.write_bool(self.can_always_eat)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let nutrition = seq.get_var_int()?.0;
        let saturation = seq.get_f32()?;
        let can_always_eat = seq.get_bool()?;
        Ok(Self {
            nutrition,
            saturation,
            can_always_eat,
        })
    }
}

impl DataComponentCodec<Self> for UseRemainderImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = deserialize_item_stack_template(seq)?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for DamageResistantImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_string(self.res_type.as_str())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let tag = seq.get_str()?;
        Ok(Self {
            res_type: DamageResistantType::from_tag(&tag),
        })
    }
}

impl DataComponentCodec<Self> for ToolImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.rules.len() as i32))?;
        for rule in self.rules.iter() {
            serialize_idset(&rule.blocks, seq)?;
            seq.write_bool(rule.speed.is_some())?;
            if let Some(speed) = rule.speed {
                seq.write_f32(speed)?;
            }
            seq.write_bool(rule.correct_for_drops.is_some())?;
            if let Some(correct) = rule.correct_for_drops {
                seq.write_bool(correct)?;
            }
        }
        seq.write_f32(self.default_mining_speed)?;
        seq.write_var_int(&VarInt::from(self.damage_per_block as i32))?;
        seq.write_bool(self.can_destroy_blocks_in_creative)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let rules_len = seq.get_var_int()?.0 as usize;
        let mut rules = Vec::with_capacity(rules_len);
        for _ in 0..rules_len {
            let blocks = deserialize_idset(seq)?;
            let speed = if seq.get_bool()? {
                Some(seq.get_f32()?)
            } else {
                None
            };
            let correct_for_drops = if seq.get_bool()? {
                Some(seq.get_bool()?)
            } else {
                None
            };
            rules.push(pumpkin_data::data_component_impl::ToolRule {
                blocks,
                speed,
                correct_for_drops,
            });
        }
        let default_mining_speed = seq.get_f32()?;
        let damage_per_block = seq.get_var_int()?.0 as u32;
        let can_destroy_blocks_in_creative = seq.get_bool()?;
        Ok(Self {
            rules: Cow::Owned(rules),
            default_mining_speed,
            damage_per_block,
            can_destroy_blocks_in_creative,
        })
    }
}

impl DataComponentCodec<Self> for WeaponImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.item_damage_per_attack as i32))?;
        seq.write_f32(0.0)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let item_damage_per_attack = seq.get_var_int()?.0 as u32;
        let _disable_blocking_for_seconds = seq.get_f32()?;
        Ok(Self {
            item_damage_per_attack,
        })
    }
}

impl DataComponentCodec<Self> for AttackRangeImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(self.min_reach)?;
        seq.write_f32(self.max_reach)?;
        seq.write_f32(self.min_creative_reach)?;
        seq.write_f32(self.max_creative_reach)?;
        seq.write_f32(self.hitbox_margin)?;
        seq.write_f32(self.mob_factor)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let min_reach = seq.get_f32()?;
        let max_reach = seq.get_f32()?;
        let min_creative_reach = seq.get_f32()?;
        let max_creative_reach = seq.get_f32()?;
        let hitbox_margin = seq.get_f32()?;
        let mob_factor = seq.get_f32()?;
        Ok(Self {
            min_reach,
            max_reach,
            min_creative_reach,
            max_creative_reach,
            hitbox_margin,
            mob_factor,
        })
    }
}

impl DataComponentCodec<Self> for EnchantableImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.value))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let value = seq.get_var_int()?.0;
        Ok(Self { value })
    }
}

impl DataComponentCodec<Self> for GliderImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for TooltipStyleImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_string(&self.id)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_str()?.to_string();
        Ok(Self { id })
    }
}

impl DataComponentCodec<Self> for DeathProtectionImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        for _ in 0..len {
            let _ = deserialize_consume_effect(seq)?;
        }
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for BlocksAttacksImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(0.0)?;
        seq.write_f32(1.0)?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_bool(false)?;
        seq.write_bool(false)?;
        seq.write_bool(false)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _block_delay = seq.get_f32()?;
        let _disable_scale = seq.get_f32()?;
        let red_len = seq.get_var_int()?.0 as usize;
        for _ in 0..red_len {
            let _ = seq.get_f32()?;
            if seq.get_bool()? {
                let id_type = seq.get_var_int()?.0;
                if id_type == 0 {
                    let _ = seq.get_str()?;
                } else if id_type > 0 {
                    for _ in 0..(id_type - 1) {
                        let _ = seq.get_var_int()?;
                    }
                }
            }
            let _ = seq.get_f32()?;
            let _ = seq.get_f32()?;
        }
        let item_damage_type = seq.get_var_int()?.0;
        if item_damage_type == 1 {
            let _ = seq.get_f32()?;
            let _ = seq.get_f32()?;
        }
        if seq.get_bool()? {
            let id_type = seq.get_var_int()?.0;
            if id_type == 0 {
                let _ = seq.get_str()?;
            } else if id_type > 0 {
                for _ in 0..(id_type - 1) {
                    let _ = seq.get_var_int()?;
                }
            }
        }
        if seq.get_bool()? {
            let _ = seq.get_var_int()?;
        }
        if seq.get_bool()? {
            let _ = seq.get_var_int()?;
        }
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for PiercingWeaponImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_bool(self.deals_knockback)?;
        seq.write_bool(self.dismounts)?;
        if let Some(sound) = &self.sound {
            seq.write_bool(true)?;
            let proto_sound = data_to_proto_sound(sound);
            crate::IdOr::<crate::SoundEvent>::write(&proto_sound, seq, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            })?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(sound) = &self.hit_sound {
            seq.write_bool(true)?;
            let proto_sound = data_to_proto_sound(sound);
            crate::IdOr::<crate::SoundEvent>::write(&proto_sound, seq, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            })?;
        } else {
            seq.write_bool(false)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let deals_knockback = seq.get_bool()?;
        let dismounts = seq.get_bool()?;
        let sound = if seq.get_bool()? {
            let proto = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.to_string();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })
            .map_err(|e| ReadingError::Message(format!("No sound: {e}")))?;
            proto_to_data_sound(&proto)
        } else {
            None
        };
        let hit_sound = if seq.get_bool()? {
            let proto = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.to_string();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })
            .map_err(|e| ReadingError::Message(format!("No sound: {e}")))?;
            proto_to_data_sound(&proto)
        } else {
            None
        };
        Ok(Self {
            deals_knockback,
            dismounts,
            sound,
            hit_sound,
        })
    }
}

fn serialize_kinetic_condition(
    cond: &pumpkin_data::data_component_impl::KineticConditionImpl,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    seq.write_var_int(&VarInt::from(cond.max_duration_ticks))?;
    seq.write_f32(cond.min_speed)?;
    seq.write_f32(cond.min_relative_speed)
}

fn deserialize_kinetic_condition(
    seq: &mut impl NetworkReadExt,
) -> Result<pumpkin_data::data_component_impl::KineticConditionImpl, ReadingError> {
    let max_duration_ticks = seq.get_var_int()?.0;
    let min_speed = seq.get_f32()?;
    let min_relative_speed = seq.get_f32()?;
    Ok(pumpkin_data::data_component_impl::KineticConditionImpl {
        max_duration_ticks,
        min_speed,
        min_relative_speed,
    })
}

impl DataComponentCodec<Self> for KineticWeaponImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.contact_cooldown_ticks))?;
        seq.write_var_int(&VarInt::from(self.delay_ticks))?;
        if let Some(cond) = &self.dismount_conditions {
            seq.write_bool(true)?;
            serialize_kinetic_condition(cond, seq)?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(cond) = &self.knockback_conditions {
            seq.write_bool(true)?;
            serialize_kinetic_condition(cond, seq)?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(cond) = &self.damage_conditions {
            seq.write_bool(true)?;
            serialize_kinetic_condition(cond, seq)?;
        } else {
            seq.write_bool(false)?;
        }
        seq.write_f32(self.forward_movement)?;
        seq.write_f32(self.damage_multiplier)?;
        if let Some(sound) = &self.sound {
            seq.write_bool(true)?;
            let proto = data_to_proto_sound(sound);
            crate::IdOr::<crate::SoundEvent>::write(&proto, seq, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            })?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(sound) = &self.hit_sound {
            seq.write_bool(true)?;
            let proto = data_to_proto_sound(sound);
            crate::IdOr::<crate::SoundEvent>::write(&proto, seq, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            })?;
        } else {
            seq.write_bool(false)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let contact_cooldown_ticks = seq.get_var_int()?.0;
        let delay_ticks = seq.get_var_int()?.0;
        let dismount_conditions = if seq.get_bool()? {
            Some(deserialize_kinetic_condition(seq)?)
        } else {
            None
        };
        let knockback_conditions = if seq.get_bool()? {
            Some(deserialize_kinetic_condition(seq)?)
        } else {
            None
        };
        let damage_conditions = if seq.get_bool()? {
            Some(deserialize_kinetic_condition(seq)?)
        } else {
            None
        };
        let forward_movement = seq.get_f32()?;
        let damage_multiplier = seq.get_f32()?;
        let sound = if seq.get_bool()? {
            let proto = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.to_string();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })
            .map_err(|e| ReadingError::Message(format!("No sound: {e}")))?;
            proto_to_data_sound(&proto)
        } else {
            None
        };
        let hit_sound = if seq.get_bool()? {
            let proto = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.to_string();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })
            .map_err(|e| ReadingError::Message(format!("No sound: {e}")))?;
            proto_to_data_sound(&proto)
        } else {
            None
        };
        Ok(Self {
            contact_cooldown_ticks,
            delay_ticks,
            dismount_conditions,
            knockback_conditions,
            damage_conditions,
            forward_movement,
            damage_multiplier,
            sound,
            hit_sound,
        })
    }
}

impl DataComponentCodec<Self> for AdditionalTradeCostImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_var_int()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for DyeImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_var_int()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for MapColorImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_i32(0)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_i32()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for MapDecorationsImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for MapPostProcessingImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.processing.map_or(0, |p| p as i32)))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let val = seq.get_var_int()?.0;
        let processing = match val {
            0 => Some(pumpkin_data::data_component_impl::MapPostProcessing::Lock),
            1 => Some(pumpkin_data::data_component_impl::MapPostProcessing::Scale),
            _ => None,
        };
        Ok(Self { processing })
    }
}

impl DataComponentCodec<Self> for ChargedProjectilesImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.projectiles.len() as i32))?;
        for _ in &self.projectiles {
            seq.write_var_int(&VarInt(0))?;
            seq.write_var_int(&VarInt(0))?;
            seq.write_var_int(&VarInt(0))?;
            seq.write_var_int(&VarInt(0))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        let mut projectiles = Vec::with_capacity(len);
        for _ in 0..len {
            let _ = deserialize_item_stack_template(seq)?;
            projectiles.push(pumpkin_nbt::compound::NbtCompound::new());
        }
        Ok(Self { projectiles })
    }
}

impl DataComponentCodec<Self> for PotionDurationScaleImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(self.scale)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let scale = seq.get_f32()?;
        Ok(Self { scale })
    }
}

impl DataComponentCodec<Self> for WritableBookContentImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.pages.len() as i32))?;
        for page in &self.pages {
            seq.write_string(page)?;
            seq.write_bool(false)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        let mut pages = Vec::with_capacity(len);
        for _ in 0..len {
            let raw = seq.get_str()?.to_string();
            let has_filtered = seq.get_bool()?;
            if has_filtered {
                let _ = seq.get_str()?;
            }
            pages.push(raw);
        }
        Ok(Self { pages })
    }
}

impl DataComponentCodec<Self> for WrittenBookContentImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_string(&self.title)?;
        seq.write_bool(false)?;
        seq.write_string(&self.author)?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt::from(self.pages.len() as i32))?;
        for page in &self.pages {
            let comp = pumpkin_util::text::TextComponent::text(page.clone());
            seq.write_slice(&comp.encode_for_version(&JavaMinecraftVersion::V_26_2))?;
            seq.write_bool(false)?;
        }
        seq.write_bool(true)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let title = seq.get_str()?.to_string();
        if seq.get_bool()? {
            let _ = seq.get_str()?;
        }
        let author = seq.get_str()?.to_string();
        let _generation = seq.get_var_int()?.0;
        let pages_len = seq.get_var_int()?.0 as usize;
        let mut pages = Vec::with_capacity(pages_len);
        for _ in 0..pages_len {
            let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            let comp = tag.as_ref().map_or_else(
                pumpkin_util::text::TextComponent::empty,
                pumpkin_util::text::TextComponent::from_nbt,
            );
            if seq.get_bool()? {
                let _ = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            }
            pages.push(comp.get_text());
        }
        let _resolved = seq.get_bool()?;
        Ok(Self {
            title,
            author,
            pages,
        })
    }
}

impl DataComponentCodec<Self> for TrimImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _material = seq.get_var_int()?;
        let _pattern = seq.get_var_int()?;
        Ok(Self {
            material: NbtTag::String("minecraft:quartz".into()),
            pattern: NbtTag::String("minecraft:coast".into()),
        })
    }
}

impl DataComponentCodec<Self> for DebugStickStateImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for EntityDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))?;
        seq.write_nbt(NbtTag::Compound(pumpkin_nbt::compound::NbtCompound::new()))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _type_id = seq.get_var_int()?;
        let _nbt = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for BucketEntityDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_nbt(NbtTag::Compound(pumpkin_nbt::compound::NbtCompound::new()))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _nbt = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for BlockEntityDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))?;
        seq.write_nbt(NbtTag::Compound(self.nbt.clone()))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _type_id = seq.get_var_int()?;
        let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
        let nbt = if let Some(NbtTag::Compound(c)) = tag {
            c
        } else {
            pumpkin_nbt::compound::NbtCompound::new()
        };
        Ok(Self { nbt })
    }
}

impl DataComponentCodec<Self> for InstrumentImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_var_int()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for ProvidesTrimMaterialImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_var_int()?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for OminousBottleAmplifierImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.amplifier))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let amplifier = seq.get_var_int()?.0;
        Ok(Self { amplifier })
    }
}

impl DataComponentCodec<Self> for JukeboxPlayableImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let song_id = Sound::from_name(self.song).map_or(0, |s| s as i32);
        seq.write_var_int(&VarInt::from(song_id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_var_int()?;
        Ok(Self { song: "" })
    }
}

impl DataComponentCodec<Self> for ProvidesBannerPatternsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id_type = seq.get_var_int()?.0;
        if id_type == 0 {
            let _ = seq.get_str()?;
        } else if id_type > 0 {
            for _ in 0..(id_type - 1) {
                let _ = seq.get_var_int()?;
            }
        }
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for RecipesImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for LodestoneTrackerImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        if let Some(target) = &self.target {
            seq.write_bool(true)?;
            seq.write_string(&target.dimension)?;
            let pos = pumpkin_util::math::position::BlockPos::new(target.x, target.y, target.z);
            seq.write_block_pos(&pos, &JavaMinecraftVersion::V_26_2)?;
        } else {
            seq.write_bool(false)?;
        }
        seq.write_bool(self.tracked)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let target = if seq.get_bool()? {
            let dimension = seq.get_str()?.to_string();
            let pos = seq.get_block_pos(&JavaMinecraftVersion::V_26_2)?;
            Some(pumpkin_data::data_component_impl::LodestoneTarget {
                dimension,
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            })
        } else {
            None
        };
        let tracked = seq.get_bool()?;
        Ok(Self { target, tracked })
    }
}

impl DataComponentCodec<Self> for ProfileImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(1))?;
        if let Some(name) = &self.name {
            seq.write_bool(true)?;
            seq.write_string(name)?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(id) = &self.id {
            seq.write_bool(true)?;
            let uuid = uuid::Uuid::from_u128(
                ((id[0] as u128) << 96)
                    | ((id[1] as u128 & 0xFFFFFFFF) << 64)
                    | ((id[2] as u128 & 0xFFFFFFFF) << 32)
                    | (id[3] as u128 & 0xFFFFFFFF),
            );
            seq.write_uuid(&uuid)?;
        } else {
            seq.write_bool(false)?;
        }
        seq.write_var_int(&VarInt::from(self.properties.len() as i32))?;
        for prop in &self.properties {
            seq.write_string(&prop.name)?;
            seq.write_string(&prop.value)?;
            if let Some(sig) = &prop.signature {
                seq.write_bool(true)?;
                seq.write_string(sig)?;
            } else {
                seq.write_bool(false)?;
            }
        }
        if let Some(texture) = &self.texture {
            seq.write_bool(true)?;
            seq.write_string(texture)?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(cape) = &self.cape {
            seq.write_bool(true)?;
            seq.write_string(cape)?;
        } else {
            seq.write_bool(false)?;
        }
        if let Some(elytra) = &self.elytra {
            seq.write_bool(true)?;
            seq.write_string(elytra)?;
        } else {
            seq.write_bool(false)?;
        }
        if self.model.is_some() {
            seq.write_bool(true)?;
            seq.write_var_int(&VarInt(0))?;
        } else {
            seq.write_bool(false)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let either = seq.get_var_int()?.0;
        let mut name = None;
        let mut id = None;
        let mut properties = Vec::new();
        if either == 0 {
            let uuid = seq.get_uuid()?;
            let u = uuid.as_u128();
            id = Some([
                (u >> 96) as i32,
                (u >> 64) as i32,
                (u >> 32) as i32,
                u as i32,
            ]);
            name = Some(seq.get_str()?.to_string());
        } else {
            if seq.get_bool()? {
                name = Some(seq.get_str()?.to_string());
            }
            if seq.get_bool()? {
                let uuid = seq.get_uuid()?;
                let u = uuid.as_u128();
                id = Some([
                    (u >> 96) as i32,
                    (u >> 64) as i32,
                    (u >> 32) as i32,
                    u as i32,
                ]);
            }
        }
        let props_len = seq.get_var_int()?.0 as usize;
        for _ in 0..props_len {
            let prop_name = seq.get_str()?.to_string();
            let prop_value = seq.get_str()?.to_string();
            let sig = if seq.get_bool()? {
                Some(seq.get_str()?.to_string())
            } else {
                None
            };
            properties.push(pumpkin_data::data_component_impl::ProfileProperty {
                name: prop_name,
                value: prop_value,
                signature: sig,
            });
        }
        let texture = if seq.get_bool()? {
            Some(seq.get_str()?.to_string())
        } else {
            None
        };
        let cape = if seq.get_bool()? {
            Some(seq.get_str()?.to_string())
        } else {
            None
        };
        let elytra = if seq.get_bool()? {
            Some(seq.get_str()?.to_string())
        } else {
            None
        };
        let model = if seq.get_bool()? {
            let _ = seq.get_var_int()?;
            Some("wide".to_string())
        } else {
            None
        };
        Ok(Self {
            name,
            id,
            properties,
            texture,
            cape,
            elytra,
            model,
        })
    }
}

impl DataComponentCodec<Self> for NoteBlockSoundImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_string(&self.sound)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let sound = seq.get_str()?.to_string();
        Ok(Self { sound })
    }
}

impl DataComponentCodec<Self> for BannerPatternsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.layers.len() as i32))?;
        for layer in &self.layers {
            seq.write_var_int(&VarInt(0))?;
            seq.write_var_int(&VarInt::from(layer.color.id() as i32))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        let mut layers = Vec::with_capacity(len);
        for _ in 0..len {
            let _pattern = seq.get_var_int()?.0;
            let color_id = seq.get_var_int()?.0 as u8;
            let color = pumpkin_data::dye_color::DyeColor::by_id(color_id).unwrap_or_default();
            layers.push(pumpkin_data::data_component_impl::BannerPatternLayer {
                pattern: String::new(),
                color,
            });
        }
        Ok(Self { layers })
    }
}

impl DataComponentCodec<Self> for BaseColorImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let color_id =
            pumpkin_data::dye_color::DyeColor::by_name(&self.color).map_or(0, |c| c.id() as i32);
        seq.write_var_int(&VarInt::from(color_id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0 as u8;
        let color = pumpkin_data::dye_color::DyeColor::by_id(id)
            .map_or("white", |c| c.name())
            .to_string();
        Ok(Self { color })
    }
}

impl DataComponentCodec<Self> for PotDecorationsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        for _ in 0..len {
            let _ = seq.get_var_int()?;
        }
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for ContainerImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.items.len() as i32))?;
        for (_slot, stack) in &self.items {
            seq.write_bool(true)?;
            serialize_item_stack_template(stack, seq)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        let mut items = Vec::with_capacity(len);
        for slot in 0..len {
            if seq.get_bool()? {
                let stack = deserialize_item_stack_template(seq)?;
                items.push((slot as u8, stack));
            }
        }
        Ok(Self { items })
    }
}

impl DataComponentCodec<Self> for BlockStateImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.properties.len() as i32))?;
        for (k, v) in self.properties.iter() {
            seq.write_string(k)?;
            seq.write_string(v)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        let mut properties = Vec::with_capacity(len);
        for _ in 0..len {
            let k = seq.get_str()?.to_string();
            let v = seq.get_str()?.to_string();
            properties.push((Cow::Owned(k), Cow::Owned(v)));
        }
        Ok(Self {
            properties: Cow::Owned(properties),
        })
    }
}

impl DataComponentCodec<Self> for BeesImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        for _ in 0..len {
            let _entity_type = seq.get_var_int()?;
            let _nbt = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            let _ticks = seq.get_var_int()?;
            let _min_ticks = seq.get_var_int()?;
        }
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for SulfurCubeContentImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))?;
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = deserialize_item_stack_template(seq)?;
        Ok(Self)
    }
}

impl DataComponentCodec<Self> for LockImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            predicate: pumpkin_nbt::compound::NbtCompound::new(),
        })
    }
}

impl DataComponentCodec<Self> for ContainerLootImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            loot_table: String::new(),
            seed: 0,
        })
    }
}

impl DataComponentCodec<Self> for BreakSoundImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt(0))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let _ = seq.get_var_int()?;
        Ok(Self)
    }
}

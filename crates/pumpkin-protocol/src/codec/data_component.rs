#![allow(clippy::wildcard_imports)]

use std::{borrow::Cow, cell::Cell};

use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::*;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_data::trim_material::TrimMaterial;
use pumpkin_data::trim_pattern::TrimPattern;
use pumpkin_nbt::{compound::NbtCompound, serializer::NbtWriteHelperJava, tag::NbtTag};
use pumpkin_util::version::JavaMinecraftVersion;

const MAX_STATUS_EFFECTS: usize = 128;
const MAX_INITIAL_COLLECTION_SIZE: usize = 65536;

fn read_count(seq: &mut impl NetworkReadExt, max_size: usize) -> Result<usize, ReadingError> {
    let count = seq.get_var_int()?.0;
    usize::try_from(count)
        .ok()
        .filter(|&count| count <= max_size)
        .ok_or_else(|| {
            ReadingError::TooLarge(format!("{count} elements exceeded max size of {max_size}"))
        })
}

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
            let mut content_vec =
                Vec::with_capacity((len as usize).min(MAX_INITIAL_COLLECTION_SIZE));

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
            let effects = Cow::Owned(deserialize_status_effects(seq)?);
            let probability = seq.get_f32()?;
            Ok(ConsumeEffect::ApplyEffects((effects, probability)))
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
            if !(1..=MAX_ENCHANTMENT_LEVEL).contains(&level) {
                return Err(ReadingError::Message(format!(
                    "Enchantment level {level} out of range"
                )));
            }
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
        let tag = seq.get_nbt_with_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2)?;
        let name = match tag {
            Some(NbtTag::Compound(c)) => c
                .get_string("translate")
                .or_else(|| c.get_string("text"))
                .unwrap_or("")
                .to_string(),
            Some(NbtTag::String(s)) => s.to_string(),
            Some(t) => pumpkin_util::text::TextComponent::from_nbt(&t).get_text(),
            None => String::new(),
        };
        Ok(Self {
            name: Cow::Owned(name),
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
        const MAX_CONSUME_EFFECTS: i32 = 256;

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
        if !(0..=MAX_CONSUME_EFFECTS).contains(&effects_len) {
            return Err(ReadingError::Message("Invalid consume effect count".into()));
        }

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
            if !(1..=MAX_ENCHANTMENT_LEVEL).contains(&level) {
                return Err(ReadingError::Message(format!(
                    "Enchantment level {level} out of range"
                )));
            }
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

pub(crate) fn write_swing_animation(
    seq: &mut impl NetworkWriteExt,
    animation: SwingAnimationType,
    duration: i32,
) -> Result<(), WritingError> {
    // These are Java wire IDs. The numeric NBT form keeps its existing mapping.
    let animation_id = match animation {
        SwingAnimationType::None => 0,
        SwingAnimationType::Whack => 1,
        SwingAnimationType::Stab => 2,
    };
    seq.write_var_int(&VarInt(animation_id))?;
    seq.write_var_int(&VarInt(duration))
}

impl DataComponentCodec<Self> for SwingAnimationImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        write_swing_animation(seq, self.animation_type, self.duration)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        // Java's ID mapper uses the zero entry for unrecognized IDs.
        let animation_type = match seq.get_var_int()?.0 {
            1 => SwingAnimationType::Whack,
            2 => SwingAnimationType::Stab,
            _ => SwingAnimationType::None,
        };
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
        DataComponent::AttackAnimation => Ok(SwingAnimationImpl::deserialize(seq)?.to_dyn()),
        DataComponent::AdditionalTradeCost => {
            Ok(AdditionalTradeCostImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::StoredEnchantments => Ok(StoredEnchantmentsImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Dye => Ok(DyeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::DyedColor => Ok(DyedColorImpl::deserialize(seq)?.to_dyn()),
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
        DataComponent::InteractAnimation => Ok(InteractAnimationImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BlockTransformer => Ok(BlockTransformerImpl::deserialize(seq)?.to_dyn()),
        DataComponent::VillagerFood => Ok(VillagerFoodImpl::deserialize(seq)?.to_dyn()),
        DataComponent::MobVisibility => Ok(MobVisibilityImpl::deserialize(seq)?.to_dyn()),
        DataComponent::ProvidesPotteryPattern => {
            Ok(ProvidesPotteryPatternImpl::deserialize(seq)?.to_dyn())
        }
        DataComponent::SignTextFront => Ok(SignTextFrontImpl::deserialize(seq)?.to_dyn()),
        DataComponent::SignTextBack => Ok(SignTextBackImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CushionColor => Ok(CushionColorImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Compostable => Ok(CompostableImpl::deserialize(seq)?.to_dyn()),
        DataComponent::CookingFuel => Ok(CookingFuelImpl::deserialize(seq)?.to_dyn()),
        DataComponent::BrewingFuel => Ok(BrewingFuelImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Waxed => Ok(WaxedImpl::deserialize(seq)?.to_dyn()),
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
        DataComponent::AttackAnimation => get::<SwingAnimationImpl>(value).serialize(seq),
        DataComponent::AdditionalTradeCost => get::<AdditionalTradeCostImpl>(value).serialize(seq),
        DataComponent::StoredEnchantments => get::<StoredEnchantmentsImpl>(value).serialize(seq),
        DataComponent::Dye => get::<DyeImpl>(value).serialize(seq),
        DataComponent::DyedColor => get::<DyedColorImpl>(value).serialize(seq),
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
        DataComponent::InteractAnimation => get::<InteractAnimationImpl>(value).serialize(seq),
        DataComponent::BlockTransformer => get::<BlockTransformerImpl>(value).serialize(seq),
        DataComponent::VillagerFood => get::<VillagerFoodImpl>(value).serialize(seq),
        DataComponent::MobVisibility => get::<MobVisibilityImpl>(value).serialize(seq),
        DataComponent::ProvidesPotteryPattern => {
            get::<ProvidesPotteryPatternImpl>(value).serialize(seq)
        }
        DataComponent::SignTextFront => get::<SignTextFrontImpl>(value).serialize(seq),
        DataComponent::SignTextBack => get::<SignTextBackImpl>(value).serialize(seq),
        DataComponent::CushionColor => get::<CushionColorImpl>(value).serialize(seq),
        DataComponent::Compostable => get::<CompostableImpl>(value).serialize(seq),
        DataComponent::CookingFuel => get::<CookingFuelImpl>(value).serialize(seq),
        DataComponent::BrewingFuel => get::<BrewingFuelImpl>(value).serialize(seq),
        DataComponent::Waxed => get::<WaxedImpl>(value).serialize(seq),
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
    let item = pumpkin_data::item::Item::from_id(item_id)
        .ok_or_else(|| ReadingError::Message(format!("Unknown item ID: {item_id}")))?;
    if item == &pumpkin_data::item::Item::AIR {
        return Err(ReadingError::Message(
            "ItemStackTemplate item must be non-empty".into(),
        ));
    }

    let raw_count = seq.get_var_int()?.0;
    let Ok(count) = u8::try_from(raw_count) else {
        return Err(ReadingError::Message(
            "Invalid ItemStackTemplate count".into(),
        ));
    };
    if count == 0 {
        return Err(ReadingError::Message(
            "ItemStackTemplate count must be non-zero".into(),
        ));
    }

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
        count, item, patch,
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
        Ok(Self::new(items))
    }
}

fn unknown_variant(component: &str, value: &str) -> WritingError {
    WritingError::Message(format!("unknown {component} value {value}"))
}

/// Vanilla `ByteBufCodecs.holderRegistry`: the entry's id in the synced registry.
macro_rules! codec_registry_variant {
    ($struct_name:ident, $variant:ty) => {
        impl DataComponentCodec<Self> for $struct_name {
            fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
                let variant = <$variant>::from_name(&self.value)
                    .ok_or_else(|| unknown_variant(stringify!($struct_name), &self.value))?;
                let id = i32::try_from(variant.id())
                    .map_err(|_| unknown_variant(stringify!($struct_name), &self.value))?;
                seq.write_var_int(&VarInt(id))
            }
            fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
                let id = seq.get_var_int()?.0;
                let variant = usize::try_from(id)
                    .ok()
                    .and_then(|id| <$variant>::all().get(id))
                    .ok_or_else(|| {
                        ReadingError::Message(format!(
                            "unknown {} id {id}",
                            stringify!($struct_name)
                        ))
                    })?;
                Ok(Self {
                    value: Cow::Owned(format!("minecraft:{}", variant.to_name())),
                })
            }
        }
    };
}

codec_registry_variant!(WolfVariantImpl, pumpkin_data::wolf_variant::WolfVariant);
codec_registry_variant!(
    WolfSoundVariantImpl,
    pumpkin_data::wolf_sound_variant::WolfSoundVariant
);
codec_registry_variant!(PigVariantImpl, pumpkin_data::pig_variant::PigVariant);
codec_registry_variant!(
    PigSoundVariantImpl,
    pumpkin_data::pig_sound_variant::PigSoundVariant
);
codec_registry_variant!(CowVariantImpl, pumpkin_data::cow_variant::CowVariant);
codec_registry_variant!(
    CowSoundVariantImpl,
    pumpkin_data::cow_sound_variant::CowSoundVariant
);
codec_registry_variant!(
    ChickenVariantImpl,
    pumpkin_data::chicken_variant::ChickenVariant
);
codec_registry_variant!(
    ChickenSoundVariantImpl,
    pumpkin_data::chicken_sound_variant::ChickenSoundVariant
);
codec_registry_variant!(
    ZombieNautilusVariantImpl,
    pumpkin_data::zombie_nautilus_variant::ZombieNautilusVariant
);
codec_registry_variant!(FrogVariantImpl, pumpkin_data::frog_variant::FrogVariant);
codec_registry_variant!(CatVariantImpl, pumpkin_data::cat_variant::CatVariant);
codec_registry_variant!(
    CatSoundVariantImpl,
    pumpkin_data::cat_sound_variant::CatSoundVariant
);

impl DataComponentCodec<Self> for VillagerVariantImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let variant = pumpkin_data::villager::VillagerType::from_name(&self.value)
            .ok_or_else(|| unknown_variant("VillagerVariantImpl", &self.value))?;
        seq.write_var_int(&VarInt(variant as i32))
    }
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let variant = pumpkin_data::villager::VillagerType::from_i32(id)
            .ok_or_else(|| ReadingError::Message(format!("unknown VillagerVariantImpl id {id}")))?;
        Ok(Self {
            value: Cow::Owned(format!("minecraft:{}", variant.to_name())),
        })
    }
}

/// Vanilla `ByteBufCodecs.holder`: 0 for an inline value, otherwise the registry id plus one.
impl DataComponentCodec<Self> for PaintingVariantImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let variant = pumpkin_data::painting_variant::PaintingVariant::from_name(&self.value)
            .ok_or_else(|| unknown_variant("PaintingVariantImpl", &self.value))?;
        let id = i32::try_from(variant.id())
            .map_err(|_| unknown_variant("PaintingVariantImpl", &self.value))?;
        seq.write_var_int(&VarInt(id + 1))
    }
    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let variant = id
            .checked_sub(1)
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| pumpkin_data::painting_variant::PaintingVariant::all().get(id))
            .ok_or_else(|| {
                ReadingError::Message(format!("unsupported PaintingVariantImpl holder {id}"))
            })?;
        Ok(Self {
            value: Cow::Owned(format!("minecraft:{}", variant.to_name())),
        })
    }
}

/// Vanilla `ByteBufCodecs.idMapper` over a Java enum; unknown ids follow its `ByIdMap` strategy.
macro_rules! codec_id_mapped_variant {
    ($struct_name:ident, $variant:ty) => {
        impl DataComponentCodec<Self> for $struct_name {
            fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
                let variant = <$variant>::from_name(&self.value)
                    .ok_or_else(|| unknown_variant(stringify!($struct_name), &self.value))?;
                seq.write_var_int(&VarInt(variant.id()))
            }
            fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
                let id = seq.get_var_int()?.0;
                Ok(Self {
                    value: Cow::Borrowed(<$variant>::by_id(id).name()),
                })
            }
        }
    };
}

codec_id_mapped_variant!(FoxVariantImpl, pumpkin_data::entity_variant::FoxVariant);
codec_id_mapped_variant!(SalmonSizeImpl, pumpkin_data::entity_variant::SalmonSize);
codec_id_mapped_variant!(
    ParrotVariantImpl,
    pumpkin_data::entity_variant::ParrotVariant
);
codec_id_mapped_variant!(
    TropicalFishPatternImpl,
    pumpkin_data::entity_variant::TropicalFishPattern
);
codec_id_mapped_variant!(
    MooshroomVariantImpl,
    pumpkin_data::entity_variant::MooshroomVariant
);
codec_id_mapped_variant!(
    RabbitVariantImpl,
    pumpkin_data::entity_variant::RabbitVariant
);
codec_id_mapped_variant!(HorseVariantImpl, pumpkin_data::entity_variant::HorseVariant);
codec_id_mapped_variant!(LlamaVariantImpl, pumpkin_data::entity_variant::LlamaVariant);
codec_id_mapped_variant!(
    AxolotlVariantImpl,
    pumpkin_data::entity_variant::AxolotlVariant
);

/// Vanilla `DyeColor.STREAM_CODEC`: the color id, unknown ids read as white.
macro_rules! codec_dye_color {
    ($struct_name:ident) => {
        impl DataComponentCodec<Self> for $struct_name {
            fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
                let color = pumpkin_data::dye_color::DyeColor::by_name(&self.value)
                    .ok_or_else(|| unknown_variant(stringify!($struct_name), &self.value))?;
                seq.write_var_int(&VarInt(i32::from(color.id())))
            }
            fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
                let color = pumpkin_data::dye_color::DyeColor::by_id_or_white(seq.get_var_int()?.0);
                Ok(Self {
                    value: Cow::Borrowed(color.name()),
                })
            }
        }
    };
}

codec_dye_color!(WolfCollarImpl);
codec_dye_color!(TropicalFishBaseColorImpl);
codec_dye_color!(TropicalFishPatternColorImpl);
codec_dye_color!(CatCollarImpl);
codec_dye_color!(SheepColorImpl);
codec_dye_color!(ShulkerColorImpl);

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
        seq.write_bool(self.can_sprint)?;
        seq.write_bool(self.interact_vibrations)?;
        seq.write_f32(self.speed_multiplier)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let can_sprint = seq.get_bool()?;
        let interact_vibrations = seq.get_bool()?;
        let speed_multiplier = seq.get_f32()?;
        Ok(Self {
            can_sprint,
            interact_vibrations,
            speed_multiplier,
        })
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

fn write_adventure_state(
    state: &NbtTag,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    let compound = state
        .extract_compound()
        .ok_or_else(|| WritingError::Message("Expected an Adventure state compound".into()))?;
    let mut properties: Vec<_> = compound.child_tags.iter().collect();
    properties.sort_unstable_by_key(|(name, _)| *name);
    seq.write_list(&properties, |seq, (name, matcher)| {
        seq.write_string(name)?;
        match matcher {
            NbtTag::String(value) => {
                seq.write_bool(true)?;
                seq.write_string(value)
            }
            NbtTag::Compound(bounds) => {
                seq.write_bool(false)?;
                for bound in ["min", "max"] {
                    seq.write_option(&bounds.get(bound), |seq, tag| {
                        let value = tag.extract_string().ok_or_else(|| {
                            WritingError::Message(format!(
                                "Expected a string for Adventure state {bound}"
                            ))
                        })?;
                        seq.write_string(value)
                    })?;
                }
                Ok(())
            }
            _ => Err(WritingError::Message(format!(
                "Invalid Adventure state matcher for {name}"
            ))),
        }
    })
}

fn read_adventure_state(seq: &mut impl NetworkReadExt) -> Result<NbtCompound, ReadingError> {
    let count = read_count(seq, MAX_INITIAL_COLLECTION_SIZE)?;
    let mut state = NbtCompound::new();
    for _ in 0..count {
        let name = seq.get_str()?;
        let matcher = if seq.get_bool()? {
            NbtTag::String(seq.get_str()?)
        } else {
            let mut bounds = NbtCompound::new();
            for bound in ["min", "max"] {
                if let Some(value) = seq.get_option(NetworkReadExt::get_str)? {
                    bounds.put(bound, NbtTag::String(value));
                }
            }
            NbtTag::Compound(bounds)
        };
        if state.has(&name) {
            return Err(ReadingError::Message(format!(
                "Duplicate property {name} in Adventure state matcher"
            )));
        }
        state.child_tags.insert(name, matcher);
    }
    Ok(state)
}

// Native 26.3 built-in DATA_COMPONENT_PREDICATE_TYPE registration order.
// This registry is separate from DATA_COMPONENT_TYPE and has no Pumpkin data table.
const ADVENTURE_PREDICATE_TYPES: [&str; 15] = [
    "damage",
    "enchantments",
    "stored_enchantments",
    "potion_contents",
    "custom_data",
    "container",
    "bundle_contents",
    "firework_explosion",
    "fireworks",
    "writable_book_content",
    "written_book_content",
    "attribute_modifiers",
    "trim",
    "jukebox_playable",
    "villager/variant",
];
const MAX_ADVENTURE_PARTIAL_MATCHERS: usize = 64;
const MAX_ADVENTURE_PREDICATE_DEPTH: u8 = 32;

thread_local! {
    static ADVENTURE_PREDICATE_DEPTH: Cell<u8> = const { Cell::new(0) };
}

// Exact matches can nest through can_place_on, can_break or item stack components.
struct AdventurePredicateDepth;

impl AdventurePredicateDepth {
    fn enter() -> Option<Self> {
        ADVENTURE_PREDICATE_DEPTH.with(|depth| {
            let current = depth.get();
            if current >= MAX_ADVENTURE_PREDICATE_DEPTH {
                return None;
            }
            depth.set(current + 1);
            Some(Self)
        })
    }
}

impl Drop for AdventurePredicateDepth {
    fn drop(&mut self) {
        ADVENTURE_PREDICATE_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

fn adventure_matchers<'a>(
    predicate: &'a NbtCompound,
    field: &str,
) -> Result<Vec<(&'a str, &'a NbtTag)>, WritingError> {
    let Some(tag) = predicate.get(field) else {
        return Ok(Vec::new());
    };
    let compound = tag
        .extract_compound()
        .ok_or_else(|| WritingError::Message(format!("Expected an Adventure {field} compound")))?;
    let mut entries: Vec<_> = compound
        .child_tags
        .iter()
        .map(|(name, value)| (name.as_ref(), value))
        .collect();
    entries.sort_unstable_by_key(|(name, _)| *name);
    Ok(entries)
}

fn adventure_predicate_type(name: &str) -> Option<usize> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    ADVENTURE_PREDICATE_TYPES
        .iter()
        .position(|&kind| kind == name)
}

fn write_adventure_matchers(
    predicate: &NbtCompound,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    let exact = adventure_matchers(predicate, "components")?;
    seq.write_list(&exact, |seq, (name, data)| {
        let component = DataComponent::try_from_name(name).ok_or_else(|| {
            WritingError::Message(format!("Unknown Adventure exact component {name}"))
        })?;
        let value = read_data(component, data).ok_or_else(|| {
            WritingError::Message(format!("Invalid Adventure exact component {name}"))
        })?;
        seq.write_var_int(&VarInt(i32::from(component.to_id())))?;
        serialize(component, value.as_ref(), seq)
    })?;

    let partial = adventure_matchers(predicate, "predicates")?;
    if partial.len() > MAX_ADVENTURE_PARTIAL_MATCHERS {
        return Err(WritingError::Message(
            "Too many Adventure partial matchers".into(),
        ));
    }
    let mut ids = std::collections::HashSet::new();
    seq.write_var_int(&VarInt(partial.len() as i32))?;
    for (name, data) in &partial {
        // Java's persistent codec resolves a concrete predicate name before a component name.
        if let Some(kind) = adventure_predicate_type(name) {
            if !ids.insert((true, kind as i32)) {
                return Err(WritingError::Message(format!(
                    "Duplicate Adventure partial matcher {name}"
                )));
            }
            seq.write_bool(true)?;
            seq.write_var_int(&VarInt(kind as i32))?;
        } else {
            let component = DataComponent::try_from_name(name).ok_or_else(|| {
                WritingError::Message(format!("Unknown Adventure partial matcher {name}"))
            })?;
            if !matches!(data, NbtTag::Compound(compound) if compound.is_empty()) {
                return Err(WritingError::Message(format!(
                    "Expected an empty compound for Adventure component presence matcher {name}"
                )));
            }
            if !ids.insert((false, i32::from(component.to_id()))) {
                return Err(WritingError::Message(format!(
                    "Duplicate Adventure partial matcher {name}"
                )));
            }
            seq.write_bool(false)?;
            seq.write_var_int(&VarInt(i32::from(component.to_id())))?;
        }
        if matches!(data, NbtTag::End) {
            return Err(WritingError::Message(format!(
                "Missing Adventure partial matcher payload for {name}"
            )));
        }
        seq.write_nbt_with_version(Some(data), &JavaMinecraftVersion::V_26_3)?;
    }
    Ok(())
}

fn read_adventure_component_type(
    seq: &mut impl NetworkReadExt,
) -> Result<DataComponent, ReadingError> {
    let raw_id = seq.get_var_int()?.0;
    u8::try_from(raw_id)
        .ok()
        .and_then(DataComponent::try_from_id)
        .ok_or_else(|| ReadingError::Message(format!("Invalid Adventure component ID {raw_id}")))
}

fn exact_adventure_component_survives_storage(
    component: DataComponent,
    original: &dyn DataComponentImpl,
    tag: &NbtTag,
) -> bool {
    if matches!(tag, NbtTag::End) {
        return false;
    }
    let Some(restored) = read_data(component, tag) else {
        return false;
    };
    // Java enchantments are maps. Pumpkin's vectors may change order when read
    // back from NBT, so compare the distinct saved entries with the originals.
    let (expected, stored) = match component {
        DataComponent::Container => {
            let original = &get::<ContainerImpl>(original).items;
            let restored = &get::<ContainerImpl>(restored.as_ref()).items;
            return original.len() == restored.len()
                && original.iter().zip(restored).all(
                    |((slot, stack), (restored_slot, restored_stack))| {
                        slot == restored_slot && stack.are_equal(restored_stack)
                    },
                );
        }
        DataComponent::Enchantments => (
            &get::<EnchantmentsImpl>(original).enchantment,
            &get::<EnchantmentsImpl>(restored.as_ref()).enchantment,
        ),
        DataComponent::StoredEnchantments => (
            &get::<StoredEnchantmentsImpl>(original).enchantment,
            &get::<StoredEnchantmentsImpl>(restored.as_ref()).enchantment,
        ),
        _ => return original.equal(restored.as_ref()),
    };
    expected.len() == stored.len() && stored.iter().all(|entry| expected.contains(entry))
}

fn read_adventure_exact_matchers(
    seq: &mut impl NetworkReadExt,
) -> Result<NbtCompound, ReadingError> {
    let exact_len = read_count(seq, MAX_INITIAL_COLLECTION_SIZE)?;
    let mut exact = NbtCompound::new();
    for _ in 0..exact_len {
        let component = read_adventure_component_type(seq)?;
        let name = component.to_name();
        if exact.has(name) {
            return Err(ReadingError::Message(format!(
                "Duplicate Adventure exact component {name}"
            )));
        }
        let value = deserialize(component, seq)?;
        let tag = value.write_data();
        // Use the same NBT representation as a saved item, without silently discarding
        // fields from components whose persistent codecs are unavailable or lossy.
        if !exact_adventure_component_survives_storage(component, value.as_ref(), &tag) {
            return Err(ReadingError::Message(format!(
                "Adventure exact component {name} cannot be preserved as NBT"
            )));
        }
        exact.put(name, tag);
    }
    Ok(exact)
}

fn read_adventure_partial_matchers(
    seq: &mut impl NetworkReadExt,
) -> Result<NbtCompound, ReadingError> {
    let partial_len = read_count(seq, MAX_ADVENTURE_PARTIAL_MATCHERS)?;
    let mut partial = NbtCompound::new();
    for _ in 0..partial_len {
        let typed = seq.get_bool()?;
        let name = if typed {
            let raw_id = seq.get_var_int()?.0;
            let kind = usize::try_from(raw_id)
                .ok()
                .and_then(|id| ADVENTURE_PREDICATE_TYPES.get(id))
                .ok_or_else(|| {
                    ReadingError::Message(format!("Invalid Adventure predicate ID {raw_id}"))
                })?;
            format!("minecraft:{kind}")
        } else {
            let name = read_adventure_component_type(seq)?.to_name();
            if adventure_predicate_type(name).is_some() {
                return Err(ReadingError::Message(format!(
                    "Adventure presence matcher {name} conflicts with a named predicate"
                )));
            }
            name.to_owned()
        };
        if partial.has(&name) {
            return Err(ReadingError::Message(format!(
                "Duplicate Adventure partial matcher {name}"
            )));
        }
        // Both Java matcher types use fromCodecWithRegistries, an unnamed NBT
        // value following the type discriminator and the appropriate registry ID.
        let tag = seq
            .get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?
            .ok_or_else(|| {
                ReadingError::Message(format!(
                    "Missing Adventure partial matcher payload for {name}"
                ))
            })?;
        if !typed && !matches!(&tag, NbtTag::Compound(compound) if compound.is_empty()) {
            return Err(ReadingError::Message(format!(
                "Invalid Adventure component presence matcher payload for {name}"
            )));
        }
        partial.put(&name, tag);
    }
    Ok(partial)
}

fn serialize_adventure_predicate(
    predicate: &NbtTag,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    let _depth = AdventurePredicateDepth::enter().ok_or_else(|| {
        WritingError::Message("Adventure component matchers are nested too deeply".into())
    })?;
    let list: Vec<&NbtCompound> = match predicate {
        NbtTag::Compound(c) => c.get_list("predicates").map_or_else(
            || vec![c],
            |preds| preds.iter().filter_map(NbtTag::extract_compound).collect(),
        ),
        NbtTag::List(preds) => preds.iter().filter_map(NbtTag::extract_compound).collect(),
        _ => Vec::new(),
    };
    seq.write_list(&list, |seq, predicate| {
        if let Some(blocks_tag) = predicate.get("blocks")
            && let Some(idset) = IDSet::<pumpkin_data::Block>::read(blocks_tag)
        {
            seq.write_bool(true)?;
            serialize_idset(&idset, seq)?;
        } else {
            seq.write_bool(false)?;
        }
        seq.write_option(&predicate.get("state"), |seq, value| {
            write_adventure_state(value, seq)
        })?;
        seq.write_option(&predicate.get("nbt"), |seq, tag| {
            if tag.extract_compound().is_none() {
                return Err(WritingError::Message(
                    "Expected an Adventure block entity NBT compound".into(),
                ));
            }
            seq.write_nbt_with_version(Some(*tag), &JavaMinecraftVersion::V_26_3)
        })?;
        write_adventure_matchers(predicate, seq)
    })
}

fn deserialize_adventure_predicate(seq: &mut impl NetworkReadExt) -> Result<NbtTag, ReadingError> {
    let _depth = AdventurePredicateDepth::enter().ok_or_else(|| {
        ReadingError::TooLarge("Adventure component matchers are nested too deeply".into())
    })?;
    let count = read_count(seq, MAX_INITIAL_COLLECTION_SIZE)?;
    let mut preds = Vec::with_capacity(count.min(64));
    for _ in 0..count {
        let mut comp = NbtCompound::new();
        if seq.get_bool()? {
            let idset = deserialize_idset::<pumpkin_data::Block>(seq)?;
            idset.write(&mut comp, "blocks");
        }
        if seq.get_bool()? {
            comp.put_compound("state", read_adventure_state(seq)?);
        }
        if seq.get_bool()? {
            let Some(NbtTag::Compound(nbt)) =
                seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?
            else {
                return Err(ReadingError::Message(
                    "Expected an Adventure block entity NBT compound".into(),
                ));
            };
            comp.put_compound("nbt", nbt);
        }
        let exact = read_adventure_exact_matchers(seq)?;
        if !exact.is_empty() {
            comp.put_compound("components", exact);
        }
        let partial = read_adventure_partial_matchers(seq)?;
        if !partial.is_empty() {
            comp.put_compound("predicates", partial);
        }
        preds.push(NbtTag::Compound(comp));
    }
    Ok(NbtTag::List(preds))
}

impl DataComponentCodec<Self> for CanPlaceOnImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_adventure_predicate(&self.predicate, seq)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            predicate: deserialize_adventure_predicate(seq)?,
        })
    }
}

impl DataComponentCodec<Self> for CanBreakImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_adventure_predicate(&self.predicate, seq)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            predicate: deserialize_adventure_predicate(seq)?,
        })
    }
}

impl DataComponentCodec<Self> for AttributeModifiersImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.attribute_modifiers.len() as i32))?;
        for modifier in self.attribute_modifiers.iter() {
            seq.write_var_int(&VarInt::from(modifier.r#type.id as i32))?;
            seq.write_string(&modifier.id)?;
            seq.write_f64(modifier.amount)?;
            seq.write_var_int(&VarInt::from(modifier.operation as i32))?;
            seq.write_var_int(&VarInt(pumpkin_data::data_component_impl::slot_to_id(
                &modifier.slot,
            )))?;
            match &modifier.display {
                pumpkin_data::data_component_impl::ModifierDisplay::Default => {
                    seq.write_var_int(&VarInt(0))?;
                }
                pumpkin_data::data_component_impl::ModifierDisplay::Hidden => {
                    seq.write_var_int(&VarInt(1))?;
                }
                pumpkin_data::data_component_impl::ModifierDisplay::Override(text) => {
                    seq.write_var_int(&VarInt(2))?;
                    seq.write_slice(&text.encode_for_version(&JavaMinecraftVersion::V_26_3))?;
                }
            }
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_MODIFIERS: usize = 256;
        let len = read_count(seq, MAX_MODIFIERS)?;
        let mut modifiers = Vec::with_capacity(len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..len {
            let attr_id = u8::try_from(seq.get_var_int()?.0)
                .ok()
                .and_then(pumpkin_data::data_component_impl::attribute_from_id)
                .ok_or_else(|| ReadingError::Message("Unknown attribute ID".into()))?;
            let id = Cow::Owned(seq.get_str()?.to_string());
            let amount = seq.get_f64()?;
            let operation =
                pumpkin_data::data_component_impl::Operation::from_id(seq.get_var_int()?.0)
                    .ok_or_else(|| {
                        ReadingError::Message("Invalid attribute modifier operation".into())
                    })?;
            let slot = pumpkin_data::data_component_impl::slot_from_id(seq.get_var_int()?.0)
                .ok_or_else(|| ReadingError::Message("Invalid attribute modifier slot".into()))?;
            let display_type = seq.get_var_int()?.0;
            let display = match display_type {
                0 => pumpkin_data::data_component_impl::ModifierDisplay::Default,
                1 => pumpkin_data::data_component_impl::ModifierDisplay::Hidden,
                2 => {
                    let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?;
                    let text = tag.as_ref().map_or_else(
                        pumpkin_util::text::TextComponent::empty,
                        pumpkin_util::text::TextComponent::from_nbt,
                    );
                    pumpkin_data::data_component_impl::ModifierDisplay::Override(text)
                }
                _ => {
                    return Err(ReadingError::Message(
                        "Invalid attribute modifier display type".into(),
                    ));
                }
            };
            modifiers.push(pumpkin_data::data_component_impl::Modifier {
                r#type: attr_id,
                id,
                amount,
                operation,
                slot,
                display,
            });
        }
        Ok(Self {
            attribute_modifiers: Cow::Owned(modifiers),
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
        let floats_len = read_count(seq, usize::MAX)?;
        let mut floats = Vec::with_capacity(floats_len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..floats_len {
            floats.push(seq.get_f32()?);
        }
        let flags_len = read_count(seq, usize::MAX)?;
        let mut flags = Vec::with_capacity(flags_len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..flags_len {
            flags.push(seq.get_bool()?);
        }
        let strings_len = read_count(seq, usize::MAX)?;
        let mut strings = Vec::with_capacity(strings_len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..strings_len {
            strings.push(seq.get_str()?.to_string());
        }
        let colors_len = read_count(seq, usize::MAX)?;
        let mut colors = Vec::with_capacity(colors_len.min(MAX_INITIAL_COLLECTION_SIZE));
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
        seq.write_bool(self.hide_tooltip)?;
        seq.write_var_int(&VarInt(self.hidden_components.len() as i32))?;
        for comp in self.hidden_components.iter() {
            seq.write_var_int(&VarInt(i32::from(comp.to_id())))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_HIDDEN: usize = 256;
        let hide_tooltip = seq.get_bool()?;
        let len = read_count(seq, MAX_HIDDEN)?;
        let mut hidden_components = Vec::with_capacity(len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..len {
            let comp_id = u8::try_from(seq.get_var_int()?.0)
                .ok()
                .and_then(DataComponent::try_from_id)
                .ok_or_else(|| ReadingError::Message("Unknown hidden DataComponent ID".into()))?;
            if !hidden_components.contains(&comp_id) {
                hidden_components.push(comp_id);
            }
        }
        Ok(Self {
            hide_tooltip,
            hidden_components: Cow::Owned(hidden_components),
        })
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
        seq.write_bool(self.glint)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let glint = seq.get_bool()?;
        Ok(Self { glint })
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
        let stack = self.remainder.as_ref().ok_or_else(|| {
            WritingError::Message("UseRemainderImpl has no remainder stack".into())
        })?;
        serialize_item_stack_template(stack, seq)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let stack = deserialize_item_stack_template(seq)?;
        Ok(Self {
            remainder: Some(stack),
        })
    }
}

impl DataComponentCodec<Self> for DamageResistantImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_idset(&self.types, seq)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            types: deserialize_idset(seq)?,
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
        let rules_len = read_count(seq, usize::MAX)?;
        let mut rules = Vec::with_capacity(rules_len.min(MAX_INITIAL_COLLECTION_SIZE));
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
        seq.write_f32(self.disable_blocking_for_seconds)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let item_damage_per_attack = seq.get_var_int()?.0 as u32;
        let disable_blocking_for_seconds = seq.get_f32()?;
        Ok(Self {
            item_damage_per_attack,
            disable_blocking_for_seconds,
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
        seq.write_var_int(&VarInt(self.death_effects.len() as i32))?;
        for effect in self.death_effects.iter() {
            serialize_consume_effect(effect, seq)?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_DEATH_EFFECTS: usize = 256;
        let len = read_count(seq, MAX_DEATH_EFFECTS)?;
        let mut death_effects = Vec::with_capacity(len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..len {
            death_effects.push(deserialize_consume_effect(seq)?);
        }
        Ok(Self {
            death_effects: Cow::Owned(death_effects),
        })
    }
}

impl DataComponentCodec<Self> for BlocksAttacksImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_f32(self.block_delay_seconds)?;
        seq.write_f32(self.disable_cooldown_scale)?;
        seq.write_var_int(&VarInt(self.damage_reductions.len() as i32))?;
        for r in self.damage_reductions.iter() {
            seq.write_f32(r.horizontal_blocking_angle)?;
            if let Some(dt) = &r.damage_type {
                seq.write_bool(true)?;
                serialize_idset(dt, seq)?;
            } else {
                seq.write_bool(false)?;
            }
            seq.write_f32(r.base)?;
            seq.write_f32(r.factor)?;
        }
        seq.write_f32(self.item_damage.threshold)?;
        seq.write_f32(self.item_damage.base)?;
        seq.write_f32(self.item_damage.factor)?;
        if let Some(bypassed) = &self.bypassed_by {
            seq.write_bool(true)?;
            serialize_idset(bypassed, seq)?;
        } else {
            seq.write_bool(false)?;
        }
        for opt_sound in [&self.block_sound, &self.disabled_sound] {
            if let Some(sound) = opt_sound {
                seq.write_bool(true)?;
                let proto_sound = data_to_proto_sound(sound);
                crate::IdOr::<crate::SoundEvent>::write(&proto_sound, seq, |w, e| {
                    w.write_string(&e.sound_name)?;
                    w.write_option(&e.range, |w2, r| w2.write_f32(*r))
                })?;
            } else {
                seq.write_bool(false)?;
            }
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_REDUCTIONS: usize = 256;
        fn read_opt_sound(
            seq: &mut impl NetworkReadExt,
        ) -> Result<Option<IdOr<SoundEvent>>, ReadingError> {
            if seq.get_bool()? {
                let proto = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                    let sound_name = r.get_str()?.to_string();
                    let range = r.get_option(NetworkReadExt::get_f32)?;
                    Ok(crate::SoundEvent { sound_name, range })
                })
                .map_err(|e| ReadingError::Message(format!("No sound: {e}")))?;
                Ok(proto_to_data_sound(&proto))
            } else {
                Ok(None)
            }
        }
        let block_delay_seconds = seq.get_f32()?;
        let disable_cooldown_scale = seq.get_f32()?;
        let red_len = read_count(seq, MAX_REDUCTIONS)?;
        let mut damage_reductions = Vec::with_capacity(red_len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..red_len {
            let horizontal_blocking_angle = seq.get_f32()?;
            let damage_type = if seq.get_bool()? {
                Some(deserialize_idset(seq)?)
            } else {
                None
            };
            let base = seq.get_f32()?;
            let factor = seq.get_f32()?;
            damage_reductions.push(pumpkin_data::data_component_impl::DamageReduction {
                horizontal_blocking_angle,
                damage_type,
                base,
                factor,
            });
        }
        let item_damage = pumpkin_data::data_component_impl::ItemDamageFunction {
            threshold: seq.get_f32()?,
            base: seq.get_f32()?,
            factor: seq.get_f32()?,
        };
        let bypassed_by = if seq.get_bool()? {
            Some(deserialize_idset(seq)?)
        } else {
            None
        };
        let block_sound = read_opt_sound(seq)?;
        let disabled_sound = read_opt_sound(seq)?;
        Ok(Self {
            block_delay_seconds,
            disable_cooldown_scale,
            damage_reductions: Cow::Owned(damage_reductions),
            item_damage,
            bypassed_by,
            block_sound,
            disabled_sound,
        })
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
        seq.write_var_int(&VarInt::from(self.cost))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let cost = seq.get_var_int()?.0;
        Ok(Self { cost })
    }
}

impl DataComponentCodec<Self> for DyeImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.color.id() as i32))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let color = u8::try_from(id)
            .ok()
            .and_then(pumpkin_data::dye_color::DyeColor::by_id)
            .ok_or_else(|| ReadingError::Message(format!("Invalid DyeColor id {id}")))?;
        Ok(Self { color })
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
        const MAX_PROJECTILES: usize = 1024;
        let len = read_count(seq, MAX_PROJECTILES)?;
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
        const MAX_PAGES: usize = 100;
        let len = read_count(seq, MAX_PAGES)?;
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
        seq.write_var_int(&VarInt(self.generation))?;
        seq.write_var_int(&VarInt::from(self.pages.len() as i32))?;
        for page in &self.pages {
            seq.write_slice(&page.encode_for_version(&JavaMinecraftVersion::V_26_3))?;
            seq.write_bool(false)?;
        }
        seq.write_bool(self.resolved)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_BOOK_PAGES: usize = 100;
        let title = seq.get_str()?.to_string();
        if seq.get_bool()? {
            let _ = seq.get_str()?;
        }
        let author = seq.get_str()?.to_string();
        let generation = seq.get_var_int()?.0.clamp(0, 3);
        let pages_len = read_count(seq, MAX_BOOK_PAGES)?;
        let mut pages = Vec::with_capacity(pages_len.min(MAX_INITIAL_COLLECTION_SIZE));
        for _ in 0..pages_len {
            let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?;
            let comp = tag.as_ref().map_or_else(
                pumpkin_util::text::TextComponent::empty,
                pumpkin_util::text::TextComponent::from_nbt,
            );
            if seq.get_bool()? {
                let _ = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?;
            }
            pages.push(comp);
        }
        let resolved = seq.get_bool()?;
        Ok(Self {
            title,
            author,
            generation,
            pages,
            resolved,
        })
    }
}

fn read_holder_reference<T: Copy>(
    seq: &mut impl NetworkReadExt,
    entries: &[T],
    kind: &str,
) -> Result<T, ReadingError> {
    // Not IdOr::read: it truncates to u16, so out-of-range ids would alias valid entries.
    let id = seq.get_var_int()?.0;
    if id == 0 {
        return Err(ReadingError::Message(format!(
            "Inline {kind} is not supported"
        )));
    }
    usize::try_from(id)
        .ok()
        .and_then(|id| entries.get(id - 1))
        .copied()
        .ok_or_else(|| ReadingError::Message(format!("Unknown {kind} reference {id}")))
}

impl DataComponentCodec<Self> for TrimImpl {
    // ArmorTrim.STREAM_CODEC: two ByteBufCodecs.holder values, written as registry id + 1.
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let material = self.material_enum().ok_or_else(|| {
            WritingError::Message(format!("Unknown trim material {:?}", self.material))
        })?;
        let pattern = self.pattern_enum().ok_or_else(|| {
            WritingError::Message(format!("Unknown trim pattern {:?}", self.pattern))
        })?;
        seq.write_var_int(&VarInt(material.id() as i32 + 1))?;
        seq.write_var_int(&VarInt(pattern.id() as i32 + 1))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let material = read_holder_reference(seq, TrimMaterial::all(), "trim material")?;
        let pattern = read_holder_reference(seq, TrimPattern::all(), "trim pattern")?;
        Ok(Self::new(material, pattern))
    }
}

impl DataComponentCodec<Self> for DebugStickStateImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            properties: Vec::new(),
        })
    }
}

impl DataComponentCodec<Self> for EntityDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let entity_type = self
            .get_entity_type()
            .ok_or_else(|| WritingError::Message("entity_data has no entity type".into()))?;
        let mut nbt = self.nbt.clone().unwrap_or_default();
        nbt.child_tags.remove("id");
        seq.write_var_int(&VarInt(i32::from(entity_type.id)))?;
        seq.write_nbt(NbtTag::Compound(nbt))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let type_id = seq.get_var_int()?.0;
        let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?;
        let entity_type = u16::try_from(type_id)
            .ok()
            .and_then(EntityType::from_raw)
            .ok_or_else(|| ReadingError::Message(format!("Unknown entity type id {type_id}")))?;
        let mut nbt = match tag {
            Some(NbtTag::Compound(c)) => c,
            _ => pumpkin_nbt::compound::NbtCompound::new(),
        };
        nbt.child_tags.remove("id");
        Ok(Self {
            entity_type: Some(entity_type),
            nbt: (!nbt.child_tags.is_empty()).then_some(nbt),
        })
    }
}

impl DataComponentCodec<Self> for BucketEntityDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_nbt(NbtTag::Compound(self.nbt.clone().unwrap_or_default()))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
        let nbt = match tag {
            Some(NbtTag::Compound(c)) => Some(c),
            _ => None,
        };
        Ok(Self { nbt })
    }
}

impl DataComponentCodec<Self> for BlockEntityDataImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let mut nbt = self.nbt.clone();
        let type_id = if let Some(id) = nbt.get_string("id") {
            let bare = id.strip_prefix("minecraft:").unwrap_or(id);
            pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
                .iter()
                .position(|&name| name == bare)
                .ok_or_else(|| WritingError::Message(format!("Unknown block entity type {id}")))?
                as i32
        } else {
            0
        };
        nbt.child_tags.remove("id");
        seq.write_var_int(&VarInt(type_id))?;
        seq.write_nbt(NbtTag::Compound(nbt))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let type_id = seq.get_var_int()?.0;
        let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?;
        let type_name = usize::try_from(type_id)
            .ok()
            .and_then(|idx| {
                pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
                    .get(idx)
                    .copied()
            })
            .ok_or_else(|| {
                ReadingError::Message(format!("Unknown block entity type id {type_id}"))
            })?;
        let mut nbt = if let Some(NbtTag::Compound(c)) = tag {
            c
        } else {
            pumpkin_nbt::compound::NbtCompound::new()
        };
        nbt.put_string("id", format!("minecraft:{type_name}"));
        Ok(Self { nbt })
    }
}

impl DataComponentCodec<Self> for InstrumentImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let id = pumpkin_data::instrument::Instrument::from_name(&self.instrument)
            .map_or(1, |inst| inst.id() as i32 + 1);
        seq.write_var_int(&VarInt(id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let raw_id = seq.get_var_int()?.0;
        let instrument = if raw_id <= 0 {
            let _sound = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
                let sound_name = r.get_str()?.to_string();
                let range = r.get_option(NetworkReadExt::get_f32)?;
                Ok(crate::SoundEvent { sound_name, range })
            })
            .map_err(|e| ReadingError::Message(format!("No sound: {e}")))?;
            let _use_duration = seq.get_f32()?;
            let _range = seq.get_f32()?;
            let _description = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            Cow::Borrowed("minecraft:ponder_goat_horn")
        } else {
            let idx = (raw_id - 1) as usize;
            pumpkin_data::instrument::Instrument::all()
                .get(idx)
                .map_or(Cow::Borrowed("minecraft:ponder_goat_horn"), |inst| {
                    Cow::Borrowed(inst.asset_id())
                })
        };
        Ok(Self { instrument })
    }
}

impl DataComponentCodec<Self> for ProvidesTrimMaterialImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let id = pumpkin_data::trim_material::TrimMaterial::from_name(&self.material)
            .map_or(1, |mat| mat.id() as i32 + 1);
        seq.write_var_int(&VarInt(id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let raw_id = seq.get_var_int()?.0;
        let material = if raw_id <= 0 {
            let _base_asset = seq.get_str()?;
            let overrides_len = seq.get_var_int()?.0 as usize;
            for _ in 0..overrides_len.min(256) {
                let _key = seq.get_str()?;
                let _val = seq.get_str()?;
            }
            let _description = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            Cow::Borrowed("minecraft:quartz")
        } else {
            let idx = (raw_id - 1) as usize;
            pumpkin_data::trim_material::TrimMaterial::all()
                .get(idx)
                .map_or(Cow::Borrowed("minecraft:quartz"), |mat| {
                    Cow::Borrowed(mat.asset_id())
                })
        };
        Ok(Self { material })
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
        let name = self.song.strip_prefix("minecraft:").unwrap_or(self.song);
        let song_id = pumpkin_data::jukebox_song::JukeboxSong::from_name(name)
            .map_or(1, |s| i32::try_from(s.get_id()).unwrap_or(0) + 1);
        seq.write_var_int(&VarInt::from(song_id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let raw = seq.get_var_int()?.0;
        if raw < 0 {
            return Err(ReadingError::Message(
                "negative JukeboxSong holder id".to_string(),
            ));
        }
        if raw == 0 {
            let sound_raw = seq.get_var_int()?.0;
            if sound_raw == 0 {
                let _ = seq.get_str()?;
                if seq.get_bool()? {
                    let _ = seq.get_f32()?;
                }
            }
            let _ =
                seq.get_nbt_with_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2)?;
            let _ = seq.get_f32()?;
            let _ = seq.get_var_int()?;
            return Ok(Self { song: "" });
        }
        let song = Self::song_from_id((raw - 1) as u32).map_or("", Self::song_registry_key);
        Ok(Self { song })
    }
}

impl DataComponentCodec<Self> for ProvidesBannerPatternsImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_string(self.key.strip_prefix('#').unwrap_or(&self.key))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let tag = seq.get_str()?;
        let key = if tag.starts_with('#') {
            Cow::Owned(tag.to_string())
        } else {
            Cow::Owned(format!("#{tag}"))
        };
        Ok(Self { key })
    }
}

impl DataComponentCodec<Self> for RecipesImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self {
            recipes: Cow::Borrowed(&[]),
        })
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
        seq.write_bool(false)?;
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
        let full_profile = seq.get_bool()?;
        let mut name = None;
        let mut id = None;
        let mut properties = Vec::new();
        if full_profile {
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
        let len = read_count(seq, usize::MAX)?;
        let mut layers = Vec::with_capacity(len.min(MAX_INITIAL_COLLECTION_SIZE));
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
        for side in [&self.back, &self.left, &self.right, &self.front] {
            if let Some(stack) = side {
                seq.write_bool(true)?;
                serialize_item_stack_template(stack, seq)?;
            } else {
                seq.write_bool(false)?;
            }
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let mut read_side =
            || -> Result<Option<pumpkin_data::item_stack::ItemStack>, ReadingError> {
                if seq.get_bool()? {
                    Ok(Some(deserialize_item_stack_template(seq)?))
                } else {
                    Ok(None)
                }
            };
        Ok(Self {
            back: read_side()?,
            left: read_side()?,
            right: read_side()?,
            front: read_side()?,
        })
    }
}

impl DataComponentCodec<Self> for ContainerImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let mut slots = [None; 256];
        let mut len = 0;
        for (slot, stack) in &self.items {
            let slot = usize::from(*slot);
            if slots[slot].replace(stack).is_some() {
                return Err(WritingError::Message("Duplicate container slot".into()));
            }
            len = len.max(slot + 1);
        }
        seq.write_var_int(&VarInt::from(len as i32))?;
        for stack in &slots[..len] {
            seq.write_bool(stack.is_some())?;
            if let Some(stack) = stack {
                serialize_item_stack_template(stack, seq)?;
            }
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        const MAX_SIZE: usize = 256;
        let len = read_count(seq, MAX_SIZE)?;
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
        let len = read_count(seq, usize::MAX)?;
        let mut properties = Vec::with_capacity(len.min(MAX_INITIAL_COLLECTION_SIZE));
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
        seq.write_var_int(&VarInt::from(self.bees.len() as i32))?;
        for bee in &self.bees {
            let comp = bee.extract_compound();
            let entity_data = comp
                .and_then(|c| c.get_compound("entity_data"))
                .cloned()
                .unwrap_or_default();
            let ticks = comp.and_then(|c| c.get_int("ticks_in_hive")).unwrap_or(0);
            let min_ticks = comp
                .and_then(|c| c.get_int("min_ticks_in_hive"))
                .unwrap_or(0);
            seq.write_var_int(&VarInt(i32::from(EntityType::BEE.id)))?;
            seq.write_nbt(NbtTag::Compound(entity_data))?;
            seq.write_var_int(&VarInt(ticks))?;
            seq.write_var_int(&VarInt(min_ticks))?;
        }
        Ok(())
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let len = seq.get_var_int()?.0 as usize;
        let mut bees = Vec::with_capacity(len.min(64));
        for _ in 0..len {
            let _entity_type = seq.get_var_int()?;
            let nbt = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            let ticks = seq.get_var_int()?.0;
            let min_ticks = seq.get_var_int()?.0;
            let mut bee_comp = pumpkin_nbt::compound::NbtCompound::new();
            if let Some(NbtTag::Compound(c)) = nbt {
                bee_comp.put_compound("entity_data", c);
            }
            bee_comp.put_int("ticks_in_hive", ticks);
            bee_comp.put_int("min_ticks_in_hive", min_ticks);
            bees.push(NbtTag::Compound(bee_comp));
        }
        Ok(Self { bees })
    }
}

impl DataComponentCodec<Self> for SulfurCubeContentImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_item_stack_template(&self.item, seq)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let item = deserialize_item_stack_template(seq)?;
        Ok(Self { item })
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
        crate::IdOr::<crate::SoundEvent>::write(&data_to_proto_sound(&self.sound), seq, |w, e| {
            w.write_string(&e.sound_name)?;
            w.write_option(&e.range, |w2, r| w2.write_f32(*r))
        })
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let proto = crate::IdOr::<crate::SoundEvent>::read(seq, |r| {
            let sound_name = r.get_str()?.into();
            let range = r.get_option(NetworkReadExt::get_f32)?;
            Ok(crate::SoundEvent { sound_name, range })
        })?;
        let sound = proto_to_data_sound(&proto)
            .ok_or_else(|| ReadingError::Message("Invalid sound in BreakSoundImpl".into()))?;
        Ok(Self { sound })
    }
}

impl DataComponentCodec<Self> for InteractAnimationImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        write_swing_animation(seq, self.animation_type, self.duration)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let animation = SwingAnimationImpl::deserialize(seq)?;
        Ok(Self {
            animation_type: animation.animation_type,
            duration: animation.duration,
        })
    }
}

impl DataComponentCodec<Self> for BlockTransformerImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let id = match self.transformer.as_ref() {
            "minecraft:hoe" | "hoe" => 1,
            "minecraft:shovel" | "shovel" => 2,
            _ => 0,
        };
        seq.write_var_int(&VarInt(id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let transformer = match id {
            0 => Cow::Borrowed("minecraft:axe"),
            1 => Cow::Borrowed("minecraft:hoe"),
            2 => Cow::Borrowed("minecraft:shovel"),
            _ => {
                return Err(ReadingError::Message(format!(
                    "Invalid BlockTransformer id {id}"
                )));
            }
        };
        Ok(Self { transformer })
    }
}

impl DataComponentCodec<Self> for VillagerFoodImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.nutrition))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let nutrition = seq.get_var_int()?.0;
        Ok(Self { nutrition })
    }
}

impl DataComponentCodec<Self> for MobVisibilityImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_idset(&self.targeting_entity_types, seq)?;
        seq.write_f32(self.visibility)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let targeting_entity_types = deserialize_idset(seq)?;
        let visibility = seq.get_f32()?;
        // Like vanilla's validated item stacks, the persistent range applies here too.
        if !Self::is_valid_visibility(visibility) {
            return Err(ReadingError::Message(format!(
                "Mob visibility {visibility} outside 0..={}",
                Self::MAX_VISIBILITY
            )));
        }
        Ok(Self {
            targeting_entity_types,
            visibility,
        })
    }
}

impl DataComponentCodec<Self> for ProvidesPotteryPatternImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let id = pumpkin_data::decorated_pot_pattern::DecoratedPotPattern::from_name(&self.pattern)
            .map_or(0, |p| p.id() as i32);
        seq.write_var_int(&VarInt(id))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let pattern = usize::try_from(id)
            .ok()
            .and_then(|idx| {
                pumpkin_data::decorated_pot_pattern::DecoratedPotPattern::all().get(idx)
            })
            .map_or(Cow::Borrowed("minecraft:angler"), |p| {
                Cow::Owned(format!("minecraft:{}", p.to_name()))
            });
        Ok(Self { pattern })
    }
}

fn serialize_sign_messages(
    messages: &[Cow<'static, str>; 4],
    filtered_messages: Option<&[Cow<'static, str>; 4]>,
    color: pumpkin_data::dye_color::DyeColor,
    has_glowing_text: bool,
    seq: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    for msg in messages {
        seq.write_nbt(NbtTag::String(msg.to_string().into()))?;
    }
    if let Some(filtered) = filtered_messages {
        seq.write_bool(true)?;
        for msg in filtered {
            seq.write_nbt(NbtTag::String(msg.to_string().into()))?;
        }
    } else {
        seq.write_bool(false)?;
    }
    seq.write_var_int(&VarInt::from(color.id() as i32))?;
    seq.write_bool(has_glowing_text)
}

type DeserializedSignData = (
    [Cow<'static, str>; 4],
    Option<[Cow<'static, str>; 4]>,
    pumpkin_data::dye_color::DyeColor,
    bool,
);

fn deserialize_sign_messages(
    seq: &mut impl NetworkReadExt,
) -> Result<DeserializedSignData, ReadingError> {
    let mut messages = [
        Cow::Borrowed(""),
        Cow::Borrowed(""),
        Cow::Borrowed(""),
        Cow::Borrowed(""),
    ];
    for slot in &mut messages {
        let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
        if let Some(t) = tag {
            *slot = Cow::Owned(pumpkin_util::text::TextComponent::from_nbt(&t).get_text());
        }
    }
    let filtered_messages = if seq.get_bool()? {
        let mut filtered = [
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
        ];
        for slot in &mut filtered {
            let tag = seq.get_nbt_with_version(&JavaMinecraftVersion::V_26_2)?;
            if let Some(t) = tag {
                *slot = Cow::Owned(pumpkin_util::text::TextComponent::from_nbt(&t).get_text());
            }
        }
        Some(filtered)
    } else {
        None
    };
    let color_id = seq.get_var_int()?.0;
    let color = u8::try_from(color_id)
        .ok()
        .and_then(pumpkin_data::dye_color::DyeColor::by_id)
        .ok_or_else(|| ReadingError::Message(format!("Invalid SignText DyeColor {color_id}")))?;
    let has_glowing_text = seq.get_bool()?;
    Ok((messages, filtered_messages, color, has_glowing_text))
}

impl DataComponentCodec<Self> for SignTextFrontImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_sign_messages(
            &self.messages,
            self.filtered_messages.as_ref(),
            self.color,
            self.has_glowing_text,
            seq,
        )
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let (messages, filtered_messages, color, has_glowing_text) =
            deserialize_sign_messages(seq)?;
        Ok(Self {
            messages,
            filtered_messages,
            color,
            has_glowing_text,
        })
    }
}

impl DataComponentCodec<Self> for SignTextBackImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        serialize_sign_messages(
            &self.messages,
            self.filtered_messages.as_ref(),
            self.color,
            self.has_glowing_text,
            seq,
        )
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let (messages, filtered_messages, color, has_glowing_text) =
            deserialize_sign_messages(seq)?;
        Ok(Self {
            messages,
            filtered_messages,
            color,
            has_glowing_text,
        })
    }
}

impl DataComponentCodec<Self> for CushionColorImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        seq.write_var_int(&VarInt::from(self.color.id() as i32))
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let id = seq.get_var_int()?.0;
        let color = u8::try_from(id)
            .ok()
            .and_then(pumpkin_data::dye_color::DyeColor::by_id)
            .ok_or_else(|| ReadingError::Message(format!("Invalid CushionColor id {id}")))?;
        Ok(Self { color })
    }
}

impl DataComponentCodec<Self> for CompostableImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        let layers = match self.chance {
            0.3 => "minecraft:compostable/low",
            0.5 => "minecraft:compostable/low_medium",
            0.65 => "minecraft:compostable/medium",
            0.85 => "minecraft:compostable/medium_high",
            _ => "minecraft:compostable/always_add_one",
        };
        seq.write_bool(false)?;
        seq.write_string(layers)
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let is_direct = seq.get_bool()?;
        let chance = if is_direct {
            let val = seq.get_i32()?;
            if val > 0 { 1.0 } else { 0.3 }
        } else {
            let key = seq.get_str()?;
            match key.strip_prefix("minecraft:").unwrap_or(&key) {
                "compostable/low" => 0.3,
                "compostable/low_medium" => 0.5,
                "compostable/medium" => 0.65,
                "compostable/medium_high" => 0.85,
                _ => 1.0,
            }
        };
        Ok(Self { chance })
    }
}

impl DataComponentCodec<Self> for CookingFuelImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        match &self.burn_time {
            pumpkin_data::data_component_impl::IntProvider::Inline(val) => {
                seq.write_bool(true)?;
                seq.write_i32(*val)?;
            }
            pumpkin_data::data_component_impl::IntProvider::Id(id) => {
                seq.write_bool(false)?;
                seq.write_string(id)?;
            }
        }
        match &self.speed_multiplier {
            pumpkin_data::data_component_impl::FloatProvider::Inline(val) => {
                seq.write_bool(true)?;
                seq.write_f32(*val)
            }
            pumpkin_data::data_component_impl::FloatProvider::Id(id) => {
                seq.write_bool(false)?;
                seq.write_string(id)
            }
        }
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let burn_time = if seq.get_bool()? {
            pumpkin_data::data_component_impl::IntProvider::Inline(seq.get_i32()?)
        } else {
            pumpkin_data::data_component_impl::IntProvider::Id(Cow::Owned(
                seq.get_str()?.to_string(),
            ))
        };
        let speed_multiplier = if seq.get_bool()? {
            pumpkin_data::data_component_impl::FloatProvider::Inline(seq.get_f32()?)
        } else {
            pumpkin_data::data_component_impl::FloatProvider::Id(Cow::Owned(
                seq.get_str()?.to_string(),
            ))
        };
        Ok(Self {
            burn_time,
            speed_multiplier,
        })
    }
}

impl DataComponentCodec<Self> for BrewingFuelImpl {
    fn serialize(&self, seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        match &self.uses {
            pumpkin_data::data_component_impl::IntProvider::Inline(val) => {
                seq.write_bool(true)?;
                seq.write_i32(*val)?;
            }
            pumpkin_data::data_component_impl::IntProvider::Id(id) => {
                seq.write_bool(false)?;
                seq.write_string(id)?;
            }
        }
        match &self.speed_multiplier {
            pumpkin_data::data_component_impl::FloatProvider::Inline(val) => {
                seq.write_bool(true)?;
                seq.write_f32(*val)
            }
            pumpkin_data::data_component_impl::FloatProvider::Id(id) => {
                seq.write_bool(false)?;
                seq.write_string(id)
            }
        }
    }

    fn deserialize(seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let uses = if seq.get_bool()? {
            pumpkin_data::data_component_impl::IntProvider::Inline(seq.get_i32()?)
        } else {
            pumpkin_data::data_component_impl::IntProvider::Id(Cow::Owned(
                seq.get_str()?.to_string(),
            ))
        };
        let speed_multiplier = if seq.get_bool()? {
            pumpkin_data::data_component_impl::FloatProvider::Inline(seq.get_f32()?)
        } else {
            pumpkin_data::data_component_impl::FloatProvider::Id(Cow::Owned(
                seq.get_str()?.to_string(),
            ))
        };
        Ok(Self {
            uses,
            speed_multiplier,
        })
    }
}

impl DataComponentCodec<Self> for WaxedImpl {
    fn serialize(&self, _seq: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        Ok(())
    }

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_container_preserves_sparse_slot_indices() -> Result<(), Box<dyn std::error::Error>> {
        use pumpkin_data::{item::Item, item_stack::ItemStack};

        let container = ContainerImpl {
            items: vec![(2, ItemStack::new(1, &Item::STICK))],
        };
        let mut encoded = Vec::new();
        container.serialize(&mut encoded)?;
        assert_eq!(encoded.get(..4), Some([3, 0, 0, 1].as_slice()));
        let mut input = encoded.as_slice();
        let decoded = ContainerImpl::deserialize(&mut input)?;
        assert!(input.is_empty());
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].0, 2);
        assert_eq!(decoded.items[0].1.get_item(), &Item::STICK);
        Ok(())
    }

    fn textured_profile() -> ProfileImpl {
        ProfileImpl {
            name: Some("Notch".to_string()),
            properties: vec![ProfileProperty {
                name: "textures".to_string(),
                value: "eyJ0ZXh0dXJlcyI6e319".to_string(),
                signature: None,
            }],
            ..Default::default()
        }
    }

    /// The wire form vanilla expects for `textured_profile`.
    fn textured_profile_bytes() -> Vec<u8> {
        let mut bytes = vec![
            0, // partial profile tag
            1, 5, // name present, 5 bytes
        ];
        bytes.extend_from_slice(b"Notch");
        bytes.push(0); // no id
        bytes.push(1); // one property
        bytes.push(8); // property name, 8 bytes
        bytes.extend_from_slice(b"textures");
        bytes.push(20); // property value, 20 bytes
        bytes.extend_from_slice(b"eyJ0ZXh0dXJlcyI6e319");
        bytes.push(0); // no signature
        bytes.extend_from_slice(&[0, 0, 0, 0]); // no texture, cape, elytra, model
        bytes
    }

    #[test]
    fn partial_profile_is_written_with_a_false_tag() {
        let mut encoded = Vec::new();
        textured_profile().serialize(&mut encoded).unwrap();
        assert_eq!(encoded, textured_profile_bytes());
    }

    #[test]
    fn partial_profile_is_read_as_name_then_id() {
        let decoded = ProfileImpl::deserialize(&mut textured_profile_bytes().as_slice()).unwrap();
        assert_eq!(decoded, textured_profile());
    }

    #[test]
    fn full_profile_is_read_as_id_then_name() {
        let id = uuid::Uuid::from_u128(0x0000_0001_0000_0002_0000_0003_0000_0004);
        let mut encoded = vec![1];
        encoded.extend_from_slice(id.as_bytes());
        encoded.push(5);
        encoded.extend_from_slice(b"Notch");
        encoded.extend_from_slice(&[0, 0, 0, 0, 0]);

        let decoded = ProfileImpl::deserialize(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded.name.as_deref(), Some("Notch"));
        assert_eq!(decoded.id, Some([1, 2, 3, 4]));
    }

    #[test]
    fn variant_components_use_vanilla_network_ids() {
        let mut encoded = Vec::new();
        SalmonSizeImpl {
            value: Cow::Borrowed("large"),
        }
        .serialize(&mut encoded)
        .unwrap();
        TropicalFishPatternImpl {
            value: Cow::Borrowed("stripey"),
        }
        .serialize(&mut encoded)
        .unwrap();
        SheepColorImpl {
            value: Cow::Borrowed("red"),
        }
        .serialize(&mut encoded)
        .unwrap();
        // Holders send the registry id plus one.
        PaintingVariantImpl {
            value: Cow::Borrowed("minecraft:alban"),
        }
        .serialize(&mut encoded)
        .unwrap();
        assert_eq!(encoded, [2, 0x81, 0x02, 14, 1]);

        let clamped = SalmonSizeImpl::deserialize(&mut [9u8].as_slice()).unwrap();
        assert_eq!(clamped.value, "large");

        let min_holder = [0x80, 0x80, 0x80, 0x80, 0x08];
        assert!(PaintingVariantImpl::deserialize(&mut min_holder.as_slice()).is_err());
    }

    #[test]
    fn all_default_item_components_serialize_and_deserialize() {
        for id in 0..=2000 {
            let Some(item) = pumpkin_data::item::Item::from_id(id) else {
                continue;
            };
            for (comp, val) in item.components {
                if *comp == DataComponent::EntityData {
                    // Static Item constants use nbt: None because NbtCompound is not const-constructible.
                    continue;
                }
                let mut buf = Vec::new();
                serialize(*comp, *val, &mut buf).unwrap_or_else(|e| {
                    panic!("Failed to serialize {comp:?} on {}: {e}", item.registry_key)
                });
                let mut slice = buf.as_slice();
                let _decoded = deserialize(*comp, &mut slice).unwrap_or_else(|e| {
                    panic!(
                        "Failed to deserialize {comp:?} on {}: {e}",
                        item.registry_key
                    )
                });
                assert!(
                    slice.is_empty(),
                    "Trailing bytes after deserializing {comp:?} on {}: {} bytes left",
                    item.registry_key,
                    slice.len()
                );
            }
        }
    }

    #[test]
    fn oversized_tool_rule_count_is_rejected_without_preallocating() {
        for count in [
            [0xFF, 0xFF, 0xFF, 0xFF, 0x07],
            [0xFF, 0xFF, 0xFF, 0xFF, 0x0F],
        ] {
            assert!(ToolImpl::deserialize(&mut count.as_slice()).is_err());
        }
    }

    #[test]
    fn trim_rejects_out_of_range_material_references() {
        let past_end = TrimMaterial::all().len() as i32 + 1;
        for material in [0, past_end, 65537, -65535, i32::MIN] {
            let mut encoded = Vec::new();
            encoded.write_var_int(&VarInt(material)).unwrap();
            encoded.write_var_int(&VarInt(1)).unwrap();
            assert!(
                TrimImpl::deserialize(&mut encoded.as_slice()).is_err(),
                "material reference {material} was accepted"
            );
        }
    }

    fn enchantment_wire_bytes(id: u8, level: i32) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.write_var_int(&VarInt(1)).unwrap();
        encoded.write_var_int(&VarInt(i32::from(id))).unwrap();
        encoded.write_var_int(&VarInt(level)).unwrap();
        encoded
    }

    #[test]
    fn enchantment_levels_outside_the_vanilla_range_are_rejected() {
        for level in [1, 255] {
            assert!(
                EnchantmentsImpl::deserialize(
                    &mut enchantment_wire_bytes(Enchantment::RIPTIDE.id, level).as_slice()
                )
                .is_ok(),
                "level {level} was rejected"
            );
        }
        for level in [-1, 0, 256, 2000] {
            assert!(
                EnchantmentsImpl::deserialize(
                    &mut enchantment_wire_bytes(Enchantment::RIPTIDE.id, level).as_slice()
                )
                .is_err(),
                "level {level} was accepted"
            );
        }
        assert!(
            StoredEnchantmentsImpl::deserialize(
                &mut enchantment_wire_bytes(Enchantment::MENDING.id, 2000).as_slice()
            )
            .is_err()
        );
    }

    #[test]
    fn pot_decorations_wire_codec_matches_26_3_composite() {
        let mut named_sherd = pumpkin_data::item_stack::ItemStack::new(
            1,
            &pumpkin_data::item::Item::FLOW_POTTERY_SHERD,
        );
        named_sherd.set_data_component(UnbreakableImpl);

        let decorations = PotDecorationsImpl {
            back: Some(named_sherd),
            left: None,
            right: Some(pumpkin_data::item_stack::ItemStack::new(
                1,
                &pumpkin_data::item::Item::BRICK,
            )),
            front: None,
        };

        let mut encoded = Vec::new();
        decorations.serialize(&mut encoded).unwrap();
        // Leading byte must be `true` (`1`) for `back`, not a legacy VarInt list length.
        assert_eq!(encoded[0], 1);

        let decoded = PotDecorationsImpl::deserialize(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, decorations);
        assert!(decoded.left.is_none());
        assert!(decoded.front.is_none());
        assert_eq!(
            decoded.back.as_ref().map(|s| s.item),
            Some(&pumpkin_data::item::Item::FLOW_POTTERY_SHERD)
        );
        assert!(
            decoded
                .back
                .as_ref()
                .and_then(|s| s.get_data_component::<UnbreakableImpl>())
                .is_some()
        );

        let mut empty_bytes = Vec::new();
        PotDecorationsImpl::EMPTY
            .serialize(&mut empty_bytes)
            .unwrap();
        assert_eq!(empty_bytes, [0, 0, 0, 0]);
        assert_eq!(
            PotDecorationsImpl::deserialize(&mut empty_bytes.as_slice()).unwrap(),
            PotDecorationsImpl::EMPTY
        );
    }

    const VANILLA_CART_WITH_ADVENTURE_RANGE_AND_NBT: &[u8] =
        b"\x01\xc6\x07\x02\x00\x0e\x01\x01\x02\x88\x01\x01\x01\x05shape\
            \x00\x01\x0bnorth_south\x01\x0fascending_south\x01\x0a\x08\x00\x04Lock\x00\x03key\
            \x00\x00\x00\x29\x00\x00";

    #[test]
    fn java_adventure_and_swing_components_match_official_cart_slots() {
        use crate::{
            ClientPacket, codec::item_stack_seralizer::ItemStackSerializer,
            java::client::play::CInteractionSwing,
        };

        // Bare item stacks captured in the official 26.3 container_set_slot packets.
        let powered = b"\x01\xc6\x07\x02\x00\x0e\x01\x01\x02\x88\x01\x01\x01\x07powered\
            \x01\x04true\x00\x00\x00\x29\x02\x09";
        let max_only = b"\x01\xc6\x07\x02\x00\x0e\x01\x00\x01\x01\x07powered\x00\x00\x01\x04true\
            \x00\x00\x00\x29\x01\x09";

        let cases = [
            (powered.as_slice(), SwingAnimationType::Stab, 9),
            (
                VANILLA_CART_WITH_ADVENTURE_RANGE_AND_NBT,
                SwingAnimationType::None,
                0,
            ),
            (max_only.as_slice(), SwingAnimationType::Whack, 9),
        ];
        for (wire, animation_type, duration) in cases {
            let mut input = wire;
            let stack = ItemStackSerializer::read(&mut input).unwrap().to_stack();
            assert!(input.is_empty());
            assert_eq!(stack.get_item(), &pumpkin_data::item::Item::MINECART);
            let animation = *stack.get_data_component::<InteractAnimationImpl>().unwrap();
            assert_eq!(
                animation,
                InteractAnimationImpl {
                    animation_type,
                    duration,
                }
            );

            let mut tracker_packet = Vec::new();
            CInteractionSwing {
                entity_id: VarInt(1),
                off_hand: false,
                animation,
            }
            .write_packet_data(&mut tracker_packet, &JavaMinecraftVersion::V_26_3)
            .unwrap();
            assert_eq!(&tracker_packet[2..], &wire[wire.len() - 2..]);

            let mut held_slot = Vec::new();
            ItemStackSerializer::from(stack)
                .write(&mut held_slot)
                .unwrap();
            assert_eq!(held_slot, wire);
        }
    }

    #[test]
    fn java_adventure_matchers_preserve_vanilla_cart_slot_and_following_component() {
        use crate::codec::item_stack_seralizer::ItemStackSerializer;

        // Bare stacks captured in three official 26.3 container_set_slot packets.
        let exact = b"\x01\xc6\x07\x02\x00\x0e\x01\x01\x02\x88\x01\x01\x01\x07powered\
            \x01\x04true\x00\x01\x00\x0a\x03\x00\x06Marker\x00\x00\x00\x11\x00\x00\x29\x02\x09";
        let typed = b"\x01\xc6\x07\x02\x00\x0e\x01\x01\x02\x88\x01\x01\x01\x07powered\
            \x01\x04true\x00\x00\x01\x01\x04\x08\x00\x0b{Marker:18}\x29\x01\x05";
        let presence = b"\x01\xc6\x07\x02\x00\x0e\x01\x01\x02\x88\x01\x01\x01\x07powered\
            \x01\x04true\x00\x00\x01\x00\x04\x0a\x00\x29\x00\x07";

        let mut marker = NbtCompound::new();
        marker.put("Marker", NbtTag::Int(17));
        let cases = [
            (
                exact.as_slice(),
                "components",
                "minecraft:custom_data",
                NbtTag::Compound(marker),
                SwingAnimationType::Stab,
                9,
            ),
            (
                typed.as_slice(),
                "predicates",
                "minecraft:custom_data",
                NbtTag::String("{Marker:18}".into()),
                SwingAnimationType::Whack,
                5,
            ),
            (
                presence.as_slice(),
                "predicates",
                "minecraft:unbreakable",
                NbtTag::Compound(NbtCompound::new()),
                SwingAnimationType::None,
                7,
            ),
        ];
        for (wire, field, name, matcher, animation_type, duration) in cases {
            let mut input = wire;
            let stack = ItemStackSerializer::read(&mut input).unwrap().to_stack();
            assert!(input.is_empty());
            assert_eq!(stack.get_item(), &pumpkin_data::item::Item::MINECART);

            let predicate = stack.get_data_component::<CanPlaceOnImpl>().unwrap();
            let entry = predicate.predicate.extract_list().unwrap()[0]
                .extract_compound()
                .unwrap();
            assert_eq!(entry.get_compound(field).unwrap().get(name), Some(&matcher));
            assert_eq!(
                stack.get_data_component::<InteractAnimationImpl>(),
                Some(&InteractAnimationImpl {
                    animation_type,
                    duration,
                })
            );

            let mut encoded = Vec::new();
            ItemStackSerializer::from(stack)
                .write(&mut encoded)
                .unwrap();
            assert_eq!(encoded, wire);
        }
    }

    #[test]
    fn java_adventure_matchers_bound_recursive_exact_components() {
        let mut wire = Vec::new();
        for _ in 0..MAX_ADVENTURE_PREDICATE_DEPTH {
            // One block predicate, only an exact minecraft:can_place_on component.
            wire.extend_from_slice(&[1, 0, 0, 0, 1, DataComponent::CanPlaceOn.to_id()]);
        }
        wire.extend_from_slice(&[1, 0, 0, 0, 0, 0]);
        wire.extend(std::iter::repeat_n(
            0,
            usize::from(MAX_ADVENTURE_PREDICATE_DEPTH),
        ));
        assert!(matches!(
            CanPlaceOnImpl::deserialize(&mut wire.as_slice()),
            Err(ReadingError::TooLarge(_))
        ));

        let simple = CanPlaceOnImpl::deserialize(&mut [1, 0, 0, 0, 0, 0].as_slice());
        assert!(simple.is_ok());
    }

    #[test]
    fn java_creative_slot_retains_adventure_range_and_block_entity_nbt() {
        use crate::{ServerPacket, java::server::play::SSetCreativeSlot};

        // Creative slot updates add a length before each component payload.
        let wire = VANILLA_CART_WITH_ADVENTURE_RANGE_AND_NBT;
        let mut packet_bytes = b"\x00\x24\x01\xc6\x07\x01\x00\x0e\x3d".to_vec();
        packet_bytes.extend_from_slice(&wire[6..wire.len() - 3]);
        let mut bytes = packet_bytes.as_slice();
        let packet = SSetCreativeSlot::read(&mut bytes, &JavaMinecraftVersion::V_26_3).unwrap();
        assert!(bytes.is_empty());
        let stack = packet.clicked_item.to_stack();
        let predicate = stack.get_data_component::<CanPlaceOnImpl>().unwrap();
        let entry = predicate.predicate.extract_list().unwrap()[0]
            .extract_compound()
            .unwrap();
        let shape = entry
            .get_compound("state")
            .unwrap()
            .get_compound("shape")
            .unwrap();
        assert_eq!(shape.get_string("min"), Some("north_south"));
        assert_eq!(shape.get_string("max"), Some("ascending_south"));
        assert_eq!(
            entry.get_compound("nbt").unwrap().get_string("Lock"),
            Some("key")
        );
        assert!(!predicate.matches_block_state_without_entity(
            pumpkin_data::Block::POWERED_RAIL.default_state.id
        ));
    }

    #[test]
    fn item_components_26_3_wire_roundtrips() {
        fn roundtrip<T: DataComponentImpl + DataComponentCodec<T> + PartialEq + std::fmt::Debug>(
            val: &T,
        ) {
            let mut bytes = Vec::new();
            val.serialize(&mut bytes).unwrap();
            let mut slice = bytes.as_slice();
            let decoded = T::deserialize(&mut slice).unwrap();
            assert!(slice.is_empty(), "leftover bytes for {val:?}");
            assert_eq!(&decoded, val);
        }

        let shield = pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::SHIELD);
        roundtrip(shield.get_data_component::<BlocksAttacksImpl>().unwrap());

        let totem = pumpkin_data::item_stack::ItemStack::new(
            1,
            &pumpkin_data::item::Item::TOTEM_OF_UNDYING,
        );
        roundtrip(totem.get_data_component::<DeathProtectionImpl>().unwrap());

        let blaze =
            pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::BLAZE_POWDER);
        roundtrip(blaze.get_data_component::<BrewingFuelImpl>().unwrap());

        let egg = pumpkin_data::item_stack::ItemStack::new(
            1,
            &pumpkin_data::item::Item::CHICKEN_SPAWN_EGG,
        );
        roundtrip(egg.get_data_component::<EntityDataImpl>().unwrap());

        let sword =
            pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::NETHERITE_SWORD);
        roundtrip(sword.get_data_component::<DamageResistantImpl>().unwrap());

        let pumpkin =
            pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::CARVED_PUMPKIN);
        roundtrip(
            pumpkin
                .get_data_component::<AttributeModifiersImpl>()
                .unwrap(),
        );

        let stew =
            pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::MUSHROOM_STEW);
        roundtrip(stew.get_data_component::<UseRemainderImpl>().unwrap());

        roundtrip(&TooltipDisplayImpl {
            hide_tooltip: false,
            hidden_components: Cow::Owned(vec![DataComponent::Trim, DataComponent::Enchantments]),
        });

        let mut be_nbt = pumpkin_nbt::compound::NbtCompound::new();
        be_nbt.put_string("id", "minecraft:chest".to_string());
        be_nbt.put_string("Lock", "key".to_string());
        roundtrip(&BlockEntityDataImpl { nbt: be_nbt });

        let disc = pumpkin_data::item_stack::ItemStack::new(
            1,
            &pumpkin_data::item::Item::MUSIC_DISC_PRECIPICE,
        );
        roundtrip(disc.get_data_component::<JukeboxPlayableImpl>().unwrap());

        roundtrip(&WrittenBookContentImpl {
            title: "Chronicle".to_string(),
            author: "Alex".to_string(),
            generation: 2,
            pages: vec![pumpkin_util::text::TextComponent::text("Page 1")],
            resolved: true,
        });
    }
}

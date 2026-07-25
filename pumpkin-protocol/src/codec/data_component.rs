use std::borrow::Cow;

use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    AxolotlVariantImpl, BundleContentsImpl, CatCollarImpl, CatSoundVariantImpl, CatVariantImpl,
    ChickenSoundVariantImpl, ChickenVariantImpl, ConsumableImpl, ConsumeAnimation, ConsumeEffect,
    CowSoundVariantImpl, CowVariantImpl, CustomDataImpl, CustomNameImpl, DamageImpl,
    DataComponentImpl, EnchantmentsImpl, EquipmentSlot, EquippableImpl, FireworkExplosionImpl,
    FireworkExplosionShape, FireworksImpl, FoxVariantImpl, FrogVariantImpl, HorseVariantImpl,
    IDSet, IDSetContent, IdOr, ItemModelImpl, LlamaVariantImpl, MapIdImpl, MaxStackSizeImpl,
    MooshroomVariantImpl, PaintingVariantImpl, ParrotVariantImpl, PigSoundVariantImpl,
    PigVariantImpl, PotionContentsImpl, RabbitVariantImpl, SalmonSizeImpl, SheepColorImpl,
    ShulkerColorImpl, SoundEvent, StatusEffectInstance, StoredEnchantmentsImpl,
    TropicalFishBaseColorImpl, TropicalFishPatternColorImpl, TropicalFishPatternImpl,
    UnbreakableImpl, UseCooldownImpl, VillagerVariantImpl, WolfCollarImpl, WolfSoundVariantImpl,
    WolfVariantImpl, ZombieNautilusVariantImpl, get,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::{serializer::NbtWriteHelperJava, tag::NbtTag};

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
        let name = seq.get_str()?;
        Ok(Self {
            name: pumpkin_util::text::TextComponent::text(String::from(name)),
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

    fn deserialize(_seq: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        Err(ReadingError::Message(
            "CustomData raw component decoding is not supported; use the custom-data item-stack API".into(),
        ))
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
        _ => Err(WritingError::Message(format!(
            "{} not yet implemented",
            id.to_name()
        ))),
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

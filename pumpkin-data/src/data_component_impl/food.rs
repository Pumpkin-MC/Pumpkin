use crate::data_component_impl::basic::SoundEvent;
use crate::data_component_impl::{
    DataComponentImpl, IDSet, IdOr, default_impl, get_f32_hash, get_i32_hash, get_idor,
    get_idor_hash, get_idset_hash, get_str_hash, put_idor,
};
use crate::effect::StatusEffect;
use crate::sound::Sound;
use crc_fast::CrcAlgorithm::Crc32Iscsi;
use crc_fast::Digest;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct FoodImpl {
    pub nutrition: i32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
impl DataComponentImpl for FoodImpl {
    default_impl!(Food);
}
impl Hash for FoodImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nutrition.hash(state);
        unsafe { (*(&raw const self.saturation).cast::<u32>()).hash(state) };
        self.can_always_eat.hash(state);
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StatusEffectInstance {
    pub effect_id: Cow<'static, str>,
    pub amplifier: i32,
    pub duration: i32,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
}

impl StatusEffectInstance {
    pub fn read_data(nbt: &NbtTag) -> Option<Self> {
        let compound = nbt.extract_compound()?;
        let effect_id = Cow::Owned(compound.get_string("id")?.to_string());
        let amplifier = compound.get_int("amplifier")?;
        let duration = compound.get_int("duration")?;
        let ambient = compound.get_bool("ambient")?;
        let show_particles = compound.get_bool("show_particles")?;
        let show_icon = compound.get_bool("show_icon")?;
        Some(Self {
            effect_id,
            amplifier,
            duration,
            ambient,
            show_particles,
            show_icon,
        })
    }

    pub fn as_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("id", self.effect_id.to_string());
        compound.put_int("amplifier", self.amplifier);
        compound.put_int("duration", self.duration);
        compound.put_bool("ambient", self.ambient);
        compound.put_bool("show_particles", self.show_particles);
        compound.put_bool("show_icon", self.show_icon);
        NbtTag::Compound(compound)
    }

    pub fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&get_str_hash(self.effect_id.as_ref()).to_le_bytes());
        digest.update(&get_i32_hash(self.amplifier).to_le_bytes());
        digest.update(&get_i32_hash(self.duration).to_le_bytes());
        digest.update(&[self.ambient as u8]);
        digest.update(&[self.show_particles as u8]);
        digest.update(&[self.show_icon as u8]);
        digest.finalize() as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConsumeAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Spear,
    Crossbow,
    Spyglass,
    Horn,
    Brush,
}

impl ConsumeAnimation {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            ConsumeAnimation::None => "none",
            ConsumeAnimation::Eat => "eat",
            ConsumeAnimation::Drink => "drink",
            ConsumeAnimation::Block => "block",
            ConsumeAnimation::Bow => "bow",
            ConsumeAnimation::Spear => "spear",
            ConsumeAnimation::Crossbow => "crossbow",
            ConsumeAnimation::Spyglass => "spyglass",
            ConsumeAnimation::Horn => "horn",
            ConsumeAnimation::Brush => "brush",
        }
    }
}
impl TryFrom<i32> for ConsumeAnimation {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Eat),
            2 => Ok(Self::Drink),
            3 => Ok(Self::Block),
            4 => Ok(Self::Bow),
            5 => Ok(Self::Spear),
            6 => Ok(Self::Crossbow),
            7 => Ok(Self::Spyglass),
            8 => Ok(Self::Horn),
            9 => Ok(Self::Brush),
            _ => Err(()),
        }
    }
}
impl FromStr for ConsumeAnimation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "eat" => Ok(Self::Eat),
            "drink" => Ok(Self::Drink),
            "block" => Ok(Self::Block),
            "bow" => Ok(Self::Bow),
            "spear" => Ok(Self::Spear),
            "crossbow" => Ok(Self::Crossbow),
            "spyglass" => Ok(Self::Spyglass),
            "horn" => Ok(Self::Horn),
            "brush" => Ok(Self::Brush),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConsumeEffect {
    ApplyEffects((Cow<'static, [StatusEffectInstance]>, f32)),
    RemoveEffects(IDSet<StatusEffect>),
    ClearAllEffects,
    TeleportRandomly(f32),
    PlaySound(IdOr<SoundEvent>),
}
impl Hash for ConsumeEffect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_str().hash(state);
        match self {
            ConsumeEffect::ApplyEffects(tuple) => {
                tuple.0.hash(state);
                unsafe { (*(&raw const tuple.1).cast::<u32>()).hash(state) };
            }
            ConsumeEffect::RemoveEffects(status_effect_instances) => {
                status_effect_instances.hash(state)
            }
            ConsumeEffect::ClearAllEffects => (),
            ConsumeEffect::TeleportRandomly(dst) => unsafe {
                (*(&raw const dst).cast::<u32>()).hash(state)
            },
            ConsumeEffect::PlaySound(id_or) => id_or.hash(state),
        }
    }
}
impl ConsumeEffect {
    pub fn to_str(&self) -> &str {
        match self {
            ConsumeEffect::ApplyEffects(_) => "apply_effects",
            ConsumeEffect::RemoveEffects(_) => "remove_effects",
            ConsumeEffect::ClearAllEffects => "clear_all_effects",
            ConsumeEffect::TeleportRandomly(_) => "teleport_randomly",
            ConsumeEffect::PlaySound(_) => "play_sound",
        }
    }
    pub fn registry_id(&self) -> u8 {
        match self {
            ConsumeEffect::ApplyEffects(_) => 0,
            ConsumeEffect::RemoveEffects(_) => 1,
            ConsumeEffect::ClearAllEffects => 2,
            ConsumeEffect::TeleportRandomly(_) => 3,
            ConsumeEffect::PlaySound(_) => 4,
        }
    }
    pub fn read_data(nbt: &NbtTag) -> Option<Self> {
        let compound = nbt.extract_compound()?;
        let r#type = compound.get_string("type")?;
        match r#type {
            "remove_effects" => {
                let idset = IDSet::read(compound.get("effects")?)?;
                Some(Self::RemoveEffects(idset))
            }
            "clear_all_effects" => Some(Self::ClearAllEffects),
            "teleport_randomly" => {
                let dst = compound.get_float("diameter")?;
                Some(Self::TeleportRandomly(dst))
            }
            "play_sound" => {
                let sound = get_idor(compound, "sound", Sound::EntityGenericEat);
                Some(Self::PlaySound(sound))
            }
            "apply_effects" => {
                let probability = compound.get_float("probability")?;
                let effects_vec: Vec<StatusEffectInstance> = compound
                    .get_list("effects")?
                    .iter()
                    .filter_map(StatusEffectInstance::read_data)
                    .collect();
                let effects: Cow<'static, [StatusEffectInstance]> = Cow::Owned(effects_vec);
                Some(Self::ApplyEffects((effects, probability)))
            }
            _ => None,
        }
    }
    pub fn as_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("type", self.to_str().to_string());
        match self {
            ConsumeEffect::ApplyEffects(data) => {
                let nbt_arr = data.0.iter().map(|x| x.as_nbt()).collect();
                compound.put_list("effects", nbt_arr);
                compound.put_float("probability", data.1);
            }
            ConsumeEffect::RemoveEffects(idset) => idset.write(&mut compound, "effects"),
            ConsumeEffect::ClearAllEffects => (),
            ConsumeEffect::TeleportRandomly(dst) => compound.put_float("diameter", *dst),
            ConsumeEffect::PlaySound(id_or) => {
                put_idor(&mut compound, "sound", id_or);
            }
        }
        NbtTag::Compound(compound)
    }
    pub fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        match self {
            ConsumeEffect::ApplyEffects((effects, probability)) => {
                digest.update(&[1u8]);
                for effect in effects.iter() {
                    digest.update(&effect.get_hash().to_le_bytes());
                }
                digest.update(&[13u8]);
                digest.update(&get_f32_hash(*probability).to_le_bytes());
            }
            ConsumeEffect::RemoveEffects(idset) => {
                digest.update(&[2u8]);
                digest.update(&get_idset_hash(idset).to_le_bytes());
            }
            ConsumeEffect::ClearAllEffects => {
                digest.update(&[3u8]);
            }
            ConsumeEffect::TeleportRandomly(dst) => {
                digest.update(&[4u8]);
                digest.update(&get_f32_hash(*dst).to_le_bytes());
            }
            ConsumeEffect::PlaySound(id_or) => {
                digest.update(&[5u8]);
                digest.update(&get_idor_hash(id_or).to_le_bytes());
            }
        }
        digest.finalize() as i32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsumableImpl {
    pub consume_seconds: f32,
    pub animation: ConsumeAnimation,
    pub sound_event: IdOr<SoundEvent>,
    pub consume_particles: bool,
    pub effects: Cow<'static, [ConsumeEffect]>,
}
impl ConsumableImpl {
    pub const fn new(
        consume_seconds: f32,
        animation: ConsumeAnimation,
        sound_event: IdOr<SoundEvent>,
        consume_particles: bool,
        effects: Cow<'static, [ConsumeEffect]>,
    ) -> Self {
        Self {
            consume_seconds,
            animation,
            sound_event,
            consume_particles,
            effects,
        }
    }
    #[must_use]
    pub fn consume_ticks(&self) -> i32 {
        (self.consume_seconds * 20.0) as i32
    }
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let consume_seconds = compound.get_float("consume_seconds")?;
        let animation = compound
            .get_string("animation")?
            .parse::<ConsumeAnimation>()
            .ok()?;
        let sound_event = get_idor(compound, "sound", Sound::EntityGenericEat);
        let consume_particles = compound.get_bool("has_consume_particles").unwrap_or(false);
        let opt_list = compound.get_list("on_consume_effects");
        let effects: Cow<'static, [ConsumeEffect]> = if let Some(effect_list) = opt_list {
            effect_list
                .iter()
                .filter_map(ConsumeEffect::read_data)
                .collect()
        } else {
            Cow::Borrowed(&[])
        };
        Some(Self {
            consume_seconds,
            animation,
            sound_event,
            consume_particles,
            effects,
        })
    }
}
impl DataComponentImpl for ConsumableImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_float("consume_seconds", self.consume_seconds);
        compound.put_string("animation", self.animation.to_str().to_string());
        put_idor(&mut compound, "sound", &self.sound_event);
        compound.put_bool("has_consume_particles", self.consume_particles);
        let nbt_vec = self.effects.iter().map(|x| x.as_nbt()).collect();
        compound.put_list("on_consume_effects", nbt_vec);
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&[2u8]);
        digest.update(&get_f32_hash(self.consume_seconds).to_le_bytes());
        digest.update(&get_i32_hash(self.animation as i32).to_le_bytes());
        digest.update(&get_idor_hash(&self.sound_event).to_be_bytes());
        digest.update(&[self.consume_particles as u8]);
        for effect in self.effects.iter() {
            digest.update(&effect.get_hash().to_le_bytes());
        }
        digest.update(&[3u8]);
        digest.finalize() as i32
    }
    default_impl!(Consumable);
}
impl Hash for ConsumableImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*(&raw const self.consume_seconds).cast::<u32>()).hash(state) };
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UseEffectsImpl;
impl DataComponentImpl for UseEffectsImpl {
    default_impl!(UseEffects);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UseRemainderImpl;
impl DataComponentImpl for UseRemainderImpl {
    default_impl!(UseRemainder);
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UseCooldownImpl {
    pub seconds: f32,
    pub cooldown_group: Option<String>,
}
impl UseCooldownImpl {
    pub fn new(seconds: f32, cooldown_group: Option<String>) -> Self {
        Self {
            seconds,
            cooldown_group,
        }
    }
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let seconds = compound.get_float("seconds")?;
        let cooldown_group = compound.get_string("cooldown_group").map(|s| s.to_string());
        Some(Self {
            seconds,
            cooldown_group,
        })
    }
}
impl DataComponentImpl for UseCooldownImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_float("seconds", self.seconds);
        if let Some(group) = &self.cooldown_group {
            compound.put_string("cooldown_group", group.clone());
        }
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&get_f32_hash(self.seconds).to_le_bytes());
        if let Some(group) = &self.cooldown_group {
            digest.update(&get_str_hash(group).to_le_bytes());
        }
        digest.finalize() as i32
    }
    default_impl!(UseCooldown);
}
impl Eq for UseCooldownImpl {}
impl Hash for UseCooldownImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.seconds.to_bits().hash(state);
        self.cooldown_group.hash(state);
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotionContentsImpl {
    pub potion_id: Option<i32>,
    pub custom_color: Option<i32>,
    pub custom_effects: Vec<StatusEffectInstance>,
    pub custom_name: Option<String>,
}
impl PotionContentsImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let potion_id = if let Some(id) = compound.get_int("potion") {
            Some(id)
        } else if let Some(name) = compound.get_string("potion") {
            let name = name.strip_prefix("minecraft:").unwrap_or(name);
            crate::potion::Potion::from_name(name).map(|p| p.id as i32)
        } else {
            None
        };
        let custom_color = compound.get_int("custom_color");
        let custom_name = compound.get_string("custom_name").map(|s| s.to_string());
        let custom_effects = compound
            .get_list("custom_effects")
            .map(|list| {
                list.iter()
                    .filter_map(|item| {
                        let effect_tag = item.extract_compound()?;
                        let id: Cow<'static, str> =
                            Cow::Owned(effect_tag.get_string("id")?.to_string());
                        let amplifier = effect_tag
                            .get_int("amplifier")
                            .or_else(|| effect_tag.get_byte("amplifier").map(i32::from))
                            .unwrap_or(0);
                        let duration = effect_tag
                            .get_int("duration")
                            .or_else(|| effect_tag.get_byte("duration").map(i32::from))
                            .unwrap_or(0);
                        let ambient = effect_tag.get_bool("ambient").unwrap_or(false);
                        let show_particles = effect_tag.get_bool("show_particles").unwrap_or(true);
                        let show_icon = effect_tag.get_bool("show_icon").unwrap_or(true);
                        Some(StatusEffectInstance {
                            effect_id: id,
                            amplifier,
                            duration,
                            ambient,
                            show_particles,
                            show_icon,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Some(Self {
            potion_id,
            custom_color,
            custom_effects,
            custom_name,
        })
    }
}
impl DataComponentImpl for PotionContentsImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        if let Some(potion_id) = self.potion_id {
            compound.put_int("potion", potion_id);
        }
        if let Some(color) = self.custom_color {
            compound.put_int("custom_color", color);
        }
        if !self.custom_effects.is_empty() {
            let mut effects_list = Vec::new();
            for effect in &self.custom_effects {
                let mut effect_compound = NbtCompound::new();
                effect_compound.put_string("id", effect.effect_id.to_string());
                effect_compound.put_int("amplifier", effect.amplifier);
                effect_compound.put_int("duration", effect.duration);
                effect_compound.put_byte("ambient", effect.ambient as i8);
                effect_compound.put_byte("show_particles", effect.show_particles as i8);
                effect_compound.put_byte("show_icon", effect.show_icon as i8);
                effects_list.push(NbtTag::Compound(effect_compound));
            }
            compound.put("custom_effects", NbtTag::List(effects_list));
        }
        if let Some(name) = &self.custom_name {
            compound.put_string("custom_name", name.clone());
        }
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        if let Some(id) = self.potion_id {
            digest.update(&[1u8]);
            digest.update(&get_i32_hash(id).to_le_bytes());
        }
        if let Some(color) = self.custom_color {
            digest.update(&[2u8]);
            digest.update(&get_i32_hash(color).to_le_bytes());
        }
        if let Some(name) = &self.custom_name {
            digest.update(&[3u8]);
            digest.update(&get_str_hash(name).to_le_bytes());
        }
        if !self.custom_effects.is_empty() {
            digest.update(&[4u8]);
            for effect in &self.custom_effects {
                digest.update(&effect.get_hash().to_le_bytes());
            }
        }
        digest.finalize() as i32
    }
    default_impl!(PotionContents);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotionDurationScaleImpl;
impl DataComponentImpl for PotionDurationScaleImpl {
    default_impl!(PotionDurationScale);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SuspiciousStewEffectsImpl;
impl DataComponentImpl for SuspiciousStewEffectsImpl {
    default_impl!(SuspiciousStewEffects);
}

use crate::entity::NBTStorage;
use async_trait::async_trait;
use pumpkin_data::entity::EffectType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Effect {
    pub r#type: EffectType,
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub blend: bool,
}

#[async_trait]
impl NBTStorage for Effect {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put("id", self.r#type.to_minecraft_name());
        if self.amplifier > 0 {
            nbt.put("amplifier", NbtTag::Int(i32::from(self.amplifier)));
        }
        nbt.put("duration", NbtTag::Int(self.duration));
        if self.ambient {
            nbt.put("ambient", NbtTag::Byte(1));
        }
        if !self.show_particles {
            nbt.put("show_particles", NbtTag::Byte(0));
        }
        let show_icon: i8 = i8::from(self.show_icon);
        nbt.put("show_icon", NbtTag::Byte(show_icon));
    }

    async fn create_from_nbt(nbt: &mut NbtCompound) -> Option<Self> {
        let Some(effect_id) = nbt.get_string("id") else {
            log::warn!("Unable to read effect. Effect id is not present");
            return None;
        };
        let Some(effect_type) = EffectType::from_minecraft_name(effect_id) else {
            log::warn!("Unable to read effect. Unknown effect type: {effect_id}");
            return None;
        };
        let Some(show_icon) = nbt.get_byte("show_icon") else {
            log::warn!("Unable to read effect. Show icon is not present");
            return None;
        };
        let r#type = effect_type;
        let amplifier = nbt.get_int("amplifier").unwrap_or(0) as u8;
        let duration = nbt.get_int("duration").unwrap_or(0);
        let ambient = nbt.get_byte("ambient").unwrap_or(0) == 1;
        let show_particles = nbt.get_byte("show_particles").unwrap_or(1) == 1;
        let show_icon = show_icon == 1;
        Some(Self {
            r#type,
            duration,
            amplifier,
            ambient,
            show_particles,
            show_icon,
            blend: false,
        })
    }
}

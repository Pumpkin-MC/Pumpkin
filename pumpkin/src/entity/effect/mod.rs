use crate::entity::{NBTInitFuture, NBTStorage, NBTStorageInit, NbtFuture};
use pumpkin_data::effect::StatusEffect;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

impl NBTStorage for pumpkin_data::potion::Effect {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            nbt.put("id", self.effect_type.minecraft_name);
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
        })
    }
}

impl NBTStorageInit for pumpkin_data::potion::Effect {
    fn create_from_nbt<'a>(nbt: &'a mut NbtCompound) -> NBTInitFuture<'a, Self>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let Some(effect_id) = nbt.get_string("id") else {
                log::warn!("Unable to read effect. Effect id is not present");
                return None;
            };
            let Some(effect_type) = StatusEffect::from_minecraft_name(effect_id) else {
                log::warn!("Unable to read effect. Unknown effect type: {effect_id}");
                return None;
            };
            let Some(show_icon) = nbt.get_byte("show_icon") else {
                log::warn!("Unable to read effect. Show icon is not present");
                return None;
            };
            let amplifier = nbt.get_int("amplifier").unwrap_or(0) as u8;
            let duration = nbt.get_int("duration").unwrap_or(0);
            let ambient = nbt.get_byte("ambient").unwrap_or(0) == 1;
            let show_particles = nbt.get_byte("show_particles").unwrap_or(1) == 1;
            let show_icon = show_icon == 1;
            Some(Self {
                effect_type,
                duration,
                amplifier,
                ambient,
                show_particles,
                show_icon,
                blend: false,
            })
        })
    }
}

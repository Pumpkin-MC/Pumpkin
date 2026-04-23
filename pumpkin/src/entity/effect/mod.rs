use crate::entity::{NBTInitFuture, NBTStorage, NBTStorageInit, NbtFuture};
use pumpkin_data::effect::StatusEffect;
use pumpkin_nbt::pnbt::PNbtCompound;
use tracing::warn;

impl NBTStorage for pumpkin_data::potion::Effect {
    fn write_nbt<'a>(&'a self, nbt: &'a mut PNbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            nbt.put_string(self.effect_type.minecraft_name);
            nbt.put_u8(self.amplifier);
            nbt.put_int(self.duration);
            nbt.put_bool(self.ambient);
            nbt.put_bool(self.show_particles);
            nbt.put_bool(self.show_icon);
        })
    }
}

impl NBTStorageInit for pumpkin_data::potion::Effect {
    fn create_from_nbt<'a>(nbt: &'a mut PNbtCompound) -> NBTInitFuture<'a, Self>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let effect_id = nbt.get_string().ok()?;
            let effect_type = StatusEffect::from_minecraft_name(&effect_id).or_else(|| {
                warn!("Unable to read effect. Unknown effect type: {effect_id}");
                None
            })?;
            let amplifier = nbt.get_u8().unwrap_or(0);
            let duration = nbt.get_int().unwrap_or(0);
            let ambient = nbt.get_bool().unwrap_or(false);
            let show_particles = nbt.get_bool().unwrap_or(true);
            let show_icon = nbt.get_bool().unwrap_or(true);

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

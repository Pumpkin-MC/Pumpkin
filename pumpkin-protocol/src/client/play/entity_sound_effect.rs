use bytes::BufMut;
use pumpkin_data::{packet::clientbound::PLAY_SOUND_ENTITY, sound::SoundCategory};
use pumpkin_macros::packet;

use crate::{ClientPacket, IDOrSoundEvent, SoundEvent, VarInt, bytebuf::ByteBufMut};

#[packet(PLAY_SOUND_ENTITY)]
pub struct CEntitySoundEffect {
    sound_event: IDOrSoundEvent,
    sound_category: VarInt,
    entity_id: VarInt,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CEntitySoundEffect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sound_id: VarInt,
        sound_event: Option<SoundEvent>,
        sound_category: SoundCategory,
        entity_id: VarInt,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_event: IDOrSoundEvent {
                id: VarInt(sound_id.0 + 1),
                sound_event,
            },
            sound_category: VarInt(sound_category as i32),
            entity_id,
            volume,
            pitch,
            seed,
        }
    }
}

impl ClientPacket for CEntitySoundEffect {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_var_int(&self.sound_event.id);
        if self.sound_event.id.0 == 0 {
            if let Some(test) = &self.sound_event.sound_event {
                bytebuf.put_identifier(&test.sound_name);

                bytebuf.put_option(&test.range, |p, v| {
                    p.put_f32(*v);
                });
            }
        }
        bytebuf.put_var_int(&self.sound_category);
        bytebuf.put_var_int(&self.entity_id);
        bytebuf.put_f32(self.volume);
        bytebuf.put_f32(self.pitch);
        bytebuf.put_f64(self.seed);
    }
}

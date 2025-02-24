use bytes::BufMut;
use pumpkin_data::{packet::clientbound::PLAY_SOUND, sound::SoundCategory};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{ClientPacket, IDOrSoundEvent, SoundEvent, VarInt, bytebuf::ByteBufMut};

#[packet(PLAY_SOUND)]
pub struct CSoundEffect {
    sound_event: IDOrSoundEvent,
    sound_category: VarInt,
    position: Vector3<i32>,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CSoundEffect {
    pub fn new(
        sound_id: VarInt,
        sound_event: Option<SoundEvent>,
        sound_category: SoundCategory,
        position: &Vector3<f64>,
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
            position: Vector3::new(
                (position.x * 8.0) as i32,
                (position.y * 8.0) as i32,
                (position.z * 8.0) as i32,
            ),
            volume,
            pitch,
            seed,
        }
    }
}

impl ClientPacket for CSoundEffect {
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
        bytebuf.put_i32(self.position.x);
        bytebuf.put_i32(self.position.y);
        bytebuf.put_i32(self.position.z);
        bytebuf.put_f32(self.volume);
        bytebuf.put_f32(self.pitch);
        bytebuf.put_f64(self.seed);
    }
}

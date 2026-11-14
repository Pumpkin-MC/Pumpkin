use pumpkin_data::particle::Particle;

use crate::{
    VarInt,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, ReadingError},
};

/// 26.3 option payload that follows a particle type id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleOptionsLayout {
    None,
    BlockState,
    Color,
    Dust,
    DustColorTransition,
    Spell,
    Power,
    Float,
    VarInt,
    Geyser,
    GeyserBase,
    Trail,
    Vibration,
    Item,
}

impl ParticleOptionsLayout {
    #[must_use]
    pub const fn of(particle: Particle) -> Self {
        use Particle as P;
        match particle {
            P::Block | P::BlockMarker | P::FallingDust | P::DustPillar | P::BlockCrumble => {
                Self::BlockState
            }
            P::EntityEffect | P::TintedLeaves | P::Flash => Self::Color,
            P::Dust => Self::Dust,
            P::DustColorTransition => Self::DustColorTransition,
            P::Effect | P::InstantEffect => Self::Spell,
            P::DragonBreath => Self::Power,
            P::SculkCharge => Self::Float,
            P::Shriek => Self::VarInt,
            P::Geyser | P::GeyserPlume => Self::Geyser,
            P::GeyserBase | P::GeyserPoof => Self::GeyserBase,
            P::Trail => Self::Trail,
            P::Vibration => Self::Vibration,
            P::Item => Self::Item,
            _ => Self::None,
        }
    }

    pub fn skip(self, bytebuf: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
        match self {
            Self::None => {}
            Self::BlockState | Self::VarInt => {
                bytebuf.get_var_int()?;
            }
            Self::Color | Self::Geyser => {
                bytebuf.get_i32_be()?;
            }
            Self::Dust | Self::Spell | Self::GeyserBase => {
                bytebuf.get_i32_be()?;
                bytebuf.get_f32_be()?;
            }
            Self::DustColorTransition => {
                bytebuf.get_i32_be()?;
                bytebuf.get_i32_be()?;
                bytebuf.get_f32_be()?;
            }
            Self::Power | Self::Float => {
                bytebuf.get_f32_be()?;
            }
            Self::Trail => {
                bytebuf.get_f64_be()?;
                bytebuf.get_f64_be()?;
                bytebuf.get_f64_be()?;
                bytebuf.get_i32_be()?;
                bytebuf.get_var_int()?;
            }
            Self::Vibration => {
                match bytebuf.get_var_int()?.0 {
                    0 => {
                        bytebuf.get_block_pos()?;
                    }
                    1 => {
                        bytebuf.get_var_int()?;
                        bytebuf.get_f32_be()?;
                    }
                    source => {
                        return Err(ReadingError::Message(format!(
                            "Unknown vibration position source {source}"
                        )));
                    }
                }
                bytebuf.get_var_int()?;
            }
            Self::Item => {
                ItemStackSerializer::read_template(bytebuf)?;
            }
        }
        Ok(())
    }

    pub fn skip_from_id(
        bytebuf: &mut impl NetworkReadExt,
        particle_id: VarInt,
    ) -> Result<(), ReadingError> {
        let particle = u16::try_from(particle_id.0)
            .ok()
            .and_then(Particle::from_id)
            .ok_or_else(|| {
                ReadingError::Message(format!("Unknown particle id {}", particle_id.0))
            })?;
        Self::of(particle).skip(bytebuf)
    }

    pub fn read_bytes<'a>(
        bytebuf: &mut &'a [u8],
        particle_id: VarInt,
    ) -> Result<&'a [u8], ReadingError> {
        let remaining_before = *bytebuf;
        Self::skip_from_id(bytebuf, particle_id)?;
        Ok(&remaining_before[..remaining_before.len() - bytebuf.len()])
    }
}
